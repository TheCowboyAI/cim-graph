# Graph Pattern Recognition and Invocation

## Overview

The Graph Domain provides comprehensive pattern recognition for well-known graph structures and the ability to invoke (construct) these patterns programmatically.

## Pattern Categories

### 1. Structural Patterns

```rust
pub enum StructuralPattern {
    // Complete structures
    CompleteGraph { n: usize },
    CompleteBipartite { m: usize, n: usize },
    
    // Cycles and paths
    Cycle { length: usize },
    Path { length: usize },
    HamiltonianCycle,
    HamiltonianPath,
    EulerianCycle,
    EulerianPath,
    
    // Trees
    Star { n: usize },
    BinaryTree { height: usize },
    CompleteBinaryTree { height: usize },
    BalancedTree { degree: usize, height: usize },
    SpanningTree,
    
    // Regular structures
    RegularGraph { degree: usize, n: usize },
    PetersenGraph,
    HypercubeGraph { dimension: usize },
    GridGraph { rows: usize, cols: usize },
    TorusGraph { rows: usize, cols: usize },
}
```

### 2. Topological Patterns

```rust
pub enum TopologicalPattern {
    // Shapes
    Triangle,
    Square,
    Pentagon,
    Hexagon,
    Octahedron,
    Tetrahedron,
    Cube,
    Dodecahedron,
    Icosahedron,
    
    // Complex shapes
    MoebiusStrip { width: usize },
    KleinBottle { resolution: usize },
    Torus { major_radius: usize, minor_radius: usize },
    
    // Fractals
    SierpinskiTriangle { depth: usize },
    MengerSponge { level: usize },
    KochSnowflake { iterations: usize },
}
```

### 3. Social Network Patterns

```rust
pub enum SocialPattern {
    // Centrality patterns
    StarNetwork { center: NodeId, satellites: Vec<NodeId> },
    Hub { hub_nodes: Vec<NodeId>, spoke_ratio: f64 },
    
    // Community structures
    Clique { members: Vec<NodeId> },
    BiClique { set_a: Vec<NodeId>, set_b: Vec<NodeId> },
    CorePeriphery { core: Vec<NodeId>, periphery: Vec<NodeId> },
    
    // Hierarchy patterns
    Hierarchy { levels: Vec<Vec<NodeId>> },
    BowTie { in_component: Vec<NodeId>, core: Vec<NodeId>, out_component: Vec<NodeId> },
    
    // Motifs
    FeedForwardLoop,
    FeedBackLoop,
    BiParallelMotif,
}
```

### 4. Flow Network Patterns

```rust
pub enum FlowPattern {
    // Basic flow structures
    SourceSink { source: NodeId, sink: NodeId },
    SeriesFlow { nodes: Vec<NodeId> },
    ParallelFlow { branches: Vec<Vec<NodeId>> },
    
    // Complex flow patterns
    WheatstoneNetwork,
    DeltaWyeTransform,
    ButterflyNetwork { stages: usize },
    BenešNetwork { size: usize },
    
    // Distribution patterns
    BroadcastTree { root: NodeId, fanout: usize },
    GatherTree { collectors: Vec<NodeId> },
    ScatterGather { scatter_point: NodeId, gather_point: NodeId },
}
```

## Pattern Recognition API

### Detection Interface

```rust
pub trait PatternDetector: MathematicalGraph {
    /// Detect all instances of a pattern
    fn detect_pattern(&self, pattern: &GraphPattern) -> Vec<PatternMatch>;
    
    /// Check if graph contains pattern
    fn contains_pattern(&self, pattern: &GraphPattern) -> bool;
    
    /// Find approximate pattern matches
    fn fuzzy_detect(&self, pattern: &GraphPattern, tolerance: f64) -> Vec<FuzzyMatch>;
    
    /// Detect all patterns in graph
    fn detect_all_patterns(&self) -> PatternInventory;
}

pub struct PatternMatch {
    pub pattern: GraphPattern,
    pub node_mapping: HashMap<PatternNodeId, GraphNodeId>,
    pub edge_mapping: HashMap<PatternEdgeId, GraphEdgeId>,
    pub confidence: f64,
    pub exact: bool,
}
```

### Pattern Detection Algorithms

