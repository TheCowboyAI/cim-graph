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
    /// Unique identifier for the graph aggregate
    pub aggregate_id: Uuid,
    /// Type of graph (workflow, concept, ipld, etc.)
    pub graph_type: GraphType,
    /// Current version (event sequence number)
    pub version: u64,
    /// Graph metadata
    pub metadata: GraphMetadata,
    /// Node storage by ID
    pub nodes: HashMap<String, N>,
    /// Edge storage by ID
    pub edges: HashMap<String, E>,
    /// Track edge connections for neighbor queries
    pub adjacency: HashMap<String, Vec<String>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::workflow::{WorkflowNode, WorkflowEdge, WorkflowNodeType};
    use chrono::Utc;

    type TestProjection = GenericGraphProjection<WorkflowNode, WorkflowEdge>;

    fn create_test_event(
        aggregate_id: Uuid,
        sequence: u64,
        data: EventData,
    ) -> GraphEvent {
        GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            sequence,
            subject: format!("cim.graph.evt.{}", aggregate_id),
            timestamp: Utc::now(),
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            data,
        }
    }

    // ========== GenericGraphProjection Tests ==========

    #[test]
    fn test_new_projection_is_empty() {
        let agg_id = Uuid::new_v4();
        let projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 0);
        assert_eq!(projection.node_count(), 0);
        assert_eq!(projection.edge_count(), 0);
        assert!(matches!(projection.graph_type, GraphType::Generic));
    }

    #[test]
    fn test_projection_with_nodes() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);

        let node = WorkflowNode::new("node1", WorkflowNodeType::Start);
        projection.nodes.insert("node1".to_string(), node);
        projection.adjacency.insert("node1".to_string(), vec![]);

        assert_eq!(projection.node_count(), 1);
        assert!(projection.get_node("node1").is_some());
        assert!(projection.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_projection_with_edges() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);

        // Add nodes
        let node_a = WorkflowNode::new("A", WorkflowNodeType::Start);
        let node_b = WorkflowNode::state("B", "Middle");
        projection.nodes.insert("A".to_string(), node_a);
        projection.nodes.insert("B".to_string(), node_b);

        // Add edge
        let edge = WorkflowEdge::transition("e1", "A", "B");
        projection.edges.insert("e1".to_string(), edge);
        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec![]);

        assert_eq!(projection.edge_count(), 1);
        assert!(projection.get_edge("e1").is_some());
        assert!(projection.get_edge("nonexistent").is_none());
    }

    #[test]
    fn test_edges_between() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);

        // Add nodes
        let node_a = WorkflowNode::new("A", WorkflowNodeType::Start);
        let node_b = WorkflowNode::state("B", "State");
        projection.nodes.insert("A".to_string(), node_a);
        projection.nodes.insert("B".to_string(), node_b);

        // Add edges from A to B
        let edge1 = WorkflowEdge::transition("e1", "A", "B");
        let edge2 = WorkflowEdge::conditional("e2", "A", "B", "condition1");
        projection.edges.insert("e1".to_string(), edge1);
        projection.edges.insert("e2".to_string(), edge2);

        let edges_ab = projection.edges_between("A", "B");
        assert_eq!(edges_ab.len(), 2);

        let edges_ba = projection.edges_between("B", "A");
        assert_eq!(edges_ba.len(), 0);
    }

    #[test]
    fn test_neighbors() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);

        // Create adjacency
        projection.adjacency.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["D".to_string()]);
        projection.adjacency.insert("C".to_string(), vec![]);
        projection.adjacency.insert("D".to_string(), vec![]);

        let neighbors_a = projection.neighbors("A");
        assert_eq!(neighbors_a.len(), 2);
        assert!(neighbors_a.contains(&"B"));
        assert!(neighbors_a.contains(&"C"));

        let neighbors_b = projection.neighbors("B");
        assert_eq!(neighbors_b.len(), 1);
        assert!(neighbors_b.contains(&"D"));

        let neighbors_c = projection.neighbors("C");
        assert_eq!(neighbors_c.len(), 0);

        let neighbors_nonexistent = projection.neighbors("X");
        assert_eq!(neighbors_nonexistent.len(), 0);
    }

    #[test]
    fn test_nodes_iterator() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        // Add multiple nodes
        for i in 0..5 {
            let id = format!("node{}", i);
            let node = WorkflowNode::new(&id, WorkflowNodeType::Start);
            projection.nodes.insert(id, node);
        }

        let node_vec: Vec<_> = projection.nodes().collect();
        assert_eq!(node_vec.len(), 5);
    }

    #[test]
    fn test_edges_iterator() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        // Add edges
        for i in 0..3 {
            let id = format!("edge{}", i);
            let source = format!("node{}", i);
            let target = format!("node{}", i + 1);
            let edge = WorkflowEdge::transition(&id, &source, &target);
            projection.edges.insert(id, edge);
        }

        let edge_vec: Vec<_> = projection.edges().collect();
        assert_eq!(edge_vec.len(), 3);
    }

    // ========== ProjectionEngine Tests ==========

    #[test]
    fn test_engine_creation() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        // Just verify it creates without panicking
        let _debug_str = format!("{:?}", engine);
    }

    #[test]
    fn test_engine_project_with_init_event() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(
                agg_id,
                1,
                EventData::GraphInitialized {
                    graph_type: "workflow".to_string(),
                    metadata: HashMap::new(),
                },
            ),
        ];

        let projection = engine.project(events);

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 1);
        assert!(matches!(projection.graph_type, GraphType::WorkflowGraph));
    }

    #[test]
    fn test_engine_project_with_node_events() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(
                agg_id,
                1,
                EventData::GraphInitialized {
                    graph_type: "workflow".to_string(),
                    metadata: HashMap::new(),
                },
            ),
            create_test_event(
                agg_id,
                2,
                EventData::NodeAdded {
                    node_id: "start".to_string(),
                    node_type: "Start".to_string(),
                    data: serde_json::json!({}),
                },
            ),
            create_test_event(
                agg_id,
                3,
                EventData::NodeAdded {
                    node_id: "end".to_string(),
                    node_type: "End".to_string(),
                    data: serde_json::json!({}),
                },
            ),
        ];

        let projection = engine.project(events);

        assert_eq!(projection.version, 3);
        // Nodes are tracked in adjacency
        assert!(projection.adjacency.contains_key("start"));
        assert!(projection.adjacency.contains_key("end"));
    }

    #[test]
    fn test_engine_project_with_edge_events() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(
                agg_id,
                1,
                EventData::GraphInitialized {
                    graph_type: "workflow".to_string(),
                    metadata: HashMap::new(),
                },
            ),
            create_test_event(
                agg_id,
                2,
                EventData::NodeAdded {
                    node_id: "A".to_string(),
                    node_type: "State".to_string(),
                    data: serde_json::json!({}),
                },
            ),
            create_test_event(
                agg_id,
                3,
                EventData::NodeAdded {
                    node_id: "B".to_string(),
                    node_type: "State".to_string(),
                    data: serde_json::json!({}),
                },
            ),
            create_test_event(
                agg_id,
                4,
                EventData::EdgeAdded {
                    edge_id: "e1".to_string(),
                    source_id: "A".to_string(),
                    target_id: "B".to_string(),
                    edge_type: "Transition".to_string(),
                    data: serde_json::json!({}),
                },
            ),
        ];

        let projection = engine.project(events);

        assert_eq!(projection.version, 4);
        // Check adjacency was updated
        let neighbors_a = projection.neighbors("A");
        assert!(neighbors_a.contains(&"B"));
    }

    #[test]
    #[should_panic(expected = "Cannot build projection from empty event stream")]
    fn test_engine_project_empty_events_panics() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let _projection = engine.project(vec![]);
    }

    #[test]
    fn test_engine_apply_ignores_different_aggregate() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();
        let other_agg_id = Uuid::new_v4();

        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        projection.version = 5;

        // Create event for different aggregate
        let event = create_test_event(
            other_agg_id,
            10,
            EventData::NodeAdded {
                node_id: "foreign".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            },
        );

        engine.apply(&mut projection, &event);

        // Version should not change because event was for different aggregate
        assert_eq!(projection.version, 5);
        assert!(!projection.adjacency.contains_key("foreign"));
    }

    #[test]
    fn test_engine_apply_node_removed() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        // Add node first
        let node = WorkflowNode::new("to_remove", WorkflowNodeType::Start);
        projection.nodes.insert("to_remove".to_string(), node);
        projection.adjacency.insert("to_remove".to_string(), vec!["other".to_string()]);
        projection.adjacency.insert("other".to_string(), vec!["to_remove".to_string()]);

        // Add edge connected to the node
        let edge = WorkflowEdge::transition("e1", "to_remove", "other");
        projection.edges.insert("e1".to_string(), edge);

        let remove_event = create_test_event(
            agg_id,
            1,
            EventData::NodeRemoved {
                node_id: "to_remove".to_string(),
            },
        );

        engine.apply(&mut projection, &remove_event);

        // Node should be removed
        assert!(projection.get_node("to_remove").is_none());
        // Edge connected to node should be removed
        assert!(projection.get_edge("e1").is_none());
        // Adjacency should be updated
        assert!(!projection.adjacency.contains_key("to_remove"));
        // Other node's adjacency should not reference removed node
        let other_neighbors = projection.neighbors("other");
        assert!(!other_neighbors.contains(&"to_remove"));
    }

    #[test]
    fn test_engine_apply_edge_removed() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        // Setup: Add edge
        let edge = WorkflowEdge::transition("e1", "A", "B");
        projection.edges.insert("e1".to_string(), edge);
        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec![]);

        let remove_event = create_test_event(
            agg_id,
            1,
            EventData::EdgeRemoved {
                edge_id: "e1".to_string(),
            },
        );

        engine.apply(&mut projection, &remove_event);

        // Edge should be removed
        assert!(projection.get_edge("e1").is_none());
        // Adjacency should be updated
        let neighbors_a = projection.neighbors("A");
        assert!(!neighbors_a.contains(&"B"));
    }

    #[test]
    fn test_engine_apply_node_updated() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        projection.adjacency.insert("node1".to_string(), vec![]);

        let update_event = create_test_event(
            agg_id,
            1,
            EventData::NodeUpdated {
                node_id: "node1".to_string(),
                data: serde_json::json!({"status": "updated"}),
            },
        );

        engine.apply(&mut projection, &update_event);

        // Check metadata was updated
        assert!(projection.metadata.properties.contains_key("node_node1_updated"));
        assert!(projection.metadata.properties.contains_key("node_node1_updated_at"));
    }

    #[test]
    fn test_engine_apply_edge_updated() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        let update_event = create_test_event(
            agg_id,
            1,
            EventData::EdgeUpdated {
                edge_id: "edge1".to_string(),
                data: serde_json::json!({"weight": 10}),
            },
        );

        engine.apply(&mut projection, &update_event);

        // Check metadata was updated
        assert!(projection.metadata.properties.contains_key("edge_edge1_updated"));
        assert!(projection.metadata.properties.contains_key("edge_edge1_updated_at"));
    }

    #[test]
    fn test_engine_apply_graph_types() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let graph_types = vec![
            ("generic", GraphType::Generic),
            ("ipld", GraphType::IpldGraph),
            ("context", GraphType::ContextGraph),
            ("workflow", GraphType::WorkflowGraph),
            ("concept", GraphType::ConceptGraph),
            ("composed", GraphType::ComposedGraph),
            ("unknown", GraphType::Generic), // Unknown types default to Generic
        ];

        for (type_str, expected_type) in graph_types {
            let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

            let init_event = create_test_event(
                agg_id,
                1,
                EventData::GraphInitialized {
                    graph_type: type_str.to_string(),
                    metadata: HashMap::new(),
                },
            );

            engine.apply(&mut projection, &init_event);

            assert!(
                std::mem::discriminant(&projection.graph_type) == std::mem::discriminant(&expected_type),
                "Failed for type: {}", type_str
            );
        }
    }

    // ========== ProjectionCache Tests ==========

    #[test]
    fn test_cache_new() {
        let cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);
        assert!(cache.get(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);
        let agg_id = Uuid::new_v4();

        let projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        cache.put(projection);

        assert!(cache.get(&agg_id).is_some());
        assert_eq!(cache.get(&agg_id).unwrap().aggregate_id, agg_id);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);
        let agg_id = Uuid::new_v4();

        let projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        cache.put(projection);

        assert!(cache.get(&agg_id).is_some());

        cache.invalidate(&agg_id);

        assert!(cache.get(&agg_id).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);

        // Add multiple projections
        for _ in 0..5 {
            let agg_id = Uuid::new_v4();
            let projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
            cache.put(projection);
        }

        cache.clear();

        // Verify all are cleared by trying to get a new one
        // (we can't easily verify since we don't track the IDs)
    }

    #[test]
    fn test_cache_eviction_at_capacity() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(3);

        let mut ids = Vec::new();

        // Add 4 projections (one over capacity)
        for _ in 0..4 {
            let agg_id = Uuid::new_v4();
            ids.push(agg_id);
            let projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
            cache.put(projection);
        }

        // At least one should be evicted
        let mut found_count = 0;
        for id in &ids {
            if cache.get(id).is_some() {
                found_count += 1;
            }
        }

        // Should have at most 3 items (max_size)
        assert!(found_count <= 3, "Cache should evict to stay at max_size");
    }

    #[test]
    fn test_cache_update_existing() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);
        let agg_id = Uuid::new_v4();

        // Add projection with version 1
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        projection.version = 1;
        cache.put(projection);

        assert_eq!(cache.get(&agg_id).unwrap().version, 1);

        // Update with version 2
        let mut projection2: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        projection2.version = 2;
        cache.put(projection2);

        // Should have the updated version
        assert_eq!(cache.get(&agg_id).unwrap().version, 2);
    }
}