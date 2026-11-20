use std::path::{Path, PathBuf};
use tauri::State;

use crate::core::app_state::AppState;
use crate::core::errors::{AppError, Result};
use crate::filesystem::operations::{
    execute_file_operations, validate_operation_paths, FileOperationAction,
    OperationJournalEntry,
};
use crate::organization::deduplication::{detect_duplicates, DuplicateGroup};
use crate::organization::folder_planner::{create_organization_plan, OrganizationPlan};

#[tauri::command]
pub async fn plan_organization(
    target_dir: Option<String>,
    naming_convention: Option<String>,
    organization_behavior: Option<String>,
    state: State<'_, AppState>,
) -> Result<OrganizationPlan> {
    let repo = state.repository.lock().unwrap();
    let settings = repo.get_settings();
    let files = repo.get_all_files();
    let analyses = repo.get_all_analyses();

    let root_path = match target_dir {
        Some(t) => PathBuf::from(t),
        None => {
            if let Some(first_file) = files.first() {
                Path::new(&first_file.current_path)
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf()
            } else {
                PathBuf::from(".")
            }
        }
    };

    let convention = naming_convention.unwrap_or(settings.default_naming_convention);
    let behavior = organization_behavior.unwrap_or(settings.default_organization_behavior);

    Ok(create_organization_plan(
        &root_path,
        &files,
        &analyses,
        &convention,
        &behavior,
    ))
}

#[tauri::command]
pub async fn execute_organization(
    actions: Vec<FileOperationAction>,
    state: State<'_, AppState>,
) -> Result<OperationJournalEntry> {
    let existing_files = {
        let repo = state.repository.lock().unwrap();
        repo.get_all_files()
    };

    // Step 1: Strict validation - never overwrite existing files!
    let validation = validate_operation_paths(&actions, &existing_files);
    if !validation.valid {
        return Err(AppError::Validation(format!(
            "Operation validation failed: {}",
            validation.errors.join("; ")
        )));
    }

    // Step 2: Safe execution with rollback on mid-operation failure
    let entry = execute_file_operations(&actions)?;

    // Step 3: Atomic journal record and repository path update
    {
        let mut repo = state.repository.lock().unwrap();
        repo.apply_operation_updates(&entry)?;
    }

    Ok(entry)
}

#[tauri::command]
pub async fn get_duplicate_files(state: State<'_, AppState>) -> Result<Vec<DuplicateGroup>> {
    let repo = state.repository.lock().unwrap();
    let files = repo.get_all_files();
    Ok(detect_duplicates(&files))
}
