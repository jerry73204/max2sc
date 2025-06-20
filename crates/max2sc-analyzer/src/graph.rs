//! Signal flow graph implementation

use crate::AnalysisError;
use max2sc_max_types::MaxPatch;
use petgraph::Graph;

pub type SignalFlowGraph = Graph<String, String>;

pub fn build_signal_flow_graph(patch: &MaxPatch) -> Result<SignalFlowGraph, AnalysisError> {
    todo!("Implement signal flow graph construction")
}
