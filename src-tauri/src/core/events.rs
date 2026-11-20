use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgressEvent {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCompletedEvent {
    pub total_scanned: usize,
    pub new_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeProgressEvent {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeCompletedEvent {
    pub total_analyzed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchChangeEvent {
    pub path: String,
    pub change_type: String,
}
