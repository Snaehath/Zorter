use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Ollama API error: {0}")]
    Ollama(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Image processing error: {0}")]
    Image(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File operation error: {0}")]
    Operation(String),

    #[error("Job cancelled")]
    Cancelled,

    #[error("Generic error: {0}")]
    Generic(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
