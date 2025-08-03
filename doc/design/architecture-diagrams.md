# Graph Domain Architecture Diagrams

## System Architecture Overview

```mermaid
graph TB
    subgraph "External Systems"
        Client[Client Applications]
        NATS[NATS Message Bus]
        Storage[Event Store]
    end
    
    subgraph "Graph Domain Core"
        API[Graph API Layer]
        Registry[Graph Registry]
        EventHandler[Event Handlers]
        Validator[Semantic Validator]
        Composer[Graph Composer]
        
        API --> Registry
        API --> EventHandler
        EventHandler --> Validator
        EventHandler --> Registry
        Composer --> Registry
        Composer --> Validator
    end
    
    subgraph "Graph Types"
        IPLD[IPLD Graph]
        Context[Context Graph]
        Workflow[Workflow Graph]
        Concept[Concept Graph]
    end
    
    subgraph "Infrastructure"
        EventStore[Event Store]
        ProjectionStore[Projection Store]
        QueryEngine[Query Engine]
    end
    
    Client --> API
    API --> NATS
    NATS --> EventHandler
    EventHandler --> EventStore
    EventStore --> ProjectionStore
    QueryEngine --> ProjectionStore
    
    Registry --> IPLD
    Registry --> Context
    Registry --> Workflow
    Registry --> Concept
    
    style API fill:#f9f,stroke:#333,stroke-width:4px
    style EventHandler fill:#bbf,stroke:#333,stroke-width:2px
    style Registry fill:#bfb,stroke:#333,stroke-width:2px
```

## Event Flow Architecture

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant EventBus
    participant Handler
    participant Graph
    participant EventStore
    participant Projections
    
    Client->>API: Request (e.g., AddNode)
    API->>API: Validate Request
    API->>EventBus: Publish Command Event
    EventBus->>Handler: Route to Handler
    Handler->>Handler: Validate Semantics
    Handler->>Graph: Apply Mutation
    Graph-->>Handler: Return Result
    Handler->>EventStore: Persist Event
    EventStore->>Projections: Update Projections
    Handler->>EventBus: Publish Domain Event
    EventBus-->>Client: Event Notification
    API-->>Client: Return Result
```

## Graph Type Hierarchy

```mermaid
classDiagram
    class CimGraph {
        <<trait>>
        +id() GraphId
        +semantic_type() SemanticGraphType
        +handle_event(event) Result
        +query(query) Result
        +validate() Result
    }
    
    class SemanticGraph {
        <<trait>>
        +semantic_context() Context
        +invariants() Vec~Invariant~
    }
    
    class EventDrivenGraph {
        <<trait>>
        +event_log() EventLog
        +replay_events() Result
    }
    
    class ComposableGraph {
        <<trait>>
        +can_compose_with(other) bool
        +compose(other) Result
    }
    
    class IpldGraph {
        +root_cid() Cid
        +resolve_path(path) Result
        +merkle_proof(node) Proof
    }
    
    class ContextGraph {
        +bounded_context() Context
        +aggregates() Vec~Aggregate~
        +validate_ddd() Result
    }
    
    class WorkflowGraph {
        +current_states() Vec~State~
        +transitions() Vec~Transition~
        +execute(event) Result
    }
    
    class ConceptGraph {
        +dimensions() Vec~Dimension~
        +distance(a, b) f64
        +infer_relations() Vec~Relation~
    }
    
    CimGraph <|-- SemanticGraph
    CimGraph <|-- EventDrivenGraph
    CimGraph <|-- ComposableGraph
    
    SemanticGraph <|-- IpldGraph
    SemanticGraph <|-- ContextGraph
    SemanticGraph <|-- WorkflowGraph
    SemanticGraph <|-- ConceptGraph
    
    EventDrivenGraph <|-- IpldGraph
    EventDrivenGraph <|-- ContextGraph
    EventDrivenGraph <|-- WorkflowGraph
    EventDrivenGraph <|-- ConceptGraph
