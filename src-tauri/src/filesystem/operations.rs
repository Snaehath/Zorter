use std::fs;
use std::path::Path;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::errors::{AppError, Result};
use crate::storage::repository::IndexedFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationAction {
    pub file_id: String,
    pub source_path: String,
    pub destination_path: String,
    pub action_type: String, // "rename", "move", "rename_and_move"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationJournalEntry {
    pub id: String,
    pub timestamp: String,
    pub operation_type: String,
    pub status: String, // "completed", "rolled_back", "failed"
    pub actions: Vec<FileOperationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn validate_operation_paths(
    actions: &[FileOperationAction],
    _existing_files: &[IndexedFile],
) -> OperationValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut target_paths_set = std::collections::HashSet::new();

    for action in actions {
        let src = Path::new(&action.source_path);
        let dst = Path::new(&action.destination_path);

        if !src.exists() {
            errors.push(format!("Source file does not exist: {:?}", src));
            continue;
        }

        if src == dst {
            warnings.push(format!("Source and destination are identical: {:?}", src));
            continue;
        }

        // Never overwrite existing files!
        if dst.exists() {
            errors.push(format!(
                "Destination file already exists, overwriting forbidden: {:?}",
                dst
            ));
        }

        let dst_str = action.destination_path.to_lowercase();
        if target_paths_set.contains(&dst_str) {
            errors.push(format!(
                "Collision: Multiple files target same destination path: {:?}",
                dst
            ));
        } else {
            target_paths_set.insert(dst_str);
        }
    }

    OperationValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

pub fn execute_file_operations(
    actions: &[FileOperationAction],
) -> Result<OperationJournalEntry> {
    let mut executed_actions: Vec<FileOperationAction> = Vec::new();

    for action in actions {
        if let Err(e) = execute_single_action(action) {
            // All-or-nothing: undo every move already made in this batch so a
            // failure never leaves files moved without a journal entry.
            for prev in executed_actions.iter().rev() {
                let _ = move_file(Path::new(&prev.destination_path), Path::new(&prev.source_path));
            }
            return Err(e);
        }
        if action.source_path != action.destination_path {
            executed_actions.push(action.clone());
        }
    }

    let entry = OperationJournalEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        operation_type: "batch_organize".to_string(),
        status: "completed".to_string(),
        actions: executed_actions,
    };

    Ok(entry)
}

fn execute_single_action(action: &FileOperationAction) -> Result<()> {
    let src = Path::new(&action.source_path);
    let dst = Path::new(&action.destination_path);

    if src == dst {
        return Ok(());
    }

    if !src.exists() {
        return Err(AppError::Operation(format!(
            "Cannot perform operation, source not found: {:?}",
            src
        )));
    }

    if dst.exists() {
        return Err(AppError::Operation(format!(
            "Destination file already exists, aborting to prevent overwrite: {:?}",
            dst
        )));
    }

    if let Some(parent) = dst.parent()
        && !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Operation(format!(
                    "Failed to create target directory {:?}: {}",
                    parent, e
                ))
            })?;
        }

    move_file(src, dst)
}

/// Rename, falling back to copy + remove for cross-device moves.
fn move_file(src: &Path, dst: &Path) -> Result<()> {
    if let Err(e) = fs::rename(src, dst) {
        if let Err(copy_err) = fs::copy(src, dst) {
            return Err(AppError::Operation(format!(
                "Failed to move {:?} to {:?}: rename ({}) copy ({})",
                src, dst, e, copy_err
            )));
        }
        let _ = fs::remove_file(src);
    }
    Ok(())
}

pub fn rollback_operation(entry: &OperationJournalEntry) -> Result<OperationJournalEntry> {
    let mut rolled_back_actions = Vec::new();

    for action in entry.actions.iter().rev() {
        let curr = Path::new(&action.destination_path);
        let orig = Path::new(&action.source_path);

        if !curr.exists() {
            continue;
        }

        if orig.exists() {
            return Err(AppError::Operation(format!(
                "Rollback collision: Original path already occupied: {:?}",
                orig
            )));
        }

        if let Some(parent) = orig.parent()
            && !parent.exists() {
                fs::create_dir_all(parent).map_err(AppError::Io)?;
            }

        if let Err(e) = fs::rename(curr, orig) {
            if let Err(copy_err) = fs::copy(curr, orig) {
                return Err(AppError::Operation(format!(
                    "Failed to rollback {:?} to {:?}: rename ({}) copy ({})",
                    curr, orig, e, copy_err
                )));
            } else {
                let _ = fs::remove_file(curr);
            }
        }

        rolled_back_actions.push(action.clone());
    }

    Ok(OperationJournalEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        operation_type: format!("rollback:{}", entry.id),
        status: "rolled_back".to_string(),
        actions: rolled_back_actions,
    })
}

/// Most recent organization that has not been rolled back yet.
///
/// The journal is append-only, so an undone operation keeps its "completed"
/// entry; it is recognised by a later `rollback:<id>` entry.
pub fn find_undoable_operation(history: &[OperationJournalEntry]) -> Option<OperationJournalEntry> {
    let undone: std::collections::HashSet<&str> = history
        .iter()
        .filter_map(|e| e.operation_type.strip_prefix("rollback:"))
        .collect();
    history
        .iter()
        .rev()
        .find(|op| {
            op.status == "completed"
                && !op.operation_type.starts_with("rollback:")
                && !undone.contains(op.id.as_str())
                && !op.actions.is_empty()
        })
        .cloned()
}

/// After a rollback, remove the folders the organization created that are now
/// empty. Walks up from each destination folder and stops at the file's
/// original folder. `remove_dir` refuses non-empty directories, so nothing
/// with content is ever deleted.
pub fn remove_empty_folders(entry: &OperationJournalEntry) {
    for action in &entry.actions {
        let stop = Path::new(&action.source_path).parent();
        let mut dir = Path::new(&action.destination_path).parent();
        while let Some(d) = dir {
            if Some(d) == stop || stop.is_some_and(|s| !d.starts_with(s)) || fs::remove_dir(d).is_err() {
                break;
            }
            dir = d.parent();
        }
    }
}
