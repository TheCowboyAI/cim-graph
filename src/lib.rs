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
//! - **Conceptual Spaces**: Topological spaces with Voronoi tessellation
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

/// Event stream optimization utilities
pub mod optimization;

/// Event analytics and metrics
pub mod analytics;

/// Conceptual spaces with topological structure
pub mod conceptual_space;

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