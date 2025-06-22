//! Functional tests for SuperCollider object instantiation and behavior

use crate::assertions::{Assertion, AssertionResult};
use crate::error::{Result, TestError};
use crate::runner::SCTestRunner;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info};

/// Functional test configuration
#[derive(Debug)]
pub struct FunctionalTest {
    /// Setup code to run before the test
    pub setup: String,
    /// Test code to execute
    pub test_code: String,
    /// Cleanup code to run after test
    pub cleanup: String,
    /// Assertions to validate
    pub assertions: Vec<Assertion>,
    /// Test timeout
    pub timeout: Duration,
}

/// Functional test output
#[derive(Debug)]
pub struct FunctionalOutput {
    /// Whether all assertions passed
    pub success: bool,
    /// Assertion results
    pub assertion_results: Vec<AssertionResult>,
    /// Server output/logs
    pub server_output: String,
    /// Execution time
    pub execution_time: Duration,
    /// Any errors encountered
    pub errors: Vec<String>,
}

impl FunctionalTest {
    /// Create a new functional test
    pub fn new(test_code: impl Into<String>) -> Self {
        Self {
            setup: String::new(),
            test_code: test_code.into(),
            cleanup: String::new(),
            assertions: Vec::new(),
            timeout: Duration::from_secs(10),
        }
    }

    /// Add setup code
    pub fn with_setup(mut self, setup: impl Into<String>) -> Self {
        self.setup = setup.into();
        self
    }

    /// Add cleanup code
    pub fn with_cleanup(mut self, cleanup: impl Into<String>) -> Self {
        self.cleanup = cleanup.into();
        self
    }

    /// Add an assertion
    pub fn assert(mut self, assertion: Assertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Add multiple assertions
    pub fn assert_all(mut self, assertions: Vec<Assertion>) -> Self {
        self.assertions.extend(assertions);
        self
    }

    /// Set test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Run the functional test
    pub async fn run(&self, runner: &SCTestRunner) -> Result<FunctionalOutput> {
        let start = std::time::Instant::now();

        // Generate the complete test script
        let script = self.generate_test_script();
        let script_path = runner
            .create_temp_file("functional_test.scd", &script)
            .await?;

        info!("Running functional test");
        debug!("Test script: {}", script);

        // Execute the test with timeout
        let execution_result = timeout(
            self.timeout,
            runner.execute_sclang(vec![script_path.to_string_lossy().to_string()]),
        )
        .await;

        let (stdout, stderr) = match execution_result {
            Ok(result) => result?,
            Err(_) => {
                return Ok(FunctionalOutput {
                    success: false,
                    assertion_results: vec![],
                    server_output: String::new(),
                    execution_time: start.elapsed(),
                    errors: vec![format!("Test timed out after {}s", self.timeout.as_secs())],
                });
            }
        };

        // Parse assertion results from output
        let assertion_results = self.parse_assertion_results(&stdout)?;

        // Check for any errors in stderr
        let errors = self.parse_errors(&stderr);

        let success = assertion_results.iter().all(|r| r.passed) && errors.is_empty();

        Ok(FunctionalOutput {
            success,
            assertion_results,
            server_output: stdout,
            execution_time: start.elapsed(),
            errors,
        })
    }

    /// Generate the complete SuperCollider test script
    fn generate_test_script(&self) -> String {
        let mut script = String::new();

        // Add server boot and wait
        script.push_str(&format!(
            r#"
// Boot server and wait
Server.default.waitForBoot {{
    var testResults = ();
    
    // Setup phase
    try {{
        {}
        "Setup completed".postln;
    }} {{ |error|
        ("Setup failed: " ++ error.errorString).postln;
        0.exit;
    }};

    // Test execution phase
    try {{
        {}
        "Test execution completed".postln;
    }} {{ |error|
        ("Test execution failed: " ++ error.errorString).postln;
        0.exit;
    }};

    // Assertion phase
"#,
            self.setup, self.test_code
        ));

        // Add assertion checks
        for (i, assertion) in self.assertions.iter().enumerate() {
            script.push_str(&format!(
                r#"
    // Assertion {}: {}
    try {{
        var result = {};
        var passed = {};
        testResults["assertion_{}"] = (
            description: "{}",
            passed: passed,
            result: result
        );
        ("ASSERTION_{}: " ++ passed ++ " - {}").postln;
    }} {{ |error|
        testResults["assertion_{}"] = (
            description: "{}",
            passed: false,
            error: error.errorString
        );
        ("ASSERTION_{}: false - Error: " ++ error.errorString).postln;
    }};
"#,
                i,
                assertion.description(),
                assertion.generate_sc_code(),
                assertion.generate_validation_code(),
                i,
                assertion.description(),
                i,
                assertion.description(),
                i,
                assertion.description(),
                i
            ));
        }

        // Add cleanup and exit
        script.push_str(&format!(
            r#"
    // Cleanup phase
    try {{
        {}
        "Cleanup completed".postln;
    }} {{ |error|
        ("Cleanup failed: " ++ error.errorString).postln;
    }};

    // Print summary
    testResults.keysValuesDo {{ |key, value|
        ("RESULT: " ++ key ++ " = " ++ value.asCompileString).postln;
    }};

    "All tests completed".postln;
    0.exit;
}};
"#,
            self.cleanup
        ));

        script
    }

    /// Parse assertion results from SuperCollider output
    fn parse_assertion_results(&self, output: &str) -> Result<Vec<AssertionResult>> {
        let mut results = Vec::new();

        for (i, assertion) in self.assertions.iter().enumerate() {
            let assertion_marker = format!("ASSERTION_{}:", i);

            if let Some(line) = output.lines().find(|line| line.contains(&assertion_marker)) {
                let passed = line.contains("true");
                let message = if passed {
                    "Assertion passed".to_string()
                } else {
                    // Extract error message if present
                    line.split(" - ")
                        .nth(1)
                        .unwrap_or("Assertion failed")
                        .to_string()
                };

                results.push(AssertionResult {
                    assertion: assertion.clone(),
                    passed,
                    message,
                    actual_value: None, // Could be enhanced to parse actual values
                });
            } else {
                // Assertion not found in output - assume it failed
                results.push(AssertionResult {
                    assertion: assertion.clone(),
                    passed: false,
                    message: "Assertion not executed".to_string(),
                    actual_value: None,
                });
            }
        }

        Ok(results)
    }

    /// Parse errors from stderr
    fn parse_errors(&self, stderr: &str) -> Vec<String> {
        let mut errors = Vec::new();

        for line in stderr.lines() {
            if line.contains("ERROR:") || line.contains("Exception:") {
                errors.push(line.to_string());
            }
        }

        errors
    }
}

impl FunctionalOutput {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.success
    }

