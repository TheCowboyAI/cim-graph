# Pattern Recognition Algorithms

## Core Algorithms for Pattern Detection

### 1. Clique Detection Algorithms

```rust
/// Bron-Kerbosch algorithm with pivoting for maximal clique enumeration
pub struct BronKerboschPivot {
    pub max_clique_size: Option<usize>,
    pub timeout: Option<Duration>,
}

impl BronKerboschPivot {
    pub fn find_all_maximal_cliques<G: MathematicalGraph>(
        &self,
        graph: &G,
    ) -> Vec<Clique> {
        let mut cliques = Vec::new();
        let mut r = HashSet::new();
        let mut p: HashSet<_> = graph.nodes().collect();
        let mut x = HashSet::new();
        
        self.bron_kerbosch_pivot(graph, &mut r, &mut p, &mut x, &mut cliques);
        
        cliques
    }
    
    fn bron_kerbosch_pivot<G: MathematicalGraph>(
        &self,
        graph: &G,
        r: &mut HashSet<NodeId>,
        p: &mut HashSet<NodeId>,
        x: &mut HashSet<NodeId>,
        cliques: &mut Vec<Clique>,
    ) {
        if p.is_empty() && x.is_empty() {
            if let Some(max) = self.max_clique_size {
                if r.len() <= max {
                    cliques.push(Clique::new(r.clone()));
                }
            } else {
                cliques.push(Clique::new(r.clone()));
            }
            return;
        }
        
        // Choose pivot
        let pivot = p.union(x)
            .max_by_key(|&&v| {
                graph.neighbors(v)
                    .filter(|n| p.contains(n))
                    .count()
            })
            .copied();
            
        if let Some(pivot_node) = pivot {
            let candidates: Vec<_> = p.difference(&graph.neighbors(pivot_node).collect())
                .copied()
                .collect();
                
            for v in candidates {
                r.insert(v);
                
                let v_neighbors: HashSet<_> = graph.neighbors(v).collect();
                let mut new_p: HashSet<_> = p.intersection(&v_neighbors).copied().collect();
                let mut new_x: HashSet<_> = x.intersection(&v_neighbors).copied().collect();
                
                self.bron_kerbosch_pivot(graph, r, &mut new_p, &mut new_x, cliques);
                
                r.remove(&v);
                p.remove(&v);
                x.insert(v);
            }
        }
    }
}
```

### 2. Hamiltonian Path/Cycle Detection

```rust
/// Dynamic programming approach for small graphs (Held-Karp)
pub struct HamiltonianDetector {
    pub find_path: bool,
    pub find_cycle: bool,
}

impl HamiltonianDetector {
    pub fn detect<G: MathematicalGraph>(&self, graph: &G) -> Option<HamiltonianResult> {
        let n = graph.node_count();
        
        // For small graphs, use exact algorithm
        if n <= 20 {
            self.held_karp(graph)
        } else {
            // For larger graphs, use heuristics
            self.ant_colony_optimization(graph)
                .or_else(|| self.genetic_algorithm(graph))
        }
    }
    
    fn held_karp<G: MathematicalGraph>(&self, graph: &G) -> Option<HamiltonianResult> {
        let n = graph.node_count();
        let nodes: Vec<_> = graph.nodes().collect();
        
        // dp[mask][i] = shortest path visiting nodes in mask, ending at i
        let mut dp = vec![vec![None; n]; 1 << n];
        let mut parent = vec![vec![None; n]; 1 << n];
        
        // Initialize
        for i in 0..n {
            dp[1 << i][i] = Some(0.0);
        }
        
        // Fill DP table
        for mask in 1..(1 << n) {
            for last in 0..n {
                if mask & (1 << last) == 0 {
                    continue;
                }
                
                let prev_mask = mask ^ (1 << last);
                if prev_mask == 0 {
                    continue;
                }
                
                for prev in 0..n {
                    if prev_mask & (1 << prev) == 0 {
                        continue;
                    }
                    
                    if let Some(dist) = dp[prev_mask][prev] {
                        if graph.has_edge(nodes[prev], nodes[last]) {
                            let new_dist = dist + 1.0;
                            
                            if dp[mask][last].map_or(true, |d| new_dist < d) {
                                dp[mask][last] = Some(new_dist);
                                parent[mask][last] = Some(prev);
                            }
                        }
                    }
                }
            }
        }
        
        // Check for Hamiltonian path/cycle
        let full_mask = (1 << n) - 1;
        
        if self.find_cycle {
            // Find cycle: any node can be the end, must connect back to start
            for start in 0..n {
                for end in 0..n {
                    if let Some(_) = dp[full_mask][end] {
                        if graph.has_edge(nodes[end], nodes[start]) {
                            let path = self.reconstruct_path(&parent, full_mask, end, &nodes);
                            return Some(HamiltonianResult::Cycle(path));
                        }
                    }
                }
            }
        }
        
        if self.find_path {
            // Find path: any node can be the end
            for end in 0..n {
                if let Some(_) = dp[full_mask][end] {
                    let path = self.reconstruct_path(&parent, full_mask, end, &nodes);
                    return Some(HamiltonianResult::Path(path));
                }
            }
        }
        
        None
    }
}
```

