//! Composed graph - multi-domain compositions (event-driven projection)

// Projections are ephemeral - no serialization
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub use crate::core::projection_engine::GenericGraphProjection;
pub use crate::core::{Node, Edge};

/// Composed graph projection
pub type ComposedGraph = GenericGraphProjection<ComposedNode, ComposedEdge>;

/// Composed projection with additional multi-graph methods
pub type ComposedProjection = ComposedGraph;

/// Type of graph being composed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphDomain {
    /// IPLD graph reference
    Ipld { 
        /// ID of the IPLD graph being referenced
        graph_id: Uuid 
    },
    /// Context graph reference
    Context { 
        /// ID of the context graph being referenced
        graph_id: Uuid 
    },
    /// Workflow graph reference
    Workflow { 
        /// ID of the workflow graph being referenced
        graph_id: Uuid 
    },
    /// Concept graph reference
    Concept { 
        /// ID of the concept graph being referenced
        graph_id: Uuid 
    },
    /// Another composed graph reference
    Composed { 
        /// ID of the composed graph being referenced
        graph_id: Uuid 
    },
}

/// Type of composed node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComposedNodeType {
    /// Reference to a graph
    GraphReference { 
        /// Domain and ID of the referenced graph
        domain: GraphDomain 
    },
    /// Reference to a node in another graph
    NodeReference { 
        /// ID of the graph containing the node
        graph_id: Uuid, 
        /// ID of the node within that graph
        node_id: String 
    },
    /// Junction node connecting multiple graphs
    Junction { 
        /// IDs of graphs connected at this junction
        connected_graphs: Vec<Uuid> 
    },
    /// Transformation node
    Transform { 
        /// Transformation operation to apply
        operation: String 
    },
    /// Aggregation node
    Aggregate { 
        /// Type of aggregation (sum, count, merge, etc.)
        aggregation_type: String 
    },
}

/// Composed node represents elements from multiple graph domains
#[derive(Debug, Clone)]
pub struct ComposedNode {
    /// Unique identifier for the node
    pub id: String,
    /// Type of composed node
    pub node_type: ComposedNodeType,
    /// Additional metadata for the node
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComposedEdgeType {
    /// Cross-graph link
    CrossGraphLink {
        /// ID of the source graph
        source_graph: Uuid,
        /// ID of the target graph
        target_graph: Uuid,
    },
    /// Data flow
    DataFlow { 
        /// Type of data flow (push, pull, stream, etc.)
        flow_type: String 
    },
    /// Control flow
    ControlFlow,
    /// Dependency
    Dependency { 
        /// Type of dependency (requires, provides, etc.)
        dependency_type: String 
    },
    /// Transformation
    Transformation { 
        /// Transformation operation
        transform: String 
    },
    /// Synchronization
    Synchronization,
}

/// Composed edge represents relationships across graph domains
#[derive(Debug, Clone)]
pub struct ComposedEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of composed edge
    pub edge_type: ComposedEdgeType,
    /// Additional metadata for the edge
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
        use std::collections::VecDeque;
        
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

    // ========== GraphDomain Tests ==========

    #[test]
    fn test_graph_domain_variants() {
        let id = Uuid::new_v4();

        let ipld = GraphDomain::Ipld { graph_id: id };
        let context = GraphDomain::Context { graph_id: id };
        let workflow = GraphDomain::Workflow { graph_id: id };
        let concept = GraphDomain::Concept { graph_id: id };
        let composed = GraphDomain::Composed { graph_id: id };

        assert!(matches!(ipld, GraphDomain::Ipld { graph_id } if graph_id == id));
        assert!(matches!(context, GraphDomain::Context { graph_id } if graph_id == id));
        assert!(matches!(workflow, GraphDomain::Workflow { graph_id } if graph_id == id));
        assert!(matches!(concept, GraphDomain::Concept { graph_id } if graph_id == id));
        assert!(matches!(composed, GraphDomain::Composed { graph_id } if graph_id == id));
    }

    #[test]
    fn test_graph_domain_equality() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let d1 = GraphDomain::Ipld { graph_id: id1 };
        let d2 = GraphDomain::Ipld { graph_id: id1 };
        let d3 = GraphDomain::Ipld { graph_id: id2 };
        let d4 = GraphDomain::Context { graph_id: id1 };

