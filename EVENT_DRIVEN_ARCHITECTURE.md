# Event-Driven Architecture Guide

## Overview

CIM Graph has been completely refactored to use a pure event-driven architecture. This means:

- **Events are the ONLY way to change state**
- **Projections are ephemeral read models** rebuilt from events
- **IPLD provides content-addressed storage** for all event payloads
- **NATS JetStream handles event persistence** and streaming
- **State machines control valid transitions**
- **Policies provide automated behaviors**

## Core Concepts

### 1. Events Are Truth

All state changes MUST go through events. There are no direct mutations.

```rust
// ❌ OLD WAY - Direct mutations (no longer supported)
graph.add_node(node);
graph.add_edge(edge);

// ✅ NEW WAY - Events only
let event = GraphEvent {
    event_id: Uuid::new_v4(),
    aggregate_id: workflow_id,
    correlation_id: Uuid::new_v4(),
    causation_id: None,
    payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
        workflow_id,
        state_id: "processing".to_string(),
        state_type: "normal".to_string(),
    }),
};
```

### 2. Projections Are Ephemeral

Projections are read-only views computed from events. They are NEVER persisted.

```rust
// Build projection from events
let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
let projection = engine.project(events);

// Query the projection (read-only)
let node_count = projection.node_count();
let version = projection.version();
```

### 3. Event Flow

```
Command → State Machine → Event → IPLD → NATS → Projection
                             ↓
                          Policies
                             ↓
                     Additional Events
```

### 4. Bounded Contexts

The system is organized into 5 bounded contexts:

1. **IPLD Context** - Content-addressed storage
2. **Context Context** - Data schemas and transformations
3. **Workflow Context** - State machines and processes
4. **Concept Context** - Domain knowledge and reasoning
5. **Composed Context** - Multi-graph orchestration

## Working with Events

### Creating Events

```rust
use cim_graph::events::{GraphEvent, EventPayload, WorkflowPayload};

let event = GraphEvent {
    event_id: Uuid::new_v4(),
    aggregate_id: workflow_id,
    correlation_id: correlation_id,  // Links related events
    causation_id: Some(previous_event_id),  // What caused this event
    payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
        workflow_id,
        name: "Order Processing".to_string(),
        version: "1.0.0".to_string(),
    }),
};
```

### Event Types

Each bounded context has its own event types:

- **WorkflowPayload**: WorkflowDefined, StateAdded, TransitionAdded, StateTransitioned, etc.
- **ConceptPayload**: ConceptDefined, RelationAdded, PropertyAdded, InferenceRuleAdded, etc.
- **ContextPayload**: ContextDefined, AggregateAdded, EntityAdded, SchemaAdded, etc.
- **IpldPayload**: CidAdded, CidLinked, ChainValidated, CidPinned, etc.
- **ComposedPayload**: SubGraphAdded, CrossGraphLinkCreated, CompositionRuleAdded, etc.

### Correlation and Causation

- **correlation_id**: Groups related events across aggregates
- **causation_id**: Points to the event that caused this one

```rust
// Initial event
let create_order = GraphEvent {
    event_id: Uuid::new_v4(),
    correlation_id: Uuid::new_v4(),  // New correlation
    causation_id: None,  // No cause - this starts the chain
    // ...
};

// Caused event
let process_payment = GraphEvent {
    event_id: Uuid::new_v4(),
    correlation_id: create_order.correlation_id,  // Same correlation
    causation_id: Some(create_order.event_id),  // Caused by create_order
    // ...
};
```

## Building Projections

### Basic Projection

```rust
use cim_graph::core::ProjectionEngine;
use cim_graph::graphs::{WorkflowNode, WorkflowEdge};

// Create projection engine
let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();

// Build projection from events
let projection = engine.project(events);

// Query projection (read-only)
assert_eq!(projection.node_count(), 5);
assert_eq!(projection.edge_count(), 4);
```

### Custom Projections

You can create custom projections by implementing the `GraphProjection` trait:

```rust
use cim_graph::core::GraphProjection;

struct MyCustomProjection {
    // Your projection state
}

impl GraphProjection for MyCustomProjection {
    fn node_count(&self) -> usize { /* ... */ }
    fn edge_count(&self) -> usize { /* ... */ }
    fn version(&self) -> u64 { /* ... */ }
    fn has_node(&self, id: &str) -> bool { /* ... */ }
    fn has_edge(&self, from: &str, to: &str) -> bool { /* ... */ }
}
```

## State Machines

State machines validate transitions and enforce business rules:

```rust
use cim_graph::core::GraphStateMachine;

let mut state_machine = GraphStateMachine::new();

// State machine validates commands before creating events
let command = GraphCommand::CreateGraph {
    aggregate_id: Uuid::new_v4(),
    graph_type: "workflow".to_string(),
    metadata: HashMap::new(),
};

let events = state_machine.handle_command(command, &projection)?;
```

## Policies

Policies provide automated behaviors that react to events:

```rust
use cim_graph::core::{PolicyEngine, CidGenerationPolicy, StateValidationPolicy};

let mut policy_engine = PolicyEngine::new();
policy_engine.add_policy(Box::new(CidGenerationPolicy));
policy_engine.add_policy(Box::new(StateValidationPolicy));

// Policies generate additional events
let mut context = PolicyContext {
    state_machine: &mut state_machine,
    ipld_chains: &mut ipld_chains,
    metrics: Default::default(),
};

let actions = policy_engine.evaluate(&event, &mut context)?;
```

## NATS JetStream Integration

Events are persisted to NATS JetStream for durability:

```rust
#[cfg(feature = "nats")]
use cim_graph::nats::JetStreamEventStore;

// Connect to NATS
let store = JetStreamEventStore::new("nats://localhost:4222").await?;

// Publish events
store.publish_events(&events).await?;

// Subscribe to events
let subscription = store.subscribe("cim.graph.workflow.*").await?;
while let Some(event) = subscription.next().await {
    // Process event
}
```

## Event Storage

### File-Based Storage

```rust
use cim_graph::serde_support::{EventJournal, save_events_to_file};

// Save events
let journal = EventJournal::new(events);
journal.save_to_file("events.json")?;

// Load events
let journal = EventJournal::load_from_file("events.json")?;
let events = journal.events;
```

### IPLD Chain Storage

All event payloads get CIDs for content addressing:

```rust
use cim_graph::core::IpldChainAggregate;

let mut chain = IpldChainAggregate::new(aggregate_id);
chain.add_event(event)?;

// Each event gets a CID
let cid = chain.get_cid_for_event(&event_id)?;
```

## Migration Guide

### From Mutable API to Events

**Old way:**
```rust
let mut graph = WorkflowGraph::new();
graph.add_state(state)?;
graph.add_transition(from, to, trigger)?;
```

**New way:**
```rust
let events = vec![
    GraphEvent {
        payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
            workflow_id,
            state_id: "processing".to_string(),
            state_type: "normal".to_string(),
        }),
        // ...
    },
    GraphEvent {
        payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
            workflow_id,
            from_state: "submitted".to_string(),
            to_state: "processing".to_string(),
            trigger: "approve".to_string(),
        }),
        // ...
    },
];

let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
let projection = engine.project(events);
```

## Best Practices

1. **Always use events** - Never try to mutate state directly
2. **Track causation** - Set causation_id to maintain event chains
3. **Use correlation** - Group related events with correlation_id
4. **Projections are temporary** - Rebuild from events, don't persist
5. **Let policies work** - They automate CID generation, validation, etc.
6. **Test with events** - Write tests that create events and verify projections

## Examples

See the `examples/` directory for complete examples:

- `complete_event_driven.rs` - Demonstrates all concepts
- `order_processing_system.rs` - Real-world order processing
- `nats_event_driven.rs` - NATS JetStream integration

## Troubleshooting

### "No method named add_node"
You're trying to use the old mutable API. Use events instead.

### "Projection is read-only"
Correct! Create events to change state, then rebuild the projection.

### "Events not persisted"
Make sure to save events using EventJournal or NATS JetStream.

### "State machine rejected command"
The command violates state machine rules. Check valid transitions.