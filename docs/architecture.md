# CIM Graph Architecture

## Overview

CIM Graph is designed as a unified graph abstraction layer that provides a consistent interface for multiple graph paradigms while preserving their semantic differences. The architecture follows several key principles:

1. **Zero-cost abstractions** - No runtime overhead compared to using petgraph directly
2. **Type safety** - Leverage Rust's type system to prevent errors at compile time
3. **Event sourcing** - All mutations produce events for audit and replay
4. **Composability** - Graphs can be composed without losing type information
5. **Extensibility** - Easy to add new graph types and algorithms

## Core Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Public API Layer                      │
├─────────────────────────────────────────────────────────┤
│  GraphBuilder │ EventGraph │ Algorithms │ Serialization │
├─────────────────────────────────────────────────────────┤
│                  Graph Type Layer                        │
├──────────┬──────────┬──────────┬──────────┬────────────┤
│   IPLD   │ Context  │ Workflow │ Concept  │  Composed  │
│  Graph   │  Graph   │  Graph   │  Graph   │   Graph    │
├──────────┴──────────┴──────────┴──────────┴────────────┤
│                    Core Abstractions                     │
├─────────────────────────────────────────────────────────┤
│    Node    │    Edge    │   Graph Trait  │   Events    │
├─────────────────────────────────────────────────────────┤
│                  Storage Backend                         │
├─────────────────────────────────────────────────────────┤
│                     Petgraph                            │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### Node and Edge Types

The library uses generic node and edge types that can be specialized for different domains:

```rust
pub struct Node<T> {
    pub id: NodeId,
    pub data: T,
    pub metadata: HashMap<String, Value>,
}

pub struct Edge<T> {
    pub id: EdgeId,
    pub data: T,
    pub weight: f64,
    pub metadata: HashMap<String, Value>,
}
```

### Graph Trait

All graph types implement the core `Graph` trait:

```rust
pub trait Graph {
    type Node;
    type Edge;
    
    fn add_node(&mut self, node: Self::Node) -> Result<NodeId>;
    fn add_edge(&mut self, from: NodeId, to: NodeId, edge: Self::Edge) -> Result<EdgeId>;
    fn get_node(&self, id: NodeId) -> Option<&Self::Node>;
    fn get_edge(&self, id: EdgeId) -> Option<&Self::Edge>;
    fn neighbors(&self, id: NodeId) -> Vec<NodeId>;
    // ... more methods
}
```

### Event System

All graph mutations emit events through the event system:

```rust
pub enum GraphEvent {
    NodeAdded { id: NodeId, timestamp: DateTime<Utc> },
    NodeRemoved { id: NodeId, timestamp: DateTime<Utc> },
    EdgeAdded { id: EdgeId, from: NodeId, to: NodeId, timestamp: DateTime<Utc> },
    EdgeRemoved { id: EdgeId, timestamp: DateTime<Utc> },
    // ... more events
}
```

## Graph Types

### IPLD Graph

Specialized for content-addressed data with CID nodes:

```rust
pub struct IpldNode {
    pub cid: String,
    pub codec: String,
    pub size: u64,
}

pub struct IpldEdge {
    pub link_type: String,
    pub path: Option<String>,
}
```

### Context Graph

Models domain relationships with aggregate roots:

```rust
pub struct ContextNode {
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub version: u64,
}

pub struct ContextEdge {
    pub relationship: String,
    pub cardinality: Cardinality,
}
```

### Workflow Graph

Represents state machines and transitions:

```rust
pub struct WorkflowNode {
    pub state: String,
    pub entry_actions: Vec<Action>,
    pub exit_actions: Vec<Action>,
}

pub struct WorkflowEdge {
    pub event: String,
    pub guard: Option<Guard>,
    pub actions: Vec<Action>,
}
```

### Concept Graph

Semantic reasoning with concepts and relations:

