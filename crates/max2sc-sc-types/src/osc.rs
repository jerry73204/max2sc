//! OSC responder definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCResponder {
    pub address: String,
    pub params: Vec<OSCParam>,
    pub action: String, // SC code as string
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSCParam {
    pub name: String,
    pub param_type: OSCParamType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OSCParamType {
    Float,
    Int,
    String,
    Symbol,
}
