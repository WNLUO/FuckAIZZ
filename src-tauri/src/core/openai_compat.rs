use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    cost::{apply_billing_multiplier, estimate_cost, estimate_tokens_from_text},
    models::{ProbeInput, ProviderModel, RequestOutcome, UsageSource},
};

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    model: Option<String>,
    choices: Option<Vec<Choice>>,
    usage: Option<Usage>,
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionStreamResponse {
    model: Option<String>,
    choices: Option<Vec<StreamChoice>>,
    usage: Option<Usage>,
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<Message>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<Message>,
    message: Option<Message>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
    prompt_tokens_details: Option<TokenDetails>,
    input_tokens_details: Option<TokenDetails>,
    cache_read_input_tokens: Option<u32>,
    prompt_cache_hit_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct TokenDetails {
    cached_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelItem>,
}

#[derive(Debug, Deserialize)]
struct ModelItem {
    id: String,
    owned_by: Option<String>,
}

pub async fn request_chat_completion(
    client: &Client,
    base_url: &str,
    input: &ProbeInput,
) -> Result<RequestOutcome, String> {
    let attempts = [
        RequestMode {
            stream: Some(true),
            include_stream_usage: true,
        },
        RequestMode {
            stream: Some(true),
            include_stream_usage: false,
        },
        RequestMode {
            stream: None,
            include_stream_usage: false,
        },
    ];
    let mut next_attempt = 0;

    while next_attempt < attempts.len() {
        let mode = attempts[next_attempt];
        let started = Instant::now();
        let response = send_chat_request(client, base_url, input, mode).await?;
        let status = response.status();

        if status.is_success() {
            return if mode.stream == Some(true) {
                read_streaming_or_json_response(response, started, input).await
            } else {
                read_json_response(response, started, input).await
            };
        }

        let text = response
            .text()
            .await
            .map_err(|err| format!("读取响应失败：{err}"))?;

        if next_attempt == 0 && can_retry_without_stream_usage(status, &text) {
            next_attempt = 1;
            continue;
        }
        if next_attempt < 2 && can_retry_without_stream(status, &text) {
            next_attempt = 2;
            continue;
        }

        return Err(format!(
            "接口返回 HTTP {}：{}",
            status.as_u16(),
            summarize(&text, 300)
        ));
    }

    Err("请求失败：没有可用的 OpenAI 兼容请求模式".to_string())
}

#[derive(Debug, Clone, Copy)]
struct RequestMode {
    stream: Option<bool>,
    include_stream_usage: bool,
}

async fn send_chat_request(
    client: &Client,
    base_url: &str,
    input: &ProbeInput,
    mode: RequestMode,
) -> Result<Response, String> {
    let body = ChatCompletionRequest {
        model: input.model.trim(),
        messages: vec![ChatMessage {
            role: "user",
            content: input.prompt.trim(),
        }],
        max_tokens: input.max_tokens,
        temperature: 0.2,
        stream: mode.stream,
        stream_options: mode.include_stream_usage.then_some(StreamOptions {
            include_usage: true,
        }),
    };

    client
        .post(chat_completions_url(base_url))
        .bearer_auth(input.api_key.trim())
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("请求失败：{err}"))
}

async fn read_json_response(
    response: Response,
    started: Instant,
    input: &ProbeInput,
) -> Result<RequestOutcome, String> {
    let text = response
        .text()
        .await
        .map_err(|err| format!("读取响应失败：{err}"))?;
    outcome_from_json_text(&text, started, input)
}

