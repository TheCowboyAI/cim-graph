# Contributing to CIM Graph

Thank you for your interest in contributing to CIM Graph! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [How to Contribute](#how-to-contribute)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Documentation](#documentation)
8. [Pull Request Process](#pull-request-process)
9. [Release Process](#release-process)

## Code of Conduct

This project adheres to the Contributor Covenant [Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/cim-graph.git
   cd cim-graph
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/thecowboyai/cim-graph.git
   ```
4. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo
- Git

### Building the Project

```bash
# Build the project
cargo build

# Build with all features
cargo build --all-features

# Build for release
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests with all features
cargo test --all-features

# Run documentation tests
cargo test --doc
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_name

# Compare with baseline
cargo bench -- --save-baseline my-baseline
```

### Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues. When creating a bug report, include:

1. **Clear title and description**
2. **Steps to reproduce**
3. **Expected behavior**
4. **Actual behavior**
5. **System information** (OS, Rust version)
6. **Minimal reproducible example**

Example:
```rust
// Minimal code that reproduces the issue
use cim_graph::{GraphBuilder, Node};

fn main() {
    let mut graph = GraphBuilder::new().build();
    // Code that triggers the bug
}
```

### Suggesting Enhancements

Enhancement suggestions are welcome! Please include:

1. **Use case description**
2. **Proposed solution**
3. **Alternative solutions considered**
4. **Potential impact on existing code**

### Contributing Code

1. **Check existing issues** for related work
2. **Discuss major changes** by opening an issue first
3. **Write tests** for new functionality
4. **Update documentation** as needed
5. **Follow coding standards** (see below)

## Coding Standards

### Rust Style Guide

We follow the standard Rust style guide with some additions:

```rust
// Use explicit imports
use cim_graph::{Graph, Node, Edge};

// Not
use cim_graph::*;

// Document public APIs
/// Creates a new graph with the specified capacity.
///
/// # Arguments
///
/// * `node_capacity` - Initial capacity for nodes
/// * `edge_capacity` - Initial capacity for edges
///
/// # Example
///
/// ```
/// use cim_graph::GraphBuilder;
///
/// let graph = GraphBuilder::new()
///     .with_capacity(100, 200)
///     .build();
/// ```
pub fn with_capacity(node_capacity: usize, edge_capacity: usize) -> Self {
    // Implementation
}
```

### Error Handling

```rust
// Use Result types consistently
pub fn operation() -> Result<Value, GraphError> {
    // Don't use unwrap in library code
    let value = something()?;
    
    // Provide context for errors
    other_operation()
        .map_err(|e| GraphError::OperationFailed {
            operation: "other_operation",
            cause: Box::new(e),
        })?;
    
    Ok(value)
}
```

### Performance Considerations

```rust
// Avoid unnecessary allocations
// Good
pub fn get_neighbors(&self, node: NodeId) -> &[NodeId] {
    &self.adjacency[node]
}

// Avoid
pub fn get_neighbors(&self, node: NodeId) -> Vec<NodeId> {
    self.adjacency[node].clone()
}
```

### Generic Constraints

```rust
// Be explicit about trait bounds
pub fn algorithm<G, N, E>(graph: &G) -> Result<Vec<NodeId>>
where
    G: Graph<Node = N, Edge = E>,
    N: Node + Clone + Debug,
    E: Edge + Weighted,
{
    // Implementation
}
```

## Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();
        let node = Node::new("test", "type");
        let id = graph.add_node(node.clone()).unwrap();
        
        assert_eq!(graph.get_node(id).unwrap().data, node.data);
    }
    
    #[test]
    #[should_panic(expected = "node not found")]
    fn test_invalid_node_access() {
        let graph = Graph::new();
        graph.get_node(NodeId::from(999)).unwrap();
    }
}
```

### Integration Tests

Place integration tests in `tests/`:

```rust
// tests/graph_operations.rs
use cim_graph::{GraphBuilder, Node, Edge};

#[test]
fn test_complex_workflow() {
    let mut graph = GraphBuilder::new().build();
    // Test complete workflows
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_graph_invariants(
        nodes in prop::collection::vec(any::<String>(), 0..100)
    ) {
        let mut graph = Graph::new();
        for node in nodes {
            graph.add_node(Node::new(node, "test")).unwrap();
        }
        
        // Test invariants hold
        assert!(graph.validate().is_ok());
    }
}
```

### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_add_nodes(c: &mut Criterion) {
    c.bench_function("add 1000 nodes", |b| {
        b.iter(|| {
            let mut graph = Graph::new();
            for i in 0..1000 {
                graph.add_node(Node::new(i, "bench")).unwrap();
            }
            black_box(graph);
        });
    });
}

criterion_group!(benches, bench_add_nodes);
criterion_main!(benches);
```

## Documentation

### Code Documentation

All public APIs must be documented:

```rust
/// A directed graph implementation optimized for IPLD data structures.
///
/// This graph type is specifically designed for content-addressed data
/// where nodes represent IPLD objects identified by CIDs.
///
/// # Example
///
/// ```
/// use cim_graph::graphs::IpldGraph;
///
/// let mut graph = IpldGraph::new();
/// let cid = graph.add_cid("QmHash...", "dag-cbor", 1024)?;
/// ```
pub struct IpldGraph {
    // Fields
}
```

### Module Documentation

Each module should have documentation:

```rust
//! Graph algorithms for pathfinding and analysis.
//!
//! This module provides efficient implementations of common graph algorithms
//! including shortest path, traversal, and connectivity analysis.
//!
//! # Example
//!
//! ```
//! use cim_graph::algorithms::{dijkstra, bfs};
//! ```
```

### Examples

Add examples to `examples/`:

```rust
//! examples/shortest_path.rs
//! Demonstrates finding shortest paths in different graph types.

use cim_graph::{GraphBuilder, Node, Edge, algorithms::dijkstra};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = GraphBuilder::new().build();
    
    // Build graph...
    
    let paths = dijkstra(&graph, start, None)?;
    
    Ok(())
}
```

## Pull Request Process

1. **Update your fork**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run tests and checks**:
   ```bash
   cargo test
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```

3. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add new graph algorithm"
   ```

   Follow conventional commits:
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation changes
   - `test:` Test additions/changes
   - `perf:` Performance improvements
   - `refactor:` Code refactoring
   - `chore:` Maintenance tasks

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Create Pull Request**:
   - Use a clear title and description
   - Reference any related issues
   - Include test results
   - Add examples if applicable

### PR Review Process

1. **Automated checks** must pass
2. **Code review** by maintainers
3. **Address feedback** promptly
4. **Squash commits** if requested
5. **Merge** when approved

## Release Process

Releases follow semantic versioning:

- **MAJOR**: Breaking API changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

### Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Tag release: `git tag -a v0.1.0 -m "Release v0.1.0"`
- [ ] Push tag: `git push upstream v0.1.0`
- [ ] Publish to crates.io: `cargo publish`

## Getting Help

- **Discord**: Join our community server
- **GitHub Issues**: For bugs and features
- **Discussions**: For questions and ideas
- **Email**: maintainers@cim-graph.dev

## Recognition

Contributors will be recognized in:
- CHANGELOG.md
- GitHub contributors page
- Project documentation

Thank you for contributing to CIM Graph!