```rust
pub struct ConceptNode {
    pub concept: String,
    pub attributes: HashMap<String, f64>,
    pub prototype: Option<Prototype>,
}

pub struct ConceptEdge {
    pub relation: SemanticRelation,
    pub strength: f64,
}
```

## Composition System

The composition system allows combining multiple graphs:

```rust
pub struct ComposedGraph {
    graphs: HashMap<String, Box<dyn Graph>>,
    mappings: Vec<NodeMapping>,
    strategy: CompositionStrategy,
}
```

### Mapping Rules

Nodes can be mapped between graphs using various strategies:

1. **By ID** - Direct ID mapping
2. **By Property** - Match nodes with same property values
3. **By Predicate** - Custom matching function
4. **By Type** - Match nodes of compatible types

### Composition Strategies

- **PreserveAll** - Keep all nodes and edges from all graphs
- **Union** - Merge nodes that map to each other
- **Intersection** - Only keep nodes present in all graphs
- **Difference** - Keep nodes unique to specific graphs

## Algorithm Framework

Algorithms are implemented as generic functions over the Graph trait:

```rust
pub fn dijkstra<G: Graph>(
    graph: &G,
    start: NodeId,
    goal: Option<NodeId>,
) -> HashMap<NodeId, (f64, Vec<NodeId>)>
where
    G::Edge: Weighted,
{
    // Implementation
}
```

### Available Algorithms

- **Pathfinding**: Dijkstra, A*, Bellman-Ford
- **Traversal**: BFS, DFS, topological sort
- **Analysis**: Connected components, strongly connected components
- **Metrics**: Centrality measures, clustering coefficients
- **Pattern Matching**: Subgraph isomorphism, motif finding

## Storage and Serialization

### In-Memory Storage

The default storage backend uses petgraph's adjacency list:

```rust
type Storage<N, E> = petgraph::Graph<N, E, petgraph::Directed>;
```

### Serialization Formats

1. **JSON** - Human-readable, web-compatible
2. **Nix** - For integration with Nix package manager
3. **Binary** - Efficient storage and transmission (planned)

### Schema

```json
{
  "version": "1.0",
  "metadata": {},
  "nodes": [
    {
      "id": "node-1",
      "type": "Person",
      "data": { "name": "Alice" },
      "metadata": {}
    }
  ],
  "edges": [
    {
      "id": "edge-1",
      "from": "node-1",
      "to": "node-2",
      "type": "knows",
      "data": {},
      "metadata": {}
    }
  ]
}
```

## Performance Considerations

### Memory Layout

- Nodes and edges are stored contiguously for cache efficiency
- Metadata is stored separately to avoid bloating core structures
- Indices are maintained for O(1) lookups

### Parallelism

- Read operations are thread-safe
- Write operations require exclusive access
- Some algorithms support parallel execution

### Optimization Strategies

1. **Lazy Loading** - Load graph data on demand
2. **Incremental Updates** - Efficient batch modifications
3. **Index Structures** - Maintain indices for common queries
4. **Memory Pooling** - Reuse allocations for temporary data

## Extension Points

### Custom Graph Types

New graph types can be added by:

1. Defining node and edge types
2. Implementing the Graph trait
3. Adding specialized methods
4. Registering with the composition system

### Custom Algorithms

Algorithms can be added as generic functions:

```rust
pub fn my_algorithm<G: Graph>(graph: &G) -> Result<MyResult> 
where
    G::Node: MyNodeTrait,
    G::Edge: MyEdgeTrait,
{
    // Implementation
}
```

### Event Handlers

Custom event handlers can be registered:

```rust
graph.subscribe(|event: &GraphEvent| {
    // Custom handling logic
});
```

## Future Directions

1. **Persistent Storage** - Database backends
2. **Distributed Graphs** - Multi-machine graph processing
3. **GPU Acceleration** - CUDA/OpenCL for large graphs
4. **Streaming APIs** - Process graphs as streams
5. **Query Language** - GraphQL-like query interface