//! Cross-platform comparison between Max MSP and SuperCollider outputs

use crate::audio::{AudioComparison, AudioTolerance};
use crate::error::{Result, TestError};
use crate::runner::SCTestRunner;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Configuration for Max2SC comparison tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    /// Audio comparison tolerances
    pub audio_tolerance: AudioTolerance,
    /// Timing tolerance in milliseconds
    pub timing_tolerance_ms: f32,
    /// OSC message tolerance
    pub osc_tolerance: OscTolerance,
    /// Test duration
    pub test_duration: Duration,
}

/// OSC message comparison tolerances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscTolerance {
    /// Time difference tolerance (seconds)
    pub time_tolerance: f32,
    /// Value difference tolerance (percentage)
    pub value_tolerance: f32,
    /// Allow missing messages ratio
    pub missing_tolerance: f32,
}

/// Result of Max/SC comparison
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Test name
    pub test_name: String,
    /// Whether the test passed overall
    pub passed: bool,
    /// Audio similarity score (0-1)
    pub audio_similarity: f32,
    /// Timing accuracy score (0-1)
    pub timing_accuracy: f32,
    /// OSC message accuracy score (0-1)
    pub osc_accuracy: f32,
    /// List of differences found
    pub differences: Vec<String>,
    /// Execution time
    pub execution_time: Duration,
}

/// Max MSP runner using WINE
pub struct MaxRunner {
    /// WINE prefix path
    wine_prefix: Option<PathBuf>,
    /// Max MSP executable path
    max_executable: PathBuf,
    /// Temporary directory
    temp_dir: TempDir,
    /// Whether to run in headless mode
    headless: bool,
}

/// Main comparison test runner
pub struct Max2SCComparison {
    /// Max MSP runner
    max_runner: MaxRunner,
    /// SuperCollider runner
    sc_runner: SCTestRunner,
    /// Comparison configuration
    config: ComparisonConfig,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            audio_tolerance: AudioTolerance::default(),
            timing_tolerance_ms: 10.0,
            osc_tolerance: OscTolerance::default(),
            test_duration: Duration::from_secs(10),
        }
    }
}

impl Default for OscTolerance {
    fn default() -> Self {
        Self {
            time_tolerance: 0.01,    // 10ms
            value_tolerance: 0.02,   // 2%
            missing_tolerance: 0.05, // 5% missing messages OK
        }
    }
}

impl MaxRunner {
    /// Create a new Max runner
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()
            .map_err(|e| TestError::other(format!("Failed to create temp dir: {e}")))?;

        // Try to find Max MSP executable
        let max_executable = Self::find_max_executable()?;

