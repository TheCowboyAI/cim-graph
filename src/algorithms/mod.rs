//! Graph algorithms for pathfinding, analysis, and metrics
//!
//! This module provides efficient implementations of common graph algorithms
//! that work with all CIM Graph types. All algorithms are generic over the
//! Graph trait, allowing them to work seamlessly with any graph implementation.
//!
//! # Categories
//!
//! ## Pathfinding
//! - [`shortest_path`] - Dijkstra's algorithm for shortest paths
//! - [`all_paths`] - Find all paths between two nodes
//! - A* search (with heuristic function)
//! - Bellman-Ford (handles negative weights)
//!
//! ## Traversal
//! - [`bfs`] - Breadth-first search
//! - [`dfs`] - Depth-first search
//! - [`topological_sort`] - Order nodes in a DAG
//!
//! ## Analysis & Metrics
//! - [`centrality`] - Various centrality measures (degree, betweenness, etc.)
//! - [`clustering_coefficient`] - Measure graph clustering
//! - Connected components
//! - Cycle detection
//!
//! # Example
//!
//! Algorithms work with graph projections built from events:
//!
//! ```rust,ignore
//! use cim_graph::algorithms::{shortest_path, bfs};
//!
//! // Build a projection from events (see core::ProjectionEngine)
//! let projection = engine.project(events);
//!
//! // Use algorithms on the projection
//! let path = shortest_path(&projection, "A", "C")?;
//! let visited = bfs(&projection, "A")?;
//! ```
//!
//! For a complete example with event sourcing, see the crate-level documentation.

pub mod pathfinding;
pub mod traversal;
pub mod metrics;

pub use pathfinding::{shortest_path, all_paths};
pub use traversal::{dfs, bfs, topological_sort};
pub use metrics::{centrality, clustering_coefficient};