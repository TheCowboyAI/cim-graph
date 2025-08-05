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
//! ```rust
//! use cim_graph::graphs::{IpldGraph, ContextGraph, ComposedGraph};
//! use cim_graph::Result;
//!
//! # fn main() -> Result<()> {
//! // Create domain-specific graphs
//! let mut ipld = IpldGraph::new();
//! let cid = ipld.add_cid("QmHash123", "dag-cbor", 1024)?;
//!
//! let mut context = ContextGraph::new("sales");
//! let customer = context.add_aggregate("Customer", uuid::Uuid::new_v4(), 
//!     serde_json::json!({ "name": "Alice" }))?;
//!
//! // Compose graphs for cross-domain queries
//! let composed = ComposedGraph::builder()
//!     .add_graph("data", ipld)
//!     .add_graph("domain", context)
//!     .build()?;
//! # Ok(())
//! # }
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
pub use self::ipld_projection::{IpldProjection, IpldCommand, ipld_command_to_graph_command};
pub use self::ipld_event_chain::{Cid, CidChain, EventPayload, IpldEventNode, EventChainBuilder, CidGenerator};
pub use self::ipld_projection_engine::{IpldGraphProjection, build_ipld_projection};
pub use self::context::ContextGraph;
pub use self::workflow::WorkflowGraph;
pub use self::concept::ConceptGraph;
pub use self::composed::ComposedGraph;