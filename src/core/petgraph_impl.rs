//! Event-driven wrapper around petgraph

use crate::core::{Edge, EventHandler, GraphEvent, GraphId, GraphMetadata, GraphType, Node};
use crate::error::{GraphError, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::sync::Arc;

/// Event-driven graph implementation using petgraph
pub struct EventGraph<N: Node, E: Edge> {
    id: GraphId,
    graph_type: GraphType,
    metadata: GraphMetadata,
    
    // Petgraph as the underlying engine
    graph: DiGraph<N, E>,
    
    // Mappings between our string IDs and petgraph's NodeIndex
    node_id_to_index: HashMap<String, NodeIndex>,
    index_to_node_id: HashMap<NodeIndex, String>,
    
    // Edge ID to petgraph edge index
    edge_id_to_index: HashMap<String, petgraph::graph::EdgeIndex>,
    
    // Event handlers
    event_handlers: Vec<Arc<dyn EventHandler>>,
}

impl<N: Node, E: Edge> EventGraph<N, E> {
    /// Create a new event-driven graph
    pub fn new(graph_type: GraphType) -> Self {
        let id = GraphId::new();
        let graph = DiGraph::new();
        
        Self {
            id,
            graph_type,
            metadata: GraphMetadata::default(),
            graph,
            node_id_to_index: HashMap::new(),
            index_to_node_id: HashMap::new(),
            edge_id_to_index: HashMap::new(),
            event_handlers: Vec::new(),
        }
    }
    
    /// Add an event handler
    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.event_handlers.push(handler);
    }
    
    /// Emit an event to all handlers
    fn emit_event(&self, event: GraphEvent) {
        for handler in &self.event_handlers {
            handler.handle_event(&event);
        }
    }
    
    /// Get the underlying petgraph
    pub fn petgraph(&self) -> &DiGraph<N, E> {
        &self.graph
    }
    
    /// Get mutable reference to the underlying petgraph
    pub fn petgraph_mut(&mut self) -> &mut DiGraph<N, E> {
        &mut self.graph
    }
}

impl<N: Node + Clone, E: Edge + Clone> EventGraph<N, E> {
    /// Get unique identifier
    pub fn id(&self) -> &GraphId {
        &self.id
    }
    
    /// Get graph type
    pub fn graph_type(&self) -> GraphType {
        self.graph_type
    }
    
    /// Get metadata
    pub fn metadata(&self) -> &GraphMetadata {
        &self.metadata
    }
    
    /// Get mutable metadata
    pub fn metadata_mut(&mut self) -> &mut GraphMetadata {
        &mut self.metadata
    }
    
    /// Add a node
    pub fn add_node(&mut self, node: N) -> Result<String> {
        let node_id = node.id();
        
        // Check for duplicate
        if self.node_id_to_index.contains_key(&node_id) {
            return Err(GraphError::DuplicateNode(node_id));
        }
        
        // Add to petgraph
        let index = self.graph.add_node(node);
        
        // Update mappings
        self.node_id_to_index.insert(node_id.clone(), index);
        self.index_to_node_id.insert(index, node_id.clone());
        
        // Update metadata
        self.metadata.updated_at = chrono::Utc::now();
        
        // Emit event
        self.emit_event(GraphEvent::NodeAdded {
            graph_id: self.id.clone(),
            node_id: node_id.clone(),
        });
        
        Ok(node_id)
    }
    
    /// Remove a node
    pub fn remove_node(&mut self, node_id: &str) -> Result<N> {
        let index = *self
            .node_id_to_index
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        
        // Get the node data before removal
        let node = self
            .graph
            .node_weight(index)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?
            .clone();
        
        // Find all edges connected to this node (both incoming and outgoing)
        let outgoing_edges: Vec<_> = self.graph.edges(index).map(|e| e.id()).collect();
        let incoming_edges: Vec<_> = self
            .graph
            .edges_directed(index, petgraph::Direction::Incoming)
            .map(|e| e.id())
            .collect();
        
        // Collect all edge IDs to remove
        let mut edge_ids_to_remove = Vec::new();
        for (edge_id, &edge_index) in &self.edge_id_to_index {
            if outgoing_edges.contains(&edge_index) || incoming_edges.contains(&edge_index) {
                edge_ids_to_remove.push(edge_id.clone());
            }
        }
        
        // Remove the edges from our mapping and emit events
        for edge_id in edge_ids_to_remove {
            self.edge_id_to_index.remove(&edge_id);
            
            // Emit edge removed event
            self.emit_event(GraphEvent::EdgeRemoved {
                graph_id: self.id.clone(),
                edge_id,
            });
        }
        
        // Remove from petgraph (this also removes connected edges)
        self.graph.remove_node(index);
        
        // Update mappings
        self.node_id_to_index.remove(node_id);
        self.index_to_node_id.remove(&index);
        
        // Update metadata
        self.metadata.updated_at = chrono::Utc::now();
        
        // Emit event
        self.emit_event(GraphEvent::NodeRemoved {
            graph_id: self.id.clone(),
            node_id: node_id.to_string(),
        });
        
        Ok(node)
    }
    
