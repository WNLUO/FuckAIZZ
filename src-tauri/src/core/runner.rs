use std::sync::atomic::Ordering;

use chrono::Utc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::{storage::reports::save_report, AppState};

use super::{
    models::{
        ProbeInput, RequestLog, RequestStatus, StartTestRunInput, TestProgress, TestRunReport,
        TestRunStatus, UsageSource,
    },
    openai_compat::{build_http_client, request_chat_completion},
    safety::{validate_base_url, validate_start_input},
};

pub async fn run_test(
    app: AppHandle,
    state: State<'_, AppState>,
    input: StartTestRunInput,
) -> Result<TestRunReport, String> {
    validate_start_input(&input)?;
    let normalized_base_url = validate_base_url(&input.base_url).await?;
    let client = build_http_client(input.timeout_secs)?;
    let run_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let probe_input: ProbeInput = input.clone().into();
    let target_run_cost = input.target_usd;

    state.stop_requested.store(false, Ordering::SeqCst);

    let mut logs = Vec::new();
    let mut estimated_cost = 0.0;
    let mut total_tokens = 0;
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut consecutive_failures = 0;
    let mut model_reported = None;
    let mut aggregate_usage_source = UsageSource::Api;

    let mut index = 1;

    let status = loop {
        if state.stop_requested.load(Ordering::SeqCst) {
            let status = TestRunStatus::Stopped;
            emit_progress(
                &app,
                &run_id,
                logs.len() as u32,
                success_count,
                failed_count,
                estimated_cost,
                total_tokens,
                status,
                None,
            );
            break status;
        }

        match request_chat_completion(&client, &normalized_base_url, &probe_input).await {
            Ok(outcome) => {
                consecutive_failures = 0;
                success_count += 1;
                estimated_cost += outcome.estimated_cost;
                total_tokens += outcome.total_tokens;
                let recent_success_cost = outcome.estimated_cost;
                if model_reported.is_none() {
                    model_reported = outcome.model_reported.clone();
                }
                if outcome.usage_source == UsageSource::Estimated {
                    aggregate_usage_source = UsageSource::Estimated;
                }
                let log = RequestLog {
                    request_index: index,
                    status: RequestStatus::Success,
                    latency_ms: outcome.latency_ms,
                    prompt_tokens: outcome.prompt_tokens,
                    cached_prompt_tokens: outcome.cached_prompt_tokens,
                    completion_tokens: outcome.completion_tokens,
                    total_tokens: outcome.total_tokens,
                    raw_estimated_cost: outcome.raw_estimated_cost,
                    estimated_cost: outcome.estimated_cost,
                    response_summary: outcome.response_summary,
                    error_message: None,
                    usage_source: outcome.usage_source,
                    created_at: Utc::now().to_rfc3339(),
                };
                logs.push(log.clone());
                emit_progress(
                    &app,
                    &run_id,
                    logs.len() as u32,
                    success_count,
                    failed_count,
                    estimated_cost,
                    total_tokens,
                    TestRunStatus::Running,
                    Some(log),
                );

                if estimated_cost >= target_run_cost {
                    break TestRunStatus::Completed;
                }

                let remaining = target_run_cost - estimated_cost;
                if remaining > 0.0 && remaining < recent_success_cost * 0.35 {
                    break TestRunStatus::StoppedOnBudgetGuard;
                }

                if success_count >= 5 && estimated_cost <= f64::EPSILON {
                    break TestRunStatus::Failed;
                }
            }
            Err(err) => {
                failed_count += 1;
                consecutive_failures += 1;
                let log = RequestLog {
                    request_index: index,
                    status: RequestStatus::Error,
                    latency_ms: 0,
                    prompt_tokens: 0,
                    cached_prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                    raw_estimated_cost: 0.0,
                    estimated_cost: 0.0,
                    response_summary: String::new(),
                    error_message: Some(redact_error(&err)),
                    usage_source: UsageSource::Estimated,
                    created_at: Utc::now().to_rfc3339(),
                };
                logs.push(log.clone());
                emit_progress(
                    &app,
                    &run_id,
                    logs.len() as u32,
                    success_count,
                    failed_count,
                    estimated_cost,
                    total_tokens,
                    TestRunStatus::Running,
                    Some(log),
                );

                if consecutive_failures >= 3 {
                    break TestRunStatus::PausedOnFailures;
                }
            }
        }

        index += 1;
    };

    let report = TestRunReport {
        id: run_id,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        provider_name: input.name.trim().to_string(),
        base_url: normalized_base_url,
        model_requested: input.model.trim().to_string(),
        model_reported,
        current_usd: input.current_usd,
        target_usd: input.target_usd,
        input_price_per_1m: input.input_price_per_1m,
        cached_input_price_per_1m: input.cached_input_price_per_1m,
        output_price_per_1m: input.output_price_per_1m,
        billing_multiplier: input.billing_multiplier,
        balance_before: input.balance_before,
        balance_after: None,
        estimated_cost,
        actual_cost: None,
        diff_cost: None,
        diff_ratio: None,
        status,
        created_at,
        completed_at: Some(Utc::now().to_rfc3339()),
        usage_source: aggregate_usage_source,
        request_logs: logs,
    };

    save_report(&app, &report)?;
    emit_progress(
        &app,
        &report.id,
        report.request_logs.len() as u32,
        success_count,
        failed_count,
        report.estimated_cost,
        total_tokens,
        report.status,
        None,
    );

    Ok(report)
}

fn emit_progress(
    app: &AppHandle,
    run_id: &str,
    request_count: u32,
    success_count: u32,
    failed_count: u32,
    estimated_cost: f64,
    total_tokens: u32,
    status: TestRunStatus,
    latest_log: Option<RequestLog>,
) {
    let _ = app.emit(
        "test-progress",
        TestProgress {
            run_id: run_id.to_string(),
            request_count,
            success_count,
            failed_count,
            estimated_cost,
            total_tokens,
            status,
            latest_log,
        },
    );
}

fn redact_error(value: &str) -> String {
    value
        .replace("Authorization", "[redacted-header]")
        .replace("Bearer ", "Bearer [redacted]")
}
