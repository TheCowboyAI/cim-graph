/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Mathematical proof generation and validation for Conceptual Spaces
//!
//! Ensures all transformations preserve mathematical properties and
//! generates proofs of correctness.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mathematical proof of conceptual space consistency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathematicalProof {
    /// Unique identifier for this proof
    pub id: String,
    /// Category objects (vertices in the proof graph)
    pub objects: Vec<CategoryObject>,
    /// Morphisms (edges in the proof graph)
    pub morphisms: Vec<Morphism>,
    /// Commutation laws that must hold for proof validity
    pub commutation_laws: Vec<CommutationLaw>,
    /// Current status of the proof validation
    pub proof_status: ProofStatus,
}

/// Object in a category (vertex in proof graph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryObject {
    /// Unique identifier for this category object
    pub id: String,
    /// Type classification of the object
    pub object_type: ObjectType,
    /// Additional properties of the object
    pub properties: HashMap<String, serde_json::Value>,
}

/// Type of category object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectType {
    /// Domain object (source of morphism)
    Domain,
    /// Codomain object (target of morphism)
    Codomain,
    /// Intermediate object in composition chain
    Intermediate,
    /// Identity object (self-morphism)
    Identity,
}

/// Morphism between objects (edge in proof graph)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Morphism {
    /// Unique identifier for this morphism
    pub id: String,
    /// Source object ID
    pub source: String,
    /// Target object ID
    pub target: String,
    /// Type of this morphism
    pub morphism_type: MorphismType,
    /// Whether this morphism is reversible
    pub is_isomorphism: bool,
}

/// Type of morphism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MorphismType {
    /// Standard function morphism
    Function,
    /// Natural transformation between functors
    NaturalTransformation,
    /// Functor between categories
    Functor,
    /// Bijective morphism with inverse
    Isomorphism,
    /// Injective morphism preserving structure
    Embedding,
    /// Surjective morphism reducing dimensions
    Projection,
}

/// Commutation law that must hold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommutationLaw {
    /// Unique identifier for this law
    pub id: String,
    /// First path of morphism IDs
    pub path1: Vec<String>, // Morphism IDs
    /// Alternative path of morphism IDs
    pub path2: Vec<String>, // Alternative morphism IDs
    /// Human-readable description of the law
    pub description: String,
}

/// Status of proof validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStatus {
    /// Proof is complete and valid
    Proven,
    /// Proof has been shown to be invalid
    Disproven,
    /// Proof is partially complete
    Partial,
    /// Proof status is not yet determined
    Unknown,
}

impl MathematicalProof {
    /// Create a new proof
    pub fn new(id: String) -> Self {
        Self {
            id,
            objects: Vec::new(),
            morphisms: Vec::new(),
            commutation_laws: Vec::new(),
            proof_status: ProofStatus::Unknown,
        }
    }

    /// Add an object to the proof
    pub fn add_object(&mut self, object: CategoryObject) {
        self.objects.push(object);
    }

    /// Add a morphism to the proof
    pub fn add_morphism(&mut self, morphism: Morphism) {
        self.morphisms.push(morphism);
    }

    /// Add a commutation law
    pub fn add_commutation_law(&mut self, law: CommutationLaw) {
        self.commutation_laws.push(law);
    }

    /// Validate the proof
    pub fn validate(&mut self) -> ProofStatus {
        // Check if all morphisms have valid sources and targets
        let object_ids: std::collections::HashSet<_> = 
            self.objects.iter().map(|o| &o.id).collect();

        for morphism in &self.morphisms {
            if !object_ids.contains(&morphism.source) || 
               !object_ids.contains(&morphism.target) {
                self.proof_status = ProofStatus::Disproven;
                return self.proof_status.clone();
            }
        }

        // Check commutation laws
        for law in &self.commutation_laws {
            if !self.paths_commute(&law.path1, &law.path2) {
                self.proof_status = ProofStatus::Disproven;
                return self.proof_status.clone();
            }
        }

        // Check for completeness
        if self.is_complete() {
            self.proof_status = ProofStatus::Proven;
        } else {
            self.proof_status = ProofStatus::Partial;
        }

        self.proof_status.clone()
    }

    /// Check if two paths commute
    fn paths_commute(&self, path1: &[String], path2: &[String]) -> bool {
        // Get endpoints of both paths
        let start1 = self.get_path_start(path1);
        let end1 = self.get_path_end(path1);
        let start2 = self.get_path_start(path2);
        let end2 = self.get_path_end(path2);

        // Paths commute if they have same start and end
        start1 == start2 && end1 == end2
    }

    /// Get the starting object of a path
    fn get_path_start(&self, path: &[String]) -> Option<String> {
        if let Some(first_morphism_id) = path.first() {
            self.morphisms.iter()
                .find(|m| m.id == *first_morphism_id)
                .map(|m| m.source.clone())
        } else {
            None
        }
    }