### 3. Subgraph Isomorphism (VF2 Algorithm)

```rust
pub struct VF2Matcher {
    pub induced: bool,
    pub all_matches: bool,
}

impl VF2Matcher {
    pub fn find_matches<G1, G2>(&self, pattern: &G1, target: &G2) -> Vec<Mapping>
    where
        G1: MathematicalGraph,
        G2: MathematicalGraph,
    {
        let mut state = VF2State::new(pattern, target);
        let mut matches = Vec::new();
        
        self.match_recursive(&mut state, &mut matches);
        
        matches
    }
    
    fn match_recursive<G1, G2>(
        &self,
        state: &mut VF2State<G1, G2>,
        matches: &mut Vec<Mapping>,
    ) where
        G1: MathematicalGraph,
        G2: MathematicalGraph,
    {
        if state.is_complete() {
            matches.push(state.get_mapping());
            if !self.all_matches {
                return;
            }
        }
        
        let candidates = state.get_candidates();
        
        for (p_node, t_node) in candidates {
            if self.is_feasible(state, p_node, t_node) {
                state.add_pair(p_node, t_node);
                self.match_recursive(state, matches);
                state.remove_pair(p_node, t_node);
                
                if !self.all_matches && !matches.is_empty() {
                    return;
                }
            }
        }
    }
    
    fn is_feasible<G1, G2>(
        &self,
        state: &VF2State<G1, G2>,
        p_node: NodeId,
        t_node: NodeId,
    ) -> bool
    where
        G1: MathematicalGraph,
        G2: MathematicalGraph,
    {
        // Syntactic feasibility
        if !self.check_syntactic_feasibility(state, p_node, t_node) {
            return false;
        }
        
        // Semantic feasibility
        if !self.check_semantic_feasibility(state, p_node, t_node) {
            return false;
        }
        
        // Check 1-look-ahead
        self.check_lookahead(state, p_node, t_node)
    }
}
```

### 4. Community Detection Algorithms

```rust
/// Louvain algorithm for community detection
pub struct LouvainDetector {
    pub resolution: f64,
    pub randomize: bool,
}

impl LouvainDetector {
    pub fn detect_communities<G: MathematicalGraph>(
        &self,
        graph: &G,
    ) -> Communities {
        let mut partition = self.initialize_partition(graph);
        let mut modularity = self.compute_modularity(graph, &partition);
        
        loop {
            let mut improved = false;
            
            // Phase 1: Local optimization
            let nodes: Vec<_> = if self.randomize {
                let mut nodes = graph.nodes().collect::<Vec<_>>();
                nodes.shuffle(&mut thread_rng());
                nodes
            } else {
                graph.nodes().collect()
            };
            
            for node in nodes {
                let current_community = partition.get_community(node);
                let mut best_community = current_community;
                let mut best_gain = 0.0;
                
                // Try moving to neighboring communities
                let neighbor_communities = self.get_neighbor_communities(graph, node, &partition);
                
                for community in neighbor_communities {
                    if community == current_community {
                        continue;
                    }
                    
                    let gain = self.compute_modularity_gain(graph, node, community, &partition);
                    if gain > best_gain {
                        best_gain = gain;
                        best_community = community;
                    }
                }
                
                if best_community != current_community {
                    partition.move_node(node, best_community);
                    modularity += best_gain;
                    improved = true;
                }
            }
            
            if !improved {
                break;
            }
            
            // Phase 2: Community aggregation
            partition = self.aggregate_communities(graph, &partition);
        }
        
        Communities {
            partition,
            modularity,
        }
    }
}
```

