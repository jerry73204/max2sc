//! Object mapping traits and types

use crate::error::Result;

/// Trait for converting Max objects to SuperCollider equivalents
pub trait MaxToSC {
    /// The SuperCollider output type
    type Output;

    /// Convert the Max object to SuperCollider representation
    fn to_sc(&self) -> Result<Self::Output>;
}

/// Represents a mapped SuperCollider object
#[derive(Debug, Clone)]
pub struct SCObject {
    /// The SC class or UGen name
    pub class_name: String,
    /// Arguments for the object
    pub args: Vec<SCValue>,
    /// Method (e.g., .ar, .kr)
    pub method: Option<String>,
    /// Additional properties
    pub properties: Vec<(String, SCValue)>,
}

/// SuperCollider value types
#[derive(Debug, Clone)]
pub enum SCValue {
    Float(f32),
    Int(i32),
    String(String),
    Symbol(String),
    Array(Vec<SCValue>),
    Object(Box<SCObject>),
}

impl SCValue {
    /// Convert to SC code string
    pub fn to_code(&self) -> String {
        match self {
            SCValue::Float(f) => f.to_string(),
            SCValue::Int(i) => i.to_string(),
            SCValue::String(s) => format!("\"{s}\""),
            SCValue::Symbol(s) => format!("\\{s}"),
            SCValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_code()).collect();
                format!("[{}]", items.join(", "))
            }
            SCValue::Object(obj) => obj.to_code(),
        }
    }
}

impl SCObject {
    /// Create a new SCObject
    pub fn new(class_name: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            args: Vec::new(),
            method: None,
            properties: Vec::new(),
        }
    }

    /// Set the method (e.g., .ar, .kr)
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Add an argument
    pub fn arg(mut self, value: impl Into<SCValue>) -> Self {
        self.args.push(value.into());
        self
    }

    /// Add a property
    pub fn prop(mut self, name: impl Into<String>, value: impl Into<SCValue>) -> Self {
        self.properties.push((name.into(), value.into()));
        self
    }

    /// Convert to SC code string
    pub fn to_code(&self) -> String {
        let mut code = self.class_name.clone();

        if let Some(method) = &self.method {
            code.push('.');
            code.push_str(method);
        }

        if !self.args.is_empty() {
            let args: Vec<String> = self.args.iter().map(|a| a.to_code()).collect();
            code.push('(');
            code.push_str(&args.join(", "));
            code.push(')');
        }

        for (name, value) in &self.properties {
            code.push('.');
            code.push_str(name);
            code.push('(');
            code.push_str(&value.to_code());
            code.push(')');
        }

        code
    }
}

// Implement From traits for convenience
impl From<f32> for SCValue {
    fn from(v: f32) -> Self {
        SCValue::Float(v)
    }
}

impl From<i32> for SCValue {
    fn from(v: i32) -> Self {
        SCValue::Int(v)
    }
}

impl From<&str> for SCValue {
    fn from(v: &str) -> Self {
        SCValue::String(v.to_string())
    }
}

impl From<String> for SCValue {
    fn from(v: String) -> Self {
        SCValue::String(v)
    }
}

impl From<Vec<SCValue>> for SCValue {
    fn from(v: Vec<SCValue>) -> Self {
        SCValue::Array(v)
    }
}

impl From<SCObject> for SCValue {
    fn from(v: SCObject) -> Self {
        SCValue::Object(Box::new(v))
    }
}

impl From<u32> for SCValue {
    fn from(v: u32) -> Self {
        SCValue::Int(v as i32)
    }
}

impl From<bool> for SCValue {
    fn from(v: bool) -> Self {
        SCValue::Int(if v { 1 } else { 0 })
    }
}
