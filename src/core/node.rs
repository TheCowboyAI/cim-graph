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

    #[test]
    fn test_generic_node_creation() {
        let node = GenericNode::new("test_id", "test_data");
        assert_eq!(node.id(), "test_id");
        assert_eq!(node.data(), &"test_data");
    }

    #[test]
    fn test_node_trait_implementation() {
        let node = GenericNode::new("node1", 42);
        // Node trait is not object-safe due to Clone requirement
        // Just test that it implements the trait
        assert_eq!(node.id(), "node1");
    }
}