        Ok(Self {
            wine_prefix: Self::find_wine_prefix(),
            max_executable,
            temp_dir,
            headless: true,
        })
    }

    /// Find Max MSP executable
    fn find_max_executable() -> Result<PathBuf> {
        // Check environment variable first
        if let Ok(path) = std::env::var("MAX_MSP_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Common Max MSP locations in WINE
        let wine_prefix = Self::find_wine_prefix()
            .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap()).join(".wine"));

        let common_paths = [
            "drive_c/Program Files/Cycling '74/Max 8/Max.exe",
            "drive_c/Program Files (x86)/Cycling '74/Max 8/Max.exe",
            "drive_c/Program Files/Max/Max.exe",
        ];

        for path in &common_paths {
            let full_path = wine_prefix.join(path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        Err(TestError::other("Max MSP executable not found"))
    }

    /// Find WINE prefix
    fn find_wine_prefix() -> Option<PathBuf> {
        if let Ok(prefix) = std::env::var("WINEPREFIX") {
            Some(PathBuf::from(prefix))
        } else {
            None
        }
    }

    /// Create a recording patch that loads the target patch and records output
    fn create_recording_patch(
        &self,
        target_patch: &Path,
        duration: Duration,
        output_file: &Path,
    ) -> Result<PathBuf> {
        let patch_content = format!(
            r#"{{
    "patcher": {{
        "boxes": [
            {{
                "box": {{
                    "maxclass": "newobj",
                    "text": "pcontrol",
                    "patching_rect": [10, 10, 100, 22]
                }}
            }},
            {{
                "box": {{
                    "maxclass": "message",
                    "text": "load {}",
                    "patching_rect": [10, 40, 200, 22]
                }}
            }},
            {{
                "box": {{
                    "maxclass": "newobj",
                    "text": "sfrecord~ 2",
                    "patching_rect": [10, 100, 100, 22]
                }}
            }},
            {{
                "box": {{
                    "maxclass": "message",
                    "text": "open {}",
                    "patching_rect": [10, 130, 200, 22]
                }}
            }},
            {{
                "box": {{
                    "maxclass": "newobj",
                    "text": "delay {}",
                    "patching_rect": [10, 160, 100, 22]
                }}
            }},
            {{
                "box": {{
                    "maxclass": "message",
                    "text": "stop",
                    "patching_rect": [10, 190, 50, 22]
                }}
            }},
            {{
                "box": {{
                    "maxclass": "newobj",
                    "text": "quitout",
                    "patching_rect": [10, 220, 50, 22]
                }}
            }}
        ],
        "lines": [
            {{ "patchline": {{ "source": [0, 0], "destination": [1, 0] }} }},
            {{ "patchline": {{ "source": [3, 0], "destination": [2, 0] }} }},
            {{ "patchline": {{ "source": [4, 0], "destination": [5, 0] }} }},
            {{ "patchline": {{ "source": [5, 0], "destination": [2, 0] }} }},
            {{ "patchline": {{ "source": [5, 0], "destination": [6, 0] }} }}
        ]
    }}
}}"#,
            target_patch.display(),
            output_file.display(),
            duration.as_millis()
        );

        let patch_path = self.temp_dir.path().join("recording_patch.maxpat");
        std::fs::write(&patch_path, patch_content)
            .map_err(|e| TestError::other(format!("Failed to write patch: {e}")))?;

        Ok(patch_path)
    }

    /// Render a Max patch to audio file
    pub async fn render_to_file(
        &self,
        patch: &Path,
        duration: Duration,
    ) -> Result<PathBuf> {
        let output_file = self.temp_dir.path().join("max_output.wav");
        let recording_patch = self.create_recording_patch(patch, duration, &output_file)?;

        info!("Rendering Max patch to file: {}", output_file.display());

        let mut cmd = Command::new("wine");

        // Set WINE environment
        if let Some(prefix) = &self.wine_prefix {
            cmd.env("WINEPREFIX", prefix);
        }

        cmd.env("WINEDEBUG", "-all"); // Reduce WINE debug output

        if self.headless {
            cmd.env("DISPLAY", ":99"); // Virtual display
        }

        cmd.arg(&self.max_executable)
            .arg("-nogui")
            .arg("-nosplash")
            .arg(recording_patch);

        debug!("Executing Max MSP: {:?}", cmd);

        let output = cmd.output().await.map_err(|e| {
            TestError::other(format!("Failed to execute Max MSP: {e}"))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TestError::other(format!(
                "Max MSP execution failed: {stderr}"
            )));
        }

        // Wait for output file to be created
        tokio::time::sleep(Duration::from_millis(100)).await;

        if !output_file.exists() {
            return Err(TestError::other(
                "Max MSP did not create output file".to_string(),
            ));
        }

        Ok(output_file)
    }
}

impl Max2SCComparison {
    /// Create a new comparison runner
    pub fn new(config: ComparisonConfig) -> Result<Self> {
        Ok(Self {
            max_runner: MaxRunner::new()?,
            sc_runner: SCTestRunner::new()?,
            config,
        })
    }

    /// Run a comparison test between Max patch and SC code
    pub async fn run_comparison(
        &self,
        test_name: &str,
        max_patch: &Path,
        sc_code: &str,
    ) -> Result<ComparisonResult> {
        let start = std::time::Instant::now();

        info!("Running comparison test: {}", test_name);

        // 1. Render Max MSP output
        let max_output = match self
            .max_runner
            .render_to_file(max_patch, self.config.test_duration)
            .await
        {
            Ok(output) => output,
            Err(e) => {
                warn!("Max MSP rendering failed: {}", e);
                return Ok(ComparisonResult {
                    test_name: test_name.to_string(),
                    passed: false,
                    audio_similarity: 0.0,
                    timing_accuracy: 0.0,
                    osc_accuracy: 0.0,
                    differences: vec![format!("Max MSP rendering failed: {e}")],
                    execution_time: start.elapsed(),
                });
            }
        };

        // 2. Render SuperCollider output
        let sc_output = self.render_sc_to_file(sc_code).await?;

        // 3. Compare audio outputs
        let audio_comparison = self.compare_audio(&max_output, &sc_output)?;

        // 4. Compare timing
        let timing_accuracy = self.compare_timing(&max_output, &sc_output)?;

        // 5. Determine overall result
        let passed = audio_comparison >= self.config.audio_tolerance.rms_tolerance as f32
            && timing_accuracy >= (1.0 - self.config.timing_tolerance_ms / 1000.0);

        let mut differences = Vec::new();
        if (audio_comparison as f64) < self.config.audio_tolerance.rms_tolerance {
            differences.push(format!(
                "Audio similarity {} below threshold {}",
                audio_comparison, self.config.audio_tolerance.rms_tolerance
            ));
        }

        Ok(ComparisonResult {
            test_name: test_name.to_string(),
            passed,
            audio_similarity: audio_comparison,
            timing_accuracy,
            osc_accuracy: 1.0, // TODO: Implement OSC comparison
            differences,
            execution_time: start.elapsed(),
        })
    }

