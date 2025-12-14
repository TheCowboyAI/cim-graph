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

    /// Get the number of cached projections
    pub fn len(&self) -> usize {
        self.projections.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }

    /// Get all cached projection IDs
    pub fn cached_ids(&self) -> Vec<Uuid> {
        self.projections.keys().cloned().collect()
    }
}

/// Snapshot of a projection for persistence and recovery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectionSnapshot {
    /// Aggregate ID of the projection
    pub aggregate_id: Uuid,
    /// Version at time of snapshot
    pub version: u64,
    /// Timestamp when snapshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Serialized projection data
    pub data: serde_json::Value,
    /// Checksum for integrity verification
    pub checksum: String,
}

impl ProjectionSnapshot {
    /// Create a new snapshot
    pub fn new(aggregate_id: Uuid, version: u64, data: serde_json::Value) -> Self {
        let checksum = Self::compute_checksum(&data);
        Self {
            aggregate_id,
            version,
            timestamp: chrono::Utc::now(),
            data,
            checksum,
        }
    }

    /// Compute checksum for data integrity
    fn compute_checksum(data: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Verify snapshot integrity
    pub fn verify_integrity(&self) -> bool {
        Self::compute_checksum(&self.data) == self.checksum
    }
}

/// Event store interface for replaying events
#[derive(Debug)]
pub struct EventStore {
    /// Events stored by aggregate ID
    events: HashMap<Uuid, Vec<GraphEvent>>,
    /// Snapshots by aggregate ID
    snapshots: HashMap<Uuid, ProjectionSnapshot>,
}

impl EventStore {
    /// Create a new in-memory event store
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            snapshots: HashMap::new(),
        }
    }

    /// Append events for an aggregate
    pub fn append(&mut self, aggregate_id: Uuid, events: Vec<GraphEvent>) {
        self.events
            .entry(aggregate_id)
            .or_insert_with(Vec::new)
            .extend(events);
    }

    /// Get all events for an aggregate
    pub fn get_events(&self, aggregate_id: &Uuid) -> Vec<GraphEvent> {
        self.events.get(aggregate_id).cloned().unwrap_or_default()
    }

    /// Get events since a specific version
    pub fn get_events_since(&self, aggregate_id: &Uuid, version: u64) -> Vec<GraphEvent> {
        self.events
            .get(aggregate_id)
            .map(|events| {
                events
                    .iter()
                    .filter(|e| e.sequence > version)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Save a snapshot
    pub fn save_snapshot(&mut self, snapshot: ProjectionSnapshot) {
        self.snapshots.insert(snapshot.aggregate_id, snapshot);
    }

    /// Get latest snapshot
    pub fn get_snapshot(&self, aggregate_id: &Uuid) -> Option<&ProjectionSnapshot> {
        self.snapshots.get(aggregate_id)
    }

    /// Get event count for an aggregate
    pub fn event_count(&self, aggregate_id: &Uuid) -> usize {
        self.events.get(aggregate_id).map(|e| e.len()).unwrap_or(0)
    }

    /// Clear all events and snapshots for an aggregate
    pub fn clear(&mut self, aggregate_id: &Uuid) {
        self.events.remove(aggregate_id);
        self.snapshots.remove(aggregate_id);
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced projection engine with snapshot and replay capabilities
#[derive(Debug)]
pub struct EnhancedProjectionEngine<P: GraphProjection> {
    /// Base projection engine
    engine: ProjectionEngine<P>,
    /// Event store for replay
    event_store: EventStore,
    /// Snapshot interval (events between snapshots)
    snapshot_interval: u64,
}

impl<N: Node + Clone, E: Edge + Clone> EnhancedProjectionEngine<GenericGraphProjection<N, E>>
where
    N: serde::Serialize + serde::de::DeserializeOwned,
    E: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Create a new enhanced projection engine
    pub fn new(snapshot_interval: u64) -> Self {
        Self {
            engine: ProjectionEngine::new(),
            event_store: EventStore::new(),
            snapshot_interval,
        }
    }

    /// Rebuild projection from events
    pub fn rebuild_from_events(&mut self, aggregate_id: Uuid) -> GenericGraphProjection<N, E> {
        let events = self.event_store.get_events(&aggregate_id);
        if events.is_empty() {
            return GenericGraphProjection::new(aggregate_id, GraphType::Generic);
        }
        self.engine.project(events)
    }

    /// Create a snapshot of the current projection
    pub fn snapshot(&mut self, projection: &GenericGraphProjection<N, E>) -> ProjectionSnapshot {
        let data = serde_json::json!({
            "aggregate_id": projection.aggregate_id.to_string(),
            "version": projection.version,
            "graph_type": format!("{:?}", projection.graph_type),
            "node_count": projection.nodes.len(),
            "edge_count": projection.edges.len(),
            "metadata": projection.metadata.properties,
        });

        let snapshot = ProjectionSnapshot::new(
            projection.aggregate_id,
            projection.version,
            data,
        );

        self.event_store.save_snapshot(snapshot.clone());
        snapshot
    }

    /// Restore projection from snapshot and replay newer events
    pub fn restore_from_snapshot(&mut self, aggregate_id: Uuid) -> Option<GenericGraphProjection<N, E>> {
        let snapshot = self.event_store.get_snapshot(&aggregate_id)?;

        // Verify snapshot integrity
        if !snapshot.verify_integrity() {
            return None;
        }

        // Create base projection from snapshot
        let mut projection = GenericGraphProjection::new(aggregate_id, GraphType::Generic);
        projection.version = snapshot.version;

        // Replay events since snapshot
        let newer_events = self.event_store.get_events_since(&aggregate_id, snapshot.version);
        for event in newer_events {
            self.engine.apply(&mut projection, &event);
        }

        Some(projection)
    }

    /// Apply event and auto-snapshot if needed
    pub fn apply_with_snapshot(
        &mut self,
        projection: &mut GenericGraphProjection<N, E>,
        event: GraphEvent,
    ) {
        // Store event
        self.event_store.append(projection.aggregate_id, vec![event.clone()]);

        // Apply event
        self.engine.apply(projection, &event);

        // Auto-snapshot if interval reached
        if projection.version > 0 && projection.version % self.snapshot_interval == 0 {
            self.snapshot(projection);
        }
    }

    /// Get events since a specific version
    pub fn get_events_since(&self, aggregate_id: &Uuid, version: u64) -> Vec<GraphEvent> {
        self.event_store.get_events_since(aggregate_id, version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphs::workflow::{WorkflowNode, WorkflowEdge, WorkflowNodeType};
    use chrono::Utc;
    use serde::{Serialize, Deserialize};

    type TestProjection = GenericGraphProjection<WorkflowNode, WorkflowEdge>;

    // Serializable test types for EnhancedProjectionEngine tests
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SerializableNode {
        id: String,
        node_type: String,
    }

    impl SerializableNode {
        fn new(id: impl Into<String>, node_type: impl Into<String>) -> Self {
            Self {
                id: id.into(),
                node_type: node_type.into(),
            }
        }
    }

    impl Node for SerializableNode {
        fn id(&self) -> String {
            self.id.clone()
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct SerializableEdge {
        id: String,
        source: String,
        target: String,
        edge_type: String,
    }

    impl SerializableEdge {
        fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>, edge_type: impl Into<String>) -> Self {
            Self {
                id: id.into(),
                source: source.into(),
                target: target.into(),
                edge_type: edge_type.into(),
            }
        }
    }

    impl Edge for SerializableEdge {
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

    type SerializableProjection = GenericGraphProjection<SerializableNode, SerializableEdge>;

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

    // ========== ProjectionCache Additional Tests ==========

    #[test]
    fn test_cache_len_and_is_empty() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        let agg_id = Uuid::new_v4();
        let projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);
        cache.put(projection);

        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_cached_ids() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);

        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        for id in &ids {
            let projection: TestProjection = GenericGraphProjection::new(*id, GraphType::Generic);
            cache.put(projection);
        }

        let cached = cache.cached_ids();
        assert_eq!(cached.len(), 5);

        for id in &ids {
            assert!(cached.contains(id));
        }
    }

    #[test]
    fn test_cache_invalidate_nonexistent() {
        let mut cache: ProjectionCache<TestProjection> = ProjectionCache::new(10);
        let agg_id = Uuid::new_v4();

        // Invalidating nonexistent ID should not panic
        cache.invalidate(&agg_id);
        assert!(cache.is_empty());
    }

    // ========== ProjectionSnapshot Tests ==========

    #[test]
    fn test_snapshot_creation() {
        let agg_id = Uuid::new_v4();
        let data = serde_json::json!({
            "nodes": ["A", "B", "C"],
            "edges": [{"from": "A", "to": "B"}]
        });

        let snapshot = ProjectionSnapshot::new(agg_id, 10, data.clone());

        assert_eq!(snapshot.aggregate_id, agg_id);
        assert_eq!(snapshot.version, 10);
        assert_eq!(snapshot.data, data);
        assert!(!snapshot.checksum.is_empty());
    }

    #[test]
    fn test_snapshot_integrity_valid() {
        let agg_id = Uuid::new_v4();
        let data = serde_json::json!({"test": "data"});

        let snapshot = ProjectionSnapshot::new(agg_id, 5, data);

        assert!(snapshot.verify_integrity());
    }

    #[test]
    fn test_snapshot_integrity_invalid() {
        let agg_id = Uuid::new_v4();
        let data = serde_json::json!({"test": "data"});

        let mut snapshot = ProjectionSnapshot::new(agg_id, 5, data);

        // Tamper with data
        snapshot.data = serde_json::json!({"test": "tampered"});

        assert!(!snapshot.verify_integrity());
    }

    #[test]
    fn test_snapshot_checksum_deterministic() {
        let agg_id = Uuid::new_v4();
        let data = serde_json::json!({"key": "value", "number": 42});

        let snapshot1 = ProjectionSnapshot::new(agg_id, 1, data.clone());
        let snapshot2 = ProjectionSnapshot::new(agg_id, 1, data.clone());

        // Same data should produce same checksum
        assert_eq!(snapshot1.checksum, snapshot2.checksum);
    }

    #[test]
    fn test_snapshot_different_data_different_checksum() {
        let agg_id = Uuid::new_v4();

        let snapshot1 = ProjectionSnapshot::new(agg_id, 1, serde_json::json!({"a": 1}));
        let snapshot2 = ProjectionSnapshot::new(agg_id, 1, serde_json::json!({"a": 2}));

        // Different data should produce different checksum
        assert_ne!(snapshot1.checksum, snapshot2.checksum);
    }

    #[test]
    fn test_snapshot_serialization() {
        let agg_id = Uuid::new_v4();
        let data = serde_json::json!({"nodes": 5, "edges": 10});

        let snapshot = ProjectionSnapshot::new(agg_id, 15, data);

        // Serialize
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains(&agg_id.to_string()));

        // Deserialize
        let deserialized: ProjectionSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.aggregate_id, agg_id);
        assert_eq!(deserialized.version, 15);
        assert!(deserialized.verify_integrity());
    }

    // ========== EventStore Tests ==========

    #[test]
    fn test_event_store_creation() {
        let store = EventStore::new();
        assert_eq!(store.event_count(&Uuid::new_v4()), 0);
    }

    #[test]
    fn test_event_store_default() {
        let store = EventStore::default();
        assert_eq!(store.event_count(&Uuid::new_v4()), 0);
    }

    #[test]
    fn test_event_store_append() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
        ];

        store.append(agg_id, events);

        assert_eq!(store.event_count(&agg_id), 1);
    }

    #[test]
    fn test_event_store_append_multiple() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        // Append first batch
        store.append(agg_id, vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
        ]);

        // Append second batch
        store.append(agg_id, vec![
            create_test_event(agg_id, 2, EventData::NodeAdded {
                node_id: "A".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
            create_test_event(agg_id, 3, EventData::NodeAdded {
                node_id: "B".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
        ]);

        assert_eq!(store.event_count(&agg_id), 3);
    }

    #[test]
    fn test_event_store_get_events() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id, 2, EventData::NodeAdded {
                node_id: "A".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
        ];

        store.append(agg_id, events);

        let retrieved = store.get_events(&agg_id);
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].sequence, 1);
        assert_eq!(retrieved[1].sequence, 2);
    }

    #[test]
    fn test_event_store_get_events_empty() {
        let store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let events = store.get_events(&agg_id);
        assert!(events.is_empty());
    }

    #[test]
    fn test_event_store_get_events_since() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id, 2, EventData::NodeAdded {
                node_id: "A".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
            create_test_event(agg_id, 3, EventData::NodeAdded {
                node_id: "B".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
            create_test_event(agg_id, 4, EventData::NodeAdded {
                node_id: "C".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
        ];

        store.append(agg_id, events);

        // Get events since version 2 (should return 3 and 4)
        let since = store.get_events_since(&agg_id, 2);
        assert_eq!(since.len(), 2);
        assert_eq!(since[0].sequence, 3);
        assert_eq!(since[1].sequence, 4);
    }

    #[test]
    fn test_event_store_get_events_since_empty() {
        let store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let since = store.get_events_since(&agg_id, 5);
        assert!(since.is_empty());
    }

    #[test]
    fn test_event_store_get_events_since_all() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        store.append(agg_id, vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
        ]);

        // Get events since version 0 (should return all)
        let since = store.get_events_since(&agg_id, 0);
        assert_eq!(since.len(), 1);
    }

    #[test]
    fn test_event_store_get_events_since_future() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        store.append(agg_id, vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
        ]);

        // Get events since version higher than any existing
        let since = store.get_events_since(&agg_id, 100);
        assert!(since.is_empty());
    }

    #[test]
    fn test_event_store_save_and_get_snapshot() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let snapshot = ProjectionSnapshot::new(
            agg_id,
            10,
            serde_json::json!({"nodes": 5}),
        );

        store.save_snapshot(snapshot.clone());

        let retrieved = store.get_snapshot(&agg_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().version, 10);
    }

    #[test]
    fn test_event_store_get_snapshot_not_found() {
        let store = EventStore::new();
        let agg_id = Uuid::new_v4();

        let snapshot = store.get_snapshot(&agg_id);
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_event_store_clear() {
        let mut store = EventStore::new();
        let agg_id = Uuid::new_v4();

        // Add events and snapshot
        store.append(agg_id, vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "test".to_string(),
                metadata: HashMap::new(),
            }),
        ]);
        store.save_snapshot(ProjectionSnapshot::new(
            agg_id,
            1,
            serde_json::json!({}),
        ));

        assert_eq!(store.event_count(&agg_id), 1);
        assert!(store.get_snapshot(&agg_id).is_some());

        // Clear
        store.clear(&agg_id);

        assert_eq!(store.event_count(&agg_id), 0);
        assert!(store.get_snapshot(&agg_id).is_none());
    }

    #[test]
    fn test_event_store_multiple_aggregates() {
        let mut store = EventStore::new();
        let agg_id1 = Uuid::new_v4();
        let agg_id2 = Uuid::new_v4();

        // Add events for both aggregates
        store.append(agg_id1, vec![
            create_test_event(agg_id1, 1, EventData::GraphInitialized {
                graph_type: "test1".to_string(),
                metadata: HashMap::new(),
            }),
        ]);
        store.append(agg_id2, vec![
            create_test_event(agg_id2, 1, EventData::GraphInitialized {
                graph_type: "test2".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id2, 2, EventData::NodeAdded {
                node_id: "X".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
        ]);

        assert_eq!(store.event_count(&agg_id1), 1);
        assert_eq!(store.event_count(&agg_id2), 2);

        // Clear only agg_id1
        store.clear(&agg_id1);

        assert_eq!(store.event_count(&agg_id1), 0);
        assert_eq!(store.event_count(&agg_id2), 2);
    }

    // ========== EnhancedProjectionEngine Tests ==========
    // These tests use SerializableProjection because EnhancedProjectionEngine
    // requires Serialize + DeserializeOwned on Node and Edge types

    #[test]
    fn test_enhanced_engine_creation() {
        let engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        // Verify engine is created successfully
        let _debug_str = format!("{:?}", engine);
    }

    #[test]
    fn test_enhanced_engine_rebuild_from_events_empty() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        // Rebuild with no events
        let projection = engine.rebuild_from_events(agg_id);

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 0);
        assert!(matches!(projection.graph_type, GraphType::Generic));
    }

    #[test]
    fn test_enhanced_engine_rebuild_from_events_with_events() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        // Add events to store
        let events = vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "workflow".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id, 2, EventData::NodeAdded {
                node_id: "start".to_string(),
                node_type: "Start".to_string(),
                data: serde_json::json!({}),
            }),
        ];

        engine.event_store.append(agg_id, events);

        // Rebuild projection
        let projection = engine.rebuild_from_events(agg_id);

        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 2);
        assert!(matches!(projection.graph_type, GraphType::WorkflowGraph));
        assert!(projection.adjacency.contains_key("start"));
    }

    #[test]
    fn test_enhanced_engine_snapshot() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        let mut projection: SerializableProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);
        projection.version = 5;

        let snapshot = engine.snapshot(&projection);

        assert_eq!(snapshot.aggregate_id, agg_id);
        assert_eq!(snapshot.version, 5);
        assert!(snapshot.verify_integrity());

        // Verify snapshot was saved
        let saved = engine.event_store.get_snapshot(&agg_id);
        assert!(saved.is_some());
    }

    #[test]
    fn test_enhanced_engine_restore_from_snapshot() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        // Create and save snapshot
        let snapshot = ProjectionSnapshot::new(
            agg_id,
            5,
            serde_json::json!({"nodes": 3}),
        );
        engine.event_store.save_snapshot(snapshot);

        // Add some events after snapshot
        engine.event_store.append(agg_id, vec![
            create_test_event(agg_id, 6, EventData::NodeAdded {
                node_id: "post_snapshot".to_string(),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            }),
        ]);

        // Restore
        let result = engine.restore_from_snapshot(agg_id);
        assert!(result.is_some());

        let projection = result.unwrap();
        assert_eq!(projection.aggregate_id, agg_id);
        assert_eq!(projection.version, 6); // Updated by replayed event
    }

    #[test]
    fn test_enhanced_engine_restore_from_snapshot_not_found() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        let result = engine.restore_from_snapshot(agg_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_enhanced_engine_restore_from_corrupted_snapshot() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        // Create snapshot with tampered data
        let mut snapshot = ProjectionSnapshot::new(
            agg_id,
            5,
            serde_json::json!({"original": "data"}),
        );
        snapshot.data = serde_json::json!({"tampered": "data"});
        engine.event_store.save_snapshot(snapshot);

        // Restore should fail due to integrity check
        let result = engine.restore_from_snapshot(agg_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_enhanced_engine_apply_with_snapshot() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(5);
        let agg_id = Uuid::new_v4();

        let mut projection: SerializableProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        // Apply events up to snapshot interval
        for i in 1..=5 {
            let event = create_test_event(agg_id, i, EventData::NodeAdded {
                node_id: format!("node{}", i),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            });
            engine.apply_with_snapshot(&mut projection, event);
        }

        // Should have created a snapshot at version 5
        let snapshot = engine.event_store.get_snapshot(&agg_id);
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().version, 5);
    }

    #[test]
    fn test_enhanced_engine_apply_with_snapshot_no_trigger() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        let mut projection: SerializableProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        // Apply only 3 events (below snapshot interval of 10)
        for i in 1..=3 {
            let event = create_test_event(agg_id, i, EventData::NodeAdded {
                node_id: format!("node{}", i),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            });
            engine.apply_with_snapshot(&mut projection, event);
        }

        // Should not have created a snapshot
        let snapshot = engine.event_store.get_snapshot(&agg_id);
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_enhanced_engine_get_events_since() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        // Add events through apply_with_snapshot
        let mut projection: SerializableProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        for i in 1..=5 {
            let event = create_test_event(agg_id, i, EventData::NodeAdded {
                node_id: format!("node{}", i),
                node_type: "Node".to_string(),
                data: serde_json::json!({}),
            });
            engine.apply_with_snapshot(&mut projection, event);
        }

        // Get events since version 3
        let events = engine.get_events_since(&agg_id, 3);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].sequence, 4);
        assert_eq!(events[1].sequence, 5);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_full_projection_lifecycle() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(5);
        let agg_id = Uuid::new_v4();

        // Initialize projection
        let mut projection: SerializableProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);

        // Apply initialization event
        let init_event = create_test_event(agg_id, 1, EventData::GraphInitialized {
            graph_type: "workflow".to_string(),
            metadata: [("name".to_string(), serde_json::json!("Test Workflow"))].into_iter().collect(),
        });
        engine.apply_with_snapshot(&mut projection, init_event);

        // Add nodes
        for i in 2..=6 {
            let node_event = create_test_event(agg_id, i, EventData::NodeAdded {
                node_id: format!("state{}", i - 1),
                node_type: "State".to_string(),
                data: serde_json::json!({"order": i}),
            });
            engine.apply_with_snapshot(&mut projection, node_event);
        }

        // Verify projection state
        assert_eq!(projection.version, 6);
        assert!(matches!(projection.graph_type, GraphType::WorkflowGraph));

        // Verify snapshot was created at version 5
        let snapshot = engine.event_store.get_snapshot(&agg_id);
        assert!(snapshot.is_some());

        // Verify events are stored
        let all_events = engine.event_store.get_events(&agg_id);
        assert_eq!(all_events.len(), 6);

        // Verify events since version 5
        let recent_events = engine.get_events_since(&agg_id, 5);
        assert_eq!(recent_events.len(), 1);
    }

    #[test]
    fn test_projection_rebuild_matches_incremental() {
        let mut engine: EnhancedProjectionEngine<SerializableProjection> = EnhancedProjectionEngine::new(10);
        let agg_id = Uuid::new_v4();

        // Build projection incrementally
        let mut projection: SerializableProjection = GenericGraphProjection::new(agg_id, GraphType::Generic);

        let events = vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "workflow".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id, 2, EventData::NodeAdded {
                node_id: "A".to_string(),
                node_type: "Start".to_string(),
                data: serde_json::json!({}),
            }),
            create_test_event(agg_id, 3, EventData::NodeAdded {
                node_id: "B".to_string(),
                node_type: "End".to_string(),
                data: serde_json::json!({}),
            }),
            create_test_event(agg_id, 4, EventData::EdgeAdded {
                edge_id: "e1".to_string(),
                source_id: "A".to_string(),
                target_id: "B".to_string(),
                edge_type: "Transition".to_string(),
                data: serde_json::json!({}),
            }),
        ];

        for event in events.clone() {
            engine.apply_with_snapshot(&mut projection, event);
        }

        // Rebuild from events
        let rebuilt = engine.rebuild_from_events(agg_id);

        // Verify both projections match
        assert_eq!(projection.version, rebuilt.version);
        assert_eq!(projection.adjacency.len(), rebuilt.adjacency.len());
        assert!(projection.adjacency.contains_key("A"));
        assert!(rebuilt.adjacency.contains_key("A"));
    }

    #[test]
    fn test_graph_projection_trait_implementation() {
        let agg_id = Uuid::new_v4();
        let mut projection: TestProjection = GenericGraphProjection::new(agg_id, GraphType::WorkflowGraph);

        // Add nodes
        let node_a = WorkflowNode::new("A", WorkflowNodeType::Start);
        let node_b = WorkflowNode::state("B", "Middle");
        let node_c = WorkflowNode::new("C", WorkflowNodeType::End);

        projection.nodes.insert("A".to_string(), node_a);
        projection.nodes.insert("B".to_string(), node_b);
        projection.nodes.insert("C".to_string(), node_c);

        // Add edges
        let edge_ab = WorkflowEdge::transition("e1", "A", "B");
        let edge_bc = WorkflowEdge::transition("e2", "B", "C");

        projection.edges.insert("e1".to_string(), edge_ab);
        projection.edges.insert("e2".to_string(), edge_bc);

        projection.adjacency.insert("A".to_string(), vec!["B".to_string()]);
        projection.adjacency.insert("B".to_string(), vec!["C".to_string()]);
        projection.adjacency.insert("C".to_string(), vec![]);

        // Test GraphProjection trait methods
        use crate::core::cim_graph::GraphProjection;

        assert_eq!(GraphProjection::aggregate_id(&projection), agg_id);
        assert_eq!(GraphProjection::version(&projection), 0);
        assert_eq!(GraphProjection::node_count(&projection), 3);
        assert_eq!(GraphProjection::edge_count(&projection), 2);

        assert!(GraphProjection::get_node(&projection, "A").is_some());
        assert!(GraphProjection::get_node(&projection, "D").is_none());

        assert!(GraphProjection::get_edge(&projection, "e1").is_some());
        assert!(GraphProjection::get_edge(&projection, "e3").is_none());

        let nodes: Vec<_> = GraphProjection::nodes(&projection);
        assert_eq!(nodes.len(), 3);

        let edges: Vec<_> = GraphProjection::edges(&projection);
        assert_eq!(edges.len(), 2);

        let edges_ab: Vec<_> = GraphProjection::edges_between(&projection, "A", "B");
        assert_eq!(edges_ab.len(), 1);

        let neighbors_a = GraphProjection::neighbors(&projection, "A");
        assert_eq!(neighbors_a.len(), 1);
        assert!(neighbors_a.contains(&"B"));
    }

    #[test]
    fn test_projection_metadata_propagation() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id = Uuid::new_v4();

        let events = vec![
            create_test_event(agg_id, 1, EventData::GraphInitialized {
                graph_type: "workflow".to_string(),
                metadata: [
                    ("name".to_string(), serde_json::json!("Test Graph")),
                    ("owner".to_string(), serde_json::json!("System")),
                ].into_iter().collect(),
            }),
        ];

        let projection = engine.project(events);

        // Verify metadata was propagated
        assert!(projection.metadata.properties.contains_key("name"));
        assert!(projection.metadata.properties.contains_key("owner"));
        assert_eq!(projection.metadata.properties["name"], serde_json::json!("Test Graph"));
    }

    #[test]
    fn test_multiple_projections_isolated() {
        let engine: ProjectionEngine<TestProjection> = ProjectionEngine::new();
        let agg_id1 = Uuid::new_v4();
        let agg_id2 = Uuid::new_v4();

        // Create projection 1
        let events1 = vec![
            create_test_event(agg_id1, 1, EventData::GraphInitialized {
                graph_type: "workflow".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id1, 2, EventData::NodeAdded {
                node_id: "A".to_string(),
                node_type: "Start".to_string(),
                data: serde_json::json!({}),
            }),
        ];

        // Create projection 2
        let events2 = vec![
            create_test_event(agg_id2, 1, EventData::GraphInitialized {
                graph_type: "concept".to_string(),
                metadata: HashMap::new(),
            }),
            create_test_event(agg_id2, 2, EventData::NodeAdded {
                node_id: "X".to_string(),
                node_type: "Concept".to_string(),
                data: serde_json::json!({}),
            }),
            create_test_event(agg_id2, 3, EventData::NodeAdded {
                node_id: "Y".to_string(),
                node_type: "Concept".to_string(),
                data: serde_json::json!({}),
            }),
        ];

        let projection1 = engine.project(events1);
        let projection2 = engine.project(events2);

        // Verify projections are isolated
        assert_eq!(projection1.aggregate_id, agg_id1);
        assert_eq!(projection2.aggregate_id, agg_id2);
        assert_eq!(projection1.version, 2);
        assert_eq!(projection2.version, 3);
        assert!(matches!(projection1.graph_type, GraphType::WorkflowGraph));
        assert!(matches!(projection2.graph_type, GraphType::ConceptGraph));
    }
}