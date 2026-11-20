use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use image::imageops::FilterType;
use tokio::sync::Semaphore;

use crate::ai::provider::VisionProvider;
use crate::core::errors::{AppError, Result};
use crate::storage::repository::{FileAnalysis, IndexedFile};

pub struct VisionCoordinator {
    provider: Arc<dyn VisionProvider>,
    concurrency_limit: Arc<Semaphore>,
}

impl VisionCoordinator {
    pub fn new(provider: Arc<dyn VisionProvider>, concurrency: usize) -> Self {
        let concurrency = if concurrency == 0 { 1 } else { concurrency };
        Self {
            provider,
            concurrency_limit: Arc::new(Semaphore::new(concurrency)),
        }
    }

    /// Prepares image for local vision model: if dimensions exceed 1024x1024,
    /// resize safely to conserve VRAM (crucial for RTX 3050 4GB laptop GPUs),
    /// and encode to standard base64 string.
    pub fn prepare_image_base64(&self, image_path: &Path) -> Result<String> {
        let img = image::open(image_path)
            .map_err(|e| AppError::Image(format!("Failed to open image for analysis: {}", e)))?;

        // If greater than 1024 on any dimension, downscale with aspect ratio preserved
        let (w, h) = (img.width(), img.height());
        let resized = if w > 1024 || h > 1024 {
            img.resize(1024, 1024, FilterType::Triangle)
        } else {
            img
        };

        let mut buf = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .map_err(|e| AppError::Image(format!("Failed to encode image to JPEG buffer: {}", e)))?;

        Ok(STANDARD.encode(&buf))
    }

    pub async fn analyze_file(
        &self,
        file: &IndexedFile,
        model: &str,
    ) -> Result<FileAnalysis> {
        let _permit = self.concurrency_limit.acquire().await.map_err(|e| {
            AppError::Generic(format!("Semaphore acquisition failed: {}", e))
        })?;

        let path = Path::new(&file.current_path);
        let b64 = self.prepare_image_base64(path)?;

        let resp = self.provider.analyze_image(model, &b64)?;

        Ok(FileAnalysis {
            file_id: file.id.clone(),
            description: resp.description,
            subject: resp.subject,
            category: resp.category,
            subcategory: resp.subcategory,
            tags: resp.style.clone(),
            visual_style: resp.style.first().cloned().unwrap_or_else(|| "Digital".to_string()),
            environment: resp.environment,
            lighting: resp.lighting,
            colors: resp.colors,
            orientation: resp.orientation,
            suggested_filename: resp.suggested_filename,
            model: model.to_string(),
            analyzed_at: Utc::now().to_rfc3339(),
        })
    }
}
