//! Common error types for max2sc

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Unsupported object type: {0}")]
    UnsupportedObject(String),

    #[error("Invalid parameter range: {name} = {value}")]
    InvalidParameter { name: String, value: f32 },

    #[error("Missing required attribute: {0}")]
    MissingAttribute(String),
}

/// Result type alias for max2sc-core
pub type Result<T> = std::result::Result<T, ConversionError>;
