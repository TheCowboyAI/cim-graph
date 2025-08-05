//! Composed graph - multi-domain compositions (event-driven projection)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub use crate::core::projection_engine::GenericGraphProjection;
pub use crate::core::{Node, Edge};

/// Composed graph projection
pub type ComposedGraph = GenericGraphProjection<ComposedNode, ComposedEdge>;

/// Composed projection with additional multi-graph methods
pub type ComposedProjection = ComposedGraph;

/// Type of graph being composed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphDomain {
    /// IPLD graph reference
    Ipld { graph_id: Uuid },
    /// Context graph reference
    Context { graph_id: Uuid },
    /// Workflow graph reference
    Workflow { graph_id: Uuid },
    /// Concept graph reference
    Concept { graph_id: Uuid },
    /// Another composed graph reference
    Composed { graph_id: Uuid },
}

/// Type of composed node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComposedNodeType {
    /// Reference to a graph
    GraphReference { domain: GraphDomain },
    /// Reference to a node in another graph
    NodeReference { graph_id: Uuid, node_id: String },
    /// Junction node connecting multiple graphs
    Junction { connected_graphs: Vec<Uuid> },
    /// Transformation node
    Transform { operation: String },
    /// Aggregation node
    Aggregate { aggregation_type: String },
}

/// Composed node represents elements from multiple graph domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedNode {
    pub id: String,
    pub node_type: ComposedNodeType,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ComposedNode {
    /// Create a new composed node
    pub fn new(id: impl Into<String>, node_type: ComposedNodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            metadata: HashMap::new(),
        }
    }

    /// Create a graph reference node
    pub fn graph_ref(id: impl Into<String>, domain: GraphDomain) -> Self {
        Self::new(id, ComposedNodeType::GraphReference { domain })
    }

    /// Create an IPLD graph reference
    pub fn ipld_ref(id: impl Into<String>, graph_id: Uuid) -> Self {
        Self::graph_ref(id, GraphDomain::Ipld { graph_id })
    }

    /// Create a context graph reference
    pub fn context_ref(id: impl Into<String>, graph_id: Uuid) -> Self {
        Self::graph_ref(id, GraphDomain::Context { graph_id })
    }

    /// Create a workflow graph reference
    pub fn workflow_ref(id: impl Into<String>, graph_id: Uuid) -> Self {
        Self::graph_ref(id, GraphDomain::Workflow { graph_id })
    }

    /// Create a concept graph reference
    pub fn concept_ref(id: impl Into<String>, graph_id: Uuid) -> Self {
        Self::graph_ref(id, GraphDomain::Concept { graph_id })
    }

    /// Create a node reference
    pub fn node_ref(id: impl Into<String>, graph_id: Uuid, node_id: impl Into<String>) -> Self {
        Self::new(
            id,
            ComposedNodeType::NodeReference {
                graph_id,
                node_id: node_id.into(),
            },
        )
    }

    /// Create a junction node
    pub fn junction(id: impl Into<String>, connected_graphs: Vec<Uuid>) -> Self {
        Self::new(id, ComposedNodeType::Junction { connected_graphs })
    }

    /// Create a transformation node
    pub fn transform(id: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::new(
            id,
            ComposedNodeType::Transform {
                operation: operation.into(),
            },
        )
    }

    /// Create an aggregation node
    pub fn aggregate(id: impl Into<String>, aggregation_type: impl Into<String>) -> Self {
        Self::new(
            id,
            ComposedNodeType::Aggregate {
                aggregation_type: aggregation_type.into(),
            },
        )
    }
}

impl Node for ComposedNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Type of composed edge
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComposedEdgeType {
    /// Cross-graph link
    CrossGraphLink {
        source_graph: Uuid,
        target_graph: Uuid,
    },
    /// Data flow
    DataFlow { flow_type: String },
    /// Control flow
    ControlFlow,
    /// Dependency
    Dependency { dependency_type: String },
    /// Transformation
    Transformation { transform: String },
    /// Synchronization
    Synchronization,
}

