use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

use crate::core::app_state::AppState;
use crate::core::errors::{AppError, Result};
use crate::core::events::{ScanCompletedEvent, ScanProgressEvent};
use crate::filesystem::scanner::scan_directory as fs_scan;
use crate::storage::repository::IndexedFile;

#[tauri::command]
pub async fn select_directory() -> Result<Option<String>> {
    let folder = rfd::AsyncFileDialog::new()
        .set_title("Select Directory to Organize")
        .pick_folder()
        .await;

    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn scan_directory(
    dir_path: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Vec<IndexedFile>> {
    let job_id = format!("scan:{}", dir_path);
    let cancel_flag = state.job_manager.start_job(&job_id, &format!("Scanning {}", dir_path));

    let (cache_dir, existing_files) = {
        let repo = state.repository.lock().unwrap();
        (repo.get_cache_dir(), repo.get_all_files())
    };

    let target_path = PathBuf::from(&dir_path);
    let handle_clone = app_handle.clone();

    let scan_res = tokio::task::spawn_blocking(move || {
        fs_scan(
            &target_path,
            &cache_dir,
            &existing_files,
            false,
            Some(&cancel_flag),
            |prog| {
                let _ = handle_clone.emit(
                    "scan:progress",
                    ScanProgressEvent {
                        current: prog.current,
                        total: prog.total,
                        current_file: prog.current_path,
                    },
                );
            },
        )
    })
    .await
    .map_err(|e| AppError::Generic(format!("Scan task panicked: {}", e)))??;

    state.job_manager.finish_job(&job_id);

    // Save newly scanned files into repository
    {
        let mut repo = state.repository.lock().unwrap();
        repo.save_files(&scan_res)?;
        repo.add_monitored_directory(&dir_path, scan_res.len())?;
    }

    let _ = app_handle.emit(
        "scan:completed",
        ScanCompletedEvent {
            total_scanned: scan_res.len(),
            new_files: scan_res.len(),
        },
    );

    Ok(scan_res)
}

#[tauri::command]
pub async fn get_indexed_files(state: State<'_, AppState>) -> Result<Vec<IndexedFile>> {
    let repo = state.repository.lock().unwrap();
    Ok(repo.get_all_files())
}
