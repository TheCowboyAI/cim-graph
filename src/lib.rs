//! # CIM Graph
//!
//! A unified graph abstraction library consolidating all graph operations across the CIM ecosystem.
//!
//! ## Overview
//!
//! CIM Graph provides a comprehensive set of graph types and algorithms for various domains:
//!
//! - **IPLD Graphs**: Content-addressed data structures
//! - **Context Graphs**: Domain-Driven Design bounded contexts
//! - **Workflow Graphs**: State machines and process flows
//! - **Concept Graphs**: Semantic reasoning and knowledge representation
//! - **Composed Graphs**: Multi-domain graph compositions
//!
//! ## Features
//!
//! - Event-driven architecture with full audit trails
//! - Type-safe graph operations
//! - High-performance algorithms
//! - Serialization support
//! - Extensible design
//!
//! ## Example
//!
//! ```rust
//! use cim_graph::{
//!     core::{ProjectionEngine, GraphProjection},
//!     events::{GraphEvent, EventPayload, WorkflowPayload},
//!     graphs::{WorkflowNode, WorkflowEdge, WorkflowProjection},
//! };
//! use uuid::Uuid;
//!
//! # fn main() -> cim_graph::Result<()> {
//! // Create events to build a workflow
//! let workflow_id = Uuid::new_v4();
//! let events = vec![
//!     GraphEvent {
//!         event_id: Uuid::new_v4(),
//!         aggregate_id: workflow_id,
//!         correlation_id: Uuid::new_v4(),
//!         causation_id: None,
//!         payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
//!             workflow_id,
//!             name: "Example Workflow".to_string(),
//!             version: "1.0.0".to_string(),
//!         }),
//!     },
//!     GraphEvent {
//!         event_id: Uuid::new_v4(),
//!         aggregate_id: workflow_id,
//!         correlation_id: Uuid::new_v4(),
//!         causation_id: None,
//!         payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
//!             workflow_id,
//!             state_id: "start".to_string(),
//!             state_type: "initial".to_string(),
//!         }),
//!     },
//! ];
//!
//! // Build projection from events
//! let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
//! let projection = engine.project(events);
//! 
//! // Query the projection
//! assert_eq!(projection.node_count(), 1);
//! assert_eq!(projection.version(), 2); // Two events processed
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Core graph traits and types
pub mod core;

/// Event schemas and handlers
pub mod events;

/// Graph implementations
pub mod graphs;

/// Graph algorithms
pub mod algorithms;

/// Error types
pub mod error;

/// Serialization support
pub mod serde_support;

/// Performance optimizations
pub mod performance;

/// Bounded contexts and domain boundaries
pub mod contexts;

/// NATS JetStream integration
#[cfg(feature = "nats")]
#[cfg_attr(docsrs, doc(cfg(feature = "nats")))]
pub mod nats;

