# CIM Graph Event Storming Session

Following the CIM Event Storming Guide, this document captures the event storming session for the cim-graph domain refactoring.

## Session Details

- **Date**: 2025-08-05
- **Domain**: Graph Management in CIM
- **Focus**: Event-driven graph operations
- **Participants**: Domain experts (conceptual)

## Phase 1: Chaotic Exploration

### Initial Event Discovery (Orange Sticky Notes)

Events discovered in no particular order:

```
Graph Created                    Node Added                      Edge Added
Graph Initialized               Node Removed                    Edge Removed  
CID Generated                   Node Data Updated               Edge Data Updated
CID Linked                      State Transitioned              Workflow Completed
Context Bounded                 Aggregate Defined               Entity Created
Value Object Attached           Relationship Established        Concept Defined
Property Inferred               Cluster Formed                  Reasoning Path Found
SubGraph Added                  Cross Graph Link Created        Query Executed
Projection Built                Event Applied                   Command Validated
State Machine Triggered         Policy Executed                 Snapshot Taken
CID Pinned                      CID Unpinned                   DAG Verified
Chain Validated                 Event Replayed                  Projection Cached
Collaboration Started           Client Subscribed               Event Published
```

## Phase 2: Timeline Enforcement

Organizing events chronologically:

```mermaid
flowchart LR
    Start([START]) --> GC[Graph Created]
    GC --> GI[Graph Initialized]
    GI --> NA[Node Added]
    NA --> EA[Edge Added]
    EA --> EP[Event Published]
    
    NA --> CG[CID Generated]
    CG --> CL[CID Linked]
    CL --> CV[Chain Validated]
    
    EP --> PB[Projection Built]
    PB --> PC[Projection Cached]
    
    PC --> CS[Client Subscribed]
    CS --> ER[Event Replayed]
    
    %% Start/Root Nodes (Dark Gray)
    style Start fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    
    %% Secondary Elements (Teal) - Events
    style GC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style GI fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style NA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style EA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style EP fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CG fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CL fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Results/Outcomes (Light Green)
    style PB fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CS fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style ER fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

### Parallel Flows Identified

1. **IPLD Flow**: CID Generated → CID Linked → DAG Verified
2. **DDD Flow**: Context Bounded → Aggregate Defined → Entity Created
3. **Workflow Flow**: State Transitioned → Policy Executed → Workflow Completed
4. **Reasoning Flow**: Concept Defined → Property Inferred → Reasoning Path Found

```mermaid
flowchart TB
    subgraph "IPLD Flow"
        CG1[CID Generated] --> CL1[CID Linked]
        CL1 --> DV1[DAG Verified]
    end
    
    subgraph "DDD Flow"
        CB2[Context Bounded] --> AD2[Aggregate Defined]
        AD2 --> EC2[Entity Created]
    end
    
    subgraph "Workflow Flow"
        ST3[State Transitioned] --> PE3[Policy Executed]
        PE3 --> WC3[Workflow Completed]
    end
    
    subgraph "Reasoning Flow"
        CD4[Concept Defined] --> PI4[Property Inferred]
        PI4 --> RP4[Reasoning Path Found]
    end
    
    %% Secondary Elements (Teal) - Events and Processing
    style CG1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CL1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CB2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style AD2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style ST3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PE3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CD4 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PI4 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Results/Outcomes (Light Green)
    style DV1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style EC2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style WC3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style RP4 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Phase 3: Commands Discovery (Blue Sticky Notes)

### Command-Event Pairs

```yaml
Commands → Events:
  # Graph Lifecycle
  Create Graph → Graph Created
  Initialize Graph → Graph Initialized
  
  # IPLD Operations
  Add CID → CID Generated + Node Added
  Link CIDs → CID Linked + Edge Added
  Pin CID → CID Pinned
  Unpin CID → CID Unpinned
  
  # Context Operations
  Create Bounded Context → Context Bounded
  Add Aggregate → Aggregate Defined
  Add Entity → Entity Created
  Attach Value Object → Value Object Attached
  
  # Workflow Operations
  Define Workflow → Workflow Created
  Add State → State Added
  Add Transition → Transition Added
  Trigger Transition → State Transitioned
  
  # Concept Operations
  Define Concept → Concept Defined
  Add Properties → Properties Added
  Add Relation → Relation Added
  Run Inference → Property Inferred
  
  # Composed Operations
  Add SubGraph → SubGraph Added
  Link Across Graphs → Cross Graph Link Created
```