```

## Component Interaction Diagram

```mermaid
graph LR
    subgraph "Graph Registry"
        R1[Active Graphs]
        R2[Graph Metadata]
        R3[Graph Handles]
    end
    
    subgraph "Event System"
        E1[Event Bus]
        E2[Event Store]
        E3[Event Replay]
    end
    
    subgraph "Semantic Layer"
        S1[Type Validator]
        S2[Invariant Checker]
        S3[Semantic Mapper]
    end
    
    subgraph "Composition Engine"
        C1[Merge Strategy]
        C2[Conflict Resolver]
        C3[Bridge Builder]
    end
    
    subgraph "Query Engine"
        Q1[Query Parser]
        Q2[Query Optimizer]
        Q3[Result Projector]
    end
    
    R1 --> S1
    R1 --> C1
    E1 --> R1
    E2 --> E3
    E3 --> R1
    S2 --> E1
    C2 --> S3
    Q1 --> R1
    Q2 --> R2
    Q3 --> E2
```

## Data Flow Through Graph Types

```mermaid
graph TD
    subgraph "Input Layer"
        JSON[JSON Data]
        NIX[Nix Expressions]
        API[API Calls]
        Events[Domain Events]
    end
    
    subgraph "Transformation Layer"
        Parser[Parser/Deserializer]
        Validator[Semantic Validator]
        Transformer[Type Transformer]
    end
    
    subgraph "Graph Layer"
        IPLD[IPLD Graph<br/>CID: abc123]
        Context[Context Graph<br/>BC: Orders]
        Workflow[Workflow Graph<br/>State: Active]
        Concept[Concept Graph<br/>Space: 5D]
    end
    
    subgraph "Composition Layer"
        Bridge1[IPLD ↔ Context]
        Bridge2[Context ↔ Workflow]
        Bridge3[Workflow ↔ Concept]
        Unified[Unified Graph View]
    end
    
    subgraph "Output Layer"
        Queries[Query Results]
        Projections[Projections]
        Subscriptions[Event Streams]
        Exports[JSON/Nix Export]
    end
    
    JSON --> Parser
    NIX --> Parser
    API --> Parser
    Events --> Parser
    
    Parser --> Validator
    Validator --> Transformer
    
    Transformer --> IPLD
    Transformer --> Context
    Transformer --> Workflow
    Transformer --> Concept
    
    IPLD --> Bridge1
    Context --> Bridge1
    Context --> Bridge2
    Workflow --> Bridge2
    Workflow --> Bridge3
    Concept --> Bridge3
    
    Bridge1 --> Unified
    Bridge2 --> Unified
    Bridge3 --> Unified
    
    Unified --> Queries
    Unified --> Projections
    Unified --> Subscriptions
    Unified --> Exports
    
    style Unified fill:#f96,stroke:#333,stroke-width:4px
```

## Graph Lifecycle State Machine

```mermaid
stateDiagram-v2
    [*] --> Created: GraphCreated Event
    Created --> Initializing: Initialize
    Initializing --> Active: Ready
    
    Active --> Mutating: Mutation Event
    Mutating --> Active: Mutation Complete
    
    Active --> Querying: Query Request
    Querying --> Active: Query Complete
    
    Active --> Composing: Composition Request
    Composing --> Active: Composition Complete
    Composing --> CompositionFailed: Semantic Conflict
    CompositionFailed --> Active: Retry/Abort
    
    Active --> Validating: Validation Triggered
    Validating --> Active: Valid
    Validating --> Invalid: Validation Failed
    Invalid --> Repairing: Auto-Repair
    Repairing --> Active: Repaired
    
    Active --> Archiving: Archive Request
    Archiving --> Archived: Archived
    Archived --> [*]
    
    Active --> Transforming: Transform Request
    Transforming --> Transformed: New Graph Created
    Transformed --> Active: Continue as New
```

## Semantic Composition Patterns

```mermaid
graph TB
    subgraph "Composition Patterns"
        subgraph "Pattern 1: Overlay"
            A1[Base Graph]
            A2[Overlay Graph]
            A3[Combined View]
            A1 --> A3
            A2 --> A3
        end
        
        subgraph "Pattern 2: Bridge"
            B1[Graph A]
            B2[Graph B]
            B3[Semantic Bridge]
            B1 <--> B3
            B3 <--> B2
        end
        
        subgraph "Pattern 3: Transform"
            C1[Source Graph]
            C2[Transformation]
            C3[Target Graph]
            C1 --> C2
            C2 --> C3
        end
        
        subgraph "Pattern 4: Merge"
            D1[Graph Set]
            D2[Merge Rules]
            D3[Unified Graph]
            D1 --> D2
            D2 --> D3
        end
    end
