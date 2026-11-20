use serde::{Deserialize, Serialize};
use tauri::State;

use crate::core::app_state::AppState;
use crate::core::errors::Result;
use crate::organization::deduplication::detect_duplicates;
use crate::storage::repository::AppSettings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStats {
    pub total_files: usize,
    pub total_analyzed: usize,
    pub total_embeddings: usize,
    pub total_duplicates: usize,
    pub wasted_bytes: u64,
    pub monitored_directories: usize,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings> {
    let repo = state.repository.lock().unwrap();
    Ok(repo.get_settings())
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<()> {
    let mut repo = state.repository.lock().unwrap();
    repo.save_settings(settings)?;
    Ok(())
}

#[tauri::command]
pub async fn get_app_stats(state: State<'_, AppState>) -> Result<AppStats> {
    let repo = state.repository.lock().unwrap();
    let files = repo.get_all_files();
    let analyses = repo.get_all_analyses();
    let embeddings = repo.get_all_embeddings();
    let directories = repo.get_monitored_directories();

    let duplicates = detect_duplicates(&files);
    let wasted_bytes: u64 = duplicates.iter().map(|d| d.wasted_bytes).sum();
    let total_duplicate_files: usize = duplicates.iter().map(|d| d.files.len()).sum();

    Ok(AppStats {
        total_files: files.len(),
        total_analyzed: analyses.len(),
        total_embeddings: embeddings.len(),
        total_duplicates: total_duplicate_files,
        wasted_bytes,
        monitored_directories: directories.len(),
    })
}
