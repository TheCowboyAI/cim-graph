//! Context graph projection - Domain-Driven Design relationships
//!
//! This is a read-only projection computed from events.

use crate::core::{Node, Edge};
use crate::core::projection_engine::GenericGraphProjection;
// Projections are ephemeral - no serialization

/// Context node types
#[derive(Debug, Clone)]
pub enum ContextNodeType {
    /// Bounded context - defines a boundary
    BoundedContext,
    /// Aggregate root - consistency boundary
    Aggregate,
    /// Entity - has identity
    Entity,
    /// Value object - immutable value
    ValueObject,
}

/// Context node
#[derive(Debug, Clone, Default)]
pub struct ContextNode {
    /// Unique identifier for the node
    pub id: String,
    /// Type of DDD element
    pub node_type: ContextNodeType,
    /// Human-readable name
    pub name: String,
    /// Additional data for the node
    pub data: serde_json::Value,
}

impl Node for ContextNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Default for ContextNodeType {
    fn default() -> Self {
        ContextNodeType::Entity
    }
}

/// Context edge - relationships in DDD
#[derive(Debug, Clone, Default)]
pub struct ContextEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Type of relationship (contains, references, etc.)
    pub relationship: String,
}

impl Edge for ContextEdge {
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

/// Context graph projection
pub type ContextProjection = GenericGraphProjection<ContextNode, ContextEdge>;

/// Context graph type alias for backward compatibility
pub type ContextGraph = ContextProjection;

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::core::GraphType;
    use crate::core::GraphProjection;

    // ========== ContextNodeType Tests ==========

    #[test]
    fn test_context_node_type_bounded_context() {
        let node_type = ContextNodeType::BoundedContext;
        assert!(matches!(node_type, ContextNodeType::BoundedContext));
    }

    #[test]
    fn test_context_node_type_aggregate() {
        let node_type = ContextNodeType::Aggregate;
        assert!(matches!(node_type, ContextNodeType::Aggregate));
    }

    #[test]
    fn test_context_node_type_entity() {
        let node_type = ContextNodeType::Entity;
        assert!(matches!(node_type, ContextNodeType::Entity));
    }

    #[test]
    fn test_context_node_type_value_object() {
        let node_type = ContextNodeType::ValueObject;
        assert!(matches!(node_type, ContextNodeType::ValueObject));
    }

    #[test]
    fn test_context_node_type_default() {
        let node_type = ContextNodeType::default();
        assert!(matches!(node_type, ContextNodeType::Entity));
    }

    #[test]
    fn test_context_node_type_clone() {
        let node_type = ContextNodeType::Aggregate;
        let cloned = node_type.clone();
        assert!(matches!(cloned, ContextNodeType::Aggregate));
    }

    #[test]
    fn test_context_node_type_debug() {
        let node_type = ContextNodeType::BoundedContext;
        let debug_str = format!("{:?}", node_type);
        assert!(debug_str.contains("BoundedContext"));
    }

    // ========== ContextNode Tests ==========

    #[test]
    fn test_context_node_creation() {
        let node = ContextNode {
            id: "order_context".to_string(),
            node_type: ContextNodeType::BoundedContext,
            name: "Order Management".to_string(),
            data: serde_json::json!({"team": "sales"}),
        };

        assert_eq!(node.id, "order_context");
        assert!(matches!(node.node_type, ContextNodeType::BoundedContext));
        assert_eq!(node.name, "Order Management");
        assert_eq!(node.data["team"], "sales");
    }

    #[test]
    fn test_context_node_default() {
        let node = ContextNode::default();
        assert_eq!(node.id, "");
        assert!(matches!(node.node_type, ContextNodeType::Entity));
        assert_eq!(node.name, "");
        assert_eq!(node.data, serde_json::Value::Null);
    }

    #[test]
    fn test_context_node_implements_node_trait() {
        let node = ContextNode {
            id: "test_node".to_string(),
            node_type: ContextNodeType::Aggregate,
            name: "Test Aggregate".to_string(),
            data: serde_json::json!({}),
        };

        assert_eq!(Node::id(&node), "test_node");
    }