async fn read_streaming_or_json_response(
    response: Response,
    started: Instant,
    input: &ProbeInput,
) -> Result<RequestOutcome, String> {
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut full_text = String::new();
    let mut response_text = String::new();
    let mut model_reported = None;
    let mut usage = None;
    let mut first_token_latency_ms = None;
    let mut saw_sse_event = false;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|err| format!("读取流式响应失败：{err}"))?;
        let text = String::from_utf8_lossy(&chunk);
        full_text.push_str(&text);
        buffer.push_str(&text.replace("\r\n", "\n"));

        while let Some(event_end) = buffer.find("\n\n") {
            let event = buffer[..event_end].to_string();
            buffer.drain(..event_end + 2);
            if apply_stream_event(
                &event,
                started,
                &mut response_text,
                &mut model_reported,
                &mut usage,
                &mut first_token_latency_ms,
            )? {
                saw_sse_event = true;
            }
        }
    }

    if !buffer.trim().is_empty()
        && apply_stream_event(
            &buffer,
            started,
            &mut response_text,
            &mut model_reported,
            &mut usage,
            &mut first_token_latency_ms,
        )?
    {
        saw_sse_event = true;
    }

    if !saw_sse_event {
        return outcome_from_json_text(&full_text, started, input);
    }

    Ok(outcome_from_parts(
        model_reported,
        usage,
        response_text,
        started,
        first_token_latency_ms,
        input,
    ))
}

fn apply_stream_event(
    event: &str,
    started: Instant,
    response_text: &mut String,
    model_reported: &mut Option<String>,
    usage: &mut Option<Usage>,
    first_token_latency_ms: &mut Option<u128>,
) -> Result<bool, String> {
    let payload = event
        .lines()
        .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .collect::<Vec<_>>()
        .join("\n");

    if payload.trim().is_empty() {
        return Ok(false);
    }
    if payload.trim() == "[DONE]" {
        return Ok(true);
    }

    let parsed: ChatCompletionStreamResponse = serde_json::from_str(&payload)
        .map_err(|err| format!("流式响应不是 OpenAI 兼容 JSON：{err}"))?;
    if let Some(error) = parsed.error {
        return Err(format!(
            "流式接口返回错误：{}",
            summarize(&error.to_string(), 300)
        ));
    }

    if model_reported.is_none() {
        *model_reported = parsed.model;
    }
    if parsed.usage.is_some() {
        *usage = parsed.usage;
    }

    for choice in parsed.choices.unwrap_or_default() {
        let content = choice
            .text
            .or_else(|| {
                choice.delta.and_then(|message| {
                    message.content.map(|value| content_value_to_string(&value))
                })
            })
            .or_else(|| {
                choice.message.and_then(|message| {
                    message.content.map(|value| content_value_to_string(&value))
                })
            })
            .unwrap_or_default();

        if !content.is_empty() {
            if first_token_latency_ms.is_none() {
                *first_token_latency_ms = Some(started.elapsed().as_millis());
            }
            response_text.push_str(&content);
        }
    }

    Ok(true)
}

fn outcome_from_json_text(
    text: &str,
    started: Instant,
    input: &ProbeInput,
) -> Result<RequestOutcome, String> {
    let parsed: ChatCompletionResponse =
        serde_json::from_str(text).map_err(|err| format!("响应不是 OpenAI 兼容 JSON：{err}"))?;
    if let Some(error) = parsed.error {
        return Err(format!(
            "接口返回错误：{}",
            summarize(&error.to_string(), 300)
        ));
    }
    let response_text = extract_response_text(&parsed);
    Ok(outcome_from_parts(
        parsed.model,
        parsed.usage,
        response_text,
        started,
        None,
        input,
    ))
}

