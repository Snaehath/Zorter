//! App overview (home / status / models), folder picker, thumbnails.

use std::path::Path;

use serde::Serialize;
use tauri::State;

use crate::ai::provider::{model_installed, RuntimeProvider};
use crate::core::app_state::AppState;
use crate::core::errors::{AppError, Result};
use crate::filesystem::operations::find_undoable_operation;
use crate::storage::repository::VISION_MODELS;
use crate::filesystem::thumbnail::{generate_thumbnail, thumbnail_to_data_url};

use super::blocking;

#[derive(Serialize)]
pub struct ModelStatus {
    pub kind: &'static str,
    pub name: String,
    pub installed: bool,
    pub size_bytes: u64,
}

#[derive(Serialize)]
pub struct UndoableOperation {
    pub timestamp: String,
    pub files: usize,
}

#[derive(Serialize)]
pub struct Overview {
    pub version: &'static str,
    pub endpoint: String,
    pub online: bool,
    pub models: Vec<ModelStatus>,
    /// Every vision model the user can choose from; `models` holds the one in use.
    pub vision_models: Vec<ModelStatus>,
    pub files: usize,
    pub analyzed: usize,
    pub searchable: usize,
    pub storage_path: String,
    pub storage_bytes: u64,
    pub last_scan: Option<String>,
    pub last_analysis: Option<String>,
    pub operations_completed: usize,
    pub operations_failed: usize,
    pub operations_undone: usize,
    pub undoable: Option<UndoableOperation>,
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Switch the vision model used for analysis (one of `VISION_MODELS`).
#[tauri::command]
pub async fn set_vision_model(model: String, state: State<'_, AppState>) -> Result<()> {
    if !VISION_MODELS.contains(&model.as_str()) {
        return Err(AppError::Model(format!("Unknown vision model '{model}'.")));
    }
    let repo = state.repository.clone();
    blocking(move || repo.lock().unwrap().set_vision_model(&model)).await
}

#[tauri::command]
pub async fn get_overview(state: State<'_, AppState>) -> Result<Overview> {
    let repo = state.repository.clone();
    let ollama = state.ollama.clone();
    blocking(move || {
        let online = ollama.health_check().unwrap_or(false);
        let installed = if online { ollama.fetch_models().unwrap_or_default() } else { Vec::new() };

        let repo = repo.lock().unwrap();
        let settings = repo.get_settings();
        let analyses = repo.get_all_analyses();
        let history = repo.get_operation_history().unwrap_or_default();
        let base = repo.get_base_dir().to_path_buf();

        let status = |kind: &'static str, name: String| {
            let found = installed.iter().find(|m| model_installed(std::slice::from_ref(m), &name));
            ModelStatus { kind, installed: found.is_some(), size_bytes: found.map(|m| m.size_bytes).unwrap_or(0), name }
        };
        let models = vec![status("Vision", settings.vision_model.clone()), status("Embedding", settings.preferred_embedding_model.clone())];
        let vision_models = VISION_MODELS.iter().map(|n| status("Vision", n.to_string())).collect();

        let files = repo.get_all_files().len();
        let last_scan = std::fs::metadata(base.join("index").join("files.json"))
            .and_then(|m| m.modified())
            .ok()
            .filter(|_| files > 0)
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
        let last_analysis = analyses
            .iter()
            .filter_map(|a| chrono::DateTime::parse_from_rfc3339(&a.analyzed_at).ok())
            .max()
            .map(|d| d.to_rfc3339());

        Ok(Overview {
            version: env!("CARGO_PKG_VERSION"),
            endpoint: settings.ollama_endpoint.clone(),
            online,
            models,
            vision_models,
            files,
            analyzed: analyses.len(),
            searchable: repo.get_all_embeddings().len(),
            storage_bytes: dir_size(&base),
            storage_path: base.display().to_string(),
            last_scan,
            last_analysis,
            operations_completed: history.iter().filter(|e| e.status == "completed").count(),
            operations_failed: history.iter().filter(|e| e.status == "failed").count(),
            operations_undone: history.iter().filter(|e| e.operation_type.starts_with("rollback:")).count(),
            undoable: find_undoable_operation(&history)
                .map(|op| UndoableOperation { timestamp: op.timestamp, files: op.actions.len() }),
        })
    })
    .await
}

#[tauri::command]
pub async fn pick_directory() -> Result<Option<String>> {
    let folder = rfd::AsyncFileDialog::new().set_title("Choose a folder").pick_folder().await;
    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

/// Small cached JPEG thumbnail as a data URL (`None` if the image can't be read).
#[tauri::command]
pub async fn get_thumbnail(path: String, hash: String, state: State<'_, AppState>) -> Result<Option<String>> {
    let cache = state.repository.lock().unwrap().get_cache_dir();
    blocking(move || {
        let key = if hash.is_empty() { blake3::hash(path.as_bytes()).to_hex().to_string() } else { hash };
        Ok(generate_thumbnail(Path::new(&path), &cache, &key, 320)
            .and_then(|p| thumbnail_to_data_url(&p))
            .ok())
    })
    .await
}

/// Show a file selected in the system file manager.
#[tauri::command]
pub fn reveal_in_folder(path: String) -> Result<()> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    let r = Command::new("explorer").arg(format!("/select,{path}")).spawn();
    #[cfg(target_os = "macos")]
    let r = Command::new("open").args(["-R", &path]).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let r = Command::new("xdg-open")
        .arg(Path::new(&path).parent().unwrap_or(Path::new("/")))
        .spawn();
    r.map(|_| ()).map_err(Into::into)
}
