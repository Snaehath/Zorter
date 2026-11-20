use std::collections::HashSet;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::organization::classifier::clean_category_name;
use crate::organization::naming::generate_deterministic_filename;
use crate::storage::repository::{FileAnalysis, IndexedFile};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItemPreview {
    pub file_id: String,
    pub original_name: String,
    pub original_path: String,
    pub proposed_name: String,
    pub proposed_destination_path: String,
    pub category: String,
    pub subcategory: String,
    pub action_type: String,
    pub thumbnail_url: Option<String>,
    pub included: bool,
    pub is_analyzed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPlan {
    pub items: Vec<PlanItemPreview>,
    pub total_files: usize,
    pub destination_root: String,
    pub unanalyzed_count: usize,
}

pub fn create_organization_plan(
    target_root: &Path,
    files: &[IndexedFile],
    analyses: &[FileAnalysis],
    naming_convention: &str,
    organization_behavior: &str,
) -> OrganizationPlan {
    let analysis_map: std::collections::HashMap<String, &FileAnalysis> =
        analyses.iter().map(|a| (a.file_id.clone(), a)).collect();

    let mut items = Vec::new();
    let mut reserved_paths: HashSet<String> = HashSet::new();
    let mut unanalyzed_count = 0;

    for file in files {
        let analysis_opt = analysis_map.get(&file.id);
        let is_analyzed = analysis_opt.is_some();
        if !is_analyzed {
            unanalyzed_count += 1;
        }

        let orig_path = Path::new(&file.current_path);
        let ext = orig_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");

        let (category, subcategory, proposed_name) = if let Some(analysis) = analysis_opt {
            let cat = clean_category_name(&analysis.category);
            let subcat = clean_category_name(&analysis.subcategory);
            let name = generate_deterministic_filename(analysis, ext, naming_convention);
            (cat, subcat, name)
        } else {
            // Unanalyzed file fallback: clean random hashes/numbers if present
            ("Uncategorized".to_string(), "General".to_string(), file.current_name.clone())
        };

        // Determine destination folder based on organization behavior
        let dest_folder = match organization_behavior {
            "flat_rename" => target_root.to_path_buf(),
            "category_and_style" => {
                let style = analysis_opt
                    .and_then(|a| a.tags.first())
                    .map(|s| clean_category_name(s))
                    .unwrap_or_else(|| "General".to_string());
                target_root.join(&category).join(&style)
            }
            _ => target_root.join(&category).join(&subcategory), // "by_category"
        };

        // Collision detection and unique safe name assignment
        let mut candidate_name = proposed_name.clone();
        let mut candidate_path = dest_folder.join(&candidate_name);
        let mut counter = 1;

        let stem = Path::new(&proposed_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");

        while candidate_path.exists() || reserved_paths.contains(&candidate_path.to_string_lossy().to_string().to_lowercase()) {
            // If identical to current original file, no collision with self
            if candidate_path == orig_path {
                break;
            }
            candidate_name = format!("{}_{:03}.{}", stem, counter, ext);
            candidate_path = dest_folder.join(&candidate_name);
            counter += 1;
        }

        let proposed_dest_str = candidate_path.to_string_lossy().to_string();
        reserved_paths.insert(proposed_dest_str.to_lowercase());

        let action_type = if dest_folder == orig_path.parent().unwrap_or(target_root) {
            "rename".to_string()
        } else {
            "rename_and_move".to_string()
        };

        items.push(PlanItemPreview {
            file_id: file.id.clone(),
            original_name: file.current_name.clone(),
            original_path: file.current_path.clone(),
            proposed_name: candidate_name,
            proposed_destination_path: proposed_dest_str,
            category,
            subcategory,
            action_type,
            thumbnail_url: file.thumbnail_url.clone(),
            included: true,
            is_analyzed,
        });
    }

    let total_files = items.len();
    OrganizationPlan {
        items,
        total_files,
        destination_root: target_root.to_string_lossy().to_string(),
        unanalyzed_count,
    }
}
