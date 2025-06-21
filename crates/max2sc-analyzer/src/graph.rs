//! Signal flow graph implementation

use crate::AnalysisError;
use max2sc_max_types::MaxPatch;
use petgraph::{Directed, Graph};
use std::collections::HashMap;

pub type SignalFlowGraph = Graph<AudioNode, AudioConnection, Directed>;

#[derive(Debug, Clone)]
pub struct AudioNode {
    pub id: String,
    pub object_type: String,
    pub text: Option<String>,
    pub num_inlets: u32,
    pub num_outlets: u32,
    pub position: Option<[f32; 4]>, // patching_rect
}

#[derive(Debug, Clone)]
pub struct AudioConnection {
    pub source_outlet: u32,
    pub dest_inlet: u32,
    pub connection_type: ConnectionType,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Audio,
    Control,
    Message,
    Unknown,
}

pub fn build_signal_flow_graph(patch: &MaxPatch) -> Result<SignalFlowGraph, AnalysisError> {
    let mut graph = Graph::new();
    let mut node_map: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();

    // First pass: Add all nodes to the graph
    for box_container in &patch.patcher.boxes {
        let content = &box_container.content;

        let node = AudioNode {
            id: content.id.clone(),
            object_type: content.maxclass.clone(),
            text: content.text.clone(),
            num_inlets: content.numinlets,
            num_outlets: content.numoutlets,
            position: content.patching_rect,
        };

        let node_index = graph.add_node(node);
        node_map.insert(content.id.clone(), node_index);
    }

    // Second pass: Add connections between nodes
    for line_container in &patch.patcher.lines {
        let patchline = &line_container.patchline;

        // Parse source and destination
        let (source_id, source_outlet) = parse_connection_endpoint(&patchline.source)?;
        let (dest_id, dest_inlet) = parse_connection_endpoint(&patchline.destination)?;

        // Find corresponding node indices
        let source_idx = node_map
            .get(&source_id)
            .ok_or_else(|| AnalysisError::InvalidRouting)?;
        let dest_idx = node_map
            .get(&dest_id)
            .ok_or_else(|| AnalysisError::InvalidRouting)?;

        // Determine connection type based on object types
        let source_node = graph.node_weight(*source_idx).unwrap();
        let dest_node = graph.node_weight(*dest_idx).unwrap();
        let connection_type =
            determine_connection_type(source_node, dest_node, source_outlet, dest_inlet);

        let connection = AudioConnection {
            source_outlet,
            dest_inlet,
            connection_type,
        };

        graph.add_edge(*source_idx, *dest_idx, connection);
    }

    Ok(graph)
}

fn parse_connection_endpoint(endpoint: &serde_json::Value) -> Result<(String, u32), AnalysisError> {
    if let Some(array) = endpoint.as_array() {
        if array.len() >= 2 {
            let id = array[0]
                .as_str()
                .ok_or(AnalysisError::InvalidRouting)?
                .to_string();
            let port = array[1].as_u64().ok_or(AnalysisError::InvalidRouting)? as u32;
            return Ok((id, port));
        }
    }

    Err(AnalysisError::InvalidRouting)
}

fn determine_connection_type(
    source_node: &AudioNode,
    dest_node: &AudioNode,
    _source_outlet: u32,
    _dest_inlet: u32,
) -> ConnectionType {
    // Heuristics for determining connection type
    let source_text = source_node.text.as_deref().unwrap_or("");
    let dest_text = dest_node.text.as_deref().unwrap_or("");

    // Control objects always send control signals
    match source_node.object_type.as_str() {
        "flonum" | "slider" | "dial" | "number" => return ConnectionType::Control,
        _ => {}
    }

    // Audio connections: both source and dest must be audio objects
    let source_is_audio = source_text.contains('~') || source_text.starts_with("spat5");
    let dest_is_audio = dest_text.contains('~') || dest_text.starts_with("spat5");

    if source_is_audio && dest_is_audio {
        return ConnectionType::Audio;
    }

    // Control connections from control objects or to control parameters
    if source_node.object_type == "newobj" {
        if source_text.starts_with("line") || source_text.starts_with("ramp") {
            return ConnectionType::Control;
        }
    }

    // Default for other connections
    if source_is_audio || dest_is_audio {
        ConnectionType::Control // Mixed audio/control is usually control
    } else {
        ConnectionType::Message
    }
}

/// Get all audio source nodes (nodes with no audio inputs)
pub fn get_audio_sources(graph: &SignalFlowGraph) -> Vec<petgraph::graph::NodeIndex> {
    let mut sources = Vec::new();

    for node_idx in graph.node_indices() {
        let node = graph.node_weight(node_idx).unwrap();
        if is_audio_generator(node) {
            let has_audio_input = graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
                .any(|edge| matches!(edge.weight().connection_type, ConnectionType::Audio));

            if !has_audio_input {
                sources.push(node_idx);
            }
        }
    }

    sources
}

/// Get all audio sink nodes (nodes with no audio outputs)
pub fn get_audio_sinks(graph: &SignalFlowGraph) -> Vec<petgraph::graph::NodeIndex> {
    let mut sinks = Vec::new();

    for node_idx in graph.node_indices() {
        let node = graph.node_weight(node_idx).unwrap();
        if is_audio_sink(node) {
            let has_audio_output = graph
                .edges_directed(node_idx, petgraph::Direction::Outgoing)
                .any(|edge| matches!(edge.weight().connection_type, ConnectionType::Audio));

            if !has_audio_output {
                sinks.push(node_idx);
            }
        }
    }

    sinks
}

