//! Payloads of events emitted to the frontend during long-running jobs.

use serde::Serialize;

use crate::storage::repository::FileAnalysis;

/// `scan:progress` and `analyze:indexing`
#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub current: usize,
    pub total: usize,
}

/// `analyze:started`
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeStartedEvent {
    pub total: usize,
    pub already_analyzed: usize,
    pub model: String,
    pub search_model: Option<String>,
}

/// `analyze:file` — one per processed file, success or failure.
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzeFileEvent {
    pub index: usize,
    pub total: usize,
    pub file_id: String,
    pub name: String,
    pub path: String,
    pub hash: String,
    pub analysis: Option<FileAnalysis>,
    pub proposed_name: Option<String>,
    pub error: Option<String>,
}