/// Composed edge represents relationships across graph domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: ComposedEdgeType,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ComposedEdge {
    /// Create a new composed edge
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        edge_type: ComposedEdgeType,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            edge_type,
            metadata: HashMap::new(),
        }
    }

    /// Create a cross-graph link
    pub fn cross_graph_link(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        source_graph: Uuid,
        target_graph: Uuid,
    ) -> Self {
        Self::new(
            id,
            source,
            target,
            ComposedEdgeType::CrossGraphLink {
                source_graph,
                target_graph,
            },
        )
    }

    /// Create a data flow edge
    pub fn data_flow(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        flow_type: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            source,
            target,
            ComposedEdgeType::DataFlow {
                flow_type: flow_type.into(),
            },
        )
    }

    /// Create a control flow edge
    pub fn control_flow(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, ComposedEdgeType::ControlFlow)
    }

    /// Create a dependency edge
    pub fn dependency(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        dependency_type: impl Into<String>,
    ) -> Self {
        Self::new(
            id,
            source,
            target,
            ComposedEdgeType::Dependency {
                dependency_type: dependency_type.into(),
            },
        )
    }

    /// Create a synchronization edge
    pub fn synchronization(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, ComposedEdgeType::Synchronization)
    }
}

impl Edge for ComposedEdge {
    fn id(&self) -> String {
        self.id.clone()
    }
    fn source(&self) -> String {
        self.source.clone()
    }
    fn target(&self) -> String {
        self.target.clone()
    }
}

