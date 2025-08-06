//! Concept graph - semantic reasoning (event-driven projection)

// Projections are ephemeral - no serialization
use std::collections::HashMap;

pub use crate::core::projection_engine::GenericGraphProjection;
pub use crate::core::{Node, Edge};

/// Concept graph projection
pub type ConceptGraph = GenericGraphProjection<ConceptNode, ConceptEdge>;

/// Concept projection with additional semantic reasoning methods
pub type ConceptProjection = ConceptGraph;

/// Type of concept node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConceptNodeType {
    /// Core concept or idea
    Concept,
    /// Property of a concept
    Property,
    /// Instance of a concept
    Instance,
    /// Category or class
    Category,
    /// Rule or constraint
    Rule,
    /// Axiom or fundamental truth
    Axiom,
}

/// Concept node represents semantic knowledge
#[derive(Debug, Clone)]
pub struct ConceptNode {
    /// Unique identifier for the node
    pub id: String,
    /// Human-readable name of the concept
    pub name: String,
    /// Type of concept node
    pub node_type: ConceptNodeType,
    /// Optional description of the concept
    pub description: Option<String>,
    /// Properties defining the concept
    pub properties: HashMap<String, serde_json::Value>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ConceptNode {
    /// Create a new concept node
    pub fn new(id: impl Into<String>, name: impl Into<String>, node_type: ConceptNodeType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            node_type,
            description: None,
            properties: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a concept
    pub fn concept(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, ConceptNodeType::Concept)
    }

    /// Create a property
    pub fn property(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, ConceptNodeType::Property)
    }

    /// Create an instance
    pub fn instance(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, ConceptNodeType::Instance)
    }

    /// Create a category
    pub fn category(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, ConceptNodeType::Category)
    }

    /// Create a rule
    pub fn rule(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, ConceptNodeType::Rule)
    }

    /// Create an axiom
    pub fn axiom(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(id, name, ConceptNodeType::Axiom)
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }
}

impl Node for ConceptNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Type of semantic relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// IS-A relationship (inheritance)
    IsA,
    /// HAS-A relationship (composition)
    HasA,
    /// PART-OF relationship
    PartOf,
    /// RELATED-TO relationship
    RelatedTo,
    /// DEPENDS-ON relationship
    DependsOn,
    /// IMPLIES relationship
    Implies,
    /// CONTRADICTS relationship
    Contradicts,
    /// SIMILAR-TO relationship
    SimilarTo,
    /// DIFFERENT-FROM relationship
    DifferentFrom,
    /// INSTANCE-OF relationship
    InstanceOf,
    /// PROPERTY-OF relationship
    PropertyOf,
    /// CAUSES relationship
    Causes,
    /// PRECEDES relationship
    Precedes,
    /// Custom relationship
    Custom(String),
}

/// Concept edge represents semantic relationships
#[derive(Debug, Clone)]
pub struct ConceptEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source concept ID
    pub source: String,
    /// Target concept ID
    pub target: String,
    /// Type of semantic relationship
    pub relation_type: RelationType,
    /// Strength of the relationship (0.0 to 1.0)
    pub strength: f32,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ConceptEdge {
    /// Create a new concept edge
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        relation_type: RelationType,
    ) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            relation_type,
            strength: 1.0,
            metadata: HashMap::new(),
        }
    }

    /// Create an IS-A relationship
    pub fn is_a(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, RelationType::IsA)
    }

    /// Create a HAS-A relationship
    pub fn has_a(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, RelationType::HasA)
    }

    /// Create an INSTANCE-OF relationship
    pub fn instance_of(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, RelationType::InstanceOf)
    }

    /// Create a PROPERTY-OF relationship
    pub fn property_of(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self::new(id, source, target, RelationType::PropertyOf)
    }

    /// Set the strength of the relationship
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }
}

