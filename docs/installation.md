# Installation Guide

This guide walks you through installing and setting up CIM Graph in your Rust project.

## Requirements

- Rust 1.70 or higher
- Cargo (comes with Rust)

## Basic Installation

Add CIM Graph to your `Cargo.toml`:

```toml
[dependencies]
cim-graph = "0.1.0"
```

Then run:
```bash
cargo build
```

## Feature Flags

CIM Graph supports optional features that can be enabled:

```toml
[dependencies]
# Default features only
cim-graph = "0.1.0"

# With async support
cim-graph = { version = "0.1.0", features = ["async"] }

# All features
cim-graph = { version = "0.1.0", features = ["full"] }
```

### Available Features

- `async` - Enables async/await support (requires tokio)
- `full` - Enables all optional features

## Platform Support

CIM Graph is tested on:
- Linux (Ubuntu 20.04+)
- macOS (10.15+)
- Windows (Windows 10+)

## Verifying Installation

Create a simple test file `src/main.rs`:

```rust
use cim_graph::graphs::workflow::WorkflowGraph;

fn main() {
    let graph = WorkflowGraph::new();
    println!("CIM Graph installed successfully!");
    println!("Graph ID: {:?}", graph.id());
}
```

Run it:
```bash
cargo run
```

## Development Setup

For contributing to CIM Graph:

```bash
# Clone the repository
git clone https://github.com/thecowboyai/cim-graph.git
cd cim-graph

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin cargo-criterion

# Run tests
cargo test

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

## Troubleshooting

### Compilation Errors

If you encounter compilation errors:

1. Ensure you have Rust 1.70+:
   ```bash
   rustc --version
   ```

2. Update your toolchain:
   ```bash
   rustup update stable
   ```

3. Clean and rebuild:
   ```bash
   cargo clean
   cargo build
   ```

### Dependency Conflicts

If you have dependency version conflicts:

1. Update your dependencies:
   ```bash
   cargo update
   ```

2. Check for incompatible versions:
   ```bash
   cargo tree -d
   ```

### Performance Issues

For optimal performance in release builds:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## Next Steps

- Follow the [Quick Start Guide](./quick-start.md)
- Explore the [Examples](https://github.com/thecowboyai/cim-graph/tree/main/examples)
- Read about [Graph Types](./graph-types.md)