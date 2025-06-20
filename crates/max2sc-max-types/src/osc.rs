//! OSC configuration types

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCConfig {
    pub commands: Vec<OSCCommand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCCommand {
    pub address: String,
    pub args: Vec<OSCValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OSCValue {
    Float(f32),
    Double(f64),
    Int(i32),
    String(String),
    Bool(bool),
    List(Vec<OSCValue>),
}
