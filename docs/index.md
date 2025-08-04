# CIM Graph Documentation

Welcome to the CIM Graph documentation! This site contains comprehensive guides and references for using the CIM Graph library.

## Quick Links

- [API Documentation](https://docs.rs/cim-graph) - Auto-generated from source
- [GitHub Repository](https://github.com/thecowboyai/cim-graph) - Source code and issues
- [Crates.io](https://crates.io/crates/cim-graph) - Package registry

## Documentation Sections

### Getting Started
- [Installation](./installation.md) - How to add CIM Graph to your project
- [Quick Start](./quick-start.md) - Your first graph in 5 minutes
- [Examples](https://github.com/thecowboyai/cim-graph/tree/main/examples) - Working code examples

### User Guides
- [Graph Types](./graph-types.md) - Choosing the right graph for your use case
- [API Reference](./api.md) - Core APIs and traits
- [Algorithms](./algorithms.md) - Available graph algorithms
- [Serialization](./serialization.md) - Saving and loading graphs

### Advanced Topics
- [Architecture](./architecture.md) - Internal design and extension points
- [Performance](./performance.md) - Optimization strategies
- [Best Practices](./best-practices.md) - Patterns and anti-patterns

### Migration
- [Migration Guide](./migration-guide.md) - Migrating from other graph libraries
- [Changelog](https://github.com/thecowboyai/cim-graph/blob/main/CHANGELOG.md) - Version history

## Graph Types Overview

CIM Graph provides five specialized graph types:

| Type | Use Case | Key Features |
|------|----------|--------------|
| **IpldGraph** | Content-addressed data | IPFS/IPLD compatible, immutable blocks |
| **ContextGraph** | Domain modeling | DDD patterns, bounded contexts |
| **WorkflowGraph** | State machines | Transitions, validation, parallel flows |
| **ConceptGraph** | Knowledge graphs | Semantic relations, inference |
| **ComposedGraph** | Multi-domain | Layer management, cross-graph queries |

## Quick Example

```rust
use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};

// Create a simple workflow
let mut workflow = WorkflowGraph::new();

// Add states
let start = WorkflowNode::new("start", "Start Process", StateType::Start);
let process = WorkflowNode::new("process", "Process Data", StateType::Normal);
let end = WorkflowNode::new("end", "Complete", StateType::End);

workflow.add_state(start)?;
workflow.add_state(process)?;
workflow.add_state(end)?;

// Add transitions
workflow.add_transition("start", "process", "begin")?;
workflow.add_transition("process", "end", "complete")?;

// Use the workflow
workflow.start("order_123")?;
workflow.process_event("begin")?;
```

## Community

- [GitHub Issues](https://github.com/thecowboyai/cim-graph/issues) - Bug reports and feature requests
- [Discussions](https://github.com/thecowboyai/cim-graph/discussions) - Questions and ideas
- [Contributing](https://github.com/thecowboyai/cim-graph/blob/main/CONTRIBUTING.md) - How to contribute

## License

CIM Graph is dual-licensed under MIT and Apache 2.0. See [LICENSE](https://github.com/thecowboyai/cim-graph/blob/main/LICENSE) for details.