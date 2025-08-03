# CIM Graph API Contracts

## Overview

This document defines the complete API contracts for the CIM Graph library based on the user stories and domain model. All APIs are designed as graph operations that can be composed and validated.

## Core Graph API

### Graph Creation and Management

```rust
/// Primary graph creation API
pub trait GraphFactory {
    /// Create a new graph with specified type and constraints
    /// 
    /// # Contract
    /// - Returns `Ok(Graph)` if all constraints are valid
    /// - Returns `Err(GraphError)` if constraints conflict
    /// 
    /// # Example
    /// ```rust
    /// let graph = GraphBuilder::new()
    ///     .graph_type(GraphType::IpldGraph)
    ///     .with_constraint(Constraint::Acyclic)
    ///     .build()?;
    /// ```
    fn build(self) -> Result<Graph, GraphError>;
}

/// Core graph operations
pub trait GraphOperations {
    type Node;
    type Edge;
    type NodeId;
    type EdgeId;
    
    /// Add a node to the graph
    /// 
    /// # Contract
    /// - Node ID must be unique within the graph
    /// - Node type must match graph type constraints
    /// - Returns generated NodeId on success
    /// 
    /// # Errors
    /// - `DuplicateNode` if node ID already exists
    /// - `TypeMismatch` if node type incompatible
    /// - `ConstraintViolation` if operation violates constraints
    fn add_node(&mut self, node: Self::Node) -> Result<Self::NodeId, GraphError>;
    
    /// Remove a node and all connected edges
    /// 
    /// # Contract
    /// - Removes all edges connected to the node
    /// - Returns the removed node data
    /// - Maintains referential integrity
    /// 
    /// # Errors
    /// - `NodeNotFound` if node doesn't exist
    fn remove_node(&mut self, id: &Self::NodeId) -> Result<Self::Node, GraphError>;
    
    /// Connect two nodes with an edge
    /// 
    /// # Contract
    /// - Both nodes must exist in the graph
    /// - Edge must satisfy graph constraints
    /// - Returns generated EdgeId on success
    /// 
    /// # Errors
    /// - `NodeNotFound` if either node doesn't exist
    /// - `DuplicateEdge` if edge already exists
    /// - `ConstraintViolation` if edge violates constraints
    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        to: &Self::NodeId,
        edge: Self::Edge
    ) -> Result<Self::EdgeId, GraphError>;
    
    /// Remove an edge between nodes
    /// 
    /// # Contract
    /// - Returns the removed edge data
    /// - Nodes remain in the graph
    /// 
    /// # Errors
    /// - `EdgeNotFound` if edge doesn't exist
    fn remove_edge(&mut self, id: &Self::EdgeId) -> Result<Self::Edge, GraphError>;
}
```

### Graph Query API

```rust
/// Query operations on graphs
pub trait GraphQuery {
    type NodeId;
    type Node;
    type Edge;
    
    /// Get a node by ID
    /// 
    /// # Contract
    /// - Returns `Some(node)` if exists
    /// - Returns `None` if not found
    /// - O(1) complexity with indexing
    fn get_node(&self, id: &Self::NodeId) -> Option<&Self::Node>;
    
    /// Get all neighbors of a node
    /// 
    /// # Contract
    /// - Returns outgoing neighbors for directed graphs
    /// - Returns all connected nodes for undirected graphs
    /// - Empty vec if node has no neighbors
    fn neighbors(&self, id: &Self::NodeId) -> Vec<&Self::NodeId>;
    
    /// Get nodes matching a predicate
    /// 
    /// # Contract
    /// - Predicate evaluated on all nodes
    /// - Returns references to matching nodes
    /// - Order not guaranteed unless specified
    fn find_nodes<P>(&self, predicate: P) -> Vec<&Self::Node>
    where
        P: Fn(&Self::Node) -> bool;
    
    /// Check if path exists between nodes
    /// 
    /// # Contract
    /// - Returns true if any path exists
    /// - Works for directed and undirected graphs
    /// - Uses BFS for unweighted graphs
    fn has_path(&self, from: &Self::NodeId, to: &Self::NodeId) -> bool;
    
