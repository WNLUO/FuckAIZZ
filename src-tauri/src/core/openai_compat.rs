use std::time::{Duration, Instant};

use reqwest::Client;
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
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    model: Option<String>,
    choices: Option<Vec<Choice>>,
    usage: Option<Usage>,
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
    let started = Instant::now();
    let url = chat_completions_url(base_url);
    let body = ChatCompletionRequest {
        model: input.model.trim(),
        messages: vec![ChatMessage {
            role: "user",
            content: input.prompt.trim(),
        }],
        max_tokens: input.max_tokens,
        temperature: 0.2,
    };

    let response = client
        .post(url)
        .bearer_auth(input.api_key.trim())
        .json(&body)
        .send()
        .await
        .map_err(|err| format!("请求失败：{err}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|err| format!("读取响应失败：{err}"))?;

    if !status.is_success() {
        return Err(format!(
            "接口返回 HTTP {}：{}",
            status.as_u16(),
            summarize(&text, 300)
        ));
    }

    let parsed: ChatCompletionResponse =
        serde_json::from_str(&text).map_err(|err| format!("响应不是 OpenAI 兼容 JSON：{err}"))?;
    let response_text = extract_response_text(&parsed);
    let usage = parsed.usage;
    let (usage_source, prompt_tokens, cached_prompt_tokens, completion_tokens, total_tokens) = if let Some(usage) = usage
    {
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

    Ok(RequestOutcome {
        model_reported: parsed.model,
        usage_source,
        prompt_tokens,
        cached_prompt_tokens,
        completion_tokens,
        total_tokens,
        raw_estimated_cost,
        estimated_cost: apply_billing_multiplier(raw_estimated_cost, input.billing_multiplier),
        latency_ms: started.elapsed().as_millis(),
        response_summary: summarize(&response_text, 200),
    })
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
    value
        .rsplit('/')
        .next()
        .is_some_and(|segment| segment.len() > 1 && segment.starts_with('v') && segment[1..].chars().any(|ch| ch.is_ascii_digit()))
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
