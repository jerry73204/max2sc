//! Max patch format structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxPatch {
    pub patcher: Patcher,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Patcher {
    #[serde(default)]
    pub fileversion: i32,
    #[serde(default)]
    pub appversion: Option<AppVersion>,
    #[serde(default)]
    pub classnamespace: Option<String>,
    pub rect: [f32; 4],
    #[serde(default)]
    pub bglocked: i32,
    #[serde(default)]
    pub openinpresentation: i32,
    #[serde(default)]
    pub default_fontsize: f32,
    #[serde(default)]
    pub default_fontface: i32,
    #[serde(default)]
    pub default_fontname: Option<String>,
    #[serde(default)]
    pub gridonopen: i32,
    #[serde(default)]
    pub gridsize: Option<[f32; 2]>,
    #[serde(default)]
    pub gridsnaponopen: i32,
    #[serde(default)]
    pub objectsnaponopen: i32,
    #[serde(default)]
    pub statusbarvisible: i32,
    #[serde(default)]
    pub toolbarvisible: i32,
    #[serde(default)]
    pub boxes: Vec<BoxContainer>,
    #[serde(default)]
    pub lines: Vec<LineContainer>,
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
    pub architecture: String,
    pub modernui: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxContainer {
    #[serde(rename = "box")]
    pub content: BoxContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxContent {
    #[serde(default)]
    pub id: String,
    pub maxclass: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub numinlets: u32,
    #[serde(default)]
    pub numoutlets: u32,
    #[serde(default)]
    pub patching_rect: Option<[f32; 4]>,
    #[serde(default)]
    pub outlettype: Option<Vec<String>>,
    #[serde(default)]
    pub parameter_enable: Option<i32>,
    #[serde(flatten)]
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineContainer {
    pub patchline: PatchLine,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchLine {
    pub source: serde_json::Value, // Can be [string, number] or other formats
    pub destination: serde_json::Value, // Can be [string, number] or other formats
    #[serde(default)]
    pub midpoints: Option<Vec<f32>>,
    #[serde(flatten)]
    pub extra_fields: HashMap<String, serde_json::Value>,
}
