# CIM Graph

[![Crates.io](https://img.shields.io/crates/v/cim-graph.svg)](https://crates.io/crates/cim-graph)
[![Documentation](https://docs.rs/cim-graph/badge.svg)](https://docs.rs/cim-graph)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](LICENSE)

A high-performance, type-safe graph abstraction library that unifies multiple graph paradigms under a single, consistent API. CIM Graph provides specialized graph types for different domains while maintaining semantic clarity and zero-cost abstractions.

## Overview

CIM Graph consolidates various graph operations from across the CIM ecosystem into a unified interface:

- **IPLD Graphs**: Content-addressed data relationships and Markov chains
- **Context Graphs**: Domain-Driven Design object relationships and hierarchies
- **Workflow Graphs**: State machines and workflow transitions
- **Concept Graphs**: Semantic reasoning and conceptual spaces
- **Composed Graphs**: Multi-domain graph compositions with cross-graph queries

## Key Features

- 🚀 **High Performance**: Built on petgraph with zero-cost abstractions
- 🔒 **Type Safety**: Leverage Rust's type system to prevent runtime errors
- 🎯 **Domain-Specific**: Specialized graph types for different use cases
- 🔄 **Event Sourcing**: All operations emit domain events for audit trails
- 📦 **Serialization**: Native support for JSON and Nix expressions
- 🧩 **Composable**: Combine graphs from different domains seamlessly
- 📊 **Rich Algorithms**: Built-in pathfinding, traversal, and analysis tools

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cim-graph = "0.1.0"

# Optional features
cim-graph = { version = "0.1.0", features = ["async"] }
```

## Quick Start

### Basic Graph Operations

```rust
use cim_graph::{GraphBuilder, Node, Edge, Result};

fn main() -> Result<()> {
    // Create a simple graph
    let mut graph = GraphBuilder::new()
        .with_capacity(100, 200)
        .build();
    
    // Add nodes
    let node1 = graph.add_node(Node::new("Alice", "Person"))?;
    let node2 = graph.add_node(Node::new("Bob", "Person"))?;
    
    // Connect nodes
    graph.add_edge(node1, node2, Edge::new("knows"))?;
    
    // Query the graph
    let neighbors = graph.neighbors(node1)?;
    println!("Alice knows: {:?}", neighbors);
    
    Ok(())
}
```

### Domain-Specific Graphs

```rust
use cim_graph::graphs::{IpldGraph, ContextGraph, WorkflowGraph, ConceptGraph};

// IPLD Graph for content-addressed data
let mut ipld = IpldGraph::new();
let cid1 = ipld.add_cid("QmHash1...")?;
let cid2 = ipld.add_cid("QmHash2...")?;
ipld.add_link(cid1, cid2, "contains")?;

// Context Graph for domain modeling
let mut context = ContextGraph::new();
let user = context.add_aggregate("User", user_id)?;
let order = context.add_aggregate("Order", order_id)?;
context.add_relationship(user, order, "placed")?;

// Workflow Graph for state machines
let mut workflow = WorkflowGraph::new();
let draft = workflow.add_state("Draft")?;
let published = workflow.add_state("Published")?;
workflow.add_transition(draft, published, "publish")?;

// Concept Graph for semantic reasoning
let mut concepts = ConceptGraph::new();
let vehicle = concepts.add_concept("Vehicle")?;
let car = concepts.add_concept("Car")?;
concepts.add_relation(car, vehicle, "is_a")?;
```

### Graph Composition

```rust
use cim_graph::{compose_graphs, CompositionStrategy};

// Compose multiple graphs
let composed = compose_graphs()
    .add_graph("ipld", ipld_graph)
    .add_graph("context", context_graph)
    .add_graph("workflow", workflow_graph)
    .with_strategy(CompositionStrategy::PreserveAll)
    .with_mapping(|node| {
        // Define how nodes map between graphs
        match node.graph_type() {
            "ipld" => Some(MappingRule::ByProperty("cid")),
            "context" => Some(MappingRule::ByProperty("aggregate_id")),
            _ => None,
        }
    })
    .compose()?;

// Query across composed graphs
let results = composed
    .query()
    .start_from("context", user_id)
    .traverse_to("workflow")
    .where_property("state", "active")
    .execute()?;
```

### Event Handling

```rust
use cim_graph::{EventGraph, GraphEvent, EventHandler};

// Create an event-aware graph
let mut graph = EventGraph::new();

// Subscribe to events
graph.subscribe(|event: &GraphEvent| {
    match event {
        GraphEvent::NodeAdded { id, data, .. } => {
            println!("Node {} added with data: {:?}", id, data);
        }
        GraphEvent::EdgeAdded { from, to, .. } => {
            println!("Edge added: {} -> {}", from, to);
        }
        _ => {}
    }
});

// All operations emit events
let node = graph.add_node(Node::new("data", "type"))?;
```

### Serialization

```rust
use cim_graph::serde_support::{to_json, from_json, to_nix};

// Serialize to JSON
let json = to_json(&graph)?;
std::fs::write("graph.json", json)?;

// Deserialize from JSON
let loaded: IpldGraph = from_json(&json)?;

// Export to Nix expression
let nix_expr = to_nix(&graph)?;
std::fs::write("graph.nix", nix_expr)?;
```

## Documentation

- [API Documentation](https://docs.rs/cim-graph) - Complete API reference
- [Architecture Guide](docs/architecture.md) - System design and internals
- [Graph Types Guide](docs/graph-types.md) - Detailed guide for each graph type
- [Algorithms Guide](docs/algorithms.md) - Available algorithms and usage
- [Best Practices](docs/best-practices.md) - Recommended patterns and tips
- [Examples](examples/) - Runnable example code

## Performance

CIM Graph is designed for high performance:

- Zero-cost abstractions over petgraph
- Optimized memory layout for cache efficiency
- Parallel algorithms where applicable
- Benchmarks available in `benches/`

Run benchmarks with:
```bash
cargo bench
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Built on top of the excellent [petgraph](https://github.com/petgraph/petgraph) library.