// Re-exports
pub use crate::core::{Node, Edge, EventHandler, GraphEvent as CoreGraphEvent, GraphProjection};
pub use crate::error::{GraphError, Result};
pub use crate::graphs::{
    IpldProjection, IpldCommand,
    ContextGraph,
    WorkflowGraph,
    ConceptGraph,
    ComposedGraph,
};
pub use crate::events::{GraphEvent, GraphCommand, EventPayload};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        // Ensure all main types are accessible
        fn _test_projection_trait<G: GraphProjection>(_projection: &G) {}
        fn _test_node_trait<N: Node>(_node: &N) {}
        fn _test_edge_trait<E: Edge>(_edge: &E) {}
    }

    #[test]
    fn test_error_types() {
        use crate::error::GraphError;
        
        let err = GraphError::NodeNotFound("test".to_string());
        assert!(err.to_string().contains("Node not found"));
        
        let err = GraphError::EdgeNotFound("test".to_string());
        assert!(err.to_string().contains("Edge not found"));
        
        let err = GraphError::TypeMismatch {
            expected: "A".to_string(),
            actual: "B".to_string(),
        };
        assert!(err.to_string().contains("Type mismatch"));
        
        let err = GraphError::InvalidOperation("test".to_string());
        assert!(err.to_string().contains("Invalid operation"));
        
        let err = GraphError::SerializationError("test".to_string());
        assert!(err.to_string().contains("Serialization error"));
        
        let err = GraphError::ConstraintViolation("test".to_string());
        assert!(err.to_string().contains("Constraint violation"));
        
        let err = GraphError::DuplicateNode("test".to_string());
        assert!(err.to_string().contains("Duplicate node"));
        
        let err = GraphError::DuplicateEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        assert!(err.to_string().contains("Duplicate edge"));
    }
    
    #[test]
    fn test_graph_builder() {
        use crate::core::graph::GraphType;
        use crate::core::node::GenericNode;
        use crate::core::edge::GenericEdge;
        
        let graph = GraphBuilder::<GenericNode<&str>, GenericEdge<()>>::new()
            .graph_type(GraphType::Generic)
            .name("Test Graph")
            .description("A test graph")
            .build()
            .unwrap();
            
        assert_eq!(graph.metadata().name, Some("Test Graph".to_string()));
        assert_eq!(graph.metadata().description, Some("A test graph".to_string()));
    }
    
    #[test]
    fn test_ipld_graph_creation() {
        use crate::graphs::ipld::Cid;
        
        let mut graph = IpldGraph::new();
        
        // Test add_content
        let cid1 = graph.add_content(serde_json::json!({
            "data": "test"
        })).unwrap();
        
        let cid2 = graph.add_content(serde_json::json!({
            "data": "test2"
        })).unwrap();
        
        // Test add_link
        graph.add_link(&cid1, &cid2, "next").unwrap();
        
        // Test get methods
        assert!(graph.get_content(&cid1).is_some());
        let children: Vec<Cid> = graph.graph().neighbors(&cid1.as_str()).unwrap()
            .into_iter()
            .map(|s| Cid::new(s))
            .collect();
        assert!(children.contains(&cid2));
        assert_eq!(graph.graph().node_count(), 2);
        assert_eq!(graph.graph().edge_count(), 1);
    }
    
    #[test]
    fn test_context_graph_creation() {
        let mut graph = ContextGraph::new();
        
        // Add bounded context
        graph.add_bounded_context("orders", "Order Management").unwrap();
        
        // Add aggregate
        let order_id = graph.add_aggregate("order-123", "Order", "orders").unwrap();
        
        // Add entity
        let _item_id = graph.add_entity("item-456", "OrderItem", &order_id).unwrap();
        
        // The Contains relationship is already added by add_entity
        // So we don't need to add it again
        
        assert_eq!(graph.graph().node_count(), 3); // context + aggregate + entity
        assert_eq!(graph.graph().edge_count(), 1);
    }
    
    #[test]
    fn test_workflow_graph_creation() {
        use crate::graphs::workflow::{WorkflowNode, StateType};
        
        let mut graph = WorkflowGraph::new();
        
        // Add states
        let start = graph.add_state(WorkflowNode::new("start", "Start", StateType::Initial)).unwrap();
        let middle = graph.add_state(WorkflowNode::new("middle", "Middle", StateType::Normal)).unwrap();
        let end = graph.add_state(WorkflowNode::new("end", "End", StateType::Final)).unwrap();
        
        // Add transitions
        graph.add_transition(&start, &middle, "next").unwrap();
        graph.add_transition(&middle, &end, "complete").unwrap();
        
        // Test state queries
        let start_node = graph.graph().get_node(&start).unwrap();
        assert_eq!(start_node.state_type(), StateType::Initial);
        assert_eq!(graph.current_states().len(), 1);
        assert!(graph.current_states().contains(&start));
        
        // Test transition
        graph.process_event("next").unwrap();
        assert!(graph.current_states().contains(&middle));
    }
    
    #[test]
    fn test_concept_graph_creation() {
        use crate::graphs::concept::SemanticRelation;
        
        let mut graph = ConceptGraph::new();
        
        // Add concepts
        graph.add_concept("animal", "Animal", serde_json::json!({
            "description": "Living creature"
        })).unwrap();
        
        graph.add_concept("dog", "Dog", serde_json::json!({
            "description": "Man's best friend"
        })).unwrap();
        
        // Add relation
        graph.add_relation("dog", "animal", SemanticRelation::IsA).unwrap();
        
        // Test graph structure
        assert_eq!(graph.graph().node_count(), 2);
        assert_eq!(graph.graph().edge_count(), 1);
        
        // Test inference
        let count = graph.apply_inference();
        assert_eq!(count, 0); // No new inferences from simple IsA
    }
    
    #[test]
    fn test_composed_graph_creation() {
        use crate::graphs::workflow::{WorkflowNode, StateType};
        use crate::graphs::concept::{ConceptNode, ConceptType};
        use crate::graphs::composed::{ComposedNode, ComposedEdge};
        
        let mut graph = ComposedGraph::new();
        
        // Add nodes of different types
        let workflow_node = WorkflowNode::new("state1", "Processing", StateType::Normal);
        let concept_node = ConceptNode::new("ProcessConcept", "Process", ConceptType::Class);
        
        let n1 = graph.add_node(ComposedNode::Workflow(workflow_node)).unwrap();
        let n2 = graph.add_node(ComposedNode::Concept(concept_node)).unwrap();
        
        // Add cross-type edge with a valid relation
        // First add the constraint to allow this relation
        graph.add_constraint("workflow-concept", "workflow", "concept", vec!["models".to_string()]);
        let edge = ComposedEdge::cross_type(&n1, &n2, "workflow", "concept", "models");
        graph.add_edge(edge).unwrap();
        
        assert_eq!(graph.graph().node_count(), 2);
        assert_eq!(graph.graph().edge_count(), 1);
    }
    
    #[test]
    fn test_algorithms() {
        use crate::core::graph::GraphType;
        use crate::core::node::GenericNode;
        use crate::core::edge::GenericEdge;
        use crate::algorithms::{bfs, dfs, shortest_path, topological_sort, centrality};
        
        let mut graph = GraphBuilder::<GenericNode<&str>, GenericEdge<()>>::new()
            .graph_type(GraphType::Generic)
            .build_event()
            .unwrap();
        
        // Create a simple graph: A -> B -> C
        graph.add_node(GenericNode::new("A", "data")).unwrap();
        graph.add_node(GenericNode::new("B", "data")).unwrap();
        graph.add_node(GenericNode::new("C", "data")).unwrap();
        
        graph.add_edge(GenericEdge::new("A", "B", ())).unwrap();
        graph.add_edge(GenericEdge::new("B", "C", ())).unwrap();
        
        // Test BFS
        let bfs_result = bfs(&graph, "A").unwrap();
        assert_eq!(bfs_result, vec!["A", "B", "C"]);
        
        // Test DFS
        let dfs_result = dfs(&graph, "A").unwrap();
        assert_eq!(dfs_result, vec!["A", "B", "C"]);
        
        // Test shortest path
        let path = shortest_path(&graph, "A", "C").unwrap();
        assert_eq!(path, Some(vec!["A".to_string(), "B".to_string(), "C".to_string()]));
        
        // Test topological sort
        let topo = topological_sort(&graph).unwrap();
        assert_eq!(topo, vec!["A", "B", "C"]);
        
        // Test centrality
        let centralities = centrality(&graph).unwrap();
        assert_eq!(centralities.len(), 3);
    }
    
    #[test]
    fn test_event_serialization() {
        use crate::serde_support::{serialize_events, deserialize_events};
        use crate::events::{GraphEvent, EventPayload, IpldPayload};
        use uuid::Uuid;
        
        // Create IPLD events
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidAdded {
                    cid: "Qm123".to_string(),
                    codec: "dag-cbor".to_string(),
                    size: 256,
                    data: serde_json::json!({"test": "data"}),
                }),
            },
        ];
        
        // Serialize events (not the graph!)
        let json = serialize_events(&events).unwrap();
        assert!(json.contains("CidAdded"));
        
        // Deserialize events
        let deserialized = deserialize_events(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
    }
    
    #[test]
    fn test_performance_features() {
        use crate::performance::{NodeIndex, EdgeIndex, GraphCache};
        use crate::core::node::GenericNode;
        use crate::core::edge::GenericEdge;
        use std::sync::Arc;
        
        // Test NodeIndex
        let mut node_index = NodeIndex::<GenericNode<&str>>::new();
        let node = GenericNode::new("test", "data");
        let node_arc = Arc::new(node);
        node_index.insert(node_arc.clone());
        
        assert!(node_index.get("test").is_some());
        // Type indexing defaults to "default" type
        let by_type = node_index.get_by_type("default");
        assert!(by_type.is_some());
        assert_eq!(by_type.unwrap().len(), 1);
        
        // Test EdgeIndex
        let mut edge_index = EdgeIndex::<GenericEdge<()>>::new();
        let edge = GenericEdge::new("A", "B", ());
        let edge_arc = Arc::new(edge);
        edge_index.insert(edge_arc);
        
        let from_edges = edge_index.edges_from("A");
        assert!(from_edges.is_some());
        assert_eq!(from_edges.unwrap().len(), 1);
        let to_edges = edge_index.edges_to("B");
        assert!(to_edges.is_some());
        assert_eq!(to_edges.unwrap().len(), 1);
        
        // Test GraphCache
        let cache = GraphCache::new();
        
        // Cache uses a compute closure
        let result = cache.get_shortest_path("A", "B", || {
            Ok(vec!["A".to_string(), "B".to_string()])
        }).unwrap();
        assert_eq!(result, vec!["A".to_string(), "B".to_string()]);
        
        // Second call should use cached value
        let cached = cache.get_shortest_path("A", "B", || {
            panic!("Should not be called - should use cache")
        }).unwrap();
        assert_eq!(cached, vec!["A".to_string(), "B".to_string()]);
    }
}