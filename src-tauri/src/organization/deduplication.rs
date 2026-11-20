use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::storage::repository::IndexedFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub total_size: u64,
    pub wasted_bytes: u64,
    pub files: Vec<IndexedFile>,
}

pub fn detect_duplicates(files: &[IndexedFile]) -> Vec<DuplicateGroup> {
    let mut by_hash: HashMap<String, Vec<IndexedFile>> = HashMap::new();

    for f in files {
        by_hash
            .entry(f.hash.clone())
            .or_default()
            .push(f.clone());
    }

    let mut duplicates = Vec::new();
    for (hash, group) in by_hash {
        if group.len() > 1 {
            let single_size = group.first().map(|f| f.size).unwrap_or(0);
            let wasted = single_size * (group.len() as u64 - 1);
            let total = single_size * (group.len() as u64);

            duplicates.push(DuplicateGroup {
                hash,
                total_size: total,
                wasted_bytes: wasted,
                files: group,
            });
        }
    }

    // Sort by wasted bytes descending
    duplicates.sort_by(|a, b| b.wasted_bytes.cmp(&a.wasted_bytes));
    duplicates
}
