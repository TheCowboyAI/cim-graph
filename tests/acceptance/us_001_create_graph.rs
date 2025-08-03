//! Acceptance tests for US-001: Create Basic Graph

use cim_graph::core::{GenericEdge, GenericNode, GraphType};
use cim_graph::{Graph, GraphBuilder};

type TestNode = GenericNode<String>;
type TestEdge = GenericEdge<()>;

#[test]
fn test_ac_001_1_create_empty_graph() {
    // Given: no existing graph
    // When: I create a new graph
    let graph = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create graph");

    // Then: an empty graph is initialized
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.metadata().created_at <= chrono::Utc::now());
    assert_eq!(graph.graph_type(), GraphType::Generic);
}

#[test]
fn test_ac_001_2_create_typed_graph() {
    // Given: a graph type specification
    // When: I create a typed graph
    let ipld_graph = GraphBuilder::<TestNode, TestEdge>::new()
        .graph_type(GraphType::IpldGraph)
        .build()
        .expect("Failed to create IPLD graph");

    // Then: the graph enforces type constraints
    assert_eq!(ipld_graph.graph_type(), GraphType::IpldGraph);

    // Test other graph types
    let context_graph = GraphBuilder::<TestNode, TestEdge>::new()
        .graph_type(GraphType::ContextGraph)
        .build()
        .expect("Failed to create Context graph");
    assert_eq!(context_graph.graph_type(), GraphType::ContextGraph);

    let workflow_graph = GraphBuilder::<TestNode, TestEdge>::new()
        .graph_type(GraphType::WorkflowGraph)
        .build()
        .expect("Failed to create Workflow graph");
    assert_eq!(workflow_graph.graph_type(), GraphType::WorkflowGraph);

    let concept_graph = GraphBuilder::<TestNode, TestEdge>::new()
        .graph_type(GraphType::ConceptGraph)
        .build()
        .expect("Failed to create Concept graph");
    assert_eq!(concept_graph.graph_type(), GraphType::ConceptGraph);
}

#[test]
fn test_graph_metadata() {
    // Test that graphs can have metadata
    let graph = GraphBuilder::<TestNode, TestEdge>::new()
        .name("Test Graph")
        .description("A graph for testing purposes")
        .build()
        .expect("Failed to create graph with metadata");

    assert_eq!(graph.metadata().name, Some("Test Graph".to_string()));
    assert_eq!(
        graph.metadata().description,
        Some("A graph for testing purposes".to_string())
    );
    assert_eq!(graph.metadata().version, "1.0.0");
}

#[test]
fn test_graph_unique_ids() {
    // Test that each graph gets a unique ID
    let graph1 = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create first graph");

    let graph2 = GraphBuilder::<TestNode, TestEdge>::new()
        .build()
        .expect("Failed to create second graph");

    assert_ne!(graph1.id(), graph2.id());
}

#[test]
fn test_graph_serialization() {
    // Test that graphs can be serialized to JSON
    let graph = GraphBuilder::<TestNode, TestEdge>::new()
        .name("Serializable Graph")
        .graph_type(GraphType::WorkflowGraph)
        .build()
        .expect("Failed to create graph");

    let json = graph.to_json().expect("Failed to serialize graph");

    assert_eq!(json["type"], "WorkflowGraph");
    assert_eq!(json["node_count"], 0);
    assert_eq!(json["edge_count"], 0);
    assert_eq!(json["metadata"]["name"], "Serializable Graph");
}
