//! Syntax validation tests for SuperCollider code

use crate::error::{Result, TestError};
use crate::runner::SCTestRunner;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

/// Syntax test configuration
#[derive(Debug, Clone)]
pub struct SyntaxTest {
    /// The SuperCollider code to compile
    pub code: String,
    /// Expected compilation result
    pub expected: CompileExpectation,
    /// Additional flags for sclang
    pub flags: Vec<String>,
}

/// Expected compilation outcome
#[derive(Debug, Clone)]
pub enum CompileExpectation {
    /// Should compile successfully
    Success,
    /// Should fail with specific error pattern
    Error(String),
    /// Should produce specific warning pattern
    Warning(String),
}

/// Compilation output information
#[derive(Debug)]
pub struct CompileOutput {
    /// Whether compilation succeeded
    pub success: bool,
    /// Compilation errors
    pub errors: Vec<CompileError>,
    /// Compilation warnings
    pub warnings: Vec<CompileWarning>,
    /// Raw stdout output
    pub stdout: String,
    /// Raw stderr output
    pub stderr: String,
}

/// Compilation error details
#[derive(Debug, Clone)]
pub struct CompileError {
    /// Error message
    pub message: String,
    /// File path (if available)
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<u32>,
    /// Column number (if available)
    pub column: Option<u32>,
    /// Error type
    pub error_type: String,
}

/// Compilation warning details
#[derive(Debug, Clone)]
pub struct CompileWarning {
    /// Warning message
    pub message: String,
    /// File path (if available)
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<u32>,
}

impl SyntaxTest {
    /// Create a new syntax test
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            expected: CompileExpectation::Success,
            flags: vec![],
        }
    }

    /// Set expected compilation outcome
    pub fn expect(mut self, expected: CompileExpectation) -> Self {
        self.expected = expected;
        self
    }

    /// Add compilation flags
    pub fn with_flags(mut self, flags: Vec<String>) -> Self {
        self.flags = flags;
        self
    }

    /// Run the syntax test
    pub async fn run(&self, runner: &SCTestRunner) -> Result<CompileOutput> {
        // Create temporary file with the code
        let file_path = runner
            .create_temp_file("test_syntax.scd", &self.code)
            .await?;

        // Build sclang arguments
        let mut args = vec![
            "-d".to_string(), // Just parse, don't execute
            file_path.to_string_lossy().to_string(),
        ];
        args.extend(self.flags.clone());

        // Execute sclang
        info!("Running syntax check on generated code");
        let (stdout, stderr) = match runner.execute_sclang(args).await {
            Ok((out, err)) => (out, err),
            Err(TestError::ProcessFailed { stdout, stderr, .. }) => {
                // Parse errors from failed compilation
                (stdout, stderr)
            }
            Err(e) => return Err(e),
        };

        // Parse the output
        let output = Self::parse_output(&stdout, &stderr);

        // Validate against expectations
        self.validate_output(&output)?;

        Ok(output)
    }

    /// Parse sclang output to extract errors and warnings
    fn parse_output(stdout: &str, stderr: &str) -> CompileOutput {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Common error patterns in sclang output
        let error_regex =
            Regex::new(r"ERROR:\s*(.+?)(?:\n\s+in file '(.+?)'\n\s+line (\d+) char (\d+))?")
                .unwrap();

        let warning_regex =
            Regex::new(r"WARNING:\s*(.+?)(?:\n\s+in file '(.+?)'\n\s+line (\d+))?").unwrap();

        // Parse errors from stderr
        for cap in error_regex.captures_iter(stderr) {
            let error = CompileError {
                message: cap.get(1).map_or("", |m| m.as_str()).to_string(),
                file: cap.get(2).map(|m| m.as_str().to_string()),
                line: cap.get(3).and_then(|m| m.as_str().parse().ok()),
                column: cap.get(4).and_then(|m| m.as_str().parse().ok()),
                error_type: Self::classify_error(&cap[1]),
            };
            errors.push(error);
        }

        // Parse warnings
        for cap in warning_regex.captures_iter(stderr) {
            let warning = CompileWarning {
                message: cap.get(1).map_or("", |m| m.as_str()).to_string(),
                file: cap.get(2).map(|m| m.as_str().to_string()),
                line: cap.get(3).and_then(|m| m.as_str().parse().ok()),
            };
            warnings.push(warning);
        }

        // Also check stdout for compilation messages
        if stdout.contains("ERROR:") || stderr.contains("ERROR:") {
            // Additional error parsing for different formats
            Self::parse_alternative_errors(stdout, stderr, &mut errors);
        }

        CompileOutput {
            success: errors.is_empty() && !stderr.contains("ERROR:"),
            errors,
            warnings,
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        }
    }

    /// Parse alternative error formats
    fn parse_alternative_errors(stdout: &str, stderr: &str, errors: &mut Vec<CompileError>) {
        // Parse syntax errors like "Syntax Error: unexpected token"
        let syntax_error_regex = Regex::new(r"Syntax Error:\s*(.+)").unwrap();

        for text in [stdout, stderr] {
            for cap in syntax_error_regex.captures_iter(text) {
                if !errors.iter().any(|e| e.message.contains(&cap[1])) {
                    errors.push(CompileError {
                        message: cap[1].to_string(),
                        file: None,
                        line: None,
                        column: None,
                        error_type: "SyntaxError".to_string(),
                    });
                }
            }
        }
    }

    /// Classify error type based on message
    fn classify_error(message: &str) -> String {
        if message.contains("Syntax Error") || message.contains("unexpected") {
            "SyntaxError".to_string()
        } else if message.contains("Class not found") || message.contains("undefined") {
            "UndefinedError".to_string()
        } else if message.contains("argument") || message.contains("parameter") {
            "ArgumentError".to_string()
        } else if message.contains("type") {
            "TypeError".to_string()
        } else {
            "CompileError".to_string()
        }
    }

    /// Validate output against expectations
    fn validate_output(&self, output: &CompileOutput) -> Result<()> {
        match &self.expected {
            CompileExpectation::Success => {
                if !output.success {
                    let errors = output
                        .errors
                        .iter()
                        .map(|e| format!("  - {}: {}", e.error_type, e.message))
                        .collect::<Vec<_>>()
                        .join("\n");
                    return Err(TestError::CompilationFailed { errors });
                }
            }
            CompileExpectation::Error(pattern) => {
                if output.success {
                    return Err(TestError::AssertionFailed {
                        message: format!(
                            "Expected compilation to fail with pattern '{}', but it succeeded",
                            pattern
                        ),
                    });
                }

                let regex = Regex::new(pattern)
                    .map_err(|e| TestError::other(format!("Invalid error pattern: {}", e)))?;

                if !output.errors.iter().any(|e| regex.is_match(&e.message)) {
                    return Err(TestError::AssertionFailed {
                        message: format!("No error matched pattern '{}'", pattern),
                    });
                }
            }
            CompileExpectation::Warning(pattern) => {
                let regex = Regex::new(pattern)
                    .map_err(|e| TestError::other(format!("Invalid warning pattern: {}", e)))?;

                if !output.warnings.iter().any(|w| regex.is_match(&w.message)) {
                    return Err(TestError::AssertionFailed {
                        message: format!("No warning matched pattern '{}'", pattern),
                    });
                }
            }
        }

        Ok(())
    }
}

