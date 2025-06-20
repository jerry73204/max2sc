//! Max patch format structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxPatch {
    pub patcher: Patcher,
    #[serde(default)]
    pub fileversion: i32,
    pub appversion: AppVersion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patcher {
    pub boxes: Vec<Box>,
    pub lines: Vec<Line>,
    #[serde(default)]
    pub parameters: Parameters,
    pub rect: [f32; 4],
    #[serde(default)]
    pub openinpresentation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
    pub architecture: String,
    pub modernui: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Parameters {
    // TODO: Define parameter structure
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Box {
    pub id: String,
    #[serde(rename = "box")]
    pub content: BoxContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxContent {
    pub maxclass: String,
    #[serde(default)]
    pub text: String,
    pub numinlets: u32,
    pub numoutlets: u32,
    pub patching_rect: [f32; 4],
    #[serde(default)]
    pub args: serde_json::Value,
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    pub patchline: PatchLine,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchLine {
    pub source: [String; 2],      // [object_id, outlet_index]
    pub destination: [String; 2], // [object_id, inlet_index]
}
