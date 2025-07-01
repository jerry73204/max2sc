//! Assertion framework for SuperCollider tests

use serde_json::Value;

/// Test assertion
#[derive(Debug, Clone)]
pub enum Assertion {
    /// Assert that a variable/object exists and is not nil
    NotNil(String),
    /// Assert that a variable equals a specific value
    Equals(String, Value),
    /// Assert that a numeric value is within tolerance
    Approximately(String, f64, f64),
    /// Assert that an object responds to a method
    RespondsTo(String, String),
    /// Assert that an object has a specific number of channels
    ChannelCount(String, u32),
    /// Assert that an OSC responder exists for a path
    OscResponder(String),
    /// Assert that a condition evaluates to true
    Condition(String, String),
    /// Custom assertion with SuperCollider code
    Custom(String, String),
    /// Assert that the output contains a specific string
    ContainsOutput(String),
}

/// Result of an assertion
#[derive(Debug, Clone)]
pub struct AssertionResult {
    /// The assertion that was tested
    pub assertion: Assertion,
    /// Whether the assertion passed
    pub passed: bool,
    /// Message describing the result
    pub message: String,
    /// Actual value (if captured)
    pub actual_value: Option<Value>,
}

impl Assertion {
    /// Create a not-nil assertion
    pub fn not_nil(variable: impl Into<String>) -> Self {
        Self::NotNil(variable.into())
    }

    /// Create an equals assertion
    pub fn equals(variable: impl Into<String>, value: Value) -> Self {
        Self::Equals(variable.into(), value)
    }

    /// Create an approximately equals assertion for floats
    pub fn approximately(variable: impl Into<String>, expected: f64, tolerance: f64) -> Self {
        Self::Approximately(variable.into(), expected, tolerance)
    }

    /// Create a responds-to assertion
    pub fn responds_to(object: impl Into<String>, method: impl Into<String>) -> Self {
        Self::RespondsTo(object.into(), method.into())
    }

    /// Create a channel count assertion
    pub fn channel_count(object: impl Into<String>, channels: u32) -> Self {
        Self::ChannelCount(object.into(), channels)
    }

    /// Create an OSC responder assertion
    pub fn osc_responder(path: impl Into<String>) -> Self {
        Self::OscResponder(path.into())
    }

    /// Create a custom condition assertion
    pub fn condition(description: impl Into<String>, condition: impl Into<String>) -> Self {
        Self::Condition(description.into(), condition.into())
    }

    /// Create a custom assertion with SuperCollider code
    pub fn custom(description: impl Into<String>, sc_code: impl Into<String>) -> Self {
        Self::Custom(description.into(), sc_code.into())
    }

    /// Assert that output contains a specific string
    pub fn contains_output(expected: impl Into<String>) -> Self {
        Self::ContainsOutput(expected.into())
    }

    /// Get assertion description
    pub fn description(&self) -> String {
        match self {
            Self::NotNil(var) => format!("{var} is not nil"),
            Self::Equals(var, value) => format!("{var} equals {value}"),
            Self::Approximately(var, expected, tolerance) => {
                format!("{var} approximately equals {expected} (±{tolerance})")
            }
            Self::RespondsTo(obj, method) => format!("{obj} responds to {method}"),
            Self::ChannelCount(obj, channels) => format!("{obj} has {channels} channels"),
            Self::OscResponder(path) => format!("OSC responder exists for {path}"),
            Self::Condition(desc, _) => desc.clone(),
            Self::Custom(desc, _) => desc.clone(),
            Self::ContainsOutput(expected) => format!("output contains '{expected}'"),
        }
    }

    /// Generate SuperCollider code to evaluate the assertion
    pub fn generate_sc_code(&self) -> String {
        match self {
            Self::NotNil(var) => format!("{var}.notNil"),
            Self::Equals(var, value) => {
                let sc_value = self.value_to_sc_literal(value);
                format!("{var} == {sc_value}")
            }
            Self::Approximately(var, expected, tolerance) => {
                format!("({var} - {expected}).abs <= {tolerance}")
            }
            Self::RespondsTo(obj, method) => {
                format!("{obj}.respondsTo(\\{method})")
            }
            Self::ChannelCount(obj, channels) => {
                format!("{obj}.numChannels == {channels}")
            }
            Self::OscResponder(path) => {
                format!("OSCdef.all[\\{}].notNil", self.path_to_symbol(path))
            }
            Self::Condition(_, condition) => condition.clone(),
            Self::Custom(_, code) => code.clone(),
            Self::ContainsOutput(_) => "true".to_string(), // Handled externally by checking stdout
        }
    }

