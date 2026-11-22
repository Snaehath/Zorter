//! Keyword fallback used when semantic search is unavailable
//! (no embeddings yet, Ollama offline, or the embedding model is missing).

use std::path::Path;

use crate::search::semantic::SearchResultItem;
use crate::storage::repository::Repository;

pub fn keyword_search(repo: &Repository, query: &str) -> Vec<SearchResultItem> {
    let q = query.to_lowercase();
    let tokens: Vec<&str> = q.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }
    let mut results: Vec<SearchResultItem> = repo
        .get_all_analyses()
        .into_iter()
        .filter_map(|a| {
            let file = repo.get_file(&a.file_id)?;
            if !Path::new(&file.current_path).exists() {
                return None;
            }
            let text = format!(
                "{} {} {} {} {} {} {}",
                a.description,
                a.category,
                a.subcategory,
                a.tags.join(" "),
                a.subject.join(" "),
                a.colors.join(" "),
                file.current_name
            )
            .to_lowercase();
            let hits = tokens.iter().filter(|t| text.contains(**t)).count();
            (hits > 0).then(|| SearchResultItem {
                score: hits as f32 / tokens.len() as f32,
                file,
                analysis: Some(a),
            })
        })
        .collect();
    results.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.file.current_name.cmp(&b.file.current_name)));
    results
}
