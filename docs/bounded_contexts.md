# CIM Graph - Bounded Contexts and Relationships

This document defines the bounded contexts within the CIM Graph system and their relationships. These contexts represent distinct domains within the event-driven architecture, each with its own aggregate roots, events, and responsibilities.

## Bounded Contexts

### 1. IPLD Context
**Purpose**: Manages content-addressed data storage and linking
**Aggregate Root**: IpldChainAggregate
**Domain Events**:
- CidAdded
- CidRemoved
- CidLinkAdded
- CidMetadataUpdated
- ChainValidated
- ChainPinned

**Responsibilities**:
- Store and retrieve content-addressed data
- Maintain links between CIDs
- Validate IPLD chains
- Handle pinning/unpinning
- Generate CIDs for event payloads

**Relationships**:
- Provides CID generation service to all other contexts
- Stores event payloads as IPLD blocks
- Referenced by Context graphs for data persistence

### 2. Context Context (Data Context)
**Purpose**: Manages data schemas, transformations, and mappings
**Aggregate Root**: ContextGraph
**Domain Events**:
- ContextDefined
- SchemaAdded
- TransformAdded
- MappingCreated
- ValidationRuleAdded

**Responsibilities**:
- Define data contexts and schemas
- Create transformation pipelines
- Map between different data representations
- Validate data against schemas

**Relationships**:
- Uses IPLD Context for schema storage
- Referenced by Workflow Context for data validation
- Provides data transformation services to Composed Context

### 3. Workflow Context
**Purpose**: Manages state machines, business processes, and transitions
**Aggregate Root**: WorkflowGraph
**Domain Events**:
- WorkflowDefined
- StateAdded
- TransitionAdded
- ActionTriggered
- StateTransitioned
- WorkflowCompleted

**Responsibilities**:
- Define workflow states and transitions
- Execute state machines
- Track workflow instances
- Enforce business rules
- Trigger actions on state changes

**Relationships**:
- Uses Context Context for data validation
- References Concept Context for decision logic
- Orchestrated by Composed Context

### 4. Concept Context
**Purpose**: Manages domain knowledge, ontologies, and reasoning
**Aggregate Root**: ConceptGraph
**Domain Events**:
- ConceptDefined
- RelationAdded
- PropertiesAdded
- InferenceRuleAdded
- CategoryAssigned

**Responsibilities**:
- Build domain ontologies
- Define concept relationships
- Perform semantic reasoning
- Calculate semantic distances
- Infer new relationships

**Relationships**:
- Provides reasoning capabilities to Workflow Context
- Used by Composed Context for cross-domain reasoning
- Stores concepts in IPLD Context

### 5. Composed Context
**Purpose**: Orchestrates and integrates multiple graph types
**Aggregate Root**: ComposedGraph
**Domain Events**:
- SubGraphAdded
- CrossGraphLinkCreated
- CompositionRuleAdded
- IntegrationExecuted
- SubGraphRemoved

**Responsibilities**:
- Compose multiple graphs into cohesive systems
- Create cross-graph links and dependencies
- Execute integration patterns
- Manage graph namespaces
- Coordinate cross-context operations

**Relationships**:
- Orchestrates all other contexts
- Creates links between different graph types
- Manages graph composition rules

## Context Map

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Composed      │────▶│    Workflow     │────▶│    Context      │
│   Context       │     │    Context      │     │    Context      │
│ (Orchestrator)  │     │ (State Machine) │     │ (Data Schema)   │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                         │
         │                       ▼                         │
         │              ┌─────────────────┐               │
         └─────────────▶│    Concept      │◀──────────────┘
                        │    Context      │
                        │  (Reasoning)    │
                        └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │     IPLD        │
                        │    Context      │
                        │   (Storage)     │
                        └─────────────────┘
```

## Integration Patterns

### 1. Upstream-Downstream (U/D)
- IPLD Context is upstream to all other contexts (provides storage)
- Composed Context is downstream from all other contexts (consumes their services)

### 2. Shared Kernel
- All contexts share the core Event and GraphProjection abstractions
- Common error types (GraphError) are shared across contexts

### 3. Customer-Supplier
- IPLD Context supplies CID generation to all contexts
- Context Context supplies data validation to Workflow Context
- Concept Context supplies reasoning to Workflow and Composed Contexts

### 4. Anti-Corruption Layer
- Each context validates incoming events before processing
- Projection engines act as anti-corruption layers
- State machines enforce valid transitions

## Event Flow Between Contexts

### Cross-Context Event Patterns

1. **Workflow triggers Context validation**:
   ```
   Workflow.StateTransitioned -> Context.ValidateData -> Context.ValidationCompleted
   ```

2. **Concept reasoning affects Workflow decisions**:
   ```
   Concept.InferenceCompleted -> Workflow.DecisionMade -> Workflow.StateTransitioned
   ```

3. **Composed orchestrates multiple contexts**:
   ```
   Composed.IntegrationTriggered -> [Workflow.Execute, Concept.Reason, Context.Transform]
   ```

4. **All contexts store in IPLD**:
   ```
   *.EventEmitted -> IPLD.GenerateCID -> IPLD.CidAdded
   ```

## Context Boundaries

### Strong Boundaries
- Each context has its own aggregate root
- Events are the only way to communicate between contexts
- No direct references between context internals
- Each context maintains its own projections

### Weak Boundaries
- Shared event infrastructure (NATS JetStream)
- Common projection engine abstraction
- Shared error types for system-wide concerns

## Future Considerations

1. **Context Evolution**:
   - Contexts can evolve independently
   - New contexts can be added without affecting existing ones
   - Event schemas support versioning

2. **Scaling Strategies**:
   - Each context can be deployed separately
   - Event streams can be partitioned by context
   - Projections can be cached per context

3. **Testing Boundaries**:
   - Each context has its own test suite
   - Integration tests verify cross-context flows
   - Contract tests ensure event compatibility