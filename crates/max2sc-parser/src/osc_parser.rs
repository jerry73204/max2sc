//! OSC configuration parsing

use crate::ParseError;
use max2sc_max_types::{parse_osc_text, OSCConfig};
use std::fs;
use std::path::Path;

pub fn parse_osc_config<P: AsRef<Path>>(path: P) -> Result<OSCConfig, ParseError> {
    let content = fs::read_to_string(path)?;
    parse_osc_text(&content).map_err(|e| ParseError::InvalidFormat(e.to_string()))
}
