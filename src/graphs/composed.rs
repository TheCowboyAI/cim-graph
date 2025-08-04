//! Composed graph supporting multiple graph types in one structure
//! 
//! Allows mixing different node and edge types with cross-type relationships

use crate::core::{EventGraph, EventHandler, GraphBuilder, GraphType, Node};
use crate::error::{GraphError, Result};
use crate::graphs::{
    ipld::{IpldNode, IpldEdge},
    context::{ContextNode, ContextEdge},
    workflow::{WorkflowNode, WorkflowEdge},
    concept::{ConceptNode, ConceptEdge},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;

/// Layer types in a composed graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    /// Domain/context layer
    Domain,
    /// Process/workflow layer
    Process,
    /// Knowledge/concept layer
    Knowledge,
    /// Data/IPLD layer
    Data,
    /// Custom layer
    Custom,
}

/// Unified node type that can hold any graph-specific node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedNode {
    /// IPLD content-addressed node
    Ipld(IpldNode),
    /// Context/domain node
    Context(ContextNode),
    /// Workflow state node
    Workflow(WorkflowNode),
    /// Concept/semantic node
    Concept(ConceptNode),
}

impl ComposedNode {
    /// Get the node type as a string
    pub fn node_type(&self) -> &'static str {
        match self {
            ComposedNode::Ipld(_) => "ipld",
            ComposedNode::Context(_) => "context",
            ComposedNode::Workflow(_) => "workflow",
            ComposedNode::Concept(_) => "concept",
        }
    }
    
    /// Get a human-readable label
    pub fn label(&self) -> String {
        match self {
            ComposedNode::Ipld(n) => format!("IPLD:{}", n.id()),
            ComposedNode::Context(n) => n.name().to_string(),
            ComposedNode::Workflow(n) => n.name().to_string(),
            ComposedNode::Concept(n) => n.label().to_string(),
        }
    }
}

impl crate::core::Node for ComposedNode {
    fn id(&self) -> String {
        match self {
            ComposedNode::Ipld(n) => n.id(),
            ComposedNode::Context(n) => n.id(),
            ComposedNode::Workflow(n) => n.id(),
            ComposedNode::Concept(n) => n.id(),
        }
    }
}

/// Unified edge type that can hold any graph-specific edge or cross-type edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComposedEdge {
    /// IPLD link
    Ipld(IpldEdge),
    /// Context relationship
    Context(ContextEdge),
    /// Workflow transition
    Workflow(WorkflowEdge),
    /// Concept relation
    Concept(ConceptEdge),
    /// Cross-type edge
    CrossType {
        id: String,
        source: String,
        target: String,
        source_type: String,
        target_type: String,
        relation: String,
        metadata: serde_json::Value,
    },
}

impl ComposedEdge {
    /// Create a cross-type edge
    pub fn cross_type(
        source: impl Into<String>,
        target: impl Into<String>,
        source_type: impl Into<String>,
        target_type: impl Into<String>,
        relation: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let target = target.into();
        let relation = relation.into();
        let id = format!("{}->{}:{}", source, target, relation);
        
        ComposedEdge::CrossType {
            id,
            source,
            target,
            source_type: source_type.into(),
            target_type: target_type.into(),
            relation,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Get edge type as string
    pub fn edge_type(&self) -> &'static str {
        match self {
            ComposedEdge::Ipld(_) => "ipld",
            ComposedEdge::Context(_) => "context",
            ComposedEdge::Workflow(_) => "workflow",
            ComposedEdge::Concept(_) => "concept",
            ComposedEdge::CrossType { .. } => "cross-type",
        }
    }
}

impl crate::core::Edge for ComposedEdge {
    fn id(&self) -> String {
        match self {
            ComposedEdge::Ipld(e) => e.id(),
            ComposedEdge::Context(e) => e.id(),
            ComposedEdge::Workflow(e) => e.id(),
            ComposedEdge::Concept(e) => e.id(),
            ComposedEdge::CrossType { id, .. } => id.clone(),
        }
    }
    
