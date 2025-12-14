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
//! use cim_graph::{Result, GraphError};
//!
//! fn process_result() -> Result<String> {
//!     // All fallible operations return Result<T, GraphError>
//!     let node_id = "node-123";
//!
//!     // Error handling with GraphError variants
//!     Err(GraphError::NodeNotFound(node_id.to_string()))
//! }
//!
//! // Check error type
//! match process_result() {
//!     Ok(value) => println!("Success: {}", value),
//!     Err(GraphError::NodeNotFound(id)) => println!("Node {} not found", id),
//!     Err(e) => println!("Other error: {}", e),
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
    
    /// External error (from dependencies)
    #[error("External error: {0}")]
    External(String),
}

impl From<serde_json::Error> for GraphError {
    fn from(err: serde_json::Error) -> Self {
        GraphError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_not_found_error() {
        let err = GraphError::NodeNotFound("node123".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Node not found"));
        assert!(msg.contains("node123"));
    }

    #[test]
    fn test_edge_not_found_error() {
        let err = GraphError::EdgeNotFound("edge456".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Edge not found"));
        assert!(msg.contains("edge456"));
    }

    #[test]
    fn test_duplicate_node_error() {
        let err = GraphError::DuplicateNode("duplicate_id".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Duplicate node"));
        assert!(msg.contains("duplicate_id"));
    }

    #[test]
    fn test_duplicate_edge_error() {
        let err = GraphError::DuplicateEdge {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Duplicate edge"));
        assert!(msg.contains("A"));
        assert!(msg.contains("B"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = GraphError::TypeMismatch {
            expected: "WorkflowNode".to_string(),
            actual: "ConceptNode".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Type mismatch"));
        assert!(msg.contains("WorkflowNode"));
        assert!(msg.contains("ConceptNode"));
    }

    #[test]
    fn test_constraint_violation_error() {
        let err = GraphError::ConstraintViolation("Cannot add self-loop".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Constraint violation"));
        assert!(msg.contains("self-loop"));
    }

    #[test]
    fn test_invalid_operation_error() {
        let err = GraphError::InvalidOperation("Graph already initialized".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Invalid operation"));
        assert!(msg.contains("already initialized"));
    }

    #[test]
    fn test_not_initialized_error() {
        let err = GraphError::NotInitialized;
        let msg = err.to_string();
        assert!(msg.contains("not initialized"));
    }

    #[test]
    fn test_serialization_error() {
        let err = GraphError::SerializationError("Invalid JSON".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Serialization error"));
        assert!(msg.contains("Invalid JSON"));
    }

    #[test]
    fn test_external_error() {
        let err = GraphError::External("Connection failed".to_string());
        let msg = err.to_string();
        assert!(msg.contains("External error"));
        assert!(msg.contains("Connection failed"));
    }

    #[test]
    fn test_from_serde_json_error() {
        // Create a JSON parsing error
        let json_result: std::result::Result<serde_json::Value, _> =
            serde_json::from_str("invalid json");
        let json_err = json_result.unwrap_err();

        let graph_err: GraphError = json_err.into();

        match graph_err {
            GraphError::SerializationError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_error_is_debug() {
        let err = GraphError::NodeNotFound("test".to_string());
        // Test that Debug is implemented
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NodeNotFound"));
    }

    #[test]
    fn test_result_type_alias() {
        fn example_fn() -> Result<i32> {
            Ok(42)
        }

        fn example_err() -> Result<i32> {
            Err(GraphError::NotInitialized)
        }

        assert_eq!(example_fn().unwrap(), 42);
        assert!(example_err().is_err());
    }
}
