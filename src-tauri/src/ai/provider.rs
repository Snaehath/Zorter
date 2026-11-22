use serde::{Deserialize, Serialize};
use crate::core::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub is_vision: bool,
    pub is_embedding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysisResponse {
    pub description: String,
    pub subject: Vec<String>,
    pub category: String,
    pub subcategory: String,
    pub style: Vec<String>,
    pub environment: Option<String>,
    pub lighting: Option<String>,
    pub colors: Vec<String>,
    pub orientation: Option<String>,
    pub suggested_filename: String,
}

pub trait VisionProvider: Send + Sync {
    fn check_availability(&self) -> Result<bool>;
    fn list_models(&self) -> Result<Vec<ModelInfo>>;
    fn analyze_image(&self, model: &str, image_base64: &str) -> Result<VisionAnalysisResponse>;
}

pub trait EmbeddingProvider: Send + Sync {
    fn check_availability(&self) -> Result<bool>;
    fn list_models(&self) -> Result<Vec<ModelInfo>>;
    fn embed_text(&self, model: &str, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

pub trait RuntimeProvider: Send + Sync {
    fn health_check(&self) -> Result<bool>;
}

/// True when `wanted` is installed. A missing tag means `:latest`, matching
/// how Ollama resolves model names.
pub fn model_installed(models: &[ModelInfo], wanted: &str) -> bool {
    let norm = |s: &str| if s.contains(':') { s.to_string() } else { format!("{s}:latest") };
    let wanted = norm(wanted);
    models.iter().any(|m| norm(&m.name) == wanted)
}
