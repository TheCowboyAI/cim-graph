# CIM Graph

[![Crates.io](https://img.shields.io/crates/v/cim-graph.svg)](https://crates.io/crates/cim-graph)
[![Documentation](https://docs.rs/cim-graph/badge.svg)](https://docs.rs/cim-graph)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](LICENSE)
[![Build Status](https://github.com/thecowboyai/cim-graph/workflows/CI/badge.svg)](https://github.com/thecowboyai/cim-graph/actions)
[![Downloads](https://img.shields.io/crates/d/cim-graph.svg)](https://crates.io/crates/cim-graph)
[![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://github.com/thecowboyai/cim-graph)

A pure event-driven graph library where events are the only source of truth. CIM Graph provides specialized graph types for different domains, all built on event sourcing principles with IPLD storage and NATS JetStream persistence.

## 🚨 Major Architecture Change

**CIM Graph is now purely event-driven. All state changes MUST go through events. Direct mutations are no longer supported.**

See [EVENT_DRIVEN_ARCHITECTURE.md](EVENT_DRIVEN_ARCHITECTURE.md) for the complete guide.

## Overview

CIM Graph implements a complete event-sourcing architecture where:

- **Events are the ONLY way to change state** - No direct mutations
- **Projections are ephemeral read models** - Rebuilt from events
- **IPLD provides content-addressed storage** - Every event payload gets a CID
- **NATS JetStream handles persistence** - Durable event streams
- **State machines enforce transitions** - Business rules validation
- **Policies automate behaviors** - CID generation, validation, etc.

## Key Features

- 🎯 **Pure Event Sourcing**: Complete audit trail with correlation/causation tracking
- 🔒 **Immutable by Design**: Events are append-only, projections are read-only
- 🌐 **Content Addressed**: All event payloads stored in IPLD with CIDs
- 📡 **NATS Integration**: Stream events with JetStream persistence
- 🏗️ **Domain-Driven Design**: Clear bounded contexts with defined relationships
- 🔄 **State Machines**: Enforce valid transitions and business rules
- 🤖 **Automated Policies**: React to events with configurable behaviors
- 📊 **Rich Projections**: Query current state through type-safe projections

## Recent Improvements (v0.1.1)

- ✅ **Complete Documentation**: All public APIs now have comprehensive documentation
- ✅ **Debug Implementations**: All types now implement Debug for better debugging experience
- ✅ **Zero Warnings**: Codebase compiles cleanly with all features enabled
- ✅ **Working Examples**: All 9 examples are fully functional and demonstrate best practices
- ✅ **Better Error Messages**: Improved error handling and messages throughout
- ✅ **Test Coverage**: 49 passing tests covering core functionality

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cim-graph = "0.1.1"

# Optional features
cim-graph = { version = "0.1.1", features = ["nats", "async"] }
```

## Quick Start

### Creating Events (The Only Way to Change State)

```rust
use cim_graph::{
    events::{GraphEvent, EventPayload, WorkflowPayload},
    core::{ProjectionEngine, GraphProjection},
    graphs::{WorkflowNode, WorkflowEdge},
};
use uuid::Uuid;

// Create events to build a workflow
let workflow_id = Uuid::new_v4();
let events = vec![
    GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
            workflow_id,
            name: "Order Processing".to_string(),
            version: "1.0.0".to_string(),
        }),
    },
    GraphEvent {
        event_id: Uuid::new_v4(),
        aggregate_id: workflow_id,
        correlation_id: Uuid::new_v4(),
        causation_id: None,
        payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
            workflow_id,
            state_id: "submitted".to_string(),
            state_type: "initial".to_string(),
        }),
    },
];

// Build projection from events
let engine = ProjectionEngine::<WorkflowNode, WorkflowEdge>::new();
let projection = engine.project(events);

// Query the projection (read-only)
println!("Nodes: {}", projection.node_count());
println!("Version: {}", projection.version());
```

## Bounded Contexts

CIM Graph is organized into 5 bounded contexts:

### 1. IPLD Context
Content-addressed storage for all data. Every event payload gets a CID.

### 2. Context Context (DDD)
Data schemas, transformations, and bounded context definitions.

### 3. Workflow Context
State machines, business processes, and workflow orchestration.

### 4. Concept Context
Domain knowledge, semantic reasoning, and ontologies.

### 5. Composed Context
Multi-graph orchestration and cross-domain queries.

## Event Flow

```
Command → State Machine → Event → IPLD → NATS → Projection
                             ↓
                          Policies
                             ↓
                     Additional Events
```

## Working with Projections

Projections are read-only views computed from events:

```rust
use cim_graph::core::{ProjectionEngine, GraphProjection};
use cim_graph::graphs::{ConceptNode, ConceptEdge};

// Build projection from events
let engine = ProjectionEngine::<ConceptNode, ConceptEdge>::new();
let projection = engine.project(concept_events);

// Query methods (all read-only)
let node_count = projection.node_count();
let has_node = projection.has_node("customer");
let has_edge = projection.has_edge("order", "customer");
```

## State Machines

Validate commands and enforce business rules:

```rust
use cim_graph::core::{GraphStateMachine, GraphCommand};

let mut state_machine = GraphStateMachine::new();

// Commands are validated before creating events
let command = GraphCommand::CreateGraph {
    aggregate_id: Uuid::new_v4(),
    graph_type: "workflow".to_string(),
    metadata: HashMap::new(),
};

let events = state_machine.handle_command(command, &projection)?;
```

## Policies

Automate behaviors in response to events:

```rust
use cim_graph::core::{PolicyEngine, CidGenerationPolicy, StateValidationPolicy};

let mut policy_engine = PolicyEngine::new();
policy_engine.add_policy(Box::new(CidGenerationPolicy));
policy_engine.add_policy(Box::new(StateValidationPolicy));

// Policies can generate additional events
let actions = policy_engine.evaluate(&event, &mut context)?;
```

## NATS JetStream Integration

Persist and stream events:

```rust
#[cfg(feature = "nats")]
use cim_graph::nats::JetStreamEventStore;

// Connect to NATS
let store = JetStreamEventStore::new("nats://localhost:4222").await?;

// Publish events with subject hierarchy
store.publish_events(&events).await?;

// Subscribe using subject patterns
let subscription = store.subscribe("cim.graph.workflow.*").await?;
```

## Examples

### Examples

All examples are now fully functional and demonstrate different aspects of the event-driven architecture:

- [Basic Event-Driven](examples/basic_event_driven.rs) - Introduction to event-driven concepts
- [Complete Event-Driven Demo](examples/complete_event_driven.rs) - Comprehensive demonstration of all features
- [Workflow Event-Driven](examples/workflow_event_driven.rs) - Workflow patterns with state machines
- [Order Processing System](examples/order_processing_system.rs) - Real-world e-commerce example
- [NATS Integration](examples/nats_event_driven.rs) - JetStream persistence (requires `--features nats`)
- [Simple Workflow](examples/simple_workflow.rs) - Core event-driven concepts with CimGraphEvent
- [Event-Driven Simple](examples/event_driven_simple.rs) - High-level event API demonstration
- [Pure Event-Driven](examples/pure_event_driven.rs) - Pure event patterns
- [Collaborative Graph](examples/collaborative_graph.rs) - Collaborative operations

See [EXAMPLES.md](EXAMPLES.md) for details on running the examples.

## Documentation

- [Event-Driven Architecture Guide](EVENT_DRIVEN_ARCHITECTURE.md) - Complete guide to the new architecture
- [Event Design Best Practices](docs/EVENT_DESIGN_BEST_PRACTICES.md) - How to design events properly
- [API Documentation](https://docs.rs/cim-graph) - Full API reference
- [Bounded Contexts](docs/bounded_contexts.md) - Domain boundaries and relationships
- [Migration Guide](MIGRATION_GUIDE.md) - Migrating from the old mutable API

## Migration from Old API

The old mutable API (`graph.add_node()`, `graph.add_edge()`, etc.) is no longer supported. See the [Migration Guide](MIGRATION_GUIDE.md) for step-by-step instructions on updating your code to the event-driven architecture.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.