```rust
impl PatternDetector for CimGraph {
    fn detect_complete_subgraphs(&self, min_size: usize) -> Vec<Clique> {
        // Bron-Kerbosch algorithm for maximal cliques
        let mut cliques = Vec::new();
        self.bron_kerbosch(
            &mut HashSet::new(),  // R
            &mut self.nodes().collect(),  // P
            &mut HashSet::new(),  // X
            &mut cliques,
        );
        cliques.into_iter()
            .filter(|c| c.len() >= min_size)
            .collect()
    }
    
    fn detect_hamiltonian_cycle(&self) -> Option<Vec<NodeId>> {
        // Use dynamic programming for small graphs
        if self.node_count() <= 20 {
            self.held_karp_algorithm()
        } else {
            // Use heuristics for larger graphs
            self.christofides_algorithm()
        }
    }
    
    fn detect_bipartite_structure(&self) -> Option<(HashSet<NodeId>, HashSet<NodeId>)> {
        // Two-coloring algorithm
        let mut color = HashMap::new();
        let mut queue = VecDeque::new();
        
        for start in self.nodes() {
            if color.contains_key(&start) {
                continue;
            }
            
            color.insert(start, 0);
            queue.push_back(start);
            
            while let Some(node) = queue.pop_front() {
                let node_color = color[&node];
                for neighbor in self.neighbors(node) {
                    if let Some(&neighbor_color) = color.get(&neighbor) {
                        if neighbor_color == node_color {
                            return None; // Not bipartite
                        }
                    } else {
                        color.insert(neighbor, 1 - node_color);
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        
        let set_0: HashSet<_> = color.iter()
            .filter(|(_, &c)| c == 0)
            .map(|(n, _)| *n)
            .collect();
        let set_1: HashSet<_> = color.iter()
            .filter(|(_, &c)| c == 1)
            .map(|(n, _)| *n)
            .collect();
            
        Some((set_0, set_1))
    }
}
```

### Shape Detection

```rust
pub struct ShapeDetector {
    tolerance: f64,
}

impl ShapeDetector {
    pub fn detect_geometric_shapes(&self, graph: &impl SpatialGraph) -> Vec<GeometricShape> {
        let mut shapes = Vec::new();
        
        // Detect triangles
        for triangle in graph.enumerate_triangles() {
            if self.is_equilateral(&triangle, graph) {
                shapes.push(GeometricShape::EquilateralTriangle(triangle));
            } else if self.is_isosceles(&triangle, graph) {
                shapes.push(GeometricShape::IsoscelesTriangle(triangle));
            }
        }
        
        // Detect squares and rectangles
        for cycle in graph.find_cycles_of_length(4) {
            if self.is_square(&cycle, graph) {
                shapes.push(GeometricShape::Square(cycle));
            } else if self.is_rectangle(&cycle, graph) {
                shapes.push(GeometricShape::Rectangle(cycle));
            }
        }
        
        // Detect regular polygons
        for n in 5..=12 {
            for cycle in graph.find_cycles_of_length(n) {
                if self.is_regular_polygon(&cycle, graph) {
                    shapes.push(GeometricShape::RegularPolygon(n, cycle));
                }
            }
        }
        
        shapes
    }
    
    fn is_regular_polygon(&self, cycle: &[NodeId], graph: &impl SpatialGraph) -> bool {
        let positions: Vec<_> = cycle.iter()
            .map(|n| graph.position(*n))
            .collect();
            
        // Check all edge lengths are equal
        let edge_lengths: Vec<_> = positions.windows(2)
            .map(|w| distance(&w[0], &w[1]))
            .collect();
            
        let avg_length = edge_lengths.iter().sum::<f64>() / edge_lengths.len() as f64;
        
        edge_lengths.iter()
            .all(|&len| (len - avg_length).abs() < self.tolerance)
    }
}
```

## Pattern Invocation API

### Construction Interface

```rust
pub trait PatternInvoker {
    /// Create a graph from a pattern
    fn invoke_pattern(pattern: GraphPattern) -> Result<Self, PatternError>
    where
        Self: Sized;
    
    /// Add a pattern to existing graph
    fn add_pattern(&mut self, pattern: GraphPattern, anchor: Option<NodeId>) -> Result<PatternRef, PatternError>;
    
    /// Replace subgraph with pattern
    fn replace_with_pattern(&mut self, subgraph: SubgraphRef, pattern: GraphPattern) -> Result<(), PatternError>;
}
```

### Pattern Builders

