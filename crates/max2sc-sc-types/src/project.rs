//! SuperCollider project structure

use crate::{BusConfig, OSCResponder, Pattern, SynthDef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SCProject {
    pub main_file: String,
    pub synth_defs: Vec<SynthDef>,
    pub patterns: Vec<Pattern>,
    pub bus_config: BusConfig,
    pub osc_responders: Vec<OSCResponder>,
    pub init_code: String,
    pub cleanup_code: Option<String>,
}