fn outcome_from_parts(
    model_reported: Option<String>,
    usage: Option<Usage>,
    response_text: String,
    started: Instant,
    first_token_latency_ms: Option<u128>,
    input: &ProbeInput,
) -> RequestOutcome {
    let (usage_source, prompt_tokens, cached_prompt_tokens, completion_tokens, total_tokens) =
        if let Some(usage) = usage {
            let prompt_tokens = usage
                .prompt_tokens
                .unwrap_or_else(|| estimate_tokens_from_text(input.prompt.trim()));
            let cached_prompt_tokens = cached_tokens_from_usage(&usage).min(prompt_tokens);
            let completion_tokens = usage
                .completion_tokens
                .unwrap_or_else(|| estimate_tokens_from_text(&response_text));
            let total_tokens = usage
                .total_tokens
                .unwrap_or(prompt_tokens + completion_tokens);
            (
                UsageSource::Api,
                prompt_tokens,
                cached_prompt_tokens,
                completion_tokens,
                total_tokens,
            )
        } else {
            let prompt_tokens = estimate_tokens_from_text(input.prompt.trim());
            let completion_tokens = estimate_tokens_from_text(&response_text);
            (
                UsageSource::Estimated,
                prompt_tokens,
                0,
                completion_tokens,
                prompt_tokens + completion_tokens,
            )
        };
    let raw_estimated_cost = estimate_cost(
        prompt_tokens,
        cached_prompt_tokens,
        completion_tokens,
        input.input_price_per_1m,
        input.cached_input_price_per_1m,
        input.output_price_per_1m,
    );

    RequestOutcome {
        model_reported,
        usage_source,
        prompt_tokens,
        cached_prompt_tokens,
        completion_tokens,
        total_tokens,
        raw_estimated_cost,
        estimated_cost: apply_billing_multiplier(raw_estimated_cost, input.billing_multiplier),
        latency_ms: started.elapsed().as_millis(),
        first_token_latency_ms,
        response_summary: summarize(&response_text, 200),
    }
}

fn can_retry_without_stream_usage(status: StatusCode, text: &str) -> bool {
    matches!(status.as_u16(), 400 | 422)
        && text
            .to_lowercase()
            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .any(|word| matches!(word, "stream_options" | "include_usage"))
}

fn can_retry_without_stream(status: StatusCode, text: &str) -> bool {
    matches!(status.as_u16(), 400 | 422) && text.to_lowercase().contains("stream")
}

fn cached_tokens_from_usage(usage: &Usage) -> u32 {
    [
        usage
            .prompt_tokens_details
            .as_ref()
            .and_then(|details| details.cached_tokens),
        usage
            .input_tokens_details
            .as_ref()
            .and_then(|details| details.cached_tokens),
        usage.cache_read_input_tokens,
        usage.prompt_cache_hit_tokens,
    ]
    .into_iter()
    .flatten()
    .max()
    .unwrap_or(0)
}

