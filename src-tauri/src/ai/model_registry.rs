use std::sync::Arc;
use crate::ai::provider::{ModelInfo, VisionProvider, EmbeddingProvider};
use crate::core::errors::Result;

pub struct ModelRegistry {
    vision_provider: Arc<dyn VisionProvider>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl ModelRegistry {
    pub fn new(
        vision_provider: Arc<dyn VisionProvider>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            vision_provider,
            embedding_provider,
        }
    }

    pub fn get_available_vision_models(&self) -> Result<Vec<ModelInfo>> {
        self.vision_provider.list_models()
    }

    pub fn get_available_embedding_models(&self) -> Result<Vec<ModelInfo>> {
        self.embedding_provider.list_models()
    }
}
