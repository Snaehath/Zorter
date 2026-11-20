use tauri::{AppHandle, Emitter, State};

use crate::ai::provider::ModelInfo;
use crate::core::app_state::AppState;
use crate::core::errors::Result;
use crate::core::events::{AnalyzeCompletedEvent, AnalyzeProgressEvent};
use crate::storage::repository::{FileAnalysis, IndexedFile};

#[tauri::command]
pub fn clear_analyses(state: State<'_, AppState>) -> Result<()> {
    let mut repo = state.repository.lock().unwrap();
    repo.clear_all_analyses()?;
    Ok(())
}

#[tauri::command]
pub async fn list_available_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>> {
    state.ollama.fetch_models()
}

#[tauri::command]
pub async fn get_file_analysis(
    file_id: String,
    state: State<'_, AppState>,
) -> Result<Option<FileAnalysis>> {
    let repo = state.repository.lock().unwrap();
    Ok(repo.get_analysis(&file_id))
}

#[tauri::command]
pub fn cancel_analysis(state: State<'_, AppState>) -> Result<bool> {
    let cancelled = state.job_manager.cancel_job("analyze:batch");
    Ok(cancelled)
}

#[tauri::command]
pub async fn analyze_files(
    file_ids: Option<Vec<String>>,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Vec<FileAnalysis>> {
    let job_id = "analyze:batch".to_string();
    let cancel_flag = state.job_manager.start_job(&job_id, "Analyzing images with local AI");

    let (settings, files_to_process) = {
        let repo = state.repository.lock().unwrap();
        let settings = repo.get_settings();
        let all_files = repo.get_all_files();
        let analyses = repo.get_all_analyses();
        let analyzed_ids: std::collections::HashSet<String> =
            analyses.into_iter().map(|a| a.file_id).collect();

        let targets: Vec<IndexedFile> = match file_ids {
            Some(ids) => {
                let id_set: std::collections::HashSet<String> = ids.into_iter().collect();
                all_files.into_iter().filter(|f| id_set.contains(&f.id)).collect()
            }
            None => {
                let unanalyzed: Vec<IndexedFile> = all_files
                    .iter()
                    .filter(|f| !analyzed_ids.contains(&f.id))
                    .cloned()
                    .collect();
                if unanalyzed.is_empty() {
                    // All files already analyzed -> Re-analyze all files
                    all_files
                } else {
                    unanalyzed
                }
            }
        };

        (settings, targets)
    };

    let total = files_to_process.len();
    let mut completed_analyses = Vec::new();
    let mut failed = 0;

    let vision_model = settings.preferred_vision_model.clone();

    for (idx, file) in files_to_process.iter().enumerate() {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            log::info!("AI analysis cancelled by user. Saving partial progress...");
            if !completed_analyses.is_empty() {
                let mut repo = state.repository.lock().unwrap();
                let _ = repo.save_analyses(&completed_analyses);
            }
            state.job_manager.finish_job(&job_id);
            let _ = app_handle.emit(
                "analyze:completed",
                AnalyzeCompletedEvent {
                    total_analyzed: completed_analyses.len(),
                    failed,
                },
            );
            return Ok(completed_analyses);
        }

        let _ = app_handle.emit(
            "analyze:progress",
            AnalyzeProgressEvent {
                current: idx + 1,
                total,
                file_name: file.current_name.clone(),
                status: format!("Analyzing with {}", vision_model),
            },
        );

        match state.vision_coordinator.analyze_file(file, &vision_model).await {
            Ok(analysis) => {
                completed_analyses.push(analysis.clone());

                // Incrementally save progress to survive interruptions
                if completed_analyses.len() % 5 == 0 {
                    let mut repo = state.repository.lock().unwrap();
                    let _ = repo.save_analyses(&completed_analyses);
                }
            }
            Err(e) => {
                log::error!("Failed to analyze file {}: {}", file.current_name, e);
                failed += 1;
            }
        }
    }

    // Final save of analyses
    if !completed_analyses.is_empty() {
        let mut repo = state.repository.lock().unwrap();
        repo.save_analyses(&completed_analyses)?;
    }

    state.job_manager.finish_job(&job_id);

    let _ = app_handle.emit(
        "analyze:completed",
        AnalyzeCompletedEvent {
            total_analyzed: completed_analyses.len(),
            failed,
        },
    );

    Ok(completed_analyses)
}