/// Extension methods for ComposedProjection
impl ComposedProjection {
    /// Get all graph references
    pub fn get_graph_references(&self) -> Vec<&ComposedNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, ComposedNodeType::GraphReference { .. }))
            .collect()
    }

    /// Get all graphs of a specific domain
    pub fn get_graphs_by_domain(&self, domain_filter: impl Fn(&GraphDomain) -> bool) -> Vec<&ComposedNode> {
        self.nodes()
            .filter(|n| match &n.node_type {
                ComposedNodeType::GraphReference { domain } => domain_filter(domain),
                _ => false,
            })
            .collect()
    }

    /// Get all IPLD graph references
    pub fn get_ipld_graphs(&self) -> Vec<(&ComposedNode, Uuid)> {
        self.nodes()
            .filter_map(|n| match &n.node_type {
                ComposedNodeType::GraphReference { domain: GraphDomain::Ipld { graph_id } } => {
                    Some((n, *graph_id))
                }
                _ => None,
            })
            .collect()
    }

    /// Get all junction nodes
    pub fn get_junctions(&self) -> Vec<&ComposedNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, ComposedNodeType::Junction { .. }))
            .collect()
    }

    /// Get all cross-graph links
    pub fn get_cross_graph_links(&self) -> Vec<&ComposedEdge> {
        self.edges()
            .filter(|e| matches!(e.edge_type, ComposedEdgeType::CrossGraphLink { .. }))
            .collect()
    }

    /// Find graphs connected through a junction
    pub fn get_connected_graphs(&self, junction_id: &str) -> Vec<Uuid> {
        self.get_node(junction_id)
            .and_then(|n| match &n.node_type {
                ComposedNodeType::Junction { connected_graphs } => Some(connected_graphs.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Find all paths between two graphs
    pub fn find_graph_paths(&self, from_graph: Uuid, to_graph: Uuid) -> Vec<Vec<String>> {
        // Find nodes representing these graphs
        let from_nodes: Vec<_> = self
            .nodes()
            .filter(|n| match &n.node_type {
                ComposedNodeType::GraphReference { domain } => match domain {
                    GraphDomain::Ipld { graph_id }
                    | GraphDomain::Context { graph_id }
                    | GraphDomain::Workflow { graph_id }
                    | GraphDomain::Concept { graph_id }
                    | GraphDomain::Composed { graph_id } => *graph_id == from_graph,
                },
                _ => false,
            })
            .map(|n| n.id.clone())
            .collect();

        let to_nodes: Vec<_> = self
            .nodes()
            .filter(|n| match &n.node_type {
                ComposedNodeType::GraphReference { domain } => match domain {
                    GraphDomain::Ipld { graph_id }
                    | GraphDomain::Context { graph_id }
                    | GraphDomain::Workflow { graph_id }
                    | GraphDomain::Concept { graph_id }
                    | GraphDomain::Composed { graph_id } => *graph_id == to_graph,
                },
                _ => false,
            })
            .map(|n| n.id.clone())
            .collect();

        let mut all_paths = Vec::new();
        for from in &from_nodes {
            for to in &to_nodes {
                all_paths.extend(self.find_paths_between(from, to));
            }
        }
        all_paths
    }

    fn find_paths_between(&self, from: &str, to: &str) -> Vec<Vec<String>> {
        use std::collections::{VecDeque, HashSet};
        
        let mut paths = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(vec![from.to_string()]);
        
        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();
            
            if current == to {
                paths.push(path);
                continue;
            }
            
            if path.len() > 10 { // Prevent infinite loops
                continue;
            }
            
            for edge in self.edges().filter(|e| e.source() == *current) {
                let next = edge.target();
                if !path.contains(&next) {
                    let mut new_path = path.clone();
                    new_path.push(next);
                    queue.push_back(new_path);
                }
            }
        }
        
        paths
    }

    /// Get all transformations in the composition
    pub fn get_transformations(&self) -> Vec<&ComposedNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, ComposedNodeType::Transform { .. }))
            .collect()
    }

    /// Get all aggregations in the composition
    pub fn get_aggregations(&self) -> Vec<&ComposedNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, ComposedNodeType::Aggregate { .. }))
            .collect()
    }

    /// Validate the composition
    pub fn validate(&self) -> Result<(), String> {
        // Check for orphaned graph references
        for node in self.get_graph_references() {
            let has_connections = self.edges().any(|e| e.source() == node.id || e.target() == node.id);
            if !has_connections {
                return Err(format!("Graph reference {} is not connected", node.id));
            }
        }

        // Check for cyclic dependencies in control flow
        for edge in self.edges() {
            if matches!(edge.edge_type, ComposedEdgeType::ControlFlow) {
                if self.has_cycle_from(&edge.source) {
                    return Err("Cyclic control flow detected".to_string());
                }
            }
        }

        Ok(())
    }

    fn has_cycle_from(&self, start: &str) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        self.dfs_cycle_check(start, &mut visited, &mut rec_stack)
    }

    fn dfs_cycle_check(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        
        for edge in self.edges().filter(|e| e.source() == node && matches!(e.edge_type, ComposedEdgeType::ControlFlow)) {
            let target = edge.target();
            if !visited.contains(&target) {
                if self.dfs_cycle_check(&target, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&target) {
                return true;
            }
        }
        
        rec_stack.remove(node);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composed_node_creation() {
        let ipld_id = Uuid::new_v4();
        let ipld_ref = ComposedNode::ipld_ref("ref1", ipld_id);
        assert!(matches!(
            ipld_ref.node_type,
            ComposedNodeType::GraphReference { domain: GraphDomain::Ipld { .. } }
        ));

        let junction = ComposedNode::junction("j1", vec![ipld_id]);
        assert!(matches!(
            junction.node_type,
            ComposedNodeType::Junction { connected_graphs } if connected_graphs.len() == 1
        ));
    }

    #[test]
    fn test_composed_edge_creation() {
        let source_graph = Uuid::new_v4();
        let target_graph = Uuid::new_v4();
        
        let cross_link = ComposedEdge::cross_graph_link(
            "link1",
            "node1",
            "node2",
            source_graph,
            target_graph,
        );
        assert!(matches!(
            cross_link.edge_type,
            ComposedEdgeType::CrossGraphLink { .. }
        ));

        let data_flow = ComposedEdge::data_flow("flow1", "transform1", "aggregate1", "json");
        assert!(matches!(
            data_flow.edge_type,
            ComposedEdgeType::DataFlow { flow_type } if flow_type == "json"
        ));
    }
}