```rust
pub mod patterns {
    /// Create a complete graph K_n
    pub fn complete_graph(n: usize) -> GraphBuilder {
        let mut builder = GraphBuilder::new();
        
        for i in 0..n {
            builder.add_node(format!("v{}", i));
        }
        
        for i in 0..n {
            for j in i+1..n {
                builder.add_edge(
                    format!("v{}", i),
                    format!("v{}", j),
                    EdgeType::Undirected,
                );
            }
        }
        
        builder
    }
    
    /// Create a complete bipartite graph K_{m,n}
    pub fn complete_bipartite(m: usize, n: usize) -> GraphBuilder {
        let mut builder = GraphBuilder::new();
        
        for i in 0..m {
            builder.add_node(format!("u{}", i))
                .with_property("partition", "A");
        }
        
        for j in 0..n {
            builder.add_node(format!("v{}", j))
                .with_property("partition", "B");
        }
        
        for i in 0..m {
            for j in 0..n {
                builder.add_edge(
                    format!("u{}", i),
                    format!("v{}", j),
                    EdgeType::Undirected,
                );
            }
        }
        
        builder
    }
    
    /// Create a hypercube graph Q_n
    pub fn hypercube(dimension: usize) -> GraphBuilder {
        let mut builder = GraphBuilder::new();
        let n = 1 << dimension; // 2^dimension
        
        // Add nodes (binary strings)
        for i in 0..n {
            let binary = format!("{:0width$b}", i, width = dimension);
            builder.add_node(binary);
        }
        
        // Add edges (differ by one bit)
        for i in 0..n {
            for bit in 0..dimension {
                let j = i ^ (1 << bit);
                if i < j {
                    let node_i = format!("{:0width$b}", i, width = dimension);
                    let node_j = format!("{:0width$b}", j, width = dimension);
                    builder.add_edge(node_i, node_j, EdgeType::Undirected);
                }
            }
        }
        
        builder
    }
    
    /// Create a Petersen graph
    pub fn petersen_graph() -> GraphBuilder {
        let mut builder = GraphBuilder::new();
        
        // Outer pentagon
        for i in 0..5 {
            builder.add_node(format!("outer_{}", i));
        }
        for i in 0..5 {
            builder.add_edge(
                format!("outer_{}", i),
                format!("outer_{}", (i + 1) % 5),
                EdgeType::Undirected,
            );
        }
        
        // Inner pentagram
        for i in 0..5 {
            builder.add_node(format!("inner_{}", i));
        }
        for i in 0..5 {
            builder.add_edge(
                format!("inner_{}", i),
                format!("inner_{}", (i + 2) % 5),
                EdgeType::Undirected,
            );
        }
        
        // Spokes
        for i in 0..5 {
            builder.add_edge(
                format!("outer_{}", i),
                format!("inner_{}", i),
                EdgeType::Undirected,
            );
        }
        
        builder
    }
}
```

### Advanced Pattern Construction

```rust
pub struct PatternLibrary {
    patterns: HashMap<String, PatternTemplate>,
}

impl PatternLibrary {
    /// Create a fractal pattern
    pub fn sierpinski_triangle(&self, depth: usize) -> GraphBuilder {
        fn sierpinski_recursive(
            builder: &mut GraphBuilder,
            top: NodeId,
            left: NodeId,
            right: NodeId,
            depth: usize,
        ) {
            if depth == 0 {
                builder.add_edge(top, left, EdgeType::Undirected);
                builder.add_edge(left, right, EdgeType::Undirected);
                builder.add_edge(right, top, EdgeType::Undirected);
                return;
            }
            
            let mid_left = builder.add_node(format!("ml_{}_{}", top, left));
            let mid_right = builder.add_node(format!("mr_{}_{}", top, right));
            let mid_bottom = builder.add_node(format!("mb_{}_{}", left, right));
            
            sierpinski_recursive(builder, top, mid_left, mid_right, depth - 1);
            sierpinski_recursive(builder, mid_left, left, mid_bottom, depth - 1);
            sierpinski_recursive(builder, mid_right, mid_bottom, right, depth - 1);
        }
        
        let mut builder = GraphBuilder::new();
        let top = builder.add_node("top");
        let left = builder.add_node("left");
        let right = builder.add_node("right");
        
        sierpinski_recursive(&mut builder, top, left, right, depth);
        builder
    }
    
    /// Create a small-world graph (Watts-Strogatz)
    pub fn small_world(&self, n: usize, k: usize, p: f64) -> GraphBuilder {
        let mut builder = GraphBuilder::new();
        let mut rng = thread_rng();
        
        // Create ring lattice
        for i in 0..n {
            builder.add_node(format!("v{}", i));
        }
        
        // Connect each node to k/2 neighbors on each side
        for i in 0..n {
            for j in 1..=k/2 {
                let neighbor = (i + j) % n;
                builder.add_edge(
                    format!("v{}", i),
                    format!("v{}", neighbor),
                    EdgeType::Undirected,
                );
            }
        }
        
        // Rewire edges with probability p
        let edges: Vec<_> = builder.edges().collect();
        for (from, to) in edges {
            if rng.gen::<f64>() < p {
                builder.remove_edge(from, to);
                let new_target = loop {
                    let candidate = rng.gen_range(0..n);
                    let candidate_id = format!("v{}", candidate);
                    if candidate_id != from && !builder.has_edge(&from, &candidate_id) {
                        break candidate_id;
                    }
                };
                builder.add_edge(from, new_target, EdgeType::Undirected);
            }
        }
        
        builder
    }
}
```

