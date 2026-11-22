//! Fast file-type census of a directory (no hashing or metadata reads).

use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;

use crate::filesystem::metadata::is_supported_image;

const DOCUMENT_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "txt", "md", "rtf", "odt", "xls", "xlsx", "csv", "ppt", "pptx", "epub",
];

#[derive(Debug, Default, Clone, Serialize)]
pub struct Census {
    pub total: usize,
    pub images: usize,
    pub documents: usize,
    pub other: usize,
}

/// Walk `dir` with the same skip rules as the scanner (hidden entries,
/// `node_modules`, `AppData`) so the counts agree with what gets indexed.
pub fn walk_files(dir: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            e.depth() == 0 || (!name.starts_with('.') && name != "node_modules" && name != "AppData")
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
}

pub fn census(dir: &Path) -> Census {
    let mut c = Census::default();
    for path in walk_files(dir) {
        c.total += 1;
        if is_supported_image(&path) {
            c.images += 1;
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| DOCUMENT_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        {
            c.documents += 1;
        } else {
            c.other += 1;
        }
    }
    c
}
