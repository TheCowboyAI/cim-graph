//! Edge trait and implementations

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Base trait for all edge types
pub trait Edge: Clone + Debug {
    /// Get the unique identifier for this edge
    fn id(&self) -> String;

    /// Get the source node ID
    fn source(&self) -> String;

    /// Get the target node ID
    fn target(&self) -> String;
}

/// Direction of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Edge goes from source to target only
    Directed,
    /// Edge can be traversed in both directions
    Undirected,
    /// Explicitly bidirectional (two directed edges)
    Bidirectional,
}

/// Generic edge implementation for basic graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericEdge<T> {
    id: String,
    source: String,
    target: String,
    direction: Direction,
    data: T,
}

impl<T> GenericEdge<T> {
    /// Create a new directed edge
    pub fn new(source: impl Into<String>, target: impl Into<String>, data: T) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            target: target.into(),
            direction: Direction::Directed,
            data,
        }
    }

    /// Create a new edge with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        data: T,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            direction: Direction::Directed,
            data,
        }
    }

    /// Create an undirected edge
    pub fn undirected(source: impl Into<String>, target: impl Into<String>, data: T) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            target: target.into(),
            direction: Direction::Undirected,
            data,
        }
    }

    /// Get the edge direction
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Set the edge direction
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Get a reference to the edge's data
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the edge's data
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T: Clone + Debug> Edge for GenericEdge<T> {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn source(&self) -> String {
        self.source.clone()
    }

    fn target(&self) -> String {
        self.target.clone()
    }
}

/// Edge data for weighted graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedEdgeData {
    /// Weight of the edge
    pub weight: f64,
    /// Optional label
    pub label: Option<String>,
}

impl Default for WeightedEdgeData {
    fn default() -> Self {
        Self {
            weight: 1.0,
            label: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_edge_creation() {
        let edge = GenericEdge::new("node1", "node2", "test_data");
        assert_eq!(edge.source(), "node1");
        assert_eq!(edge.target(), "node2");
        assert_eq!(edge.data(), &"test_data");
        assert_eq!(edge.direction(), Direction::Directed);
    }

    #[test]
    fn test_edge_with_id() {
        let edge = GenericEdge::with_id("edge1", "a", "b", 42);
        assert_eq!(edge.id(), "edge1");
        assert_eq!(edge.source(), "a");
        assert_eq!(edge.target(), "b");
        assert_eq!(edge.data(), &42);
    }

    #[test]
    fn test_undirected_edge() {
        let edge = GenericEdge::undirected("x", "y", ());
        assert_eq!(edge.direction(), Direction::Undirected);
    }

    #[test]
    fn test_weighted_edge() {
        let data = WeightedEdgeData {
            weight: 2.5,
            label: Some("important".to_string()),
        };
        let edge = GenericEdge::new("start", "end", data.clone());
        assert_eq!(edge.data().weight, 2.5);
        assert_eq!(edge.data().label, Some("important".to_string()));
    }
}
