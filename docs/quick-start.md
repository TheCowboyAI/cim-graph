# Quick Start Guide

Get up and running with CIM Graph in 5 minutes!

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
cim-graph = "0.1.0"
```

## Your First Graph

Let's create a simple workflow graph:

```rust
use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::error::Result;

fn main() -> Result<()> {
    // Create a new workflow graph
    let mut workflow = WorkflowGraph::new();
    
    // Add some states
    workflow.add_state(WorkflowNode::new(
        "draft",
        "Draft",
        StateType::Start
    ))?;
    
    workflow.add_state(WorkflowNode::new(
        "review",
        "Under Review",
        StateType::Normal
    ))?;
    
    workflow.add_state(WorkflowNode::new(
        "published",
        "Published",
        StateType::End
    ))?;
    
    // Add transitions
    workflow.add_transition("draft", "review", "submit")?;
    workflow.add_transition("review", "published", "approve")?;
    workflow.add_transition("review", "draft", "reject")?;
    
    // Start the workflow
    workflow.start("document_123")?;
    
    // Process events
    workflow.process_event("submit")?;
    println!("Current state: {:?}", workflow.current_state());
    
    Ok(())
}
```

## Different Graph Types

### IPLD Graph (Content-Addressed)

```rust
use cim_graph::graphs::ipld::IpldGraph;

fn ipld_example() -> Result<()> {
    let mut graph = IpldGraph::new();
    
    // Add content blocks
    let block1 = graph.add_block(
        "block1".to_string(),
        b"Hello, World!".to_vec()
    )?;
    
    let block2 = graph.add_block(
        "block2".to_string(),
        b"Content addressing".to_vec()
    )?;
    
    // Link blocks
    graph.add_link("block1", "block2", "reference")?;
    
    Ok(())
}
```

### Context Graph (Domain Modeling)

```rust
use cim_graph::graphs::context::{ContextGraph, ContextNode};

fn context_example() -> Result<()> {
    let mut graph = ContextGraph::new();
    
    // Add domain entities
    let customer = ContextNode::new(
        "customer_1",
        "John Doe",
        "Customer"
    );
    graph.add_context(customer)?;
    
    let order = ContextNode::new(
        "order_1",
        "Order #12345",
        "Order"
    );
    graph.add_context(order)?;
    
    // Add relationships
    graph.add_relationship("customer_1", "order_1", "placed")?;
    
    Ok(())
}
```

### Concept Graph (Knowledge Representation)

```rust
use cim_graph::graphs::concept::{ConceptGraph, ConceptNode, SemanticRelation};

fn concept_example() -> Result<()> {
    let mut graph = ConceptGraph::new();
    
    // Add concepts
    graph.add_concept(ConceptNode::new(
        "animal",
        "Animal",
        "Living organism"
    ))?;
    
    graph.add_concept(ConceptNode::new(
        "dog",
        "Dog",
        "Domestic canine"
    ))?;
    
    // Add semantic relation
    graph.add_relation("dog", "animal", SemanticRelation::IsA)?;
    
    Ok(())
}
```

## Graph Operations

### Adding Nodes and Edges

```rust
// Generic pattern for all graph types
let mut graph = WorkflowGraph::new();

// Add nodes
graph.add_state(node)?;

// Add edges
graph.add_transition(from, to, label)?;

// Query
let node = graph.get_state("node_id");
let neighbors = graph.graph().neighbors("node_id")?;
```

### Using Algorithms

```rust
use cim_graph::algorithms;

// Find shortest path
let path = algorithms::shortest_path(
    graph.graph(),
    "start",
    "end"
)?;

// Traverse graph
let visited = algorithms::bfs(graph.graph(), "start")?;

// Calculate metrics
let centrality = algorithms::centrality(graph.graph())?;
```

### Serialization

```rust
use cim_graph::serde_support::GraphSerialize;

// Save to JSON
let json = graph.to_json()?;
std::fs::write("graph.json", json)?;

// Load from JSON
let json = std::fs::read_to_string("graph.json")?;
let graph = WorkflowGraph::from_json(&json)?;
```

## Event Handling

```rust
use cim_graph::core::{EventHandler, GraphEvent};

struct MyHandler;

impl EventHandler for MyHandler {
    fn handle_event(&self, event: &GraphEvent) {
        match event {
            GraphEvent::NodeAdded { id, .. } => {
                println!("Node {} added", id);
            }
            GraphEvent::EdgeAdded { from, to, .. } => {
                println!("Edge {} -> {} added", from, to);
            }
            _ => {}
        }
    }
}

// Register handler
graph.graph_mut().add_event_handler(Box::new(MyHandler));
```

## Error Handling

CIM Graph uses the `Result` type for fallible operations:

```rust
use cim_graph::error::{GraphError, Result};

fn safe_operation() -> Result<()> {
    let mut graph = WorkflowGraph::new();
    
    // Handle specific errors
    match graph.add_state(node) {
        Ok(id) => println!("Added node: {}", id),
        Err(GraphError::DuplicateNode(id)) => {
            println!("Node {} already exists", id);
        }
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

## Best Practices

1. **Choose the right graph type** for your use case
2. **Pre-allocate capacity** for better performance:
   ```rust
   let graph = GraphBuilder::new()
       .with_capacity(1000, 5000)
       .build();
   ```

3. **Use bulk operations** when possible:
   ```rust
   graph.add_nodes_bulk(nodes)?;
   ```

4. **Handle events** for audit trails and reactive systems

5. **Serialize graphs** for persistence and debugging

## Next Steps

- Explore the [Examples](https://github.com/thecowboyai/cim-graph/tree/main/examples)
- Read about [Graph Types](./graph-types.md) in detail
- Learn about [Algorithms](./algorithms.md)
- Check out [Best Practices](./best-practices.md)

## Getting Help

- [API Documentation](https://docs.rs/cim-graph)
- [GitHub Issues](https://github.com/thecowboyai/cim-graph/issues)
- [Discussions](https://github.com/thecowboyai/cim-graph/discussions)