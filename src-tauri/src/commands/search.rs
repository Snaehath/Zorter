//! Semantic Search screen.

use std::path::Path;

use serde::Serialize;
use tauri::State;

use crate::ai::provider::{model_installed, RuntimeProvider};
use crate::core::app_state::AppState;
use crate::core::errors::Result;
use crate::search::keyword::keyword_search;
use crate::search::semantic::SearchResultItem;

use super::blocking;

#[derive(Serialize)]
pub struct SearchHit {
    pub rank: usize,
    pub score: f32,
    pub name: String,
    pub path: String,
    pub folder: String,
    pub hash: String,
    pub description: String,
    pub category: String,
}

#[derive(Serialize)]
pub struct SearchResponse {
    /// "semantic" or "keyword"
    pub mode: &'static str,
    /// Why keyword mode was used, if it was.
    pub reason: Option<String>,
    pub model: String,
    pub total: usize,
    pub hits: Vec<SearchHit>,
}

/// Last two folders of the file's parent, e.g. `Wallpapers/Sci-Fi/`.
fn short_folder(path: &str) -> String {
    let mut parts: Vec<String> = Path::new(path)
        .parent()
        .map(|d| d.components().rev().take(2).map(|c| c.as_os_str().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    parts.reverse();
    format!("{}/", parts.join("/"))
}

#[tauri::command]
pub async fn search_files(query: String, limit: usize, state: State<'_, AppState>) -> Result<SearchResponse> {
    let repo = state.repository.clone();
    let ollama = state.ollama.clone();
    let engine = state.search_engine.clone();
    blocking(move || {
        let repo = repo.lock().unwrap();
        let model = repo.get_settings().preferred_embedding_model;

        let reason = if repo.get_all_embeddings().is_empty() {
            Some("No search index yet — analyze a folder first")
        } else if !ollama.health_check().unwrap_or(false) {
            Some("Ollama is offline")
        } else if !model_installed(&ollama.fetch_models().unwrap_or_default(), &model) {
            Some("Embedding model is not installed")
        } else {
            None
        };

        let semantic = match reason {
            None => engine.search(&repo, &model, &query, &Default::default(), 500).ok(),
            Some(_) => None,
        };
        let (mode, reason, items): (_, Option<String>, Vec<SearchResultItem>) = match semantic {
            Some(items) => ("semantic", None, items),
            None => (
                "keyword",
                Some(reason.unwrap_or("Semantic search failed").to_string()),
                keyword_search(&repo, &query),
            ),
        };

        let items: Vec<SearchResultItem> = items.into_iter().filter(|i| Path::new(&i.file.current_path).exists()).collect();
        let total = items.len();
        let hits = items
            .into_iter()
            .take(limit.max(1))
            .enumerate()
            .map(|(i, item)| SearchHit {
                rank: i + 1,
                score: item.score,
                folder: short_folder(&item.file.current_path),
                name: item.file.current_name,
                path: item.file.current_path,
                hash: item.file.hash,
                description: item.analysis.as_ref().map(|a| a.description.clone()).unwrap_or_default(),
                category: item.analysis.map(|a| format!("{} / {}", a.category, a.subcategory)).unwrap_or_default(),
            })
            .collect();
        Ok(SearchResponse { mode, reason, model, total, hits })
    })
    .await
}