    fn source(&self) -> String {
        match self {
            ComposedEdge::Ipld(e) => e.source(),
            ComposedEdge::Context(e) => e.source(),
            ComposedEdge::Workflow(e) => e.source(),
            ComposedEdge::Concept(e) => e.source(),
            ComposedEdge::CrossType { source, .. } => source.clone(),
        }
    }
    
    fn target(&self) -> String {
        match self {
            ComposedEdge::Ipld(e) => e.target(),
            ComposedEdge::Context(e) => e.target(),
            ComposedEdge::Workflow(e) => e.target(),
            ComposedEdge::Concept(e) => e.target(),
            ComposedEdge::CrossType { target, .. } => target.clone(),
        }
    }
}

/// Multi-type composed graph
pub struct ComposedGraph {
    /// Underlying event-driven graph
    graph: EventGraph<ComposedNode, ComposedEdge>,
    /// Node type registry
    node_types: HashMap<String, String>, // node_id -> type
    /// Type-specific constraints
    constraints: Vec<TypeConstraint>,
    /// Layers in the graph
    layers: HashMap<String, LayerType>,
}

/// Constraint for cross-type relationships
#[derive(Debug, Clone)]
struct TypeConstraint {
    name: String,
    source_type: String,
    target_type: String,
    allowed_relations: Vec<String>,
}

impl ComposedGraph {
    /// Create a new composed graph
    pub fn new() -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::ComposedGraph)
            .build_event()
            .expect("Failed to create composed graph");
            