    /// Add an edge
    pub fn add_edge(&mut self, edge: E) -> Result<String> {
        let edge_id = edge.id();
        let source_id = edge.source();
        let target_id = edge.target();
        
        // Check for duplicate edge ID
        if self.edge_id_to_index.contains_key(&edge_id) {
            return Err(GraphError::DuplicateEdge {
                from: source_id.clone(),
                to: target_id.clone(),
            });
        }
        
        // Get node indices
        let source_index = self
            .node_id_to_index
            .get(&source_id)
            .ok_or_else(|| GraphError::NodeNotFound(source_id.clone()))?;
        
        let target_index = self
            .node_id_to_index
            .get(&target_id)
            .ok_or_else(|| GraphError::NodeNotFound(target_id.clone()))?;
        
        // Add to petgraph
        let edge_index = self.graph.add_edge(*source_index, *target_index, edge);
        
        // Update mapping
        self.edge_id_to_index.insert(edge_id.clone(), edge_index);
        
        // Update metadata
        self.metadata.updated_at = chrono::Utc::now();
        
        // Emit event
        self.emit_event(GraphEvent::EdgeAdded {
            graph_id: self.id.clone(),
            edge_id: edge_id.clone(),
            source: source_id,
            target: target_id,
        });
        
        Ok(edge_id)
    }
    
    /// Remove an edge
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<E> {
        let edge_index = self
            .edge_id_to_index
            .remove(edge_id)
            .ok_or_else(|| GraphError::EdgeNotFound(edge_id.to_string()))?;
        
        // Get edge data before removal
        let edge = self
            .graph
            .edge_weight(edge_index)
            .ok_or_else(|| GraphError::EdgeNotFound(edge_id.to_string()))?
            .clone();
        
        // Remove from petgraph
        self.graph.remove_edge(edge_index);
        
        // Update metadata
        self.metadata.updated_at = chrono::Utc::now();
        
        // Emit event
        self.emit_event(GraphEvent::EdgeRemoved {
            graph_id: self.id.clone(),
            edge_id: edge_id.to_string(),
        });
        
        Ok(edge)
    }
    
    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&N> {
        self.node_id_to_index
            .get(node_id)
            .and_then(|&index| self.graph.node_weight(index))
    }
    
    /// Get mutable node by ID
    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut N> {
        self.node_id_to_index
            .get(node_id)
            .copied()
            .and_then(move |index| self.graph.node_weight_mut(index))
    }
    
    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: &str) -> Option<&E> {
        self.edge_id_to_index
            .get(edge_id)
            .and_then(|&index| self.graph.edge_weight(index))
    }
    
    /// Check if node exists
    pub fn contains_node(&self, node_id: &str) -> bool {
        self.node_id_to_index.contains_key(node_id)
    }
    
    /// Check if edge exists
    pub fn contains_edge(&self, edge_id: &str) -> bool {
        self.edge_id_to_index.contains_key(edge_id)
    }
    
    /// Get node count
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }
    
    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    
    /// Get all node IDs
    pub fn node_ids(&self) -> Vec<String> {
        self.node_id_to_index.keys().cloned().collect()
    }
    
    /// Get all edge IDs
    pub fn edge_ids(&self) -> Vec<String> {
        self.edge_id_to_index.keys().cloned().collect()
    }
    
    /// Get edges between two nodes
    pub fn edges_between(&self, source: &str, target: &str) -> Vec<&E> {
        let source_index = match self.node_id_to_index.get(source) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        
        let target_index = match self.node_id_to_index.get(target) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };
        
        self.graph
            .edges_connecting(source_index, target_index)
            .map(|edge| edge.weight())
            .collect()
    }
    
    /// Get neighbors (outgoing edges)
    pub fn neighbors(&self, node_id: &str) -> Result<Vec<String>> {
        let index = self
            .node_id_to_index
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        
        let neighbor_ids: Vec<String> = self
            .graph
            .neighbors(*index)
            .filter_map(|idx| self.index_to_node_id.get(&idx))
            .cloned()
            .collect();
        
        Ok(neighbor_ids)
    }
    
    /// Get incoming neighbors
    pub fn predecessors(&self, node_id: &str) -> Result<Vec<String>> {
        let index = self
            .node_id_to_index
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        
        let predecessor_ids: Vec<String> = self
            .graph
            .neighbors_directed(*index, petgraph::Direction::Incoming)
            .filter_map(|idx| self.index_to_node_id.get(&idx))
            .cloned()
            .collect();
        
        Ok(predecessor_ids)
    }
    
    /// Get out-degree (number of outgoing edges)
    pub fn degree(&self, node_id: &str) -> Result<usize> {
        let index = self
            .node_id_to_index
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        
        Ok(self.graph.edges(*index).count())
    }
    
    /// Get total degree (number of all edges connected to node)
    pub fn total_degree(&self, node_id: &str) -> Result<usize> {
        let index = self
            .node_id_to_index
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        
        let outgoing = self.graph.edges(*index).count();
        let incoming = self.graph.edges_directed(*index, petgraph::Direction::Incoming).count();
        
        Ok(outgoing + incoming)
    }
    
    /// Get all edges from a node
    pub fn edges_from(&self, node_id: &str) -> Result<Vec<String>> {
        let index = self.node_id_to_index.get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
            
        let edge_ids: Vec<String> = self.graph.edges(*index)
            .filter_map(|edge_ref| {
                self.edge_id_to_index.iter()
                    .find(|(_, &idx)| idx == edge_ref.id())
                    .map(|(id, _)| id.clone())
            })
            .collect();
            
        Ok(edge_ids)
    }
    
    /// Clear the graph
    pub fn clear(&mut self) {
        self.graph.clear();
        self.node_id_to_index.clear();
        self.index_to_node_id.clear();
        self.edge_id_to_index.clear();
        self.metadata.updated_at = chrono::Utc::now();
        
        // Emit event
        self.emit_event(GraphEvent::GraphCleared {
            graph_id: self.id.clone(),
        });
    }
    
    /// Convert to JSON
    pub fn to_json(&self) -> Result<serde_json::Value> {
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
}