impl CompileOutput {
    /// Check if compilation was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the first error (if any)
    pub fn first_error(&self) -> Option<&CompileError> {
        self.errors.first()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Get all error messages
    pub fn error_messages(&self) -> Vec<String> {
        self.errors.iter().map(|e| e.message.clone()).collect()
    }
}

/// Helper to create a syntax test that expects success
pub fn expect_compile_success(code: &str) -> SyntaxTest {
    SyntaxTest::new(code).expect(CompileExpectation::Success)
}

/// Helper to create a syntax test that expects an error
pub fn expect_compile_error(code: &str, pattern: &str) -> SyntaxTest {
    SyntaxTest::new(code).expect(CompileExpectation::Error(pattern.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        assert_eq!(
            SyntaxTest::classify_error("Syntax Error: unexpected token"),
            "SyntaxError"
        );
        assert_eq!(
            SyntaxTest::classify_error("Class not found: UnknownClass"),
            "UndefinedError"
        );
        assert_eq!(
            SyntaxTest::classify_error("Wrong number of arguments"),
            "ArgumentError"
        );
    }

    #[test]
    fn test_compile_output_helpers() {
        let output = CompileOutput {
            success: false,
            errors: vec![CompileError {
                message: "Test error".to_string(),
                file: None,
                line: None,
                column: None,
                error_type: "TestError".to_string(),
            }],
            warnings: vec![],
            stdout: String::new(),
            stderr: String::new(),
        };

        assert!(!output.is_success());
        assert_eq!(output.error_count(), 1);
        assert_eq!(output.warning_count(), 0);
        assert_eq!(output.error_messages(), vec!["Test error"]);
    }
}
