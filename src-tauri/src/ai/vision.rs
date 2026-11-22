use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use image::imageops::FilterType;

use crate::ai::provider::VisionProvider;
use crate::core::errors::{AppError, Result};
use crate::storage::repository::{FileAnalysis, IndexedFile};

pub struct VisionCoordinator {
    provider: Arc<dyn VisionProvider>,
}

impl VisionCoordinator {
    pub fn new(provider: Arc<dyn VisionProvider>) -> Self {
        Self { provider }
    }

    /// Prepares image for local vision model: if dimensions exceed 768x768,
    /// resize safely to conserve VRAM (crucial for RTX 3050 4GB laptop GPUs),
    /// and encode to standard base64 string.
    pub fn prepare_image_base64(&self, image_path: &Path) -> Result<String> {
        let img = image::open(image_path)
            .map_err(|e| AppError::Image(format!("Failed to open image for analysis: {}", e)))?;

        // If greater than 768 on any dimension, downscale with aspect ratio preserved
        let (w, h) = (img.width(), img.height());
        let resized = if w > 768 || h > 768 {
            img.resize(768, 768, FilterType::Triangle)
        } else {
            img
        };

        let mut buf = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
            .map_err(|e| AppError::Image(format!("Failed to encode image to JPEG buffer: {}", e)))?;

        Ok(STANDARD.encode(&buf))
    }

    pub fn analyze_file_sync(
        &self,
        file: &IndexedFile,
        model: &str,
    ) -> Result<FileAnalysis> {
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