    /// Find shortest path between nodes
    /// 
    /// # Contract
    /// - Returns None if no path exists
    /// - Returns optimal path for weighted graphs
    /// - Uses Dijkstra's algorithm for weighted graphs
    fn shortest_path(
        &self,
        from: &Self::NodeId,
        to: &Self::NodeId
    ) -> Option<GraphPath<Self::NodeId>>;
    
    /// Detect cycles in the graph
    /// 
    /// # Contract
    /// - Returns all cycles found
    /// - Empty vec for acyclic graphs
    /// - Each cycle starts with lowest ID node
    fn find_cycles(&self) -> Vec<GraphPath<Self::NodeId>>;
}
```

### Graph Metrics API

```rust
/// Compute graph metrics and statistics
pub trait GraphMetrics {
    /// Calculate basic graph metrics
    /// 
    /// # Contract
    /// - All metrics computed in single pass where possible
    /// - Expensive metrics (diameter) computed lazily
    /// - Returns complete metrics snapshot
    fn metrics(&self) -> GraphMetricsData;
    
    /// Get degree of a specific node
    /// 
    /// # Contract
    /// - In-degree + out-degree for directed graphs
    /// - Total connections for undirected graphs
    /// - Returns 0 for non-existent nodes
    fn degree(&self, node_id: &NodeId) -> usize;
    
    /// Calculate clustering coefficient
    /// 
    /// # Contract
    /// - Returns value between 0 and 1
    /// - 0 for graphs with no triangles
    /// - 1 for complete graphs
    fn clustering_coefficient(&self) -> f64;
    
    /// Count connected components
    /// 
    /// # Contract
    /// - Returns 1 for connected graphs
    /// - Returns node count for graphs with no edges
    /// - Uses union-find for efficiency
    fn connected_components(&self) -> usize;
}
```

## Type-Specific APIs

### IpldGraph API

```rust
/// IPLD-specific graph operations
pub trait IpldGraphOperations {
    /// Add content-addressed node
    /// 
    /// # Contract
    /// - Computes CID from content
    /// - Links must reference existing CIDs
    /// - Returns computed CID
    fn add_ipld_node(&mut self, content: IpldData) -> Result<Cid, IpldError>;
    
    /// Create Merkle DAG link
    /// 
    /// # Contract
    /// - Target CID must exist in graph
    /// - Link name must be unique for source
    /// - Maintains DAG property (no cycles)
    fn add_merkle_link(
        &mut self,
        from: &Cid,
        to: &Cid,
        link: Link
    ) -> Result<(), IpldError>;
    
    /// Traverse Merkle path
    /// 
    /// # Contract
    /// - Path segments follow link names
    /// - Returns None if path invalid
    /// - Supports nested path traversal
    fn resolve_path(&self, base: &Cid, path: &str) -> Option<&IpldData>;
}
```

### ContextGraph API

```rust
/// Domain-driven design graph operations
pub trait ContextGraphOperations {
    /// Add domain entity
    /// 
    /// # Contract
    /// - Entity must belong to bounded context
    /// - Aggregate roots marked appropriately
    /// - Returns entity ID
    fn add_entity(
        &mut self,
        entity: DomainEntity,
        context: BoundedContext
    ) -> Result<EntityId, ContextError>;
    
    /// Create domain relationship
    /// 
    /// # Contract
    /// - Validates cardinality constraints
    /// - Ensures relationship makes domain sense
    /// - Bidirectional relationships created atomically
    fn add_relationship(
        &mut self,
        from: &EntityId,
        to: &EntityId,
        relationship: DomainRelationship
    ) -> Result<(), ContextError>;
    
