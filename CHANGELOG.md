# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2025-10-01

### Changed
- Replaced deprecated `cim-subject` with `cim-domain` subject module (v0.7.8).
- Updated docs, examples, and tests to reference `cim-domain`’s subject module.

### Removed
- Removed dependency on `cim-subject`.

### Dependency
- `cim-domain` pinned to tag `v0.7.8` over SSH (`ssh://git@github.com/TheCowboyAI/cim-domain.git`).

## [0.1.0] - 2025-08-04

### Added
- Initial release of CIM Graph library
- Core graph abstraction with event-driven architecture
- Five specialized graph types:
  - `IpldGraph` - Content-addressed graph for IPLD data structures
  - `ContextGraph` - Domain-driven design bounded contexts
  - `WorkflowGraph` - State machine workflows
  - `ConceptGraph` - Semantic reasoning and knowledge graphs
  - `ComposedGraph` - Multi-layer graph composition
- Graph algorithms:
  - Pathfinding (shortest path, all paths)
  - Traversal (DFS, BFS, topological sort)
  - Analysis (centrality, clustering coefficient)
- Serialization support:
  - JSON format with pretty printing
  - Binary format for efficiency
  - Schema evolution support
- Comprehensive test suite with 88+ tests
- Performance benchmarks
- Full documentation and examples

### Architecture
- Built on petgraph v0.8 for proven graph algorithms
- Event-driven design for extensibility
- Type-safe API with strong Rust types
- Zero-copy operations where possible
- Async-ready architecture (feature flag)

### Dependencies
- `petgraph` 0.8 - Core graph algorithms
- `serde` 1.0 - Serialization framework
- `uuid` 1.6 - Unique identifiers
- `chrono` 0.4 - Timestamp handling
- `thiserror` 1.0 - Error handling

[Unreleased]: https://github.com/thecowboyai/cim-graph/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/thecowboyai/cim-graph/releases/tag/v0.5.0
[0.1.0]: https://github.com/thecowboyai/cim-graph/releases/tag/v0.1.0
