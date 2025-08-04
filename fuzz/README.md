# Fuzzing CIM Graph

This directory contains fuzz tests for the CIM Graph library using cargo-fuzz and libFuzzer.

## Prerequisites

Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

### Fuzz Graph Operations
Tests graph construction, modification, and querying:
```bash
cargo fuzz run fuzz_graph_operations
```

### Fuzz Serialization
Tests serialization/deserialization robustness:
```bash
cargo fuzz run fuzz_serialization
```

### Fuzz Algorithms
Tests graph algorithms with random graph structures:
```bash
cargo fuzz run fuzz_algorithms
```

## Running with Options

### Run for a specific duration
```bash
cargo fuzz run fuzz_graph_operations -- -max_total_time=300
```

### Run with more threads
```bash
cargo fuzz run fuzz_graph_operations -- -jobs=4
```

### Run with a corpus
```bash
cargo fuzz run fuzz_graph_operations corpus/
```

## Analyzing Crashes

If a crash is found, it will be saved in `fuzz/artifacts/`. To reproduce:
```bash
cargo fuzz run fuzz_graph_operations fuzz/artifacts/fuzz_graph_operations/crash-<hash>
```

## Coverage

To generate coverage information:
```bash
cargo fuzz coverage fuzz_graph_operations
cargo cov -- show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_graph_operations \
    --format=html \
    -instr-profile=fuzz/coverage/fuzz_graph_operations/coverage.profdata \
    > coverage.html
```

## What We're Testing

### Graph Operations Fuzzer
- Random node and edge additions
- Node and edge removals
- Graph queries
- Bulk operations
- Stress testing with large operation sequences

### Serialization Fuzzer
- Deserializing arbitrary data
- Round-trip serialization
- Handling malformed JSON
- Large graph serialization
- Special characters and edge cases

### Algorithms Fuzzer
- Pathfinding on random graphs
- Traversal algorithms (DFS, BFS)
- Topological sorting
- Graph metrics calculation
- Special graph patterns (chains, stars, complete graphs, trees)

## Adding New Fuzz Targets

1. Add the new target to `fuzz/Cargo.toml`
2. Create the fuzz target in `fuzz/fuzz_targets/`
3. Follow the pattern of existing fuzzers
4. Document what the fuzzer tests