```

## Query Execution Flow

```mermaid
flowchart TD
    Start([Query Request]) --> Parse{Parse Query}
    Parse -->|Valid| Analyze[Semantic Analysis]
    Parse -->|Invalid| Error1[Syntax Error]
    
    Analyze --> Optimize[Query Optimization]
    Optimize --> Route{Route to Graph}
    
    Route -->|IPLD| IPLD_Query[IPLD Path Resolution]
    Route -->|Context| Context_Query[DDD Traversal]
    Route -->|Workflow| Workflow_Query[State Navigation]
    Route -->|Concept| Concept_Query[Semantic Search]
    
    IPLD_Query --> Merge[Merge Results]
    Context_Query --> Merge
    Workflow_Query --> Merge
    Concept_Query --> Merge
    
    Merge --> Project[Apply Projections]
    Project --> Cache{Cache Result?}
    Cache -->|Yes| Store[Store in Cache]
    Cache -->|No| Return[Return Result]
    Store --> Return
    
    Return --> End([Query Response])
    Error1 --> End
```

## Event Sourcing Architecture

```mermaid
graph TD
    subgraph "Write Side"
        Command[Command/Request]
        CommandHandler[Command Handler]
        DomainLogic[Domain Logic]
        EventCreation[Event Creation]
        EventStore[(Event Store)]
        
        Command --> CommandHandler
        CommandHandler --> DomainLogic
        DomainLogic --> EventCreation
        EventCreation --> EventStore
    end
    
    subgraph "Read Side"
        EventStore2[(Event Store)]
        Projector[Event Projector]
        ReadModel1[Graph State]
        ReadModel2[Statistics]
        ReadModel3[Search Index]
        QueryHandler[Query Handler]
        
        EventStore2 --> Projector
        Projector --> ReadModel1
        Projector --> ReadModel2
        Projector --> ReadModel3
        
        QueryHandler --> ReadModel1
        QueryHandler --> ReadModel2
        QueryHandler --> ReadModel3
    end
    
    EventStore -.-> EventStore2
    
    subgraph "Event Bus"
        EventPublisher[Event Publisher]
        Subscribers[Event Subscribers]
        
        EventStore --> EventPublisher
        EventPublisher --> Subscribers
    end
    
    style EventStore fill:#faa,stroke:#333,stroke-width:2px
    style EventStore2 fill:#faa,stroke:#333,stroke-width:2px
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        CLI[CLI Tool]
        WebUI[Web UI]
        API_Client[API Client]
    end
    
    subgraph "Gateway Layer"
        Gateway[API Gateway]
        Auth[Auth Service]
        RateLimit[Rate Limiter]
    end
    
    subgraph "Service Layer"
        GraphAPI[Graph API Service]
        EventService[Event Service]
        QueryService[Query Service]
        ComposeService[Composition Service]
    end
    
    subgraph "Domain Layer"
        GraphDomain[Graph Domain Core]
        Registry[Graph Registry]
        EventHandlers[Event Handlers]
    end
    
    subgraph "Infrastructure Layer"
        NATS[NATS Cluster]
        EventStore[(Event Store)]
        ProjectionDB[(Projections)]
        Cache[Redis Cache]
    end
    
    CLI --> Gateway
    WebUI --> Gateway
    API_Client --> Gateway
    
    Gateway --> Auth
    Gateway --> RateLimit
    Gateway --> GraphAPI
    
    GraphAPI --> GraphDomain
    EventService --> GraphDomain
    QueryService --> GraphDomain
    ComposeService --> GraphDomain
    
    GraphDomain --> Registry
    GraphDomain --> EventHandlers
    
    EventHandlers --> NATS
    NATS --> EventStore
    EventStore --> ProjectionDB
    QueryService --> Cache
    Cache --> ProjectionDB
```

These architecture diagrams illustrate:
1. **System Architecture**: High-level component organization
2. **Event Flow**: How events propagate through the system
3. **Type Hierarchy**: Trait relationships and inheritance
4. **Component Interactions**: How subsystems communicate
5. **Data Flow**: Movement through graph types
6. **Lifecycle Management**: State transitions for graphs
7. **Composition Patterns**: Ways graphs can be combined
8. **Query Execution**: How queries are processed
9. **Event Sourcing**: CQRS implementation
10. **Deployment View**: Production architecture