//! Organization screen: plan, apply, export and undo.

use std::path::Path;

use serde::Serialize;
use tauri::State;

use crate::core::app_state::AppState;
use crate::core::errors::{AppError, Result};
use crate::filesystem::operations::{
    execute_file_operations, find_undoable_operation, remove_empty_folders, rollback_operation,
    validate_operation_paths, FileOperationAction,
};
use crate::organization::changes::{build_changes, folder_counts, FolderCount, PlannedChange};
use crate::organization::folder_planner::create_organization_plan;

use super::scan::index_directory;
use super::{blocking, folder};

#[derive(Serialize)]
pub struct PlanReport {
    pub root: String,
    pub root_name: String,
    pub total_files: usize,
    pub analyzed: usize,
    pub ready: usize,
    pub review: usize,
    pub unchanged: usize,
    pub folders: Vec<FolderCount>,
    pub changes: Vec<PlannedChange>,
}

#[tauri::command]
pub async fn plan_folder(dir: String, state: State<'_, AppState>) -> Result<PlanReport> {
    let dir = folder(&dir)?;
    let repo = state.repository.clone();
    blocking(move || {
        let files = index_directory(&repo, &dir, |_, _| {})?;
        let r = repo.lock().unwrap();
        let settings = r.get_settings();
        let analyzed = files.iter().filter(|f| r.get_analysis(&f.id).is_some()).count();
        let root_name = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| dir.display().to_string());

        let (changes, unchanged) = if analyzed == 0 {
            (Vec::new(), 0)
        } else {
            let plan = create_organization_plan(
                &dir,
                &files,
                &r.get_all_analyses(),
                &settings.default_naming_convention,
                &settings.default_organization_behavior,
            );
            build_changes(&dir, &plan.items, &r)
        };
        let ready = changes.iter().filter(|c| !c.uncertain).count();
        Ok(PlanReport {
            root: dir.display().to_string(),
            root_name,
            total_files: files.len(),
            analyzed,
            ready,
            review: changes.len() - ready,
            unchanged,
            folders: folder_counts(&changes),
            changes,
        })
    })
    .await
}

#[derive(Serialize)]
pub struct ApplyResult {
    pub organized: usize,
}

#[tauri::command]
pub async fn apply_changes(changes: Vec<PlannedChange>, state: State<'_, AppState>) -> Result<ApplyResult> {
    if changes.is_empty() {
        return Ok(ApplyResult { organized: 0 });
    }
    let repo = state.repository.clone();
    blocking(move || {
        let actions: Vec<FileOperationAction> = changes
            .iter()
            .map(|c| FileOperationAction {
                file_id: c.file_id.clone(),
                source_path: c.from.clone(),
                destination_path: c.to.clone(),
                action_type: if Path::new(&c.from).parent() == Path::new(&c.to).parent() { "rename" } else { "rename_and_move" }
                    .into(),
            })
            .collect();

        let check = validate_operation_paths(&actions, &[]);
        if !check.valid {
            return Err(AppError::Operation(format!(
                "{} conflict(s) found, nothing was changed. Refresh the plan and try again. ({})",
                check.errors.len(),
                check.errors.first().cloned().unwrap_or_default()
            )));
        }
        // All-or-nothing: a failure part-way rolls back every move.
        let entry = execute_file_operations(&actions)?;
        repo.lock().unwrap().apply_operation_updates(&entry)?;
        Ok(ApplyResult { organized: entry.actions.len() })
    })
    .await
}

/// Ask where to save, then write the plan as JSON. Returns the saved path.
#[tauri::command]
pub async fn export_plan(changes: Vec<PlannedChange>, root_name: String) -> Result<Option<String>> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Export organization plan")
        .set_file_name(format!("zorter-plan-{root_name}.json"))
        .add_filter("JSON", &["json"])
        .save_file()
        .await;
    let Some(file) = file else { return Ok(None) };
    let path = file.path().to_path_buf();
    std::fs::write(&path, serde_json::to_string_pretty(&changes)?)?;
    Ok(Some(path.display().to_string()))
}

#[derive(Serialize)]
pub struct UndoResult {
    pub restored: usize,
}

#[tauri::command]
pub async fn undo_last(state: State<'_, AppState>) -> Result<UndoResult> {
    let repo = state.repository.clone();
    blocking(move || {
        let history = repo.lock().unwrap().get_operation_history()?;
        let op = find_undoable_operation(&history).ok_or_else(|| AppError::Operation("Nothing to undo.".into()))?;
        let entry = rollback_operation(&op)?;
        repo.lock().unwrap().apply_operation_updates(&entry)?;
        remove_empty_folders(&entry);
        Ok(UndoResult { restored: entry.actions.len() })
    })
    .await
}
