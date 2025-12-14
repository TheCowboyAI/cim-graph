/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Domain Functor: Graphs → cim-domain
//!
//! Implements the functor F: Cat(Graphs) → Cat(cim-domain) that maps:
//! - Graph nodes → Domain aggregates
//! - Graph edges → Domain relationships
//! - Graph paths → Domain workflows/sagas
//!
//! This functor preserves composition, making it a true categorical functor.

use super::{Functor, MorphismData};
use crate::core::node::Node;
use crate::core::edge::Edge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Functor from Category of Graphs to Category of Domain Objects
///
/// Maps graph structures to domain aggregates while preserving composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainFunctor {
    /// Unique identifier for this functor instance
    pub functor_id: String,
    /// Mapping from graph node IDs to domain object IDs
    pub node_to_domain: HashMap<String, DomainObject>,
    /// Mapping from graph edge IDs to domain relationships
    pub edge_to_relationship: HashMap<String, DomainRelationship>,
    /// Composition cache for performance
    pub composition_cache: HashMap<String, Vec<String>>,
}

/// Representation of a domain object (aggregate) in the target category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainObject {
    /// Unique identifier
    pub id: Uuid,
    /// Type of domain aggregate (Policy, Location, Organization, Person, etc.)
    pub aggregate_type: DomainAggregateType,
    /// Name/label of the object
    pub name: String,
    /// Properties specific to this aggregate
    pub properties: HashMap<String, serde_json::Value>,
    /// Version/state of the aggregate
    pub version: u64,
}

/// Types of domain aggregates in the CIM ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainAggregateType {
    /// Policy domain aggregate
    Policy,
    /// Location domain aggregate
    Location,
    /// Organization domain aggregate
    Organization,
    /// Person domain aggregate
    Person,
    /// Custom domain aggregate
    Custom(String),
}

impl std::fmt::Display for DomainAggregateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainAggregateType::Policy => write!(f, "Policy"),
            DomainAggregateType::Location => write!(f, "Location"),
            DomainAggregateType::Organization => write!(f, "Organization"),
            DomainAggregateType::Person => write!(f, "Person"),
            DomainAggregateType::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

/// Relationship between domain objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRelationship {
    /// Unique identifier
    pub id: String,
    /// Source domain object ID
    pub source_id: Uuid,
    /// Target domain object ID
    pub target_id: Uuid,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Additional relationship data
    pub properties: HashMap<String, serde_json::Value>,
}

/// Types of relationships between domain objects
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// One aggregate contains another
    Contains,
    /// One aggregate references another
    References,
    /// One aggregate depends on another
    DependsOn,
    /// Aggregates are part of a workflow
    WorkflowStep,
    /// Parent-child hierarchical relationship
    ParentChild,
    /// Custom relationship type
    Custom(String),
}

impl DomainFunctor {
    /// Create a new domain functor
    pub fn new(functor_id: String) -> Self {
        Self {
            functor_id,
            node_to_domain: HashMap::new(),
            edge_to_relationship: HashMap::new(),
            composition_cache: HashMap::new(),
        }
    }

    /// Map a graph node to a domain object
    pub fn map_node<N: Node>(&mut self, node: &N, aggregate_type: DomainAggregateType) -> DomainObject {
        let node_id = node.id();

        // Check if already mapped
        if let Some(existing) = self.node_to_domain.get(&node_id) {
            return existing.clone();
        }

        // Create new domain object
        let domain_obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type,
            name: node_id.clone(),
            properties: HashMap::new(),
            version: 1,
        };

