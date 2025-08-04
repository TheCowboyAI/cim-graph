//! Custom property generators for graph structures

use proptest::prelude::*;
use proptest::collection::SizeRange;
use cim_graph::core::{Graph, GraphType, GenericNode, GenericEdge, Node, Edge};
use cim_graph::core::graph::BasicGraph;
use std::collections::{HashSet, HashMap};
use uuid::Uuid;
use rand::Rng;

/// Generate a valid node ID
pub fn node_id_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple IDs
        "[a-z][a-z0-9]{0,10}",
        // UUID-like IDs
        Just(()).prop_map(|_| Uuid::new_v4().to_string()),
        // Prefixed IDs
        ("[a-z]+", 0..1000u32).prop_map(|(prefix, num)| format!("{}-{}", prefix, num)),
    ]
}

/// Generate node data
pub fn node_data_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        "\\w{1,20}",
        Just("empty"),
        Just("test-data"),
        any::<u32>().prop_map(|n| format!("node-{}", n)),
    ]
}

/// Generate edge data
pub fn edge_data_strategy() -> impl Strategy<Value = f64> {
    prop_oneof![
        0.0..=1.0,
        1.0..=100.0,
        Just(0.0),
        Just(1.0),
        Just(f64::INFINITY),
    ]
}

/// Generate a single node
pub fn node_strategy() -> impl Strategy<Value = GenericNode<String>> {
    (node_id_strategy(), node_data_strategy())
        .prop_map(|(id, data)| GenericNode::new(id, data))
}

/// Generate a collection of unique nodes
pub fn nodes_strategy(size: impl Into<SizeRange>) -> impl Strategy<Value = Vec<GenericNode<String>>> {
    prop::collection::vec(node_strategy(), size)
        .prop_map(|nodes| {
            // Ensure unique IDs
            let mut seen = HashSet::new();
            let mut unique_nodes = Vec::new();
            
            for mut node in nodes {
                let mut id = node.id();
                let mut counter = 0;
                
                while seen.contains(&id) {
                    counter += 1;
                    id = format!("{}-{}", node.id(), counter);
                }
                
                seen.insert(id.clone());
                unique_nodes.push(GenericNode::new(id, node.data().clone()));
            }
            
            unique_nodes
        })
}

/// Generate an edge between existing nodes
pub fn edge_strategy(
    nodes: &[GenericNode<String>],
) -> impl Strategy<Value = Option<GenericEdge<f64>>> {
    if nodes.len() < 2 {
        return Just(None).boxed();
    }
    
    let node_ids: Vec<String> = nodes.iter().map(|n| n.id()).collect();
    
    (
        (0..node_ids.len(), 0..node_ids.len()),
        edge_data_strategy(),
        any::<bool>(), // directed or undirected
    )
        .prop_filter("source != target", |((from, to), _, _)| from != to)
        .prop_map(move |((from, to), weight, directed)| {
            let edge = if directed {
                GenericEdge::new(
                    node_ids[from].clone(),
                    node_ids[to].clone(),
                    weight,
                )
            } else {
                GenericEdge::undirected(
                    node_ids[from].clone(),
                    node_ids[to].clone(),
                    weight,
                )
            };
            Some(edge)
        })
        .boxed()
}

/// Generate a collection of edges for a set of nodes
pub fn edges_strategy(
    nodes: &[GenericNode<String>],
    density: f64,
) -> Vec<GenericEdge<f64>> {
    if nodes.len() < 2 {
        return vec![];
    }
    
    let mut edges = Vec::new();
    let mut edge_set = HashSet::new();
    let max_edges = nodes.len() * (nodes.len() - 1);
    let target_edges = ((max_edges as f64) * density) as usize;
    
    let mut rng = rand::thread_rng();
    use rand::seq::SliceRandom;
    
    let node_ids: Vec<String> = nodes.iter().map(|n| n.id()).collect();
    
    while edges.len() < target_edges && edge_set.len() < max_edges {
        let from = node_ids.choose(&mut rng).unwrap();
        let to = node_ids.choose(&mut rng).unwrap();
        
        if from != to {
            let edge_key = (from.clone(), to.clone());
            if !edge_set.contains(&edge_key) {
                edge_set.insert(edge_key);
                edges.push(GenericEdge::new(from, to, rand::random::<f64>() * 100.0));
            }
        }
    }
    
    edges
}

