//! Property tests for serialization round-trips

use proptest::prelude::*;
use cim_graph::core::{Graph, GenericNode, GenericEdge, GraphType, Node, Edge};
use cim_graph::core::graph::BasicGraph;
use super::generators::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::convert::TryFrom;

/// A serializable representation of a graph
#[derive(Serialize, Deserialize)]
struct SerializableGraph {
    graph_type: GraphType,
    nodes: Vec<SerializableNode>,
    edges: Vec<SerializableEdge>,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
struct SerializableNode {
    id: String,
    data: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SerializableEdge {
    id: String,
    source: String,
    target: String,
    weight: f64,
}

impl From<&BasicGraph<GenericNode<String>, GenericEdge<f64>>> for SerializableGraph {
    fn from(graph: &BasicGraph<GenericNode<String>, GenericEdge<f64>>) -> Self {
        let nodes = graph.node_ids()
            .into_iter()
            .filter_map(|id| {
                graph.get_node(&id).map(|node| SerializableNode {
                    id: node.id(),
                    data: node.data().clone(),
                })
            })
            .collect();
            
        let edges = graph.edge_ids()
            .into_iter()
            .filter_map(|id| {
                graph.get_edge(&id).map(|edge| SerializableEdge {
                    id: edge.id(),
                    source: edge.source(),
                    target: edge.target(),
                    weight: *edge.data(),
                })
            })
            .collect();
            
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), 
            serde_json::Value::String(graph.metadata().name.clone().unwrap_or_default()));
        metadata.insert("version".to_string(), 
            serde_json::Value::String(graph.metadata().version.clone()));
            
        SerializableGraph {
            graph_type: graph.graph_type(),
            nodes,
            edges,
            metadata,
        }
    }
}

impl TryFrom<SerializableGraph> for BasicGraph<GenericNode<String>, GenericEdge<f64>> {
    type Error = String;
    
