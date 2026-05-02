use chrono::Utc;
use reqwest::Client;
use serde_json::Value;

use super::models::{PricingCatalogModel, PricingCatalogResult};

pub const LITELLM_PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

pub async fn fetch_pricing_catalog() -> Result<PricingCatalogResult, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|err| format!("创建价格库 HTTP client 失败：{err}"))?;

    let value: Value = client
        .get(LITELLM_PRICING_URL)
        .send()
        .await
        .map_err(|err| format!("获取在线价格库失败：{err}"))?
        .error_for_status()
        .map_err(|err| format!("在线价格库返回错误：{err}"))?
        .json()
        .await
        .map_err(|err| format!("解析在线价格库失败：{err}"))?;

    let object = value
        .as_object()
        .ok_or_else(|| "在线价格库格式不是对象".to_string())?;
    let mut models = Vec::new();

    for (model_id, item) in object {
        let Some(input_cost) = item.get("input_cost_per_token").and_then(Value::as_f64) else {
            continue;
        };
        let Some(output_cost) = item.get("output_cost_per_token").and_then(Value::as_f64) else {
            continue;
        };
        if input_cost < 0.0 || output_cost < 0.0 {
            continue;
        }

        let mode = item.get("mode").and_then(Value::as_str).unwrap_or_default();
        if !matches!(mode, "chat" | "completion" | "responses" | "") {
            continue;
        }

        let provider = item
            .get("litellm_provider")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let display_name = item
            .get("display_name")
            .and_then(Value::as_str)
            .unwrap_or(model_id)
            .to_string();

        models.push(PricingCatalogModel {
            id: model_id.to_string(),
            provider,
            display_name,
            input_usd_per_1m: input_cost * 1_000_000.0,
            cached_input_usd_per_1m: item
                .get("cache_read_input_token_cost")
                .or_else(|| item.get("cached_input_cost_per_token"))
                .and_then(Value::as_f64)
                .filter(|value| value.is_finite() && *value >= 0.0)
                .map(|value| value * 1_000_000.0),
            output_usd_per_1m: output_cost * 1_000_000.0,
            source: Some("LiteLLM model_prices_and_context_window.json".to_string()),
            note: item
                .get("notes")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        });
    }

    models.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.id.to_lowercase().cmp(&b.id.to_lowercase()))
    });
    models.dedup_by(|a, b| a.id == b.id);

    Ok(PricingCatalogResult {
        source_url: LITELLM_PRICING_URL.to_string(),
        updated_at: Utc::now().to_rfc3339(),
        models,
    })
}
