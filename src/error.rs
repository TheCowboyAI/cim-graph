//! Error types for the CIM Graph library

use thiserror::Error;

/// Result type alias for Graph operations
pub type Result<T> = std::result::Result<T, GraphError>;

/// Core error type for all graph operations
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
