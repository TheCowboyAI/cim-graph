//! CIM Graph - Unified graph abstraction library
//!
//! This library provides a single, consistent interface for working with different
//! graph types while maintaining their unique semantic properties.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

pub mod core;
pub mod error;
pub mod graphs;
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