        self.node_to_domain.insert(node_id, domain_obj.clone());
        domain_obj
    }

    /// Map a graph edge to a domain relationship
    pub fn map_edge<E: Edge>(
        &mut self,
        edge: &E,
        relationship_type: RelationshipType,
    ) -> Option<DomainRelationship> {
        let edge_id = edge.id();

        // Get source and target domain objects
        let source_node_id = edge.source();
        let target_node_id = edge.target();

        let source_domain = self.node_to_domain.get(&source_node_id)?;
        let target_domain = self.node_to_domain.get(&target_node_id)?;

        let relationship = DomainRelationship {
            id: edge_id.clone(),
            source_id: source_domain.id,
            target_id: target_domain.id,
            relationship_type,
            properties: HashMap::new(),
        };

        self.edge_to_relationship.insert(edge_id, relationship.clone());
        Some(relationship)
    }

    /// Compose a path through the graph into a domain workflow
    ///
    /// This preserves composition: F(g ∘ f) = F(g) ∘ F(f)
    pub fn compose_path(&mut self, path: &[String]) -> Option<Vec<DomainRelationship>> {
        if path.is_empty() {
            return None;
        }

        // Check cache
        let cache_key = path.join("->");
        if let Some(cached) = self.composition_cache.get(&cache_key) {
            return Some(
                cached.iter()
                    .filter_map(|id| self.edge_to_relationship.get(id).cloned())
                    .collect()
            );
        }

        // Compose relationships in order
        let mut composed: Vec<DomainRelationship> = Vec::new();
        for edge_id in path {
            if let Some(rel) = self.edge_to_relationship.get(edge_id) {
                composed.push(rel.clone());
            } else {
                return None; // Path contains unmapped edge
            }
        }

        // Verify composition is valid
        if !self.verify_composition(&composed) {
            return None;
        }

        // Cache the result
        self.composition_cache.insert(cache_key, path.to_vec());

        Some(composed)
    }

    /// Verify that a sequence of relationships forms a valid composition
    ///
    /// For composition to be valid: target of f must equal source of g for f ∘ g
    fn verify_composition(&self, relationships: &[DomainRelationship]) -> bool {
        for window in relationships.windows(2) {
            if window[0].target_id != window[1].source_id {
                return false;
            }
        }
        true
    }

    /// Get domain object by graph node ID
    pub fn get_domain_object(&self, node_id: &str) -> Option<&DomainObject> {
        self.node_to_domain.get(node_id)
    }

    /// Get relationship by graph edge ID
    pub fn get_relationship(&self, edge_id: &str) -> Option<&DomainRelationship> {
        self.edge_to_relationship.get(edge_id)
    }

    /// Get all domain objects
    pub fn domain_objects(&self) -> impl Iterator<Item = &DomainObject> {
        self.node_to_domain.values()
    }

    /// Get all relationships
    pub fn relationships(&self) -> impl Iterator<Item = &DomainRelationship> {
        self.edge_to_relationship.values()
    }

    /// Verify that this functor satisfies functor laws
    ///
    /// Checks composition preservation
    pub fn verify_laws(&self) -> bool {
        // Verify composition preservation
        for relationships in self.composition_cache.values() {
            if relationships.len() < 2 {
                continue;
            }

            // Get actual domain relationships
            let rels: Vec<DomainRelationship> = relationships.iter()
                .filter_map(|id| self.edge_to_relationship.get(id).cloned())
                .collect();

            if !self.verify_composition(&rels) {
                return false;
            }
        }

        true
    }
}

// Implement the generic Functor trait
impl<N: Node> Functor<N, DomainObject> for DomainFunctor {
    fn map_object(&self, node: &N) -> DomainObject {
        let node_id = node.id();
        self.node_to_domain.get(&node_id).cloned().unwrap_or_else(|| {
            DomainObject {
                id: Uuid::now_v7(),
                aggregate_type: DomainAggregateType::Custom("unmapped".to_string()),
                name: node_id,
                properties: HashMap::new(),
                version: 0,
            }
        })
    }

    fn map_morphism(
        &self,
        _source: &N,
        _target: &N,
        morphism_data: &MorphismData,
    ) -> MorphismData {
        // Return the morphism data as-is, potentially enriched
        morphism_data.clone()
    }

