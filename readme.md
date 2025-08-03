CIM Graph - Unified Graph Abstraction

A unified graph abstraction library that consolidates all graph operations across the CIM ecosystem. It provides a single, consistent interface for working with different graph types while maintaining their unique semantic properties.

## Overview

`cim-graph` abstracts the common graph operations from:
- **cim-ipld-graph**: Markov chains of IPLD CIDs
- **cim-contextgraph**: Domain-Driven Design object relationships
- **cim-workflow-graph**: Workflow event Markov chains  
- **cim-conceptgraph**: ConceptualSpaces reasoning graphs

All these graph concepts are unified under a single API while preserving their semantic differences through Rust's type system.

## Features

✅ **Unified Interface**: Single API for all graph operations
✅ **Type-Safe Abstractions**: Preserve semantic differences of each graph type
✅ **Graph Composition**: Combine graphs from different domains
✅ **Serialization**: Export to JSON and Nix expressions
✅ **Event-Driven**: All operations emit domain events
✅ **Zero-Cost Abstractions**: Performance equivalent to direct petgraph usage

## Usage Examples

### Working with Different Graph Types

```rust
use cim_graph::{IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};

// IPLD Graph - Track content-addressed data relationships
let mut ipld_graph = IpldGraph::new();
let cid1 = ipld_graph.add_cid(cid);
let cid2 = ipld_graph.add_cid(next_cid);
ipld_graph.add_transition(cid1, cid2, transition);

// Context Graph - Model domain relationships
let mut context_graph = ContextGraph::new();
let address = context_graph.add_object(StreetAddress::new());
let city = context_graph.add_object(City::new());
context_graph.add_relationship(address, city, Relationship::LocatedIn);

// Workflow Graph - Track state transitions
let mut workflow_graph = WorkflowGraph::new();
let state1 = workflow_graph.add_state(WorkflowState::Started);
let state2 = workflow_graph.add_state(WorkflowState::Processing);
workflow_graph.add_event(state1, state2, Event::Submit);

// Concept Graph - Semantic reasoning
let mut concept_graph = ConceptGraph::new();
let concept1 = concept_graph.add_concept(Concept::new("Vehicle"));
let concept2 = concept_graph.add_concept(Concept::new("Car"));
concept_graph.add_relation(concept1, concept2, SemanticRelation::IsA);
```

### Graph Composition

```rust
use cim_graph::{compose_graphs, CompositionStrategy};

// Compose multiple graphs with mapping rules
let composed = compose_graphs()
    .add_graph(ipld_graph)
    .add_graph(context_graph)
    .add_graph(workflow_graph)
    .with_strategy(CompositionStrategy::PreserveAll)
    .compose();

// Query across composed graphs
let results = composed.query()
    .from_graph_type::<ContextGraph>()
    .traverse_to::<WorkflowGraph>()
    .execute();
```

### Serialization

```rust
// Export to JSON
let json = graph.to_json()?;
std::fs::write("graph.json", json)?;

// Export to Nix
let nix_expr = graph.to_nix()?;
std::fs::write("graph.nix", nix_expr)?;

// Import from JSON
let imported: IpldGraph = CimGraph::from_json(&json)?;
```

## Design Principles

1. **Unified but Not Uniform**: While providing a consistent API, each graph type maintains its semantic meaning
2. **Composition Over Inheritance**: Graphs can be composed without losing type information
3. **Event Sourcing**: All graph mutations produce events for audit and replay
4. **Serialization First**: Native support for JSON and Nix serialization

## Graph Types

### IpldGraph
Represents Markov chains of IPLD CIDs, useful for tracking content-addressed data flows and dependencies.

### ContextGraph  
Models Domain-Driven Design relationships between objects, such as hierarchical structures (Country → Region → City → Street).

### WorkflowGraph
Captures workflow state transitions as Markov chains of events, enabling process visualization and analysis.

### ConceptGraph
Implements ConceptualSpaces for semantic reasoning about relationships between concepts.

## Architecture

See [doc/design/cim-graph-design.md](doc/design/cim-graph-design.md) for detailed architecture documentation.

## License

Dual-licensed under MIT and Apache 2.0
