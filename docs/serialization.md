# Event Serialization

## Overview

CIM Graph uses event sourcing, where all state changes are captured as events. Only events should be persisted - projections are ephemeral and rebuilt from events.

## What Gets Serialized

### Events ✅
- `GraphEvent` - The primary unit of state change
- `GraphCommand` - Commands that generate events
- Event metadata (correlation, causation, etc.)

### What Does NOT Get Serialized ❌
- Projections (WorkflowProjection, ConceptProjection, etc.)
- Graph state (nodes, edges)
- Computed views

## Why Projections Are Not Serialized

1. **Single Source of Truth**: Events are the authoritative record
2. **Schema Evolution**: Projection logic can change without migrating data
3. **Performance**: Projections can be optimized differently over time
4. **Consistency**: Ensures projections always reflect the latest business logic

## Event Serialization API

```rust
use cim_graph::serde_support::{serialize_events, deserialize_events};
use cim_graph::events::{GraphEvent, EventPayload};

// Serialize events
let events: Vec<GraphEvent> = vec![...];
let json = serialize_events(&events)?;

// Deserialize events
let restored = deserialize_events(&json)?;
```

## Event Journal

For persistent storage of event streams:

```rust
use cim_graph::serde_support::EventJournal;

// Create journal from events
let journal = EventJournal::new(events);

// Save to file
journal.save_to_file("events.json")?;

// Load from file
let loaded = EventJournal::load_from_file("events.json")?;
```

## Event Storage Patterns

### 1. File-Based Storage
```rust
use cim_graph::serde_support::save_events_to_file;

// Save events to file
save_events_to_file(&events, "aggregate-123.json")?;

// Load events from file
let events = load_events_from_file("aggregate-123.json")?;
```

### 2. NATS JetStream
```rust
use cim_graph::nats::JetStreamEventStore;

// Publish events to NATS
let store = JetStreamEventStore::new("nats://localhost:4222").await?;
store.publish_events(&events).await?;

// Subscribe to events
let subscription = store.subscribe("workflow.*").await?;
```

### 3. Database Storage
Events can be stored in any database that supports JSON:
- PostgreSQL with JSONB columns
- MongoDB
- EventStore
- Kafka

## Rebuilding Projections

```rust
use cim_graph::core::ProjectionEngine;
use cim_graph::graphs::{WorkflowNode, WorkflowEdge};

// Load events from storage
let events = load_events_from_file("workflow-123.json")?;

// Rebuild projection
let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
let projection = engine.project(events);

// Use the projection
assert_eq!(projection.node_count(), 10);
```

## Best Practices

1. **Event Versioning**: Include version info in event payloads
2. **Event Compression**: Use compression for large event stores
3. **Event Partitioning**: Partition events by aggregate ID
4. **Event Snapshots**: For long event streams, consider snapshots
5. **Event Replay**: Design projections to handle replay from any point

## Migration from Old Serialization

If you have serialized graphs from the old system:

1. Load the old graph format
2. Generate events that would create that state
3. Save the events using the new system
4. Discard the old serialized format

This ensures all data is migrated to the event-sourced model.