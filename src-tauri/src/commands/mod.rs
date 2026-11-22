//! Tauri commands, one module per screen.

pub mod analyze;
pub mod organize;
pub mod scan;
pub mod search;
pub mod system;

use std::path::PathBuf;

use crate::core::errors::{AppError, Result};

/// Run blocking work (disk walks, hashing, HTTP to Ollama) off the async runtime.
pub(crate) async fn blocking<T, F>(f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Generic(format!("Background task failed: {e}")))?
}

/// Validate a folder chosen in the UI.
pub(crate) fn folder(dir: &str) -> Result<PathBuf> {
    let p = PathBuf::from(dir);
    if !p.is_dir() {
        return Err(AppError::Validation(format!("Folder not found: {dir}")));
    }
    Ok(std::path::absolute(&p).unwrap_or(p))
}
