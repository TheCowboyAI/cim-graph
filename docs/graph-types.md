# Graph Types Guide

CIM Graph provides specialized graph types for different domains, each optimized for specific use cases while maintaining a consistent API.

## Table of Contents

1. [IPLD Graph](#ipld-graph)
2. [Context Graph](#context-graph)
3. [Workflow Graph](#workflow-graph)
4. [Concept Graph](#concept-graph)
5. [Composed Graph](#composed-graph)
6. [Choosing the Right Graph Type](#choosing-the-right-graph-type)

## IPLD Graph

### Overview

IPLD (InterPlanetary Linked Data) graphs represent content-addressed data structures where nodes are identified by cryptographic hashes (CIDs). This is ideal for:

- Immutable data structures
- Merkle DAGs
- Blockchain and distributed systems
- Content versioning and provenance tracking

### Node Structure

```rust
pub struct IpldNode {
    pub cid: String,           // Content identifier (e.g., "QmXoypiz...")
    pub codec: String,         // Data encoding (e.g., "dag-cbor", "dag-json")
    pub size: u64,            // Size in bytes
    pub data: Option<Vec<u8>>, // Optional raw data
}
```

### Edge Structure

```rust
pub struct IpldEdge {
    pub link_type: String,     // Type of link (e.g., "contains", "references")
    pub path: Option<String>,  // Path within the linked object
    pub metadata: HashMap<String, Value>,
}
```

### Example Usage

```rust
use cim_graph::graphs::IpldGraph;

// Create an IPLD graph
let mut graph = IpldGraph::new();

// Add content nodes
let root_cid = graph.add_cid("QmRoot123", "dag-cbor", 1024)?;
let child1_cid = graph.add_cid("QmChild456", "dag-json", 512)?;
let child2_cid = graph.add_cid("QmChild789", "raw", 256)?;

// Create links between content
graph.add_link(root_cid, child1_cid, "contains", Some("/data/users"))?;
graph.add_link(root_cid, child2_cid, "contains", Some("/data/config"))?;

// Query the Merkle DAG
let children = graph.get_children(root_cid)?;
let parents = graph.get_parents(child1_cid)?;

// Calculate merkle proof
let proof = graph.merkle_proof(root_cid, child1_cid)?;

// Export as IPLD JSON
let ipld_json = graph.to_ipld_json()?;
```

### Advanced Features

```rust
// Create a versioned DAG
let v1 = graph.add_cid("QmVersion1", "dag-cbor", 1024)?;
let v2 = graph.add_cid("QmVersion2", "dag-cbor", 1100)?;
graph.add_link(v2, v1, "previous-version", None)?;

// Find all versions
let versions = graph.find_version_chain(v2)?;

// Validate DAG structure
graph.validate_dag()?; // Ensures no cycles

// Calculate DAG statistics
let stats = graph.dag_stats()?;
println!("Total size: {} bytes", stats.total_size);
println!("Depth: {}", stats.max_depth);
```

## Context Graph

### Overview

Context graphs model Domain-Driven Design (DDD) relationships between aggregates, entities, and value objects. Perfect for:

- Microservice architectures
- Domain modeling
- Hierarchical data (e.g., geographic: Country → Region → City)
- Business object relationships

### Node Structure

```rust
pub struct ContextNode {
    pub aggregate_type: String,    // e.g., "Order", "Customer", "Product"
    pub aggregate_id: Uuid,        // Unique identifier
    pub version: u64,              // For optimistic concurrency
    pub bounded_context: String,   // DDD bounded context
    pub data: Value,               // Domain data
}
```

### Edge Structure

```rust
pub struct ContextEdge {
    pub relationship: String,      // e.g., "owns", "contains", "references"
    pub cardinality: Cardinality, // One-to-one, one-to-many, etc.
    pub direction: Direction,      // Unidirectional or bidirectional
    pub constraints: Vec<Constraint>,
}
```

### Example Usage

```rust
use cim_graph::graphs::{ContextGraph, Cardinality};

// Create a context graph for an e-commerce domain
let mut graph = ContextGraph::new("ecommerce");

// Add aggregates
let customer = graph.add_aggregate("Customer", customer_id, json!({
    "name": "Alice Smith",
    "email": "alice@example.com",
    "tier": "premium"
}))?;

let order = graph.add_aggregate("Order", order_id, json!({
    "total": 150.00,
    "status": "pending",
    "items": 3
}))?;

let address = graph.add_aggregate("Address", address_id, json!({
    "street": "123 Main St",
    "city": "Springfield",
    "postal": "12345"
}))?;

// Define relationships
graph.add_relationship(
    customer, 
    order, 
    "placed",
    Cardinality::OneToMany
)?;

graph.add_relationship(
    order,
    address,
    "ships_to",
    Cardinality::ManyToOne
)?;

// Query relationships
let customer_orders = graph.get_related(customer, "placed")?;
let shipping_address = graph.get_related(order, "ships_to")?.first();

// Navigate bounded contexts
let contexts = graph.list_bounded_contexts();
let sales_aggregates = graph.aggregates_in_context("sales")?;
```

### Advanced Features

```rust
// Aggregate lifecycle
graph.update_aggregate(order, json!({
    "status": "shipped",
    "shipped_at": "2024-01-15T10:00:00Z"
}))?;

// Version management
let version = graph.get_version(order)?;
graph.update_with_version(order, new_data, version)?;

// Consistency boundaries
let boundary = graph.consistency_boundary(order)?;
println!("Aggregates in boundary: {:?}", boundary);

// Export bounded context
let context_data = graph.export_context("sales")?;
```

## Workflow Graph

### Overview

Workflow graphs represent state machines and process flows, ideal for:

- Business process modeling
- State machines
- Saga orchestration
- Approval workflows
- Event-driven architectures

### Node Structure

```rust
pub struct WorkflowNode {
    pub state: String,                    // State name
    pub state_type: StateType,           // Start, End, Regular, Decision
    pub entry_actions: Vec<Action>,       // Actions on entry
    pub exit_actions: Vec<Action>,        // Actions on exit
    pub timeout: Option<Duration>,        // State timeout
    pub metadata: HashMap<String, Value>,
}
```

### Edge Structure

```rust
pub struct WorkflowEdge {
    pub event: String,                    // Triggering event
    pub guard: Option<Guard>,             // Condition for transition
    pub actions: Vec<Action>,             // Actions during transition
    pub priority: u32,                    // For deterministic choices
}
```

### Example Usage

```rust
use cim_graph::graphs::{WorkflowGraph, StateType, Guard};

// Create an order processing workflow
let mut workflow = WorkflowGraph::new("order_processing");

// Define states
let created = workflow.add_state("Created", StateType::Start)?;
let validated = workflow.add_state("Validated", StateType::Regular)?;
let payment_pending = workflow.add_state("PaymentPending", StateType::Regular)?;
let paid = workflow.add_state("Paid", StateType::Regular)?;
let shipped = workflow.add_state("Shipped", StateType::Regular)?;
let delivered = workflow.add_state("Delivered", StateType::End)?;
let cancelled = workflow.add_state("Cancelled", StateType::End)?;

// Define transitions
workflow.add_transition(
    created,
    validated,
    "validate",
    Some(Guard::Expression("items.length > 0"))
)?;

workflow.add_transition(
    validated,
    payment_pending,
    "request_payment",
    None
)?;

workflow.add_transition(
    payment_pending,
    paid,
    "payment_received",
    Some(Guard::Expression("payment.amount >= order.total"))
)?;

workflow.add_transition(
    paid,
    shipped,
    "ship",
    None
)?;

// Add cancel transitions from multiple states
for state in [created, validated, payment_pending, paid] {
    workflow.add_transition(
        state,
        cancelled,
        "cancel",
        Some(Guard::Expression("user.role == 'admin' || order.age < 3600"))
    )?;
}

// Execute workflow
let instance = workflow.create_instance("order-123")?;
workflow.trigger_event(&mut instance, "validate", &context)?;

// Query current state
let current = workflow.get_current_state(&instance)?;
let available_events = workflow.get_available_events(&instance)?;
```

### Advanced Features

```rust
// Parallel states (fork/join)
let picking = workflow.add_state("Picking", StateType::Regular)?;
let packing = workflow.add_state("Packing", StateType::Regular)?;
workflow.add_fork(paid, vec![picking, packing], "process")?;
workflow.add_join(vec![picking, packing], shipped, "complete")?;

// State timeouts and escalation
workflow.set_timeout(payment_pending, Duration::hours(24))?;
workflow.add_timeout_transition(payment_pending, cancelled, "payment_timeout")?;

// Workflow composition
let shipping_workflow = WorkflowGraph::load("shipping_workflow")?;
workflow.embed_subworkflow(shipped, shipping_workflow, "handle_shipping")?;

// Export as state diagram
let mermaid = workflow.to_mermaid()?;
let dot = workflow.to_graphviz()?;
```

## Concept Graph

### Overview

Concept graphs implement semantic reasoning and conceptual spaces, useful for:

- Knowledge representation
- Semantic search
- Ontology modeling
- AI/ML feature spaces
- Recommendation systems

### Node Structure

```rust
pub struct ConceptNode {
    pub concept: String,                        // Concept name
    pub prototype: Option<Prototype>,           // Prototypical instance
    pub attributes: HashMap<String, f64>,       // Dimensional values
    pub quality_dimensions: Vec<Dimension>,     // Quality space
    pub instances: Vec<Instance>,               // Example instances
}
```

### Edge Structure

```rust
pub struct ConceptEdge {
    pub relation: SemanticRelation,    // is_a, part_of, similar_to, etc.
    pub strength: f64,                 // Relation strength [0.0, 1.0]
    pub bidirectional: bool,           // Whether relation goes both ways
    pub evidence: Vec<Evidence>,       // Supporting evidence
}
```

### Example Usage

```rust
use cim_graph::graphs::{ConceptGraph, SemanticRelation, Dimension};

// Create a concept graph for vehicles
let mut graph = ConceptGraph::new("vehicle_ontology");

// Define quality dimensions
let dimensions = vec![
    Dimension::new("size", 0.0, 10.0),
    Dimension::new("speed", 0.0, 300.0),
    Dimension::new("capacity", 1.0, 500.0),
    Dimension::new("efficiency", 0.0, 100.0),
];

// Add concepts with prototypes
let vehicle = graph.add_concept("Vehicle", dimensions.clone())?;
let car = graph.add_concept_with_prototype("Car", &[
    ("size", 5.0),
    ("speed", 150.0),
    ("capacity", 5.0),
    ("efficiency", 30.0),
])?;

let truck = graph.add_concept_with_prototype("Truck", &[
    ("size", 8.0),
    ("speed", 100.0),
    ("capacity", 2.0),
    ("efficiency", 15.0),
])?;

let bicycle = graph.add_concept_with_prototype("Bicycle", &[
    ("size", 2.0),
    ("speed", 30.0),
    ("capacity", 1.0),
    ("efficiency", 95.0),
])?;

// Define semantic relations
graph.add_relation(car, vehicle, SemanticRelation::IsA, 1.0)?;
graph.add_relation(truck, vehicle, SemanticRelation::IsA, 1.0)?;
graph.add_relation(bicycle, vehicle, SemanticRelation::IsA, 1.0)?;

// Add more specific relations
let sedan = graph.add_concept("Sedan", dimensions)?;
graph.add_relation(sedan, car, SemanticRelation::IsA, 1.0)?;

// Find similar concepts
let similar_to_car = graph.find_similar(car, 0.7)?; // threshold 0.7
let between = graph.conceptual_between(bicycle, truck)?; // interpolate

// Classify new instance
let instance = Instance::new(&[
    ("size", 4.5),
    ("speed", 120.0),
    ("capacity", 4.0),
    ("efficiency", 35.0),
]);
let classification = graph.classify_instance(&instance)?;
```

### Advanced Features

```rust
// Reasoning and inference
let inferences = graph.infer_relations()?;
for (concept1, concept2, relation, confidence) in inferences {
    println!("{} {} {} (confidence: {})", 
        concept1, relation, concept2, confidence);
}

// Semantic similarity metrics
let similarity = graph.semantic_similarity(car, truck)?;
let distance = graph.conceptual_distance(car, bicycle)?;

// Concept clustering
let clusters = graph.cluster_concepts(3)?; // 3 clusters
for (i, cluster) in clusters.iter().enumerate() {
    println!("Cluster {}: {:?}", i, cluster);
}

// Export as ontology
let owl = graph.to_owl()?;
let json_ld = graph.to_json_ld()?;
```

## Composed Graph

### Overview

Composed graphs allow combining multiple graph types into a unified structure, enabling:

- Cross-domain queries
- Multi-aspect modeling
- Graph federation
- Heterogeneous graph analysis

### Example Usage

```rust
use cim_graph::compose_graphs;
use cim_graph::composition::{CompositionStrategy, MappingRule};

// Create individual graphs
let ipld_graph = create_ipld_graph()?;
let context_graph = create_context_graph()?;
let workflow_graph = create_workflow_graph()?;
let concept_graph = create_concept_graph()?;

// Compose graphs with mappings
let composed = compose_graphs()
    .add_graph("data", ipld_graph)
    .add_graph("domain", context_graph)
    .add_graph("process", workflow_graph)
    .add_graph("knowledge", concept_graph)
    .with_strategy(CompositionStrategy::Union)
    .with_mapping("data", "domain", |ipld_node, context_node| {
        // Map IPLD CIDs to aggregate IDs
        ipld_node.cid == context_node.data.get("ipld_cid")?.as_str()?
    })
    .with_mapping("domain", "process", |context_node, workflow_node| {
        // Map aggregates to workflow states
        context_node.aggregate_type == workflow_node.metadata.get("aggregate_type")?
    })
    .with_mapping("domain", "knowledge", |context_node, concept_node| {
        // Map domain objects to concepts
        context_node.aggregate_type == concept_node.concept
    })
    .compose()?;

// Cross-graph queries
let results = composed
    .query()
    .start_from_graph("domain", customer_id)
    .follow_path(&[
        ("domain", "placed"),      // Customer placed Order
        ("process", "in_state"),    // Order in Workflow state
        ("data", "stored_as"),      // Order stored as IPLD
    ])
    .execute()?;

// Multi-aspect analysis
let analysis = composed.analyze_node(order_id)?;
println!("Domain context: {:?}", analysis.domain_context);
println!("Current workflow state: {:?}", analysis.workflow_state);
println!("Storage CID: {:?}", analysis.storage_cid);
println!("Semantic classification: {:?}", analysis.concepts);
```

## Choosing the Right Graph Type

### Decision Matrix

| Use Case | Recommended Graph Type |
|----------|----------------------|
| Content-addressed storage | IPLD Graph |
| Blockchain/Merkle trees | IPLD Graph |
| Domain modeling | Context Graph |
| Microservice relationships | Context Graph |
| Business processes | Workflow Graph |
| State machines | Workflow Graph |
| Knowledge representation | Concept Graph |
| Semantic search | Concept Graph |
| Multi-domain systems | Composed Graph |

### Performance Characteristics

| Graph Type | Node Lookup | Edge Traversal | Memory Usage | Best For |
|------------|------------|----------------|--------------|----------|
| IPLD | O(1) by CID | O(degree) | Low | Large DAGs |
| Context | O(1) by ID | O(degree) | Medium | Complex domains |
| Workflow | O(1) by state | O(transitions) | Low | State machines |
| Concept | O(1) by name | O(relations) | High | Reasoning |
| Composed | O(graphs) | O(graphs × degree) | High | Integration |

### Migration Between Types

```rust
// Convert Context Graph to Workflow Graph
let workflow = WorkflowGraph::from_context_graph(&context_graph, |node| {
    // Map aggregates to states
    WorkflowState {
        name: format!("{}_{}", node.aggregate_type, node.status),
        state_type: StateType::Regular,
    }
})?;

// Convert Concept Graph to Context Graph
let context = ContextGraph::from_concept_graph(&concept_graph, |concept| {
    // Map concepts to aggregates
    ContextAggregate {
        aggregate_type: concept.concept,
        bounded_context: "knowledge",
        data: concept.to_json(),
    }
})?;
```