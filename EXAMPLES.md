# CIM Graph Examples

This document lists all the working examples that demonstrate the event-driven architecture.

## Working Examples

### 1. basic_event_driven.rs
Basic introduction to the event-driven architecture.

```bash
cargo run --example basic_event_driven
```

Features shown:
- Creating events with GraphEvent API
- Building projections from event streams  
- Event causation tracking
- Serializing events for persistence

### 2. workflow_event_driven.rs
Complete workflow example with event-driven patterns.

```bash
cargo run --example workflow_event_driven
```

Features shown:
- Creating workflow definitions through events
- Adding states and transitions
- Policy engine for automated behaviors
- Event causation chains
- Bounded contexts
- Saving events to JSON files

### 3. complete_event_driven.rs
Comprehensive example showing all graph types.

```bash
cargo run --example complete_event_driven
```

Features shown:
- Multiple graph types (workflow, concept, context, composed)
- Policy engine with CID generation
- State machine validation
- IPLD chain storage
- Complex event relationships
- Aggregate projections

### 4. order_processing_system.rs
Real-world order processing system example.

```bash
cargo run --example order_processing_system
```

Features shown:
- Building a complete order system with events
- Domain concepts and workflows
- Bounded contexts (Order Management)
- Event-driven state transitions
- Business process modeling

### 5. nats_event_driven.rs
NATS JetStream integration for event persistence.

```bash
# Requires NATS server running at localhost:4222
cargo run --example nats_event_driven --features nats
```

Features shown:
- Publishing events to NATS JetStream
- Building projections from event streams
- Real-time event subscriptions
- IPLD chain integrity verification
- Subject hierarchy patterns

### 6. simple_workflow.rs
Demonstrates the core event-driven workflow concepts using the low-level CimGraphEvent API.

```bash
cargo run --example simple_workflow
```

Features shown:
- Creating workflow events with CimGraphEvent
- Building projections from event streams
- Event causation tracking
- NATS subject patterns

### 7. event_driven_simple.rs
Shows the high-level event API without complex dependencies.

```bash
cargo run --example event_driven_simple
```

Features shown:
- Creating workflow and concept events
- Bounded contexts and their relationships
- Event correlation and serialization
- Saving events to JSON files

### 8. pure_event_driven.rs
Demonstrates pure event-driven patterns.

```bash
cargo run --example pure_event_driven
```

### 9. collaborative_graph.rs
Shows collaborative graph operations through events.

```bash
cargo run --example collaborative_graph
```

## Key Concepts Demonstrated

1. **Events are the only source of truth** - No direct mutations allowed
2. **Projections are ephemeral** - Rebuilt from events on demand
3. **Event correlation** - Group related events with correlation IDs
4. **Event causation** - Track what caused each event
5. **Bounded contexts** - Clear domain boundaries following DDD
6. **Event persistence** - Save as JSON or to NATS JetStream
7. **State machines** - Enforce valid state transitions
8. **Policies** - Automate behaviors like CID generation
9. **IPLD chains** - Content-addressed event storage

## Running All Examples

```bash
# Run all examples (except NATS which requires a server)
for example in basic_event_driven workflow_event_driven complete_event_driven order_processing_system simple_workflow event_driven_simple pure_event_driven collaborative_graph; do
    echo "=== Running $example ==="
    cargo run --example $example
    echo
done

# For NATS example (requires NATS server at localhost:4222)
cargo run --example nats_event_driven --features nats
```

## Example Categories

### Getting Started
- `basic_event_driven.rs` - Start here for the basics
- `workflow_event_driven.rs` - Learn about workflows

### Advanced Concepts  
- `complete_event_driven.rs` - All graph types together
- `order_processing_system.rs` - Real-world application

### Integration
- `nats_event_driven.rs` - NATS JetStream persistence

### Legacy/Low-level
- `simple_workflow.rs` - Low-level CimGraphEvent API
- `event_driven_simple.rs` - Alternative high-level API
- `pure_event_driven.rs` - Pure patterns demonstration
- `collaborative_graph.rs` - Collaboration patterns