pub async fn list_models(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ProviderModel>, String> {
    let response = client
        .get(models_url(base_url))
        .bearer_auth(api_key.trim())
        .send()
        .await
        .map_err(|err| format!("获取模型列表失败：{err}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| format!("读取模型列表失败：{err}"))?;

    if !status.is_success() {
        return Err(format!(
            "模型列表接口返回 HTTP {}：{}",
            status.as_u16(),
            summarize(&text, 300)
        ));
    }

    let parsed: ModelsResponse = serde_json::from_str(&text)
        .map_err(|err| format!("模型列表不是 OpenAI 兼容 JSON：{err}"))?;
    let mut models = parsed
        .data
        .into_iter()
        .filter(|item| !item.id.trim().is_empty())
        .map(|item| ProviderModel {
            id: item.id,
            owned_by: item.owned_by,
        })
        .collect::<Vec<_>>();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models.dedup_by(|a, b| a.id == b.id);
    Ok(models)
}

pub async fn generate_prompt_with_model(
    client: &Client,
    base_url: &str,
    input: &ProbeInput,
) -> Result<String, String> {
    let mut generator_input = input.clone();
    generator_input.max_tokens = 260;
    generator_input.prompt = format!(
        "请生成一个适合测试 AI 中转平台 token 计费的中文用户 prompt。要求：1. 主题具体且自然；2. 不包含敏感信息；3. 不要提到计费、额度、检测、中转平台；4. 只输出 prompt 本文；5. 120 到 180 个中文字符；6. 加入一个随机细节避免重复。随机种子：{}",
        uuid::Uuid::new_v4()
    );
    let outcome = request_chat_completion(client, base_url, &generator_input).await?;
    let prompt = outcome
        .response_summary
        .trim()
        .trim_matches('"')
        .to_string();
    if prompt.is_empty() {
        Err("生成的 prompt 为空".to_string())
    } else {
        Ok(prompt)
    }
}

pub fn build_http_client(timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|err| format!("创建 HTTP client 失败：{err}"))
}

fn chat_completions_url(base_url: &str) -> String {
    endpoint_url(base_url, "chat/completions")
}

fn models_url(base_url: &str) -> String {
    endpoint_url(base_url, "models")
}

fn endpoint_url(base_url: &str, endpoint: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let endpoint_suffix = format!("/{endpoint}");

    if base.ends_with(&endpoint_suffix) {
        return base.to_string();
    }

    if endpoint == "models" {
        if let Some(prefix) = base.strip_suffix("/chat/completions") {
            return format!("{prefix}/models");
        }
    }

    if endpoint == "chat/completions" {
        if let Some(prefix) = base.strip_suffix("/models") {
            return format!("{prefix}/chat/completions");
        }
    }

    if base.ends_with("/v1") || ends_with_api_version(base) {
        return format!("{base}/{endpoint}");
    }

    format!("{base}/v1/{endpoint}")
}

fn ends_with_api_version(value: &str) -> bool {
    value.rsplit('/').next().is_some_and(|segment| {
        segment.len() > 1
            && segment.starts_with('v')
            && segment[1..].chars().any(|ch| ch.is_ascii_digit())
    })
}

fn extract_response_text(response: &ChatCompletionResponse) -> String {
    response
        .choices
        .as_ref()
        .and_then(|choices| choices.first())
        .map(|choice| {
            if let Some(text) = &choice.text {
                return text.clone();
            }
            choice
                .message
                .as_ref()
                .and_then(|message| message.content.as_ref())
                .map(content_value_to_string)
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

fn content_value_to_string(value: &Value) -> String {
    match value {
        Value::String(content) => content.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn summarize(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    let mut summary: String = trimmed.chars().take(max_chars).collect();
    if trimmed.chars().count() > max_chars {
        summary.push_str("...");
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_chat_completions_path() {
        assert_eq!(
            chat_completions_url("https://api.example.com"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://api.example.com/v1"),
            "https://api.example.com/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://relay.example.com/proxy/openai/v1"),
            "https://relay.example.com/proxy/openai/v1/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://relay.example.com/openai/chat/completions"),
            "https://relay.example.com/openai/chat/completions"
        );
        assert_eq!(
            chat_completions_url("https://relay.example.com/openai/models"),
            "https://relay.example.com/openai/chat/completions"
        );
    }

    #[test]
    fn appends_models_path() {
        assert_eq!(
            models_url("https://api.example.com"),
            "https://api.example.com/v1/models"
        );
        assert_eq!(
            models_url("https://api.example.com/v1"),
            "https://api.example.com/v1/models"
        );
        assert_eq!(
            models_url("https://relay.example.com/proxy/openai/v1"),
            "https://relay.example.com/proxy/openai/v1/models"
        );
        assert_eq!(
            models_url("https://relay.example.com/openai/models"),
            "https://relay.example.com/openai/models"
        );
        assert_eq!(
            models_url("https://relay.example.com/openai/chat/completions"),
            "https://relay.example.com/openai/models"
        );
    }

    #[test]
    fn handles_custom_api_version_path() {
        assert_eq!(
            chat_completions_url("https://api.example.com/custom/v1beta"),
            "https://api.example.com/custom/v1beta/chat/completions"
        );
        assert_eq!(
            models_url("https://api.example.com/custom/v1beta"),
            "https://api.example.com/custom/v1beta/models"
        );
    }
}