    #[test]
    fn test_context_node_clone() {
        let node = ContextNode {
            id: "original".to_string(),
            node_type: ContextNodeType::ValueObject,
            name: "Money".to_string(),
            data: serde_json::json!({"currency": "USD"}),
        };

        let cloned = node.clone();

        assert_eq!(node.id, cloned.id);
        assert_eq!(node.name, cloned.name);
        assert_eq!(node.data, cloned.data);
    }

    #[test]
    fn test_context_node_debug() {
        let node = ContextNode {
            id: "debug_node".to_string(),
            node_type: ContextNodeType::Entity,
            name: "Customer".to_string(),
            data: serde_json::json!({}),
        };

        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("debug_node"));
        assert!(debug_str.contains("Customer"));
    }

    // ========== ContextEdge Tests ==========

    #[test]
    fn test_context_edge_creation() {
        let edge = ContextEdge {
            id: "contains_order".to_string(),
            source: "order_aggregate".to_string(),
            target: "order_item".to_string(),
            relationship: "contains".to_string(),
        };

        assert_eq!(edge.id, "contains_order");
        assert_eq!(edge.source, "order_aggregate");
        assert_eq!(edge.target, "order_item");
        assert_eq!(edge.relationship, "contains");
    }

    #[test]
    fn test_context_edge_default() {
        let edge = ContextEdge::default();
        assert_eq!(edge.id, "");
        assert_eq!(edge.source, "");
        assert_eq!(edge.target, "");
        assert_eq!(edge.relationship, "");
    }

    #[test]
    fn test_context_edge_implements_edge_trait() {
        let edge = ContextEdge {
            id: "e1".to_string(),
            source: "a".to_string(),
            target: "b".to_string(),
            relationship: "references".to_string(),
        };

        assert_eq!(Edge::id(&edge), "e1");
        assert_eq!(Edge::source(&edge), "a");
        assert_eq!(Edge::target(&edge), "b");
    }

    #[test]
    fn test_context_edge_clone() {
        let edge = ContextEdge {
            id: "original_edge".to_string(),
            source: "src".to_string(),
            target: "tgt".to_string(),
            relationship: "owns".to_string(),
        };

        let cloned = edge.clone();

        assert_eq!(edge.id, cloned.id);
        assert_eq!(edge.source, cloned.source);
        assert_eq!(edge.target, cloned.target);
        assert_eq!(edge.relationship, cloned.relationship);
    }

    #[test]
    fn test_context_edge_debug() {
        let edge = ContextEdge {
            id: "debug_edge".to_string(),
            source: "from".to_string(),
            target: "to".to_string(),
            relationship: "depends_on".to_string(),
        };

        let debug_str = format!("{:?}", edge);
        assert!(debug_str.contains("debug_edge"));
        assert!(debug_str.contains("depends_on"));
    }

    // ========== ContextProjection Tests ==========

    #[test]
    fn test_context_projection_creation() {
        let projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);
        assert_eq!(projection.node_count(), 0);
        assert_eq!(projection.edge_count(), 0);
        assert_eq!(projection.version, 0);
    }

    #[test]
    fn test_context_projection_with_nodes() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);

        let bounded_context = ContextNode {
            id: "sales_context".to_string(),
            node_type: ContextNodeType::BoundedContext,
            name: "Sales".to_string(),
            data: serde_json::json!({}),
        };

        let aggregate = ContextNode {
            id: "order_aggregate".to_string(),
            node_type: ContextNodeType::Aggregate,
            name: "Order".to_string(),
            data: serde_json::json!({}),
        };

        projection.nodes.insert("sales_context".to_string(), bounded_context);
        projection.nodes.insert("order_aggregate".to_string(), aggregate);

        assert_eq!(projection.node_count(), 2);
        assert!(projection.get_node("sales_context").is_some());
        assert!(projection.get_node("order_aggregate").is_some());
        assert!(projection.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_context_projection_with_edges() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);

        // Add nodes
        projection.nodes.insert(
            "aggregate".to_string(),
            ContextNode {
                id: "aggregate".to_string(),
                node_type: ContextNodeType::Aggregate,
                name: "Order".to_string(),
                data: serde_json::json!({}),
            },
        );
        projection.nodes.insert(
            "entity".to_string(),
            ContextNode {
                id: "entity".to_string(),
                node_type: ContextNodeType::Entity,
                name: "OrderLine".to_string(),
                data: serde_json::json!({}),
            },
        );

        // Add edge
        let edge = ContextEdge {
            id: "contains".to_string(),
            source: "aggregate".to_string(),
            target: "entity".to_string(),
            relationship: "contains".to_string(),
        };
        projection.edges.insert("contains".to_string(), edge);
        projection.adjacency.insert("aggregate".to_string(), vec!["entity".to_string()]);
        projection.adjacency.insert("entity".to_string(), vec![]);

        assert_eq!(projection.edge_count(), 1);
        assert!(projection.get_edge("contains").is_some());
    }

    #[test]
    fn test_context_projection_neighbors() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);

        projection.adjacency.insert(
            "context".to_string(),
            vec!["aggregate1".to_string(), "aggregate2".to_string()],
        );
        projection.adjacency.insert("aggregate1".to_string(), vec!["entity1".to_string()]);
        projection.adjacency.insert("aggregate2".to_string(), vec![]);
        projection.adjacency.insert("entity1".to_string(), vec![]);

        let context_neighbors = projection.neighbors("context");
        assert_eq!(context_neighbors.len(), 2);
        assert!(context_neighbors.contains(&"aggregate1"));
        assert!(context_neighbors.contains(&"aggregate2"));

        let agg1_neighbors = projection.neighbors("aggregate1");
        assert_eq!(agg1_neighbors.len(), 1);
        assert!(agg1_neighbors.contains(&"entity1"));
    }

    #[test]
    fn test_context_projection_edges_between() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);

        // Add two edges between same nodes
        projection.edges.insert(
            "e1".to_string(),
            ContextEdge {
                id: "e1".to_string(),
                source: "A".to_string(),
                target: "B".to_string(),
                relationship: "contains".to_string(),
            },
        );
        projection.edges.insert(
            "e2".to_string(),
            ContextEdge {
                id: "e2".to_string(),
                source: "A".to_string(),
                target: "B".to_string(),
                relationship: "references".to_string(),
            },
        );
        projection.edges.insert(
            "e3".to_string(),
            ContextEdge {
                id: "e3".to_string(),
                source: "B".to_string(),
                target: "C".to_string(),
                relationship: "uses".to_string(),
            },
        );

        let edges_ab = projection.edges_between("A", "B");
        assert_eq!(edges_ab.len(), 2);

        let edges_bc = projection.edges_between("B", "C");
        assert_eq!(edges_bc.len(), 1);

        let edges_ac = projection.edges_between("A", "C");
        assert_eq!(edges_ac.len(), 0);
    }

    // ========== DDD Pattern Tests ==========

    #[test]
    fn test_ddd_bounded_context_pattern() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);

        // Create bounded context with aggregates
        let sales_context = ContextNode {
            id: "sales".to_string(),
            node_type: ContextNodeType::BoundedContext,
            name: "Sales Context".to_string(),
            data: serde_json::json!({
                "team": "Sales Team",
                "responsible": "John Doe"
            }),
        };

        let order_aggregate = ContextNode {
            id: "order".to_string(),
            node_type: ContextNodeType::Aggregate,
            name: "Order Aggregate".to_string(),
            data: serde_json::json!({
                "invariants": ["order_total_must_be_positive", "at_least_one_line_item"]
            }),
        };

        let customer_aggregate = ContextNode {
            id: "customer".to_string(),
            node_type: ContextNodeType::Aggregate,
            name: "Customer Aggregate".to_string(),
            data: serde_json::json!({}),
        };

        projection.nodes.insert("sales".to_string(), sales_context);
        projection.nodes.insert("order".to_string(), order_aggregate);
        projection.nodes.insert("customer".to_string(), customer_aggregate);

        // Connect aggregates to context
        projection.edges.insert(
            "sales_order".to_string(),
            ContextEdge {
                id: "sales_order".to_string(),
                source: "sales".to_string(),
                target: "order".to_string(),
                relationship: "contains".to_string(),
            },
        );
        projection.edges.insert(
            "sales_customer".to_string(),
            ContextEdge {
                id: "sales_customer".to_string(),
                source: "sales".to_string(),
                target: "customer".to_string(),
                relationship: "contains".to_string(),
            },
        );

        projection.adjacency.insert(
            "sales".to_string(),
            vec!["order".to_string(), "customer".to_string()],
        );
        projection.adjacency.insert("order".to_string(), vec![]);
        projection.adjacency.insert("customer".to_string(), vec![]);

        // Verify DDD structure
        assert_eq!(projection.node_count(), 3);
        assert_eq!(projection.edge_count(), 2);

        let context = projection.get_node("sales").unwrap();
        assert!(matches!(context.node_type, ContextNodeType::BoundedContext));

        let order = projection.get_node("order").unwrap();
        assert!(matches!(order.node_type, ContextNodeType::Aggregate));
    }

    #[test]
    fn test_ddd_aggregate_with_entities() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);

        // Order aggregate with entities
        let order = ContextNode {
            id: "order".to_string(),
            node_type: ContextNodeType::Aggregate,
            name: "Order".to_string(),
            data: serde_json::json!({}),
        };

        let order_line = ContextNode {
            id: "order_line".to_string(),
            node_type: ContextNodeType::Entity,
            name: "OrderLine".to_string(),
            data: serde_json::json!({}),
        };

        let money = ContextNode {
            id: "money".to_string(),
            node_type: ContextNodeType::ValueObject,
            name: "Money".to_string(),
            data: serde_json::json!({"immutable": true}),
        };

        projection.nodes.insert("order".to_string(), order);
        projection.nodes.insert("order_line".to_string(), order_line);
        projection.nodes.insert("money".to_string(), money);

        projection.edges.insert(
            "order_lines".to_string(),
            ContextEdge {
                id: "order_lines".to_string(),
                source: "order".to_string(),
                target: "order_line".to_string(),
                relationship: "has_many".to_string(),
            },
        );
        projection.edges.insert(
            "line_amount".to_string(),
            ContextEdge {
                id: "line_amount".to_string(),
                source: "order_line".to_string(),
                target: "money".to_string(),
                relationship: "uses_value".to_string(),
            },
        );

        // Verify structure
        let order_node = projection.get_node("order").unwrap();
        assert!(matches!(order_node.node_type, ContextNodeType::Aggregate));

        let line_node = projection.get_node("order_line").unwrap();
        assert!(matches!(line_node.node_type, ContextNodeType::Entity));

        let money_node = projection.get_node("money").unwrap();
        assert!(matches!(money_node.node_type, ContextNodeType::ValueObject));
    }

    #[test]
    fn test_context_node_all_types() {
        let types = vec![
            ContextNodeType::BoundedContext,
            ContextNodeType::Aggregate,
            ContextNodeType::Entity,
            ContextNodeType::ValueObject,
        ];

        for node_type in types {
            let node = ContextNode {
                id: format!("{:?}", node_type),
                node_type: node_type.clone(),
                name: "Test".to_string(),
                data: serde_json::json!({}),
            };

            assert!(!node.id.is_empty());
            let _ = format!("{:?}", node.node_type); // Verify Debug works
        }
    }

    #[test]
    fn test_context_projection_version_tracking() {
        let mut projection = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);
        assert_eq!(projection.version, 0);

        projection.version = 1;
        assert_eq!(projection.version, 1);

        projection.version = 100;
        assert_eq!(projection.version, 100);
    }

    #[test]
    fn test_context_projection_aggregate_id() {
        let agg_id = Uuid::new_v4();
        let projection = ContextProjection::new(agg_id, GraphType::ContextGraph);

        use crate::core::GraphProjection;
        assert_eq!(GraphProjection::aggregate_id(&projection), agg_id);
    }

    // ========== Relationship Types Tests ==========

    #[test]
    fn test_common_ddd_relationships() {
        let relationships = vec![
            "contains",
            "references",
            "owns",
            "uses_value",
            "has_many",
            "belongs_to",
            "depends_on",
            "publishes",
            "subscribes",
        ];

        for rel in relationships {
            let edge = ContextEdge {
                id: format!("edge_{}", rel),
                source: "source".to_string(),
                target: "target".to_string(),
                relationship: rel.to_string(),
            };

            assert_eq!(edge.relationship, rel);
        }
    }

    #[test]
    fn test_context_graph_type_alias() {
        // ContextGraph should be the same as ContextProjection
        let graph: ContextGraph = ContextProjection::new(Uuid::new_v4(), GraphType::ContextGraph);
        assert_eq!(GraphProjection::node_count(&graph), 0);
    }
}