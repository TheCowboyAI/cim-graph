//! Core graph types for event-driven projections
//!
//! In CIM, graphs are read-only projections computed from event streams.
//! This module contains only the type definitions needed for projections.
//! The actual projection logic is in cim_graph.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Supported graph types with their semantic properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphType {
    /// Generic graph with no specific semantics
    Generic,
    /// Content-addressed graph (IPLD)
    IpldGraph,
    /// Domain object relationships (DDD)
    ContextGraph,
    /// State machine graph
    WorkflowGraph,
    /// Semantic reasoning graph
    ConceptGraph,
    /// Multi-type composition
    ComposedGraph,
}

impl GraphType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            GraphType::Generic => "generic",
            GraphType::IpldGraph => "ipld",
            GraphType::ContextGraph => "context",
            GraphType::WorkflowGraph => "workflow",
            GraphType::ConceptGraph => "concept",
            GraphType::ComposedGraph => "composed",
        }
    }
}

/// Unique identifier for graphs (aggregates in event sourcing)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub Uuid);

impl GraphId {
    /// Create a new unique graph ID
    pub fn new() -> Self {
        GraphId(Uuid::new_v4())
    }
}

impl Default for GraphId {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata associated with a graph projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Human-readable name
    pub name: Option<String>,
    /// Description of the graph's purpose
    pub description: Option<String>,
    /// Creation timestamp (from first event)
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modification timestamp (from last event)
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Version information
    pub version: String,
    /// Additional properties from events
    pub properties: HashMap<String, serde_json::Value>,
}

impl Default for GraphMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            name: None,
            description: None,
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            properties: HashMap::new(),
        }
    }
}

// Note: The Graph trait has been removed. Graphs are now represented by
// the GraphProjection trait in cim_graph.rs, which provides read-only access
// to graph state computed from event streams.
//
// For mutable operations, use GraphCommand to request changes, which will
// emit GraphEvent instances if valid. The events update the projections.

#[cfg(test)]
mod tests {
    use super::*;

    // ========== GraphType Tests ==========

    #[test]
    fn test_graph_type_generic() {
        let graph_type = GraphType::Generic;
        assert_eq!(graph_type.as_str(), "generic");
    }

    #[test]
    fn test_graph_type_ipld_graph() {
        let graph_type = GraphType::IpldGraph;
        assert_eq!(graph_type.as_str(), "ipld");
    }

    #[test]
    fn test_graph_type_context_graph() {
        let graph_type = GraphType::ContextGraph;
        assert_eq!(graph_type.as_str(), "context");
    }

    #[test]
    fn test_graph_type_workflow_graph() {
        let graph_type = GraphType::WorkflowGraph;
        assert_eq!(graph_type.as_str(), "workflow");
    }

    #[test]
    fn test_graph_type_concept_graph() {
        let graph_type = GraphType::ConceptGraph;
        assert_eq!(graph_type.as_str(), "concept");
    }

    #[test]
    fn test_graph_type_composed_graph() {
        let graph_type = GraphType::ComposedGraph;
        assert_eq!(graph_type.as_str(), "composed");
    }

