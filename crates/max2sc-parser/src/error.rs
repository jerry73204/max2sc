//! Parser error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid Max patch format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported Max version: {version}")]
    UnsupportedVersion { version: String },

    #[error("Object not found: {name}")]
    ObjectNotFound { name: String },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