    fn verify_functor_laws(&self) -> bool {
        // Verify composition preservation
        for relationships in self.composition_cache.values() {
            if relationships.len() < 2 {
                continue;
            }

            // Get actual domain relationships
            let rels: Vec<DomainRelationship> = relationships.iter()
                .filter_map(|id| self.edge_to_relationship.get(id).cloned())
                .collect();

            if !self.verify_composition(&rels) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::node::GenericNode;
    use crate::core::edge::GenericEdge;

    #[test]
    fn test_domain_functor_creation() {
        let functor = DomainFunctor::new("test_functor".to_string());
        assert_eq!(functor.functor_id, "test_functor");
        assert_eq!(functor.node_to_domain.len(), 0);
    }

    #[test]
    fn test_map_node_to_domain() {
        let mut functor = DomainFunctor::new("test".to_string());
        let node = GenericNode::new("node1", "data");

        let domain_obj = functor.map_node(&node, DomainAggregateType::Policy);

        assert_eq!(domain_obj.aggregate_type, DomainAggregateType::Policy);
        assert_eq!(domain_obj.name, "node1");
        assert_eq!(functor.node_to_domain.len(), 1);
    }

    #[test]
    fn test_map_edge_to_relationship() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create and map nodes first
        let node1 = GenericNode::new("node1", "data1");
        let node2 = GenericNode::new("node2", "data2");
        functor.map_node(&node1, DomainAggregateType::Policy);
        functor.map_node(&node2, DomainAggregateType::Location);

        // Create and map edge
        let edge = GenericEdge::with_id("edge1", "node1", "node2", "connects");
        let relationship = functor.map_edge(&edge, RelationshipType::References);

        assert!(relationship.is_some());
        let rel = relationship.unwrap();
        assert_eq!(rel.relationship_type, RelationshipType::References);
    }

    #[test]
    fn test_composition_preservation() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create chain: A -> B -> C
        let node_a = GenericNode::new("A", "data");
        let node_b = GenericNode::new("B", "data");
        let node_c = GenericNode::new("C", "data");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);
        functor.map_node(&node_c, DomainAggregateType::Organization);

        let edge_ab = GenericEdge::with_id("AB", "A", "B", "step1");
        let edge_bc = GenericEdge::with_id("BC", "B", "C", "step2");

        functor.map_edge(&edge_ab, RelationshipType::WorkflowStep);
        functor.map_edge(&edge_bc, RelationshipType::WorkflowStep);

        // Compose path A -> B -> C
        let path = vec!["AB".to_string(), "BC".to_string()];
        let composition = functor.compose_path(&path);

        assert!(composition.is_some());
        let composed = composition.unwrap();
        assert_eq!(composed.len(), 2);

        // Verify composition laws
        assert!(functor.verify_laws());
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_map_node_idempotent() {
        let mut functor = DomainFunctor::new("test".to_string());
        let node = GenericNode::new("node1", "data");

        // Map same node twice
        let obj1 = functor.map_node(&node, DomainAggregateType::Policy);
        let obj2 = functor.map_node(&node, DomainAggregateType::Location);

        // Should return the same object (first mapping wins)
        assert_eq!(obj1.id, obj2.id);
        assert_eq!(obj1.aggregate_type, obj2.aggregate_type);
        assert_eq!(functor.node_to_domain.len(), 1);
    }

    #[test]
    fn test_map_edge_without_nodes() {
        let mut functor = DomainFunctor::new("test".to_string());
        let edge = GenericEdge::with_id("edge1", "node1", "node2", "connects");

        // Try to map edge without mapping nodes first
        let relationship = functor.map_edge(&edge, RelationshipType::References);

        // Should return None since nodes are not mapped
        assert!(relationship.is_none());
    }

