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

    // ========================================================================
    // Direction tests
    // ========================================================================

    #[test]
    fn test_direction_variants() {
        let directed = Direction::Directed;
        let undirected = Direction::Undirected;
        let bidirectional = Direction::Bidirectional;

        // Test debug output
        assert!(format!("{:?}", directed).contains("Directed"));
        assert!(format!("{:?}", undirected).contains("Undirected"));
        assert!(format!("{:?}", bidirectional).contains("Bidirectional"));
    }

    #[test]
    fn test_direction_equality() {
        assert_eq!(Direction::Directed, Direction::Directed);
        assert_ne!(Direction::Directed, Direction::Undirected);
        assert_ne!(Direction::Undirected, Direction::Bidirectional);
    }

    #[test]
    fn test_direction_clone() {
        let dir = Direction::Bidirectional;
        let cloned = dir.clone();
        assert_eq!(dir, cloned);
    }

    #[test]
    fn test_direction_copy() {
        let dir1 = Direction::Directed;
        let dir2 = dir1; // Copy, not move
        assert_eq!(dir1, dir2);
    }

    #[test]
    fn test_direction_serialization() {
        let dir = Direction::Directed;
        let json = serde_json::to_string(&dir).unwrap();
        let deserialized: Direction = serde_json::from_str(&json).unwrap();
        assert_eq!(dir, deserialized);
    }

    // ========================================================================
    // GenericEdge creation tests
    // ========================================================================

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
    fn test_edge_new_generates_unique_ids() {
        let edge1 = GenericEdge::new("a", "b", ());
        let edge2 = GenericEdge::new("a", "b", ());
        // Each edge should have a unique UUID-based ID
        assert_ne!(edge1.id(), edge2.id());
    }

    #[test]
    fn test_edge_with_string_into() {
        let edge = GenericEdge::new(String::from("src"), String::from("tgt"), 100);
        assert_eq!(edge.source(), "src");
        assert_eq!(edge.target(), "tgt");
    }

    // ========================================================================
    // GenericEdge data access tests
    // ========================================================================

    #[test]
    fn test_generic_edge_data_immutable() {
        let edge = GenericEdge::new("a", "b", vec![1, 2, 3]);
        assert_eq!(edge.data(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_generic_edge_data_mutable() {
        let mut edge = GenericEdge::new("a", "b", vec![1, 2, 3]);
        edge.data_mut().push(4);
        assert_eq!(edge.data(), &vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_generic_edge_set_direction() {
        let mut edge = GenericEdge::new("a", "b", ());
        assert_eq!(edge.direction(), Direction::Directed);

        edge.set_direction(Direction::Undirected);
        assert_eq!(edge.direction(), Direction::Undirected);

        edge.set_direction(Direction::Bidirectional);
        assert_eq!(edge.direction(), Direction::Bidirectional);
    }

    // ========================================================================
    // Edge trait implementation tests
    // ========================================================================

    #[test]
    fn test_edge_trait_implementation() {
        fn assert_edge<E: Edge>(edge: &E, expected_id: &str, expected_src: &str, expected_tgt: &str) {
            assert_eq!(edge.id(), expected_id);
            assert_eq!(edge.source(), expected_src);
            assert_eq!(edge.target(), expected_tgt);
        }

        let edge = GenericEdge::with_id("e1", "src", "tgt", 0);
        assert_edge(&edge, "e1", "src", "tgt");
    }

    #[test]
    fn test_edge_id_returns_owned_string() {
        let edge = GenericEdge::with_id("test_edge", "a", "b", 0);
        let id1 = edge.id();
        let id2 = edge.id();
        assert_eq!(id1, id2);
        assert_eq!(id1, "test_edge");
    }

    // ========================================================================
    // WeightedEdgeData tests
    // ========================================================================

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

    #[test]
    fn test_weighted_edge_data_default() {
        let data = WeightedEdgeData::default();
        assert_eq!(data.weight, 1.0);
        assert!(data.label.is_none());
    }

    #[test]
    fn test_weighted_edge_data_clone() {
        let data = WeightedEdgeData {
            weight: 5.0,
            label: Some("test".to_string()),
        };
        let cloned = data.clone();
        assert_eq!(data.weight, cloned.weight);
        assert_eq!(data.label, cloned.label);
    }

    #[test]
    fn test_weighted_edge_data_serialization() {
        let data = WeightedEdgeData {
            weight: 3.14,
            label: Some("pi".to_string()),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: WeightedEdgeData = serde_json::from_str(&json).unwrap();
        assert_eq!(data.weight, deserialized.weight);
        assert_eq!(data.label, deserialized.label);
    }

    // ========================================================================
    // GenericEdge Clone and Debug tests
    // ========================================================================

    #[test]
    fn test_generic_edge_clone() {
        let edge1 = GenericEdge::with_id("e1", "a", "b", "data");
        let edge2 = edge1.clone();

        assert_eq!(edge1.id(), edge2.id());
        assert_eq!(edge1.source(), edge2.source());
        assert_eq!(edge1.target(), edge2.target());
        assert_eq!(edge1.data(), edge2.data());
        assert_eq!(edge1.direction(), edge2.direction());
    }

    #[test]
    fn test_generic_edge_debug() {
        let edge = GenericEdge::with_id("debug_edge", "src", "tgt", 42);
        let debug_str = format!("{:?}", edge);
        assert!(debug_str.contains("debug_edge"));
        assert!(debug_str.contains("src"));
        assert!(debug_str.contains("tgt"));
        assert!(debug_str.contains("42"));
    }

    // ========================================================================
    // Serialization tests
    // ========================================================================

    #[test]
    fn test_generic_edge_serialization() {
        let edge = GenericEdge::with_id("ser_edge", "a", "b", "value");
        let json = serde_json::to_string(&edge).unwrap();
        assert!(json.contains("ser_edge"));
        assert!(json.contains("value"));

        let deserialized: GenericEdge<&str> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id(), "ser_edge");
        assert_eq!(deserialized.source(), "a");
        assert_eq!(deserialized.target(), "b");
    }

    #[test]
    fn test_generic_edge_serialization_with_direction() {
        let mut edge = GenericEdge::with_id("dir_edge", "x", "y", 0);
        edge.set_direction(Direction::Bidirectional);

        let json = serde_json::to_string(&edge).unwrap();
        let deserialized: GenericEdge<i32> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.direction(), Direction::Bidirectional);
    }

    // ========================================================================
    // Complex data type tests
    // ========================================================================

    #[test]
    fn test_edge_with_complex_data() {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        struct EdgeMetadata {
            created_by: String,
            priority: i32,
            tags: Vec<String>,
        }

        let metadata = EdgeMetadata {
            created_by: "test_user".to_string(),
            priority: 5,
            tags: vec!["important".to_string(), "reviewed".to_string()],
        };

        let edge = GenericEdge::new("node1", "node2", metadata.clone());
        assert_eq!(edge.data(), &metadata);

        // Test modification
        let mut edge = edge;
        edge.data_mut().priority = 10;
        assert_eq!(edge.data().priority, 10);
    }

    #[test]
    fn test_edge_with_option_data() {
        let edge: GenericEdge<Option<String>> = GenericEdge::new("a", "b", Some("value".to_string()));
        assert_eq!(edge.data(), &Some("value".to_string()));

        let edge_none: GenericEdge<Option<String>> = GenericEdge::new("a", "b", None);
        assert_eq!(edge_none.data(), &None);
    }

    #[test]
    fn test_edge_with_hashmap_data() {
        use std::collections::HashMap;

        let mut data = HashMap::new();
        data.insert("key1".to_string(), 100);
        data.insert("key2".to_string(), 200);

        let edge = GenericEdge::new("src", "tgt", data.clone());
        assert_eq!(edge.data().get("key1"), Some(&100));
        assert_eq!(edge.data().get("key2"), Some(&200));
    }
}