/// Generate a complete graph
pub fn graph_strategy(
    node_count: impl Into<SizeRange>,
    edge_density: impl Strategy<Value = f64>,
) -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    (
        nodes_strategy(node_count),
        edge_density,
        prop::sample::select(vec![
            GraphType::Generic,
            GraphType::WorkflowGraph,
            GraphType::ContextGraph,
        ]),
    )
        .prop_map(|(nodes, density, graph_type)| {
            let mut graph = BasicGraph::new(graph_type);
            
            // Add all nodes
            for node in &nodes {
                let _ = graph.add_node(node.clone());
            }
            
            // Add edges based on density
            let edges = edges_strategy(&nodes, density.min(1.0).max(0.0));
            for edge in edges {
                let _ = graph.add_edge(edge);
            }
            
            graph
        })
}

/// Generate a small graph (good for exhaustive testing)
pub fn small_graph_strategy() -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    graph_strategy(1..=10, 0.0..=1.0)
}

/// Generate a medium graph
pub fn medium_graph_strategy() -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    graph_strategy(10..=100, 0.0..=0.5)
}

/// Generate a large graph (for performance testing)
pub fn large_graph_strategy() -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    graph_strategy(100..=1000, 0.0..=0.1)
}

/// Generate a tree structure
pub fn tree_graph_strategy(
    depth: usize,
    branching_factor: impl Into<SizeRange>,
) -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    let branching = branching_factor.into();
    
    Just(()).prop_map(move |_| {
        let mut graph = BasicGraph::new(GraphType::Generic);
        let mut current_level = vec![];
        let mut next_level = vec![];
        let mut node_counter = 0;
        
        // Create root
        let root = GenericNode::new(format!("node-{}", node_counter), "root");
        node_counter += 1;
        graph.add_node(root.clone()).unwrap();
        current_level.push(root.id());
        
        // Build tree level by level
        for level in 0..depth {
            for parent_id in &current_level {
                let children_count = branching.start() + 
                    (rand::random::<usize>() % (branching.end() - branching.start() + 1));
                
                for _ in 0..children_count {
                    let child = GenericNode::new(
                        format!("node-{}", node_counter),
                        format!("level-{}", level + 1),
                    );
                    node_counter += 1;
                    
                    graph.add_node(child.clone()).unwrap();
                    graph.add_edge(GenericEdge::new(
                        parent_id.clone(),
                        child.id(),
                        1.0,
                    )).unwrap();
                    
                    next_level.push(child.id());
                }
            }
            
            current_level = next_level.clone();
            next_level.clear();
        }
        
        graph
    })
}

/// Generate a cyclic graph
pub fn cyclic_graph_strategy(
    cycle_size: impl Into<SizeRange>,
) -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    prop::collection::vec(node_strategy(), cycle_size).prop_map(|nodes| {
        let mut graph = BasicGraph::new(GraphType::Generic);
        
        if nodes.is_empty() {
            return graph;
        }
        
        // Add all nodes
        for node in &nodes {
            graph.add_node(node.clone()).unwrap();
        }
        
        // Create cycle
        for i in 0..nodes.len() {
            let from = &nodes[i];
            let to = &nodes[(i + 1) % nodes.len()];
            
            graph.add_edge(GenericEdge::new(
                from.id(),
                to.id(),
                1.0,
            )).unwrap();
        }
        
        graph
    })
}

