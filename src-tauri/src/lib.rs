pub mod ai;
pub mod commands;
pub mod core;
pub mod filesystem;
pub mod organization;
pub mod search;
pub mod storage;

#[cfg(test)]
mod tests;

use crate::core::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();

    tauri::Builder::default()
        .manage(state)
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            // Scan commands
            commands::scan::select_directory,
            commands::scan::scan_directory,
            commands::scan::get_indexed_files,
            // Analyze commands
            commands::analyze::list_available_models,
            commands::analyze::get_file_analysis,
            commands::analyze::analyze_files,
            commands::analyze::cancel_analysis,
            commands::analyze::clear_analyses,
            // Organize commands
            commands::organize::plan_organization,
            commands::organize::execute_organization,
            commands::organize::get_duplicate_files,
            // Search commands
            commands::search::search_semantic,
            commands::search::find_similar_images,
            // Settings & Stats
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_app_stats,
            // Undo & History
            commands::undo::get_operation_history,
            commands::undo::undo_last_operation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running zorter application");
}
