# API Documentation Guide

This guide provides detailed documentation for the CIM Graph API, organized by module and functionality.

## Table of Contents

1. [Core API](#core-api)
2. [Graph Types API](#graph-types-api)
3. [Algorithms API](#algorithms-api)
4. [Serialization API](#serialization-api)
5. [Event System API](#event-system-api)
6. [Builder API](#builder-api)
7. [Error Handling](#error-handling)

## Core API

### Graph Trait

The fundamental trait that all graph types implement:

```rust
pub trait Graph {
    type Node: Node;
    type Edge: Edge;
    
    // Node operations
    fn add_node(&mut self, node: Self::Node) -> Result<NodeId>;
    fn remove_node(&mut self, id: NodeId) -> Result<Self::Node>;
    fn get_node(&self, id: NodeId) -> Option<&Self::Node>;
    fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Self::Node>;
    fn has_node(&self, id: NodeId) -> bool;
    fn node_count(&self) -> usize;
    fn nodes(&self) -> impl Iterator<Item = &Self::Node>;
    fn node_indices(&self) -> impl Iterator<Item = NodeId>;
    
    // Edge operations
    fn add_edge(&mut self, from: NodeId, to: NodeId, edge: Self::Edge) -> Result<EdgeId>;
    fn remove_edge(&mut self, id: EdgeId) -> Result<Self::Edge>;
    fn get_edge(&self, id: EdgeId) -> Option<&Self::Edge>;
    fn get_edge_mut(&mut self, id: EdgeId) -> Option<&mut Self::Edge>;
    fn has_edge(&self, from: NodeId, to: NodeId) -> bool;
    fn edge_count(&self) -> usize;
    fn edges(&self) -> impl Iterator<Item = &Self::Edge>;
    fn edge_indices(&self) -> impl Iterator<Item = EdgeId>;
    fn edge_endpoints(&self, id: EdgeId) -> Option<(NodeId, NodeId)>;
    
    // Graph operations
    fn neighbors(&self, id: NodeId) -> Vec<NodeId>;
    fn neighbors_directed(&self, id: NodeId, direction: Direction) -> Vec<NodeId>;
    fn degree(&self, id: NodeId) -> usize;
    fn in_degree(&self, id: NodeId) -> usize;
    fn out_degree(&self, id: NodeId) -> usize;
    fn clear(&mut self);
    fn is_directed(&self) -> bool;
    
    // Validation
    fn validate(&self) -> Result<()>;
}
```

### Node Trait

```rust
pub trait Node: Clone + Debug {
    type Data;
    
    fn id(&self) -> NodeId;
    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self) -> &mut Self::Data;
    fn node_type(&self) -> &str;
    fn metadata(&self) -> &HashMap<String, Value>;
    fn metadata_mut(&mut self) -> &mut HashMap<String, Value>;
}
```

### Edge Trait

```rust
pub trait Edge: Clone + Debug {
    type Data;
    
    fn id(&self) -> EdgeId;
    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self) -> &mut Self::Data;
    fn weight(&self) -> f64;
    fn set_weight(&mut self, weight: f64);
    fn edge_type(&self) -> &str;
    fn metadata(&self) -> &HashMap<String, Value>;
    fn metadata_mut(&mut self) -> &mut HashMap<String, Value>;
}
```

## Graph Types API

### IpldGraph

```rust
pub struct IpldGraph {
    // Private fields
}

impl IpldGraph {
    // Constructors
    pub fn new() -> Self;
    pub fn with_capacity(nodes: usize, edges: usize) -> Self;
    
    // IPLD-specific operations
    pub fn add_cid(&mut self, cid: &str, codec: &str, size: u64) -> Result<NodeId>;
    pub fn add_link(&mut self, from: NodeId, to: NodeId, link_type: &str, path: Option<&str>) -> Result<EdgeId>;
    pub fn get_cid(&self, id: NodeId) -> Option<&str>;
    pub fn get_codec(&self, id: NodeId) -> Option<&str>;
    pub fn get_size(&self, id: NodeId) -> Option<u64>;
    
    // DAG operations
    pub fn get_roots(&self) -> Vec<NodeId>;
    pub fn get_leaves(&self) -> Vec<NodeId>;
    pub fn get_children(&self, id: NodeId) -> Vec<NodeId>;
    pub fn get_parents(&self, id: NodeId) -> Vec<NodeId>;
    pub fn is_dag(&self) -> bool;
    pub fn validate_dag(&self) -> Result<()>;
    
    // Merkle operations
    pub fn merkle_proof(&self, root: NodeId, target: NodeId) -> Result<MerkleProof>;
    pub fn verify_merkle_proof(&self, proof: &MerkleProof) -> bool;
    pub fn calculate_root_hash(&self) -> Result<String>;
}
```

### ContextGraph

```rust
pub struct ContextGraph {
    // Private fields
}

impl ContextGraph {
    // Constructors
    pub fn new(bounded_context: &str) -> Self;
    pub fn with_capacity(bounded_context: &str, nodes: usize, edges: usize) -> Self;
    
    // DDD operations
    pub fn add_aggregate(&mut self, aggregate_type: &str, id: Uuid, data: Value) -> Result<NodeId>;
    pub fn add_entity(&mut self, entity_type: &str, id: Uuid, aggregate: NodeId, data: Value) -> Result<NodeId>;
    pub fn add_value_object(&mut self, vo_type: &str, parent: NodeId, data: Value) -> Result<NodeId>;
    pub fn add_relationship(&mut self, from: NodeId, to: NodeId, relationship: &str, cardinality: Cardinality) -> Result<EdgeId>;
    
    // Aggregate operations
    pub fn get_aggregate(&self, id: NodeId) -> Option<&Aggregate>;
    pub fn get_aggregate_root(&self, entity: NodeId) -> Option<NodeId>;
    pub fn get_aggregate_entities(&self, aggregate: NodeId) -> Vec<NodeId>;
    pub fn update_aggregate(&mut self, id: NodeId, data: Value) -> Result<u64>; // Returns new version
    
    // Bounded context operations
    pub fn bounded_context(&self) -> &str;
    pub fn list_aggregates(&self) -> Vec<NodeId>;
    pub fn list_entities(&self) -> Vec<NodeId>;
    pub fn list_value_objects(&self) -> Vec<NodeId>;
    pub fn aggregates_in_context(&self, context: &str) -> Result<Vec<NodeId>>;
    
    // Consistency operations
    pub fn consistency_boundary(&self, id: NodeId) -> Result<ConsistencyBoundary>;
    pub fn validate_invariants(&self) -> Result<()>;
}
```

### WorkflowGraph

```rust
pub struct WorkflowGraph {
    // Private fields
}

impl WorkflowGraph {
    // Constructors
    pub fn new(workflow_name: &str) -> Self;
    pub fn from_definition(definition: WorkflowDefinition) -> Result<Self>;
    
    // State operations
    pub fn add_state(&mut self, name: &str, state_type: StateType) -> Result<NodeId>;
    pub fn add_transition(&mut self, from: NodeId, to: NodeId, event: &str, guard: Option<Guard>) -> Result<EdgeId>;
    pub fn add_action(&mut self, state: NodeId, action: Action, trigger: ActionTrigger) -> Result<()>;
    
    // Workflow execution
    pub fn create_instance(&self, instance_id: &str) -> Result<WorkflowInstance>;
    pub fn get_current_state(&self, instance: &WorkflowInstance) -> Result<NodeId>;
    pub fn get_available_events(&self, instance: &WorkflowInstance) -> Result<Vec<String>>;
    pub fn trigger_event(&self, instance: &mut WorkflowInstance, event: &str, context: &Context) -> Result<NodeId>;
    
    // Complex state operations
    pub fn add_fork(&mut self, from: NodeId, targets: Vec<NodeId>, event: &str) -> Result<()>;
    pub fn add_join(&mut self, sources: Vec<NodeId>, target: NodeId, event: &str) -> Result<()>;
    pub fn add_choice(&mut self, from: NodeId, choices: Vec<(NodeId, Guard)>) -> Result<()>;
    
    // Validation and analysis
    pub fn validate_workflow(&self) -> Result<()>;
    pub fn find_unreachable_states(&self) -> Vec<NodeId>;
    pub fn find_deadlock_states(&self) -> Vec<NodeId>;
    pub fn get_all_paths(&self, from: NodeId, to: NodeId) -> Result<Vec<Vec<NodeId>>>;
}
```

### ConceptGraph

```rust
pub struct ConceptGraph {
    // Private fields
}

impl ConceptGraph {
    // Constructors
    pub fn new(domain: &str) -> Self;
    pub fn with_dimensions(domain: &str, dimensions: Vec<Dimension>) -> Self;
    
    // Concept operations
    pub fn add_concept(&mut self, name: &str, dimensions: Vec<Dimension>) -> Result<NodeId>;
    pub fn add_concept_with_prototype(&mut self, name: &str, prototype: &[(String, f64)]) -> Result<NodeId>;
    pub fn add_instance(&mut self, concept: NodeId, instance: Instance) -> Result<()>;
    pub fn add_relation(&mut self, from: NodeId, to: NodeId, relation: SemanticRelation, strength: f64) -> Result<EdgeId>;
    
    // Semantic operations
    pub fn semantic_similarity(&self, concept1: NodeId, concept2: NodeId) -> Result<f64>;
    pub fn conceptual_distance(&self, concept1: NodeId, concept2: NodeId) -> Result<f64>;
    pub fn find_similar(&self, concept: NodeId, threshold: f64) -> Result<Vec<(NodeId, f64)>>;
    pub fn conceptual_between(&self, concept1: NodeId, concept2: NodeId) -> Result<ConceptNode>;
    
    // Classification and reasoning
    pub fn classify_instance(&self, instance: &Instance) -> Result<Vec<(NodeId, f64)>>;
    pub fn infer_relations(&self) -> Result<Vec<InferredRelation>>;
    pub fn get_concept_hierarchy(&self, root: NodeId) -> Result<ConceptHierarchy>;
    
    // Analysis
    pub fn cluster_concepts(&self, num_clusters: usize) -> Result<Vec<Vec<NodeId>>>;
    pub fn find_prototypes(&self) -> Vec<NodeId>;
    pub fn concept_coverage(&self, instances: &[Instance]) -> Result<HashMap<NodeId, f64>>;
}
```

## Algorithms API

### Pathfinding

```rust
// Dijkstra's algorithm
pub fn dijkstra<G>(
    graph: &G,
    start: NodeId,
    goal: Option<NodeId>,
) -> Result<HashMap<NodeId, (f64, Vec<NodeId>)>>
where
    G: Graph,
    G::Edge: Weighted;

// A* algorithm
pub fn astar<G, H>(
    graph: &G,
    start: NodeId,
    goal: NodeId,
    heuristic: H,
) -> Result<Option<(f64, Vec<NodeId>)>>
where
    G: Graph,
    G::Edge: Weighted,
    H: Fn(&G::Node, &G::Node) -> f64;

// Bellman-Ford algorithm
pub fn bellman_ford<G>(
    graph: &G,
    start: NodeId,
) -> Result<Result<HashMap<NodeId, f64>, NegativeCycle>>
where
    G: Graph,
    G::Edge: Weighted;

// All pairs shortest paths
pub fn floyd_warshall<G>(
    graph: &G,
) -> Result<HashMap<(NodeId, NodeId), f64>>
where
    G: Graph,
    G::Edge: Weighted;
```

### Traversal

```rust
// Breadth-first search
pub fn bfs<G>(graph: &G, start: NodeId) -> Result<Vec<NodeId>>
where
    G: Graph;

pub fn bfs_with_callback<G, F>(
    graph: &G,
    start: NodeId,
    callback: F,
) -> Result<()>
where
    G: Graph,
    F: FnMut(BfsEvent) -> Result<()>;

// Depth-first search
pub fn dfs<G>(graph: &G) -> Result<HashMap<NodeId, DfsTimes>>
where
    G: Graph;

pub fn dfs_from<G>(graph: &G, start: NodeId) -> Result<Vec<NodeId>>
where
    G: Graph;

// Topological sort
pub fn topological_sort<G>(
    graph: &G,
) -> Result<Result<Vec<NodeId>, Vec<NodeId>>>
where
    G: Graph;
```

### Graph Analysis

```rust
// Connected components
pub fn connected_components<G>(graph: &G) -> Result<Vec<Vec<NodeId>>>
where
    G: Graph;

pub fn strongly_connected_components<G>(graph: &G) -> Result<Vec<Vec<NodeId>>>
where
    G: Graph;

// Cycle detection
pub fn has_cycle<G>(graph: &G) -> Result<bool>
where
    G: Graph;

pub fn find_cycles<G>(graph: &G) -> Result<Vec<Vec<NodeId>>>
where
    G: Graph;

// Centrality measures
pub fn degree_centrality<G>(graph: &G) -> Result<HashMap<NodeId, f64>>
where
    G: Graph;

pub fn betweenness_centrality<G>(graph: &G) -> Result<HashMap<NodeId, f64>>
where
    G: Graph;

pub fn closeness_centrality<G>(graph: &G) -> Result<HashMap<NodeId, f64>>
where
    G: Graph;

pub fn eigenvector_centrality<G>(
    graph: &G,
    iterations: usize,
) -> Result<HashMap<NodeId, f64>>
where
    G: Graph;

pub fn pagerank<G>(
    graph: &G,
    damping: f64,
    iterations: usize,
) -> Result<HashMap<NodeId, f64>>
where
    G: Graph;
```

## Serialization API

### JSON Serialization

```rust
// Basic serialization
pub fn to_json<G: Graph + Serialize>(graph: &G) -> Result<String>;
pub fn from_json<G: Graph + DeserializeOwned>(json: &str) -> Result<G>;

// Pretty printing
pub fn to_json_pretty<G: Graph + Serialize>(
    graph: &G,
    config: JsonConfig,
) -> Result<String>;

// With options
pub fn to_json_with_options<G: Graph + Serialize>(
    graph: &G,
    options: SerializeOptions,
) -> Result<String>;
```

### Binary Serialization

```rust
// Bincode
pub fn to_binary<G: Graph + Serialize>(graph: &G) -> Result<Vec<u8>>;
pub fn from_binary<G: Graph + DeserializeOwned>(bytes: &[u8]) -> Result<G>;
pub fn to_binary_compressed<G: Graph + Serialize>(
    graph: &G,
) -> Result<Vec<u8>>;

// MessagePack
pub fn to_msgpack<G: Graph + Serialize>(graph: &G) -> Result<Vec<u8>>;
pub fn from_msgpack<G: Graph + DeserializeOwned>(bytes: &[u8]) -> Result<G>;
```

### Streaming Serialization

```rust
pub struct StreamingJsonWriter<W: Write> {
    // Private fields
}

impl<W: Write> StreamingJsonWriter<W> {
    pub fn new(writer: W) -> Self;
    pub fn begin_graph(&mut self, graph_type: &str, metadata: Value) -> Result<()>;
    pub fn write_node<N: Serialize>(&mut self, node: &N) -> Result<()>;
    pub fn write_edge<E: Serialize>(&mut self, edge: &E) -> Result<()>;
    pub fn end_graph(&mut self) -> Result<()>;
}

pub struct StreamingJsonReader<R: Read> {
    // Private fields
}

impl<R: Read> StreamingJsonReader<R> {
    pub fn new(reader: R) -> Self;
    pub fn on_node<F>(&mut self, handler: F) -> &mut Self
    where
        F: FnMut(Value) -> Result<()>;
    pub fn on_edge<F>(&mut self, handler: F) -> &mut Self
    where
        F: FnMut(Value) -> Result<()>;
    pub fn read(&mut self) -> Result<()>;
}
```

## Event System API

### Event Types

```rust
#[derive(Debug, Clone)]
pub enum GraphEvent {
    NodeAdded {
        id: NodeId,
        node_type: String,
        timestamp: DateTime<Utc>,
    },
    NodeRemoved {
        id: NodeId,
        timestamp: DateTime<Utc>,
    },
    NodeUpdated {
        id: NodeId,
        old_data: Value,
        new_data: Value,
        timestamp: DateTime<Utc>,
    },
    EdgeAdded {
        id: EdgeId,
        from: NodeId,
        to: NodeId,
        edge_type: String,
        timestamp: DateTime<Utc>,
    },
    EdgeRemoved {
        id: EdgeId,
        timestamp: DateTime<Utc>,
    },
    EdgeUpdated {
        id: EdgeId,
        old_data: Value,
        new_data: Value,
        timestamp: DateTime<Utc>,
    },
    GraphCleared {
        timestamp: DateTime<Utc>,
    },
}
```

### EventGraph

```rust
pub struct EventGraph<G: Graph> {
    // Private fields
}

impl<G: Graph> EventGraph<G> {
    pub fn new() -> Self;
    pub fn wrap(graph: G) -> Self;
    
    // Event subscription
    pub fn subscribe<F>(&mut self, handler: F) -> SubscriptionId
    where
        F: Fn(&GraphEvent) + 'static;
    
    pub fn unsubscribe(&mut self, id: SubscriptionId);
    
    // Event history
    pub fn events(&self) -> &[GraphEvent];
    pub fn events_since(&self, timestamp: DateTime<Utc>) -> Vec<&GraphEvent>;
    pub fn clear_history(&mut self);
    
    // Event replay
    pub fn replay_events(&mut self, events: &[GraphEvent]) -> Result<()>;
    pub fn snapshot(&self) -> GraphSnapshot<G>;
    pub fn restore(&mut self, snapshot: GraphSnapshot<G>) -> Result<()>;
}
```

## Builder API

### GraphBuilder

```rust
pub struct GraphBuilder<N = DefaultNode, E = DefaultEdge> {
    // Private fields
}

impl<N: Node, E: Edge> GraphBuilder<N, E> {
    pub fn new() -> Self;
    
    // Configuration
    pub fn directed(mut self, directed: bool) -> Self;
    pub fn with_capacity(mut self, nodes: usize, edges: usize) -> Self;
    pub fn allow_parallel_edges(mut self, allow: bool) -> Self;
    pub fn allow_self_loops(mut self, allow: bool) -> Self;
    
    // Validation rules
    pub fn with_validation(mut self, rules: ValidationRules) -> Self;
    pub fn require_dag(mut self, require: bool) -> Self;
    pub fn max_node_degree(mut self, max: usize) -> Self;
    
    // Metadata
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self;
    pub fn with_description(mut self, description: &str) -> Self;
    
    // Build
    pub fn build(self) -> Graph<N, E>;
    pub fn build_validated(self) -> Result<Graph<N, E>>;
}
```

### ValidationRules

```rust
pub struct ValidationRules {
    pub require_connected: bool,
    pub require_dag: bool,
    pub forbid_self_loops: bool,
    pub forbid_parallel_edges: bool,
    pub max_nodes: Option<usize>,
    pub max_edges: Option<usize>,
    pub max_node_degree: Option<usize>,
    pub custom_validators: Vec<Box<dyn Validator>>,
}

pub trait Validator {
    fn validate<G: Graph>(&self, graph: &G) -> Result<()>;
}
```

## Error Handling

### Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum GraphError {
    #[error("Node {0} not found")]
    NodeNotFound(NodeId),
    
    #[error("Edge {0} not found")]
    EdgeNotFound(EdgeId),
    
    #[error("Edge between {from} and {to} not found")]
    EdgeBetweenNotFound { from: NodeId, to: NodeId },
    
    #[error("Operation would create cycle")]
    CycleDetected,
    
    #[error("Graph validation failed: {0}")]
    ValidationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Algorithm error: {0}")]
    AlgorithmError(String),
    
    #[error("Capacity exceeded: {0}")]
    CapacityExceeded(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, GraphError>;
```

### Error Context

```rust
pub trait ResultExt<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| GraphError::InvalidOperation(format!("{}: {}", f(), e)))
    }
}

// Usage
graph.add_node(node)
    .with_context(|| format!("Failed to add node with id {}", node.id()))?;
```