    /// Generate validation code (for functional tests)
    pub fn generate_validation_code(&self) -> String {
        // Most assertions are simple boolean expressions
        // This can be extended for more complex validations
        "result".to_string()
    }

    /// Convert JSON value to SuperCollider literal
    #[allow(clippy::only_used_in_recursion)]
    fn value_to_sc_literal(&self, value: &Value) -> String {
        match value {
            Value::Null => "nil".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i.to_string()
                } else if let Some(f) = n.as_f64() {
                    f.to_string()
                } else {
                    "0".to_string()
                }
            }
            Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.value_to_sc_literal(v)).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(_) => "()".to_string(), // Empty event for objects
        }
    }

    /// Convert OSC path to SuperCollider symbol
    fn path_to_symbol(&self, path: &str) -> String {
        path.replace(['/', '.'], "_")
    }
}

// Helper functions for common assertions

/// Assert that an object exists
pub fn object_exists(name: &str) -> Assertion {
    Assertion::not_nil(name)
}

/// Assert that an object responds to OSC messages
pub fn responds_to_osc(path: &str) -> Assertion {
    Assertion::osc_responder(path)
}

/// Assert specific output channel count
pub fn output_channels(count: u32) -> Assertion {
    Assertion::custom(
        format!("Output has {count} channels"),
        format!("Server.default.options.numOutputBusChannels >= {count}"),
    )
}

/// Assert that a value is within a range
pub fn in_range(variable: &str, min: f64, max: f64) -> Assertion {
    Assertion::condition(
        format!("{variable} is between {min} and {max}"),
        format!("({variable} >= {min}) && ({variable} <= {max})"),
    )
}

/// Assert that an audio signal is not silent
pub fn not_silent(variable: &str) -> Assertion {
    Assertion::condition(
        format!("{variable} is not silent"),
        format!("{variable}.squared.sum > 0.001"),
    )
}

/// Assert that two audio signals are similar
pub fn signals_similar(sig1: &str, sig2: &str, tolerance: f64) -> Assertion {
    Assertion::condition(
        format!("{sig1} and {sig2} are similar"),
        format!("({sig1} - {sig2}).squared.sum.sqrt < {tolerance}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_assertion_descriptions() {
        assert_eq!(
            Assertion::not_nil("testVar").description(),
            "testVar is not nil"
        );

        assert_eq!(
            Assertion::equals("x", json!(42)).description(),
            "x equals 42"
        );

        assert_eq!(
            Assertion::approximately("freq", 440.0, 1.0).description(),
            "freq approximately equals 440 (±1)"
        );
    }

    #[test]
    fn test_sc_code_generation() {
        assert_eq!(Assertion::not_nil("test").generate_sc_code(), "test.notNil");

        assert_eq!(
            Assertion::equals("x", json!(42)).generate_sc_code(),
            "x == 42"
        );

        assert_eq!(
            Assertion::responds_to("obj", "play").generate_sc_code(),
            "obj.respondsTo(\\play)"
        );
    }

    #[test]
    fn test_value_to_sc_literal() {
        let assertion = Assertion::not_nil("test");

        assert_eq!(assertion.value_to_sc_literal(&json!(null)), "nil");
        assert_eq!(assertion.value_to_sc_literal(&json!(true)), "true");
        assert_eq!(assertion.value_to_sc_literal(&json!(42)), "42");
        assert_eq!(assertion.value_to_sc_literal(&json!(3.15)), "3.15");
        assert_eq!(assertion.value_to_sc_literal(&json!("hello")), "\"hello\"");
        assert_eq!(
            assertion.value_to_sc_literal(&json!([1, 2, 3])),
            "[1, 2, 3]"
        );
    }

    #[test]
    fn test_helper_functions() {
        let assertion = object_exists("myObj");
        assert_eq!(assertion.description(), "myObj is not nil");

        let assertion = responds_to_osc("/source/1/azimuth");
        assert_eq!(
            assertion.description(),
            "OSC responder exists for /source/1/azimuth"
        );

        let assertion = output_channels(8);
        assert_eq!(assertion.description(), "Output has 8 channels");
    }
}
