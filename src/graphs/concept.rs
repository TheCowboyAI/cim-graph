//! Concept graph for semantic reasoning and knowledge representation
//! 
//! Represents concepts, properties, and semantic relationships

use crate::core::{EventGraph, EventHandler, GraphBuilder, GraphType};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

/// Types of concepts in a semantic graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConceptType {
    /// Class or category
    Class,
    /// Instance of a class
    Instance,
    /// Property or attribute
    Property,
    /// Relation between concepts
    Relation,
    /// Axiom or rule
    Axiom,
    /// Literal value
    Literal,
}

/// Node representing a concept in the semantic graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    /// Unique identifier (often an IRI in semantic web)
    id: String,
    /// Human-readable label
    label: String,
    /// Type of concept
    concept_type: ConceptType,
    /// Properties and their values
    properties: HashMap<String, Vec<serde_json::Value>>,
    /// Annotations (metadata)
    annotations: HashMap<String, String>,
}

impl ConceptNode {
    /// Create a new concept node
    pub fn new(id: impl Into<String>, label: impl Into<String>, concept_type: ConceptType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            concept_type,
            properties: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
    
    /// Add a property value
    pub fn add_property(&mut self, property: impl Into<String>, value: serde_json::Value) {
        self.properties
            .entry(property.into())
            .or_insert_with(Vec::new)
            .push(value);
    }
    
    /// Get property values
    pub fn get_property(&self, property: &str) -> Option<&Vec<serde_json::Value>> {
        self.properties.get(property)
    }
    
    /// Add an annotation
    pub fn add_annotation(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.annotations.insert(key.into(), value.into());
    }
    
    /// Get concept type
    pub fn concept_type(&self) -> ConceptType {
        self.concept_type
    }
    
    /// Get label
    pub fn label(&self) -> &str {
        &self.label
    }
    
    /// Get name (alias for label)
    pub fn name(&self) -> &str {
        &self.label
    }
}

impl crate::core::Node for ConceptNode {
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// Types of semantic relationships
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRelation {
    /// Subclass relationship (is-a)
    SubClassOf,
    /// Subclass relationship (alternate name)
    IsA,
    /// Instance relationship (instance-of)
    InstanceOf,
    /// Part-whole relationship (part-of)
    PartOf,
    /// Property domain
    HasDomain,
    /// Property range
    HasRange,
    /// Equivalence
    EquivalentTo,
    /// Object property
    HasProperty,
    /// Disjointness
    DisjointWith,
    /// Contradiction
    Contradicts,
    /// Causes relationship
    Causes,
    /// Treats relationship
    Treats,
    /// Custom relation
    Custom,
}

impl std::fmt::Display for SemanticRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticRelation::SubClassOf => write!(f, "subClassOf"),
            SemanticRelation::IsA => write!(f, "isA"),
            SemanticRelation::InstanceOf => write!(f, "instanceOf"),
            SemanticRelation::PartOf => write!(f, "partOf"),
            SemanticRelation::HasDomain => write!(f, "hasDomain"),
            SemanticRelation::HasRange => write!(f, "hasRange"),
            SemanticRelation::EquivalentTo => write!(f, "equivalentTo"),
            SemanticRelation::HasProperty => write!(f, "hasProperty"),
            SemanticRelation::DisjointWith => write!(f, "disjointWith"),
            SemanticRelation::Contradicts => write!(f, "contradicts"),
            SemanticRelation::Causes => write!(f, "causes"),
            SemanticRelation::Treats => write!(f, "treats"),
            SemanticRelation::Custom => write!(f, "custom"),
        }
    }
}

/// Edge representing a semantic relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    /// Unique identifier
    pub id: String,
    /// Source concept
    pub source: String,
    /// Target concept
    pub target: String,
    /// Type of relation
    pub relation: SemanticRelation,
    /// Relation label (for custom relations)
    label: String,
    /// Confidence or weight
    confidence: f32,
    /// Properties of the relation
    properties: HashMap<String, serde_json::Value>,
}

