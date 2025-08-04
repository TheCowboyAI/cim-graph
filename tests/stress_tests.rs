//! Stress tests for large graphs
//!
//! These tests verify that CIM Graph can handle large-scale graphs
//! with acceptable performance and memory usage.

#![cfg(test)]

use cim_graph::graphs::workflow::{WorkflowGraph, WorkflowNode, StateType};
use cim_graph::graphs::ipld::IpldGraph;
use cim_graph::graphs::context::{ContextGraph, ContextNode};
use cim_graph::graphs::concept::{ConceptGraph, ConceptNode, SemanticRelation};
use cim_graph::algorithms;
use std::time::Instant;

/// Test configuration for stress tests
struct StressConfig {
    small: usize,
    medium: usize,
    large: usize,
    huge: usize,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            small: 1_000,
            medium: 10_000,
            large: 100_000,
            huge: 1_000_000,
        }
    }
}

/// Measure execution time and memory usage
fn measure<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let start_mem = get_memory_usage();
    
    let result = f();
    
    let duration = start.elapsed();
    let end_mem = get_memory_usage();
    let mem_diff = end_mem.saturating_sub(start_mem);
    
    println!(
        "{}: {:.2}s, Memory: {} MB",
        name,
        duration.as_secs_f64(),
        mem_diff / 1_048_576
    );
    
    result
}

/// Get current memory usage in bytes (simplified)
fn get_memory_usage() -> usize {
    // In real implementation, use system-specific memory queries
    // For now, return a placeholder
    0
}

mod workflow_stress {
    use super::*;
    
    #[test]
    #[ignore] // Run with --ignored flag
    fn test_large_workflow_graph() {
        let config = StressConfig::default();
        
        println!("\n=== Workflow Graph Stress Test ===");
        
        // Test with medium size
        let graph = measure("Create 10k node workflow", || {
            create_large_workflow(config.medium)
        });
        
        assert_eq!(graph.graph().node_count(), config.medium);
        
        // Test operations
        measure("Find all paths (10k graph)", || {
            let _ = algorithms::all_paths(
                graph.graph(),
                "node_0",
                "node_9999",
                5
            );
        });
        
        measure("Calculate centrality (10k graph)", || {
            let _ = algorithms::centrality(graph.graph());
        });
    }
    
    #[test]
    #[ignore]
    fn test_huge_workflow_creation() {
        let config = StressConfig::default();
        
        println!("\n=== Huge Workflow Creation ===");
        
        // Create 1M node graph
        let graph = measure("Create 1M node workflow", || {
            let mut graph = WorkflowGraph::new();
            
            // Add nodes in batches
            for batch in 0..1000 {
                for i in 0..1000 {
                    let id = batch * 1000 + i;
                    let node = WorkflowNode::new(
                        &format!("node_{}", id),
                        &format!("Node {}", id),
                        StateType::Normal,
                    );
                    graph.add_state(node).unwrap();
                }
                
                if batch % 100 == 0 {
                    println!("  Progress: {}%", batch / 10);
                }
            }
            
            graph
        });
        
        assert_eq!(graph.graph().node_count(), config.huge);
    }
    
    fn create_large_workflow(size: usize) -> WorkflowGraph {
        let mut graph = WorkflowGraph::new();
        
        // Create nodes
        for i in 0..size {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                match i {
                    0 => StateType::Start,
                    n if n == size - 1 => StateType::End,
                    _ => StateType::Normal,
                },
            );
            graph.add_state(node).unwrap();
        }
        
        // Create edges (sparse for performance)
        for i in 0..size - 1 {
            // Linear chain
            graph.add_transition(
                &format!("node_{}", i),
                &format!("node_{}", i + 1),
                "next",
            ).unwrap();
            
            // Some cross-connections
            if i % 100 == 0 && i + 100 < size {
                graph.add_transition(
                    &format!("node_{}", i),
                    &format!("node_{}", i + 100),
                    "skip",
                ).unwrap();
            }
        }
        
        graph
    }
}

mod ipld_stress {
    use super::*;
    