    fn try_from(ser: SerializableGraph) -> Result<Self, Self::Error> {
        let mut graph = BasicGraph::new(ser.graph_type);
        
        // Add all nodes
        for node in ser.nodes {
            let _ = graph.add_node(GenericNode::new(node.id, node.data));
        }
        
        // Add all edges
        for edge in ser.edges {
            let _ = graph.add_edge(GenericEdge::with_id(
                edge.id,
                edge.source,
                edge.target,
                edge.weight,
            ));
        }
        
        // Update metadata
        if let Some(name) = ser.metadata.get("name").and_then(|v| v.as_str()) {
            graph.metadata_mut().name = Some(name.to_string());
        }
        if let Some(version) = ser.metadata.get("version").and_then(|v| v.as_str()) {
            graph.metadata_mut().version = version.to_string();
        }
        
        Ok(graph)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn json_serialization_roundtrip_preserves_structure(
        graph in medium_graph_strategy(),
    ) {
        // Convert to serializable form
        let serializable = SerializableGraph::from(&graph);
        
        // Serialize to JSON
        let json = serde_json::to_string(&serializable).unwrap();
        
        // Deserialize from JSON
        let deserialized: SerializableGraph = serde_json::from_str(&json).unwrap();
        
        // Convert back to graph
        let restored = BasicGraph::try_from(deserialized).unwrap();
        
        // Verify structure is preserved
        prop_assert_eq!(restored.node_count(), graph.node_count());
        prop_assert_eq!(restored.edge_count(), graph.edge_count());
        prop_assert_eq!(restored.graph_type(), graph.graph_type());
        
        // Verify all nodes exist
        for node_id in graph.node_ids() {
            prop_assert!(restored.contains_node(&node_id));
            
            let original_node = graph.get_node(&node_id).unwrap();
            let restored_node = restored.get_node(&node_id).unwrap();
            prop_assert_eq!(original_node.data(), restored_node.data());
        }
        
        // Verify all edges exist
        for edge_id in graph.edge_ids() {
            prop_assert!(restored.contains_edge(&edge_id));
            
            let original_edge = graph.get_edge(&edge_id).unwrap();
            let restored_edge = restored.get_edge(&edge_id).unwrap();
            prop_assert_eq!(original_edge.source(), restored_edge.source());
            prop_assert_eq!(original_edge.target(), restored_edge.target());
            prop_assert_eq!(original_edge.data(), restored_edge.data());
        }
    }
    
    #[test]
    fn json_serialization_handles_empty_graph(
        graph_type in prop::sample::select(vec![
            GraphType::Generic,
            GraphType::WorkflowGraph,
            GraphType::ContextGraph,
        ]),
    ) {
        let empty_graph = BasicGraph::<GenericNode<String>, GenericEdge<f64>>::new(graph_type);
        
        let serializable = SerializableGraph::from(&empty_graph);
        let json = serde_json::to_string(&serializable).unwrap();
        let deserialized: SerializableGraph = serde_json::from_str(&json).unwrap();
        let restored = BasicGraph::try_from(deserialized).unwrap();
        
        prop_assert_eq!(restored.node_count(), 0);
        prop_assert_eq!(restored.edge_count(), 0);
        prop_assert_eq!(restored.graph_type(), graph_type);
    }
    
    #[test]
    fn json_serialization_preserves_metadata(
        mut graph in small_graph_strategy(),
        name in prop::option::of("[a-zA-Z0-9 -]{1,50}"),
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        // Set metadata
        graph.metadata_mut().name = name.clone();
        graph.metadata_mut().version = version.clone();
        
        let serializable = SerializableGraph::from(&graph);
        let json = serde_json::to_string(&serializable).unwrap();
        let deserialized: SerializableGraph = serde_json::from_str(&json).unwrap();
        let restored = BasicGraph::try_from(deserialized).unwrap();
        
        prop_assert_eq!(restored.metadata().name, name);
        prop_assert_eq!(restored.metadata().version, version);
    }
    
    #[test]
    fn binary_serialization_roundtrip_preserves_structure(
        graph in small_graph_strategy(),
    ) {
        let serializable = SerializableGraph::from(&graph);
        
        // Use bincode for binary serialization
        let encoded = bincode::serialize(&serializable).unwrap();
        let decoded: SerializableGraph = bincode::deserialize(&encoded).unwrap();
        let restored = BasicGraph::try_from(decoded).unwrap();
        
        prop_assert_eq!(restored.node_count(), graph.node_count());
        prop_assert_eq!(restored.edge_count(), graph.edge_count());
    }
    
    #[test]
    fn serialization_handles_special_characters(
        mut graph in Just(BasicGraph::<GenericNode<String>, GenericEdge<f64>>::new(GraphType::Generic)),
        special_chars in prop::collection::vec(
            prop::sample::select(vec![
                "node-with-spaces",
                "node_with_underscores",
                "node.with.dots",
                "node/with/slashes",
                "node\\with\\backslashes",
                "node\"with\"quotes",
                "node'with'apostrophes",
                "🚀emoji-node🎉",
                "node\nwith\nnewlines",
                "node\twith\ttabs",
            ]),
            1..5
        ),
    ) {
        // Add nodes with special characters
        for (i, special) in special_chars.iter().enumerate() {
            let node = GenericNode::new(format!("node-{}", i), special.to_string());
            let _ = graph.add_node(node);
        }
        
        let serializable = SerializableGraph::from(&graph);
        let json = serde_json::to_string(&serializable).unwrap();
        let deserialized: SerializableGraph = serde_json::from_str(&json).unwrap();
        let restored = BasicGraph::try_from(deserialized).unwrap();
        
        prop_assert_eq!(restored.node_count(), graph.node_count());
        
        // Verify special characters are preserved
        for (i, expected_data) in special_chars.iter().enumerate() {
            let node_id = format!("node-{}", i);
            let node = restored.get_node(&node_id).unwrap();
            prop_assert_eq!(node.data(), expected_data);
        }
    }
    
    #[test]
    fn serialization_handles_large_graphs(
        graph in large_graph_strategy(),
    ) {
        let node_count = graph.node_count();
        let edge_count = graph.edge_count();
        
        let serializable = SerializableGraph::from(&graph);
        let json = serde_json::to_string(&serializable).unwrap();
        
        // Verify JSON is not absurdly large
        prop_assert!(json.len() < 100_000_000); // 100MB limit
        
        let deserialized: SerializableGraph = serde_json::from_str(&json).unwrap();
        let restored = BasicGraph::try_from(deserialized).unwrap();
        
        prop_assert_eq!(restored.node_count(), node_count);
        prop_assert_eq!(restored.edge_count(), edge_count);
    }
    
    #[test]
    fn partial_serialization_preserves_consistency(
        graph in medium_graph_strategy().prop_filter("has nodes", |g| g.node_count() > 0),
        subset_ratio in 0.1..=0.9,
    ) {
        // Serialize only a subset of nodes and their edges
        let all_nodes = graph.node_ids();
        let subset_size = ((all_nodes.len() as f64) * subset_ratio) as usize;
        let subset_nodes: Vec<_> = all_nodes.into_iter().take(subset_size).collect();
        
        let mut partial_ser = SerializableGraph {
            graph_type: graph.graph_type(),
            nodes: vec![],
            edges: vec![],
            metadata: HashMap::new(),
        };
        
        // Add subset of nodes
        for node_id in &subset_nodes {
            if let Some(node) = graph.get_node(node_id) {
                partial_ser.nodes.push(SerializableNode {
                    id: node.id(),
                    data: node.data().clone(),
                });
            }
        }
        
        // Add edges between subset nodes only
        for edge_id in graph.edge_ids() {
            if let Some(edge) = graph.get_edge(&edge_id) {
                if subset_nodes.contains(&edge.source()) && subset_nodes.contains(&edge.target()) {
                    partial_ser.edges.push(SerializableEdge {
                        id: edge.id(),
                        source: edge.source(),
                        target: edge.target(),
                        weight: *edge.data(),
                    });
                }
            }
        }
        
        // Serialize and restore
        let json = serde_json::to_string(&partial_ser).unwrap();
        let restored_ser: SerializableGraph = serde_json::from_str(&json).unwrap();
        let restored = BasicGraph::try_from(restored_ser).unwrap();
        
        // Verify partial graph is valid
        prop_assert_eq!(restored.node_count(), partial_ser.nodes.len());
        
        // All edges should have valid endpoints
        for edge_id in restored.edge_ids() {
            let edge = restored.get_edge(&edge_id).unwrap();
            prop_assert!(restored.contains_node(&edge.source()));
            prop_assert!(restored.contains_node(&edge.target()));
        }
    }
    
    #[test]
    fn serialization_format_stability(
        node_count in 1..10usize,
        edge_probability in 0.0..1.0,
    ) {
        // Create a deterministic graph
        let mut graph = BasicGraph::new(GraphType::Generic);
        
        // Add nodes with predictable IDs
        for i in 0..node_count {
            let node = GenericNode::new(format!("n{}", i), format!("data{}", i));
            let _ = graph.add_node(node);
        }
        
        // Add edges deterministically based on probability
        let mut edge_count = 0;
        for i in 0..node_count {
            for j in 0..node_count {
                if i != j && ((i * node_count + j) as f64 / (node_count * node_count) as f64) < edge_probability {
                    let edge = GenericEdge::with_id(
                        format!("e{}", edge_count),
                        format!("n{}", i),
                        format!("n{}", j),
                        1.0,
                    );
                    let _ = graph.add_edge(edge);
                    edge_count += 1;
                }
            }
        }
        
        // Serialize multiple times
        let ser1 = SerializableGraph::from(&graph);
        let json1 = serde_json::to_string(&ser1).unwrap();
        
        let ser2 = SerializableGraph::from(&graph);
        let json2 = serde_json::to_string(&ser2).unwrap();
        
        // Same graph should produce same JSON
        prop_assert_eq!(json1, json2);
    }
}

// Additional test for actual Graph trait to_json method
#[test]
fn test_graph_to_json_method() {
    let mut graph = BasicGraph::new(GraphType::Generic);
    
    // Add some nodes
    graph.add_node(GenericNode::new("a", "data_a")).unwrap();
    graph.add_node(GenericNode::new("b", "data_b")).unwrap();
    
    // Add an edge
    graph.add_edge(GenericEdge::new("a", "b", 1.5)).unwrap();
    
    // Test the to_json method
    let json_value = graph.to_json().unwrap();
    
    assert_eq!(json_value["type"], "Generic");
    assert_eq!(json_value["node_count"], 2);
    assert_eq!(json_value["edge_count"], 1);
    
    let nodes = json_value["nodes"].as_array().unwrap();
    assert!(nodes.contains(&serde_json::Value::String("a".to_string())));
    assert!(nodes.contains(&serde_json::Value::String("b".to_string())));
}