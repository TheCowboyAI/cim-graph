//! Error types and result handling for CIM Graph
//!
//! This module defines the error types used throughout the library and provides
//! utilities for error handling and context.
//!
//! # Error Handling Philosophy
//!
//! CIM Graph follows these error handling principles:
//! - All fallible operations return `Result<T, GraphError>`
//! - Errors are descriptive and include context
//! - No panics in library code (except for programmer errors)
//! - Errors can be extended with additional context
//!
//! # Example
//!
//! ```rust
//! use cim_graph::{GraphBuilder, Node, Result, GraphError};
//!
//! fn process_graph() -> Result<()> {
//!     let mut graph = GraphBuilder::new().build();
//!     
//!     // This returns Result<NodeId, GraphError>
//!     let node_id = graph.add_node(Node::new("data", "type"))?;
//!     
//!     // Error handling with match
//!     match graph.get_node(node_id) {
//!         Some(node) => println!("Found node: {:?}", node),
//!         None => return Err(GraphError::NodeNotFound(node_id.to_string())),
//!     }
//!     
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Result type alias for Graph operations
///
/// This is the standard `Result` type used throughout the library,
/// with [`GraphError`] as the error type.
pub type Result<T> = std::result::Result<T, GraphError>;

/// Core error type for all graph operations
///
/// This enum represents all possible errors that can occur during
/// graph operations. It uses the `thiserror` crate for automatic
/// `Error` trait implementation.
#[derive(Error, Debug)]
pub enum GraphError {
    /// Node with the given ID was not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Edge with the given ID was not found
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

    /// Attempted to add a duplicate node
    #[error("Duplicate node: {0}")]
    DuplicateNode(String),

    /// Attempted to add a duplicate edge
    #[error("Duplicate edge between nodes: {from} -> {to}")]
    DuplicateEdge {
        /// Source node ID
        from: String,
        /// Target node ID
        to: String,
    },

    /// Type mismatch in graph operations
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        /// Expected type name
        expected: String,
        /// Actual type name
        actual: String,
    },

    /// Constraint violation
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Invalid operation for the current graph state
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Graph is not initialized
    #[error("Graph not initialized")]
    NotInitialized,

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for GraphError {
    fn from(err: serde_json::Error) -> Self {
        GraphError::SerializationError(err.to_string())
    }
}
