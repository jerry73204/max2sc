//! SynthDef structure definitions

use crate::{Rate, UGen};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SynthDef {
    pub name: String,
    pub params: Vec<Parameter>,
    pub ugens: Vec<UGen>,
    pub variants: Option<Vec<Variant>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub default_value: f32,
    pub rate: Rate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variant {
    // TODO: Define variant structure when needed
}
