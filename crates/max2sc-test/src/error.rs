//! Error types for the testing framework

use std::path::PathBuf;
use std::process::ExitStatus;
use thiserror::Error;

/// Test framework errors
#[derive(Debug, Error)]
pub enum TestError {
    /// SuperCollider executable not found
    #[error("SuperCollider executable not found at: {path}")]
    SclangNotFound { path: PathBuf },

    /// Failed to start SuperCollider process
    #[error("Failed to start sclang process: {0}")]
    ProcessSpawn(#[from] std::io::Error),

    /// SuperCollider process failed
    #[error("sclang process failed with status: {status:?}\nstdout: {stdout}\nstderr: {stderr}")]
    ProcessFailed {
        status: ExitStatus,
        stdout: String,
        stderr: String,
    },

    /// Compilation error
    #[error("Compilation failed:\n{errors}")]
    CompilationFailed { errors: String },

    /// Timeout error
    #[error("Test timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// Audio file error
    #[error("Audio file error: {0}")]
    AudioFile(String),

    /// Audio analysis error
    #[error("Audio analysis error: {0}")]
    AudioAnalysis(String),

    /// Assertion failed
    #[error("Assertion failed: {message}")]
    AssertionFailed { message: String },

    /// Test fixture error
    #[error("Test fixture error: {0}")]
    Fixture(String),

    /// Temporary file error
    #[error("Temporary file error: {0}")]
    TempFile(#[from] tempfile::PersistError),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Result type for test operations
pub type Result<T> = std::result::Result<T, TestError>;

impl TestError {
    /// Create a new Other error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Check if this is a compilation error
    pub fn is_compilation_error(&self) -> bool {
        matches!(self, Self::CompilationFailed { .. })
    }
}