## Pattern Matching with Subgraph Isomorphism

```rust
pub struct SubgraphMatcher {
    use_vf2: bool,
    use_ullmann: bool,
}

impl SubgraphMatcher {
    pub fn find_all_matches<G1, G2>(&self, pattern: &G1, target: &G2) -> Vec<Isomorphism>
    where
        G1: MathematicalGraph,
        G2: MathematicalGraph,
    {
        if self.use_vf2 {
            self.vf2_algorithm(pattern, target)
        } else if self.use_ullmann {
            self.ullmann_algorithm(pattern, target)
        } else {
            self.backtrack_search(pattern, target)
        }
    }
    
    fn vf2_algorithm<G1, G2>(&self, pattern: &G1, target: &G2) -> Vec<Isomorphism> {
        // VF2 implementation for subgraph isomorphism
        todo!()
    }
}
```

## Pattern Composition

```rust
pub trait PatternComposer {
    /// Combine patterns
    fn compose_patterns(&self, patterns: Vec<GraphPattern>) -> ComposedPattern;
    
    /// Tile a pattern
    fn tile_pattern(&self, pattern: GraphPattern, tiling: TilingStrategy) -> TiledGraph;
    
    /// Create pattern variations
    fn vary_pattern(&self, pattern: GraphPattern, variations: Vec<Variation>) -> Vec<GraphPattern>;
}

pub enum TilingStrategy {
    Grid { rows: usize, cols: usize },
    Hexagonal { radius: usize },
    Recursive { depth: usize, scale: f64 },
}
```

## Pattern Events

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDetected {
    pub metadata: EventMetadata,
    pub graph_id: GraphId,
    pub pattern: GraphPattern,
    pub instances: Vec<PatternMatch>,
    pub detection_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInvoked {
    pub metadata: EventMetadata,
    pub graph_id: GraphId,
    pub pattern: GraphPattern,
    pub anchor_point: Option<NodeId>,
    pub created_nodes: Vec<NodeId>,
    pub created_edges: Vec<EdgeId>,
}
```

## Performance Optimizations

```rust
pub struct PatternIndex {
    /// Pre-computed pattern fingerprints
    fingerprints: HashMap<GraphPattern, PatternFingerprint>,
    
    /// Cached detection results
    cache: LruCache<(GraphId, GraphPattern), Vec<PatternMatch>>,
    
    /// Pattern-specific optimizations
    optimizers: HashMap<GraphPattern, Box<dyn PatternOptimizer>>,
}

impl PatternIndex {
    pub fn precompute_common_patterns(&mut self, graph: &impl MathematicalGraph) {
        // Pre-compute triangles
        self.triangle_census(graph);
        
        // Pre-compute k-cores
        self.k_core_decomposition(graph);
        
        // Pre-compute motif counts
        self.motif_counting(graph, 3);
        self.motif_counting(graph, 4);
    }
}
```

This pattern system provides:
1. **Comprehensive pattern catalog** - Structural, topological, social, and flow patterns
2. **Detection algorithms** - Find patterns in existing graphs
3. **Construction utilities** - Build graphs from patterns
4. **Advanced patterns** - Fractals, small-world, scale-free
5. **Subgraph isomorphism** - Full pattern matching
6. **Performance optimization** - Caching and pre-computation