impl Edge for ConceptEdge {
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

/// Extension methods for ConceptProjection
impl ConceptProjection {
    /// Get all concepts
    pub fn get_concepts(&self) -> Vec<&ConceptNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, ConceptNodeType::Concept))
            .collect()
    }

    /// Get all categories
    pub fn get_categories(&self) -> Vec<&ConceptNode> {
        self.nodes()
            .filter(|n| matches!(n.node_type, ConceptNodeType::Category))
            .collect()
    }

    /// Get all instances of a concept
    pub fn get_instances_of(&self, concept_id: &str) -> Vec<&ConceptNode> {
        self.edges()
            .filter(|e| {
                matches!(e.relation_type, RelationType::InstanceOf) && e.target() == concept_id
            })
            .filter_map(|e| self.get_node(&e.source()))
            .collect()
    }

    /// Get all properties of a concept
    pub fn get_properties_of(&self, concept_id: &str) -> Vec<&ConceptNode> {
        self.edges()
            .filter(|e| {
                matches!(e.relation_type, RelationType::PropertyOf) && e.target() == concept_id
            })
            .filter_map(|e| self.get_node(&e.source()))
            .collect()
    }

    /// Get all parent concepts (IS-A relationships)
    pub fn get_parents(&self, concept_id: &str) -> Vec<&ConceptNode> {
        self.edges()
            .filter(|e| {
                matches!(e.relation_type, RelationType::IsA) && e.source() == concept_id
            })
            .filter_map(|e| self.get_node(&e.target()))
            .collect()
    }

    /// Get all child concepts (inverse IS-A relationships)
    pub fn get_children(&self, concept_id: &str) -> Vec<&ConceptNode> {
        self.edges()
            .filter(|e| {
                matches!(e.relation_type, RelationType::IsA) && e.target() == concept_id
            })
            .filter_map(|e| self.get_node(&e.source()))
            .collect()
    }

    /// Find all concepts related by a specific relation type
    pub fn get_related(&self, concept_id: &str, relation: &RelationType) -> Vec<&ConceptNode> {
        self.edges()
            .filter(|e| &e.relation_type == relation && e.source() == concept_id)
            .filter_map(|e| self.get_node(&e.target()))
            .collect()
    }

    /// Calculate semantic distance between two concepts
    pub fn semantic_distance(&self, from: &str, to: &str) -> Option<f32> {
        use std::collections::{HashMap, BinaryHeap};
        use std::cmp::Ordering;
        
        #[derive(Clone, PartialEq)]
        struct State {
            cost: f32,
            node: String,
        }
        
        impl Eq for State {}
        
        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
            }
        }
        
        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let mut dist = HashMap::new();
        let mut heap = BinaryHeap::new();
        
        dist.insert(from.to_string(), 0.0);
        heap.push(State { cost: 0.0, node: from.to_string() });
        
        while let Some(State { cost, node }) = heap.pop() {
            if node == to {
                return Some(cost);
            }
            
            if cost > *dist.get(&node).unwrap_or(&f32::INFINITY) {
                continue;
            }
            
            for edge in self.edges().filter(|e| e.source() == node) {
                let next = State {
                    cost: cost + (1.0 - edge.strength),
                    node: edge.target(),
                };
                
                if next.cost < *dist.get(&next.node).unwrap_or(&f32::INFINITY) {
                    heap.push(next.clone());
                    dist.insert(next.node, next.cost);
                }
            }
        }
        
        None
    }

    /// Find all paths between two concepts
    pub fn find_reasoning_paths(&self, from: &str, to: &str, max_depth: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![from.to_string()];
        let mut visited = std::collections::HashSet::new();
        
        self.dfs_paths(from, to, &mut current_path, &mut visited, &mut paths, max_depth);
        
        paths
    }

    fn dfs_paths(
        &self,
        current: &str,
        target: &str,
        path: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        paths: &mut Vec<Vec<String>>,
        max_depth: usize,
    ) {
        if current == target {
            paths.push(path.clone());
            return;
        }
        
        if path.len() >= max_depth {
            return;
        }
        
        visited.insert(current.to_string());
        
        for edge in self.edges().filter(|e| e.source() == current) {
            let next = edge.target();
            if !visited.contains(&next) {
                path.push(next.clone());
                self.dfs_paths(&next, target, path, visited, paths, max_depth);
                path.pop();
            }
        }
        
        visited.remove(current);
    }

    /// Infer new relationships based on existing ones
    pub fn infer_relationships(&self) -> Vec<(String, String, RelationType)> {
        let mut inferred = Vec::new();
        
        // Transitivity of IS-A relationships
        for node in self.nodes() {
            let parents = self.get_parents(&node.id);
            for parent in &parents {
                let grandparents = self.get_parents(&parent.id);
                for grandparent in grandparents {
                    // If A IS-A B and B IS-A C, then A IS-A C
                    if !self.edges().any(|e| {
                        matches!(e.relation_type, RelationType::IsA)
                            && e.source() == node.id
                            && e.target() == grandparent.id
                    }) {
                        inferred.push((node.id.clone(), grandparent.id.clone(), RelationType::IsA));
                    }
                }
            }
        }
        
        // Instance inheritance
        for edge in self.edges() {
            if matches!(edge.relation_type, RelationType::InstanceOf) {
                let instance_id = &edge.source;
                let concept_id = &edge.target;
                
                // If A INSTANCE-OF B and B IS-A C, then A INSTANCE-OF C
                for parent_edge in self.edges() {
                    if matches!(parent_edge.relation_type, RelationType::IsA)
                        && parent_edge.source() == *concept_id
                    {
                        let parent_concept = &parent_edge.target;
                        if !self.edges().any(|e| {
                            matches!(e.relation_type, RelationType::InstanceOf)
                                && e.source() == *instance_id
                                && e.target() == *parent_concept
                        }) {
                            inferred.push((
                                instance_id.clone(),
                                parent_concept.clone(),
                                RelationType::InstanceOf,
                            ));
                        }
                    }
                }
            }
        }
        
        inferred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concept_node_creation() {
        let concept = ConceptNode::concept("c1", "Animal");
        assert!(matches!(concept.node_type, ConceptNodeType::Concept));
        assert_eq!(concept.name, "Animal");
        
        let property = ConceptNode::property("p1", "has_legs")
            .with_description("Number of legs an animal has");
        assert!(matches!(property.node_type, ConceptNodeType::Property));
        assert!(property.description.is_some());
    }

    #[test]
    fn test_concept_edge_creation() {
        let is_a = ConceptEdge::is_a("e1", "Dog", "Animal");
        assert!(matches!(is_a.relation_type, RelationType::IsA));
        assert_eq!(is_a.strength, 1.0);
        
        let weak_relation = ConceptEdge::new("e2", "Cat", "Water", RelationType::RelatedTo)
            .with_strength(0.3);
        assert_eq!(weak_relation.strength, 0.3);
    }

    #[test]
    fn test_relation_types() {
        let edge1 = ConceptEdge::instance_of("e1", "Fido", "Dog");
        assert!(matches!(edge1.relation_type, RelationType::InstanceOf));
        
        let edge2 = ConceptEdge::property_of("e2", "has_tail", "Dog");
        assert!(matches!(edge2.relation_type, RelationType::PropertyOf));
    }
}