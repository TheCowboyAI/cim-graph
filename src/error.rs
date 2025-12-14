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

    // ========== Additional Coverage Tests ==========

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        let err = GraphError::NodeNotFound("chain_test".to_string());
        // GraphError has no source (leaf error)
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_display_format() {
        let errors = vec![
            (
                GraphError::NodeNotFound("n1".to_string()),
                "Node not found: n1",
            ),
            (
                GraphError::EdgeNotFound("e1".to_string()),
                "Edge not found: e1",
            ),
            (
                GraphError::DuplicateNode("dn".to_string()),
                "Duplicate node: dn",
            ),
            (
                GraphError::DuplicateEdge {
                    from: "X".to_string(),
                    to: "Y".to_string(),
                },
                "Duplicate edge between nodes: X -> Y",
            ),
            (
                GraphError::TypeMismatch {
                    expected: "A".to_string(),
                    actual: "B".to_string(),
                },
                "Type mismatch: expected A, got B",
            ),
            (
                GraphError::ConstraintViolation("cv".to_string()),
                "Constraint violation: cv",
            ),
            (
                GraphError::InvalidOperation("io".to_string()),
                "Invalid operation: io",
            ),
            (
                GraphError::NotInitialized,
                "Graph not initialized",
            ),
            (
                GraphError::SerializationError("se".to_string()),
                "Serialization error: se",
            ),
            (
                GraphError::External("ex".to_string()),
                "External error: ex",
            ),
        ];

        for (error, expected_msg) in errors {
            assert_eq!(error.to_string(), expected_msg);
        }
    }

    #[test]
    fn test_error_pattern_matching() {
        fn handle_error(err: GraphError) -> &'static str {
            match err {
                GraphError::NodeNotFound(_) => "node_not_found",
                GraphError::EdgeNotFound(_) => "edge_not_found",
                GraphError::DuplicateNode(_) => "duplicate_node",
                GraphError::DuplicateEdge { .. } => "duplicate_edge",
                GraphError::TypeMismatch { .. } => "type_mismatch",
                GraphError::ConstraintViolation(_) => "constraint_violation",
                GraphError::InvalidOperation(_) => "invalid_operation",
                GraphError::NotInitialized => "not_initialized",
                GraphError::SerializationError(_) => "serialization_error",
                GraphError::External(_) => "external",
            }
        }

        assert_eq!(
            handle_error(GraphError::NodeNotFound("".to_string())),
            "node_not_found"
        );
        assert_eq!(
            handle_error(GraphError::NotInitialized),
            "not_initialized"
        );
        assert_eq!(
            handle_error(GraphError::DuplicateEdge {
                from: "a".to_string(),
                to: "b".to_string()
            }),
            "duplicate_edge"
        );
    }

    #[test]
    fn test_result_map_err() {
        fn fallible_op() -> Result<()> {
            Err(GraphError::NotInitialized)
        }

        let result = fallible_op().map_err(|e| format!("Wrapped: {}", e));
        assert_eq!(result.unwrap_err(), "Wrapped: Graph not initialized");
    }

    #[test]
    fn test_result_question_mark_operator() {
        fn inner() -> Result<i32> {
            Err(GraphError::NodeNotFound("inner".to_string()))
        }

        fn outer() -> Result<i32> {
            let _ = inner()?;
            Ok(42)
        }

        let result = outer();
        assert!(result.is_err());
        match result.unwrap_err() {
            GraphError::NodeNotFound(id) => assert_eq!(id, "inner"),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_with_empty_strings() {
        let errors = vec![
            GraphError::NodeNotFound("".to_string()),
            GraphError::EdgeNotFound("".to_string()),
            GraphError::DuplicateNode("".to_string()),
            GraphError::ConstraintViolation("".to_string()),
            GraphError::InvalidOperation("".to_string()),
            GraphError::SerializationError("".to_string()),
            GraphError::External("".to_string()),
        ];

        for error in errors {
            // Should not panic with empty strings
            let _ = error.to_string();
            let _ = format!("{:?}", error);
        }
    }

    #[test]
    fn test_error_with_unicode() {
        let err = GraphError::NodeNotFound("node_\u{1F600}_unicode".to_string());
        let msg = err.to_string();
        assert!(msg.contains("\u{1F600}"));
    }

    #[test]
    fn test_error_with_long_string() {
        let long_string = "x".repeat(10000);
        let err = GraphError::NodeNotFound(long_string.clone());
        let msg = err.to_string();
        assert!(msg.contains(&long_string));
    }

    #[test]
    fn test_duplicate_edge_extraction() {
        let err = GraphError::DuplicateEdge {
            from: "source_node".to_string(),
            to: "target_node".to_string(),
        };

        if let GraphError::DuplicateEdge { from, to } = err {
            assert_eq!(from, "source_node");
            assert_eq!(to, "target_node");
        } else {
            panic!("Wrong error variant");
        }
    }

    #[test]
    fn test_type_mismatch_extraction() {
        let err = GraphError::TypeMismatch {
            expected: "ExpectedType".to_string(),
            actual: "ActualType".to_string(),
        };

        if let GraphError::TypeMismatch { expected, actual } = err {
            assert_eq!(expected, "ExpectedType");
            assert_eq!(actual, "ActualType");
        } else {
            panic!("Wrong error variant");
        }
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Test various JSON parse errors
        let invalid_jsons = vec![
            "}{",
            "[1,2,",
            r#"{"key": undefined}"#,
            "null null",
        ];

        for invalid_json in invalid_jsons {
            let json_result: std::result::Result<serde_json::Value, _> =
                serde_json::from_str(invalid_json);

            if let Err(json_err) = json_result {
                let graph_err: GraphError = json_err.into();
                match graph_err {
                    GraphError::SerializationError(msg) => {
                        assert!(!msg.is_empty());
                    }
                    _ => panic!("Expected SerializationError"),
                }
            }
        }
    }

    #[test]
    fn test_result_ok_or_err() {
        fn get_node_or_fail(should_succeed: bool) -> Result<String> {
            if should_succeed {
                Ok("found".to_string())
            } else {
                Err(GraphError::NodeNotFound("missing".to_string()))
            }
        }

        assert!(get_node_or_fail(true).is_ok());
        assert!(get_node_or_fail(false).is_err());

        assert_eq!(get_node_or_fail(true).unwrap(), "found");
    }

    #[test]
    fn test_error_chaining_with_and_then() {
        fn step1() -> Result<i32> {
            Ok(10)
        }

        fn step2(val: i32) -> Result<i32> {
            if val > 5 {
                Ok(val * 2)
            } else {
                Err(GraphError::InvalidOperation("Value too small".to_string()))
            }
        }

        let result = step1().and_then(step2);
        assert_eq!(result.unwrap(), 20);

        fn step1_fail() -> Result<i32> {
            Err(GraphError::NotInitialized)
        }

        let result2 = step1_fail().and_then(step2);
        assert!(matches!(result2, Err(GraphError::NotInitialized)));
    }
}