        Self {
            graph,
            node_types: HashMap::new(),
            constraints: Self::default_constraints(),
            layers: HashMap::new(),
        }
    }
    
    /// Create with event handler
    pub fn with_handler(handler: Arc<dyn EventHandler>) -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::ComposedGraph)
            .add_handler(handler)
            .build_event()
            .expect("Failed to create composed graph");
            
        Self {
            graph,
            node_types: HashMap::new(),
            constraints: Self::default_constraints(),
            layers: HashMap::new(),
        }
    }
    
    /// Default cross-type constraints
    fn default_constraints() -> Vec<TypeConstraint> {
        vec![
            TypeConstraint {
                name: "Workflow triggers domain events".to_string(),
                source_type: "workflow".to_string(),
                target_type: "context".to_string(),
                allowed_relations: vec!["triggers".to_string(), "emits".to_string()],
            },
            TypeConstraint {
                name: "Concepts describe domains".to_string(),
                source_type: "concept".to_string(),
                target_type: "context".to_string(),
                allowed_relations: vec!["describes".to_string(), "models".to_string()],
            },
            TypeConstraint {
                name: "IPLD stores workflow state".to_string(),
                source_type: "workflow".to_string(),
                target_type: "ipld".to_string(),
                allowed_relations: vec!["stores_state_in".to_string()],
            },
        ]
    }
    
    /// Add a node of any type
    pub fn add_node(&mut self, node: ComposedNode) -> Result<String> {
        let id = node.id();
        let node_type = node.node_type().to_string();
        
        self.graph.add_node(node)?;
        self.node_types.insert(id.clone(), node_type);
        
        Ok(id)
    }
    
    /// Add an edge of any type
    pub fn add_edge(&mut self, edge: ComposedEdge) -> Result<String> {
        // Validate cross-type constraints
        if let ComposedEdge::CrossType { source_type, target_type, relation, .. } = &edge {
            if !self.validate_cross_type(source_type, target_type, relation) {
                return Err(GraphError::ConstraintViolation(format!(
                    "Cross-type relation '{}' not allowed between {} and {}",
                    relation, source_type, target_type
                )));
            }
        }
        
        self.graph.add_edge(edge)
    }
    
    /// Validate cross-type relationship
    fn validate_cross_type(&self, source_type: &str, target_type: &str, relation: &str) -> bool {
        // Check if any constraint allows this relationship
        self.constraints.iter().any(|c| {
            c.source_type == source_type &&
            c.target_type == target_type &&
            c.allowed_relations.contains(&relation.to_string())
        })
    }
    
    /// Add a custom type constraint
    pub fn add_constraint(
        &mut self,
        name: impl Into<String>,
        source_type: impl Into<String>,
        target_type: impl Into<String>,
        allowed_relations: Vec<String>,
    ) {
        self.constraints.push(TypeConstraint {
            name: name.into(),
            source_type: source_type.into(),
            target_type: target_type.into(),
            allowed_relations,
        });
    }
    
    /// Get all nodes of a specific type
    pub fn get_nodes_by_type(&self, node_type: &str) -> Vec<&ComposedNode> {
        self.node_types
            .iter()
            .filter(|(_, t)| *t == node_type)
            .filter_map(|(id, _)| self.graph.get_node(id))
            .collect()
    }
    
    /// Get cross-type relationships
    pub fn get_cross_type_edges(&self) -> Vec<&ComposedEdge> {
        self.graph.edge_ids()
            .into_iter()
            .filter_map(|id| self.graph.get_edge(&id))
            .filter(|e| matches!(e, ComposedEdge::CrossType { .. }))
            .collect()
    }
    
    /// Create a projection of a single type
    pub fn project_type(&self, node_type: &str) -> HashMap<String, Vec<String>> {
        let mut projection = HashMap::new();
        
        // Get all nodes of the requested type
        let typed_nodes: Vec<_> = self.node_types
            .iter()
            .filter(|(_, t)| *t == node_type)
            .map(|(id, _)| id.clone())
            .collect();
        
        // For each typed node, find connections to other typed nodes
        for node_id in &typed_nodes {
            let mut connections = Vec::new();
            
            // Check outgoing edges
            if let Ok(neighbors) = self.graph.neighbors(node_id) {
                for neighbor in neighbors {
                    if typed_nodes.contains(&neighbor) {
                        connections.push(neighbor);
                    }
                }
            }
            
            projection.insert(node_id.clone(), connections);
        }
        
        projection
    }
    
    /// Analyze cross-type connectivity
    pub fn analyze_cross_type_connectivity(&self) -> HashMap<(String, String), usize> {
        let mut connectivity = HashMap::new();
        
        for edge in self.get_cross_type_edges() {
            if let ComposedEdge::CrossType { source_type, target_type, .. } = edge {
                let key = (source_type.clone(), target_type.clone());
                *connectivity.entry(key).or_insert(0) += 1;
            }
        }
        
        connectivity
    }
    
    /// Add a layer to the composed graph
    pub fn add_layer(&mut self, name: &str, layer_type: LayerType) -> Result<()> {
        if self.layers.contains_key(name) {
            return Err(GraphError::DuplicateNode(name.to_string()));
        }
        self.layers.insert(name.to_string(), layer_type);
        Ok(())
    }
    
    /// Connect nodes across layers
    pub fn connect_layers(
        &mut self,
        source_layer: &str,
        source_node: &str,
        target_layer: &str,
        target_node: &str,
        connection_type: &str,
    ) -> Result<String> {
        // Create cross-layer connection
        let edge = ComposedEdge::CrossType {
            id: format!("{}:{}:{}:{}", source_layer, source_node, target_layer, target_node),
            source: source_node.to_string(),
            target: target_node.to_string(),
            source_type: source_layer.to_string(),
            target_type: target_layer.to_string(),
            relation: connection_type.to_string(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        };
        
        self.add_edge(edge)
    }
    
    /// Get all layers
    pub fn get_layers(&self) -> Vec<(String, LayerType)> {
        self.layers.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
    
    /// Get layer statistics
    pub fn get_layer_stats(&self, layer_name: &str) -> Result<(usize, usize)> {
        if !self.layers.contains_key(layer_name) {
            return Err(GraphError::NodeNotFound(layer_name.to_string()));
        }
        
        // Count nodes and edges for this layer
        let node_count = self.node_types.iter()
            .filter(|(_, node_type)| node_type.as_str() == layer_name)
            .count();
            
        let edge_count = self.graph.edge_ids()
            .iter()
            .filter(|edge_id| {
                self.graph.get_edge(edge_id)
                    .map(|edge| match edge {
                        ComposedEdge::CrossType { source_type, target_type, .. } => {
                            source_type == layer_name || target_type == layer_name
                        }
                        _ => true, // Count other edges too
                    })
                    .unwrap_or(false)
            })
            .count();
            
        Ok((node_count, edge_count))
    }
    
    /// Get cross-layer connections
    pub fn get_cross_layer_connections(&self) -> Vec<CrossLayerConnection> {
        let mut connections = Vec::new();
        
        for edge_id in self.graph.edge_ids() {
            if let Some(edge) = self.graph.get_edge(&edge_id) {
                if let ComposedEdge::CrossType { source, target, source_type, target_type, relation, .. } = edge {
                    connections.push(CrossLayerConnection {
                        source_layer: source_type.clone(),
                        source_node: source.clone(),
                        target_layer: target_type.clone(),
                        target_node: target.clone(),
                        connection_type: relation.clone(),
                    });
                }
            }
        }
        
        connections
    }
    
    /// Get the underlying graph
    pub fn graph(&self) -> &EventGraph<ComposedNode, ComposedEdge> {
        &self.graph
    }
    
    /// Get mutable access to the underlying graph
    pub fn graph_mut(&mut self) -> &mut EventGraph<ComposedNode, ComposedEdge> {
        &mut self.graph
    }
}

/// Cross-layer connection information
#[derive(Debug, Clone)]
pub struct CrossLayerConnection {
    /// Source layer name
    pub source_layer: String,
    /// Source node ID
    pub source_node: String,
    /// Target layer name
    pub target_layer: String,
    /// Target node ID
    pub target_node: String,
    /// Connection type
    pub connection_type: String,
}

impl Default for ComposedGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::workflow::StateType;
    use crate::graphs::concept::ConceptType;
    
    #[test]
    fn test_composed_graph_creation() {
        let graph = ComposedGraph::new();
        assert_eq!(graph.graph().node_count(), 0);
        assert_eq!(graph.graph().graph_type(), GraphType::ComposedGraph);
    }
    
    #[test]
    fn test_mixed_types() {
        let mut graph = ComposedGraph::new();
        
        // Add different node types
        let workflow_node = WorkflowNode::new("state1", "Processing", StateType::Normal);
        let concept_node = ConceptNode::new("ProcessConcept", "Process", ConceptType::Class);
        
        graph.add_node(ComposedNode::Workflow(workflow_node)).unwrap();
        graph.add_node(ComposedNode::Concept(concept_node)).unwrap();
        
        // Add cross-type edge
        let edge = ComposedEdge::cross_type(
            "ProcessConcept",
            "state1",
            "concept",
            "workflow",
            "describes"
        );
        
        // This should fail due to constraint
        assert!(graph.add_edge(edge).is_err());
    }
    
    #[test]
    fn test_type_projection() {
        let mut graph = ComposedGraph::new();
        
        // Add workflow nodes
        let s1 = WorkflowNode::new("s1", "State1", StateType::Normal);
        let s2 = WorkflowNode::new("s2", "State2", StateType::Normal);
        
        graph.add_node(ComposedNode::Workflow(s1)).unwrap();
        graph.add_node(ComposedNode::Workflow(s2)).unwrap();
        
        // Add workflow edge
        let edge = WorkflowEdge::new("s1", "s2");
        graph.add_edge(ComposedEdge::Workflow(edge)).unwrap();
        
        // Add concept node
        let concept = ConceptNode::new("c1", "Concept1", ConceptType::Class);
        graph.add_node(ComposedNode::Concept(concept)).unwrap();
        
        // Project workflow type
        let projection = graph.project_type("workflow");
        assert_eq!(projection.len(), 2);
        assert!(projection["s1"].contains(&"s2".to_string()));
    }
    
    #[test]
    fn test_custom_constraints() {
        let mut graph = ComposedGraph::new();
        
        // Add custom constraint
        graph.add_constraint(
            "Custom rule",
            "workflow",
            "concept",
            vec!["models".to_string()],
        );
        
        // Add nodes
        let workflow = WorkflowNode::new("w1", "Workflow1", StateType::Normal);
        let concept = ConceptNode::new("c1", "Concept1", ConceptType::Class);
        
        graph.add_node(ComposedNode::Workflow(workflow)).unwrap();
        graph.add_node(ComposedNode::Concept(concept)).unwrap();
        
        // This should now succeed
        let edge = ComposedEdge::cross_type("w1", "c1", "workflow", "concept", "models");
        assert!(graph.add_edge(edge).is_ok());
    }
}