    /// Find aggregate root
    /// 
    /// # Contract
    /// - Returns root of aggregate containing entity
    /// - Returns self if entity is root
    /// - Follows composition relationships
    fn find_aggregate_root(&self, entity: &EntityId) -> Option<&EntityId>;
}
```

### WorkflowGraph API

```rust
/// Workflow and state machine operations
pub trait WorkflowGraphOperations {
    /// Add workflow state
    /// 
    /// # Contract
    /// - State names unique within workflow
    /// - Entry/exit actions validated
    /// - Initial state marked if first
    fn add_state(
        &mut self,
        state: WorkflowState
    ) -> Result<StateId, WorkflowError>;
    
    /// Add state transition
    /// 
    /// # Contract
    /// - Source and target states must exist
    /// - Event triggers must be unique per source
    /// - Guards evaluated before transition
    fn add_transition(
        &mut self,
        from: &StateId,
        to: &StateId,
        transition: StateTransition
    ) -> Result<(), WorkflowError>;
    
    /// Execute workflow step
    /// 
    /// # Contract
    /// - Returns new state after transition
    /// - Executes exit/entry actions
    /// - Returns None if transition blocked
    fn process_event(
        &mut self,
        current: &StateId,
        event: &Event
    ) -> Option<StateId>;
}
```

### ConceptGraph API

```rust
/// Semantic and reasoning operations
pub trait ConceptGraphOperations {
    /// Add concept with embedding
    /// 
    /// # Contract
    /// - Embedding dimension matches graph config
    /// - Quality dimensions normalized
    /// - Returns concept ID
    fn add_concept(
        &mut self,
        concept: Concept,
        embedding: Vector
    ) -> Result<ConceptId, ConceptError>;
    
    /// Create semantic relation
    /// 
    /// # Contract
    /// - Relation strength between 0 and 1
    /// - Context must be valid
    /// - Symmetric relations created bidirectionally
    fn add_semantic_relation(
        &mut self,
        from: &ConceptId,
        to: &ConceptId,
        relation: SemanticRelation
    ) -> Result<(), ConceptError>;
    
    /// Find similar concepts
    /// 
    /// # Contract
    /// - Uses embedding distance
    /// - Returns top K similar concepts
    /// - Excludes query concept from results
    fn find_similar(
        &self,
        concept: &ConceptId,
        k: usize
    ) -> Vec<(ConceptId, f64)>;
}
```

## Composition API

```rust
/// Graph composition operations
pub trait GraphComposition {
    /// Compose multiple graphs
    /// 
    /// # Contract
    /// - All graphs must exist
    /// - Mappings reference valid nodes
    /// - Returns new composed graph ID
    /// 
    /// # Example
    /// ```rust
    /// let composed = GraphComposer::new()
    ///     .add_graph("ipld", ipld_graph)
    ///     .add_graph("context", context_graph)
    ///     .add_mapping("ipld", cid, "context", entity_id)
    ///     .compose()?;
    /// ```
    fn compose(
        self,
        graphs: Vec<(&str, Box<dyn Graph>)>,
        mappings: Vec<GraphMapping>
    ) -> Result<ComposedGraph, CompositionError>;
    
    /// Query across composed graphs
    /// 
    /// # Contract
    /// - Traverses mappings transparently
    /// - Maintains type safety
    /// - Returns unified results
    fn cross_graph_query(
        &self,
        start_graph: &str,
        start_node: &NodeId,
        pattern: CrossGraphPattern
    ) -> Vec<CrossGraphResult>;
}
```

## Serialization API

```rust
/// Serialization contracts
pub trait GraphSerialization {
    /// Serialize to JSON
    /// 
    /// # Contract
    /// - Preserves all graph structure
    /// - Includes metadata and constraints
    /// - Round-trip safe
    /// 
    /// # Schema
    /// ```json
    /// {
    ///   "version": "1.0",
    ///   "type": "GraphType",
    ///   "nodes": [...],
    ///   "edges": [...],
    ///   "metadata": {...},
    ///   "constraints": [...]
    /// }
    /// ```
    fn to_json(&self) -> serde_json::Value;
    
