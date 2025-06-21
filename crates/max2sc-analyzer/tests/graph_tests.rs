//! Signal flow graph tests

use max2sc_analyzer::{
    analyze_signal_chains, build_signal_flow_graph, get_audio_sinks, get_audio_sources,
    get_spatial_nodes, ConnectionType,
};
use max2sc_parser::parse_patch_string;

#[test]
fn test_build_simple_signal_graph() {
    let simple_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "cycle~ 440",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-2", 
                        "maxclass": "newobj",
                        "text": "dac~",
                        "numinlets": 2,
                        "numoutlets": 0
                    }
                }
            ],
            "lines": [
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-2", 0]
                    }
                }
            ]
        }
    }"#;

    let patch = parse_patch_string(simple_patch).expect("Failed to parse patch");
    let graph = build_signal_flow_graph(&patch).expect("Failed to build graph");

    // Should have 2 nodes
    assert_eq!(graph.node_count(), 2);

    // Should have 1 edge
    assert_eq!(graph.edge_count(), 1);

    // Check that nodes exist
    let nodes: Vec<_> = graph.node_weights().collect();
    assert!(nodes.iter().any(|n| n.id == "obj-1"));
    assert!(nodes.iter().any(|n| n.id == "obj-2"));

    // Check edge connection type
    let edge = graph.edge_weights().next().unwrap();
    assert!(matches!(edge.connection_type, ConnectionType::Audio));

    println!(
        "Successfully built graph with {} nodes and {} edges",
        graph.node_count(),
        graph.edge_count()
    );
}

#[test]
fn test_identify_audio_sources_and_sinks() {
    let audio_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "cycle~ 440",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-2",
                        "maxclass": "newobj",
                        "text": "noise~",
                        "numinlets": 0,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-3", 
                        "maxclass": "newobj",
                        "text": "dac~",
                        "numinlets": 2,
                        "numoutlets": 0
                    }
                },
                {
                    "box": {
                        "id": "obj-4",
                        "maxclass": "flonum",
                        "numinlets": 1,
                        "numoutlets": 2
                    }
                }
            ],
            "lines": [
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-3", 0]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-2", 0], 
                        "destination": ["obj-3", 1]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-4", 0],
                        "destination": ["obj-1", 0]
                    }
                }
            ]
        }
    }"#;

    let patch = parse_patch_string(audio_patch).expect("Failed to parse patch");
    let graph = build_signal_flow_graph(&patch).expect("Failed to build graph");

    // Find audio sources
    let sources = get_audio_sources(&graph);
    assert_eq!(
        sources.len(),
        2,
        "Should find 2 audio sources (cycle~ and noise~)"
    );

    // Find audio sinks
    let sinks = get_audio_sinks(&graph);
    assert_eq!(sinks.len(), 1, "Should find 1 audio sink (dac~)");

    // Check source types
    for source_idx in &sources {
        let node = graph.node_weight(*source_idx).unwrap();
        let text = node.text.as_deref().unwrap_or("");
        assert!(text.starts_with("cycle~") || text.starts_with("noise~"));
    }

    // Check sink type
    let sink_node = graph.node_weight(sinks[0]).unwrap();
    assert_eq!(sink_node.text.as_deref().unwrap(), "dac~");

    println!(
        "Found {} audio sources and {} audio sinks",
        sources.len(),
        sinks.len()
    );
}

#[test]
fn test_spatial_node_detection() {
    let spatial_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "cycle~ 440",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-2",
                        "maxclass": "newobj",
                        "text": "spat5.panoramix~",
                        "numinlets": 1,
                        "numoutlets": 8
                    }
                },
                {
                    "box": {
                        "id": "obj-3",
                        "maxclass": "newobj",
                        "text": "pan~ 0.5",
                        "numinlets": 2,
                        "numoutlets": 2
                    }
                },
                {
                    "box": {
                        "id": "obj-4",
                        "maxclass": "newobj", 
                        "text": "dac~",
                        "numinlets": 2,
                        "numoutlets": 0
                    }
                }
            ],
            "lines": [
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-2", 0]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-3", 0]
                    }
                }
            ]
        }
    }"#;

    let patch = parse_patch_string(spatial_patch).expect("Failed to parse spatial patch");
    let graph = build_signal_flow_graph(&patch).expect("Failed to build graph");

    // Find spatial nodes
    let spatial_nodes = get_spatial_nodes(&graph);
    assert_eq!(spatial_nodes.len(), 2, "Should find 2 spatial nodes");

    // Check spatial node types
    for spatial_idx in &spatial_nodes {
        let node = graph.node_weight(*spatial_idx).unwrap();
        let text = node.text.as_deref().unwrap_or("");
        assert!(text.starts_with("spat5") || text.starts_with("pan~"));
    }

    println!("Found {} spatial processing nodes", spatial_nodes.len());
}

