//! UGen representation types

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UGen {
    pub name: String,
    pub rate: Rate,
    pub inputs: Vec<UGenInput>,
    pub outputs: Vec<UGenOutput>,
    pub special_index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UGenInput {
    Constant(f32),
    Parameter(String),
    UGen {
        ugen_index: usize,
        output_index: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UGenOutput {
    pub rate: Rate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Rate {
    Audio,
    Control,
    Scalar,
    Demand,
}