    /// Serialize to Nix
    /// 
    /// # Contract
    /// - Valid Nix expression
    /// - Evaluates to attribute set
    /// - Preserves type information
    /// 
    /// # Format
    /// ```nix
    /// {
    ///   type = "GraphType";
    ///   nodes = [...];
    ///   edges = [...];
    ///   metadata = {...};
    /// }
    /// ```
    fn to_nix(&self) -> String;
    
    /// Deserialize from JSON
    /// 
    /// # Contract
    /// - Validates schema version
    /// - Reconstructs constraints
    /// - Returns error on invalid data
    fn from_json(json: &serde_json::Value) -> Result<Self, SerdeError>;
}
```

## Transformation API

```rust
/// Graph transformation contracts
pub trait GraphTransformation<S, T> {
    /// Transform graph type
    /// 
    /// # Contract
    /// - Preserves graph topology
    /// - Maps nodes and edges according to rules
    /// - Tracks transformation provenance
    /// 
    /// # Example
    /// ```rust
    /// let concept_graph = workflow_graph
    ///     .transform::<ConceptGraph>()
    ///     .with_node_mapper(|state| state.to_concept())
    ///     .with_edge_mapper(|trans| trans.to_relation())
    ///     .execute()?;
    /// ```
    fn transform(&self) -> TransformationBuilder<S, T>;
    
    /// Validate transformation feasibility
    /// 
    /// # Contract
    /// - Checks type compatibility
    /// - Validates mapping completeness
    /// - Returns detailed compatibility report
    fn can_transform_to<T>(&self) -> TransformationReport;
}
```

## Validation API

```rust
/// Constraint validation contracts
pub trait GraphValidation {
    /// Validate all constraints
    /// 
    /// # Contract
    /// - Checks all registered constraints
    /// - Returns first violation found
    /// - Ok(()) if all constraints satisfied
    fn validate(&self) -> Result<(), ValidationError>;
    
    /// Add runtime constraint
    /// 
    /// # Contract
    /// - Constraint checked on every mutation
    /// - Can be removed later
    /// - Returns constraint ID
    fn add_constraint(
        &mut self,
        constraint: Constraint
    ) -> ConstraintId;
    
    /// Custom validation rule
    /// 
    /// # Contract
    /// - Rule has access to full graph
    /// - Can check complex properties
    /// - Returns detailed violation info
    fn add_validation_rule<F>(
        &mut self,
        name: &str,
        rule: F
    ) -> RuleId
    where
        F: Fn(&Self) -> ValidationResult;
}
```

## Error Contracts

```rust
/// Standard error types
#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),
    
    #[error("Edge not found: {0}")]
    EdgeNotFound(EdgeId),
    
    #[error("Duplicate node: {0}")]
    DuplicateNode(NodeId),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Detailed validation results
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<ConstraintViolation>,
    pub warnings: Vec<ValidationWarning>,
}
```

## Performance Contracts

### Complexity Guarantees

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| add_node | O(1) | Amortized with resizing |
| add_edge | O(1) | With adjacency list |
| get_node | O(1) | With indexing |
| neighbors | O(degree) | Direct lookup |
| shortest_path | O((V+E)log V) | Dijkstra with heap |
| find_cycles | O(V+E) | DFS-based |
| metrics | O(V+E) | Single pass |

### Memory Guarantees

```rust
/// Memory usage bounds
pub trait MemoryBounds {
    /// Maximum memory per node
    const MAX_NODE_SIZE: usize = 1024; // 1KB
    
    /// Maximum memory per edge  
    const MAX_EDGE_SIZE: usize = 256;  // 256B
    
    /// Index overhead percentage
    const INDEX_OVERHEAD: f64 = 0.2;   // 20%
}
```

## Thread Safety

```rust
/// Thread-safe graph operations
pub trait SyncGraph: Send + Sync {
    /// Concurrent read access
    fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self) -> R;
    
    /// Exclusive write access
    fn write<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}
```

## Next Steps

1. Generate Rust trait definitions from contracts
2. Create contract tests
3. Implement mock implementations
4. Set up contract verification
5. Document usage examples