use tauri::State;

use crate::core::app_state::AppState;
use crate::core::errors::{AppError, Result};
use crate::filesystem::operations::{rollback_operation, OperationJournalEntry};

#[tauri::command]
pub async fn get_operation_history(
    state: State<'_, AppState>,
) -> Result<Vec<OperationJournalEntry>> {
    let repo = state.repository.lock().unwrap();
    repo.get_operation_history()
}

#[tauri::command]
pub async fn undo_last_operation(
    state: State<'_, AppState>,
) -> Result<OperationJournalEntry> {
    let history = {
        let repo = state.repository.lock().unwrap();
        repo.get_operation_history()?
    };

    // Find the last completed operation that hasn't been rolled back yet
    let last_op = history
        .iter()
        .rev()
        .find(|op| op.status == "completed" && !op.operation_type.starts_with("rollback:"))
        .cloned()
        .ok_or_else(|| AppError::Operation("No completed operations found to undo".to_string()))?;

    // Perform filesystem rollback
    let rollback_entry = rollback_operation(&last_op)?;

    // Apply updates to repository and record in journal
    {
        let mut repo = state.repository.lock().unwrap();
        repo.apply_operation_updates(&rollback_entry)?;
    }

    Ok(rollback_entry)
}