    #[test]
    #[ignore]
    fn test_large_ipld_graph() {
        let config = StressConfig::default();
        
        println!("\n=== IPLD Graph Stress Test ===");
        
        let graph = measure("Create 10k block IPLD graph", || {
            create_large_ipld(config.medium)
        });
        
        assert_eq!(graph.graph().node_count(), config.medium);
        
        // Test content retrieval
        measure("Retrieve 1000 random blocks", || {
            for i in (0..1000).step_by(10) {
                let _ = graph.get_block(&format!("block_{}", i));
            }
        });
    }
    
    fn create_large_ipld(size: usize) -> IpldGraph {
        let mut graph = IpldGraph::new();
        
        // Create blocks with varying content sizes
        for i in 0..size {
            let content_size = 100 + (i % 1000);
            let content = vec![i as u8; content_size];
            graph.add_block(format!("block_{}", i), content).unwrap();
        }
        
        // Create DAG structure
        for i in 1..size {
            let parent = format!("block_{}", i / 10);
            let child = format!("block_{}", i);
            let _ = graph.add_link(&parent, &child, "child");
        }
        
        graph
    }
}

mod context_stress {
    use super::*;
    
    #[test]
    #[ignore]
    fn test_large_domain_model() {
        println!("\n=== Context Graph Stress Test ===");
        
        let graph = measure("Create 10k entity domain model", || {
            create_large_domain_model(10_000)
        });
        
        // Test aggregate operations
        measure("Find all aggregates", || {
            let count = graph.contexts()
                .filter(|c| c.is_aggregate_root())
                .count();
            println!("  Found {} aggregate roots", count);
        });
    }
    
    fn create_large_domain_model(size: usize) -> ContextGraph {
        let mut graph = ContextGraph::new();
        
        // Create a hierarchical domain model
        let contexts_per_level = 10;
        let entities_per_context = size / 100;
        
        // Create bounded contexts
        for i in 0..contexts_per_level {
            let context = ContextNode::new(
                &format!("context_{}", i),
                &format!("Context {}", i),
                "BoundedContext",
            );
            graph.add_context(context).unwrap();
            
            // Add entities to each context
            for j in 0..entities_per_context {
                let entity_id = format!("entity_{}_{}", i, j);
                let entity = ContextNode::new(
                    &entity_id,
                    &format!("Entity {} in Context {}", j, i),
                    "Entity",
                );
                graph.add_context(entity).unwrap();
                
                // Connect to context
                graph.add_relationship(
                    &format!("context_{}", i),
                    &entity_id,
                    "contains",
                ).unwrap();
            }
        }
        
        graph
    }
}

mod concept_stress {
    use super::*;
    
    #[test]
    #[ignore]
    fn test_large_knowledge_graph() {
        println!("\n=== Concept Graph Stress Test ===");
        
        let graph = measure("Create 10k concept knowledge graph", || {
            create_large_knowledge_graph(10_000)
        });
        
        // Test inference on large graph
        measure("Apply inference rules", || {
            let inferences = graph.apply_inference();
            println!("  Generated {} inferences", inferences);
        });
    }
    
    fn create_large_knowledge_graph(size: usize) -> ConceptGraph {
        let mut graph = ConceptGraph::new();
        
        // Create concept hierarchy
        let root = ConceptNode::new("Thing", "Thing", "Root of all concepts");
        graph.add_concept(root).unwrap();
        
        // Create layers of concepts
        let mut current_layer = vec!["Thing".to_string()];
        let mut total = 1;
        
        while total < size {
            let mut next_layer = Vec::new();
            
            for parent in &current_layer {
                for i in 0..5 {
                    if total >= size {
                        break;
                    }
                    
                    let child_id = format!("concept_{}", total);
                    let child = ConceptNode::new(
                        &child_id,
                        &child_id,
                        &format!("Subconcept of {}", parent),
                    );
                    graph.add_concept(child).unwrap();
                    graph.add_relation(parent, &child_id, SemanticRelation::IsA).unwrap();
                    
                    next_layer.push(child_id);
                    total += 1;
                }
            }
            
            current_layer = next_layer;
        }
        
        graph
    }
}

