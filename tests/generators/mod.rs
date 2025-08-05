//! Graph generators for testing
//!
//! This module provides various graph generators for creating test data
//! with specific patterns and properties.

use cim_graph::core::{GraphBuilder, GraphType};
use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::graphs::ipld::{IpldGraph, IpldNode};
use cim_graph::graphs::context::{ContextGraph, ContextNode};
use cim_graph::graphs::concept::{ConceptGraph, ConceptNode, SemanticRelation};
use rand::prelude::*;
use std::collections::HashSet;

/// Trait for generating graphs with specific patterns
pub trait GraphGenerator<G> {
    /// Generate a random graph with the specified number of nodes and edges
    fn generate_random(&self, nodes: usize, edges: usize) -> G;
    
    /// Generate a graph with a specific pattern
    fn generate_pattern(&self, pattern: GraphPattern) -> G;
    
    /// Generate a pathological case for stress testing
    fn generate_pathological(&self, case: PathologicalCase) -> G;
}

/// Common graph patterns for testing
#[derive(Debug, Clone, Copy)]
pub enum GraphPattern {
    /// Complete graph where every node is connected to every other node
    Complete(usize),
    /// Bipartite graph with two sets of nodes
    Bipartite(usize, usize),
    /// Tree structure with specified depth and branching factor
    Tree { depth: usize, branching: usize },
    /// Cycle graph with n nodes
    Cycle(usize),
    /// Directed acyclic graph
    DAG { layers: usize, width: usize },
    /// Small-world network
    SmallWorld { nodes: usize, neighbors: usize, rewire_prob: f64 },
    /// Scale-free network (preferential attachment)
    ScaleFree { nodes: usize, edges_per_node: usize },
    /// Grid/lattice structure
    Grid { rows: usize, cols: usize },
    /// Star graph with one central node
    Star(usize),
}

/// Pathological cases for stress testing
#[derive(Debug, Clone, Copy)]
pub enum PathologicalCase {
    /// Node with maximum degree (connected to all other nodes)
    MaxDegree(usize),
    /// Deep nesting/hierarchy
    DeepNesting(usize),
    /// Large cycle that uses all nodes
    LargeCycle(usize),
    /// Many disconnected components
    Disconnected { components: usize, size: usize },
    /// Extremely dense graph (near-complete)
    Dense(usize),
    /// Long chain of nodes
    LongChain(usize),
    /// Many parallel edges between same nodes
    ParallelEdges { nodes: usize, edges_per_pair: usize },
}

/// Generator for WorkflowGraph
pub struct WorkflowGraphGenerator;

impl GraphGenerator<WorkflowGraph> for WorkflowGraphGenerator {
    fn generate_random(&self, nodes: usize, edges: usize) -> WorkflowGraph {
        let mut rng = thread_rng();
        let mut graph = WorkflowGraph::new();
        
        // Add nodes
        for i in 0..nodes {
            let state_type = match i {
                0 => StateType::Start,
                n if n == nodes - 1 => StateType::End,
                _ => StateType::Normal,
            };
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                state_type,
            );
            graph.add_state(node).unwrap();
        }
        
        // Add random edges
        let node_ids: Vec<_> = (0..nodes).map(|i| format!("node_{}", i)).collect();
        let mut added_edges = HashSet::new();
        let mut edge_count = 0;
        
        while edge_count < edges && edge_count < nodes * (nodes - 1) {
            let from = node_ids.choose(&mut rng).unwrap();
            let to = node_ids.choose(&mut rng).unwrap();
            
            if from != to && !added_edges.contains(&(from.clone(), to.clone())) {
                if graph.add_transition(from, to, &format!("edge_{}", edge_count)).is_ok() {
                    added_edges.insert((from.clone(), to.clone()));
                    edge_count += 1;
                }
            }
        }
        
