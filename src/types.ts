export type UsageSource = "api" | "estimated";

export interface ProbeInput {
  name: string;
  base_url: string;
  api_key: string;
  model: string;
  prompt: string;
  input_price_per_1m: number;
  cached_input_price_per_1m: number;
  output_price_per_1m: number;
  billing_multiplier: number;
  max_tokens: number;
  timeout_secs: number;
}

export interface StartTestRunInput extends ProbeInput {
  current_usd: number;
  target_usd: number;
  max_requests: number;
  concurrency: number;
  balance_before: number;
}

export interface ProviderModel {
  id: string;
  owned_by?: string | null;
}

export interface PricingCatalogModel {
  id: string;
  provider: string;
  display_name: string;
  input_usd_per_1m: number;
  cached_input_usd_per_1m?: number | null;
  output_usd_per_1m: number;
  source?: string | null;
  note?: string | null;
}

export interface PricingCatalogResult {
  source_url: string;
  updated_at: string;
  models: PricingCatalogModel[];
}

export interface PromptGenerationResult {
  prompt: string;
}

export interface RequestLog {
  request_index: number;
  status: "success" | "error";
  latency_ms: number;
  first_token_latency_ms?: number | null;
  prompt_tokens: number;
  cached_prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  raw_estimated_cost: number;
  estimated_cost: number;
  response_summary: string;
  error_message?: string | null;
  usage_source: UsageSource;
  created_at: string;
}

export interface TestProgress {
  run_id: string;
  request_count: number;
  success_count: number;
  failed_count: number;
  estimated_cost: number;
  total_tokens: number;
  status: TestRunStatus;
  latest_log?: RequestLog | null;
}

export type TestRunStatus =
  | "running"
  | "completed"
  | "stopped"
  | "paused_on_failures"
  | "stopped_on_budget_guard"
  | "failed";

export interface TestRunReport {
  id: string;
  app_version: string;
  provider_name: string;
  base_url: string;
  model_requested: string;
  model_reported?: string | null;
  current_usd: number;
  target_usd: number;
  input_price_per_1m: number;
  cached_input_price_per_1m: number;
  output_price_per_1m: number;
  billing_multiplier: number;
  balance_before: number;
  balance_after?: number | null;
  estimated_cost: number;
  actual_cost?: number | null;
  diff_cost?: number | null;
  diff_ratio?: number | null;
  status: TestRunStatus;
  created_at: string;
  completed_at?: string | null;
  usage_source: UsageSource;
  request_logs: RequestLog[];
}

export interface TestRunSummary {
  id: string;
  provider_name: string;
  base_url: string;
  model_requested: string;
  estimated_cost: number;
  actual_cost?: number | null;
  diff_ratio?: number | null;
  status: TestRunStatus;
  created_at: string;
  completed_at?: string | null;
}

export type ExportFormat = "json" | "markdown" | "csv";

export interface ExportResult {
  path: string;
  content: string;
}
