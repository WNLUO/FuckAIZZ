use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInput {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub input_price_per_1m: f64,
    pub cached_input_price_per_1m: f64,
    pub output_price_per_1m: f64,
    pub billing_multiplier: f64,
    pub max_tokens: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTestRunInput {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub prompt: String,
    pub input_price_per_1m: f64,
    pub cached_input_price_per_1m: f64,
    pub output_price_per_1m: f64,
    pub billing_multiplier: f64,
    pub max_tokens: u32,
    pub timeout_secs: u64,
    pub current_usd: f64,
    pub target_usd: f64,
    pub max_requests: u32,
    pub balance_before: f64,
}

impl From<StartTestRunInput> for ProbeInput {
    fn from(value: StartTestRunInput) -> Self {
        Self {
            name: value.name,
            base_url: value.base_url,
            api_key: value.api_key,
            model: value.model,
            prompt: value.prompt,
            input_price_per_1m: value.input_price_per_1m,
            cached_input_price_per_1m: value.cached_input_price_per_1m,
            output_price_per_1m: value.output_price_per_1m,
            billing_multiplier: value.billing_multiplier,
            max_tokens: value.max_tokens,
            timeout_secs: value.timeout_secs,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageSource {
    Api,
    Estimated,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestRunStatus {
    Running,
    Completed,
    Stopped,
    PausedOnFailures,
    StoppedOnBudgetGuard,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModel {
    pub id: String,
    pub owned_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingCatalogModel {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub input_usd_per_1m: f64,
    pub cached_input_usd_per_1m: Option<f64>,
    pub output_usd_per_1m: f64,
    pub source: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingCatalogResult {
    pub source_url: String,
    pub updated_at: String,
    pub models: Vec<PricingCatalogModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGenerationResult {
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub request_index: u32,
    pub status: RequestStatus,
    pub latency_ms: u128,
    pub prompt_tokens: u32,
    #[serde(default)]
    pub cached_prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub raw_estimated_cost: f64,
    pub estimated_cost: f64,
    pub response_summary: String,
    pub error_message: Option<String>,
    pub usage_source: UsageSource,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestProgress {
    pub run_id: String,
    pub request_count: u32,
    pub success_count: u32,
    pub failed_count: u32,
    pub estimated_cost: f64,
    pub total_tokens: u32,
    pub status: TestRunStatus,
    pub latest_log: Option<RequestLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunReport {
    pub id: String,
    pub app_version: String,
    pub provider_name: String,
    pub base_url: String,
    pub model_requested: String,
    pub model_reported: Option<String>,
    #[serde(default)]
    pub current_usd: f64,
    pub target_usd: f64,
    pub input_price_per_1m: f64,
    #[serde(default)]
    pub cached_input_price_per_1m: f64,
    pub output_price_per_1m: f64,
    #[serde(default = "default_billing_multiplier")]
    pub billing_multiplier: f64,
    pub balance_before: f64,
    pub balance_after: Option<f64>,
    pub estimated_cost: f64,
    pub actual_cost: Option<f64>,
    pub diff_cost: Option<f64>,
    pub diff_ratio: Option<f64>,
    pub status: TestRunStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub usage_source: UsageSource,
    pub request_logs: Vec<RequestLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunSummary {
    pub id: String,
    pub provider_name: String,
    pub base_url: String,
    pub model_requested: String,
    pub estimated_cost: f64,
    pub actual_cost: Option<f64>,
    pub diff_ratio: Option<f64>,
    pub status: TestRunStatus,
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl From<&TestRunReport> for TestRunSummary {
    fn from(report: &TestRunReport) -> Self {
        Self {
            id: report.id.clone(),
            provider_name: report.provider_name.clone(),
            base_url: report.base_url.clone(),
            model_requested: report.model_requested.clone(),
            estimated_cost: report.estimated_cost,
            actual_cost: report.actual_cost,
            diff_ratio: report.diff_ratio,
            status: report.status,
            created_at: report.created_at.clone(),
            completed_at: report.completed_at.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RequestOutcome {
    pub model_reported: Option<String>,
    pub usage_source: UsageSource,
    pub prompt_tokens: u32,
    pub cached_prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub raw_estimated_cost: f64,
    pub estimated_cost: f64,
    pub latency_ms: u128,
    pub response_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub path: String,
    pub content: String,
}

fn default_billing_multiplier() -> f64 {
    1.0
}