        graph
    }
    
    fn generate_pattern(&self, pattern: GraphPattern) -> WorkflowGraph {
        let mut graph = WorkflowGraph::new();
        
        match pattern {
            GraphPattern::Complete(n) => {
                // Add nodes
                for i in 0..n {
                    let node = WorkflowNode::new(
                        &format!("node_{}", i),
                        &format!("Node {}", i),
                        StateType::Normal,
                    );
                    graph.add_state(node).unwrap();
                }
                
                // Connect every node to every other node
                for i in 0..n {
                    for j in 0..n {
                        if i != j {
                            graph.add_transition(
                                &format!("node_{}", i),
                                &format!("node_{}", j),
                                &format!("edge_{}_{}", i, j),
                            ).unwrap();
                        }
                    }
                }
            }
            
            GraphPattern::Cycle(n) => {
                // Create a cycle of n nodes
                for i in 0..n {
                    let node = WorkflowNode::new(
                        &format!("node_{}", i),
                        &format!("Node {}", i),
                        StateType::Normal,
                    );
                    graph.add_state(node).unwrap();
                }
                
                for i in 0..n {
                    let from = format!("node_{}", i);
                    let to = format!("node_{}", (i + 1) % n);
                    graph.add_transition(&from, &to, &format!("edge_{}", i)).unwrap();
                }
            }
            
            GraphPattern::Tree { depth, branching } => {
                // Create root
                let root = WorkflowNode::new("root", "Root", StateType::Start);
                graph.add_state(root).unwrap();
                
                // BFS to create tree
                let mut queue = vec![("root".to_string(), 0)];
                let mut node_counter = 1;
                
                while let Some((parent, level)) = queue.pop() {
                    if level < depth {
                        for i in 0..branching {
                            let child_id = format!("node_{}", node_counter);
                            let child = WorkflowNode::new(
                                &child_id,
                                &format!("Node {}", node_counter),
                                if level == depth - 1 { StateType::End } else { StateType::Normal },
                            );
                            graph.add_state(child).unwrap();
                            graph.add_transition(
                                &parent,
                                &child_id,
                                &format!("edge_{}", node_counter),
                            ).unwrap();
                            
                            queue.insert(0, (child_id, level + 1));
                            node_counter += 1;
                        }
                    }
                }
            }
            
            GraphPattern::Star(n) => {
                // Create center node
                let center = WorkflowNode::new("center", "Center", StateType::Normal);
                graph.add_state(center).unwrap();
                
                // Create and connect outer nodes
                for i in 0..n {
                    let node_id = format!("outer_{}", i);
                    let node = WorkflowNode::new(&node_id, &format!("Outer {}", i), StateType::Normal);
                    graph.add_state(node).unwrap();
                    graph.add_transition("center", &node_id, &format!("edge_{}", i)).unwrap();
                }
            }
            
            _ => {
                // For other patterns, create a simple random graph
                return self.generate_random(10, 15);
            }
        }
        
        graph
    }
    
    fn generate_pathological(&self, case: PathologicalCase) -> WorkflowGraph {
        let mut graph = WorkflowGraph::new();
        
        match case {
            PathologicalCase::MaxDegree(n) => {
                // Create a hub node connected to all others
                let hub = WorkflowNode::new("hub", "Hub", StateType::Normal);
                graph.add_state(hub).unwrap();
                
                for i in 0..n {
                    let node_id = format!("node_{}", i);
                    let node = WorkflowNode::new(&node_id, &format!("Node {}", i), StateType::Normal);
                    graph.add_state(node).unwrap();
                    
                    // Bidirectional connections
                    graph.add_transition("hub", &node_id, &format!("out_{}", i)).unwrap();
                    graph.add_transition(&node_id, "hub", &format!("in_{}", i)).unwrap();
                }
            }
            
            PathologicalCase::DeepNesting(depth) => {
                // Create a very deep linear chain
                let mut prev = "root".to_string();
                let root = WorkflowNode::new(&prev, "Root", StateType::Start);
                graph.add_state(root).unwrap();
                
                for i in 1..=depth {
                    let current = format!("level_{}", i);
                    let node = WorkflowNode::new(
                        &current,
                        &format!("Level {}", i),
                        if i == depth { StateType::End } else { StateType::Normal },
                    );
                    graph.add_state(node).unwrap();
                    graph.add_transition(&prev, &current, &format!("edge_{}", i)).unwrap();
                    prev = current;
                }
            }
            
            PathologicalCase::LongChain(n) => {
                // Similar to deep nesting but horizontal
                for i in 0..n {
                    let node = WorkflowNode::new(
                        &format!("chain_{}", i),
                        &format!("Chain {}", i),
                        match i {
                            0 => StateType::Start,
                            _ if i == n - 1 => StateType::End,
                            _ => StateType::Normal,
                        },
                    );
                    graph.add_state(node).unwrap();
                    
                    if i > 0 {
                        graph.add_transition(
                            &format!("chain_{}", i - 1),
                            &format!("chain_{}", i),
                            &format!("link_{}", i),
                        ).unwrap();
                    }
                }
            }
            
            _ => {
                // For other cases, create a dense graph
                return self.generate_random(20, 100);
            }
        }
        
        graph
    }
}

