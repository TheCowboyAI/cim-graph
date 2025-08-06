# Migration Guide: From Mutable API to Event-Driven Architecture

This guide helps you migrate from the old mutable graph API to the new event-driven architecture.

## Overview of Changes

The fundamental change is that **all state modifications must now go through events**. Direct mutations like `graph.add_node()` or `graph.add_edge()` are no longer supported.

### Old API (No Longer Supported)
```rust
// Direct mutations
let mut graph = EventGraph::new(GraphType::Generic);
let node_id = graph.add_node(node)?;
let edge_id = graph.add_edge(edge)?;
```

### New API (Event-Driven)
```rust
// Create events
let event = GraphEvent {
    event_id: Uuid::new_v4(),
    aggregate_id,
    correlation_id: Uuid::new_v4(),
    causation_id: None,
    payload: EventPayload::Generic(GenericPayload {
        event_type: "NodeAdded".to_string(),
        data: serde_json::json!({ "node_id": "n1" }),
    }),
};

// Build projection from events
let projection = build_projection(vec![(event, 1)]);
```

## Step-by-Step Migration

### 1. Replace Direct Graph Creation

**Before:**
```rust
use cim_graph::core::{EventGraph, GraphType, GenericNode, GenericEdge};

let mut graph = EventGraph::<GenericNode<String>, GenericEdge<String>>::new(GraphType::Generic);
```

**After:**
```rust
use cim_graph::core::{GraphAggregateProjection, build_projection};
use cim_graph::events::{GraphEvent, EventPayload, GenericPayload};
use uuid::Uuid;

let aggregate_id = Uuid::new_v4();
let mut events = Vec::new();
```

### 2. Replace Node Operations

**Before:**
```rust
let node = GenericNode::new("n1".to_string(), "Node 1".to_string());
let node_id = graph.add_node(node)?;
```

**After:**
```rust
let event = GraphEvent {
    event_id: Uuid::new_v4(),
    aggregate_id,
    correlation_id: Uuid::new_v4(),
    causation_id: None,
    payload: EventPayload::Generic(GenericPayload {
        event_type: "NodeAdded".to_string(),
        data: serde_json::json!({
            "node_id": "n1",
            "label": "Node 1"
        }),
    }),
};
events.push((event, events.len() as u64 + 1));
```

### 3. Replace Edge Operations

**Before:**
```rust
let edge = GenericEdge::new("e1".to_string(), "n1".to_string(), "n2".to_string());
let edge_id = graph.add_edge(edge)?;
```

**After:**
```rust
let event = GraphEvent {
    event_id: Uuid::new_v4(),
    aggregate_id,
    correlation_id: Uuid::new_v4(),
    causation_id: Some(events.last().unwrap().0.event_id), // Link to previous event
    payload: EventPayload::Generic(GenericPayload {
        event_type: "EdgeAdded".to_string(),
        data: serde_json::json!({
            "edge_id": "e1",
            "source": "n1",
            "target": "n2"
        }),
    }),
};
events.push((event, events.len() as u64 + 1));
```

### 4. Query Operations

**Before:**
```rust
let node = graph.get_node("n1")?;
let neighbors = graph.neighbors("n1")?;
```

**After:**
```rust
// Build projection from events
let projection = build_projection(events);

// Query the projection
let component = projection.get_component("n1");
let relationships = projection.relationships_for("n1");
```

### 5. Using Specialized Graph Types

Each graph type now has its own event payload:

```rust
// Workflow events
EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
    workflow_id: Uuid::new_v4(),
    name: "My Workflow".to_string(),
    version: "1.0.0".to_string(),
})

// IPLD events
EventPayload::Ipld(IpldPayload::CidAdded {
    cid: "QmXxx...".to_string(),
    codec: "dag-cbor".to_string(),
    size: 256,
    data: serde_json::json!({}),
})

// Context events
EventPayload::Context(ContextPayload::BoundedContextCreated {
    context_id: "order_management".to_string(),
    name: "Order Management".to_string(),
    description: "Handles orders".to_string(),
})
```

## Common Patterns

### Pattern 1: Building a Complete Graph

```rust
use cim_graph::core::{GraphAggregateProjection, build_projection};
use cim_graph::events::{GraphEvent, EventPayload, GenericPayload};

fn create_graph() -> GraphAggregateProjection {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let mut events = Vec::new();
    
    // Add nodes
    for i in 0..3 {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: events.last().map(|(e, _): &(GraphEvent, u64)| e.event_id),
            payload: EventPayload::Generic(GenericPayload {
                event_type: "NodeAdded".to_string(),
                data: serde_json::json!({
                    "node_id": format!("n{}", i),
                    "label": format!("Node {}", i)
                }),
            }),
        };
        events.push((event, events.len() as u64 + 1));
    }
    
    // Add edges
    let edges = vec![("n0", "n1"), ("n1", "n2")];
    for (i, (source, target)) in edges.iter().enumerate() {
        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id,
            causation_id: Some(events.last().unwrap().0.event_id),
            payload: EventPayload::Generic(GenericPayload {
                event_type: "EdgeAdded".to_string(),
                data: serde_json::json!({
                    "edge_id": format!("e{}", i),
                    "source": source,
                    "target": target
                }),
            }),
        };
        events.push((event, events.len() as u64 + 1));
    }
    
    build_projection(events)
}
```

### Pattern 2: Using Policy Engine

```rust
use cim_graph::core::{PolicyEngine, PolicyContext, GraphStateMachine};

let mut policy_engine = PolicyEngine::new();
let mut state_machine = GraphStateMachine::new();
let mut ipld_chains = HashMap::new();

let mut context = PolicyContext {
    state_machine: &mut state_machine,
    ipld_chains: &mut ipld_chains,
    metrics: Default::default(),
};

// Process event through policies
let actions = policy_engine.execute_policies(&event, &mut context)?;
```

### Pattern 3: Persisting Events

```rust
use cim_graph::serde_support::EventJournal;

// Save events
let journal = EventJournal::new(events);
journal.save_to_file("events.json")?;

// Load events
let journal = EventJournal::load_from_file("events.json")?;
let projection = build_projection(
    journal.events.into_iter()
        .enumerate()
        .map(|(i, e)| (e, i as u64 + 1))
        .collect()
);
```

## Benefits of the New Architecture

1. **Complete Audit Trail**: Every change is recorded as an event
2. **Time Travel**: Replay events to any point in time
3. **Event Sourcing**: Natural fit for distributed systems
4. **Immutability**: Projections are read-only, ensuring consistency
5. **Correlation Tracking**: Understand causation chains
6. **Policy Automation**: React to events with custom logic

## Need Help?

Check out the examples in the `examples/` directory for complete working code:
- `basic_event_driven.rs` - Simple introduction
- `complete_event_driven.rs` - All features demonstrated
- `order_processing_system.rs` - Real-world example

For questions, please open an issue on GitHub.