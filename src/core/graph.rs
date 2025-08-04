//! Core Graph trait and implementation

use crate::core::{Edge, Node};
use crate::error::{GraphError, Result};
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

/// Unique identifier for graphs
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

/// Metadata associated with a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Human-readable name
    pub name: Option<String>,
    /// Description of the graph's purpose
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modification timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Version information
    pub version: String,
    /// Additional properties
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

/// Core trait that all graph types implement
pub trait Graph: Sized {
    /// Node type for this graph
    type Node: Node;
    /// Edge type for this graph
    type Edge: Edge;

    /// Get the graph's unique identifier
    fn id(&self) -> &GraphId;

    /// Get the graph type
    fn graph_type(&self) -> GraphType;

    /// Get graph metadata
    fn metadata(&self) -> &GraphMetadata;

    /// Get mutable reference to metadata
    fn metadata_mut(&mut self) -> &mut GraphMetadata;

    /// Add a node to the graph
    fn add_node(&mut self, node: Self::Node) -> Result<String>;

    /// Remove a node and all connected edges
    fn remove_node(&mut self, node_id: &str) -> Result<Self::Node>;

    /// Add an edge between two nodes
    fn add_edge(&mut self, edge: Self::Edge) -> Result<String>;

    /// Remove an edge between nodes
    fn remove_edge(&mut self, edge_id: &str) -> Result<Self::Edge>;
    
    /// Get all edges between two nodes
    fn edges_between(&self, from: &str, to: &str) -> Vec<&Self::Edge>;

    /// Get a node by ID
    fn get_node(&self, node_id: &str) -> Option<&Self::Node>;

    /// Get a mutable reference to a node
    fn get_node_mut(&mut self, node_id: &str) -> Option<&mut Self::Node>;

    /// Get an edge by ID
    fn get_edge(&self, edge_id: &str) -> Option<&Self::Edge>;

    /// Get all node IDs
    fn node_ids(&self) -> Vec<String>;

    /// Get all edge IDs
    fn edge_ids(&self) -> Vec<String>;

    /// Get the number of nodes
    fn node_count(&self) -> usize;

    /// Get the number of edges
    fn edge_count(&self) -> usize;

    /// Check if the graph contains a node
    fn contains_node(&self, node_id: &str) -> bool {
        self.get_node(node_id).is_some()
    }

    /// Check if the graph contains an edge
    fn contains_edge(&self, edge_id: &str) -> bool {
        self.get_edge(edge_id).is_some()
    }

    /// Get neighbors of a node (outgoing edges)
    fn neighbors(&self, node_id: &str) -> Result<Vec<String>>;

    /// Serialize the graph to JSON
    fn to_json(&self) -> Result<serde_json::Value>;

    /// Clear all nodes and edges
    fn clear(&mut self);
}

/// Basic graph implementation for testing
#[derive(Debug)]
pub struct BasicGraph<N: Node, E: Edge> {
    id: GraphId,
    graph_type: GraphType,
    metadata: GraphMetadata,
    nodes: HashMap<String, N>,
    edges: HashMap<String, E>,
    // Track edge connections for neighbor queries
    adjacency: HashMap<String, Vec<String>>,
}

