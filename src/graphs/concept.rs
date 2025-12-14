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

    // ========== ConceptNodeType Tests ==========

    #[test]
    fn test_concept_node_type_equality() {
        assert_eq!(ConceptNodeType::Concept, ConceptNodeType::Concept);
        assert_eq!(ConceptNodeType::Property, ConceptNodeType::Property);
        assert_eq!(ConceptNodeType::Instance, ConceptNodeType::Instance);
        assert_eq!(ConceptNodeType::Category, ConceptNodeType::Category);
        assert_eq!(ConceptNodeType::Rule, ConceptNodeType::Rule);
        assert_eq!(ConceptNodeType::Axiom, ConceptNodeType::Axiom);

        assert_ne!(ConceptNodeType::Concept, ConceptNodeType::Property);
        assert_ne!(ConceptNodeType::Instance, ConceptNodeType::Category);
    }

    #[test]
    fn test_concept_node_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ConceptNodeType::Concept);
        set.insert(ConceptNodeType::Property);
        set.insert(ConceptNodeType::Concept); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&ConceptNodeType::Concept));
        assert!(set.contains(&ConceptNodeType::Property));
    }

    // ========== ConceptNode Factory Methods ==========

    #[test]
    fn test_concept_node_new() {
        let node = ConceptNode::new("id1", "Test Concept", ConceptNodeType::Concept);

        assert_eq!(node.id, "id1");
        assert_eq!(node.name, "Test Concept");
        assert!(matches!(node.node_type, ConceptNodeType::Concept));
        assert!(node.description.is_none());
        assert!(node.properties.is_empty());
        assert!(node.metadata.is_empty());
    }

    #[test]
    fn test_concept_node_property() {
        let node = ConceptNode::property("p1", "color");
        assert_eq!(node.id, "p1");
        assert_eq!(node.name, "color");
        assert!(matches!(node.node_type, ConceptNodeType::Property));
    }

    #[test]
    fn test_concept_node_instance() {
        let node = ConceptNode::instance("i1", "Fido the Dog");
        assert_eq!(node.id, "i1");
        assert_eq!(node.name, "Fido the Dog");
        assert!(matches!(node.node_type, ConceptNodeType::Instance));
    }

    #[test]
    fn test_concept_node_category() {
        let node = ConceptNode::category("cat1", "Mammals");
        assert_eq!(node.id, "cat1");
        assert_eq!(node.name, "Mammals");
        assert!(matches!(node.node_type, ConceptNodeType::Category));
    }

    #[test]
    fn test_concept_node_rule() {
        let node = ConceptNode::rule("r1", "All mammals breathe air");
        assert_eq!(node.id, "r1");
        assert_eq!(node.name, "All mammals breathe air");
        assert!(matches!(node.node_type, ConceptNodeType::Rule));
    }

    #[test]
    fn test_concept_node_axiom() {
        let node = ConceptNode::axiom("ax1", "Living things require energy");
        assert_eq!(node.id, "ax1");
        assert_eq!(node.name, "Living things require energy");
        assert!(matches!(node.node_type, ConceptNodeType::Axiom));
    }

    #[test]
    fn test_concept_node_with_description() {
        let node = ConceptNode::concept("c1", "Animal")
            .with_description("A living organism that can move");

        assert_eq!(node.description, Some("A living organism that can move".to_string()));
    }

    #[test]
    fn test_concept_node_with_property() {
        let node = ConceptNode::concept("c1", "Animal")
            .with_property("legs", serde_json::json!(4))
            .with_property("vertebrate", serde_json::json!(true));

        assert_eq!(node.properties.len(), 2);
        assert_eq!(node.properties["legs"], serde_json::json!(4));
        assert_eq!(node.properties["vertebrate"], serde_json::json!(true));
    }

    #[test]
    fn test_concept_node_builder_chain() {
        let node = ConceptNode::concept("c1", "Dog")
            .with_description("A domesticated carnivorous mammal")
            .with_property("domesticated", serde_json::json!(true))
            .with_property("species", serde_json::json!("Canis familiaris"));

        assert_eq!(node.id, "c1");
        assert_eq!(node.name, "Dog");
        assert!(node.description.is_some());
        assert_eq!(node.properties.len(), 2);
    }

    #[test]
    fn test_concept_node_implements_node_trait() {
        let node = ConceptNode::concept("trait_test", "Test");
        assert_eq!(Node::id(&node), "trait_test");
    }

    // ========== RelationType Tests ==========

    #[test]
    fn test_relation_type_variants() {
        let isa = RelationType::IsA;
        let hasa = RelationType::HasA;
        let partof = RelationType::PartOf;
        let relatedto = RelationType::RelatedTo;
        let dependson = RelationType::DependsOn;
        let implies = RelationType::Implies;
        let contradicts = RelationType::Contradicts;
        let similarto = RelationType::SimilarTo;
        let differentfrom = RelationType::DifferentFrom;
        let instanceof = RelationType::InstanceOf;
        let propertyof = RelationType::PropertyOf;
        let causes = RelationType::Causes;
        let precedes = RelationType::Precedes;
        let custom = RelationType::Custom("custom_rel".to_string());

        assert!(matches!(isa, RelationType::IsA));
        assert!(matches!(hasa, RelationType::HasA));
        assert!(matches!(partof, RelationType::PartOf));
        assert!(matches!(relatedto, RelationType::RelatedTo));
        assert!(matches!(dependson, RelationType::DependsOn));
        assert!(matches!(implies, RelationType::Implies));
        assert!(matches!(contradicts, RelationType::Contradicts));
        assert!(matches!(similarto, RelationType::SimilarTo));
        assert!(matches!(differentfrom, RelationType::DifferentFrom));
        assert!(matches!(instanceof, RelationType::InstanceOf));
        assert!(matches!(propertyof, RelationType::PropertyOf));
        assert!(matches!(causes, RelationType::Causes));
        assert!(matches!(precedes, RelationType::Precedes));
        assert!(matches!(custom, RelationType::Custom(_)));
    }

    #[test]
    fn test_relation_type_equality() {
        assert_eq!(RelationType::IsA, RelationType::IsA);
        assert_ne!(RelationType::IsA, RelationType::HasA);

        let custom1 = RelationType::Custom("rel".to_string());
        let custom2 = RelationType::Custom("rel".to_string());
        let custom3 = RelationType::Custom("other".to_string());

        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
    }

    #[test]
    fn test_relation_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(RelationType::IsA);
        set.insert(RelationType::HasA);
        set.insert(RelationType::IsA); // Duplicate

        assert_eq!(set.len(), 2);
    }

    // ========== ConceptEdge Factory Methods ==========

    #[test]
    fn test_concept_edge_new() {
        let edge = ConceptEdge::new("e1", "src", "tgt", RelationType::RelatedTo);

        assert_eq!(edge.id, "e1");
        assert_eq!(edge.source, "src");
        assert_eq!(edge.target, "tgt");
        assert!(matches!(edge.relation_type, RelationType::RelatedTo));
        assert_eq!(edge.strength, 1.0);
        assert!(edge.metadata.is_empty());
    }

    #[test]
    fn test_concept_edge_is_a() {
        let edge = ConceptEdge::is_a("e1", "Dog", "Animal");

        assert_eq!(edge.source, "Dog");
        assert_eq!(edge.target, "Animal");
        assert!(matches!(edge.relation_type, RelationType::IsA));
        assert_eq!(edge.strength, 1.0);
    }

    #[test]
    fn test_concept_edge_has_a() {
        let edge = ConceptEdge::has_a("e1", "Car", "Engine");

        assert_eq!(edge.source, "Car");
        assert_eq!(edge.target, "Engine");
        assert!(matches!(edge.relation_type, RelationType::HasA));
    }

    #[test]
    fn test_concept_edge_instance_of() {
        let edge = ConceptEdge::instance_of("e1", "Fido", "Dog");

        assert_eq!(edge.source, "Fido");
        assert_eq!(edge.target, "Dog");
        assert!(matches!(edge.relation_type, RelationType::InstanceOf));
    }

    #[test]
    fn test_concept_edge_property_of() {
        let edge = ConceptEdge::property_of("e1", "has_fur", "Dog");

        assert_eq!(edge.source, "has_fur");
        assert_eq!(edge.target, "Dog");
        assert!(matches!(edge.relation_type, RelationType::PropertyOf));
    }

    #[test]
    fn test_concept_edge_with_strength() {
        let edge = ConceptEdge::new("e1", "Cat", "Water", RelationType::RelatedTo)
            .with_strength(0.2);

        assert_eq!(edge.strength, 0.2);
    }

    #[test]
    fn test_concept_edge_strength_clamping() {
        let edge1 = ConceptEdge::new("e1", "A", "B", RelationType::RelatedTo)
            .with_strength(-0.5);
        assert_eq!(edge1.strength, 0.0);

        let edge2 = ConceptEdge::new("e2", "C", "D", RelationType::RelatedTo)
            .with_strength(1.5);
        assert_eq!(edge2.strength, 1.0);

        let edge3 = ConceptEdge::new("e3", "E", "F", RelationType::RelatedTo)
            .with_strength(0.5);
        assert_eq!(edge3.strength, 0.5);
    }

    #[test]
    fn test_concept_edge_implements_edge_trait() {
        let edge = ConceptEdge::is_a("trait_test", "A", "B");

        assert_eq!(Edge::id(&edge), "trait_test");
        assert_eq!(Edge::source(&edge), "A");
        assert_eq!(Edge::target(&edge), "B");
    }

    // ========== ConceptProjection Extension Methods ==========

    fn create_animal_taxonomy() -> ConceptProjection {
        let mut projection = ConceptProjection::new(uuid::Uuid::new_v4(), crate::core::GraphType::ConceptGraph);

        // Categories
        projection.nodes.insert("living_thing".to_string(), ConceptNode::category("living_thing", "Living Thing"));
        projection.nodes.insert("animal".to_string(), ConceptNode::category("animal", "Animal"));
        projection.nodes.insert("mammal".to_string(), ConceptNode::category("mammal", "Mammal"));
        projection.nodes.insert("bird".to_string(), ConceptNode::category("bird", "Bird"));

        // Concepts
        projection.nodes.insert("dog".to_string(), ConceptNode::concept("dog", "Dog"));
        projection.nodes.insert("cat".to_string(), ConceptNode::concept("cat", "Cat"));
        projection.nodes.insert("eagle".to_string(), ConceptNode::concept("eagle", "Eagle"));

        // Instances
        projection.nodes.insert("fido".to_string(), ConceptNode::instance("fido", "Fido"));
        projection.nodes.insert("whiskers".to_string(), ConceptNode::instance("whiskers", "Whiskers"));

        // Properties
        projection.nodes.insert("has_fur".to_string(), ConceptNode::property("has_fur", "Has Fur"));
        projection.nodes.insert("has_feathers".to_string(), ConceptNode::property("has_feathers", "Has Feathers"));

        // IS-A relationships (taxonomy hierarchy)
        projection.edges.insert("e1".to_string(), ConceptEdge::is_a("e1", "animal", "living_thing"));
        projection.edges.insert("e2".to_string(), ConceptEdge::is_a("e2", "mammal", "animal"));
        projection.edges.insert("e3".to_string(), ConceptEdge::is_a("e3", "bird", "animal"));
        projection.edges.insert("e4".to_string(), ConceptEdge::is_a("e4", "dog", "mammal"));
        projection.edges.insert("e5".to_string(), ConceptEdge::is_a("e5", "cat", "mammal"));
        projection.edges.insert("e6".to_string(), ConceptEdge::is_a("e6", "eagle", "bird"));

        // INSTANCE-OF relationships
        projection.edges.insert("e7".to_string(), ConceptEdge::instance_of("e7", "fido", "dog"));
        projection.edges.insert("e8".to_string(), ConceptEdge::instance_of("e8", "whiskers", "cat"));

        // PROPERTY-OF relationships
        projection.edges.insert("e9".to_string(), ConceptEdge::property_of("e9", "has_fur", "mammal"));
        projection.edges.insert("e10".to_string(), ConceptEdge::property_of("e10", "has_feathers", "bird"));

        // Update adjacency
        for edge in projection.edges.values() {
            projection.adjacency
                .entry(edge.source.clone())
                .or_insert_with(Vec::new)
                .push(edge.target.clone());
        }

        projection
    }

    #[test]
    fn test_get_concepts() {
        let projection = create_animal_taxonomy();
        let concepts = projection.get_concepts();

        assert_eq!(concepts.len(), 3); // dog, cat, eagle
        let ids: Vec<_> = concepts.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"dog"));
        assert!(ids.contains(&"cat"));
        assert!(ids.contains(&"eagle"));
    }

    #[test]
    fn test_get_categories() {
        let projection = create_animal_taxonomy();
        let categories = projection.get_categories();

        assert_eq!(categories.len(), 4); // living_thing, animal, mammal, bird
        let ids: Vec<_> = categories.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"living_thing"));
        assert!(ids.contains(&"animal"));
        assert!(ids.contains(&"mammal"));
        assert!(ids.contains(&"bird"));
    }

    #[test]
    fn test_get_instances_of() {
        let projection = create_animal_taxonomy();

        let dog_instances = projection.get_instances_of("dog");
        assert_eq!(dog_instances.len(), 1);
        assert_eq!(dog_instances[0].id, "fido");

        let cat_instances = projection.get_instances_of("cat");
        assert_eq!(cat_instances.len(), 1);
        assert_eq!(cat_instances[0].id, "whiskers");

        let mammal_instances = projection.get_instances_of("mammal");
        assert_eq!(mammal_instances.len(), 0); // No direct instances
    }

    #[test]
    fn test_get_properties_of() {
        let projection = create_animal_taxonomy();

        let mammal_props = projection.get_properties_of("mammal");
        assert_eq!(mammal_props.len(), 1);
        assert_eq!(mammal_props[0].id, "has_fur");

        let bird_props = projection.get_properties_of("bird");
        assert_eq!(bird_props.len(), 1);
        assert_eq!(bird_props[0].id, "has_feathers");

        let dog_props = projection.get_properties_of("dog");
        assert_eq!(dog_props.len(), 0); // No direct properties
    }

    #[test]
    fn test_get_parents() {
        let projection = create_animal_taxonomy();

        let dog_parents = projection.get_parents("dog");
        assert_eq!(dog_parents.len(), 1);
        assert_eq!(dog_parents[0].id, "mammal");

        let mammal_parents = projection.get_parents("mammal");
        assert_eq!(mammal_parents.len(), 1);
        assert_eq!(mammal_parents[0].id, "animal");

        let living_thing_parents = projection.get_parents("living_thing");
        assert_eq!(living_thing_parents.len(), 0); // Root node
    }

    #[test]
    fn test_get_children() {
        let projection = create_animal_taxonomy();

        let mammal_children = projection.get_children("mammal");
        assert_eq!(mammal_children.len(), 2); // dog, cat
        let ids: Vec<_> = mammal_children.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"dog"));
        assert!(ids.contains(&"cat"));

        let animal_children = projection.get_children("animal");
        assert_eq!(animal_children.len(), 2); // mammal, bird

        let dog_children = projection.get_children("dog");
        assert_eq!(dog_children.len(), 0); // Leaf node (no IS-A children)
    }

    #[test]
    fn test_get_related() {
        let projection = create_animal_taxonomy();

        let dog_is_a = projection.get_related("dog", &RelationType::IsA);
        assert_eq!(dog_is_a.len(), 1);
        assert_eq!(dog_is_a[0].id, "mammal");

        let fido_instance_of = projection.get_related("fido", &RelationType::InstanceOf);
        assert_eq!(fido_instance_of.len(), 1);
        assert_eq!(fido_instance_of[0].id, "dog");
    }

    // ========== Semantic Distance Tests ==========

    #[test]
    fn test_semantic_distance_adjacent() {
        let projection = create_animal_taxonomy();

        let distance = projection.semantic_distance("dog", "mammal");
        assert!(distance.is_some());
        // With strength 1.0, cost = 1.0 - 1.0 = 0.0
        assert_eq!(distance.unwrap(), 0.0);
    }

    #[test]
    fn test_semantic_distance_two_hops() {
        let projection = create_animal_taxonomy();

        let distance = projection.semantic_distance("dog", "animal");
        assert!(distance.is_some());
        // dog -> mammal -> animal = 0.0 + 0.0 = 0.0 (all edges have strength 1.0)
        assert_eq!(distance.unwrap(), 0.0);
    }

    #[test]
    fn test_semantic_distance_no_path() {
        let projection = create_animal_taxonomy();

        // No path from fido to eagle (different branches)
        let distance = projection.semantic_distance("fido", "eagle");
        // Actually there is a path through instance_of -> is_a chain
        // fido -> dog -> mammal -> animal <- bird <- eagle
        // But edges are directional, so no path from fido to eagle
        assert!(distance.is_none());
    }

    #[test]
    fn test_semantic_distance_same_node() {
        let projection = create_animal_taxonomy();

        let distance = projection.semantic_distance("dog", "dog");
        assert!(distance.is_some());
        assert_eq!(distance.unwrap(), 0.0);
    }

    #[test]
    fn test_semantic_distance_nonexistent_node() {
        let projection = create_animal_taxonomy();

        let distance = projection.semantic_distance("nonexistent", "dog");
        // Starting from nonexistent node, should return None
        assert!(distance.is_none());
    }

    #[test]
    fn test_semantic_distance_with_weak_edges() {
        let mut projection = ConceptProjection::new(uuid::Uuid::new_v4(), crate::core::GraphType::ConceptGraph);

        projection.nodes.insert("a".to_string(), ConceptNode::concept("a", "A"));
        projection.nodes.insert("b".to_string(), ConceptNode::concept("b", "B"));
        projection.nodes.insert("c".to_string(), ConceptNode::concept("c", "C"));

        // Strong path: a -> b (strength 1.0, cost 0.0)
        projection.edges.insert("e1".to_string(),
            ConceptEdge::new("e1", "a", "b", RelationType::RelatedTo).with_strength(1.0));

        // Weak path: a -> c (strength 0.5, cost 0.5)
        projection.edges.insert("e2".to_string(),
            ConceptEdge::new("e2", "a", "c", RelationType::RelatedTo).with_strength(0.5));

        projection.adjacency.insert("a".to_string(), vec!["b".to_string(), "c".to_string()]);
        projection.adjacency.insert("b".to_string(), vec![]);
        projection.adjacency.insert("c".to_string(), vec![]);

        let distance_ab = projection.semantic_distance("a", "b").unwrap();
        let distance_ac = projection.semantic_distance("a", "c").unwrap();

        assert!(distance_ab < distance_ac); // Strong edge = shorter semantic distance
    }

    // ========== Reasoning Path Tests ==========

    #[test]
    fn test_find_reasoning_paths_direct() {
        let projection = create_animal_taxonomy();

        let paths = projection.find_reasoning_paths("dog", "mammal", 5);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["dog", "mammal"]);
    }

    #[test]
    fn test_find_reasoning_paths_multiple_hops() {
        let projection = create_animal_taxonomy();

        let paths = projection.find_reasoning_paths("dog", "living_thing", 5);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["dog", "mammal", "animal", "living_thing"]);
    }

    #[test]
    fn test_find_reasoning_paths_no_path() {
        let projection = create_animal_taxonomy();

        // No path from living_thing to dog (edges go the other way)
        let paths = projection.find_reasoning_paths("living_thing", "dog", 5);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_find_reasoning_paths_same_node() {
        let projection = create_animal_taxonomy();

        let paths = projection.find_reasoning_paths("dog", "dog", 5);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["dog"]);
    }

    #[test]
    fn test_find_reasoning_paths_max_depth() {
        let projection = create_animal_taxonomy();

        // Path exists but is 4 nodes: dog -> mammal -> animal -> living_thing
        let paths_shallow = projection.find_reasoning_paths("dog", "living_thing", 2);
        assert!(paths_shallow.is_empty()); // Too shallow

        let paths_deep = projection.find_reasoning_paths("dog", "living_thing", 5);
        assert_eq!(paths_deep.len(), 1); // Deep enough
    }

    #[test]
    fn test_find_reasoning_paths_multiple_paths() {
        let mut projection = ConceptProjection::new(uuid::Uuid::new_v4(), crate::core::GraphType::ConceptGraph);

        projection.nodes.insert("a".to_string(), ConceptNode::concept("a", "A"));
        projection.nodes.insert("b".to_string(), ConceptNode::concept("b", "B"));
        projection.nodes.insert("c".to_string(), ConceptNode::concept("c", "C"));
        projection.nodes.insert("d".to_string(), ConceptNode::concept("d", "D"));

        // Diamond: a -> b -> d, a -> c -> d
        projection.edges.insert("e1".to_string(), ConceptEdge::new("e1", "a", "b", RelationType::RelatedTo));
        projection.edges.insert("e2".to_string(), ConceptEdge::new("e2", "a", "c", RelationType::RelatedTo));
        projection.edges.insert("e3".to_string(), ConceptEdge::new("e3", "b", "d", RelationType::RelatedTo));
        projection.edges.insert("e4".to_string(), ConceptEdge::new("e4", "c", "d", RelationType::RelatedTo));

        projection.adjacency.insert("a".to_string(), vec!["b".to_string(), "c".to_string()]);
        projection.adjacency.insert("b".to_string(), vec!["d".to_string()]);
        projection.adjacency.insert("c".to_string(), vec!["d".to_string()]);
        projection.adjacency.insert("d".to_string(), vec![]);

        let paths = projection.find_reasoning_paths("a", "d", 5);
        assert_eq!(paths.len(), 2);
    }

    // ========== Inference Tests ==========

    #[test]
    fn test_infer_relationships_transitive_isa() {
        let projection = create_animal_taxonomy();

        let inferred = projection.infer_relationships();

        // Should infer: dog IS-A animal (transitive through mammal)
        // Should infer: cat IS-A animal (transitive through mammal)
        // Should infer: eagle IS-A animal (transitive through bird) - already exists
        // Should infer: dog IS-A living_thing, cat IS-A living_thing, etc.

        let dog_animal = inferred.iter().find(|(src, tgt, rel)|
            src == "dog" && tgt == "animal" && matches!(rel, RelationType::IsA)
        );
        assert!(dog_animal.is_some(), "Should infer dog IS-A animal");

        let cat_animal = inferred.iter().find(|(src, tgt, rel)|
            src == "cat" && tgt == "animal" && matches!(rel, RelationType::IsA)
        );
        assert!(cat_animal.is_some(), "Should infer cat IS-A animal");
    }

    #[test]
    fn test_infer_relationships_instance_inheritance() {
        let projection = create_animal_taxonomy();

        let inferred = projection.infer_relationships();

        // Should infer: fido INSTANCE-OF mammal (since fido INSTANCE-OF dog and dog IS-A mammal)
        let fido_mammal = inferred.iter().find(|(src, tgt, rel)|
            src == "fido" && tgt == "mammal" && matches!(rel, RelationType::InstanceOf)
        );
        assert!(fido_mammal.is_some(), "Should infer fido INSTANCE-OF mammal");
    }

    #[test]
    fn test_infer_relationships_empty_graph() {
        let projection = ConceptProjection::new(uuid::Uuid::new_v4(), crate::core::GraphType::ConceptGraph);
        let inferred = projection.infer_relationships();
        assert!(inferred.is_empty());
    }

    #[test]
    fn test_infer_no_duplicate_existing() {
        let mut projection = ConceptProjection::new(uuid::Uuid::new_v4(), crate::core::GraphType::ConceptGraph);

        projection.nodes.insert("a".to_string(), ConceptNode::concept("a", "A"));
        projection.nodes.insert("b".to_string(), ConceptNode::concept("b", "B"));
        projection.nodes.insert("c".to_string(), ConceptNode::concept("c", "C"));

        // a IS-A b, b IS-A c, and a IS-A c already exists
        projection.edges.insert("e1".to_string(), ConceptEdge::is_a("e1", "a", "b"));
        projection.edges.insert("e2".to_string(), ConceptEdge::is_a("e2", "b", "c"));
        projection.edges.insert("e3".to_string(), ConceptEdge::is_a("e3", "a", "c")); // Already exists

        for edge in projection.edges.values() {
            projection.adjacency
                .entry(edge.source.clone())
                .or_insert_with(Vec::new)
                .push(edge.target.clone());
        }

        let inferred = projection.infer_relationships();

        // Should NOT infer a IS-A c since it already exists
        let a_c = inferred.iter().filter(|(src, tgt, rel)|
            src == "a" && tgt == "c" && matches!(rel, RelationType::IsA)
        ).count();
        assert_eq!(a_c, 0, "Should not infer already existing relationship");
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_empty_projection_methods() {
        let projection = ConceptProjection::new(uuid::Uuid::new_v4(), crate::core::GraphType::ConceptGraph);

        assert!(projection.get_concepts().is_empty());
        assert!(projection.get_categories().is_empty());
        assert!(projection.get_instances_of("anything").is_empty());
        assert!(projection.get_properties_of("anything").is_empty());
        assert!(projection.get_parents("anything").is_empty());
        assert!(projection.get_children("anything").is_empty());
        assert!(projection.get_related("anything", &RelationType::IsA).is_empty());
        assert!(projection.semantic_distance("a", "b").is_none());
        assert!(projection.find_reasoning_paths("a", "b", 5).is_empty());
    }

    #[test]
    fn test_custom_relation_type() {
        let edge = ConceptEdge::new("e1", "concept1", "concept2",
            RelationType::Custom("specializes".to_string()));

        match &edge.relation_type {
            RelationType::Custom(name) => assert_eq!(name, "specializes"),
            _ => panic!("Expected Custom relation type"),
        }
    }

    #[test]
    fn test_node_with_complex_properties() {
        let node = ConceptNode::concept("c1", "Complex Concept")
            .with_property("array", serde_json::json!([1, 2, 3]))
            .with_property("nested", serde_json::json!({
                "level1": {
                    "level2": "value"
                }
            }))
            .with_property("null", serde_json::Value::Null)
            .with_property("float", serde_json::json!(3.14));

        assert_eq!(node.properties.len(), 4);
        assert!(node.properties["array"].is_array());
        assert!(node.properties["nested"].is_object());
        assert!(node.properties["null"].is_null());
        assert!(node.properties["float"].is_number());
    }
}