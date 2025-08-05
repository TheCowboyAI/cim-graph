# IPLD Storage Architecture - The Heart of CIM

## Overview

IPLD (InterPlanetary Linked Data) graphs form the heart of CIM's storage system. Every event in the system has its payload content-addressed with a CID (Content Identifier), creating an immutable, verifiable, and efficient storage mechanism.

## Key Concepts

### Event Payloads and CIDs

When an event is created in CIM:
1. The event payload (data without metadata) is extracted
2. The payload is given a CID through the `cim-ipld` library
3. This CID uniquely identifies and names the payload content

```rust
// Event with metadata
GraphEvent {
    event_id: Uuid,
    aggregate_id: Uuid,
    sequence: 64,
    timestamp: DateTime,
    // ... other metadata
    data: EventData::NodeAdded { /* payload */ }
}

// Payload gets CID
CID: "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
```

### Merkle DAGs and CID Chains

Events don't exist in isolation - they form chains:
1. Each event references the CID of the previous event
2. This creates a Merkle DAG (Directed Acyclic Graph)
3. The entire aggregate transaction history becomes a CID chain

```
Event 1: CID_1 (no previous)
Event 2: CID_2 (previous: CID_1)
Event 3: CID_3 (previous: CID_2)
Event 4: CID_4 (previous: CID_3) <- Root CID
```

### Single CID References

The power of CID chains:
- The root CID (latest event) identifies the entire transaction
- One CID request to NATS JetStream retrieves the complete event stream
- The Merkle DAG ensures integrity - any tampering changes all downstream CIDs

## Architecture Components

### 1. cim-graph
- Defines graph events and projections
- Manages event-driven graph state
- Provides GraphEvent and EventData types

### 2. cim-ipld
- Generates CIDs from event payloads
- Manages Merkle DAG construction
- Handles IPLD codec operations
- Provides CID verification

### 3. cim-subject
- Defines NATS subject algebra for events
- Routes events to proper streams
- Manages subject hierarchies

### 4. NATS JetStream
- Persists events with CID indexing
- Provides stream replay by CID
- Handles event ordering and delivery

## Storage Flow

```
1. Command → Graph System
2. Graph System → Event (with payload)
3. Event Payload → cim-ipld → CID
4. Event + CID → NATS JetStream
5. CID Chain updated with new root
```

## Retrieval Flow

```
1. Client requests by root CID
2. NATS JetStream lookup by CID
3. Retrieve event and previous CID
4. Follow chain to retrieve all events
5. Rebuild projection from events
```

## Benefits

### Immutability
- CIDs are content-addressed - data cannot be changed without changing the CID
- Provides cryptographic proof of data integrity
- Perfect audit trail for compliance

### Efficiency
- Content deduplication - same data always has same CID
- Incremental sync - only fetch missing CIDs
- Parallel retrieval of DAG branches

### Verifiability
- Anyone can verify the integrity of data given its CID
- Chain of events can be cryptographically verified
- No trust required - math ensures correctness

### Simplicity
- One CID references entire aggregate history
- Share a CID to share complete transaction
- Universal addressing across all CIM systems

## Example Use Cases

### 1. Audit Trails
```rust
// Share audit trail for order processing
let audit_cid = "QmOrderProcessingAudit123";
// Anyone can retrieve and verify complete history
```

### 2. Collaborative Editing
```rust
// Multiple clients working on same graph
// Each shares same root CID
// All see identical event history
```

### 3. Time Travel
```rust
// Get graph state at specific version
let cid_at_v10 = chain.get_cid(10);
// Rebuild projection from events up to v10
```

### 4. Cross-System References
```rust
// Reference graph state in other systems
let invoice = Invoice {
    workflow_state: "QmWorkflowCID123",
    // ...
};
```

## Integration Example

```rust
// When creating an event
let event = GraphEvent {
    aggregate_id: uuid,
    data: EventData::NodeAdded { /* ... */ },
    // ...
};

// cim-ipld generates CID
let payload_cid = ipld.generate_cid(&event.data);

// Store in JetStream with CID
jetstream.publish(
    subject: "graph.{}.events".format(aggregate_id),
    payload: event,
    headers: {
        "Nats-Msg-Id": payload_cid,
        "CID": payload_cid,
        "Previous-CID": previous_cid,
    }
);

// Update CID chain
cid_chain.add(payload_cid, previous_cid);
```

## Conclusion

IPLD graphs are not just another graph type - they are the foundational storage mechanism that enables CIM's event-driven, distributed, and verifiable architecture. Every event becomes part of an immutable Merkle DAG, with entire transaction histories referenceable and retrievable through a single CID.