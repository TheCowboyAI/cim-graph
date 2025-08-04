# CIM Graph Examples

This directory contains comprehensive examples demonstrating the various graph types and features provided by the CIM Graph library.

## Running Examples

To run any example:

```bash
cargo run --example <example_name>
```

## Available Examples

### 1. IPLD Graph Example (`ipld_example.rs`)

Demonstrates content-addressed graphs similar to IPFS/IPLD:

```bash
cargo run --example ipld_example
```

Features shown:
- Creating content-addressed nodes with CIDs
- Building directory-like structures
- Traversing DAG structures
- Content retrieval by CID

### 2. Context Graph Example (`context_example.rs`)

Shows Domain-Driven Design with bounded contexts:

```bash
cargo run --example context_example
```

Features shown:
- Defining bounded contexts
- Creating aggregates, entities, and value objects
- Domain events and relationships
- Boundary validation
- Business invariants

### 3. Workflow Graph Example (`workflow_example.rs`)

Demonstrates state machines and process workflows:

```bash
cargo run --example workflow_example
```

Features shown:
- Creating workflow states and transitions
- Processing events through the workflow
- Parallel state support
- Workflow validation
- Transition history tracking

### 4. Concept Graph Example (`concept_example.rs`)

Shows semantic reasoning and knowledge representation:

```bash
cargo run --example concept_example
```

Features shown:
- Building taxonomies and ontologies
- Semantic relationships (isA, hasProperty, etc.)
- Inference rules (simplified)
- Concept queries
- Knowledge graph traversal

### 5. Composed Graph Example (`composed_example.rs`)

Demonstrates multi-layer graph composition:

```bash
cargo run --example composed_example
```

Features shown:
- Combining multiple graph types
- Cross-layer connections
- Layer-specific constraints
- Healthcare system modeling example

## Key Concepts

All graph types are built on top of `petgraph` and provide:
- Event-driven architecture
- Type safety with Rust's trait system
- Efficient graph operations
- Extensible design

## Creating Your Own Examples

To create a new example:

1. Add a new file in the `examples/` directory
2. Import the necessary graph types from `cim_graph`
3. Create a `main()` function demonstrating your use case
4. Add the example to `examples/Cargo.toml` if needed

## More Information

For detailed API documentation, run:

```bash
cargo doc --open
```

For questions or issues, please visit the project repository.