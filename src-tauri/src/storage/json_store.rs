use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::core::errors::{AppError, Result};

pub fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Err(AppError::Storage(format!("File does not exist: {:?}", path)));
    }
    let content = fs::read_to_string(path).map_err(AppError::Io)?;
    let parsed: T = serde_json::from_str(&content).map_err(AppError::Json)?;
    Ok(parsed)
}

/// Atomically write JSON data: writes to a temporary file, flushes to disk,
/// and renames to the destination path. Also creates a .bak backup.
pub fn write_json_atomic<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }

    let json_bytes = serde_json::to_vec_pretty(data).map_err(AppError::Json)?;

    // Backup existing file if it exists
    if path.exists() {
        let bak_path = path.with_extension("json.bak");
        let _ = fs::copy(path, bak_path);
    }

    // Write to temporary file
    let tmp_path = path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
    {
        let mut file = File::create(&tmp_path).map_err(AppError::Io)?;
        file.write_all(&json_bytes).map_err(AppError::Io)?;
        file.sync_all().map_err(AppError::Io)?;
    }

    // Atomic replacement
    fs::rename(&tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        AppError::Storage(format!("Atomic replacement failed for {:?}: {}", path, e))
    })?;

    Ok(())
}

pub fn append_jsonl<T: Serialize>(path: &Path, entry: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
    }

    let mut line = serde_json::to_string(entry).map_err(AppError::Json)?;
    line.push('\n');

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(AppError::Io)?;

    file.write_all(line.as_bytes()).map_err(AppError::Io)?;
    file.sync_all().map_err(AppError::Io)?;

    Ok(())
}