impl<N: Node, E: Edge> BasicGraph<N, E> {
    /// Create a new basic graph
    pub fn new(graph_type: GraphType) -> Self {
        Self {
            id: GraphId::new(),
            graph_type,
            metadata: GraphMetadata::default(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }
}

impl<N: Node, E: Edge> Graph for BasicGraph<N, E> {
    type Node = N;
    type Edge = E;

    fn id(&self) -> &GraphId {
        &self.id
    }

    fn graph_type(&self) -> GraphType {
        self.graph_type
    }

    fn metadata(&self) -> &GraphMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut GraphMetadata {
        &mut self.metadata
    }

    fn add_node(&mut self, node: Self::Node) -> Result<String> {
        let node_id = node.id();

        if self.nodes.contains_key(&node_id) {
            return Err(GraphError::DuplicateNode(node_id));
        }

        self.nodes.insert(node_id.clone(), node);
        self.adjacency.insert(node_id.clone(), Vec::new());
        self.metadata.updated_at = chrono::Utc::now();

        Ok(node_id)
    }

    fn remove_node(&mut self, node_id: &str) -> Result<Self::Node> {
        let node = self
            .nodes
            .remove(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;

        // Remove all edges connected to this node
        let edges_to_remove: Vec<String> = self
            .edges
            .iter()
            .filter(|(_, edge)| edge.source() == node_id || edge.target() == node_id)
            .map(|(id, _)| id.clone())
            .collect();

        for edge_id in edges_to_remove {
            self.edges.remove(&edge_id);
        }

        // Update adjacency
        self.adjacency.remove(node_id);
        for adj_list in self.adjacency.values_mut() {
            adj_list.retain(|id| id != node_id);
        }

        self.metadata.updated_at = chrono::Utc::now();
        Ok(node)
    }

    fn add_edge(&mut self, edge: Self::Edge) -> Result<String> {
        let edge_id = edge.id();
        let from = edge.source();
        let to = edge.target();
        
        // Verify nodes exist
        if !self.nodes.contains_key(&from) {
            return Err(GraphError::NodeNotFound(from.to_string()));
        }
        if !self.nodes.contains_key(&to) {
            return Err(GraphError::NodeNotFound(to.to_string()));
        }

        // Check for duplicate edge ID
        if self.edges.contains_key(&edge_id) {
            return Err(GraphError::DuplicateEdge {
                from: from.to_string(),
                to: to.to_string(),
            });
        }

        self.edges.insert(edge_id.clone(), edge);

        // Update adjacency
        if let Some(adj) = self.adjacency.get_mut(&from) {
            adj.push(to.to_string());
        }

        self.metadata.updated_at = chrono::Utc::now();
        Ok(edge_id)
    }

    fn remove_edge(&mut self, edge_id: &str) -> Result<Self::Edge> {
        let edge = self
            .edges
            .remove(edge_id)
            .ok_or_else(|| GraphError::EdgeNotFound(edge_id.to_string()))?;

        // Update adjacency
        if let Some(adj) = self.adjacency.get_mut(&edge.source()) {
            adj.retain(|id| id != &edge.target());
        }

        self.metadata.updated_at = chrono::Utc::now();
        Ok(edge)
    }
    
    fn edges_between(&self, from: &str, to: &str) -> Vec<&Self::Edge> {
        self.edges
            .values()
            .filter(|e| e.source() == from && e.target() == to)
            .collect()
    }

    fn get_node(&self, node_id: &str) -> Option<&Self::Node> {
        self.nodes.get(node_id)
    }

    fn get_node_mut(&mut self, node_id: &str) -> Option<&mut Self::Node> {
        self.nodes.get_mut(node_id)
    }

    fn get_edge(&self, edge_id: &str) -> Option<&Self::Edge> {
        self.edges.get(edge_id)
    }

    fn node_ids(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }

    fn edge_ids(&self) -> Vec<String> {
        self.edges.keys().cloned().collect()
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn neighbors(&self, node_id: &str) -> Result<Vec<String>> {
        self.adjacency
            .get(node_id)
            .cloned()
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "id": self.id,
            "type": self.graph_type,
            "metadata": self.metadata,
            "nodes": self.node_ids(),
            "edges": self.edge_ids(),
            "node_count": self.node_count(),
            "edge_count": self.edge_count(),
        }))
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.adjacency.clear();
        self.metadata.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock node type for testing
    #[derive(Debug, Clone)]
    struct TestNode {
        id: String,
        data: String,
    }

    impl Node for TestNode {
        fn id(&self) -> String {
            self.id.clone()
        }
    }

    // Mock edge type for testing
    #[derive(Debug, Clone)]
    struct TestEdge {
        id: String,
        source: String,
        target: String,
    }

    impl Edge for TestEdge {
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

    #[test]
    fn test_create_empty_graph() {
        let graph: BasicGraph<TestNode, TestEdge> = BasicGraph::new(GraphType::Generic);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.graph_type(), GraphType::Generic);
    }

    #[test]
    fn test_add_node() {
        let mut graph: BasicGraph<TestNode, TestEdge> = BasicGraph::new(GraphType::Generic);
        let node = TestNode {
            id: "node1".to_string(),
            data: "test".to_string(),
        };

        let result = graph.add_node(node);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "node1");
        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node("node1"));
    }

    #[test]
    fn test_duplicate_node_error() {
        let mut graph: BasicGraph<TestNode, TestEdge> = BasicGraph::new(GraphType::Generic);
        let node1 = TestNode {
            id: "node1".to_string(),
            data: "test1".to_string(),
        };
        let node2 = TestNode {
            id: "node1".to_string(), // Same ID
            data: "test2".to_string(),
        };

        assert!(graph.add_node(node1).is_ok());
        assert!(matches!(
            graph.add_node(node2),
            Err(GraphError::DuplicateNode(_))
        ));
    }
}
