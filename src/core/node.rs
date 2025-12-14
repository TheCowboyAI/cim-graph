//! Node trait and implementations

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Base trait for all node types
pub trait Node: Clone + Debug {
    /// Get the unique identifier for this node
    fn id(&self) -> String;
}

/// Generic node implementation for basic graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericNode<T> {
    id: String,
    data: T,
}

impl<T> GenericNode<T> {
    /// Create a new generic node
    pub fn new(id: impl Into<String>, data: T) -> Self {
        Self {
            id: id.into(),
            data,
        }
    }

    /// Get a reference to the node's data
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the node's data
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Clone + Debug> Node for GenericNode<T> {
    fn id(&self) -> String {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // GenericNode creation tests
    // ========================================================================

    #[test]
    fn test_generic_node_creation() {
        let node = GenericNode::new("test_id", "test_data");
        assert_eq!(node.id(), "test_id");
        assert_eq!(node.data(), &"test_data");
    }

    #[test]
    fn test_generic_node_with_string_id() {
        let node = GenericNode::new(String::from("string_id"), 42);
        assert_eq!(node.id(), "string_id");
        assert_eq!(node.data(), &42);
    }

    #[test]
    fn test_generic_node_with_complex_data() {
        #[derive(Debug, Clone, PartialEq)]
        struct ComplexData {
            name: String,
            value: f64,
            tags: Vec<String>,
        }

        let data = ComplexData {
            name: "test".to_string(),
            value: 3.14,
            tags: vec!["a".to_string(), "b".to_string()],
        };

        let node = GenericNode::new("complex", data.clone());
        assert_eq!(node.id(), "complex");
        assert_eq!(node.data(), &data);
    }

    #[test]
    fn test_generic_node_with_empty_id() {
        let node = GenericNode::new("", "data");
        assert_eq!(node.id(), "");
        assert_eq!(node.data(), &"data");
    }

    #[test]
    fn test_generic_node_with_unicode_id() {
        let node = GenericNode::new("nodo_espanol_123", "value");
        assert_eq!(node.id(), "nodo_espanol_123");
    }

    // ========================================================================
    // GenericNode data access tests
    // ========================================================================

    #[test]
    fn test_node_trait_implementation() {
        let node = GenericNode::new("node1", 42);
        // Node trait is not object-safe due to Clone requirement
        // Just test that it implements the trait
        assert_eq!(node.id(), "node1");
    }

    #[test]
    fn test_generic_node_data_immutable_access() {
        let node = GenericNode::new("id", vec![1, 2, 3]);
        let data = node.data();
        assert_eq!(data, &vec![1, 2, 3]);
    }

    #[test]
    fn test_generic_node_data_mutable_access() {
        let mut node = GenericNode::new("id", vec![1, 2, 3]);
        {
            let data = node.data_mut();
            data.push(4);
        }
        assert_eq!(node.data(), &vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_generic_node_data_mut_modify_struct() {
        #[derive(Debug, Clone, PartialEq)]
        struct MutableData {
            counter: i32,
        }

        let mut node = GenericNode::new("id", MutableData { counter: 0 });
        node.data_mut().counter += 10;
        assert_eq!(node.data().counter, 10);
    }

    // ========================================================================
    // GenericNode Clone and Debug tests
    // ========================================================================

    #[test]
    fn test_generic_node_clone() {
        let node1 = GenericNode::new("original", 100);
        let node2 = node1.clone();

        assert_eq!(node1.id(), node2.id());
        assert_eq!(node1.data(), node2.data());
    }

    #[test]
    fn test_generic_node_debug() {
        let node = GenericNode::new("debug_test", "value");
        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("value"));
    }

    // ========================================================================
    // Serialization tests
    // ========================================================================

    #[test]
    fn test_generic_node_serialization() {
        let node = GenericNode::new("ser_test", "serialize_me");
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("ser_test"));
        assert!(json.contains("serialize_me"));

        let deserialized: GenericNode<&str> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id(), "ser_test");
    }

    #[test]
    fn test_generic_node_serialization_with_numbers() {
        let node = GenericNode::new("num_node", 42i64);
        let json = serde_json::to_string(&node).unwrap();

        let deserialized: GenericNode<i64> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id(), "num_node");
        assert_eq!(deserialized.data(), &42i64);
    }

    // ========================================================================
    // Node trait tests with different implementations
    // ========================================================================

    #[test]
    fn test_node_trait_with_various_types() {
        // Test with different data types
        fn assert_node_id<N: Node>(node: &N, expected: &str) {
            assert_eq!(node.id(), expected);
        }

        let string_node = GenericNode::new("str", "data".to_string());
        let int_node = GenericNode::new("int", 123);
        let float_node = GenericNode::new("float", 1.5f64);
        let bool_node = GenericNode::new("bool", true);

        assert_node_id(&string_node, "str");
        assert_node_id(&int_node, "int");
        assert_node_id(&float_node, "float");
        assert_node_id(&bool_node, "bool");
    }

    #[test]
    fn test_node_id_returns_owned_string() {
        let node = GenericNode::new("test", 0);
        let id1 = node.id();
        let id2 = node.id();

        // Both calls should return equal strings
        assert_eq!(id1, id2);
        // They are independent copies
        assert_eq!(id1, "test");
        assert_eq!(id2, "test");
    }
}
