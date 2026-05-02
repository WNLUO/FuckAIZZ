use std::sync::atomic::Ordering;

use tauri::{AppHandle, State};

use crate::{
    core::{
        models::{StartTestRunInput, TestRunReport, TestRunSummary},
        runner::run_test,
    },
    storage::reports::{finalize_report, list_reports, load_report},
    AppState,
};

#[tauri::command]
pub async fn start_test_run(
    app: AppHandle,
    state: State<'_, AppState>,
    input: StartTestRunInput,
) -> Result<TestRunReport, String> {
    run_test(app, state, input).await
}

#[tauri::command]
pub fn stop_test_run(state: State<'_, AppState>) {
    state.stop_requested.store(true, Ordering::SeqCst);
}

#[tauri::command]
pub fn list_test_runs(app: AppHandle) -> Result<Vec<TestRunSummary>, String> {
    list_reports(&app)
}

#[tauri::command]
pub fn get_test_run(app: AppHandle, report_id: String) -> Result<TestRunReport, String> {
    load_report(&app, &report_id)
}

#[tauri::command]
pub fn finalize_test_run(
    app: AppHandle,
    report_id: String,
    balance_after: f64,
) -> Result<TestRunReport, String> {
    finalize_report(&app, &report_id, balance_after)
}
