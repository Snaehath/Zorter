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
            commands::system::get_overview,
            commands::system::set_vision_model,
            commands::system::pick_directory,
            commands::system::get_thumbnail,
            commands::system::reveal_in_folder,
            commands::scan::scan_folder,
            commands::analyze::analyze_preview,
            commands::analyze::analyze_folder,
            commands::analyze::cancel_analysis,
            commands::organize::plan_folder,
            commands::organize::apply_changes,
            commands::organize::export_plan,
            commands::organize::undo_last,
            commands::search::search_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running zorter application");
}
