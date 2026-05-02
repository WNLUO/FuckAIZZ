mod commands;
mod core;
mod storage;

use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub stop_requested: AtomicBool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            stop_requested: AtomicBool::new(false),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::provider::list_provider_models,
            commands::provider::generate_test_prompt,
            commands::provider::refresh_pricing_catalog,
            commands::test_run::start_test_run,
            commands::test_run::stop_test_run,
            commands::test_run::list_test_runs,
            commands::test_run::get_test_run,
            commands::test_run::finalize_test_run,
            commands::report::export_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
