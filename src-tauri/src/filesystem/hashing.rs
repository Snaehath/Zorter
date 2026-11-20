use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use blake3::Hasher;
use crate::core::errors::{AppError, Result};

/// Calculate BLAKE3 hash of a file efficiently using streaming buffers
pub fn calculate_blake3_hash(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(AppError::Io)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Hasher::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(AppError::Io)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}
