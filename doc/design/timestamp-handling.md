# Timestamp Handling in Graph Domain

## Architectural Decision

**Timestamps are NOT stored in domain events**. They are managed entirely by the messaging infrastructure (NATS).

## Rationale

1. **Separation of Concerns**
   - Domain events represent business facts
   - Timestamps are infrastructure concerns
   - NATS provides reliable, consistent timestamps

2. **Single Source of Truth**
   - NATS headers provide authoritative timestamps
   - Prevents timestamp inconsistencies
   - Eliminates clock synchronization issues

3. **Event Ordering**
   - NATS provides guaranteed ordering via sequence numbers
   - Stream position is more reliable than timestamps
   - No need for complex timestamp-based ordering

## Implementation

### Domain Events
```rust
// ❌ WRONG - Don't include timestamps
pub struct EventWithTimestamp {
    pub occurred_at: DateTime<Utc>,
    // ...
}

// ✅ CORRECT - Let NATS handle it
pub struct DomainEvent {
    pub event_id: Uuid,
    pub data: EventData,
    // No timestamp field
}
```

### Reading Timestamps
```rust
// When processing events from NATS
impl NatsEventHandler {
    async fn handle_message(&self, msg: Message) -> Result<()> {
        // Get timestamp from NATS headers
        let timestamp = msg.headers
            .get("Nats-Time-Stamp")
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok());
        
        // Get sequence for ordering
        let sequence = msg.headers
            .get("Nats-Sequence")
            .and_then(|v| v.parse::<u64>().ok());
        
        // Process the event
        let event: GraphDomainEvent = serde_json::from_slice(&msg.data)?;
        
        // Use timestamp/sequence as needed for projections
        self.process_event(event, timestamp, sequence).await
    }
}
```

### Temporal Queries
```rust
// Query by sequence number (reliable)
pub fn events_before(sequence: u64) -> Vec<Event> {
    self.events.iter()
        .filter(|e| e.sequence <= sequence)
        .collect()
}

// Query by time range (if needed)
pub fn events_in_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Event> {
    // Use NATS consumer with time-based replay
    self.nats_consumer
        .replay_from_time(start)
        .take_until(|msg| msg.timestamp > end)
        .collect()
}
```

## Benefits

1. **Consistency**: All timestamps come from one source
2. **Reliability**: NATS handles time synchronization
3. **Simplicity**: Events focus on domain logic only
4. **Auditability**: Infrastructure provides audit trail

## Event Replay

When replaying events:
- Use NATS replay features
- Sequence numbers ensure correct ordering
- Original timestamps preserved in headers
- No timestamp manipulation in domain code

## Conclusion

By deferring timestamp handling to NATS, we achieve:
- Cleaner domain models
- More reliable event ordering
- Better separation of concerns
- Simplified event processing