use crate::core::{
    models::{PricingCatalogResult, ProbeInput, PromptGenerationResult, ProviderModel},
    openai_compat::{build_http_client, generate_prompt_with_model, list_models},
    pricing_catalog::fetch_pricing_catalog,
    safety::{validate_base_url, validate_probe_input},
};

#[tauri::command]
pub async fn list_provider_models(input: ProbeInput) -> Result<Vec<ProviderModel>, String> {
    validate_probe_input(&input)?;
    let normalized_base_url = validate_base_url(&input.base_url).await?;
    let client = build_http_client(input.timeout_secs)?;
    list_models(&client, &normalized_base_url, &input.api_key).await
}

#[tauri::command]
pub async fn generate_test_prompt(input: ProbeInput) -> Result<PromptGenerationResult, String> {
    validate_probe_input(&input)?;
    let normalized_base_url = validate_base_url(&input.base_url).await?;
    let client = build_http_client(input.timeout_secs)?;
    Ok(PromptGenerationResult {
        prompt: generate_prompt_with_model(&client, &normalized_base_url, &input).await?,
    })
}

#[tauri::command]
pub async fn refresh_pricing_catalog() -> Result<PricingCatalogResult, String> {
    fetch_pricing_catalog().await
}
