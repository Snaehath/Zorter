use tauri::State;

use crate::core::app_state::AppState;
use crate::core::errors::Result;
use crate::search::filters::SearchFilters;
use crate::search::semantic::SearchResultItem;

#[tauri::command]
pub async fn search_semantic(
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultItem>> {
    let (repo_guard, preferred_model) = {
        let repo = state.repository.lock().unwrap();
        let model = repo.get_settings().preferred_embedding_model;
        (repo, model)
    };

    let active_filters = filters.unwrap_or_default();
    let active_limit = limit.unwrap_or(50);

    state.search_engine.search(
        &*repo_guard,
        &preferred_model,
        &query,
        &active_filters,
        active_limit,
    )
}

#[tauri::command]
pub async fn find_similar_images(
    file_id: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResultItem>> {
    let repo = state.repository.lock().unwrap();
    let active_limit = limit.unwrap_or(20);
    state.search_engine.find_similar(&*repo, &file_id, active_limit)
}
