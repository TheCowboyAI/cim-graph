# Event Design Best Practices

This guide provides best practices for designing events in the CIM Graph event-driven architecture.

## Core Principles

### 1. Events are Immutable Facts
- Events represent something that **has happened** in the past
- Never modify an event after creation
- Use past tense for event names: `NodeAdded`, `EdgeRemoved`, `WorkflowCompleted`

### 2. Events are the Single Source of Truth
- All state changes MUST go through events
- Current state is derived by replaying events
- Projections are disposable and can be rebuilt

## Event Design Guidelines

### Event Naming

✅ **DO:**
```rust
// Past tense, descriptive
EventPayload::Workflow(WorkflowPayload::StateTransitioned { ... })
EventPayload::Ipld(IpldPayload::CidAdded { ... })
EventPayload::Context(ContextPayload::EntityAdded { ... })
```

❌ **DON'T:**
```rust
// Present tense, commands
EventPayload::Generic("AddNode")      // This is a command, not an event
EventPayload::Generic("TransitionState") // Should be StateTransitioned
```

### Event Granularity

Events should be fine-grained and focused on a single change:

✅ **Good - Single Responsibility:**
```rust
// Each event does one thing
vec![
    GraphEvent {
        payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined { ... }),
    },
    GraphEvent {
        payload: EventPayload::Workflow(WorkflowPayload::StateAdded { ... }),
    },
    GraphEvent {
        payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded { ... }),
    },
]
```

❌ **Bad - Too Coarse:**
```rust
// Trying to do too much in one event
GraphEvent {
    payload: EventPayload::Generic(GenericPayload {
        event_type: "WorkflowCreatedWithStatesAndTransitions",
        data: // Complex nested structure
    }),
}
```

### Event Payloads

Design payloads to be self-contained and descriptive:

✅ **Good Payload Design:**
```rust
EventPayload::Ipld(IpldPayload::CidLinkAdded {
    cid: "QmSource".to_string(),        // Clear field names
    link_name: "references".to_string(), // Descriptive link type
    target_cid: "QmTarget".to_string(),  // Explicit relationship
})
```

❌ **Bad Payload Design:**
```rust
EventPayload::Generic(GenericPayload {
    event_type: "LinkAdded",
    data: json!({
        "from": "id1",    // Ambiguous field names
        "to": "id2",      // No type information
        "type": "link"    // Generic type
    }),
})
```

## Correlation and Causation

### Correlation IDs
Group related events that are part of the same business transaction:

```rust
let correlation_id = Uuid::new_v4(); // One per business operation

// All events in an order processing flow share the same correlation_id
let order_created = GraphEvent {
    correlation_id,
    payload: EventPayload::Context(ContextPayload::EntityAdded {
        entity_type: "Order",
        // ...
    }),
};

let payment_processed = GraphEvent {
    correlation_id, // Same correlation_id
    causation_id: Some(order_created.event_id),
    // ...
};
```

### Causation IDs
Link events that directly cause other events:

```rust
// Event chain with proper causation
let mut events = Vec::new();
let mut last_event_id = None;

for i in 0..5 {
    let event = GraphEvent {
        event_id: Uuid::new_v4(),
        causation_id: last_event_id, // Links to previous event
        // ...
    };
    last_event_id = Some(event.event_id);
    events.push(event);
}
```

## Domain-Specific Event Types

### Use Type-Safe Payloads
Prefer strongly-typed payloads over generic ones:

✅ **Type-Safe:**
```rust
match event.payload {
    EventPayload::Workflow(workflow) => match workflow {
        WorkflowPayload::StateTransitioned { instance_id, from_state, to_state } => {
            // Compiler ensures all fields exist
        }
        // ...
    }
    // ...
}
```

❌ **Stringly-Typed:**
```rust
match event.payload {
    EventPayload::Generic(generic) => {
        // Runtime errors possible
        let from = generic.data["from_state"].as_str().unwrap();
        let to = generic.data["to_state"].as_str().unwrap();
    }
    // ...
}
```

## Event Ordering

### Sequence Numbers
Always track event order explicitly:

```rust
let events_with_sequence: Vec<(GraphEvent, u64)> = events
    .into_iter()
    .enumerate()
    .map(|(i, event)| (event, (i + 1) as u64))
    .collect();

let projection = build_projection(events_with_sequence);
```

