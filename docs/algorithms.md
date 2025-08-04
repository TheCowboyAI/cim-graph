# Algorithms Guide

CIM Graph provides a comprehensive suite of graph algorithms optimized for different use cases. All algorithms are generic over the Graph trait, allowing them to work with any graph type.

## Table of Contents

1. [Pathfinding Algorithms](#pathfinding-algorithms)
2. [Traversal Algorithms](#traversal-algorithms)
3. [Analysis Algorithms](#analysis-algorithms)
4. [Pattern Matching](#pattern-matching)
5. [Graph Metrics](#graph-metrics)
6. [Custom Algorithms](#custom-algorithms)
7. [Performance Tips](#performance-tips)

## Pathfinding Algorithms

### Dijkstra's Algorithm

Find the shortest path between nodes using non-negative edge weights.

```rust
use cim_graph::algorithms::dijkstra;

let graph = create_graph()?;
let start = NodeId::from("A");
let goal = NodeId::from("F");

// Find shortest path
let result = dijkstra(&graph, start, Some(goal))?;

if let Some((distance, path)) = result.get(&goal) {
    println!("Shortest distance: {}", distance);
    println!("Path: {:?}", path);
}

// Find shortest paths to all reachable nodes
let all_paths = dijkstra(&graph, start, None)?;
for (node, (distance, path)) in all_paths {
    println!("{}: distance={}, path={:?}", node, distance, path);
}
```

### A* Algorithm

Heuristic-based pathfinding for better performance when you have domain knowledge.

```rust
use cim_graph::algorithms::astar;

// Define a heuristic function (e.g., Euclidean distance for geographic data)
fn heuristic(a: &Node, b: &Node) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

// Find path with A*
let result = astar(&graph, start, goal, heuristic)?;

match result {
    Some((cost, path)) => {
        println!("Found path with cost {}: {:?}", cost, path);
    }
    None => {
        println!("No path exists");
    }
}
```

### Bellman-Ford Algorithm

Handle negative edge weights and detect negative cycles.

```rust
use cim_graph::algorithms::bellman_ford;

let result = bellman_ford(&graph, start)?;

match result {
    Ok(distances) => {
        for (node, distance) in distances {
            println!("{}: {}", node, distance);
        }
    }
    Err(NegativeCycle) => {
        println!("Graph contains a negative cycle!");
    }
}
```

### All Pairs Shortest Paths

Find shortest paths between all pairs of nodes.

```rust
use cim_graph::algorithms::floyd_warshall;

let distances = floyd_warshall(&graph)?;

// Query distance between any two nodes
let distance = distances.get(&(node1, node2));
```

## Traversal Algorithms

### Breadth-First Search (BFS)

Level-order traversal of the graph.

```rust
use cim_graph::algorithms::{bfs, BfsEvent};

// Simple BFS traversal
let visited = bfs(&graph, start)?;
println!("Visited {} nodes", visited.len());

// BFS with callbacks
bfs_with_callback(&graph, start, |event| {
    match event {
        BfsEvent::Discover(node) => println!("Discovered: {}", node),
        BfsEvent::Examine(edge) => println!("Examining edge: {:?}", edge),
        BfsEvent::Finish(node) => println!("Finished: {}", node),
    }
})?;

// Find all nodes at a specific distance
let nodes_at_distance = bfs_distance(&graph, start, 3)?;
```

### Depth-First Search (DFS)

Explore as far as possible along each branch.

```rust
use cim_graph::algorithms::{dfs, DfsEvent};

// DFS with timestamps
let result = dfs(&graph)?;
for (node, times) in result {
    println!("{}: discovered={}, finished={}", 
        node, times.discovered, times.finished);
}

// DFS from specific start
let visited = dfs_from(&graph, start)?;

// DFS with early termination
let found = dfs_find(&graph, start, |node| {
    node.data == "target"
})?;
```

### Topological Sort

Order nodes in a directed acyclic graph (DAG).

```rust
use cim_graph::algorithms::topological_sort;

match topological_sort(&graph)? {
    Ok(ordering) => {
        println!("Topological order: {:?}", ordering);
        // Use for dependency resolution, build systems, etc.
    }
    Err(cycle) => {
        println!("Graph contains cycle: {:?}", cycle);
    }
}

// Kahn's algorithm (alternative implementation)
let ordering = kahns_algorithm(&graph)?;
```

## Analysis Algorithms

### Connected Components

Find all connected components in an undirected graph.

```rust
use cim_graph::algorithms::connected_components;

let components = connected_components(&graph)?;
println!("Found {} components", components.len());

for (i, component) in components.iter().enumerate() {
    println!("Component {}: {} nodes", i, component.len());
}

// Check if two nodes are connected
let same_component = is_connected(&graph, node1, node2)?;
```

### Strongly Connected Components

Find strongly connected components in a directed graph.

```rust
use cim_graph::algorithms::{strongly_connected_components, tarjan_scc};

// Kosaraju's algorithm
let sccs = strongly_connected_components(&graph)?;

// Tarjan's algorithm (often faster)
let sccs = tarjan_scc(&graph)?;

// Find SCC containing a specific node
let component = find_scc(&graph, node)?;
```

### Cycle Detection

Detect and analyze cycles in graphs.

```rust
use cim_graph::algorithms::{has_cycle, find_cycles, find_cycle_from};

// Check if graph has any cycle
if has_cycle(&graph)? {
    println!("Graph contains cycles");
}

// Find all cycles
let cycles = find_cycles(&graph)?;
for cycle in cycles {
    println!("Cycle: {:?}", cycle);
}

// Find cycle reachable from a node
if let Some(cycle) = find_cycle_from(&graph, start)? {
    println!("Found cycle: {:?}", cycle);
}
```

### Minimum Spanning Tree

Find the minimum spanning tree of a weighted graph.

```rust
use cim_graph::algorithms::{kruskal, prim};

// Kruskal's algorithm
let mst_edges = kruskal(&graph)?;
let total_weight: f64 = mst_edges.iter()
    .map(|e| e.weight())
    .sum();

// Prim's algorithm
let mst = prim(&graph, start)?;
```

## Pattern Matching

### Subgraph Isomorphism

Find occurrences of a pattern graph within a larger graph.

```rust
use cim_graph::algorithms::subgraph_isomorphism;

let pattern = create_pattern_graph()?;
let matches = subgraph_isomorphism(&graph, &pattern)?;

for mapping in matches {
    println!("Found match: {:?}", mapping);
}

// With constraints
let matches = subgraph_isomorphism_constrained(&graph, &pattern, |n1, n2| {
    n1.node_type == n2.node_type
})?;
```

### Motif Finding

Find common patterns or motifs in graphs.

```rust
use cim_graph::algorithms::{find_motifs, MotifType};

// Find all triangles
let triangles = find_motifs(&graph, MotifType::Triangle)?;

// Find 4-node motifs
let motifs = find_motifs(&graph, MotifType::Size(4))?;

// Custom motif patterns
let custom_motif = MotifPattern::new()
    .add_edge(0, 1)
    .add_edge(1, 2)
    .add_edge(0, 2);
let matches = find_custom_motif(&graph, &custom_motif)?;
```

### Graph Matching

Match nodes between two graphs based on structural similarity.

```rust
use cim_graph::algorithms::graph_matching;

let graph1 = create_graph1()?;
let graph2 = create_graph2()?;

// Find best node correspondence
let matching = graph_matching(&graph1, &graph2)?;

// With similarity threshold
let matching = graph_matching_threshold(&graph1, &graph2, 0.8)?;
```

## Graph Metrics

### Centrality Measures

Identify important nodes in the graph.

```rust
use cim_graph::algorithms::{
    degree_centrality,
    betweenness_centrality,
    closeness_centrality,
    eigenvector_centrality,
    pagerank
};

// Degree centrality
let degrees = degree_centrality(&graph)?;

// Betweenness centrality
let betweenness = betweenness_centrality(&graph)?;

// Closeness centrality
let closeness = closeness_centrality(&graph)?;

// Eigenvector centrality
let eigenvector = eigenvector_centrality(&graph, 100)?; // 100 iterations

// PageRank
let ranks = pagerank(&graph, 0.85, 100)?; // damping=0.85, iterations=100

// Find most central nodes
let mut central: Vec<_> = ranks.into_iter().collect();
central.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
let top_10: Vec<_> = central.into_iter().take(10).collect();
```

### Clustering Coefficients

Measure how clustered the graph is.

```rust
use cim_graph::algorithms::{
    clustering_coefficient,
    local_clustering_coefficient,
    average_clustering
};

// Local clustering coefficient for a node
let coeff = local_clustering_coefficient(&graph, node)?;

// All nodes' clustering coefficients
let coefficients = clustering_coefficient(&graph)?;

// Average clustering coefficient
let avg = average_clustering(&graph)?;
println!("Average clustering: {}", avg);
```

### Community Detection

Find communities or clusters in graphs.

```rust
use cim_graph::algorithms::{louvain, label_propagation, girvan_newman};

// Louvain method
let communities = louvain(&graph)?;
for (i, community) in communities.iter().enumerate() {
    println!("Community {}: {} nodes", i, community.len());
}

// Label propagation
let communities = label_propagation(&graph, 100)?; // max 100 iterations

// Girvan-Newman (hierarchical)
let dendrogram = girvan_newman(&graph)?;
let communities = dendrogram.cut_at_level(3)?; // 3 communities
```

## Custom Algorithms

### Writing Generic Algorithms

Create algorithms that work with any graph type:

```rust
use cim_graph::{Graph, Node, Edge};

pub fn my_algorithm<G>(graph: &G) -> Result<Vec<G::Node>>
where
    G: Graph,
    G::Node: Clone + Debug,
    G::Edge: Weighted,
{
    let mut result = Vec::new();
    
    // Your algorithm logic
    for node_id in graph.node_indices() {
        let node = graph.get_node(node_id)?;
        let neighbors = graph.neighbors(node_id)?;
        
        // Process node and neighbors
        if some_condition(node, &neighbors) {
            result.push(node.clone());
        }
    }
    
    Ok(result)
}
```

### Algorithm Combinators

Combine existing algorithms:

```rust
use cim_graph::algorithms::{compose, parallel, sequential};

// Run algorithms in sequence
let result = sequential()
    .then(|g| connected_components(g))
    .then(|g| find_cycles(g))
    .then(|g| topological_sort(g))
    .run(&graph)?;

// Run algorithms in parallel
let (components, cycles) = parallel()
    .and(|g| connected_components(g))
    .and(|g| find_cycles(g))
    .run(&graph)?;

// Compose algorithms
let analyze = compose(
    |g| dfs(g),
    |g, dfs_result| use_dfs_for_analysis(g, dfs_result)
);
let result = analyze(&graph)?;
```

### Streaming Algorithms

Process large graphs efficiently:

```rust
use cim_graph::algorithms::streaming::{StreamingBFS, StreamingPageRank};

// Streaming BFS for very large graphs
let mut bfs = StreamingBFS::new(start);
while let Some(node) = bfs.next(&graph)? {
    process_node(node);
    
    // Optionally limit exploration
    if should_stop(node) {
        break;
    }
}

// Streaming PageRank
let mut pagerank = StreamingPageRank::new(0.85);
for _ in 0..10 {
    pagerank.iterate(&graph)?;
    let top_nodes = pagerank.top_k(100)?;
    // Process intermediate results
}
```

## Performance Tips

### Algorithm Selection

Choose the right algorithm for your use case:

| Task | Small Graphs | Large Graphs | Dense Graphs | Sparse Graphs |
|------|--------------|--------------|--------------|---------------|
| Shortest path | Dijkstra | A* with heuristic | Floyd-Warshall | Dijkstra |
| All pairs | Floyd-Warshall | Johnson's | Floyd-Warshall | Johnson's |
| Components | DFS | Union-Find | DFS | Union-Find |
| Centrality | Direct | Approximation | Matrix methods | Iterative |

### Memory Optimization

```rust
// Pre-allocate for known sizes
let mut visited = BitVec::with_capacity(graph.node_count());

// Use iterators instead of collecting
graph.nodes()
    .filter(|n| n.active)
    .map(|n| process(n))
    .for_each(|r| handle_result(r));

// Clear caches between runs
graph.clear_caches();
```

### Parallel Execution

```rust
use rayon::prelude::*;

// Parallel centrality computation
let centralities: Vec<_> = graph.nodes()
    .par_iter()
    .map(|node| compute_centrality(&graph, node))
    .collect();

// Parallel community detection
let communities = parallel_louvain(&graph, num_threads)?;
```

### Early Termination

```rust
// Stop when condition is met
let result = dijkstra_with_stop(&graph, start, |node, distance| {
    distance > max_distance || found_target(node)
})?;

// Bounded search
let nearby = bfs_bounded(&graph, start, max_depth)?;
```

### Caching Results

```rust
use cim_graph::algorithms::CachedAlgorithms;

// Create cached algorithm wrapper
let mut cached = CachedAlgorithms::new(&graph);

// First call computes, subsequent calls return cached
let sp1 = cached.shortest_path(a, b)?; // Computes
let sp2 = cached.shortest_path(a, b)?; // Returns cached

// Invalidate cache on graph change
graph.add_edge(x, y, edge)?;
cached.invalidate();
```