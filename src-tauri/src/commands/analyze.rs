use std::path::Path;
use tauri::{AppHandle, Emitter, State};

use crate::ai::provider::ModelInfo;
use crate::core::app_state::AppState;
use crate::core::errors::Result;
use crate::core::events::{AnalyzeCompletedEvent, AnalyzeProgressEvent};
use crate::filesystem::metadata_extractor::{extract_png_metadata, is_valid_filename_keyword};
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

/// Instant heuristic analyzer (0ms, no waiting on LLMs):
/// Reads embedded PNG generation metadata (Flux, ComfyUI, Stable Diffusion, etc.)
/// and filename cues to generate instant subject, category, and names.
#[tauri::command]
pub async fn fast_heuristic_analyze(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<Vec<FileAnalysis>> {
    let job_id = "analyze:batch".to_string();
    let cancel_flag = state.job_manager.start_job(&job_id, "Instant metadata heuristic analysis");

    let files_to_process = {
        let repo = state.repository.lock().unwrap();
        repo.get_all_files()
    };

    let total = files_to_process.len();
    let mut completed = Vec::new();

    for (idx, file) in files_to_process.iter().enumerate() {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            log::info!("Instant analysis cancelled by user");
            if !completed.is_empty() {
                let mut repo = state.repository.lock().unwrap();
                let _ = repo.save_analyses(&completed);
            }
            state.job_manager.finish_job(&job_id);
            let _ = app_handle.emit(
                "analyze:completed",
                AnalyzeCompletedEvent {
                    total_analyzed: completed.len(),
                    failed: 0,
                },
            );
            return Ok(completed);
        }

        let path = Path::new(&file.current_path);
        let meta = extract_png_metadata(path);

        let mut subjects = meta.detected_keywords;
        if subjects.is_empty() {
            // Extract from cleaned filename using the validator
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .replace('_', " ")
                .replace('-', " ");
            for word in stem.split_whitespace() {
                if is_valid_filename_keyword(word) {
                    subjects.push(word.to_lowercase());
                }
            }
        }

        // If no embedded prompt AND no valid semantic words could be extracted,
        // DO NOT create a fake analysis with random hashes. Leave it unanalyzed so Deep AI Vision inspects it!
        if subjects.is_empty() && meta.embedded_prompt.is_none() {
            let _ = app_handle.emit(
                "analyze:progress",
                AnalyzeProgressEvent {
                    current: idx + 1,
                    total,
                    file_name: file.current_name.clone(),
                    status: "Skipped (no prompt in file, needs Deep AI Vision)".to_string(),
                },
            );
            continue;
        }

        let primary_subject = subjects.first().cloned().unwrap_or_else(|| "wallpaper".to_string());
        let category = match primary_subject.as_str() {
            "waterfall" | "waterfalls" | "forest" | "mountain" | "mountains" | "desert" | "river" | "ocean" | "beach" => "Nature",
            "pyramid" | "pyramids" | "egypt" | "castle" | "city" => "Architecture",
            "lion" | "tiger" | "wolf" | "dragon" => "Animals",
            "cyberpunk" | "space" | "galaxy" | "planet" | "futuristic" => "Sci-Fi",
            _ => "Wallpapers",
        }.to_string();

        let subcategory = primary_subject
            .chars()
            .next()
            .map(|f| f.to_uppercase().to_string() + &primary_subject[1..])
            .unwrap_or_else(|| "General".to_string());

        let suggested_filename = if subjects.len() >= 2 {
            format!("{}_{}.png", subjects[0], subjects[1])
        } else {
            format!("{}.png", primary_subject)
        };

        let description = if let Some(prompt) = meta.embedded_prompt {
            prompt
        } else {
            format!("Image featuring {}", primary_subject)
        };

        let analysis = FileAnalysis {
            file_id: file.id.clone(),
            description: description.clone(),
            subject: subjects,
            category,
            subcategory,
            tags: vec![primary_subject],
            visual_style: "Digital".to_string(),
            environment: None,
            lighting: None,
            colors: Vec::new(),
            orientation: None,
            suggested_filename,
            model: "fast-heuristic".to_string(),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        };

        completed.push(analysis.clone());

        let _ = app_handle.emit(
            "analyze:progress",
            AnalyzeProgressEvent {
                current: idx + 1,
                total,
                file_name: file.current_name.clone(),
                status: "Instant prompt & metadata analysis".to_string(),
            },
        );
    }

    {
        let mut repo = state.repository.lock().unwrap();
        repo.save_analyses(&completed)?;
    }

    state.job_manager.finish_job(&job_id);

    let _ = app_handle.emit(
        "analyze:completed",
        AnalyzeCompletedEvent {
            total_analyzed: completed.len(),
            failed: 0,
        },
    );

    Ok(completed)
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
    let mut new_embeddings: Vec<(String, Vec<f32>)> = Vec::new();
    let mut failed = 0;

    let vision_model = settings.preferred_vision_model.clone();
    let embedding_model = settings.preferred_embedding_model.clone();

    for (idx, file) in files_to_process.iter().enumerate() {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            log::info!("AI analysis cancelled by user. Saving partial progress...");
            if !completed_analyses.is_empty() {
                let mut repo = state.repository.lock().unwrap();
                let _ = repo.save_analyses(&completed_analyses);
                if !new_embeddings.is_empty() {
                    let dims = new_embeddings[0].1.len();
                    let _ = repo.save_embeddings(&embedding_model, dims, &new_embeddings);
                }
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
                // Generate vector embedding for semantic search
                if let Ok(vec) = state
                    .embedding_coordinator
                    .generate_analysis_embedding(&embedding_model, &analysis)
                {
                    new_embeddings.push((file.id.clone(), vec));
                }

                completed_analyses.push(analysis.clone());

                // Incrementally save progress to survive interruptions
                if completed_analyses.len() % 5 == 0 {
                    let mut repo = state.repository.lock().unwrap();
                    let _ = repo.save_analyses(&completed_analyses);
                    if !new_embeddings.is_empty() {
                        let dims = new_embeddings[0].1.len();
                        let _ = repo.save_embeddings(&embedding_model, dims, &new_embeddings);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to analyze file {}: {}", file.current_name, e);
                failed += 1;
            }
        }
    }

    // Final save of analyses and embeddings
    if !completed_analyses.is_empty() {
        let mut repo = state.repository.lock().unwrap();
        repo.save_analyses(&completed_analyses)?;
        if !new_embeddings.is_empty() {
            let dims = new_embeddings[0].1.len();
            repo.save_embeddings(&embedding_model, dims, &new_embeddings)?;
        }
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
