//! Scan screen: index a folder and report what needs attention.

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::core::app_state::AppState;
use crate::core::errors::Result;
use crate::core::events::ProgressEvent;
use crate::filesystem::census::census;
use crate::filesystem::scanner::scan_directory;
use crate::organization::deduplication::detect_duplicates;
use crate::storage::repository::{IndexedFile, Repository};

use super::{blocking, folder};

#[derive(Serialize)]
pub struct ScanReport {
    pub directory: String,
    pub files_discovered: usize,
    pub images: usize,
    pub documents: usize,
    pub other: usize,
    pub already_indexed: usize,
    pub new_files: usize,
    pub needs_analysis: usize,
    pub duplicates: usize,
    pub wasted_bytes: u64,
    pub elapsed_ms: u128,
}

/// Scan `dir` into the index. The repository lock is only held to read the
/// existing index and to save results, never during the disk walk.
pub(crate) fn index_directory(
    repo: &Arc<Mutex<Repository>>,
    dir: &Path,
    on_progress: impl Fn(usize, usize),
) -> Result<Vec<IndexedFile>> {
    let (existing, cache) = {
        let r = repo.lock().unwrap();
        (r.get_all_files(), r.get_cache_dir())
    };
    let files = scan_directory(dir, &cache, &existing, false, None, |p| on_progress(p.current, p.total))?;
    let mut r = repo.lock().unwrap();
    r.save_files(&files)?;
    let _ = r.add_monitored_directory(&dir.to_string_lossy(), files.len());
    Ok(files)
}

#[tauri::command]
pub async fn scan_folder(dir: String, state: State<'_, AppState>, app: AppHandle) -> Result<ScanReport> {
    let dir = folder(&dir)?;
    let repo = state.repository.clone();
    blocking(move || {
        let started = Instant::now();
        let counts = census(&dir);
        let (indexed_ids, analyzed_ids): (HashSet<String>, HashSet<String>) = {
            let r = repo.lock().unwrap();
            (
                r.get_all_files().into_iter().map(|f| f.id).collect(),
                r.get_all_analyses().into_iter().map(|a| a.file_id).collect(),
            )
        };

        let files = index_directory(&repo, &dir, |current, total| {
            let _ = app.emit("scan:progress", ProgressEvent { current, total });
        })?;

        let unique: HashSet<&str> = files.iter().map(|f| f.id.as_str()).collect();
        let already = files.iter().filter(|f| indexed_ids.contains(&f.id)).count();
        let dupes = detect_duplicates(&files);

        Ok(ScanReport {
            directory: dir.display().to_string(),
            files_discovered: counts.total,
            images: files.len(),
            documents: counts.documents,
            other: counts.other,
            already_indexed: already,
            new_files: files.len() - already,
            needs_analysis: unique.iter().filter(|id| !analyzed_ids.contains(**id)).count(),
            duplicates: dupes.iter().map(|d| d.files.len() - 1).sum(),
            wasted_bytes: dupes.iter().map(|d| d.wasted_bytes).sum(),
            elapsed_ms: started.elapsed().as_millis(),
        })
    })
    .await
}
