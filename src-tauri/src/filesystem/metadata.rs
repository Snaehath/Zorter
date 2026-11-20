use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use serde::{Deserialize, Serialize};
use crate::core::errors::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub path: String,
    pub extension: String,
    pub mime_type: String,
    pub size: u64,
    pub modified_ms: u64,
}

const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp"];

pub fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

pub fn get_mime_type(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

pub fn extract_metadata(path: &Path) -> Result<FileMetadata> {
    let metadata = fs::metadata(path).map_err(AppError::Io)?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime_type = get_mime_type(&extension).to_string();

    let modified_ms = metadata
        .modified()
        .ok()
        .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    Ok(FileMetadata {
        name,
        path: path.to_string_lossy().to_string(),
        extension,
        mime_type,
        size: metadata.len(),
        modified_ms,
    })
}