### Handling Concurrent Events
When events happen concurrently, use correlation IDs to group them:

```rust
// User actions stream
let user_events = create_events_with_correlation(user_correlation_id);

// System reactions stream  
let system_events = create_events_with_correlation(system_correlation_id);

// Merge and sort by timestamp or sequence
let all_events = merge_event_streams(vec![user_events, system_events]);
```

## Event Storage

### Serialization
Events must be serializable for persistence:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEvent {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub payload: EventPayload,
}

// Save events
let journal = EventJournal::new(events);
journal.save_to_file("events.json")?;

// Load events
let journal = EventJournal::load_from_file("events.json")?;
```

### Event Metadata
Include sufficient metadata for debugging and auditing:

```rust
EventPayload::Generic(GenericPayload {
    event_type: "UserAction",
    data: json!({
        "action": "clicked_button",
        "timestamp": Utc::now().to_rfc3339(),
        "user_id": user_id,
        "session_id": session_id,
        "client_version": "1.2.3",
    }),
})
```

## Testing Events

### Property-Based Testing
Test event sourcing invariants:

```rust
proptest! {
    #[test]
    fn projection_version_equals_event_count(num_events in 1..100) {
        let events = generate_events(num_events);
        let projection = build_projection(events);
        prop_assert_eq!(projection.version, num_events as u64);
    }
}
```

### Event Replay Testing
Ensure projections are deterministic:

```rust
#[test]
fn test_replay_produces_same_result() {
    let events = create_test_events();
    
    let projection1 = build_projection(events.clone());
    let projection2 = build_projection(events);
    
    assert_eq!(projection1, projection2);
}
```

## Anti-Patterns to Avoid

### 1. Mutable Events
❌ **Never modify events after creation:**
```rust
// WRONG
event.payload = new_payload; // Events are immutable!
```

### 2. Direct State Mutation
❌ **Never bypass the event system:**
```rust
// WRONG
projection.nodes.insert(node_id, node); // Must go through events!
```

### 3. Fat Events
❌ **Avoid events that do too much:**
```rust
// WRONG
EventPayload::Generic("EverythingChanged") // Too vague and coarse
```

### 4. Present Tense Events
❌ **Events describe the past, not intentions:**
```rust
// WRONG
EventPayload::Generic("AddNode")    // This is a command
// RIGHT
EventPayload::Generic("NodeAdded")  // This is an event
```

## Example: Well-Designed Event Flow

```rust
use cim_graph::events::*;
use uuid::Uuid;

fn create_order_processing_events() -> Vec<(GraphEvent, u64)> {
    let aggregate_id = Uuid::new_v4();
    let correlation_id = Uuid::new_v4();
    let mut events = Vec::new();
    let mut sequence = 1;
    
    // 1. Order created
    let order_created = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: None,
        payload: EventPayload::Context(ContextPayload::EntityAdded {
            aggregate_id,
            entity_id: Uuid::new_v4(),
            entity_type: "Order".to_string(),
            properties: json!({
                "customer_id": "CUST-123",
                "total": 99.99,
                "items": 3
            }),
        }),
    };
    events.push((order_created.clone(), sequence));
    sequence += 1;
    
    // 2. Payment initiated (caused by order creation)
    let payment_initiated = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id, // Same business transaction
        causation_id: Some(order_created.event_id), // Caused by order
        payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
            instance_id: aggregate_id,
            from_state: "order_placed".to_string(),
            to_state: "payment_pending".to_string(),
        }),
    };
    events.push((payment_initiated.clone(), sequence));
    sequence += 1;
    
    // 3. Payment completed
    let payment_completed = GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id,
        correlation_id,
        causation_id: Some(payment_initiated.event_id),
        payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
            instance_id: aggregate_id,
            from_state: "payment_pending".to_string(),
            to_state: "payment_completed".to_string(),
        }),
    };
    events.push((payment_completed, sequence));
    
    events
}
```

## Summary

1. **Events are immutable facts** - They represent what happened, not what should happen
2. **Use past tense** - `NodeAdded`, not `AddNode`
3. **Be specific** - Use typed payloads over generic ones
4. **Track relationships** - Use correlation and causation IDs
5. **Test thoroughly** - Use property-based testing for invariants
6. **Keep events focused** - One event, one change
7. **Include metadata** - Timestamps, user info, versions

Following these practices will lead to a more maintainable, debuggable, and scalable event-driven system.