/// Generate a disconnected graph (multiple components)
pub fn disconnected_graph_strategy(
    num_components: impl Into<SizeRange>,
) -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    prop::collection::vec(small_graph_strategy(), num_components).prop_map(|components| {
        let mut merged_graph = BasicGraph::new(GraphType::Generic);
        let mut node_id_map = HashMap::new();
        let mut id_counter = 0;
        
        for component in components {
            // Re-ID nodes to avoid conflicts
            for old_id in component.node_ids() {
                if let Some(node) = component.get_node(&old_id) {
                    let new_id = format!("component-node-{}", id_counter);
                    id_counter += 1;
                    
                    let new_node = GenericNode::new(new_id.clone(), node.data().clone());
                    merged_graph.add_node(new_node).unwrap();
                    node_id_map.insert(old_id.clone(), new_id);
                }
            }
            
            // Add edges with updated IDs
            for old_edge_id in component.edge_ids() {
                if let Some(edge) = component.get_edge(&old_edge_id) {
                    if let (Some(new_source), Some(new_target)) = (
                        node_id_map.get(&edge.source()),
                        node_id_map.get(&edge.target()),
                    ) {
                        merged_graph.add_edge(GenericEdge::new(
                            new_source.clone(),
                            new_target.clone(),
                            *edge.data(),
                        )).unwrap();
                    }
                }
            }
            
            node_id_map.clear();
        }
        
        merged_graph
    })
}

/// Generate pathological edge cases
pub fn pathological_graph_strategy() -> impl Strategy<Value = BasicGraph<GenericNode<String>, GenericEdge<f64>>> {
    prop_oneof![
        // Empty graph
        Just(BasicGraph::new(GraphType::Generic)),
        
        // Single node, no edges
        Just(()).prop_map(|_| {
            let mut graph = BasicGraph::new(GraphType::Generic);
            graph.add_node(GenericNode::new("single", "data")).unwrap();
            graph
        }),
        
        // Complete graph (all nodes connected)
        (3..=10usize).prop_map(|n| {
            let mut graph = BasicGraph::new(GraphType::Generic);
            let nodes: Vec<_> = (0..n).map(|i| GenericNode::new(format!("n{}", i), "data")).collect();
            
            for node in &nodes {
                graph.add_node(node.clone()).unwrap();
            }
            
            for i in 0..n {
                for j in 0..n {
                    if i != j {
                        graph.add_edge(GenericEdge::new(
                            format!("n{}", i),
                            format!("n{}", j),
                            1.0,
                        )).unwrap();
                    }
                }
            }
            
            graph
        }),
        
        // Star graph (one central node connected to all others)
        (3..=20usize).prop_map(|n| {
            let mut graph = BasicGraph::new(GraphType::Generic);
            let center = GenericNode::new("center", "hub");
            graph.add_node(center.clone()).unwrap();
            
            for i in 0..n {
                let node = GenericNode::new(format!("leaf{}", i), "leaf");
                graph.add_node(node.clone()).unwrap();
                graph.add_edge(GenericEdge::new("center", node.id(), 1.0)).unwrap();
            }
            
            graph
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_node_id_generation_is_valid(id in node_id_strategy()) {
            assert!(!id.is_empty());
            assert!(id.chars().all(|c| c.is_alphanumeric() || c == '-'));
        }
        
        #[test]
        fn test_unique_nodes_have_unique_ids(nodes in nodes_strategy(0..20)) {
            let ids: HashSet<_> = nodes.iter().map(|n| n.id()).collect();
            assert_eq!(ids.len(), nodes.len());
        }
        
        #[test]
        fn test_generated_graphs_are_valid(graph in small_graph_strategy()) {
            // All nodes should be accessible
            for node_id in graph.node_ids() {
                assert!(graph.get_node(&node_id).is_some());
            }
            
            // All edges should be accessible and reference valid nodes
            for edge_id in graph.edge_ids() {
                let edge = graph.get_edge(&edge_id).unwrap();
                assert!(graph.contains_node(&edge.source()));
                assert!(graph.contains_node(&edge.target()));
            }
        }
    }
}