fn is_audio_generator(node: &AudioNode) -> bool {
    let text = node.text.as_deref().unwrap_or("");

    match node.object_type.as_str() {
        "newobj" => {
            text.starts_with("cycle~")
                || text.starts_with("noise~")
                || text.starts_with("adc~")
                || text.starts_with("mc.adc~")
                || text.starts_with("phasor~")
                || text.starts_with("saw~")
                || text.starts_with("tri~")
                || text.starts_with("rect~")
        }
        _ => false,
    }
}

fn is_audio_sink(node: &AudioNode) -> bool {
    let text = node.text.as_deref().unwrap_or("");

    match node.object_type.as_str() {
        "newobj" => {
            text.starts_with("dac~")
                || text.starts_with("mc.dac~")
                || text.starts_with("spat5.panoramix~")
                || text.starts_with("spat5.spat~")
        }
        _ => false,
    }
}

/// Find all spatial processing nodes
pub fn get_spatial_nodes(graph: &SignalFlowGraph) -> Vec<petgraph::graph::NodeIndex> {
    let mut spatial_nodes = Vec::new();

    for node_idx in graph.node_indices() {
        let node = graph.node_weight(node_idx).unwrap();
        if is_spatial_node(node) {
            spatial_nodes.push(node_idx);
        }
    }

    spatial_nodes
}

fn is_spatial_node(node: &AudioNode) -> bool {
    let text = node.text.as_deref().unwrap_or("");

    text.starts_with("spat5")
        || text.starts_with("pan~")
        || text.starts_with("pan2~")
        || text.starts_with("pan4~")
        || text.starts_with("pan8~")
}

/// Analyze signal flow chains from sources to sinks
pub fn analyze_signal_chains(graph: &SignalFlowGraph) -> Vec<SignalChain> {
    let sources = get_audio_sources(graph);
    let sinks = get_audio_sinks(graph);
    let mut chains = Vec::new();

    for source_idx in sources {
        for sink_idx in &sinks {
            if let Some(path) = find_audio_path(graph, source_idx, *sink_idx) {
                chains.push(SignalChain {
                    source: source_idx,
                    sink: *sink_idx,
                    path,
                });
            }
        }
    }

    chains
}

#[derive(Debug)]
pub struct SignalChain {
    pub source: petgraph::graph::NodeIndex,
    pub sink: petgraph::graph::NodeIndex,
    pub path: Vec<petgraph::graph::NodeIndex>,
}

fn find_audio_path(
    graph: &SignalFlowGraph,
    source: petgraph::graph::NodeIndex,
    sink: petgraph::graph::NodeIndex,
) -> Option<Vec<petgraph::graph::NodeIndex>> {
    use petgraph::algo::simple_paths::all_simple_paths;

    // Find the first simple path that only uses audio connections
    let paths = all_simple_paths::<Vec<_>, _>(graph, source, sink, 0, None);

    for path in paths {
        // Check if all connections in the path are audio connections
        let mut is_audio_path = true;
        for i in 0..path.len() - 1 {
            let edge = graph.find_edge(path[i], path[i + 1]);
            if let Some(edge_idx) = edge {
                let connection = graph.edge_weight(edge_idx).unwrap();
                if !matches!(connection.connection_type, ConnectionType::Audio) {
                    is_audio_path = false;
                    break;
                }
            }
        }

        if is_audio_path {
            return Some(path);
        }
    }

    None
}

/// Extension methods for SignalFlowGraph
pub trait SignalFlowGraphExt {
    fn new_graph() -> Self;
    fn build_from_patch(&mut self, patch: &MaxPatch) -> Result<(), crate::AnalysisError>;
    fn node_count(&self) -> usize;
    fn count_audio_sources(&self) -> usize;
    fn count_audio_sinks(&self) -> usize;
    fn count_control_objects(&self) -> usize;
    fn count_spatial_objects(&self) -> usize;
}

impl SignalFlowGraphExt for SignalFlowGraph {
    /// Create a new empty signal flow graph
    fn new_graph() -> Self {
        Graph::new()
    }

    /// Build graph from Max patch
    fn build_from_patch(&mut self, patch: &MaxPatch) -> Result<(), crate::AnalysisError> {
        *self = build_signal_flow_graph(patch)?;
        Ok(())
    }

    /// Get the number of nodes in the graph
    fn node_count(&self) -> usize {
        Graph::node_count(self)
    }

    /// Count audio source objects (objects that generate audio)
    fn count_audio_sources(&self) -> usize {
        get_audio_sources(self).len()
    }

    /// Count audio sink objects (objects that consume audio)
    fn count_audio_sinks(&self) -> usize {
        get_audio_sinks(self).len()
    }

    /// Count control objects
    fn count_control_objects(&self) -> usize {
        self.node_weights()
            .filter(|node| is_control_object(node))
            .count()
    }

    /// Count spatial objects
    fn count_spatial_objects(&self) -> usize {
        get_spatial_nodes(self).len()
    }
}

fn is_control_object(node: &AudioNode) -> bool {
    matches!(
        node.object_type.as_str(),
        "flonum"
            | "slider"
            | "dial"
            | "number"
            | "toggle"
            | "button"
            | "live.dial"
            | "live.slider"
            | "live.numbox"
    )
}
