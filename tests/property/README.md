# Property-Based Tests for CIM Graph

This directory contains comprehensive property-based tests using the `proptest` framework to thoroughly test graph properties and invariants.

## Structure

- `generators.rs` - Custom strategies for generating graph structures
  - Node and edge generators
  - Graph structure generators (trees, cycles, disconnected graphs)
  - Pathological case generators
  
- `graph_invariants.rs` - Tests for fundamental graph properties
  - Node/edge count invariants
  - Adjacency consistency
  - Removal operations
  
- `algebraic_properties.rs` - Tests for algebraic properties
  - Commutativity and associativity of operations
  - Identity elements
  - Graph operations (union, intersection)
  
- `serialization_properties.rs` - Serialization round-trip tests
  - JSON serialization
  - Binary serialization
  - Metadata preservation
  
- `algorithm_properties.rs` - Algorithm correctness tests
  - BFS/DFS equivalence
  - Path finding properties
  - Cycle detection
  - Connected components
  
- `pathological_cases.rs` - Edge case and stress tests
  - Empty graphs
  - Single node graphs
  - Highly connected nodes
  - Extreme values

## Running Tests

```bash
# Run all property tests
cargo test --test property_tests

# Run specific test module
cargo test --test property_tests -- graph_invariants

# Run with more test cases
PROPTEST_CASES=1000 cargo test --test property_tests

# Run with verbose output
cargo test --test property_tests -- --nocapture
```

## Key Properties Tested

1. **Invariants**
   - Adding a node increases count by 1
   - Removing a node removes all its edges
   - Node and edge IDs are unique

2. **Algebraic Properties**
   - Graph union is commutative and associative
   - Empty graph is identity for union
   - Graph intersection is commutative

3. **Serialization**
   - Round-trip preservation of structure
   - Handling of special characters
   - Large graph serialization

4. **Algorithms**
   - BFS and DFS find same reachable nodes
   - Shortest path satisfies triangle inequality
   - Tree structures have no cycles

## Note on Compilation

The tests use `cim_graph::core::graph::BasicGraph` for testing purposes. If compilation issues arise, ensure that:

1. The `BasicGraph` type is accessible via the full module path
2. All required traits (Node, Edge) are imported
3. The `proptest` and other dev dependencies are properly installed