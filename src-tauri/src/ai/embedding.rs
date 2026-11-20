use std::sync::Arc;
use crate::ai::provider::EmbeddingProvider;
use crate::core::errors::Result;
use crate::storage::repository::FileAnalysis;

pub struct EmbeddingCoordinator {
    provider: Arc<dyn EmbeddingProvider>,
}

impl EmbeddingCoordinator {
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { provider }
    }

    /// Construct a rich semantic description text from structured analysis
    pub fn build_semantic_text(analysis: &FileAnalysis) -> String {
        let subject_str = analysis.subject.join(", ");
        let style_str = analysis.tags.join(", ");
        let color_str = analysis.colors.join(", ");
        let env_str = analysis.environment.as_deref().unwrap_or("general scene");
        let light_str = analysis.lighting.as_deref().unwrap_or("natural");

        format!(
            "{} in {} style. Subject: {}. Category: {} / {}. Setting: {}. Lighting: {}. Colors: {}. Description: {}",
            analysis.visual_style,
            style_str,
            subject_str,
            analysis.category,
            analysis.subcategory,
            env_str,
            light_str,
            color_str,
            analysis.description
        )
    }

    pub fn generate_embedding(&self, model: &str, text: &str) -> Result<Vec<f32>> {
        self.provider.embed_text(model, text)
    }

    pub fn generate_analysis_embedding(
        &self,
        model: &str,
        analysis: &FileAnalysis,
    ) -> Result<Vec<f32>> {
        let text = Self::build_semantic_text(analysis);
        self.provider.embed_text(model, &text)
    }
}
