//! AI Analysis screen: describe images with the local vision model.

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::ai::embedding::build_semantic_text;
use crate::ai::provider::{model_installed, EmbeddingProvider, RuntimeProvider};
use crate::core::app_state::AppState;
use crate::core::errors::{AppError, Result};
use crate::core::events::{AnalyzeFileEvent, AnalyzeStartedEvent, ProgressEvent};
use crate::organization::naming::generate_deterministic_filename;
use crate::storage::repository::IndexedFile;

use super::scan::index_directory;
use super::{blocking, folder};

const JOB_ID: &str = "analyze";

#[derive(Serialize)]
pub struct AnalyzeSummary {
    pub total: usize,
    pub analyzed: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub search_indexed: usize,
    pub elapsed_ms: u128,
}

#[tauri::command]
pub fn cancel_analysis(state: State<'_, AppState>) -> bool {
    state.job_manager.cancel_job(JOB_ID)
}

#[tauri::command]
pub async fn analyze_folder(
    dir: String,
    force: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<AnalyzeSummary> {
    let dir = folder(&dir)?;
    if state.job_manager.get_running_jobs().contains_key(JOB_ID) {
        return Err(AppError::Validation("An analysis is already running.".into()));
    }
    let cancel = state.job_manager.start_job(JOB_ID, "Analyzing images");
    let repo = state.repository.clone();
    let ollama = state.ollama.clone();
    let vision = state.vision_coordinator.clone();

    let result = blocking(move || {
        if !ollama.health_check().unwrap_or(false) {
            return Err(AppError::Ollama("Ollama is not running. Start it with `ollama serve`.".into()));
        }
        let installed = ollama.fetch_models().unwrap_or_default();
        let settings = repo.lock().unwrap().get_settings();
        let model = settings.preferred_vision_model.clone();
        if !model_installed(&installed, &model) {
            return Err(AppError::Model(format!("Vision model '{model}' is not installed. Run `ollama pull {model}`.")));
        }
        let embed_model = Some(settings.preferred_embedding_model.clone()).filter(|m| model_installed(&installed, m));

        let files = index_directory(&repo, &dir, |current, total| {
            let _ = app.emit("analyze:indexing", ProgressEvent { current, total });
        })?;
        let analyzed: HashSet<String> = repo.lock().unwrap().get_all_analyses().into_iter().map(|a| a.file_id).collect();

        // IDs derive from content hashes, so byte-identical copies share one analysis.
        let mut seen = HashSet::new();
        let targets: Vec<&IndexedFile> = files
            .iter()
            .filter(|f| (force || !analyzed.contains(&f.id)) && seen.insert(f.id.clone()))
            .collect();
        let total = targets.len();
        let unique_total = files.iter().map(|f| &f.id).collect::<HashSet<_>>().len();

        let _ = app.emit(
            "analyze:started",
            AnalyzeStartedEvent {
                total,
                already_analyzed: unique_total - total,
                model: model.clone(),
                search_model: embed_model.clone(),
            },
        );

        let started = Instant::now();
        let (mut done, mut failed) = (0, 0);
        let mut pending: Vec<(String, Vec<f32>)> = Vec::new();
        let mut search_indexed = 0;

        let flush = |pending: &mut Vec<(String, Vec<f32>)>, model: &str| -> Result<usize> {
            let n = pending.len();
            if let Some(dims) = pending.first().map(|(_, v)| v.len()) {
                repo.lock().unwrap().save_embeddings(model, dims, &std::mem::take(pending))?;
            }
            Ok(n)
        };

        for (i, file) in targets.iter().enumerate() {
            if cancel.load(Ordering::SeqCst) {
                break;
            }
            let mut event = AnalyzeFileEvent {
                index: i + 1,
                total,
                file_id: file.id.clone(),
                name: file.current_name.clone(),
                path: file.current_path.clone(),
                hash: file.hash.clone(),
                analysis: None,
                proposed_name: None,
                error: None,
            };
            match vision.analyze_file_sync(file, &model) {
                Ok(analysis) => {
                    repo.lock().unwrap().save_analyses(std::slice::from_ref(&analysis))?;
                    done += 1;
                    if let Some(m) = &embed_model {
                        let text = build_semantic_text(&analysis);
                        if let Ok(v) = ollama.embed_text(m, &text) {
                            pending.push((analysis.file_id.clone(), v));
                        }
                        if pending.len() >= 10 {
                            search_indexed += flush(&mut pending, m)?;
                        }
                    }
                    let ext = Path::new(&file.current_name).extension().and_then(|e| e.to_str()).unwrap_or("png");
                    event.proposed_name =
                        Some(generate_deterministic_filename(&analysis, ext, &settings.default_naming_convention));
                    event.analysis = Some(analysis);
                }
                Err(e) => {
                    failed += 1;
                    event.error = Some(e.to_string());
                }
            }
            let _ = app.emit("analyze:file", event);
        }

        if let Some(m) = &embed_model {
            search_indexed += flush(&mut pending, m)?;
            // Backfill search vectors for files analyzed earlier without one.
            if !cancel.load(Ordering::SeqCst) {
                let missing: Vec<_> = {
                    let r = repo.lock().unwrap();
                    let have = r.get_all_embeddings();
                    files.iter().filter(|f| !have.contains_key(&f.id)).filter_map(|f| r.get_analysis(&f.id)).collect()
                };
                for a in missing {
                    if cancel.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(v) = ollama.embed_text(m, &build_semantic_text(&a)) {
                        pending.push((a.file_id.clone(), v));
                    }
                }
                search_indexed += flush(&mut pending, m)?;
            }
        }

        Ok(AnalyzeSummary {
            total,
            analyzed: done,
            failed,
            cancelled: cancel.load(Ordering::SeqCst),
            search_indexed,
            elapsed_ms: started.elapsed().as_millis(),
        })
    })
    .await;

    state.job_manager.finish_job(JOB_ID);
    result
}