### 5. Motif Detection

```rust
/// Efficient motif counting using FANMOD algorithm
pub struct MotifCounter {
    pub motif_size: usize,
    pub directed: bool,
}

impl MotifCounter {
    pub fn count_motifs<G: MathematicalGraph>(
        &self,
        graph: &G,
    ) -> HashMap<MotifType, usize> {
        match self.motif_size {
            3 => self.count_triads(graph),
            4 => self.count_tetrads(graph),
            _ => self.count_general_motifs(graph),
        }
    }
    
    fn count_triads<G: MathematicalGraph>(
        &self,
        graph: &G,
    ) -> HashMap<MotifType, usize> {
        let mut counts = HashMap::new();
        
        // Enumerate all triads
        for a in graph.nodes() {
            for b in graph.neighbors(a) {
                if b <= a { continue; }
                
                for c in graph.neighbors(b) {
                    if c <= b { continue; }
                    
                    let motif_type = self.classify_triad(graph, a, b, c);
                    *counts.entry(motif_type).or_insert(0) += 1;
                }
            }
        }
        
        counts
    }
    
    fn classify_triad<G: MathematicalGraph>(
        &self,
        graph: &G,
        a: NodeId,
        b: NodeId,
        c: NodeId,
    ) -> MotifType {
        let mut edges = 0;
        
        if graph.has_edge(a, b) { edges |= 1; }
        if graph.has_edge(b, c) { edges |= 2; }
        if graph.has_edge(a, c) { edges |= 4; }
        
        if self.directed {
            if graph.has_edge(b, a) { edges |= 8; }
            if graph.has_edge(c, b) { edges |= 16; }
            if graph.has_edge(c, a) { edges |= 32; }
        }
        
        MotifType::Triad(edges)
    }
}
```

### 6. Planarity Testing

```rust
/// Boyer-Myrvold planarity testing algorithm
pub struct PlanarityTester {
    pub provide_embedding: bool,
    pub provide_kuratowski: bool,
}

impl PlanarityTester {
    pub fn test<G: MathematicalGraph>(&self, graph: &G) -> PlanarityResult {
        // Quick checks
        let n = graph.node_count();
        let m = graph.edge_count();
        
        // By Euler's formula: m <= 3n - 6 for planar graphs
        if m > 3 * n - 6 {
            return PlanarityResult::NonPlanar {
                kuratowski: if self.provide_kuratowski {
                    Some(self.find_kuratowski_subgraph(graph))
                } else {
                    None
                },
            };
        }
        
        // Boyer-Myrvold algorithm
        let mut embedding = PlanarEmbedding::new(graph);
        
        for v in graph.nodes() {
            if !self.add_vertex(&mut embedding, v) {
                return PlanarityResult::NonPlanar {
                    kuratowski: if self.provide_kuratowski {
                        Some(self.extract_kuratowski(&embedding))
                    } else {
                        None
                    },
                };
            }
        }
        
        PlanarityResult::Planar {
            embedding: if self.provide_embedding {
                Some(embedding)
            } else {
                None
            },
        }
    }
}
```

### 7. Graph Coloring