```mermaid
flowchart LR
    subgraph Commands [Commands]
        C1[Create Graph]
        C2[Add Node]
        C3[Add CID]
        C4[Link CIDs]
        C5[Define Workflow]
    end
    
    subgraph Events [Events]
        E1[Graph Created]
        E2[Node Added]
        E3[CID Generated]
        E4[CID Linked]
        E5[Edge Added]
        E6[Workflow Created]
    end
    
    C1 -->|produces| E1
    C2 -->|produces| E2
    C3 -->|produces| E3
    C3 -->|produces| E2
    C4 -->|produces| E4
    C4 -->|produces| E5
    C5 -->|produces| E6
    
    %% Primary/Important Elements (Red) - Commands
    style C1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C3 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C4 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C5 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Secondary Elements (Teal) - Events
    style E1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E4 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E5 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E6 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## Phase 4: Aggregate Identification (Yellow Sticky Notes)

### Core Aggregates

```yaml
Graph Aggregate:
  id: aggregate_id (UUID)
  commands:
    - Initialize Graph
    - Add Node
    - Add Edge
    - Remove Node
    - Remove Edge
  events:
    - Graph Initialized
    - Node Added
    - Edge Added
    - Node Removed
    - Edge Removed
  invariants:
    - Nodes must exist before edges can connect them
    - Graph type cannot change after initialization
    - All changes must go through state machine

IPLD Chain Aggregate:
  id: root_cid
  commands:
    - Add CID
    - Link CIDs
    - Pin CID
    - Verify Chain
  events:
    - CID Added
    - CID Linked
    - CID Pinned
    - Chain Verified
  invariants:
    - CIDs are immutable
    - Links form DAG (no cycles)
    - Previous CID must exist

Projection Aggregate:
  id: aggregate_id + version
  commands:
    - Build Projection
    - Cache Projection
    - Invalidate Cache
  events:
    - Projection Built
    - Projection Cached
    - Cache Invalidated
  invariants:
    - Projections are read-only
    - Version matches event sequence
    - Built from complete event stream
```

### Aggregate Relationships

```mermaid
graph TB
    subgraph GA["Graph Aggregate"]
        GAI[Graph ID<br/>aggregate_id: UUID]
        N1[Node 1<br/>ID: String]
        N2[Node 2<br/>ID: String]
        E1[Edge 1<br/>connects nodes]
        
        GAI --> N1
        GAI --> N2
        GAI --> E1
        E1 -.->|source| N1
        E1 -.->|target| N2
    end
    
    subgraph IC["IPLD Chain"]
        CID1[Event 1 CID<br/>hash: abc123]
        CID2[Event 2 CID<br/>hash: def456]
        CID3[Event 3 CID<br/>hash: ghi789]
        
        CID1 -->|previous| CID2
        CID2 -->|previous| CID3
    end
    
    subgraph PR["Projection States"]
        P1[Version 1<br/>Initial]
        P2[Version 2<br/>+Nodes]
        P3[Version 3<br/>Current State]
        
        P1 --> P2
        P2 --> P3
    end
    
    GAI ==>|generates events| CID1
    CID3 ==>|builds| P3
    
    %% Choice/Decision Points (Yellow) - Aggregates
    style GAI fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style N1 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style N2 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style E1 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Secondary Elements (Teal) - Storage/Chain
    style CID1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CID2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CID3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Results/Outcomes (Light Green) - Projections
    style P1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style P2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style P3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Make subgraph labels visible
    style GA fill:transparent,stroke:#FCC419,stroke-width:2px
    style IC fill:transparent,stroke:#2B8A89,stroke-width:2px
    style PR fill:transparent,stroke:#63C7B8,stroke-width:2px