    /// Render SuperCollider code to file
    async fn render_sc_to_file(&self, sc_code: &str) -> Result<PathBuf> {
        let output_file = PathBuf::from("/tmp/sc_comparison_output.wav");

        let recording_code = format!(
            r#"
// Boot server and record
Server.default.waitForBoot {{
    s.record("{}");
    
    // Execute user code
    {};
    
    // Stop recording after duration
    SystemClock.sched({}, {{
        s.stopRecording;
        0.exit;
    }});
}};
"#,
            output_file.display(),
            sc_code,
            self.config.test_duration.as_secs_f32()
        );

        self.sc_runner
            .execute_sclang(vec!["-e".to_string(), recording_code])
            .await?;

        Ok(output_file)
    }

    /// Compare audio files
    fn compare_audio(&self, file1: &Path, file2: &Path) -> Result<f32> {
        let comparison = AudioComparison::new(file1, file2)?;
        let result = comparison.compare(self.config.audio_tolerance.clone())?;
        Ok(result.similarity)
    }

    /// Compare timing characteristics
    fn compare_timing(&self, _file1: &Path, _file2: &Path) -> Result<f32> {
        // TODO: Implement onset detection and timing comparison
        // For now, return perfect timing
        Ok(1.0)
    }

    /// Generate comparison report
    pub fn generate_report(&self, results: &[ComparisonResult]) -> String {
        let mut report = String::new();
        
        report.push_str("# Max2SC Comparison Report\n\n");
        
        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();
        
        report.push_str(&format!("## Summary\n"));
        report.push_str(&format!("- Total tests: {}\n", total));
        report.push_str(&format!("- Passed: {} ({:.1}%)\n", passed, (passed as f32 / total as f32) * 100.0));
        report.push_str(&format!("- Failed: {}\n\n", total - passed));
        
        report.push_str("## Detailed Results\n\n");
        
        for result in results {
            report.push_str(&format!("### {}\n", result.test_name));
            report.push_str(&format!("- Status: {}\n", if result.passed { "✅ PASS" } else { "❌ FAIL" }));
            report.push_str(&format!("- Audio similarity: {:.1}%\n", result.audio_similarity * 100.0));
            report.push_str(&format!("- Timing accuracy: {:.1}%\n", result.timing_accuracy * 100.0));
            report.push_str(&format!("- Execution time: {:.2}s\n", result.execution_time.as_secs_f32()));
            
            if !result.differences.is_empty() {
                report.push_str("\n**Issues found:**\n");
                for diff in &result.differences {
                    report.push_str(&format!("- {}\n", diff));
                }
            }
            
            report.push('\n');
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_config_default() {
        let config = ComparisonConfig::default();
        assert!(config.timing_tolerance_ms > 0.0);
        assert!(config.test_duration.as_secs() > 0);
    }

    #[test]
    fn test_osc_tolerance_default() {
        let tolerance = OscTolerance::default();
        assert!(tolerance.time_tolerance > 0.0);
        assert!(tolerance.value_tolerance > 0.0);
        assert!(tolerance.missing_tolerance > 0.0);
    }

    #[test]
    fn test_comparison_result_creation() {
        let result = ComparisonResult {
            test_name: "test".to_string(),
            passed: true,
            audio_similarity: 0.95,
            timing_accuracy: 0.98,
            osc_accuracy: 0.99,
            differences: vec![],
            execution_time: Duration::from_secs(1),
        };

        assert!(result.passed);
        assert_eq!(result.test_name, "test");
    }
}