        assert_eq!(d1, d2);
        assert_ne!(d1, d3);
        assert_ne!(d1, d4);
    }

    #[test]
    fn test_graph_domain_hash() {
        let id = Uuid::new_v4();
        let domain = GraphDomain::Ipld { graph_id: id };

        let mut set = HashSet::new();
        set.insert(domain.clone());

        assert!(set.contains(&domain));
        assert_eq!(set.len(), 1);

        // Adding same domain should not increase size
        set.insert(domain);
        assert_eq!(set.len(), 1);
    }

    // ========== ComposedNodeType Tests ==========

    #[test]
    fn test_composed_node_type_variants() {
        let id = Uuid::new_v4();

        let graph_ref = ComposedNodeType::GraphReference {
            domain: GraphDomain::Ipld { graph_id: id }
        };
        let node_ref = ComposedNodeType::NodeReference {
            graph_id: id,
            node_id: "node1".to_string()
        };
        let junction = ComposedNodeType::Junction {
            connected_graphs: vec![id]
        };
        let transform = ComposedNodeType::Transform {
            operation: "filter".to_string()
        };
        let aggregate = ComposedNodeType::Aggregate {
            aggregation_type: "sum".to_string()
        };

        assert!(matches!(graph_ref, ComposedNodeType::GraphReference { .. }));
        assert!(matches!(node_ref, ComposedNodeType::NodeReference { .. }));
        assert!(matches!(junction, ComposedNodeType::Junction { .. }));
        assert!(matches!(transform, ComposedNodeType::Transform { .. }));
        assert!(matches!(aggregate, ComposedNodeType::Aggregate { .. }));
    }

    // ========== ComposedNode Factory Methods ==========

    #[test]
    fn test_composed_node_new() {
        let node = ComposedNode::new(
            "test_node",
            ComposedNodeType::Transform { operation: "map".to_string() }
        );

        assert_eq!(node.id, "test_node");
        assert!(node.metadata.is_empty());
        assert!(matches!(node.node_type, ComposedNodeType::Transform { operation } if operation == "map"));
    }

    #[test]
    fn test_composed_node_graph_ref() {
        let id = Uuid::new_v4();
        let domain = GraphDomain::Workflow { graph_id: id };
        let node = ComposedNode::graph_ref("ref1", domain.clone());

        assert_eq!(node.id, "ref1");
        assert!(matches!(
            node.node_type,
            ComposedNodeType::GraphReference { domain: d } if d == domain
        ));
    }

    #[test]
    fn test_composed_node_context_ref() {
        let id = Uuid::new_v4();
        let node = ComposedNode::context_ref("ctx_ref", id);

        assert_eq!(node.id, "ctx_ref");
        assert!(matches!(
            node.node_type,
            ComposedNodeType::GraphReference { domain: GraphDomain::Context { graph_id } } if graph_id == id
        ));
    }

    #[test]
    fn test_composed_node_workflow_ref() {
        let id = Uuid::new_v4();
        let node = ComposedNode::workflow_ref("wf_ref", id);

        assert_eq!(node.id, "wf_ref");
        assert!(matches!(
            node.node_type,
            ComposedNodeType::GraphReference { domain: GraphDomain::Workflow { graph_id } } if graph_id == id
        ));
    }

    #[test]
    fn test_composed_node_concept_ref() {
        let id = Uuid::new_v4();
        let node = ComposedNode::concept_ref("cpt_ref", id);

        assert_eq!(node.id, "cpt_ref");
        assert!(matches!(
            node.node_type,
            ComposedNodeType::GraphReference { domain: GraphDomain::Concept { graph_id } } if graph_id == id
        ));
    }

    #[test]
    fn test_composed_node_node_ref() {
        let graph_id = Uuid::new_v4();
        let node = ComposedNode::node_ref("ref1", graph_id, "internal_node");

        assert_eq!(node.id, "ref1");
        assert!(matches!(
            node.node_type,
            ComposedNodeType::NodeReference { graph_id: gid, node_id }
                if gid == graph_id && node_id == "internal_node"
        ));
    }

    #[test]
    fn test_composed_node_junction_multiple_graphs() {
        let ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let node = ComposedNode::junction("j1", ids.clone());

        assert_eq!(node.id, "j1");
        match node.node_type {
            ComposedNodeType::Junction { connected_graphs } => {
                assert_eq!(connected_graphs.len(), 3);
                assert_eq!(connected_graphs, ids);
            }
            _ => panic!("Expected Junction node type"),
        }
    }

    #[test]
    fn test_composed_node_transform() {
        let node = ComposedNode::transform("t1", "flatten");

        assert_eq!(node.id, "t1");
        match node.node_type {
            ComposedNodeType::Transform { operation } => {
                assert_eq!(operation, "flatten");
            }
            _ => panic!("Expected Transform node type"),
        }
    }

    #[test]
    fn test_composed_node_aggregate() {
        let node = ComposedNode::aggregate("agg1", "count");

        assert_eq!(node.id, "agg1");
        match node.node_type {
            ComposedNodeType::Aggregate { aggregation_type } => {
                assert_eq!(aggregation_type, "count");
            }
            _ => panic!("Expected Aggregate node type"),
        }
    }

    #[test]
    fn test_composed_node_implements_node_trait() {
        let node = ComposedNode::new("trait_test", ComposedNodeType::Junction { connected_graphs: vec![] });
        assert_eq!(Node::id(&node), "trait_test");
    }

    // ========== ComposedEdgeType Tests ==========

    #[test]
    fn test_composed_edge_type_variants() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let cross_link = ComposedEdgeType::CrossGraphLink { source_graph: id1, target_graph: id2 };
        let data_flow = ComposedEdgeType::DataFlow { flow_type: "stream".to_string() };
        let control = ComposedEdgeType::ControlFlow;
        let dep = ComposedEdgeType::Dependency { dependency_type: "requires".to_string() };
        let transform = ComposedEdgeType::Transformation { transform: "encode".to_string() };
        let sync = ComposedEdgeType::Synchronization;

        assert!(matches!(cross_link, ComposedEdgeType::CrossGraphLink { .. }));
        assert!(matches!(data_flow, ComposedEdgeType::DataFlow { .. }));
        assert!(matches!(control, ComposedEdgeType::ControlFlow));
        assert!(matches!(dep, ComposedEdgeType::Dependency { .. }));
        assert!(matches!(transform, ComposedEdgeType::Transformation { .. }));
        assert!(matches!(sync, ComposedEdgeType::Synchronization));
    }

    #[test]
    fn test_composed_edge_type_equality() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let e1 = ComposedEdgeType::ControlFlow;
        let e2 = ComposedEdgeType::ControlFlow;
        let e3 = ComposedEdgeType::Synchronization;

        assert_eq!(e1, e2);
        assert_ne!(e1, e3);

        let d1 = ComposedEdgeType::DataFlow { flow_type: "push".to_string() };
        let d2 = ComposedEdgeType::DataFlow { flow_type: "push".to_string() };
        let d3 = ComposedEdgeType::DataFlow { flow_type: "pull".to_string() };

        assert_eq!(d1, d2);
        assert_ne!(d1, d3);

        let c1 = ComposedEdgeType::CrossGraphLink { source_graph: id1, target_graph: id2 };
        let c2 = ComposedEdgeType::CrossGraphLink { source_graph: id1, target_graph: id2 };
        let c3 = ComposedEdgeType::CrossGraphLink { source_graph: id2, target_graph: id1 };

        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    // ========== ComposedEdge Factory Methods ==========

    #[test]
    fn test_composed_edge_new() {
        let edge = ComposedEdge::new(
            "e1",
            "src",
            "tgt",
            ComposedEdgeType::ControlFlow,
        );

        assert_eq!(edge.id, "e1");
        assert_eq!(edge.source, "src");
        assert_eq!(edge.target, "tgt");
        assert!(edge.metadata.is_empty());
        assert!(matches!(edge.edge_type, ComposedEdgeType::ControlFlow));
    }

    #[test]
    fn test_composed_edge_control_flow() {
        let edge = ComposedEdge::control_flow("cf1", "start", "end");

        assert_eq!(edge.id, "cf1");
        assert_eq!(edge.source, "start");
        assert_eq!(edge.target, "end");
        assert!(matches!(edge.edge_type, ComposedEdgeType::ControlFlow));
    }

    #[test]
    fn test_composed_edge_dependency() {
        let edge = ComposedEdge::dependency("dep1", "consumer", "provider", "requires");

        assert_eq!(edge.id, "dep1");
        assert_eq!(edge.source, "consumer");
        assert_eq!(edge.target, "provider");
        match edge.edge_type {
            ComposedEdgeType::Dependency { dependency_type } => {
                assert_eq!(dependency_type, "requires");
            }
            _ => panic!("Expected Dependency edge type"),
        }
    }

    #[test]
    fn test_composed_edge_synchronization() {
        let edge = ComposedEdge::synchronization("sync1", "primary", "replica");

        assert_eq!(edge.id, "sync1");
        assert_eq!(edge.source, "primary");
        assert_eq!(edge.target, "replica");
        assert!(matches!(edge.edge_type, ComposedEdgeType::Synchronization));
    }

    #[test]
    fn test_composed_edge_implements_edge_trait() {
        let edge = ComposedEdge::control_flow("trait_test", "a", "b");

        assert_eq!(Edge::id(&edge), "trait_test");
        assert_eq!(Edge::source(&edge), "a");
        assert_eq!(Edge::target(&edge), "b");
    }

    // ========== ComposedProjection Extension Methods ==========

    fn create_test_projection() -> ComposedProjection {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let ipld_id = Uuid::new_v4();
        let ctx_id = Uuid::new_v4();
        let wf_id = Uuid::new_v4();

        // Add graph reference nodes
        let ipld_ref = ComposedNode::ipld_ref("ipld1", ipld_id);
        let ctx_ref = ComposedNode::context_ref("ctx1", ctx_id);
        let wf_ref = ComposedNode::workflow_ref("wf1", wf_id);

        // Add junction
        let junction = ComposedNode::junction("j1", vec![ipld_id, ctx_id, wf_id]);

        // Add transform and aggregate
        let transform = ComposedNode::transform("t1", "filter");
        let aggregate = ComposedNode::aggregate("a1", "merge");

        projection.nodes.insert("ipld1".to_string(), ipld_ref);
        projection.nodes.insert("ctx1".to_string(), ctx_ref);
        projection.nodes.insert("wf1".to_string(), wf_ref);
        projection.nodes.insert("j1".to_string(), junction);
        projection.nodes.insert("t1".to_string(), transform);
        projection.nodes.insert("a1".to_string(), aggregate);

        // Add edges
        let cross_link = ComposedEdge::cross_graph_link("cl1", "ipld1", "ctx1", ipld_id, ctx_id);
        let data_flow = ComposedEdge::data_flow("df1", "t1", "a1", "json");
        let control = ComposedEdge::control_flow("cf1", "ctx1", "wf1");

        projection.edges.insert("cl1".to_string(), cross_link);
        projection.edges.insert("df1".to_string(), data_flow);
        projection.edges.insert("cf1".to_string(), control);

        // Update adjacency
        projection.adjacency.insert("ipld1".to_string(), vec!["ctx1".to_string()]);
        projection.adjacency.insert("ctx1".to_string(), vec!["wf1".to_string()]);
        projection.adjacency.insert("t1".to_string(), vec!["a1".to_string()]);
        projection.adjacency.insert("wf1".to_string(), vec![]);
        projection.adjacency.insert("j1".to_string(), vec![]);
        projection.adjacency.insert("a1".to_string(), vec![]);

        projection
    }

    #[test]
    fn test_get_graph_references() {
        let projection = create_test_projection();
        let refs = projection.get_graph_references();

        // Should find ipld1, ctx1, wf1 (3 graph references)
        assert_eq!(refs.len(), 3);

        let ids: Vec<_> = refs.iter().map(|n| n.id.as_str()).collect();
        assert!(ids.contains(&"ipld1"));
        assert!(ids.contains(&"ctx1"));
        assert!(ids.contains(&"wf1"));
    }

    #[test]
    fn test_get_graphs_by_domain() {
        let projection = create_test_projection();

        // Get only IPLD graphs
        let ipld_graphs = projection.get_graphs_by_domain(|d| matches!(d, GraphDomain::Ipld { .. }));
        assert_eq!(ipld_graphs.len(), 1);
        assert_eq!(ipld_graphs[0].id, "ipld1");

        // Get only context graphs
        let ctx_graphs = projection.get_graphs_by_domain(|d| matches!(d, GraphDomain::Context { .. }));
        assert_eq!(ctx_graphs.len(), 1);
        assert_eq!(ctx_graphs[0].id, "ctx1");

        // Get only workflow graphs
        let wf_graphs = projection.get_graphs_by_domain(|d| matches!(d, GraphDomain::Workflow { .. }));
        assert_eq!(wf_graphs.len(), 1);
        assert_eq!(wf_graphs[0].id, "wf1");

        // No concept graphs
        let cpt_graphs = projection.get_graphs_by_domain(|d| matches!(d, GraphDomain::Concept { .. }));
        assert_eq!(cpt_graphs.len(), 0);
    }

    #[test]
    fn test_get_ipld_graphs() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        projection.nodes.insert("ipld1".to_string(), ComposedNode::ipld_ref("ipld1", id1));
        projection.nodes.insert("ipld2".to_string(), ComposedNode::ipld_ref("ipld2", id2));
        projection.nodes.insert("ctx1".to_string(), ComposedNode::context_ref("ctx1", Uuid::new_v4()));

        let ipld_graphs = projection.get_ipld_graphs();

        assert_eq!(ipld_graphs.len(), 2);
        let graph_ids: Vec<_> = ipld_graphs.iter().map(|(_, gid)| *gid).collect();
        assert!(graph_ids.contains(&id1));
        assert!(graph_ids.contains(&id2));
    }

    #[test]
    fn test_get_junctions() {
        let projection = create_test_projection();
        let junctions = projection.get_junctions();

        assert_eq!(junctions.len(), 1);
        assert_eq!(junctions[0].id, "j1");
    }

    #[test]
    fn test_get_cross_graph_links() {
        let projection = create_test_projection();
        let links = projection.get_cross_graph_links();

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].id, "cl1");
    }

    #[test]
    fn test_get_connected_graphs() {
        let projection = create_test_projection();
        let connected = projection.get_connected_graphs("j1");

        assert_eq!(connected.len(), 3);
    }

    #[test]
    fn test_get_connected_graphs_nonexistent() {
        let projection = create_test_projection();
        let connected = projection.get_connected_graphs("nonexistent");

        assert!(connected.is_empty());
    }

    #[test]
    fn test_get_connected_graphs_non_junction() {
        let projection = create_test_projection();
        // t1 is a transform node, not a junction
        let connected = projection.get_connected_graphs("t1");

        assert!(connected.is_empty());
    }

    #[test]
    fn test_get_transformations() {
        let projection = create_test_projection();
        let transforms = projection.get_transformations();

        assert_eq!(transforms.len(), 1);
        assert_eq!(transforms[0].id, "t1");
    }

    #[test]
    fn test_get_aggregations() {
        let projection = create_test_projection();
        let aggregations = projection.get_aggregations();

        assert_eq!(aggregations.len(), 1);
        assert_eq!(aggregations[0].id, "a1");
    }

    // ========== Graph Path Finding Tests ==========

    #[test]
    fn test_find_graph_paths_direct() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        projection.nodes.insert("n1".to_string(), ComposedNode::ipld_ref("n1", g1));
        projection.nodes.insert("n2".to_string(), ComposedNode::ipld_ref("n2", g2));
        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "n1", "n2"));
        projection.adjacency.insert("n1".to_string(), vec!["n2".to_string()]);
        projection.adjacency.insert("n2".to_string(), vec![]);

        let paths = projection.find_graph_paths(g1, g2);

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["n1", "n2"]);
    }

    #[test]
    fn test_find_graph_paths_no_path() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        // Two disconnected nodes
        projection.nodes.insert("n1".to_string(), ComposedNode::ipld_ref("n1", g1));
        projection.nodes.insert("n2".to_string(), ComposedNode::ipld_ref("n2", g2));
        projection.adjacency.insert("n1".to_string(), vec![]);
        projection.adjacency.insert("n2".to_string(), vec![]);

        let paths = projection.find_graph_paths(g1, g2);

        assert!(paths.is_empty());
    }

    #[test]
    fn test_find_graph_paths_multiple_paths() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        // Diamond shape: n1 -> n2 -> n4, n1 -> n3 -> n4
        projection.nodes.insert("n1".to_string(), ComposedNode::ipld_ref("n1", g1));
        projection.nodes.insert("n2".to_string(), ComposedNode::transform("n2", "op1"));
        projection.nodes.insert("n3".to_string(), ComposedNode::transform("n3", "op2"));
        projection.nodes.insert("n4".to_string(), ComposedNode::ipld_ref("n4", g2));

        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "n1", "n2"));
        projection.edges.insert("e2".to_string(), ComposedEdge::control_flow("e2", "n1", "n3"));
        projection.edges.insert("e3".to_string(), ComposedEdge::control_flow("e3", "n2", "n4"));
        projection.edges.insert("e4".to_string(), ComposedEdge::control_flow("e4", "n3", "n4"));

        projection.adjacency.insert("n1".to_string(), vec!["n2".to_string(), "n3".to_string()]);
        projection.adjacency.insert("n2".to_string(), vec!["n4".to_string()]);
        projection.adjacency.insert("n3".to_string(), vec!["n4".to_string()]);
        projection.adjacency.insert("n4".to_string(), vec![]);

        let paths = projection.find_graph_paths(g1, g2);

        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_find_graph_paths_different_domains() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        // Test with different domain types that have same graph_id
        projection.nodes.insert("n1".to_string(), ComposedNode::context_ref("n1", g1));
        projection.nodes.insert("n2".to_string(), ComposedNode::workflow_ref("n2", g2));
        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "n1", "n2"));
        projection.adjacency.insert("n1".to_string(), vec!["n2".to_string()]);
        projection.adjacency.insert("n2".to_string(), vec![]);

        let paths = projection.find_graph_paths(g1, g2);

        assert_eq!(paths.len(), 1);
    }

    // ========== Validation Tests ==========

    #[test]
    fn test_validate_connected_graph_references() {
        let projection = create_test_projection();

        // All graph references have connections, should pass
        let result = projection.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_orphaned_graph_reference() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add a graph reference with no edges
        let orphan = ComposedNode::ipld_ref("orphan", Uuid::new_v4());
        projection.nodes.insert("orphan".to_string(), orphan);

        let result = projection.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not connected"));
    }

    #[test]
    fn test_validate_no_cycle_in_control_flow() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Linear control flow: a -> b -> c
        projection.nodes.insert("a".to_string(), ComposedNode::transform("a", "op1"));
        projection.nodes.insert("b".to_string(), ComposedNode::transform("b", "op2"));
        projection.nodes.insert("c".to_string(), ComposedNode::transform("c", "op3"));

        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));
        projection.edges.insert("e2".to_string(), ComposedEdge::control_flow("e2", "b", "c"));

        projection.adjacency.insert("a".to_string(), vec!["b".to_string()]);
        projection.adjacency.insert("b".to_string(), vec!["c".to_string()]);
        projection.adjacency.insert("c".to_string(), vec![]);

        let result = projection.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cycle_in_control_flow() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Cyclic control flow: a -> b -> c -> a
        projection.nodes.insert("a".to_string(), ComposedNode::transform("a", "op1"));
        projection.nodes.insert("b".to_string(), ComposedNode::transform("b", "op2"));
        projection.nodes.insert("c".to_string(), ComposedNode::transform("c", "op3"));

        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));
        projection.edges.insert("e2".to_string(), ComposedEdge::control_flow("e2", "b", "c"));
        projection.edges.insert("e3".to_string(), ComposedEdge::control_flow("e3", "c", "a"));

        projection.adjacency.insert("a".to_string(), vec!["b".to_string()]);
        projection.adjacency.insert("b".to_string(), vec!["c".to_string()]);
        projection.adjacency.insert("c".to_string(), vec!["a".to_string()]);

        let result = projection.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cyclic control flow"));
    }

    #[test]
    fn test_validate_data_flow_cycle_allowed() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Cyclic data flow (feedback loop) should be allowed
        projection.nodes.insert("a".to_string(), ComposedNode::transform("a", "op1"));
        projection.nodes.insert("b".to_string(), ComposedNode::transform("b", "op2"));

        projection.edges.insert("e1".to_string(), ComposedEdge::data_flow("e1", "a", "b", "stream"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "b", "a", "feedback"));

        projection.adjacency.insert("a".to_string(), vec!["b".to_string()]);
        projection.adjacency.insert("b".to_string(), vec!["a".to_string()]);

        let result = projection.validate();
        // No graph references, so validation should pass
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_graph() {
        let projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let result = projection.validate();
        // Empty graph should be valid
        assert!(result.is_ok());
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_empty_junction() {
        let node = ComposedNode::junction("empty_j", vec![]);

        match node.node_type {
            ComposedNodeType::Junction { connected_graphs } => {
                assert!(connected_graphs.is_empty());
            }
            _ => panic!("Expected Junction node type"),
        }
    }

    #[test]
    fn test_string_conversion() {
        // Test Into<String> trait usage
        let s: String = "test".to_string();
        let node1 = ComposedNode::new(&s, ComposedNodeType::Transform { operation: "op1".to_string() });
        let node2 = ComposedNode::new(s.clone(), ComposedNodeType::Transform { operation: "op2".to_string() });

        assert_eq!(node1.id, "test");
        assert_eq!(node2.id, "test");
    }

    // ========== Cross-Domain Query Tests ==========

    #[test]
    fn test_query_across_domains_ipld_to_context() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let ipld_id = Uuid::new_v4();
        let ctx_id = Uuid::new_v4();

        // Add IPLD and context references
        projection.nodes.insert("ipld1".to_string(), ComposedNode::ipld_ref("ipld1", ipld_id));
        projection.nodes.insert("ctx1".to_string(), ComposedNode::context_ref("ctx1", ctx_id));

        // Add cross-graph link
        let cross_link = ComposedEdge::cross_graph_link("link1", "ipld1", "ctx1", ipld_id, ctx_id);
        projection.edges.insert("link1".to_string(), cross_link);

        projection.adjacency.insert("ipld1".to_string(), vec!["ctx1".to_string()]);
        projection.adjacency.insert("ctx1".to_string(), vec![]);

        // Query: Find path from IPLD to Context
        let paths = projection.find_graph_paths(ipld_id, ctx_id);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["ipld1", "ctx1"]);
    }

    #[test]
    fn test_query_all_graphs_of_specific_type() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add multiple workflow references
        for i in 0..5 {
            let wf_id = Uuid::new_v4();
            projection.nodes.insert(format!("wf{}", i), ComposedNode::workflow_ref(format!("wf{}", i), wf_id));
        }

        // Add some non-workflow references
        projection.nodes.insert("ctx1".to_string(), ComposedNode::context_ref("ctx1", Uuid::new_v4()));
        projection.nodes.insert("ipld1".to_string(), ComposedNode::ipld_ref("ipld1", Uuid::new_v4()));

        // Query: Get all workflow graphs
        let workflows = projection.get_graphs_by_domain(|d| matches!(d, GraphDomain::Workflow { .. }));
        assert_eq!(workflows.len(), 5);

        // Query: Get all context graphs
        let contexts = projection.get_graphs_by_domain(|d| matches!(d, GraphDomain::Context { .. }));
        assert_eq!(contexts.len(), 1);
    }

    #[test]
    fn test_query_data_flow_chain() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Create data flow chain: source -> transform1 -> transform2 -> sink
        projection.nodes.insert("source".to_string(), ComposedNode::ipld_ref("source", Uuid::new_v4()));
        projection.nodes.insert("t1".to_string(), ComposedNode::transform("t1", "filter"));
        projection.nodes.insert("t2".to_string(), ComposedNode::transform("t2", "map"));
        projection.nodes.insert("sink".to_string(), ComposedNode::aggregate("sink", "collect"));

        // Add data flow edges
        projection.edges.insert("df1".to_string(), ComposedEdge::data_flow("df1", "source", "t1", "stream"));
        projection.edges.insert("df2".to_string(), ComposedEdge::data_flow("df2", "t1", "t2", "stream"));
        projection.edges.insert("df3".to_string(), ComposedEdge::data_flow("df3", "t2", "sink", "stream"));

        projection.adjacency.insert("source".to_string(), vec!["t1".to_string()]);
        projection.adjacency.insert("t1".to_string(), vec!["t2".to_string()]);
        projection.adjacency.insert("t2".to_string(), vec!["sink".to_string()]);
        projection.adjacency.insert("sink".to_string(), vec![]);

        // Query: Get all transformations in order
        let transforms = projection.get_transformations();
        assert_eq!(transforms.len(), 2);

        // Query: Get aggregations
        let aggregations = projection.get_aggregations();
        assert_eq!(aggregations.len(), 1);
        assert_eq!(aggregations[0].id, "sink");
    }

    #[test]
    fn test_get_all_nodes() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add various node types
        projection.nodes.insert("n1".to_string(), ComposedNode::ipld_ref("n1", Uuid::new_v4()));
        projection.nodes.insert("n2".to_string(), ComposedNode::transform("n2", "op"));
        projection.nodes.insert("n3".to_string(), ComposedNode::junction("n3", vec![]));

        // Get all nodes through iterator
        let all_nodes: Vec<_> = projection.nodes().collect();
        assert_eq!(all_nodes.len(), 3);
    }

    #[test]
    fn test_get_all_edges() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add nodes
        projection.nodes.insert("a".to_string(), ComposedNode::transform("a", "op1"));
        projection.nodes.insert("b".to_string(), ComposedNode::transform("b", "op2"));
        projection.nodes.insert("c".to_string(), ComposedNode::transform("c", "op3"));

        // Add various edge types
        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "b", "c", "json"));
        projection.edges.insert("e3".to_string(), ComposedEdge::synchronization("e3", "a", "c"));

        // Get all edges
        let all_edges: Vec<_> = projection.edges().collect();
        assert_eq!(all_edges.len(), 3);
    }

    #[test]
    fn test_validate_references_all_connected() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        // Add connected graph references
        projection.nodes.insert("g1".to_string(), ComposedNode::ipld_ref("g1", g1));
        projection.nodes.insert("g2".to_string(), ComposedNode::context_ref("g2", g2));

        projection.edges.insert("link".to_string(), ComposedEdge::cross_graph_link("link", "g1", "g2", g1, g2));

        projection.adjacency.insert("g1".to_string(), vec!["g2".to_string()]);
        projection.adjacency.insert("g2".to_string(), vec![]);

        // Validate should pass
        let result = projection.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_references_mixed_types() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add transform nodes (not graph references) - no edges needed for validation
        projection.nodes.insert("t1".to_string(), ComposedNode::transform("t1", "filter"));
        projection.nodes.insert("t2".to_string(), ComposedNode::aggregate("t2", "sum"));

        // Non-graph-reference nodes don't need connections for validation to pass
        let result = projection.validate();
        assert!(result.is_ok());
    }

    // ========== Advanced Path Finding Tests ==========

    #[test]
    fn test_find_paths_with_transform_intermediates() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        // Graph1 -> Transform1 -> Transform2 -> Graph2
        projection.nodes.insert("g1".to_string(), ComposedNode::ipld_ref("g1", g1));
        projection.nodes.insert("t1".to_string(), ComposedNode::transform("t1", "filter"));
        projection.nodes.insert("t2".to_string(), ComposedNode::transform("t2", "map"));
        projection.nodes.insert("g2".to_string(), ComposedNode::ipld_ref("g2", g2));

        projection.edges.insert("e1".to_string(), ComposedEdge::data_flow("e1", "g1", "t1", "stream"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "t1", "t2", "stream"));
        projection.edges.insert("e3".to_string(), ComposedEdge::data_flow("e3", "t2", "g2", "stream"));

        projection.adjacency.insert("g1".to_string(), vec!["t1".to_string()]);
        projection.adjacency.insert("t1".to_string(), vec!["t2".to_string()]);
        projection.adjacency.insert("t2".to_string(), vec!["g2".to_string()]);
        projection.adjacency.insert("g2".to_string(), vec![]);

        let paths = projection.find_graph_paths(g1, g2);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["g1", "t1", "t2", "g2"]);
    }

    #[test]
    fn test_find_paths_complex_diamond() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g4 = Uuid::new_v4();

        /*
            g1
           /  \
          t1   t2
           \  /
            t3
            |
            g4
        */

        projection.nodes.insert("g1".to_string(), ComposedNode::ipld_ref("g1", g1));
        projection.nodes.insert("t1".to_string(), ComposedNode::transform("t1", "path1"));
        projection.nodes.insert("t2".to_string(), ComposedNode::transform("t2", "path2"));
        projection.nodes.insert("t3".to_string(), ComposedNode::aggregate("t3", "merge"));
        projection.nodes.insert("g4".to_string(), ComposedNode::ipld_ref("g4", g4));

        projection.edges.insert("e1".to_string(), ComposedEdge::data_flow("e1", "g1", "t1", "s"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "g1", "t2", "s"));
        projection.edges.insert("e3".to_string(), ComposedEdge::data_flow("e3", "t1", "t3", "s"));
        projection.edges.insert("e4".to_string(), ComposedEdge::data_flow("e4", "t2", "t3", "s"));
        projection.edges.insert("e5".to_string(), ComposedEdge::data_flow("e5", "t3", "g4", "s"));

        projection.adjacency.insert("g1".to_string(), vec!["t1".to_string(), "t2".to_string()]);
        projection.adjacency.insert("t1".to_string(), vec!["t3".to_string()]);
        projection.adjacency.insert("t2".to_string(), vec!["t3".to_string()]);
        projection.adjacency.insert("t3".to_string(), vec!["g4".to_string()]);
        projection.adjacency.insert("g4".to_string(), vec![]);

        let paths = projection.find_graph_paths(g1, g4);
        // Should find 2 paths: g1->t1->t3->g4 and g1->t2->t3->g4
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_find_paths_self_loop_avoided() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        projection.nodes.insert("g1".to_string(), ComposedNode::ipld_ref("g1", g1));
        projection.nodes.insert("t1".to_string(), ComposedNode::transform("t1", "process"));
        projection.nodes.insert("g2".to_string(), ComposedNode::ipld_ref("g2", g2));

        // Add edges including a loop at t1
        projection.edges.insert("e1".to_string(), ComposedEdge::data_flow("e1", "g1", "t1", "s"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "t1", "g2", "s"));
        projection.edges.insert("loop".to_string(), ComposedEdge::data_flow("loop", "t1", "t1", "s")); // Self-loop

        projection.adjacency.insert("g1".to_string(), vec!["t1".to_string()]);
        projection.adjacency.insert("t1".to_string(), vec!["t1".to_string(), "g2".to_string()]);
        projection.adjacency.insert("g2".to_string(), vec![]);

        // Should find path without getting stuck in loop
        let paths = projection.find_graph_paths(g1, g2);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["g1", "t1", "g2"]);
    }

    // ========== Junction Tests ==========

    #[test]
    fn test_junction_with_many_graphs() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Create junction connecting 10 graphs
        let graph_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
        let junction = ComposedNode::junction("central_junction", graph_ids.clone());

        projection.nodes.insert("central_junction".to_string(), junction);

        let connected = projection.get_connected_graphs("central_junction");
        assert_eq!(connected.len(), 10);

        for id in &graph_ids {
            assert!(connected.contains(id));
        }
    }

    #[test]
    fn test_multiple_junctions() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let ids1 = vec![Uuid::new_v4(), Uuid::new_v4()];
        let ids2 = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

        projection.nodes.insert("j1".to_string(), ComposedNode::junction("j1", ids1.clone()));
        projection.nodes.insert("j2".to_string(), ComposedNode::junction("j2", ids2.clone()));

        let junctions = projection.get_junctions();
        assert_eq!(junctions.len(), 2);

        let j1_connected = projection.get_connected_graphs("j1");
        let j2_connected = projection.get_connected_graphs("j2");

        assert_eq!(j1_connected.len(), 2);
        assert_eq!(j2_connected.len(), 3);
    }

    // ========== Edge Type Tests ==========

    #[test]
    fn test_all_edge_types_in_projection() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let g1 = Uuid::new_v4();
        let g2 = Uuid::new_v4();

        // Add nodes
        projection.nodes.insert("n1".to_string(), ComposedNode::transform("n1", "op1"));
        projection.nodes.insert("n2".to_string(), ComposedNode::transform("n2", "op2"));

        // Add all edge types
        projection.edges.insert("cross".to_string(), ComposedEdge::cross_graph_link("cross", "n1", "n2", g1, g2));
        projection.edges.insert("data".to_string(), ComposedEdge::data_flow("data", "n1", "n2", "json"));
        projection.edges.insert("ctrl".to_string(), ComposedEdge::control_flow("ctrl", "n1", "n2"));
        projection.edges.insert("dep".to_string(), ComposedEdge::dependency("dep", "n1", "n2", "requires"));
        projection.edges.insert("sync".to_string(), ComposedEdge::synchronization("sync", "n1", "n2"));

        assert_eq!(projection.edges.len(), 5);

        // Verify cross-graph links can be retrieved
        let cross_links = projection.get_cross_graph_links();
        assert_eq!(cross_links.len(), 1);
    }

    #[test]
    fn test_edge_metadata() {
        let mut edge = ComposedEdge::data_flow("df1", "source", "target", "json");

        // Add metadata
        edge.metadata.insert("priority".to_string(), serde_json::json!(1));
        edge.metadata.insert("buffer_size".to_string(), serde_json::json!(1024));

        assert_eq!(edge.metadata.len(), 2);
        assert_eq!(edge.metadata["priority"], serde_json::json!(1));
    }

    #[test]
    fn test_node_metadata() {
        let mut node = ComposedNode::transform("t1", "filter");

        // Add metadata
        node.metadata.insert("config".to_string(), serde_json::json!({"threshold": 0.5}));
        node.metadata.insert("enabled".to_string(), serde_json::json!(true));

        assert_eq!(node.metadata.len(), 2);
        assert_eq!(node.metadata["enabled"], serde_json::json!(true));
    }

    // ========== Validation Edge Cases ==========

    #[test]
    fn test_validate_self_referential_control_flow() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Single node with self-loop control flow
        projection.nodes.insert("a".to_string(), ComposedNode::transform("a", "loop"));
        projection.edges.insert("self".to_string(), ComposedEdge::control_flow("self", "a", "a"));

        projection.adjacency.insert("a".to_string(), vec!["a".to_string()]);

        // Should detect cyclic control flow
        let result = projection.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cyclic"));
    }

    #[test]
    fn test_validate_indirect_cycle() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // a -> b -> c -> a (indirect cycle)
        projection.nodes.insert("a".to_string(), ComposedNode::transform("a", "op1"));
        projection.nodes.insert("b".to_string(), ComposedNode::transform("b", "op2"));
        projection.nodes.insert("c".to_string(), ComposedNode::transform("c", "op3"));

        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));
        projection.edges.insert("e2".to_string(), ComposedEdge::control_flow("e2", "b", "c"));
        projection.edges.insert("e3".to_string(), ComposedEdge::control_flow("e3", "c", "a"));

        projection.adjacency.insert("a".to_string(), vec!["b".to_string()]);
        projection.adjacency.insert("b".to_string(), vec!["c".to_string()]);
        projection.adjacency.insert("c".to_string(), vec!["a".to_string()]);

        let result = projection.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_multiple_orphaned_refs() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add multiple orphaned graph references
        projection.nodes.insert("orph1".to_string(), ComposedNode::ipld_ref("orph1", Uuid::new_v4()));
        projection.nodes.insert("orph2".to_string(), ComposedNode::context_ref("orph2", Uuid::new_v4()));

        let result = projection.validate();
        // Should fail on first orphan found
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not connected"));
    }

    // ========== Domain-Specific Graph Reference Tests ==========

    #[test]
    fn test_composed_graph_reference() {
        let composed_id = Uuid::new_v4();

        let domain = GraphDomain::Composed { graph_id: composed_id };
        let node = ComposedNode::graph_ref("composed_ref", domain);

        assert!(matches!(
            node.node_type,
            ComposedNodeType::GraphReference { domain: GraphDomain::Composed { graph_id } } if graph_id == composed_id
        ));
    }

    #[test]
    fn test_all_domain_types_in_path() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let ipld_id = Uuid::new_v4();
        let ctx_id = Uuid::new_v4();
        let wf_id = Uuid::new_v4();
        let cpt_id = Uuid::new_v4();
        let comp_id = Uuid::new_v4();

        // Create chain of all domain types
        projection.nodes.insert("ipld".to_string(), ComposedNode::ipld_ref("ipld", ipld_id));
        projection.nodes.insert("ctx".to_string(), ComposedNode::context_ref("ctx", ctx_id));
        projection.nodes.insert("wf".to_string(), ComposedNode::workflow_ref("wf", wf_id));
        projection.nodes.insert("cpt".to_string(), ComposedNode::concept_ref("cpt", cpt_id));
        projection.nodes.insert("comp".to_string(), ComposedNode::graph_ref("comp", GraphDomain::Composed { graph_id: comp_id }));

        // Link them
        projection.edges.insert("e1".to_string(), ComposedEdge::data_flow("e1", "ipld", "ctx", "s"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "ctx", "wf", "s"));
        projection.edges.insert("e3".to_string(), ComposedEdge::data_flow("e3", "wf", "cpt", "s"));
        projection.edges.insert("e4".to_string(), ComposedEdge::data_flow("e4", "cpt", "comp", "s"));

        projection.adjacency.insert("ipld".to_string(), vec!["ctx".to_string()]);
        projection.adjacency.insert("ctx".to_string(), vec!["wf".to_string()]);
        projection.adjacency.insert("wf".to_string(), vec!["cpt".to_string()]);
        projection.adjacency.insert("cpt".to_string(), vec!["comp".to_string()]);
        projection.adjacency.insert("comp".to_string(), vec![]);

        // Find path from IPLD to Composed
        let paths = projection.find_graph_paths(ipld_id, comp_id);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 5);
    }

    // ========== Node Reference Tests ==========

    #[test]
    fn test_node_reference_creation() {
        let graph_id = Uuid::new_v4();
        let node = ComposedNode::node_ref("nr1", graph_id, "internal_node_123");

        match node.node_type {
            ComposedNodeType::NodeReference { graph_id: gid, node_id } => {
                assert_eq!(gid, graph_id);
                assert_eq!(node_id, "internal_node_123");
            }
            _ => panic!("Expected NodeReference type"),
        }
    }

    #[test]
    fn test_node_reference_in_projection() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        let graph_id = Uuid::new_v4();
        projection.nodes.insert("ref1".to_string(), ComposedNode::node_ref("ref1", graph_id, "node_A"));
        projection.nodes.insert("ref2".to_string(), ComposedNode::node_ref("ref2", graph_id, "node_B"));

        // These should not appear in graph reference queries
        let refs = projection.get_graph_references();
        assert_eq!(refs.len(), 0);
    }

    // ========== Composed Projection Base Methods Tests ==========

    #[test]
    fn test_projection_aggregate_id() {
        let agg_id = Uuid::new_v4();
        let projection = ComposedProjection::new(agg_id, crate::core::GraphType::ComposedGraph);

        use crate::core::GraphProjection;
        assert_eq!(GraphProjection::aggregate_id(&projection), agg_id);
    }

    #[test]
    fn test_projection_version() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        use crate::core::GraphProjection;
        assert_eq!(GraphProjection::version(&projection), 0);

        projection.version = 42;
        assert_eq!(GraphProjection::version(&projection), 42);
    }

    #[test]
    fn test_projection_node_count() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        use crate::core::GraphProjection;
        assert_eq!(GraphProjection::node_count(&projection), 0);

        projection.nodes.insert("n1".to_string(), ComposedNode::transform("n1", "op"));
        assert_eq!(GraphProjection::node_count(&projection), 1);

        projection.nodes.insert("n2".to_string(), ComposedNode::transform("n2", "op"));
        assert_eq!(GraphProjection::node_count(&projection), 2);
    }

    #[test]
    fn test_projection_edge_count() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        use crate::core::GraphProjection;
        assert_eq!(GraphProjection::edge_count(&projection), 0);

        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));
        assert_eq!(GraphProjection::edge_count(&projection), 1);
    }

    #[test]
    fn test_projection_get_node() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        projection.nodes.insert("exists".to_string(), ComposedNode::transform("exists", "op"));

        use crate::core::GraphProjection;
        assert!(GraphProjection::get_node(&projection, "exists").is_some());
        assert!(GraphProjection::get_node(&projection, "missing").is_none());
    }

    #[test]
    fn test_projection_get_edge() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));

        use crate::core::GraphProjection;
        assert!(GraphProjection::get_edge(&projection, "e1").is_some());
        assert!(GraphProjection::get_edge(&projection, "e2").is_none());
    }

    #[test]
    fn test_projection_edges_between() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        // Add multiple edges between same nodes
        projection.edges.insert("e1".to_string(), ComposedEdge::control_flow("e1", "a", "b"));
        projection.edges.insert("e2".to_string(), ComposedEdge::data_flow("e2", "a", "b", "json"));
        projection.edges.insert("e3".to_string(), ComposedEdge::control_flow("e3", "b", "c"));

        use crate::core::GraphProjection;
        let edges_ab = GraphProjection::edges_between(&projection, "a", "b");
        assert_eq!(edges_ab.len(), 2);

        let edges_bc = GraphProjection::edges_between(&projection, "b", "c");
        assert_eq!(edges_bc.len(), 1);

        let edges_ac = GraphProjection::edges_between(&projection, "a", "c");
        assert_eq!(edges_ac.len(), 0);
    }

    #[test]
    fn test_projection_neighbors() {
        let mut projection = ComposedProjection::new(Uuid::new_v4(), crate::core::GraphType::ComposedGraph);

        projection.adjacency.insert("a".to_string(), vec!["b".to_string(), "c".to_string()]);
        projection.adjacency.insert("b".to_string(), vec!["c".to_string()]);
        projection.adjacency.insert("c".to_string(), vec![]);

        use crate::core::GraphProjection;
        let neighbors_a = GraphProjection::neighbors(&projection, "a");
        assert_eq!(neighbors_a.len(), 2);
        assert!(neighbors_a.contains(&"b"));
        assert!(neighbors_a.contains(&"c"));

        let neighbors_c = GraphProjection::neighbors(&projection, "c");
        assert_eq!(neighbors_c.len(), 0);

        let neighbors_missing = GraphProjection::neighbors(&projection, "missing");
        assert_eq!(neighbors_missing.len(), 0);
    }
}