```

### Entity Discoveries

```yaml
Entities:
  Node:
    - id: String (CID for IPLD, UUID for others)
    - components: HashMap<String, Value>
    - belongs_to: Graph Aggregate
    
  Edge:
    - id: String
    - source: Node ID
    - target: Node ID
    - components: HashMap<String, Value>
    - belongs_to: Graph Aggregate
    
  EventPayload:
    - cid: CID (from cim-ipld)
    - data: Serialized event data
    - previous: Optional<CID>
    - belongs_to: IPLD Chain Aggregate
```

### Value Objects

```yaml
Value Objects:
  CID:
    - hash: String
    - codec: String
    - immutable: true
    
  GraphType:
    - enum: Generic | IPLD | Context | Workflow | Concept | Composed
    - immutable: true
    
  ComponentData:
    - type: String
    - value: JSON
    - immutable: true
    
  StateTransition:
    - from: State
    - to: State
    - trigger: Command
    - immutable: true
```

## Phase 5: Policy Discovery (Purple Sticky Notes)

### Automated Policies

```yaml
Policies:
  CID Generation Policy:
    trigger: Any Event Created
    action: Generate CID from event payload
    rules:
      - Use cim-ipld for generation
      - Include previous CID if exists
      - Store in event metadata
      
  Projection Update Policy:
    trigger: Event Published
    action: Update affected projections
    rules:
      - Only update projections for same aggregate
      - Maintain version consistency
      - Invalidate caches
      
  State Validation Policy:
    trigger: Command Received
    action: Validate against current state
    rules:
      - Check state machine rules
      - Verify invariants
      - Return error if invalid
      
  Chain Validation Policy:
    trigger: CID Chain Modified
    action: Verify chain integrity
    rules:
      - All CIDs must be valid
      - Links must form DAG
      - Previous references must exist
      
  Collaboration Policy:
    trigger: Client Subscribed
    action: Replay events from sequence
    rules:
      - Start from client's last known sequence
      - Send in order
      - Include all metadata
```

### Policy Flow Diagram

```mermaid
flowchart TD
    E[Any Event<br/>Created] --> P1{CID Generation<br/>Policy}
    P1 -->|Generate| CID[CID from<br/>Payload]
    CID --> EM[Store in<br/>Event Metadata]
    
    E --> P2{Projection<br/>Update Policy}
    P2 -->|Check| A{Same<br/>Aggregate?}
    A -->|Yes| UP[Update<br/>Projection]
    UP --> IC[Invalidate<br/>Cache]
    A -->|No| SKIP[Skip Update]
    
    C[Command<br/>Received] --> P3{State Validation<br/>Policy}
    P3 -->|Validate| SM[State Machine<br/>Rules]
    SM -->|Valid| EV[Generate<br/>Events]
    SM -->|Invalid| ER[Return<br/>Error]
    
    CH[Chain<br/>Modified] --> P4{Chain Validation<br/>Policy}
    P4 -->|Check| DAG[Verify DAG<br/>Structure]
    DAG -->|Valid| CH_OK[Chain Valid]
    DAG -->|Invalid| CH_ERR[Chain Error]
    
    CS[Client<br/>Subscribed] --> P5{Collaboration<br/>Policy}
    P5 -->|Action| REP[Replay Events<br/>from Sequence]
    
    %% Start/Root Nodes (Dark Gray)
    style E fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style C fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style CH fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style CS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    
    %% Choice/Decision Points (Yellow) - Policies
    style P1 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style P2 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style P3 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style P4 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style P5 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style A fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Secondary Elements (Teal) - Processing
    style CID fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style SM fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style UP fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style DAG fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style REP fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Results/Outcomes (Light Green)
    style EM fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style IC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style EV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style SKIP fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CH_OK fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Primary/Important Elements (Red) - Errors
    style ER fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CH_ERR fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Phase 6: Bounded Context Definition

### Context Map

