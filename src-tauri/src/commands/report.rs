use tauri::AppHandle;

use crate::{core::models::ExportResult, storage::reports::export_report_file};

#[tauri::command]
pub fn export_report(
    app: AppHandle,
    report_id: String,
    format: String,
) -> Result<ExportResult, String> {
    export_report_file(&app, &report_id, &format)
}
