//! CIM Graph - Unified graph abstraction library
//!
//! A high-performance, type-safe graph library that unifies multiple graph paradigms
//! under a single, consistent API. CIM Graph provides specialized graph types for
//! different domains while maintaining semantic clarity and zero-cost abstractions.
//!
//! # Overview
//!
//! CIM Graph consolidates various graph operations into a unified interface:
//!
//! - **IPLD Graphs**: Content-addressed data relationships and Markov chains
//! - **Context Graphs**: Domain-Driven Design object relationships and hierarchies
//! - **Workflow Graphs**: State machines and workflow transitions
//! - **Concept Graphs**: Semantic reasoning and conceptual spaces
//! - **Composed Graphs**: Multi-domain graph compositions with cross-graph queries
//!
//! # Quick Start
//!
//! ```rust
//! use cim_graph::{GraphBuilder, Node, Edge, Result};
//!
//! # fn main() -> Result<()> {
//! // Create a simple graph
//! let mut graph = GraphBuilder::new()
//!     .with_capacity(100, 200)
//!     .build();
//!
//! // Add nodes
//! let alice = graph.add_node(Node::new("Alice", "Person"))?;
//! let bob = graph.add_node(Node::new("Bob", "Person"))?;
//!
//! // Connect nodes
//! graph.add_edge(alice, bob, Edge::new("knows"))?;
//!
//! // Query the graph
//! let neighbors = graph.neighbors(alice)?;
//! assert_eq!(neighbors.len(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! # Crate Features
//!
//! - `async` - Enables async/await support for graph operations
//! - `full` - Enables all optional features
//!
//! # Modules
//!
//! - [`algorithms`] - Graph algorithms for pathfinding, analysis, and metrics
//! - [`core`] - Core graph traits and implementations
//! - [`error`] - Error types and result handling
//! - [`graphs`] - Specialized graph types (IPLD, Context, Workflow, Concept)
//! - [`serde_support`] - Serialization and deserialization support
//! - [`types`] - Common type definitions and utilities

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod algorithms;
pub mod core;
pub mod error;
pub mod graphs;
pub mod serde_support;
pub mod types;

// Re-export commonly used types
pub use crate::core::{Edge, EventGraph, EventHandler, Graph, GraphBuilder, GraphEvent, Node};
pub use crate::error::{GraphError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        use crate::core::edge::GenericEdge;
        use crate::core::node::GenericNode;

        // Ensure basic imports work
        type TestNode = GenericNode<String>;
        type TestEdge = GenericEdge<()>;
        let _ = GraphBuilder::<TestNode, TestEdge>::new();
    }
}
