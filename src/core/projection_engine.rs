//! Projection engine for building graph projections from event streams
//!
//! The projection engine is responsible for:
//! - Replaying events to build current state
//! - Applying new events to update projections
//! - Managing projection lifecycle

use crate::core::cim_graph::{GraphEvent, EventData, GraphProjection};
use crate::core::{Node, Edge, GraphType, GraphMetadata};
use uuid::Uuid;
use std::collections::HashMap;

/// Generic graph projection implementation
#[derive(Debug, Clone)]
pub struct GenericGraphProjection<N: Node, E: Edge> {
    aggregate_id: Uuid,
    graph_type: GraphType,
    version: u64,
    metadata: GraphMetadata,
    nodes: HashMap<String, N>,
    edges: HashMap<String, E>,
    // Track edge connections for neighbor queries
    adjacency: HashMap<String, Vec<String>>,
}

impl<N: Node, E: Edge> GenericGraphProjection<N, E> {
    /// Create an empty projection for an aggregate
    pub fn new(aggregate_id: Uuid, graph_type: GraphType) -> Self {
        Self {
            aggregate_id,
            graph_type,
            version: 0,
            metadata: GraphMetadata::default(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }
}

impl<N: Node, E: Edge> GenericGraphProjection<N, E> {
    /// Get all nodes in the projection
    pub fn nodes(&self) -> impl Iterator<Item = &N> {
        self.nodes.values()
    }
    
    /// Get all edges in the projection
    pub fn edges(&self) -> impl Iterator<Item = &E> {
        self.edges.values()
    }
    
    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&N> {
        self.nodes.get(node_id)
    }
    
    /// Get an edge by ID
    pub fn get_edge(&self, edge_id: &str) -> Option<&E> {
        self.edges.get(edge_id)
    }
}

impl<N: Node, E: Edge> GraphProjection for GenericGraphProjection<N, E> {
    type Node = N;
    type Edge = E;
    
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    
    fn version(&self) -> u64 {
        self.version
    }
    
    fn get_node(&self, node_id: &str) -> Option<&Self::Node> {
        self.nodes.get(node_id)
    }
    
    fn get_edge(&self, edge_id: &str) -> Option<&Self::Edge> {
        self.edges.get(edge_id)
    }
    
    fn nodes(&self) -> Vec<&Self::Node> {
        self.nodes.values().collect()
    }
    
    fn edges(&self) -> Vec<&Self::Edge> {
        self.edges.values().collect()
    }
    
    fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    fn edge_count(&self) -> usize {
        self.edges.len()
    }
    
    fn edges_between(&self, from: &str, to: &str) -> Vec<&Self::Edge> {
        self.edges.values()
            .filter(|e| e.source() == from && e.target() == to)
            .collect()
    }
    
    fn neighbors(&self, node_id: &str) -> Vec<&str> {
        self.adjacency
            .get(node_id)
            .map(|adj| adj.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

/// Engine for building and updating projections from events
#[derive(Debug)]
pub struct ProjectionEngine<P: GraphProjection> {
    projection_type: std::marker::PhantomData<P>,
}

impl<P: GraphProjection> ProjectionEngine<P> {
    /// Create a new projection engine
    pub fn new() -> Self {
        Self {
            projection_type: std::marker::PhantomData,
        }
    }
}

impl<N: Node, E: Edge> ProjectionEngine<GenericGraphProjection<N, E>>
{
    /// Build a projection from a stream of events
    /// Note: This creates an empty projection - specific graph types should handle their own node/edge creation
    pub fn project(&self, events: Vec<GraphEvent>) -> GenericGraphProjection<N, E> {
        if events.is_empty() {
            panic!("Cannot build projection from empty event stream");
        }
        
        // Get aggregate ID from first event
        let aggregate_id = events[0].aggregate_id;
        let graph_type = GraphType::Generic;
        
        // Initialize empty projection
        let mut projection = GenericGraphProjection::new(aggregate_id, graph_type);
        
        // Apply all events in sequence
        for event in events {
            self.apply(&mut projection, &event);
        }
        
        projection
    }
    
    /// Apply a single event to update a projection
    pub fn apply(&self, projection: &mut GenericGraphProjection<N, E>, event: &GraphEvent) {
        // Validate event is for this aggregate
        if event.aggregate_id != projection.aggregate_id {
            return; // Ignore events for other aggregates
        }
        
        // Update version
        projection.version = event.sequence;
        projection.metadata.updated_at = event.timestamp;
        
        // Apply event data
        match &event.data {
            EventData::GraphInitialized { graph_type, metadata } => {
                projection.graph_type = match graph_type.as_str() {
                    "generic" => GraphType::Generic,
                    "ipld" => GraphType::IpldGraph,
                    "context" => GraphType::ContextGraph,
                    "workflow" => GraphType::WorkflowGraph,
                    "concept" => GraphType::ConceptGraph,
                    "composed" => GraphType::ComposedGraph,
                    _ => GraphType::Generic,
                };
                
                // Update metadata from event
                for (key, value) in metadata {
                    projection.metadata.properties.insert(key.clone(), value.clone());
                }
            }
            
            EventData::NodeAdded { node_id, node_type, data } => {
                // Generic projection doesn't create typed nodes
                // Specific graph types (WorkflowNode, ConceptNode, etc.) handle their own node creation
                // Here we just track the node exists for adjacency purposes
                projection.adjacency.insert(node_id.clone(), Vec::new());
                
                // Store metadata about the node for later use
                projection.metadata.properties.insert(
                    format!("node_{}", node_id),
                    serde_json::json!({
                        "type": node_type,
                        "data": data
                    })
                );
            }
            
            EventData::EdgeAdded { edge_id, source_id, target_id, edge_type, data } => {
                // Update adjacency list
                if let Some(adj) = projection.adjacency.get_mut(source_id) {
                    adj.push(target_id.clone());
                }
                
                // Store edge metadata for later use
                projection.metadata.properties.insert(
                    format!("edge_{}", edge_id),
                    serde_json::json!({
                        "source": source_id,
                        "target": target_id,
                        "type": edge_type,
                        "data": data
                    })
                );
            }
            
            EventData::NodeRemoved { node_id } => {
                projection.nodes.remove(node_id);
                
                // Remove all edges connected to this node
                let edges_to_remove: Vec<String> = projection.edges
                    .iter()
                    .filter(|(_, edge)| edge.source() == *node_id || edge.target() == *node_id)
                    .map(|(id, _)| id.clone())
                    .collect();
                
                for edge_id in edges_to_remove {
                    projection.edges.remove(&edge_id);
                }
                
                // Update adjacency
                projection.adjacency.remove(node_id);
                for adj_list in projection.adjacency.values_mut() {
                    adj_list.retain(|id| id != node_id);
                }
            }
            
            EventData::EdgeRemoved { edge_id } => {
                if let Some(edge) = projection.edges.remove(edge_id) {
                    // Update adjacency
                    if let Some(adj) = projection.adjacency.get_mut(&edge.source()) {
                        adj.retain(|id| id != &edge.target());
                    }
                }
            }
            
            EventData::NodeUpdated { node_id, data } => {
                // Update node metadata
                projection.metadata.properties.insert(
                    format!("node_{}_updated", node_id),
                    data.clone()
                );
                
                // Update timestamp to track when node was last modified
                projection.metadata.properties.insert(
                    format!("node_{}_updated_at", node_id),
                    serde_json::json!(event.timestamp.to_rfc3339())
                );
            }
            
            EventData::EdgeUpdated { edge_id, data } => {
                // Update edge metadata
                projection.metadata.properties.insert(
                    format!("edge_{}_updated", edge_id),
                    data.clone()
                );
                
                // Update timestamp to track when edge was last modified
                projection.metadata.properties.insert(
                    format!("edge_{}_updated_at", edge_id),
                    serde_json::json!(event.timestamp.to_rfc3339())
                );
            }
        }
    }
}

/// Projection cache for performance
#[derive(Debug)]
pub struct ProjectionCache<P: GraphProjection> {
    projections: HashMap<Uuid, P>,
    max_size: usize,
}

impl<P: GraphProjection> ProjectionCache<P> {
    /// Create a new cache with maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            projections: HashMap::new(),
            max_size,
        }
    }
    
    /// Get a projection from cache
    pub fn get(&self, aggregate_id: &Uuid) -> Option<&P> {
        self.projections.get(aggregate_id)
    }
    
    /// Put a projection in cache
    pub fn put(&mut self, projection: P) {
        let aggregate_id = projection.aggregate_id();
        
        // Simple LRU: if at capacity, remove a random entry
        if self.projections.len() >= self.max_size {
            if let Some(key) = self.projections.keys().next().cloned() {
                self.projections.remove(&key);
            }
        }
        
        self.projections.insert(aggregate_id, projection);
    }
    
    /// Invalidate a projection
    pub fn invalidate(&mut self, aggregate_id: &Uuid) {
        self.projections.remove(aggregate_id);
    }
    
    /// Clear all cached projections
    pub fn clear(&mut self) {
        self.projections.clear();
    }
}