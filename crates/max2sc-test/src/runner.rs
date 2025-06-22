//! Main test runner for SuperCollider integration tests

use crate::error::{Result, TestError};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::{Child, Command};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Test categories for different types of validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// Fast compilation tests
    Syntax,
    /// Object instantiation tests
    Functional,
    /// Audio output validation
    Audio,
    /// End-to-end conversion tests
    Integration,
}

/// Main test runner for SuperCollider integration
pub struct SCTestRunner {
    /// Path to sclang executable
    sclang_path: PathBuf,
    /// Server configuration options
    server_options: ServerOptions,
    /// Test timeout duration
    timeout: Duration,
    /// Temporary directory for test files
    temp_dir: TempDir,
}

/// SuperCollider server options
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Sample rate (default: 48000)
    pub sample_rate: u32,
    /// Block size (default: 64)
    pub block_size: u32,
    /// Number of output channels (default: 2)
    pub output_channels: u32,
    /// Number of input channels (default: 2)
    pub input_channels: u32,
    /// Memory size in MB (default: 8192)
    pub memory_size: u32,
    /// Number of audio buffers (default: 1024)
    pub num_buffers: u32,
    /// Use realtime priority (default: false for testing)
    pub realtime: bool,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            block_size: 64,
            output_channels: 2,
            input_channels: 2,
            memory_size: 8192,
            num_buffers: 1024,
            realtime: false,
        }
    }
}

/// Test result wrapper
#[derive(Debug)]
pub struct TestResult<T> {
    /// The actual result data
    pub data: T,
    /// Execution time
    pub duration: Duration,
    /// Test category
    pub category: TestCategory,
}

impl SCTestRunner {
    /// Create a new test runner with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(None, ServerOptions::default(), Duration::from_secs(30))
    }

    /// Create a test runner with custom configuration
    pub fn with_config(
        sclang_path: Option<PathBuf>,
        server_options: ServerOptions,
        timeout: Duration,
    ) -> Result<Self> {
        // Find sclang executable
        let sclang_path = if let Some(path) = sclang_path {
            path
        } else {
            Self::find_sclang()?
        };

        // Create temporary directory for test files
        let temp_dir = TempDir::new()
            .map_err(|e| TestError::other(format!("Failed to create temp dir: {}", e)))?;

        Ok(Self {
            sclang_path,
            server_options,
            timeout,
            temp_dir,
        })
    }

    /// Find sclang executable in common locations
    fn find_sclang() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(path) = std::env::var("SCLANG_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Common locations to check
        let common_paths = [
            "/usr/local/bin/sclang",
            "/usr/bin/sclang",
            "/opt/local/bin/sclang",
            "C:\\Program Files\\SuperCollider\\sclang.exe",
            "C:\\Program Files (x86)\\SuperCollider\\sclang.exe",
            "/Applications/SuperCollider.app/Contents/MacOS/sclang",
        ];

        for path in &common_paths {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try PATH
        if let Ok(output) = std::process::Command::new("which").arg("sclang").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(PathBuf::from(path));
            }
        }

        Err(TestError::SclangNotFound {
            path: PathBuf::from("sclang"),
        })
    }

    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a temporary file with the given content
    pub async fn create_temp_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self.temp_dir.path().join(name);
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| TestError::other(format!("Failed to write temp file: {}", e)))?;
        Ok(path)
    }

    /// Execute sclang with the given arguments
    pub async fn execute_sclang(&self, args: Vec<String>) -> Result<(String, String)> {
        let mut cmd = Command::new(&self.sclang_path);
        cmd.args(&args);

        // Set environment for non-interactive mode
        cmd.env("SC_JACK_DEFAULT_OUTPUTS", "system")
            .env("SC_JACK_DEFAULT_INPUTS", "system");

        debug!("Executing sclang with args: {:?}", args);

        let output = timeout(self.timeout, cmd.output())
            .await
            .map_err(|_| TestError::Timeout {
                seconds: self.timeout.as_secs(),
            })?
            .map_err(TestError::ProcessSpawn)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(TestError::ProcessFailed {
                status: output.status,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            });
        }

        Ok((stdout, stderr))
    }

    /// Start an sclang process (for interactive tests)
    pub async fn start_sclang_process(&self) -> Result<Child> {
        let mut cmd = Command::new(&self.sclang_path);

        // Add server options
        let server_config = self.generate_server_config();
        let config_file = self
            .create_temp_file("server_config.scd", &server_config)
            .await?;

        cmd.arg("-D") // Don't run startup file
            .arg(config_file);

        cmd.spawn().map_err(TestError::ProcessSpawn)
    }

    /// Generate server configuration code
    fn generate_server_config(&self) -> String {
        format!(
            r#"
Server.default.options.sampleRate = {};
Server.default.options.blockSize = {};
Server.default.options.numOutputBusChannels = {};
Server.default.options.numInputBusChannels = {};
Server.default.options.memSize = {} * 1024;
Server.default.options.numBuffers = {};

// Start server in testing mode
Server.default.waitForBoot {{
    "SuperCollider server booted for testing".postln;
    0.exit;  // Exit successfully after boot
}};
"#,
            self.server_options.sample_rate,
            self.server_options.block_size,
            self.server_options.output_channels,
            self.server_options.input_channels,
            self.server_options.memory_size,
            self.server_options.num_buffers,
        )
    }

    /// Run a specific test category
    pub async fn run_test<T>(
        &self,
        category: TestCategory,
        test_fn: impl std::future::Future<Output = Result<T>>,
    ) -> Result<TestResult<T>> {
        let start = std::time::Instant::now();

        info!("Running {:?} test", category);
        let data = test_fn.await?;
        let duration = start.elapsed();

        info!("Test completed in {:?}", duration);

        Ok(TestResult {
            data,
            duration,
            category,
        })
    }
}

impl ServerOptions {
    /// Create options for multichannel testing
    pub fn multichannel(channels: u32) -> Self {
        Self {
            output_channels: channels,
            input_channels: channels,
            ..Default::default()
        }
    }

    /// Create options for high-quality audio testing
    pub fn high_quality() -> Self {
        Self {
            sample_rate: 96000,
            block_size: 32,
            memory_size: 16384,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_options_default() {
        let opts = ServerOptions::default();
        assert_eq!(opts.sample_rate, 48000);
        assert_eq!(opts.output_channels, 2);
        assert!(!opts.realtime);
    }

    #[test]
    fn test_server_options_multichannel() {
        let opts = ServerOptions::multichannel(8);
        assert_eq!(opts.output_channels, 8);
        assert_eq!(opts.input_channels, 8);
    }
}
