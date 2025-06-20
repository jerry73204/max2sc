//! Routing analysis

use crate::AnalysisError;
use max2sc_max_types::MaxPatch;

pub fn analyze_routing(patch: &MaxPatch) -> Result<RoutingInfo, AnalysisError> {
    todo!("Implement routing analysis")
}

#[derive(Debug)]
pub struct RoutingInfo {
    // TODO: Define routing information structure
}