impl ConceptEdge {
    /// Create a new semantic edge
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        relation: SemanticRelation,
    ) -> Self {
        let source = source.into();
        let target = target.into();
        let label = format!("{:?}", relation);
        let id = format!("{}:{}:{}", source, label, target);
        
        Self {
            id,
            source,
            target,
            relation,
            label,
            confidence: 1.0,
            properties: HashMap::new(),
        }
    }
    
    /// Create a custom relation
    pub fn custom(
        source: impl Into<String>,
        target: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let target = target.into();
        let label = label.into();
        let id = format!("{}:{}:{}", source, label, target);
        
        Self {
            id,
            source,
            target,
            relation: SemanticRelation::Custom,
            label,
            confidence: 1.0,
            properties: HashMap::new(),
        }
    }
    
    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
    
    /// Get relation type
    pub fn relation(&self) -> SemanticRelation {
        self.relation
    }
    
    /// Get confidence
    pub fn confidence(&self) -> f32 {
        self.confidence
    }
}

impl crate::core::Edge for ConceptEdge {
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

/// Semantic reasoning graph
pub struct ConceptGraph {
    /// Underlying event-driven graph
    graph: EventGraph<ConceptNode, ConceptEdge>,
    /// Inference rules
    rules: Vec<InferenceRule>,
    /// Inferred relationships (cached)
    inferred: HashMap<String, Vec<(String, SemanticRelation)>>,
}

/// Simple inference rule
#[derive(Debug, Clone)]
pub struct InferenceRule {
    name: String,
    condition: fn(&ConceptGraph, &str, &str) -> bool,
    conclusion: fn(&str, &str) -> (String, String, SemanticRelation),
}

impl ConceptGraph {
    /// Create a new concept graph
    pub fn new() -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::ConceptGraph)
            .build_event()
            .expect("Failed to create concept graph");
            
        let mut rules = Vec::new();
        
        // Add basic inference rules
        rules.push(InferenceRule {
            name: "Transitive SubClassOf".to_string(),
            condition: |g, a, c| {
                // If A subClassOf B and B subClassOf C
                g.graph.node_ids().iter().any(|b| {
                    g.has_relation(a, b, SemanticRelation::SubClassOf) &&
                    g.has_relation(b, c, SemanticRelation::SubClassOf)
                })
            },
            conclusion: |a, c| (a.to_string(), c.to_string(), SemanticRelation::SubClassOf),
        });
        
        rules.push(InferenceRule {
            name: "Instance inheritance".to_string(),
            condition: |g, inst, class| {
                // If inst instanceOf A and A subClassOf class
                g.graph.node_ids().iter().any(|a| {
                    g.has_relation(inst, a, SemanticRelation::InstanceOf) &&
                    g.has_relation(a, class, SemanticRelation::SubClassOf)
                })
            },
            conclusion: |inst, class| (inst.to_string(), class.to_string(), SemanticRelation::InstanceOf),
        });
            
