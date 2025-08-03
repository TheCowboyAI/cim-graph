# CIM Graph Design Document

## Overview

`cim-graph` is a unified graph abstraction library that consolidates all graph operations across the CIM ecosystem. It provides a single, consistent interface for working with different graph types while maintaining their unique semantic properties.

## Problem Statement

Currently, CIM has multiple graph implementations:
- **cim-ipld-graph**: Markov chains of IPLD CIDs
- **cim-contextgraph**: DDD object relationships (e.g., StreetAddress hierarchies)
- **cim-workflow-graph**: Workflow event Markov chains
- **cim-conceptgraph**: ConceptualSpaces reasoning graphs

All use petgraph but have different ways of viewing nodes and edges. This duplication leads to:
- Inconsistent APIs across graph types
- Difficulty composing graphs from different domains
- Redundant serialization/deserialization code
- No unified way to transform between graph types

## Design Goals

1. **Unified Interface**: Single API for all graph operations
2. **Type Safety**: Preserve semantic differences through Rust's type system
3. **Composability**: Easy composition of graphs from different domains
4. **Serialization**: Consistent JSON and Nix output formats
5. **Performance**: Zero-cost abstractions where possible
6. **Extensibility**: Easy to add new graph types

## Architecture

### Core Abstraction

```rust
// Base trait that all graph types implement
trait CimGraph {
    type Node;
    type Edge;
    type Metadata;
    
    fn add_node(&mut self, node: Self::Node) -> NodeId;
    fn add_edge(&mut self, from: NodeId, to: NodeId, edge: Self::Edge) -> EdgeId;
    fn compose_with<G: CimGraph>(&self, other: &G) -> ComposedGraph;
    fn to_json(&self) -> serde_json::Value;
    fn to_nix(&self) -> String;
}
```

### Graph Types

Each specialized graph type implements the base trait:

```rust
// IPLD Graph - Markov chain of CIDs
struct IpldGraph {
    graph: Graph<Cid, Transition>,
    metadata: IpldMetadata,
}

// Context Graph - DDD relationships
struct ContextGraph {
    graph: Graph<DomainObject, Relationship>,
    metadata: ContextMetadata,
}

// Workflow Graph - Event chains
struct WorkflowGraph {
    graph: Graph<WorkflowState, Event>,
    metadata: WorkflowMetadata,
}

// Concept Graph - Semantic reasoning
struct ConceptGraph {
    graph: Graph<Concept, SemanticRelation>,
    metadata: ConceptMetadata,
}
```

### Composition Strategy

```rust
enum ComposedGraph {
    Homogeneous(Box<dyn CimGraph>),
    Heterogeneous {
        graphs: Vec<Box<dyn CimGraph>>,
        mappings: Vec<GraphMapping>,
    }
}

struct GraphMapping {
    source_graph: GraphId,
    source_node: NodeId,
    target_graph: GraphId,
    target_node: NodeId,
    mapping_type: MappingType,
}
```

### Serialization Format

#### JSON Structure
```json
{
  "type": "cim-graph",
  "version": "1.0",
  "graph_type": "context|ipld|workflow|concept|composed",
  "metadata": {},
  "nodes": [
    {
      "id": "node-1",
      "type": "specific-type",
      "data": {}
    }
  ],
  "edges": [
    {
      "id": "edge-1",
      "from": "node-1",
      "to": "node-2",
      "type": "relationship-type",
      "data": {}
    }
  ]
}
```

#### Nix Expression
```nix
{
  type = "cim-graph";
  version = "1.0";
  graphType = "context";
  nodes = [
    { id = "node-1"; type = "address"; data = { street = "123 Main"; }; }
  ];
  edges = [
    { from = "node-1"; to = "node-2"; type = "contains"; }
  ];
}
```

## Implementation Plan

### Phase 1: Core Abstractions
- Define base `CimGraph` trait
- Create node and edge ID types
- Implement basic graph operations

### Phase 2: Graph Type Implementations
- Migrate IpldGraph functionality
- Migrate ContextGraph functionality
- Migrate WorkflowGraph functionality
- Migrate ConceptGraph functionality

### Phase 3: Composition
- Implement graph composition algorithms
- Define mapping strategies
- Create conflict resolution rules

### Phase 4: Serialization
- JSON serialization/deserialization
- Nix expression generation
- Format validation

### Phase 5: Integration
- Replace existing graph implementations
- Update dependent modules
- Migration guide for existing code

## Dependencies

- `petgraph`: Core graph data structure
- `serde`/`serde_json`: JSON serialization
- `uuid`: Node and edge identifiers
- `cid`: For IPLD graph support

## Testing Strategy

1. **Unit Tests**: Each graph type implementation
2. **Integration Tests**: Graph composition scenarios
3. **Property Tests**: Graph invariants with proptest
4. **Serialization Tests**: Round-trip JSON/Nix conversion
5. **Performance Tests**: Benchmark against existing implementations

## Migration Path

1. Implement cim-graph alongside existing modules
2. Create adapters for backward compatibility
3. Gradually migrate dependent code
4. Deprecate individual graph modules
5. Remove deprecated modules after transition period