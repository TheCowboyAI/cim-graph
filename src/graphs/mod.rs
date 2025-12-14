//! Specialized graph types for different domains
//!
//! This module provides domain-specific graph implementations that maintain
//! their unique semantic properties while exposing a unified API. Each graph
//! type is optimized for its specific use case.
//!
//! # Available Graph Types
//!
//! ## [`IpldGraph`]
//! For content-addressed data structures where nodes are identified by
//! cryptographic hashes (CIDs). Ideal for:
//! - Immutable data structures
//! - Merkle DAGs
//! - Blockchain and distributed systems
//! - Content versioning
//!
//! ## [`ContextGraph`]
//! Models Domain-Driven Design relationships between aggregates, entities,
//! and value objects. Perfect for:
//! - Microservice architectures
//! - Domain modeling
//! - Hierarchical data
//! - Business object relationships
//!
//! ## [`WorkflowGraph`]
//! Represents state machines and process flows. Useful for:
//! - Business process modeling
//! - Approval workflows
//! - Saga orchestration
//! - Event-driven architectures
//!
//! ## [`ConceptGraph`]
//! Implements semantic reasoning and conceptual spaces. Great for:
//! - Knowledge representation
//! - Semantic search
//! - Ontology modeling
//! - Recommendation systems
//!
//! ## [`ComposedGraph`]
//! Allows combining multiple graph types into a unified structure for:
//! - Cross-domain queries
//! - Multi-aspect modeling
//! - Graph federation
//! - Heterogeneous analysis
//!
//! # Example
//!
//! All graph types are built from events using projection engines:
//!
//! ```rust,ignore
//! use cim_graph::{
//!     core::{ProjectionEngine, GraphProjection, GenericGraphProjection},
//!     events::{GraphEvent, EventPayload, WorkflowPayload},
//!     graphs::{WorkflowNode, WorkflowEdge},
//! };
//! use uuid::Uuid;
//!
//! // Create events to build a workflow graph
//! let workflow_id = Uuid::new_v4();
//! let events = vec![
//!     GraphEvent {
//!         event_id: Uuid::new_v4(),
//!         aggregate_id: workflow_id,
//!         correlation_id: Uuid::new_v4(),
//!         causation_id: None,
//!         payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
//!             workflow_id,
//!             name: "Order Processing".to_string(),
//!             version: "1.0.0".to_string(),
//!         }),
//!     },
//! ];
//!
//! // Build projection from events (via NATS JetStream in production)
//! let engine = ProjectionEngine::<GenericGraphProjection<WorkflowNode, WorkflowEdge>>::new();
//! let projection = engine.project(events);
//!
//! // Query the projection
//! assert_eq!(projection.version(), 1);
//! ```

pub mod ipld;
pub mod ipld_projection;
pub mod ipld_event_chain;
pub mod ipld_projection_engine;
pub mod context;
pub mod context_projection;
pub mod workflow;
pub mod concept;
pub mod composed;
pub mod event_driven_workflow;


pub use self::ipld::IpldGraph;
pub use self::ipld_projection::{IpldProjection, IpldCommand, ipld_command_to_graph_command, IpldNode, IpldEdge};
pub use self::ipld_event_chain::{Cid, CidChain, EventPayload, IpldEventNode, EventChainBuilder, CidGenerator};
pub use self::ipld_projection_engine::{IpldGraphProjection, build_ipld_projection};
pub use self::context::ContextGraph;
pub use self::context_projection::{ContextNode, ContextEdge};
pub use self::workflow::{WorkflowGraph, WorkflowNode, WorkflowEdge, WorkflowNodeType, WorkflowProjection};
pub use self::concept::{ConceptGraph, ConceptNode, ConceptEdge, ConceptNodeType, ConceptProjection};
pub use self::composed::{ComposedGraph, ComposedNode, ComposedEdge, ComposedProjection};