/// Generator for IpldGraph
pub struct IpldGraphGenerator;

impl GraphGenerator<IpldGraph> for IpldGraphGenerator {
    fn generate_random(&self, nodes: usize, edges: usize) -> IpldGraph {
        let mut rng = thread_rng();
        let mut graph = IpldGraph::new();
        
        // Add blocks
        for i in 0..nodes {
            let block_id = format!("block_{}", i);
            let content: Vec<u8> = (0..rng.gen_range(10..100))
                .map(|_| rng.gen())
                .collect();
            graph.add_block(block_id, content).unwrap();
        }
        
        // Add random links
        let block_ids: Vec<_> = (0..nodes).map(|i| format!("block_{}", i)).collect();
        let mut edge_count = 0;
        
        while edge_count < edges && edge_count < nodes * (nodes - 1) {
            let from = block_ids.choose(&mut rng).unwrap();
            let to = block_ids.choose(&mut rng).unwrap();
            
            if from != to {
                if graph.add_link(&from, &to, &format!("link_{}", edge_count)).is_ok() {
                    edge_count += 1;
                }
            }
        }
        
        graph
    }
    
    fn generate_pattern(&self, pattern: GraphPattern) -> IpldGraph {
        match pattern {
            GraphPattern::Tree { depth, branching } => {
                let mut graph = IpldGraph::new();
                let root_content = b"root".to_vec();
                graph.add_block("root".to_string(), root_content).unwrap();
                
                let mut queue = vec![("root".to_string(), 0)];
                let mut counter = 0;
                
                while let Some((parent, level)) = queue.pop() {
                    if level < depth {
                        for i in 0..branching {
                            counter += 1;
                            let child_id = format!("block_{}", counter);
                            let content = format!("content_{}", counter).into_bytes();
                            graph.add_block(child_id.clone(), content).unwrap();
                            graph.add_link(&parent, &child_id, &format!("child_{}", i)).unwrap();
                            queue.insert(0, (child_id, level + 1));
                        }
                    }
                }
                
                graph
            }
            _ => self.generate_random(10, 15),
        }
    }
    
    fn generate_pathological(&self, case: PathologicalCase) -> IpldGraph {
        match case {
            PathologicalCase::Dense(n) => {
                let mut graph = IpldGraph::new();
                
                // Create blocks with large content
                for i in 0..n {
                    let block_id = format!("block_{}", i);
                    let content: Vec<u8> = vec![i as u8; 1000]; // 1KB per block
                    graph.add_block(block_id, content).unwrap();
                }
                
                // Create many links
                for i in 0..n {
                    for j in 0..n {
                        if i != j {
                            graph.add_link(
                                &format!("block_{}", i),
                                &format!("block_{}", j),
                                &format!("link_{}_{}", i, j),
                            ).ok();
                        }
                    }
                }
                
                graph
            }
            _ => self.generate_random(20, 50),
        }
    }
}

/// Generator for ConceptGraph
pub struct ConceptGraphGenerator;

impl GraphGenerator<ConceptGraph> for ConceptGraphGenerator {
    fn generate_random(&self, nodes: usize, edges: usize) -> ConceptGraph {
        let mut rng = thread_rng();
        let mut graph = ConceptGraph::new();
        
        // Add concepts
        for i in 0..nodes {
            let concept = ConceptNode::new(
                &format!("concept_{}", i),
                &format!("Concept {}", i),
                &format!("Description of concept {}", i),
            );
            graph.add_concept(concept).unwrap();
        }
        
        // Add random relations
        let concept_ids: Vec<_> = (0..nodes).map(|i| format!("concept_{}", i)).collect();
        let relations = vec![
            SemanticRelation::IsA,
            SemanticRelation::PartOf,
            SemanticRelation::RelatedTo,
            SemanticRelation::DependsOn,
        ];
        
        let mut edge_count = 0;
        while edge_count < edges {
            let from = concept_ids.choose(&mut rng).unwrap();
            let to = concept_ids.choose(&mut rng).unwrap();
            let relation = relations.choose(&mut rng).unwrap();
            
            if from != to {
                if graph.add_relation(from, to, *relation).is_ok() {
                    edge_count += 1;
                }
            }
        }
        
        graph
    }
    
