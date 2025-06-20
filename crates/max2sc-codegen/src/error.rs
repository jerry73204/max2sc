//! Code generation error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodegenError {
    #[error("Unsupported object type for generation: {0}")]
    UnsupportedObject(String),

    #[error("Invalid parameter mapping")]
    InvalidParameterMapping,

    #[error("Code generation failed: {0}")]
    GenerationFailed(String),
}