        Self {
            graph,
            rules,
            inferred: HashMap::new(),
        }
    }
    
    /// Create with event handler
    pub fn with_handler(handler: Arc<dyn EventHandler>) -> Self {
        let graph = GraphBuilder::new()
            .graph_type(GraphType::ConceptGraph)
            .add_handler(handler)
            .build_event()
            .expect("Failed to create concept graph");
            
        let mut instance = Self {
            graph,
            rules: Vec::new(),
            inferred: HashMap::new(),
        };
        
        // Add the same inference rules
        instance.rules = Self::new().rules;
        instance
    }
    
    /// Add a concept
    pub fn add_concept(&mut self, id: &str, name: &str, properties: serde_json::Value) -> Result<String> {
        let mut concept = ConceptNode::new(id, name, ConceptType::Class);
        
        // Add properties from JSON
        if let Some(obj) = properties.as_object() {
            for (key, value) in obj {
                concept.add_property(key, value.clone());
            }
        }
        
        self.graph.add_node(concept)
    }
    
    /// Add a semantic relationship
    pub fn add_relation(&mut self, from: &str, to: &str, relation: SemanticRelation) -> Result<String> {
        // Clear inferred cache when new relations are added
        self.inferred.clear();
        let edge = ConceptEdge::new(from, to, relation);
        self.graph.add_edge(edge)
    }
    
    /// Check if a relation exists (including inferred)
    pub fn has_relation(&self, from: &str, to: &str, relation: SemanticRelation) -> bool {
        // Check explicit relations
        let edges = self.graph.edges_between(from, to);
        if edges.iter().any(|e| e.relation() == relation) {
            return true;
        }
        
        // Check inferred relations
        if let Some(inferred) = self.inferred.get(from) {
            if inferred.iter().any(|(target, rel)| target == to && *rel == relation) {
                return true;
            }
        }
        
        false
    }
    
    /// Run inference to derive new relationships
    pub fn run_inference(&mut self) {
        let mut new_inferences = Vec::new();
        
        // Apply each rule
        for rule in &self.rules {
            for a in self.graph.node_ids() {
                for c in self.graph.node_ids() {
                    if a != c && (rule.condition)(self, &a, &c) {
                        let (from, to, rel) = (rule.conclusion)(&a, &c);
                        
                        // Don't add if already exists
                        if !self.has_relation(&from, &to, rel) {
                            new_inferences.push((from, to, rel));
                        }
                    }
                }
            }
        }
        
        // Add new inferences to cache
        for (from, to, rel) in new_inferences {
            self.inferred
                .entry(from)
                .or_insert_with(Vec::new)
                .push((to, rel));
        }
    }
    
    /// Get all superclasses of a concept (including inferred)
    pub fn get_superclasses(&self, concept: &str) -> HashSet<String> {
        let mut superclasses = HashSet::new();
        let mut queue = vec![concept.to_string()];
        
        while let Some(current) = queue.pop() {
            // Direct superclasses
            for target in self.graph.neighbors(&current).unwrap_or_default() {
                let edges = self.graph.edges_between(&current, &target);
                for edge in edges {
                    if edge.relation() == SemanticRelation::SubClassOf {
                        if superclasses.insert(target.clone()) {
                            queue.push(target.clone());
                        }
                    }
                }
            }
            
            // Inferred superclasses
            if let Some(inferred) = self.inferred.get(&current) {
                for (target, rel) in inferred {
                    if *rel == SemanticRelation::SubClassOf {
                        if superclasses.insert(target.clone()) {
                            queue.push(target.clone());
                        }
                    }
                }
            }
        }
        
        superclasses
    }
    
    /// Get all instances of a class (including subclasses)
    pub fn get_instances(&self, class: &str) -> HashSet<String> {
        let mut instances = HashSet::new();
        
        // Direct instances
        for node_id in self.graph.node_ids() {
            if self.has_relation(&node_id, class, SemanticRelation::InstanceOf) {
                instances.insert(node_id);
            }
        }
        
        // Instances of subclasses
        for node_id in self.graph.node_ids() {
            if self.has_relation(&node_id, class, SemanticRelation::SubClassOf) {
                let subclass_instances = self.get_instances(&node_id);
                instances.extend(subclass_instances);
            }
        }
        
        instances
    }
    
    /// Check consistency (simple version)
    pub fn check_consistency(&self) -> Vec<String> {
        let mut violations = Vec::new();
        
        // Check for circular subclass relationships
        for node_id in self.graph.node_ids() {
            let superclasses = self.get_superclasses(&node_id);
            if superclasses.contains(&node_id) {
                violations.push(format!("Circular subclass relationship: {}", node_id));
            }
        }
        
        // Check disjointness violations
        for node_id in self.graph.node_ids() {
            let mut classes = HashSet::new();
            
            // Get all classes this is an instance of
            for target in self.graph.node_ids() {
                if self.has_relation(&node_id, &target, SemanticRelation::InstanceOf) {
                    classes.insert(target);
                }
            }
            
            // Check if any are disjoint
            for class1 in &classes {
                for class2 in &classes {
                    if class1 != class2 && self.has_relation(class1, class2, SemanticRelation::DisjointWith) {
                        violations.push(format!(
                            "{} is instance of disjoint classes {} and {}",
                            node_id, class1, class2
                        ));
                    }
                }
            }
        }
        
        violations
    }
    
    /// Add a rule
    pub fn add_rule(&mut self, rule: InferenceRule) {
        self.rules.push(rule);
    }
    
    /// Get all rules
    pub fn rules(&self) -> &[InferenceRule] {
        &self.rules
    }
    
    /// Apply inference rules and return count of new inferences
    pub fn apply_inference(&mut self) -> usize {
        // For now, just return 0 as we haven't implemented full inference
        // In a real implementation, this would apply rules and generate new relations
        0
    }
    
    /// Get relations for a concept
    pub fn get_relations(&self, concept_id: &str) -> Vec<(String, SemanticRelation)> {
        let mut relations = Vec::new();
        
        // Get direct relations
        if let Ok(edges) = self.graph.edges_from(concept_id) {
            for edge_id in edges {
                if let Some(edge) = self.graph.get_edge(&edge_id) {
                    relations.push((edge.target.clone(), edge.relation));
                }
            }
        }
        
        relations
    }
    
    /// Get inferred relations for a concept
    pub fn get_inferred(&self, concept_id: &str) -> Vec<(String, SemanticRelation)> {
        self.inferred.get(concept_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all concepts
    pub fn get_all_concepts(&self) -> Vec<&ConceptNode> {
        self.graph.node_ids()
            .iter()
            .filter_map(|id| self.graph.get_node(id))
            .collect()
    }
    
    /// Get the underlying graph
    pub fn graph(&self) -> &EventGraph<ConceptNode, ConceptEdge> {
        &self.graph
    }
    
    /// Get mutable access to the underlying graph
    pub fn graph_mut(&mut self) -> &mut EventGraph<ConceptNode, ConceptEdge> {
        &mut self.graph
    }
}

impl Default for ConceptGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_concept_graph_creation() {
        let graph = ConceptGraph::new();
        assert_eq!(graph.graph().node_count(), 0);
        assert_eq!(graph.graph().graph_type(), GraphType::ConceptGraph);
    }
    
    #[test]
    fn test_class_hierarchy() {
        let mut graph = ConceptGraph::new();
        
        // Create class hierarchy
        let animal = ConceptNode::new("Animal", "Animal", ConceptType::Class);
        let mammal = ConceptNode::new("Mammal", "Mammal", ConceptType::Class);
        let dog = ConceptNode::new("Dog", "Dog", ConceptType::Class);
        
        graph.add_concept(animal).unwrap();
        graph.add_concept(mammal).unwrap();
        graph.add_concept(dog).unwrap();
        
        // Add relationships
        graph.add_relation(ConceptEdge::new("Mammal", "Animal", SemanticRelation::SubClassOf)).unwrap();
        graph.add_relation(ConceptEdge::new("Dog", "Mammal", SemanticRelation::SubClassOf)).unwrap();
        
        // Test direct relationships
        assert!(graph.has_relation("Dog", "Mammal", SemanticRelation::SubClassOf));
        
        // Run inference
        graph.run_inference();
        
        // Test inferred relationships
        assert!(graph.has_relation("Dog", "Animal", SemanticRelation::SubClassOf));
        
        // Test superclasses
        let dog_superclasses = graph.get_superclasses("Dog");
        assert!(dog_superclasses.contains("Mammal"));
        assert!(dog_superclasses.contains("Animal"));
    }
    
    #[test]
    fn test_instance_inference() {
        let mut graph = ConceptGraph::new();
        
        // Create hierarchy
        graph.add_concept(ConceptNode::new("Animal", "Animal", ConceptType::Class)).unwrap();
        graph.add_concept(ConceptNode::new("Dog", "Dog", ConceptType::Class)).unwrap();
        graph.add_concept(ConceptNode::new("fido", "Fido", ConceptType::Instance)).unwrap();
        
        // Add relationships
        graph.add_relation(ConceptEdge::new("Dog", "Animal", SemanticRelation::SubClassOf)).unwrap();
        graph.add_relation(ConceptEdge::new("fido", "Dog", SemanticRelation::InstanceOf)).unwrap();
        
        // Run inference
        graph.run_inference();
        
        // Check inferred instance relationship
        assert!(graph.has_relation("fido", "Animal", SemanticRelation::InstanceOf));
        
        // Check get_instances
        let animal_instances = graph.get_instances("Animal");
        assert!(animal_instances.contains("fido"));
    }
    
    #[test]
    fn test_consistency_checking() {
        let mut graph = ConceptGraph::new();
        
        // Create disjoint classes
        graph.add_concept(ConceptNode::new("Plant", "Plant", ConceptType::Class)).unwrap();
        graph.add_concept(ConceptNode::new("Animal", "Animal", ConceptType::Class)).unwrap();
        graph.add_concept(ConceptNode::new("weird", "Weird Thing", ConceptType::Instance)).unwrap();
        
        // Make them disjoint
        graph.add_relation(ConceptEdge::new("Plant", "Animal", SemanticRelation::DisjointWith)).unwrap();
        
        // Add conflicting instance relationships
        graph.add_relation(ConceptEdge::new("weird", "Plant", SemanticRelation::InstanceOf)).unwrap();
        graph.add_relation(ConceptEdge::new("weird", "Animal", SemanticRelation::InstanceOf)).unwrap();
        
        // Check consistency
        let violations = graph.check_consistency();
        assert!(!violations.is_empty());
        assert!(violations[0].contains("disjoint classes"));
    }
}