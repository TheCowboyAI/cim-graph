//! IPLD (InterPlanetary Linked Data) graph implementation
//! 
//! Content-addressed graph where nodes are identified by their content hash (CID)

use crate::core::{EventGraph, EventHandler, GraphBuilder, GraphType};
use crate::error::{GraphError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Content Identifier (CID) - simplified version for this implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(String);

impl Cid {
    /// Create a new CID from a string
    pub fn new(s: impl Into<String>) -> Self {
        Cid(s.into())
    }
    
    /// Get the CID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// IPLD node containing content-addressed data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldNode {
    /// Content identifier (hash of the data)
    cid: Cid,
    /// The actual data stored in the node
    data: serde_json::Value,
    /// Links to other IPLD nodes
    links: HashMap<String, Cid>,
}

impl IpldNode {
    /// Create a new IPLD node
    pub fn new(cid: Cid, data: serde_json::Value) -> Self {
        Self {
            cid,
            data,
            links: HashMap::new(),
        }
    }
    
    /// Add a link to another IPLD node
    pub fn add_link(&mut self, name: impl Into<String>, target: Cid) {
        self.links.insert(name.into(), target);
    }
    
    /// Get a link by name
    pub fn get_link(&self, name: &str) -> Option<&Cid> {
        self.links.get(name)
    }
    
    /// Get all links
    pub fn links(&self) -> &HashMap<String, Cid> {
        &self.links
    }
    
    /// Get the data
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
    
    /// Get mutable data
    pub fn data_mut(&mut self) -> &mut serde_json::Value {
        &mut self.data
    }
}

impl crate::core::Node for IpldNode {
    fn id(&self) -> String {
        self.cid.0.clone()
    }
}

/// IPLD edge representing a named link between content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpldEdge {
    /// Unique edge identifier
    id: String,
    /// Source CID
    source: Cid,
    /// Target CID
    target: Cid,
    /// Link name/label
    label: String,
}

impl IpldEdge {
    /// Create a new IPLD edge
    pub fn new(source: Cid, target: Cid, label: impl Into<String>) -> Self {
        let label = label.into();
        let id = format!("{}:{}:{}", source.as_str(), label, target.as_str());
        Self {
            id,
            source,
            target,
            label,
        }
    }
    
    /// Get the label
    pub fn label(&self) -> &str {
        &self.label
    }
}

impl crate::core::Edge for IpldEdge {
    fn id(&self) -> String {
        self.id.clone()
    }
    
    fn source(&self) -> String {
        self.source.0.clone()
    }
    
    fn target(&self) -> String {
        self.target.0.clone()
    }
}

/// Content-addressed graph for IPLD data
pub struct IpldGraph {
    /// Underlying event-driven graph
    graph: EventGraph<IpldNode, IpldEdge>,
    /// Content store mapping CID to content
    content_store: HashMap<Cid, serde_json::Value>,
}

