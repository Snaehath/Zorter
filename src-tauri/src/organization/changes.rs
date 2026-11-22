//! Turn an organization plan into concrete, reviewable file moves.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::organization::folder_planner::PlanItemPreview;
use crate::storage::repository::Repository;

pub const NEEDS_REVIEW: &str = "Needs-Review";

/// One proposed move. `uncertain` means the file has no AI analysis and is
/// routed to `<root>/Needs-Review/` for manual sorting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedChange {
    pub file_id: String,
    pub from: String,
    pub to: String,
    pub original_name: String,
    pub proposed_name: String,
    /// Destination folder relative to the root, `/`-separated ("." = root).
    pub folder: String,
    pub uncertain: bool,
    pub description: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderCount {
    pub name: String,
    pub count: usize,
}

/// Returns `(changes, already_in_place_count)`. Ready changes come first.
/// With `move_unanalyzed == false`, files without an analysis are left untouched
/// (and not counted as changes).
pub fn build_changes(
    root: &Path,
    items: &[PlanItemPreview],
    repo: &Repository,
    move_unanalyzed: bool,
) -> (Vec<PlannedChange>, usize) {
    let mut taken: HashSet<String> = items
        .iter()
        .filter(|i| i.is_analyzed)
        .map(|i| i.proposed_destination_path.to_lowercase())
        .collect();
    let review_dir = root.join(NEEDS_REVIEW);
    let mut changes = Vec::new();
    let mut unchanged = 0;

    for item in items {
        if !item.is_analyzed && !move_unanalyzed {
            continue;
        }
        let src = PathBuf::from(&item.original_path);
        let dest = if item.is_analyzed {
            PathBuf::from(&item.proposed_destination_path)
        } else if src.parent() == Some(review_dir.as_path()) {
            src.clone()
        } else {
            unique_path(&review_dir, &item.original_name, &mut taken)
        };

        if dest == src {
            unchanged += 1;
            continue;
        }

        let folder = dest
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| ".".into());

        changes.push(PlannedChange {
            file_id: item.file_id.clone(),
            from: item.original_path.clone(),
            to: dest.to_string_lossy().to_string(),
            original_name: item.original_name.clone(),
            proposed_name: dest.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            folder,
            uncertain: !item.is_analyzed,
            description: repo.get_analysis(&item.file_id).map(|a| a.description).unwrap_or_default(),
            hash: repo.get_file(&item.file_id).map(|f| f.hash).unwrap_or_default(),
        });
    }

    changes.sort_by(|a, b| {
        a.uncertain
            .cmp(&b.uncertain)
            .then(a.folder.cmp(&b.folder))
            .then(a.proposed_name.cmp(&b.proposed_name))
    });
    (changes, unchanged)
}

/// Top-level destination folders with file counts, largest first and
/// `Needs-Review` last.
pub fn folder_counts(changes: &[PlannedChange]) -> Vec<FolderCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for c in changes {
        let top = c.folder.split('/').next().unwrap_or(".").to_string();
        *counts.entry(top).or_default() += 1;
    }
    let mut out: Vec<FolderCount> = counts.into_iter().map(|(name, count)| FolderCount { name, count }).collect();
    out.sort_by(|a, b| {
        (a.name == NEEDS_REVIEW)
            .cmp(&(b.name == NEEDS_REVIEW))
            .then(b.count.cmp(&a.count))
            .then(a.name.cmp(&b.name))
    });
    out
}

fn unique_path(dir: &Path, name: &str, taken: &mut HashSet<String>) -> PathBuf {
    let p = Path::new(name);
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut candidate = dir.join(name);
    let mut n = 1;
    while candidate.exists() || taken.contains(&candidate.to_string_lossy().to_lowercase()) {
        candidate = dir.join(format!("{stem}_{n:03}{ext}"));
        n += 1;
    }
    taken.insert(candidate.to_string_lossy().to_lowercase());
    candidate
}