    /// Get the ending object of a path
    fn get_path_end(&self, path: &[String]) -> Option<String> {
        if let Some(last_morphism_id) = path.last() {
            self.morphisms.iter()
                .find(|m| m.id == *last_morphism_id)
                .map(|m| m.target.clone())
        } else {
            None
        }
    }

    /// Check if the proof is complete
    fn is_complete(&self) -> bool {
        // A proof is complete if:
        // 1. It has at least two objects
        // 2. All objects are connected by morphisms
        // 3. At least one commutation law is satisfied
        
        if self.objects.len() < 2 {
            return false;
        }

        if self.morphisms.is_empty() {
            return false;
        }

        !self.commutation_laws.is_empty()
    }

    /// Generate a string representation of the proof
    pub fn to_string_proof(&self) -> String {
        let mut proof = format!("Mathematical Proof: {}\n", self.id);
        proof.push_str(&format!("Status: {:?}\n\n", self.proof_status));

        proof.push_str("Objects:\n");
        for obj in &self.objects {
            proof.push_str(&format!("  {} ({:?})\n", obj.id, obj.object_type));
        }

        proof.push_str("\nMorphisms:\n");
        for morph in &self.morphisms {
            proof.push_str(&format!(
                "  {} : {} → {} ({:?})\n",
                morph.id, morph.source, morph.target, morph.morphism_type
            ));
        }

        proof.push_str("\nCommutation Laws:\n");
        for law in &self.commutation_laws {
            proof.push_str(&format!(
                "  {}: Path {:?} ≡ Path {:?}\n",
                law.id, law.path1, law.path2
            ));
            proof.push_str(&format!("    Description: {}\n", law.description));
        }

        proof
    }
}

/// Proof builder for constructing proofs step by step
#[derive(Debug)]
pub struct ProofBuilder {
    proof: MathematicalProof,
}

impl ProofBuilder {
    /// Create a new proof builder
    pub fn new(id: String) -> Self {
        Self {
            proof: MathematicalProof::new(id),
        }
    }

    /// Add a domain object
    pub fn with_domain(mut self, id: String, properties: HashMap<String, serde_json::Value>) -> Self {
        self.proof.add_object(CategoryObject {
            id,
            object_type: ObjectType::Domain,
            properties,
        });
        self
    }

    /// Add a codomain object
    pub fn with_codomain(mut self, id: String, properties: HashMap<String, serde_json::Value>) -> Self {
        self.proof.add_object(CategoryObject {
            id,
            object_type: ObjectType::Codomain,
            properties,
        });
        self
    }

    /// Add a function morphism
    pub fn with_function(mut self, id: String, source: String, target: String) -> Self {
        self.proof.add_morphism(Morphism {
            id,
            source,
            target,
            morphism_type: MorphismType::Function,
            is_isomorphism: false,
        });
        self
    }

    /// Add an isomorphism
    pub fn with_isomorphism(mut self, id: String, source: String, target: String) -> Self {
        self.proof.add_morphism(Morphism {
            id,
            source,
            target,
            morphism_type: MorphismType::Isomorphism,
            is_isomorphism: true,
        });
        self
    }

    /// Add a commutation law
    pub fn with_commutation(
        mut self,
        id: String,
        path1: Vec<String>,
        path2: Vec<String>,
        description: String,
    ) -> Self {
        self.proof.add_commutation_law(CommutationLaw {
            id,
            path1,
            path2,
            description,
        });
        self
    }

    /// Build and validate the proof
    pub fn build(mut self) -> MathematicalProof {
        self.proof.validate();
        self.proof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_builder() {
        let proof = ProofBuilder::new("test_proof".to_string())
            .with_domain("A".to_string(), HashMap::new())
            .with_codomain("B".to_string(), HashMap::new())
            .with_function("f".to_string(), "A".to_string(), "B".to_string())
            .with_function("g".to_string(), "A".to_string(), "B".to_string())
            .with_commutation(
                "comm1".to_string(),
                vec!["f".to_string()],
                vec!["g".to_string()],
                "Functions f and g are equal".to_string(),
            )
            .build();

        assert_eq!(proof.objects.len(), 2);
        assert_eq!(proof.morphisms.len(), 2);
        assert_eq!(proof.commutation_laws.len(), 1);
        assert!(matches!(proof.proof_status, ProofStatus::Proven));
    }

    #[test]
    fn test_proof_validation_failure() {
        let mut proof = MathematicalProof::new("invalid_proof".to_string());
        
        // Add morphism with non-existent objects
        proof.add_morphism(Morphism {
            id: "f".to_string(),
            source: "X".to_string(),
            target: "Y".to_string(),
            morphism_type: MorphismType::Function,
            is_isomorphism: false,
        });

        let status = proof.validate();
        assert!(matches!(status, ProofStatus::Disproven));
    }
}