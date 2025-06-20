//! JSON parsing logic

use crate::ParseError;
use max2sc_max_types::MaxPatch;
use std::path::Path;

pub fn parse_patch_file<P: AsRef<Path>>(path: P) -> Result<MaxPatch, ParseError> {
    todo!("Implement Max patch file parsing")
}

pub fn parse_patch_string(content: &str) -> Result<MaxPatch, ParseError> {
    todo!("Implement Max patch string parsing")
}