    /// Get failed assertions
    pub fn failed_assertions(&self) -> Vec<&AssertionResult> {
        self.assertion_results
            .iter()
            .filter(|r| !r.passed)
            .collect()
    }

    /// Get passed assertions
    pub fn passed_assertions(&self) -> Vec<&AssertionResult> {
        self.assertion_results.iter().filter(|r| r.passed).collect()
    }

    /// Get assertion count
    pub fn assertion_count(&self) -> usize {
        self.assertion_results.len()
    }

    /// Get pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.assertion_results.is_empty() {
            return 100.0;
        }

        let passed = self.passed_assertions().len() as f64;
        let total = self.assertion_results.len() as f64;
        (passed / total) * 100.0
    }
}

/// Helper functions for common functional tests

/// Test that an object can be instantiated
pub fn test_object_instantiation(class_name: &str) -> FunctionalTest {
    FunctionalTest::new(format!("~testObj = {}.ar;", class_name))
        .assert(Assertion::not_nil("~testObj"))
}

/// Test that an object responds to a method
pub fn test_method_response(object_code: &str, method: &str) -> FunctionalTest {
    FunctionalTest::new(format!(
        "~testObj = {}; ~result = ~testObj.{};",
        object_code, method
    ))
    .assert(Assertion::not_nil("~result"))
}

/// Test OSC responder functionality
pub fn test_osc_responder(osc_path: &str, test_value: f32) -> FunctionalTest {
    FunctionalTest::new(format!(
        r#"
~responder = OSCdef(\testResponder, {{|msg|
    ~receivedValue = msg[1];
    "OSC received".postln;
}}, '{}');

// Send test message
NetAddr.localAddr.sendMsg('{}', {});
0.1.wait; // Give time for message to be processed
"#,
        osc_path, osc_path, test_value
    ))
    .assert(Assertion::equals("~receivedValue", test_value.into()))
    .with_cleanup("~responder.free;".to_string())
}

/// Test multichannel output
pub fn test_multichannel_output(object_code: &str, expected_channels: u32) -> FunctionalTest {
    FunctionalTest::new(format!(
        "~testObj = {}; ~channels = ~testObj.numChannels;",
        object_code
    ))
    .assert(Assertion::equals(
        "~channels",
        (expected_channels as i32).into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assertions::Assertion;

    #[test]
    fn test_functional_test_creation() {
        let test = FunctionalTest::new("SinOsc.ar(440)")
            .with_setup("Server.default.boot;")
            .assert(Assertion::not_nil("SinOsc"))
            .with_cleanup("Server.default.quit;");

        assert_eq!(test.test_code, "SinOsc.ar(440)");
        assert_eq!(test.assertions.len(), 1);
        assert!(!test.setup.is_empty());
        assert!(!test.cleanup.is_empty());
    }

    #[test]
    fn test_functional_output_metrics() {
        let output = FunctionalOutput {
            success: false,
            assertion_results: vec![
                AssertionResult {
                    assertion: Assertion::not_nil("test"),
                    passed: true,
                    message: "OK".to_string(),
                    actual_value: None,
                },
                AssertionResult {
                    assertion: Assertion::not_nil("test2"),
                    passed: false,
                    message: "Failed".to_string(),
                    actual_value: None,
                },
            ],
            server_output: String::new(),
            execution_time: Duration::from_millis(100),
            errors: vec![],
        };

        assert_eq!(output.assertion_count(), 2);
        assert_eq!(output.passed_assertions().len(), 1);
        assert_eq!(output.failed_assertions().len(), 1);
        assert_eq!(output.pass_rate(), 50.0);
    }
}
