# Event Design Correction - No Updates, Only Facts

## The Problem

The `PropertyUpdated` event violates Event-Driven Architecture principles:

```rust
// ❌ WRONG - This implies in-place mutation
pub struct PropertyUpdated {
    pub old_value: Option<Value>,
    pub new_value: Value,
    // ...
}
```

## The Principle

In pure event sourcing:
- Events record **facts that happened**, not changes
- There is no "update" - only removal of old state and addition of new state
- Each event should be independently meaningful

## Corrected Design

### Option 1: Atomic Property Events

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyRemoved {
    pub metadata: EventMetadata,
    pub graph_id: GraphId,
    pub target: PropertyTarget,
    pub property_path: PropertyPath,
    pub value: Value,
    pub removal_reason: RemovalReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyAdded {
    pub metadata: EventMetadata,
    pub graph_id: GraphId,
    pub target: PropertyTarget,
    pub property_path: PropertyPath,
    pub value: Value,
    pub semantic_meaning: SemanticMeaning,
}
```

### Option 2: Property State Events

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyStateRecorded {
    pub metadata: EventMetadata,
    pub graph_id: GraphId,
    pub target: PropertyTarget,
    pub property_path: PropertyPath,
    pub value: Value,
    pub effective_from: DateTime<Utc>,
    pub semantic_context: SemanticContext,
}

// Query would reconstruct current state by finding the latest PropertyStateRecorded
```

### Option 3: Convenience Wrapper (If Needed)

```rust
/// Convenience function that emits two events
pub fn update_property(
    graph_id: GraphId,
    target: PropertyTarget,
    path: PropertyPath,
    old_value: Value,
    new_value: Value,
) -> Vec<GraphDomainEvent> {
    let correlation_id = Uuid::new_v4();
    
    vec![
        GraphDomainEvent::PropertyRemoved(PropertyRemoved {
            metadata: EventMetadata::with_correlation(correlation_id),
            graph_id: graph_id.clone(),
            target: target.clone(),
            property_path: path.clone(),
            value: old_value,
            removal_reason: RemovalReason::Superseded,
        }),
        GraphDomainEvent::PropertyAdded(PropertyAdded {
            metadata: EventMetadata::with_correlation(correlation_id),
            graph_id,
            target,
            property_path: path,
            value: new_value,
            semantic_meaning: SemanticMeaning::Replacement,
        }),
    ]
}
```

## Benefits of This Approach

1. **Event Independence**: Each event is meaningful on its own
2. **No Lost Information**: We record both what was removed and what was added
3. **Better Auditability**: Can see exactly when properties were removed/added
4. **Temporal Queries**: Can query property state at any point in time
5. **No Implicit State**: Events don't assume knowledge of previous state

## Applied to Graph Domain

For the Graph Domain, this means:

```rust
// Instead of updating a node's property
graph.handle_event(PropertyUpdated { ... }); // ❌ Wrong

// We record the new state
graph.handle_event(PropertyStateRecorded { 
    value: new_value,
    effective_from: Utc::now(),
    ...
}); // ✅ Correct

// Or emit remove + add
graph.handle_events(vec![
    PropertyRemoved { ... },
    PropertyAdded { ... },
]); // ✅ Also Correct
```

## Event Sourcing Query Pattern

To get current property value:

```rust
impl GraphProjection {
    pub fn get_property_value(
        &self,
        target: PropertyTarget,
        path: PropertyPath,
        as_of: Option<DateTime<Utc>>,
    ) -> Option<Value> {
        let as_of = as_of.unwrap_or_else(Utc::now);
        
        self.events
            .iter()
            .filter(|e| matches!(e, GraphDomainEvent::PropertyStateRecorded(_)))
            .filter(|e| e.target() == target && e.path() == path)
            .filter(|e| e.effective_from() <= as_of)
            .max_by_key(|e| e.effective_from())
            .map(|e| e.value().clone())
    }
}
```

## Conclusion

Pure event-driven architecture requires us to think in terms of facts and states, not updates. This makes our event log a truthful, immutable record of everything that happened to our graphs.