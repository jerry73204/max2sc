//! Analyzer error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Invalid routing configuration")]
    InvalidRouting,

    #[error("Unsupported spatial configuration")]
    UnsupportedSpatial,
}