#[test]
fn test_signal_chain_analysis() {
    let chain_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "newobj",
                        "text": "cycle~ 440",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-2",
                        "maxclass": "newobj",
                        "text": "*~ 0.5",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-3",
                        "maxclass": "newobj",
                        "text": "pan~ 0",
                        "numinlets": 2,
                        "numoutlets": 2
                    }
                },
                {
                    "box": {
                        "id": "obj-4", 
                        "maxclass": "newobj",
                        "text": "dac~",
                        "numinlets": 2,
                        "numoutlets": 0
                    }
                }
            ],
            "lines": [
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-2", 0]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-2", 0],
                        "destination": ["obj-3", 0]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-3", 0],
                        "destination": ["obj-4", 0]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-3", 1],
                        "destination": ["obj-4", 1]
                    }
                }
            ]
        }
    }"#;

    let patch = parse_patch_string(chain_patch).expect("Failed to parse chain patch");
    let graph = build_signal_flow_graph(&patch).expect("Failed to build graph");

    // Analyze signal chains
    let chains = analyze_signal_chains(&graph);
    assert!(!chains.is_empty(), "Should find at least one signal chain");

    // Check that chain goes from cycle~ to dac~
    let chain = &chains[0];
    let source_node = graph.node_weight(chain.source).unwrap();
    let sink_node = graph.node_weight(chain.sink).unwrap();

    assert_eq!(source_node.text.as_deref().unwrap(), "cycle~ 440");
    assert_eq!(sink_node.text.as_deref().unwrap(), "dac~");

    // Should have a path through multiple nodes
    assert!(
        chain.path.len() >= 2,
        "Chain should have multiple nodes in path"
    );

    println!(
        "Found {} signal chains, first chain has {} nodes",
        chains.len(),
        chain.path.len()
    );
}

#[test]
fn test_connection_type_detection() {
    let mixed_patch = r#"{
        "patcher": {
            "rect": [0.0, 0.0, 640.0, 480.0],
            "boxes": [
                {
                    "box": {
                        "id": "obj-1",
                        "maxclass": "flonum",
                        "numinlets": 1,
                        "numoutlets": 2
                    }
                },
                {
                    "box": {
                        "id": "obj-2",
                        "maxclass": "newobj",
                        "text": "cycle~",
                        "numinlets": 2,
                        "numoutlets": 1
                    }
                },
                {
                    "box": {
                        "id": "obj-3",
                        "maxclass": "newobj",
                        "text": "dac~",
                        "numinlets": 2,
                        "numoutlets": 0
                    }
                }
            ],
            "lines": [
                {
                    "patchline": {
                        "source": ["obj-1", 0],
                        "destination": ["obj-2", 0]
                    }
                },
                {
                    "patchline": {
                        "source": ["obj-2", 0],
                        "destination": ["obj-3", 0]
                    }
                }
            ]
        }
    }"#;

    let patch = parse_patch_string(mixed_patch).expect("Failed to parse mixed patch");
    let graph = build_signal_flow_graph(&patch).expect("Failed to build graph");

    // Check connection types
    let edges: Vec<_> = graph.edge_weights().collect();
    assert_eq!(edges.len(), 2);

    // First connection (flonum -> cycle~) should be control
    // Second connection (cycle~ -> dac~) should be audio
    let control_found = edges
        .iter()
        .any(|e| matches!(e.connection_type, ConnectionType::Control));
    let audio_found = edges
        .iter()
        .any(|e| matches!(e.connection_type, ConnectionType::Audio));

    assert!(control_found, "Should find control connection");
    assert!(audio_found, "Should find audio connection");

    println!("Successfully detected different connection types");
}
