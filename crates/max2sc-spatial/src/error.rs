//! Spatial audio error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpatialError {
    #[error("Invalid speaker configuration")]
    InvalidSpeakerConfig,

    #[error("Unsupported spatial algorithm: {0}")]
    UnsupportedAlgorithm(String),

    #[error("Parameter conversion failed")]
    ConversionFailed,
}