    fn generate_pattern(&self, pattern: GraphPattern) -> ConceptGraph {
        let mut graph = ConceptGraph::new();
        
        match pattern {
            GraphPattern::Tree { depth, branching } => {
                // Create taxonomy tree
                let root = ConceptNode::new("Animal", "Animal", "Root of animal taxonomy");
                graph.add_concept(root).unwrap();
                
                let mut queue = vec![("Animal".to_string(), 0)];
                let categories = vec!["Mammal", "Bird", "Reptile", "Fish", "Insect"];
                let mut counter = 0;
                
                while let Some((parent, level)) = queue.pop() {
                    if level < depth {
                        for i in 0..branching.min(categories.len()) {
                            counter += 1;
                            let child_name = if level == 0 {
                                categories[i].to_string()
                            } else {
                                format!("{}_{}", categories[i % categories.len()], counter)
                            };
                            
                            let child = ConceptNode::new(
                                &child_name,
                                &child_name,
                                &format!("A type of {}", parent),
                            );
                            graph.add_concept(child).unwrap();
                            graph.add_relation(&parent, &child_name, SemanticRelation::IsA).unwrap();
                            
                            queue.insert(0, (child_name, level + 1));
                        }
                    }
                }
            }
            _ => return self.generate_random(10, 15),
        }
        
        graph
    }
    
    fn generate_pathological(&self, _case: PathologicalCase) -> ConceptGraph {
        // For concept graphs, pathological cases are less relevant
        // Create a highly interconnected knowledge graph
        let mut graph = ConceptGraph::new();
        let concepts = vec!["AI", "ML", "DL", "NLP", "CV", "RL", "GAN", "RNN", "CNN", "BERT"];
        
        for concept in &concepts {
            let node = ConceptNode::new(concept, concept, &format!("{} technology", concept));
            graph.add_concept(node).unwrap();
        }
        
        // Create many relationships
        for i in 0..concepts.len() {
            for j in 0..concepts.len() {
                if i != j {
                    let relation = if i < j {
                        SemanticRelation::RelatedTo
                    } else {
                        SemanticRelation::DependsOn
                    };
                    graph.add_relation(concepts[i], concepts[j], relation).ok();
                }
            }
        }
        
        graph
    }
}

/// Utility functions for generating test data
pub mod utils {
    use super::*;
    
    /// Generate a random string of given length
    pub fn random_string(len: usize) -> String {
        thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }
    
    /// Generate random bytes
    pub fn random_bytes(len: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        (0..len).map(|_| rng.gen()).collect()
    }
    
    /// Generate a random graph based on Erdős–Rényi model
    pub fn erdos_renyi_workflow(n: usize, p: f64) -> WorkflowGraph {
        let mut rng = thread_rng();
        let mut graph = WorkflowGraph::new();
        
        // Add nodes
        for i in 0..n {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                StateType::Normal,
            );
            graph.add_state(node).unwrap();
        }
        
        // Add edges with probability p
        for i in 0..n {
            for j in 0..n {
                if i != j && rng.gen_bool(p) {
                    graph.add_transition(
                        &format!("node_{}", i),
                        &format!("node_{}", j),
                        &format!("edge_{}_{}", i, j),
                    ).ok();
                }
            }
        }
        
        graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_generator() {
        let gen = WorkflowGraphGenerator;
        
        // Test random generation
        let graph = gen.generate_random(10, 15);
        assert_eq!(graph.graph().node_count(), 10);
        assert!(graph.graph().edge_count() <= 15);
        
        // Test pattern generation
        let complete = gen.generate_pattern(GraphPattern::Complete(5));
        assert_eq!(complete.graph().node_count(), 5);
        assert_eq!(complete.graph().edge_count(), 20); // 5 * 4
        
        let cycle = gen.generate_pattern(GraphPattern::Cycle(6));
        assert_eq!(cycle.graph().node_count(), 6);
        assert_eq!(cycle.graph().edge_count(), 6);
    }
    
    #[test]
    fn test_ipld_generator() {
        let gen = IpldGraphGenerator;
        
        let graph = gen.generate_random(8, 12);
        assert_eq!(graph.graph().node_count(), 8);
        
        let tree = gen.generate_pattern(GraphPattern::Tree { depth: 3, branching: 2 });
        assert!(tree.graph().node_count() > 1);
    }
}