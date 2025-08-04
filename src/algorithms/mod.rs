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
//! ```rust
//! use cim_graph::{GraphBuilder, Node, Edge};
//! use cim_graph::algorithms::{shortest_path, bfs};
//!
//! # fn main() -> cim_graph::Result<()> {
//! let mut graph = GraphBuilder::new().build();
//! let a = graph.add_node(Node::new("A", "city"))?;
//! let b = graph.add_node(Node::new("B", "city"))?;
//! let c = graph.add_node(Node::new("C", "city"))?;
//!
//! graph.add_edge(a, b, Edge::with_weight(10.0))?;
//! graph.add_edge(b, c, Edge::with_weight(20.0))?;
//!
//! // Find shortest path
//! let path = shortest_path(&graph, a, c)?;
//! assert_eq!(path.unwrap().1, vec![a, b, c]);
//!
//! // Breadth-first traversal
//! let visited = bfs(&graph, a)?;
//! assert_eq!(visited.len(), 3);
//! # Ok(())
//! # }
//! ```

pub mod pathfinding;
pub mod traversal;
pub mod metrics;

pub use pathfinding::{shortest_path, all_paths};
pub use traversal::{dfs, bfs, topological_sort};
pub use metrics::{centrality, clustering_coefficient};