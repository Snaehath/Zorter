use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::ai::provider::EmbeddingProvider;
use crate::core::errors::Result;
use crate::search::filters::{matches_filters, SearchFilters};
use crate::search::vector_index::rank_by_similarity;
use crate::storage::repository::{FileAnalysis, IndexedFile, Repository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub file: IndexedFile,
    pub analysis: Option<FileAnalysis>,
    pub score: f32,
}

pub struct SemanticSearchEngine {
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl SemanticSearchEngine {
    pub fn new(embedding_provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { embedding_provider }
    }

    pub fn search(
        &self,
        repository: &Repository,
        model: &str,
        query: &str,
        filters: &SearchFilters,
        limit: usize,
    ) -> Result<Vec<SearchResultItem>> {
        let embeddings = repository.get_all_embeddings();
        let files = repository.get_all_files();
        let analyses = repository.get_all_analyses();

        let file_map: HashMap<String, IndexedFile> =
            files.into_iter().map(|f| (f.id.clone(), f)).collect();
        let analysis_map: HashMap<String, FileAnalysis> =
            analyses.into_iter().map(|a| (a.file_id.clone(), a)).collect();

        // If query is empty, do a metadata-only filter search
        if query.trim().is_empty() {
            let mut items = Vec::new();
            for file in file_map.values() {
                let analysis = analysis_map.get(&file.id);
                if matches_filters(file, analysis, filters) {
                    items.push(SearchResultItem {
                        file: file.clone(),
                        analysis: analysis.cloned(),
                        score: 1.0,
                    });
                }
            }
            items.truncate(limit);
            return Ok(items);
        }

        // Generate query embedding vector
        let query_vector = self.embedding_provider.embed_text(model, query)?;

        // Rank by cosine similarity
        let ranked = rank_by_similarity(&query_vector, &embeddings, 0.05);

        let mut results = Vec::new();
        for (file_id, score) in ranked {
            if let Some(file) = file_map.get(&file_id) {
                let analysis = analysis_map.get(&file_id);
                if matches_filters(file, analysis, filters) {
                    results.push(SearchResultItem {
                        file: file.clone(),
                        analysis: analysis.cloned(),
                        score,
                    });
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn find_similar(
        &self,
        repository: &Repository,
        target_file_id: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultItem>> {
        let embeddings = repository.get_all_embeddings();
        let files = repository.get_all_files();
        let analyses = repository.get_all_analyses();

        let target_vec = match embeddings.get(target_file_id) {
            Some(v) => v,
            None => return Ok(Vec::new()),
        };

        let file_map: HashMap<String, IndexedFile> =
            files.into_iter().map(|f| (f.id.clone(), f)).collect();
        let analysis_map: HashMap<String, FileAnalysis> =
            analyses.into_iter().map(|a| (a.file_id.clone(), a)).collect();

        let ranked = rank_by_similarity(target_vec, &embeddings, 0.1);

        let mut results = Vec::new();
        for (file_id, score) in ranked {
            if file_id == target_file_id {
                continue; // Exclude self
            }
            if let Some(file) = file_map.get(&file_id) {
                results.push(SearchResultItem {
                    file: file.clone(),
                    analysis: analysis_map.get(&file_id).cloned(),
                    score,
                });
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }
}