impl IpldGraph {
    /// Create a new IPLD graph
    pub fn new() -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::IpldGraph)
            .build_event()
            .expect("Failed to create IPLD graph");
            
        Self {
            graph,
            content_store: HashMap::new(),
        }
    }
    
    /// Create a new IPLD graph with an event handler
    pub fn with_handler(handler: Arc<dyn EventHandler>) -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::IpldGraph)
            .add_handler(handler)
            .build_event()
            .expect("Failed to create IPLD graph");
            
        Self {
            graph,
            content_store: HashMap::new(),
        }
    }
    
    /// Add content to the graph, returning its CID
    pub fn add_content(&mut self, data: serde_json::Value) -> Result<Cid> {
        // In a real implementation, this would compute the actual content hash
        let cid = Cid::new(format!("Qm{}", uuid::Uuid::new_v4().to_string().replace("-", "")));
        
        // Store the content
        self.content_store.insert(cid.clone(), data.clone());
        
        // Create and add the node
        let node = IpldNode::new(cid.clone(), data);
        self.graph.add_node(node)?;
        
        Ok(cid)
    }
    
    /// Add a named link between two pieces of content
    pub fn add_link(&mut self, from: &Cid, to: &Cid, label: impl Into<String>) -> Result<String> {
        // Verify both nodes exist
        if !self.graph.contains_node(from.as_str()) {
            return Err(GraphError::NodeNotFound(from.0.clone()));
        }
        if !self.graph.contains_node(to.as_str()) {
            return Err(GraphError::NodeNotFound(to.0.clone()));
        }
        
        let label = label.into();
        
        // Update the source node's links
        if let Some(node) = self.graph.get_node_mut(from.as_str()) {
            node.add_link(label.clone(), to.clone());
        }
        
        // Create and add the edge
        let edge = IpldEdge::new(from.clone(), to.clone(), label);
        self.graph.add_edge(edge)
    }
    
    /// Get content by CID
    pub fn get_content(&self, cid: &Cid) -> Option<&serde_json::Value> {
        self.content_store.get(cid)
    }
    
    /// Get a node by CID
    pub fn get_node(&self, cid: &Cid) -> Option<&IpldNode> {
        self.graph.get_node(cid.as_str())
    }
    
    /// Traverse the graph from a starting CID
    pub fn traverse(&self, start: &Cid, max_depth: usize) -> Vec<Cid> {
        let mut visited = Vec::new();
        let mut queue = vec![(start.clone(), 0)];
        let mut seen = std::collections::HashSet::new();
        
        while let Some((cid, depth)) = queue.pop() {
            if depth > max_depth || !seen.insert(cid.clone()) {
                continue;
            }
            
            visited.push(cid.clone());
            
            if let Some(node) = self.get_node(&cid) {
                for (_, target_cid) in node.links() {
                    queue.push((target_cid.clone(), depth + 1));
                }
            }
        }
        
        visited
    }
    
    /// Get the underlying graph for advanced operations
    pub fn graph(&self) -> &EventGraph<IpldNode, IpldEdge> {
        &self.graph
    }
    
    /// Get mutable access to the underlying graph
    pub fn graph_mut(&mut self) -> &mut EventGraph<IpldNode, IpldEdge> {
        &mut self.graph
    }
}

impl Default for IpldGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ipld_graph_creation() {
        let graph = IpldGraph::new();
        assert_eq!(graph.graph().node_count(), 0);
        assert_eq!(graph.graph().graph_type(), GraphType::IpldGraph);
    }
    
    #[test]
    fn test_add_content() {
        let mut graph = IpldGraph::new();
        
        let data = serde_json::json!({
            "name": "example",
            "value": 42
        });
        
        let cid = graph.add_content(data.clone()).unwrap();
        
        // Verify content is stored
        let stored = graph.get_content(&cid).unwrap();
        assert_eq!(stored, &data);
        
        // Verify node exists
        let node = graph.get_node(&cid).unwrap();
        assert_eq!(node.data(), &data);
    }
    
    #[test]
    fn test_add_links() {
        let mut graph = IpldGraph::new();
        
        // Add two pieces of content
        let data1 = serde_json::json!({"type": "folder", "name": "root"});
        let data2 = serde_json::json!({"type": "file", "name": "readme.txt"});
        
        let cid1 = graph.add_content(data1).unwrap();
        let cid2 = graph.add_content(data2).unwrap();
        
        // Link them
        graph.add_link(&cid1, &cid2, "contains").unwrap();
        
        // Verify link exists
        let node1 = graph.get_node(&cid1).unwrap();
        assert_eq!(node1.get_link("contains"), Some(&cid2));
    }
    
    #[test]
    fn test_traverse() {
        let mut graph = IpldGraph::new();
        
        // Create a small DAG
        let root = graph.add_content(serde_json::json!({"name": "root"})).unwrap();
        let child1 = graph.add_content(serde_json::json!({"name": "child1"})).unwrap();
        let child2 = graph.add_content(serde_json::json!({"name": "child2"})).unwrap();
        let grandchild = graph.add_content(serde_json::json!({"name": "grandchild"})).unwrap();
        
        graph.add_link(&root, &child1, "left").unwrap();
        graph.add_link(&root, &child2, "right").unwrap();
        graph.add_link(&child1, &grandchild, "child").unwrap();
        
        // Traverse from root
        let visited = graph.traverse(&root, 2);
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&root));
        assert!(visited.contains(&child1));
        assert!(visited.contains(&child2));
        assert!(visited.contains(&grandchild));
    }
}