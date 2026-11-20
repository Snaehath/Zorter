use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use crate::core::errors::{AppError, Result};
use crate::filesystem::hashing::calculate_blake3_hash;
use crate::filesystem::metadata::{extract_metadata, is_supported_image};
use crate::filesystem::thumbnail::{generate_thumbnail, thumbnail_to_data_url};
use crate::storage::repository::IndexedFile;

pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_path: String,
}

pub fn scan_directory<F>(
    dir_path: &Path,
    cache_dir: &Path,
    existing_files: &[IndexedFile],
    generate_thumbs: bool,
    cancellation_flag: Option<&Arc<AtomicBool>>,
    on_progress: F,
) -> Result<Vec<IndexedFile>>
where
    F: Fn(ScanProgress),
{
    if !dir_path.exists() || !dir_path.is_dir() {
        return Err(AppError::Validation(format!(
            "Directory does not exist or is not a directory: {:?}",
            dir_path
        )));
    }

    let mut existing_map: HashMap<String, &IndexedFile> = HashMap::new();
    for f in existing_files {
        existing_map.insert(f.current_path.clone(), f);
    }

    // Step 1: Collect valid image candidate paths
    let mut candidates: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !file_name.starts_with('.') && file_name != "node_modules" && file_name != "AppData"
        })
    {
        if let Some(flag) = cancellation_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(AppError::Cancelled);
            }
        }

        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // Gracefully handle permission-denied or inaccessible files
        };

        let path = entry.path();
        if path.is_file() && is_supported_image(path) {
            candidates.push(path.to_path_buf());
        }
    }

    let total = candidates.len();
    let mut scanned_files: Vec<IndexedFile> = Vec::with_capacity(total);

    // Step 2: Extract metadata, incremental hash comparison, thumbnail generation
    for (idx, path) in candidates.iter().enumerate() {
        if let Some(flag) = cancellation_flag {
            if flag.load(Ordering::SeqCst) {
                return Err(AppError::Cancelled);
            }
        }

        let path_str = path.to_string_lossy().to_string();
        on_progress(ScanProgress {
            current: idx + 1,
            total,
            current_path: path_str.clone(),
        });

        let metadata = match extract_metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Check if we can reuse existing hash via incremental scanning strategy:
        // Compare path, size, and modified timestamp
        let mut blake3_hash = None;
        let mut original_name = metadata.name.clone();
        let mut original_path = path_str.clone();
        let mut thumb_url = None;

        if let Some(existing) = existing_map.get(&path_str) {
            if existing.size == metadata.size && existing.modified_ms == metadata.modified_ms {
                blake3_hash = Some(existing.hash.clone());
                original_name = existing.original_name.clone();
                original_path = existing.original_path.clone();
                thumb_url = existing.thumbnail_url.clone();
            }
        }

        let hash = match blake3_hash {
            Some(h) => h,
            None => match calculate_blake3_hash(path) {
                Ok(h) => h,
                Err(_) => continue,
            },
        };

        let id = format!("file_{}", &hash[..16]);

        let final_thumb_url = if thumb_url.is_some() {
            thumb_url
        } else if generate_thumbs {
            match generate_thumbnail(path, cache_dir, &hash, 256) {
                Ok(thumb_path) => thumbnail_to_data_url(&thumb_path).ok(),
                Err(_) => None,
            }
        } else {
            None
        };

        scanned_files.push(IndexedFile {
            id,
            original_name,
            current_name: metadata.name,
            original_path,
            current_path: path_str,
            file_type: metadata.mime_type,
            size: metadata.size,
            modified_ms: metadata.modified_ms,
            hash,
            thumbnail_url: final_thumb_url,
            indexed: true,
        });
    }

    Ok(scanned_files)
}