```rust
/// Welsh-Powell and DSATUR algorithms for graph coloring
pub struct GraphColorer {
    pub algorithm: ColoringAlgorithm,
    pub max_colors: Option<usize>,
}

impl GraphColorer {
    pub fn color<G: MathematicalGraph>(&self, graph: &G) -> Option<Coloring> {
        match self.algorithm {
            ColoringAlgorithm::Greedy => self.greedy_coloring(graph),
            ColoringAlgorithm::WelshPowell => self.welsh_powell(graph),
            ColoringAlgorithm::DSATUR => self.dsatur(graph),
            ColoringAlgorithm::Exact => self.exact_coloring(graph),
        }
    }
    
    fn dsatur<G: MathematicalGraph>(&self, graph: &G) -> Option<Coloring> {
        let mut coloring = HashMap::new();
        let mut saturation = HashMap::new();
        let mut uncolored: HashSet<_> = graph.nodes().collect();
        
        // Initialize saturation degrees
        for node in graph.nodes() {
            saturation.insert(node, 0);
        }
        
        while !uncolored.is_empty() {
            // Choose vertex with maximum saturation degree
            let &next = uncolored.iter()
                .max_by_key(|&&v| {
                    let sat = saturation[&v];
                    let deg = graph.degree(v);
                    (sat, deg)
                })
                .unwrap();
            
            // Find smallest available color
            let used_colors: HashSet<_> = graph.neighbors(next)
                .filter_map(|n| coloring.get(&n))
                .copied()
                .collect();
            
            let color = (0..)
                .find(|c| !used_colors.contains(c))
                .unwrap();
            
            if let Some(max) = self.max_colors {
                if color >= max {
                    return None; // Cannot color with given colors
                }
            }
            
            coloring.insert(next, color);
            uncolored.remove(&next);
            
            // Update saturation degrees
            for neighbor in graph.neighbors(next) {
                if uncolored.contains(&neighbor) {
                    let neighbor_colors: HashSet<_> = graph.neighbors(neighbor)
                        .filter_map(|n| coloring.get(&n))
                        .copied()
                        .collect();
                    saturation.insert(neighbor, neighbor_colors.len());
                }
            }
        }
        
        Some(Coloring { colors: coloring })
    }
}
```

### 8. Centrality Algorithms

```rust
/// Various centrality measures
pub struct CentralityCalculator {
    pub normalize: bool,
    pub weighted: bool,
}

impl CentralityCalculator {
    pub fn betweenness_centrality<G: MathematicalGraph>(
        &self,
        graph: &G,
    ) -> HashMap<NodeId, f64> {
        let mut centrality = HashMap::new();
        
        for node in graph.nodes() {
            centrality.insert(node, 0.0);
        }
        
        // Brandes algorithm
        for s in graph.nodes() {
            let mut stack = Vec::new();
            let mut paths = HashMap::new();
            let mut sigma = HashMap::new();
            let mut dist = HashMap::new();
            let mut delta = HashMap::new();
            
            for node in graph.nodes() {
                paths.insert(node, Vec::new());
                sigma.insert(node, 0.0);
                dist.insert(node, -1.0);
                delta.insert(node, 0.0);
            }
            
            sigma.insert(s, 1.0);
            dist.insert(s, 0.0);
            
            let mut queue = VecDeque::new();
            queue.push_back(s);
            
            // BFS
            while let Some(v) = queue.pop_front() {
                stack.push(v);
                
                for w in graph.neighbors(v) {
                    // First time we reach w?
                    if dist[&w] < 0.0 {
                        queue.push_back(w);
                        dist.insert(w, dist[&v] + 1.0);
                    }
                    
                    // Shortest path to w via v?
                    if dist[&w] == dist[&v] + 1.0 {
                        sigma.insert(w, sigma[&w] + sigma[&v]);
                        paths.get_mut(&w).unwrap().push(v);
                    }
                }
            }
            
            // Accumulation
            while let Some(w) = stack.pop() {
                for &v in &paths[&w] {
                    let contribution = (sigma[&v] / sigma[&w]) * (1.0 + delta[&w]);
                    delta.insert(v, delta[&v] + contribution);
                }
                
                if w != s {
                    centrality.insert(w, centrality[&w] + delta[&w]);
                }
            }
        }
        
        if self.normalize {
            let n = graph.node_count() as f64;
            let factor = 1.0 / ((n - 1.0) * (n - 2.0));
            
            for value in centrality.values_mut() {
                *value *= factor;
            }
        }
        
        centrality
    }
}
```

## Performance Characteristics

| Algorithm | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| Bron-Kerbosch | O(3^(n/3)) | O(n²) | Worst case for maximal cliques |
| Hamiltonian (Held-Karp) | O(n²2^n) | O(n2^n) | Exact for n ≤ 20 |
| VF2 Subgraph Iso | O(n!n) | O(n) | Worst case, usually much better |
| Louvain | O(m) | O(n) | Per iteration, very fast |
| Motif Counting | O(n^k) | O(1) | For k-node motifs |
| Planarity Testing | O(n) | O(n) | Linear time! |
| DSATUR Coloring | O(n²) | O(n) | Heuristic |
| Betweenness | O(nm) | O(n+m) | Brandes algorithm |

These algorithms provide the computational foundation for pattern recognition in the Graph Domain.