    #[test]
    fn test_graph_type_clone() {
        let original = GraphType::WorkflowGraph;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_graph_type_copy() {
        let original = GraphType::ConceptGraph;
        let copied = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn test_graph_type_partial_eq() {
        assert_eq!(GraphType::Generic, GraphType::Generic);
        assert_ne!(GraphType::Generic, GraphType::IpldGraph);
        assert_ne!(GraphType::WorkflowGraph, GraphType::ConceptGraph);
    }

    #[test]
    fn test_graph_type_debug() {
        let graph_type = GraphType::ComposedGraph;
        let debug_str = format!("{:?}", graph_type);
        assert!(debug_str.contains("ComposedGraph"));
    }

    #[test]
    fn test_graph_type_serialization() {
        let types = vec![
            GraphType::Generic,
            GraphType::IpldGraph,
            GraphType::ContextGraph,
            GraphType::WorkflowGraph,
            GraphType::ConceptGraph,
            GraphType::ComposedGraph,
        ];

        for graph_type in types {
            let serialized = serde_json::to_string(&graph_type).unwrap();
            let deserialized: GraphType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(graph_type, deserialized);
        }
    }

    #[test]
    fn test_graph_type_all_variants_as_str() {
        let expected = vec![
            (GraphType::Generic, "generic"),
            (GraphType::IpldGraph, "ipld"),
            (GraphType::ContextGraph, "context"),
            (GraphType::WorkflowGraph, "workflow"),
            (GraphType::ConceptGraph, "concept"),
            (GraphType::ComposedGraph, "composed"),
        ];

        for (graph_type, expected_str) in expected {
            assert_eq!(graph_type.as_str(), expected_str);
        }
    }

    // ========== GraphId Tests ==========

    #[test]
    fn test_graph_id_new() {
        let id1 = GraphId::new();
        let id2 = GraphId::new();
        // Each new() should create a unique ID
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_graph_id_default() {
        let id1 = GraphId::default();
        let id2 = GraphId::default();
        // Default should also create unique IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_graph_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let graph_id = GraphId(uuid);
        assert_eq!(graph_id.0, uuid);
    }

    #[test]
    fn test_graph_id_clone() {
        let original = GraphId::new();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_graph_id_partial_eq() {
        let uuid = Uuid::new_v4();
        let id1 = GraphId(uuid);
        let id2 = GraphId(uuid);
        assert_eq!(id1, id2);

        let id3 = GraphId::new();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_graph_id_hash() {
        use std::collections::HashSet;

        let id1 = GraphId::new();
        let id2 = GraphId::new();
        let id1_clone = id1.clone();

        let mut set = HashSet::new();
        set.insert(id1.clone());
        set.insert(id2.clone());
        set.insert(id1_clone);

        // Should only have 2 unique IDs
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_graph_id_debug() {
        let id = GraphId::new();
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("GraphId"));
    }

    #[test]
    fn test_graph_id_serialization() {
        let id = GraphId::new();
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: GraphId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_graph_id_in_hashmap() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let id1 = GraphId::new();
        let id2 = GraphId::new();

        map.insert(id1.clone(), "graph1");
        map.insert(id2.clone(), "graph2");

        assert_eq!(map.get(&id1), Some(&"graph1"));
        assert_eq!(map.get(&id2), Some(&"graph2"));
    }

    // ========== GraphMetadata Tests ==========

    #[test]
    fn test_graph_metadata_default() {
        let metadata = GraphMetadata::default();

        assert!(metadata.name.is_none());
        assert!(metadata.description.is_none());
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.properties.is_empty());
        // Timestamps should be set to now
        assert!(metadata.created_at <= chrono::Utc::now());
        assert!(metadata.updated_at <= chrono::Utc::now());
    }

    #[test]
    fn test_graph_metadata_with_name() {
        let metadata = GraphMetadata {
            name: Some("Test Graph".to_string()),
            ..Default::default()
        };

        assert_eq!(metadata.name, Some("Test Graph".to_string()));
    }

    #[test]
    fn test_graph_metadata_with_description() {
        let metadata = GraphMetadata {
            description: Some("A test graph for unit testing".to_string()),
            ..Default::default()
        };

        assert_eq!(
            metadata.description,
            Some("A test graph for unit testing".to_string())
        );
    }

    #[test]
    fn test_graph_metadata_with_properties() {
        let mut properties = HashMap::new();
        properties.insert("author".to_string(), serde_json::json!("John Doe"));
        properties.insert("tags".to_string(), serde_json::json!(["test", "example"]));

        let metadata = GraphMetadata {
            properties,
            ..Default::default()
        };

        assert_eq!(metadata.properties["author"], "John Doe");
        assert!(metadata.properties["tags"].as_array().unwrap().len() == 2);
    }

    #[test]
    fn test_graph_metadata_clone() {
        let original = GraphMetadata {
            name: Some("Original".to_string()),
            description: Some("Test".to_string()),
            version: "2.0.0".to_string(),
            ..Default::default()
        };

        let cloned = original.clone();

        assert_eq!(original.name, cloned.name);
        assert_eq!(original.description, cloned.description);
        assert_eq!(original.version, cloned.version);
    }

    #[test]
    fn test_graph_metadata_debug() {
        let metadata = GraphMetadata {
            name: Some("Debug Test".to_string()),
            ..Default::default()
        };

        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("Debug Test"));
        assert!(debug_str.contains("GraphMetadata"));
    }

    #[test]
    fn test_graph_metadata_serialization() {
        let mut properties = HashMap::new();
        properties.insert("key".to_string(), serde_json::json!("value"));

        let metadata = GraphMetadata {
            name: Some("Serialization Test".to_string()),
            description: Some("Testing serialization".to_string()),
            version: "1.2.3".to_string(),
            properties,
            ..Default::default()
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: GraphMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.description, deserialized.description);
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.properties["key"], deserialized.properties["key"]);
    }

    #[test]
    fn test_graph_metadata_timestamps() {
        let before = chrono::Utc::now();
        let metadata = GraphMetadata::default();
        let after = chrono::Utc::now();

        assert!(metadata.created_at >= before);
        assert!(metadata.created_at <= after);
        assert!(metadata.updated_at >= before);
        assert!(metadata.updated_at <= after);
    }

    #[test]
    fn test_graph_metadata_version_update() {
        let mut metadata = GraphMetadata::default();
        assert_eq!(metadata.version, "1.0.0");

        metadata.version = "2.0.0".to_string();
        assert_eq!(metadata.version, "2.0.0");
    }

    #[test]
    fn test_graph_metadata_full_construction() {
        let now = chrono::Utc::now();
        let mut props = HashMap::new();
        props.insert("custom".to_string(), serde_json::json!({"nested": true}));

        let metadata = GraphMetadata {
            name: Some("Full Graph".to_string()),
            description: Some("A complete metadata example".to_string()),
            created_at: now,
            updated_at: now,
            version: "3.1.4".to_string(),
            properties: props,
        };

        assert!(metadata.name.is_some());
        assert!(metadata.description.is_some());
        assert_eq!(metadata.created_at, now);
        assert_eq!(metadata.updated_at, now);
        assert_eq!(metadata.version, "3.1.4");
        assert!(metadata.properties.contains_key("custom"));
    }
}