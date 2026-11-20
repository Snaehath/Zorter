use std::fs;
use std::path::{Path, PathBuf};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::imageops::FilterType;
use crate::core::errors::{AppError, Result};

pub fn get_thumbnail_path(cache_dir: &Path, file_hash: &str) -> PathBuf {
    cache_dir.join("thumbnails").join(format!("{}.jpg", file_hash))
}

pub fn generate_thumbnail(
    image_path: &Path,
    cache_dir: &Path,
    file_hash: &str,
    max_dim: u32,
) -> Result<PathBuf> {
    let thumbs_dir = cache_dir.join("thumbnails");
    if !thumbs_dir.exists() {
        fs::create_dir_all(&thumbs_dir).map_err(AppError::Io)?;
    }

    let out_path = thumbs_dir.join(format!("{}.jpg", file_hash));
    if out_path.exists() {
        return Ok(out_path);
    }

    let img = image::open(image_path)
        .map_err(|e| AppError::Image(format!("Failed to open image for thumbnail: {}", e)))?;

    let thumb = img.resize(max_dim, max_dim, FilterType::Triangle);
    thumb
        .save(&out_path)
        .map_err(|e| AppError::Image(format!("Failed to save thumbnail: {}", e)))?;

    Ok(out_path)
}

pub fn thumbnail_to_data_url(thumb_path: &Path) -> Result<String> {
    if !thumb_path.exists() {
        return Err(AppError::Storage(format!("Thumbnail not found: {:?}", thumb_path)));
    }
    let bytes = fs::read(thumb_path).map_err(AppError::Io)?;
    let b64 = STANDARD.encode(&bytes);
    Ok(format!("data:image/jpeg;base64,{}", b64))
}