    #[test]
    fn test_compose_empty_path() {
        let mut functor = DomainFunctor::new("test".to_string());
        let result = functor.compose_path(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_compose_path_with_unmapped_edge() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create and map some nodes/edges but not all
        let node_a = GenericNode::new("A", "data");
        let node_b = GenericNode::new("B", "data");
        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);

        let edge_ab = GenericEdge::with_id("AB", "A", "B", "step");
        functor.map_edge(&edge_ab, RelationshipType::WorkflowStep);

        // Try to compose path with unmapped edge
        let path = vec!["AB".to_string(), "BC".to_string()];
        let result = functor.compose_path(&path);

        // Should return None since BC is not mapped
        assert!(result.is_none());
    }

    #[test]
    fn test_compose_path_invalid_chain() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create nodes
        let node_a = GenericNode::new("A", "data");
        let node_b = GenericNode::new("B", "data");
        let node_c = GenericNode::new("C", "data");
        let node_d = GenericNode::new("D", "data");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);
        functor.map_node(&node_c, DomainAggregateType::Organization);
        functor.map_node(&node_d, DomainAggregateType::Person);

        // Create edges A->B and C->D (not connected)
        let edge_ab = GenericEdge::with_id("AB", "A", "B", "step");
        let edge_cd = GenericEdge::with_id("CD", "C", "D", "step");

        functor.map_edge(&edge_ab, RelationshipType::WorkflowStep);
        functor.map_edge(&edge_cd, RelationshipType::WorkflowStep);

        // Try to compose disconnected path
        let path = vec!["AB".to_string(), "CD".to_string()];
        let result = functor.compose_path(&path);

        // Should return None since paths don't connect
        assert!(result.is_none());
    }

    #[test]
    fn test_get_domain_object_not_found() {
        let functor = DomainFunctor::new("test".to_string());
        let result = functor.get_domain_object("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_relationship_not_found() {
        let functor = DomainFunctor::new("test".to_string());
        let result = functor.get_relationship("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_domain_objects_iterator() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node1 = GenericNode::new("node1", "data");
        let node2 = GenericNode::new("node2", "data");

        functor.map_node(&node1, DomainAggregateType::Policy);
        functor.map_node(&node2, DomainAggregateType::Location);

        let objects: Vec<_> = functor.domain_objects().collect();
        assert_eq!(objects.len(), 2);
    }

    #[test]
    fn test_relationships_iterator() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node1 = GenericNode::new("A", "data");
        let node2 = GenericNode::new("B", "data");
        functor.map_node(&node1, DomainAggregateType::Policy);
        functor.map_node(&node2, DomainAggregateType::Location);

        let edge = GenericEdge::with_id("AB", "A", "B", "connects");
        functor.map_edge(&edge, RelationshipType::References);

        let relationships: Vec<_> = functor.relationships().collect();
        assert_eq!(relationships.len(), 1);
    }

    #[test]
    fn test_domain_aggregate_type_display() {
        assert_eq!(format!("{}", DomainAggregateType::Policy), "Policy");
        assert_eq!(format!("{}", DomainAggregateType::Location), "Location");
        assert_eq!(format!("{}", DomainAggregateType::Organization), "Organization");
        assert_eq!(format!("{}", DomainAggregateType::Person), "Person");
        assert_eq!(format!("{}", DomainAggregateType::Custom("Test".to_string())), "Custom(Test)");
    }

    #[test]
    fn test_functor_trait_map_object_unmapped() {
        use crate::functors::Functor;

        let functor = DomainFunctor::new("test".to_string());
        let node = GenericNode::new("unmapped", "data");

        // Should create a fallback object for unmapped nodes
        let obj = functor.map_object(&node);
        assert_eq!(obj.name, "unmapped");
        assert_eq!(obj.version, 0);
        assert!(matches!(obj.aggregate_type, DomainAggregateType::Custom(_)));
    }

    #[test]
    fn test_functor_trait_verify_laws_empty() {
        use crate::functors::Functor;

        let functor: DomainFunctor = DomainFunctor::new("test".to_string());
        let node: GenericNode<&str> = GenericNode::new("test", "data");

        // Empty functor should pass verification
        assert!(Functor::<GenericNode<&str>, DomainObject>::verify_functor_laws(&functor));

        // Also test via the direct method
        assert!(functor.verify_laws());

        // Suppress unused variable warning
        let _ = node;
    }

    #[test]
    fn test_all_relationship_types() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create nodes
        let nodes: Vec<_> = (0..6).map(|i| GenericNode::new(format!("N{}", i).leak(), "data")).collect();
        for node in &nodes {
            functor.map_node(node, DomainAggregateType::Policy);
        }

        // Test all relationship types
        let relationship_types = vec![
            RelationshipType::Contains,
            RelationshipType::References,
            RelationshipType::DependsOn,
            RelationshipType::WorkflowStep,
            RelationshipType::ParentChild,
            RelationshipType::Custom("custom_type".to_string()),
        ];

        for (i, rel_type) in relationship_types.into_iter().enumerate() {
            let from = format!("N{}", i);
            let to = format!("N{}", (i + 1) % 6);
            let edge_id: &'static str = format!("E{}", i).leak();
            let edge = GenericEdge::with_id(edge_id, from.leak(), to.leak(), "test");

            let relationship = functor.map_edge(&edge, rel_type.clone());
            assert!(relationship.is_some(), "Failed to map edge with {:?}", rel_type);
        }

        assert_eq!(functor.edge_to_relationship.len(), 6);
    }

    #[test]
    fn test_composition_cache() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create chain
        let node_a = GenericNode::new("A", "data");
        let node_b = GenericNode::new("B", "data");
        let node_c = GenericNode::new("C", "data");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);
        functor.map_node(&node_c, DomainAggregateType::Organization);

        let edge_ab = GenericEdge::with_id("AB", "A", "B", "step");
        let edge_bc = GenericEdge::with_id("BC", "B", "C", "step");

        functor.map_edge(&edge_ab, RelationshipType::WorkflowStep);
        functor.map_edge(&edge_bc, RelationshipType::WorkflowStep);

        let path = vec!["AB".to_string(), "BC".to_string()];

        // First call computes and caches
        let result1 = functor.compose_path(&path);
        assert!(result1.is_some());
        assert!(!functor.composition_cache.is_empty());

        // Second call should use cache
        let result2 = functor.compose_path(&path);
        assert!(result2.is_some());

        // Results should be equivalent
        assert_eq!(result1.unwrap().len(), result2.unwrap().len());
    }

    // ========== Natural Transformation Tests ==========

    #[test]
    fn test_functor_trait_map_morphism() {
        use crate::functors::{Functor, MorphismData};

        let mut functor = DomainFunctor::new("test".to_string());

        let node_a = GenericNode::new("A", "data");
        let node_b = GenericNode::new("B", "data");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);

        let morphism = MorphismData {
            id: "m1".to_string(),
            morphism_type: "transition".to_string(),
            properties: std::collections::HashMap::new(),
        };

        let mapped = Functor::<GenericNode<&str>, DomainObject>::map_morphism(
            &functor, &node_a, &node_b, &morphism
        );

        // map_morphism returns cloned morphism data
        assert_eq!(mapped.id, morphism.id);
        assert_eq!(mapped.morphism_type, morphism.morphism_type);
    }

    #[test]
    fn test_domain_object_properties() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node = GenericNode::new("node1", "some_data");
        let obj = functor.map_node(&node, DomainAggregateType::Person);

        assert_eq!(obj.name, "node1");
        assert_eq!(obj.version, 1);
        assert!(obj.properties.is_empty());
    }

    #[test]
    fn test_domain_relationship_properties() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node1 = GenericNode::new("N1", "d1");
        let node2 = GenericNode::new("N2", "d2");

        functor.map_node(&node1, DomainAggregateType::Organization);
        functor.map_node(&node2, DomainAggregateType::Person);

        let edge = GenericEdge::with_id("employs", "N1", "N2", "employment");
        let rel = functor.map_edge(&edge, RelationshipType::Contains);

        assert!(rel.is_some());
        let relationship = rel.unwrap();
        assert_eq!(relationship.id, "employs");
        assert!(relationship.properties.is_empty());
    }

    // ========== DomainObject Version Tests ==========

    #[test]
    fn test_domain_object_initial_version() {
        let mut functor = DomainFunctor::new("test".to_string());
        let node = GenericNode::new("versioned", "data");

        let obj = functor.map_node(&node, DomainAggregateType::Policy);

        assert_eq!(obj.version, 1);
    }

    // ========== Composition Validation Tests ==========

    #[test]
    fn test_verify_composition_empty() {
        let functor = DomainFunctor::new("test".to_string());

        // Empty composition is valid
        assert!(functor.verify_composition(&[]));
    }

    #[test]
    fn test_verify_composition_single() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node1 = GenericNode::new("A", "d");
        let node2 = GenericNode::new("B", "d");

        functor.map_node(&node1, DomainAggregateType::Policy);
        functor.map_node(&node2, DomainAggregateType::Location);

        let edge = GenericEdge::with_id("AB", "A", "B", "step");
        let rel = functor.map_edge(&edge, RelationshipType::WorkflowStep).unwrap();

        // Single relationship is always valid
        assert!(functor.verify_composition(&[rel]));
    }

    #[test]
    fn test_verify_composition_valid_chain() {
        let mut functor = DomainFunctor::new("test".to_string());

        let nodes: Vec<_> = (0..4).map(|i| GenericNode::new(format!("N{}", i).leak(), "d")).collect();
        for node in &nodes {
            functor.map_node(node, DomainAggregateType::Policy);
        }

        let edges = vec![
            GenericEdge::with_id("E01", "N0", "N1", "s"),
            GenericEdge::with_id("E12", "N1", "N2", "s"),
            GenericEdge::with_id("E23", "N2", "N3", "s"),
        ];

        let rels: Vec<_> = edges.iter()
            .map(|e| functor.map_edge(e, RelationshipType::WorkflowStep).unwrap())
            .collect();

        assert!(functor.verify_composition(&rels));
    }

    // ========== RelationshipType Tests ==========

    #[test]
    fn test_relationship_type_equality() {
        assert_eq!(RelationshipType::Contains, RelationshipType::Contains);
        assert_eq!(RelationshipType::References, RelationshipType::References);
        assert_ne!(RelationshipType::Contains, RelationshipType::References);

        let custom1 = RelationshipType::Custom("type1".to_string());
        let custom2 = RelationshipType::Custom("type1".to_string());
        let custom3 = RelationshipType::Custom("type2".to_string());

        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
    }

    #[test]
    fn test_all_relationship_types_mappable() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create 6 pairs of nodes
        for i in 0..12 {
            let node = GenericNode::new(format!("N{}", i).leak(), "d");
            functor.map_node(&node, DomainAggregateType::Policy);
        }

        let relationship_types = vec![
            RelationshipType::Contains,
            RelationshipType::References,
            RelationshipType::DependsOn,
            RelationshipType::WorkflowStep,
            RelationshipType::ParentChild,
            RelationshipType::Custom("test_custom".to_string()),
        ];

        for (i, rel_type) in relationship_types.into_iter().enumerate() {
            let from = format!("N{}", i * 2);
            let to = format!("N{}", i * 2 + 1);
            let edge_id: &'static str = format!("E{}", i).leak();
            let edge = GenericEdge::with_id(edge_id, from.leak(), to.leak(), "step");

            let result = functor.map_edge(&edge, rel_type.clone());
            assert!(result.is_some(), "Failed to map edge with {:?}", rel_type);
        }
    }

    // ========== DomainAggregateType Tests ==========

    #[test]
    fn test_domain_aggregate_type_equality() {
        assert_eq!(DomainAggregateType::Policy, DomainAggregateType::Policy);
        assert_eq!(DomainAggregateType::Location, DomainAggregateType::Location);
        assert_ne!(DomainAggregateType::Policy, DomainAggregateType::Location);

        let custom1 = DomainAggregateType::Custom("custom1".to_string());
        let custom2 = DomainAggregateType::Custom("custom1".to_string());
        let custom3 = DomainAggregateType::Custom("custom2".to_string());

        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
    }

    #[test]
    fn test_domain_aggregate_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(DomainAggregateType::Policy);
        set.insert(DomainAggregateType::Location);
        set.insert(DomainAggregateType::Organization);
        set.insert(DomainAggregateType::Person);
        set.insert(DomainAggregateType::Custom("custom".to_string()));

        assert_eq!(set.len(), 5);

        // Inserting same value should not increase size
        set.insert(DomainAggregateType::Policy);
        assert_eq!(set.len(), 5);
    }

    // ========== Functor Laws Verification ==========

    #[test]
    fn test_verify_laws_empty_functor() {
        let functor = DomainFunctor::new("test".to_string());
        assert!(functor.verify_laws());
    }

    #[test]
    fn test_verify_laws_with_mappings() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node_a = GenericNode::new("A", "d");
        let node_b = GenericNode::new("B", "d");
        let node_c = GenericNode::new("C", "d");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);
        functor.map_node(&node_c, DomainAggregateType::Organization);

        let edge_ab = GenericEdge::with_id("AB", "A", "B", "s");
        let edge_bc = GenericEdge::with_id("BC", "B", "C", "s");

        functor.map_edge(&edge_ab, RelationshipType::WorkflowStep);
        functor.map_edge(&edge_bc, RelationshipType::WorkflowStep);

        // Compose path to populate cache
        functor.compose_path(&["AB".to_string(), "BC".to_string()]);

        assert!(functor.verify_laws());
    }

    // ========== Serialization Tests ==========

    #[test]
    fn test_domain_functor_serialization() {
        let mut functor = DomainFunctor::new("serializable".to_string());

        let node = GenericNode::new("N1", "data");
        functor.map_node(&node, DomainAggregateType::Policy);

        let json = serde_json::to_string(&functor).unwrap();
        assert!(json.contains("serializable"));
        assert!(json.contains("N1"));

        let deserialized: DomainFunctor = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.functor_id, "serializable");
        assert!(deserialized.get_domain_object("N1").is_some());
    }

    #[test]
    fn test_domain_object_serialization() {
        let obj = DomainObject {
            id: Uuid::now_v7(),
            aggregate_type: DomainAggregateType::Person,
            name: "Test Person".to_string(),
            properties: {
                let mut props = std::collections::HashMap::new();
                props.insert("age".to_string(), serde_json::json!(30));
                props
            },
            version: 5,
        };

        let json = serde_json::to_string(&obj).unwrap();
        let deserialized: DomainObject = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, obj.id);
        assert_eq!(deserialized.name, "Test Person");
        assert_eq!(deserialized.version, 5);
        assert_eq!(deserialized.properties["age"], serde_json::json!(30));
    }

    #[test]
    fn test_domain_relationship_serialization() {
        let rel = DomainRelationship {
            id: "rel1".to_string(),
            source_id: Uuid::now_v7(),
            target_id: Uuid::now_v7(),
            relationship_type: RelationshipType::ParentChild,
            properties: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&rel).unwrap();
        let deserialized: DomainRelationship = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "rel1");
        assert_eq!(deserialized.relationship_type, RelationshipType::ParentChild);
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_map_edge_source_missing() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Only map target node
        let node2 = GenericNode::new("N2", "d");
        functor.map_node(&node2, DomainAggregateType::Policy);

        let edge = GenericEdge::with_id("E12", "N1", "N2", "s");
        let result = functor.map_edge(&edge, RelationshipType::References);

        // Should return None since source is not mapped
        assert!(result.is_none());
    }

    #[test]
    fn test_map_edge_target_missing() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Only map source node
        let node1 = GenericNode::new("N1", "d");
        functor.map_node(&node1, DomainAggregateType::Policy);

        let edge = GenericEdge::with_id("E12", "N1", "N2", "s");
        let result = functor.map_edge(&edge, RelationshipType::References);

        // Should return None since target is not mapped
        assert!(result.is_none());
    }

    #[test]
    fn test_compose_path_single_edge() {
        let mut functor = DomainFunctor::new("test".to_string());

        let node_a = GenericNode::new("A", "d");
        let node_b = GenericNode::new("B", "d");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);

        let edge = GenericEdge::with_id("AB", "A", "B", "s");
        functor.map_edge(&edge, RelationshipType::WorkflowStep);

        let path = vec!["AB".to_string()];
        let result = functor.compose_path(&path);

        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_compose_path_long_chain() {
        let mut functor = DomainFunctor::new("test".to_string());

        // Create a chain of 10 nodes
        for i in 0..10 {
            let node = GenericNode::new(format!("N{}", i).leak(), "d");
            functor.map_node(&node, DomainAggregateType::Policy);
        }

        // Create 9 edges connecting them
        let mut path = Vec::new();
        for i in 0..9 {
            let from = format!("N{}", i);
            let to = format!("N{}", i + 1);
            let edge_id: &'static str = format!("E{}", i).leak();
            let edge = GenericEdge::with_id(edge_id, from.leak(), to.leak(), "s");
            functor.map_edge(&edge, RelationshipType::WorkflowStep);
            path.push(format!("E{}", i));
        }

        let result = functor.compose_path(&path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 9);
    }

    #[test]
    fn test_domain_objects_iterator_empty() {
        let functor = DomainFunctor::new("test".to_string());
        let objects: Vec<_> = functor.domain_objects().collect();
        assert!(objects.is_empty());
    }

    #[test]
    fn test_relationships_iterator_empty() {
        let functor = DomainFunctor::new("test".to_string());
        let relationships: Vec<_> = functor.relationships().collect();
        assert!(relationships.is_empty());
    }

    // ========== Functor Trait Full Tests ==========

    #[test]
    fn test_functor_trait_verify_laws() {
        use crate::functors::Functor;

        let mut functor = DomainFunctor::new("test".to_string());

        // Add some mappings
        let node_a = GenericNode::new("A", "d");
        let node_b = GenericNode::new("B", "d");

        functor.map_node(&node_a, DomainAggregateType::Policy);
        functor.map_node(&node_b, DomainAggregateType::Location);

        let edge = GenericEdge::with_id("AB", "A", "B", "s");
        functor.map_edge(&edge, RelationshipType::WorkflowStep);

        // Verify through the trait
        assert!(Functor::<GenericNode<&str>, DomainObject>::verify_functor_laws(&functor));
    }

    #[test]
    fn test_functor_trait_map_object_with_mapping() {
        use crate::functors::Functor;

        let mut functor = DomainFunctor::new("test".to_string());
        let node = GenericNode::new("mapped", "data");

        // Map the node first
        let original = functor.map_node(&node, DomainAggregateType::Organization);

        // Now use the trait method
        let mapped = Functor::<GenericNode<&str>, DomainObject>::map_object(&functor, &node);

        // Should return the already-mapped object
        assert_eq!(mapped.id, original.id);
        assert_eq!(mapped.aggregate_type, DomainAggregateType::Organization);
    }

    // ========== DomainAggregateType Display Tests ==========

    #[test]
    fn test_domain_aggregate_type_display_all() {
        let types = vec![
            (DomainAggregateType::Policy, "Policy"),
            (DomainAggregateType::Location, "Location"),
            (DomainAggregateType::Organization, "Organization"),
            (DomainAggregateType::Person, "Person"),
            (DomainAggregateType::Custom("MyType".to_string()), "Custom(MyType)"),
        ];

        for (agg_type, expected) in types {
            assert_eq!(format!("{}", agg_type), expected);
        }
    }
}
