use serde::{Deserialize, Serialize};
use crate::storage::repository::{FileAnalysis, IndexedFile};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub tag: Option<String>,
    pub visual_style: Option<String>,
    pub orientation: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

pub fn matches_filters(
    file: &IndexedFile,
    analysis: Option<&FileAnalysis>,
    filters: &SearchFilters,
) -> bool {
    if let Some(min) = filters.min_size {
        if file.size < min {
            return false;
        }
    }

    if let Some(max) = filters.max_size {
        if file.size > max {
            return false;
        }
    }

    if let Some(analysis) = analysis {
        if let Some(cat) = &filters.category {
            if !analysis.category.eq_ignore_ascii_case(cat) {
                return false;
            }
        }

        if let Some(subcat) = &filters.subcategory {
            if !analysis.subcategory.eq_ignore_ascii_case(subcat) {
                return false;
            }
        }

        if let Some(tag) = &filters.tag {
            let tag_lower = tag.to_lowercase();
            if !analysis.tags.iter().any(|t| t.to_lowercase() == tag_lower) {
                return false;
            }
        }

        if let Some(style) = &filters.visual_style {
            if !analysis.visual_style.eq_ignore_ascii_case(style) {
                return false;
            }
        }

        if let Some(orient) = &filters.orientation {
            if let Some(file_orient) = &analysis.orientation {
                if !file_orient.eq_ignore_ascii_case(orient) {
                    return false;
                }
            } else {
                return false;
            }
        }
    } else if filters.category.is_some()
        || filters.subcategory.is_some()
        || filters.tag.is_some()
        || filters.visual_style.is_some()
        || filters.orientation.is_some()
    {
        return false;
    }

    true
}