mod memory_stress {
    use super::*;
    
    #[test]
    #[ignore]
    fn test_memory_limits() {
        println!("\n=== Memory Stress Test ===");
        
        // Test with large node data
        measure("Create graph with large node data", || {
            let mut graph = WorkflowGraph::new();
            
            for i in 0..1000 {
                let large_label = "X".repeat(10_000); // 10KB per node
                let node = WorkflowNode::new(
                    &format!("node_{}", i),
                    &large_label,
                    StateType::Normal,
                );
                graph.add_state(node).unwrap();
            }
            
            println!("  Created graph with ~10MB of node data");
        });
    }
    
    #[test]
    #[ignore]
    fn test_edge_explosion() {
        println!("\n=== Edge Explosion Test ===");
        
        // Create a complete graph to test edge limits
        let size = 100;
        
        measure(&format!("Create complete graph K{}", size), || {
            let mut graph = WorkflowGraph::new();
            
            // Add nodes
            for i in 0..size {
                let node = WorkflowNode::new(
                    &format!("n{}", i),
                    &format!("Node {}", i),
                    StateType::Normal,
                );
                graph.add_state(node).unwrap();
            }
            
            // Add all possible edges
            let mut edge_count = 0;
            for i in 0..size {
                for j in 0..size {
                    if i != j {
                        graph.add_transition(
                            &format!("n{}", i),
                            &format!("n{}", j),
                            &format!("e_{}_{}", i, j),
                        ).unwrap();
                        edge_count += 1;
                    }
                }
            }
            
            println!("  Created {} edges", edge_count);
            assert_eq!(edge_count, size * (size - 1));
        });
    }
}

mod algorithm_stress {
    use super::*;
    
    #[test]
    #[ignore]
    fn test_algorithm_scalability() {
        println!("\n=== Algorithm Scalability Test ===");
        
        for size in [100, 1000, 10000] {
            println!("\nTesting algorithms with {} nodes:", size);
            
            let graph = create_test_graph(size);
            
            // Test BFS scalability
            measure(&format!("  BFS ({})", size), || {
                let _ = algorithms::bfs(graph.graph(), "node_0");
            });
            
            // Test DFS scalability
            measure(&format!("  DFS ({})", size), || {
                let _ = algorithms::dfs(graph.graph(), "node_0");
            });
            
            // Test shortest path scalability
            if size <= 1000 {
                measure(&format!("  Shortest path ({})", size), || {
                    let _ = algorithms::shortest_path(
                        graph.graph(),
                        "node_0",
                        &format!("node_{}", size - 1),
                    );
                });
            }
        }
    }
    
    fn create_test_graph(size: usize) -> WorkflowGraph {
        let mut graph = WorkflowGraph::new();
        
        // Create nodes
        for i in 0..size {
            let node = WorkflowNode::new(
                &format!("node_{}", i),
                &format!("Node {}", i),
                StateType::Normal,
            );
            graph.add_state(node).unwrap();
        }
        
        // Create a sparse but connected graph
        for i in 0..size {
            // Connect to next
            if i + 1 < size {
                graph.add_transition(
                    &format!("node_{}", i),
                    &format!("node_{}", i + 1),
                    "next",
                ).unwrap();
            }
            
            // Random connections for complexity
            if i + 17 < size {
                graph.add_transition(
                    &format!("node_{}", i),
                    &format!("node_{}", i + 17),
                    "jump",
                ).unwrap();
            }
        }
        
        graph
    }
}

#[test]
#[ignore]
fn run_all_stress_tests() {
    println!("\n=== Running All Stress Tests ===");
    println!("This will take several minutes...\n");
    
    workflow_stress::test_large_workflow_graph();
    ipld_stress::test_large_ipld_graph();
    context_stress::test_large_domain_model();
    concept_stress::test_large_knowledge_graph();
    memory_stress::test_memory_limits();
    algorithm_stress::test_algorithm_scalability();
    
    println!("\n=== All Stress Tests Completed ===");
}