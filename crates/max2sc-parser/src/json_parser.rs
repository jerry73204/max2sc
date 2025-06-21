//! JSON parsing logic

use crate::ParseError;
use max2sc_max_types::MaxPatch;
use std::fs;
use std::path::Path;

pub fn parse_patch_file<P: AsRef<Path>>(path: P) -> Result<MaxPatch, ParseError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;
    parse_patch_string(&content)
}

pub fn parse_patch_string(content: &str) -> Result<MaxPatch, ParseError> {
    let patch: MaxPatch = serde_json::from_str(content)?;
    Ok(patch)
}