```mermaid
graph TB
    subgraph GC["Graph Context - Core Domain"]
        GC1[Graph Aggregate]
        GC2[Node/Edge Entities]
        GC3[GraphCommand, GraphEvent]
        GC4[GraphProjection]
    end
    
    subgraph IC["IPLD Context - Supporting Domain"]
        IC1[CID Chain Aggregate]
        IC2[EventPayload Entity]
        IC3[CID Generation Policy]
    end
    
    subgraph SMC["State Machine Context - Supporting Domain"]
        SMC1[State Transitions]
        SMC2[Command Validation]
        SMC3[Policy Execution]
    end
    
    subgraph PC["Projection Context - Generic Domain"]
        PC1[Projection Aggregate]
        PC2[Query Systems]
        PC3[Cache Management]
    end
    
    subgraph CC["Collaboration Context - Generic Domain"]
        CC1[Event Streaming]
        CC2[Client Subscriptions]
        CC3[Real-time Updates]
    end
    
    GC ==>|"Upstream-Downstream<br/>Graph events get CIDs"| IC
    GC ==>|"Upstream-Downstream<br/>Commands validated"| SMC
    GC ==>|"Customer-Supplier<br/>Events build projections"| PC
    PC ==>|"Customer-Supplier<br/>Projections served"| CC
    
    %% Primary/Important Elements (Red) - Core Domain
    style GC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style GC1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:3px,color:#FFF
    style GC2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:3px,color:#FFF
    style GC3 fill:#FF6B6B,stroke:#C92A2A,stroke-width:3px,color:#FFF
    style GC4 fill:#FF6B6B,stroke:#C92A2A,stroke-width:3px,color:#FFF
    
    %% Secondary Elements (Teal) - Supporting Domains
    style IC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style IC1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:2px,color:#FFF
    style IC2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:2px,color:#FFF
    style IC3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:2px,color:#FFF
    style SMC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style SMC1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:2px,color:#FFF
    style SMC2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:2px,color:#FFF
    style SMC3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:2px,color:#FFF
    
    %% Results/Outcomes (Light Green) - Generic Domains
    style PC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PC1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PC2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PC3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CC1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CC2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CC3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

### Context Relationships

- **Graph → IPLD**: Graph events get CIDs from IPLD context
- **Graph → State Machine**: All commands validated by state machine
- **Graph → Projection**: Events build projections
- **Projection → Collaboration**: Projections served to clients

## Discovered Event Catalog

```yaml
# cim-graph-events.yaml
domain: CIM Graph
version: 2.0.0

aggregates:
  - name: Graph
    events:
      - name: GraphInitialized
        data:
          aggregate_id: UUID
          graph_type: GraphType
          metadata: Map<String, Value>
          
      - name: NodeAdded  
        data:
          node_id: String
          node_type: String
          data: JSON
          
      - name: EdgeAdded
        data:
          edge_id: String
          source_id: String  
          target_id: String
          edge_type: String
          data: JSON
          
      - name: NodeRemoved
        data:
          node_id: String
          
      - name: EdgeRemoved
        data:
          edge_id: String

  - name: IPLDChain
    events:
      - name: CIDGenerated
        data:
          event_id: UUID
          cid: CID
          payload_size: Integer
          
      - name: CIDLinked
        data:
          current_cid: CID
          previous_cid: CID
          sequence: Integer
          
      - name: ChainValidated
        data:
          root_cid: CID
          chain_length: Integer
          is_valid: Boolean
```

## State Machines Discovered

```rust
// Graph State Machine
enum GraphState {
    Uninitialized,
    Initialized { graph_type: GraphType },
    Active { nodes: usize, edges: usize },
    Archived,
}

// Valid Transitions
Uninitialized → Initialized (via InitializeGraph command)
Initialized → Active (via AddNode command)
Active → Active (via Add/Remove commands)
Active → Archived (via ArchiveGraph command)

// Workflow Instance State Machine  
enum WorkflowState {
    Draft,
    Published,
    Running { current_state: String },
    Completed,
    Failed { error: String },
}
```

### Graph State Machine Diagram

```mermaid
stateDiagram-v2
    [*] --> Uninitialized
    Uninitialized --> Initialized: InitializeGraph
    Initialized --> Active: AddNode
    Active --> Active: Add/Remove Node/Edge
    Active --> Archived: ArchiveGraph
    Archived --> [*]
    
    state Initialized {
        [*] --> GraphTypeSet
        GraphTypeSet: graph_type: GraphType
    }
    
    state Active {
        [*] --> HasNodes
        HasNodes --> HasEdges: AddEdge
        HasEdges --> HasNodes: RemoveEdge
        
        HasNodes: nodes > 0
        HasEdges: edges > 0
    }
