//! OSC configuration parsing

use crate::ParseError;
use max2sc_max_types::OSCConfig;
use std::path::Path;

pub fn parse_osc_config<P: AsRef<Path>>(path: P) -> Result<OSCConfig, ParseError> {
    todo!("Implement OSC configuration parsing")
}