```

### Workflow State Machine Diagram

```mermaid
stateDiagram-v2
    [*] --> Draft
    Draft --> Published: PublishWorkflow
    Published --> Running: StartWorkflow
    Running --> Completed: WorkflowSuccess
    Running --> Failed: WorkflowError
    Failed --> Running: RetryWorkflow
    Completed --> [*]
    Failed --> [*]
    
    state Running {
        [*] --> ExecutingState
        ExecutingState --> WaitingForEvent: AwaitTrigger
        WaitingForEvent --> ExecutingState: EventReceived
        
        ExecutingState: current_state: String
        WaitingForEvent: awaiting trigger
    }
    
    state Failed {
        [*] --> ErrorRecorded
        ErrorRecorded: error: String
    }
```

## Implementation Priority

Based on the event storming, implementation order should be:

1. **Core Event Infrastructure** (Foundation)
   - GraphEvent with correlation/causation
   - EventPayload definitions
   - Command definitions

2. **IPLD Integration** (Heart of storage)
   - CID generation for all payloads
   - Chain construction
   - DAG verification

3. **State Machine** (Control layer)
   - Command validation
   - State transitions
   - Policy execution

4. **Projections** (Read layer)
   - Fold events to state
   - Query systems
   - Cache management

5. **Collaboration** (Distribution)
   - NATS integration
   - Event streaming
   - Multi-client sync

```mermaid
gantt
    title Implementation Phases
    dateFormat YYYY-MM-DD
    section Phase 1
    Core Event Infrastructure    :2025-08-05, 5d
    section Phase 2
    IPLD Integration            :5d
    section Phase 3
    State Machine              :5d
    section Phase 4
    Projections                :5d
    section Phase 5
    Collaboration              :5d
```

## Key Insights

1. **Everything is an Event** - No direct state changes
2. **CIDs are Central** - Every payload gets a CID
3. **Projections are Ephemeral** - Can rebuild from events
4. **State Machines Control** - All transitions validated
5. **Collaboration is Native** - Multi-client by design

### Complete Event Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant SM as State Machine
    participant V as Validator
    participant IPLD as IPLD
    participant JS as JetStream
    participant P as Projection
    participant S as Subscribers
    
    C->>SM: Send Command
    SM->>V: Validate Command
    V-->>SM: Validation Result
    
    alt Valid Command
        SM->>SM: Generate Event(s)
        SM->>IPLD: Create CID for Payload
        IPLD-->>SM: Return CID
        SM->>JS: Publish Event with CID
        JS->>JS: Persist Event
        JS->>P: Notify Projection
        P->>P: Apply Event (fold)
        P->>P: Update Cache
        JS->>S: Broadcast to Subscribers
        S-->>C: Real-time Update
    else Invalid Command
        SM-->>C: Return Error
    end
    
    Note over JS,S: All subscribers receive<br/>events in order
```

## Next Steps

1. Translate event catalog to Rust enums
2. Implement state machines
3. Create command handlers
4. Build projection engines
5. Integrate with NATS JetStream

---

*This event storming session revealed that graphs in CIM are not data structures but event streams with projections. This fundamental insight drives the entire refactoring.*

## Visual Summary

```mermaid
graph TB
    subgraph "Event-Driven Architecture"
        C[Commands] --> SM[State Machine]
        SM --> E[Events]
        E --> IPLD[IPLD CIDs]
        IPLD --> JS[JetStream]
        JS --> P[Projections]
    end
    
    subgraph "Collaboration"
        JS --> S1[Client 1]
        JS --> S2[Client 2]
        JS --> S3[Client N]
    end
    
    %% Primary/Important Elements (Red) - Commands
    style C fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style SM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Secondary Elements (Teal) - Events and Storage
    style E fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style IPLD fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style JS fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Results/Outcomes (Light Green) - Projections and Clients
    style P fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style S1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style S2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style S3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```