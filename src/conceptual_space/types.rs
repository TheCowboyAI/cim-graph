/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Core types for Conceptual Spaces
//!
//! Provides serializable mathematical types and domain structures for
//! representing concepts in topological spaces.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Custom serializable 3D point for conceptual spaces
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Point3<T> {
    /// X coordinate
    pub x: T,
    /// Y coordinate
    pub y: T,
    /// Z coordinate
    pub z: T,
}

impl<T> Point3<T> {
    /// Creates a new Point3 with the given coordinates
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy> Point3<T> {
    /// Converts to nalgebra Vector3 for computations
    pub fn coords(&self) -> nalgebra::Vector3<T> {
        nalgebra::Vector3::new(self.x, self.y, self.z)
    }
}

impl<T: Copy + std::ops::Sub<Output = T>> std::ops::Sub for Point3<T> {
    type Output = Vector3<T>;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl From<nalgebra::Point3<f64>> for Point3<f64> {
    fn from(point: nalgebra::Point3<f64>) -> Self {
        Point3::new(point.x, point.y, point.z)
    }
}

impl From<Point3<f64>> for nalgebra::Point3<f64> {
    fn from(point: Point3<f64>) -> Self {
        nalgebra::Point3::new(point.x, point.y, point.z)
    }
}

/// Custom serializable 3D vector for conceptual spaces
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vector3<T> {
    /// X component of the vector
    pub x: T,
    /// Y component of the vector
    pub y: T,
    /// Z component of the vector
    pub z: T,
}

impl<T> Vector3<T> {
    /// Creates a new Vector3 with the given components
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + num_traits::Float> Vector3<T> {
    /// Computes the magnitude (length) of the vector
    pub fn magnitude(&self) -> T {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

impl From<nalgebra::Vector3<f64>> for Vector3<f64> {
    fn from(vec: nalgebra::Vector3<f64>) -> Self {
        Vector3::new(vec.x, vec.y, vec.z)
    }
}

impl From<Vector3<f64>> for nalgebra::Vector3<f64> {
    fn from(vec: Vector3<f64>) -> Self {
        nalgebra::Vector3::new(vec.x, vec.y, vec.z)
    }
}

/// Custom serializable unit vector for conceptual spaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitVector3<T> {
    /// Normalized vector components
    pub vector: Vector3<T>,
}

impl<T: Copy + num_traits::Float> UnitVector3<T> {
    /// Creates a new unit vector by normalizing the input vector
    pub fn new_normalize(vec: Vector3<T>) -> Self {
        let magnitude = (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt();
        Self {
            vector: Vector3::new(vec.x / magnitude, vec.y / magnitude, vec.z / magnitude),
        }
    }
}

/// Concept node represents semantic knowledge in the space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    /// Unique identifier for the concept
    pub id: String,
    /// Properties defining the concept's characteristics
    pub properties: HashMap<String, serde_json::Value>,
    /// References to connected edges
    pub edges: Vec<String>,
}

/// Concept edge represents semantic relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptEdge {
    /// Unique identifier for the edge
    pub id: String,
    /// Source concept node ID
    pub from_node: String,
    /// Target concept node ID
    pub to_node: String,
    /// Type of relationship this edge represents
    pub edge_type: String,
    /// Additional properties of the relationship
    pub properties: HashMap<String, serde_json::Value>,
}

/// Quality dimensions emerging from concept relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDimension {
    /// Unique identifier for this dimension
    pub dimension_id: String,
    /// Type of quality dimension
    pub dimension_type: QualityType,
    /// Concept from which this dimension originates
    pub origin_concept_id: String,
    /// Target concept for this dimension
    pub target_concept_id: String,
    /// Direction vector in conceptual space
    pub direction: UnitVector3<f64>,
    /// Magnitude or strength of this dimension
    pub magnitude: f64,
    /// Whether this dimension emerged from interactions
    pub is_emergent: bool,
    /// Stability measure (0.0 to 1.0)
    pub stability: f64,
}

/// Types of quality dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityType {
    /// Scalar dimension (temperature, brightness)
    Scalar { 
        /// Valid range for scalar values
        range: (f64, f64) 
    },
    /// Vector dimension (force, velocity)
    Vector { 
        /// Basis vectors defining the vector space
        basis: Vec<Vector3<f64>> 
    },
    /// Categorical dimension (color, shape)
    Categorical { 
        /// Available categories
        categories: Vec<String> 
    },
    /// Ordinal dimension with ordering
    Ordinal { 
        /// Ordered list of values
        ordering: Vec<String> 
    },
    /// Derived from other dimensions
    Derived {
        /// Mathematical formula for derivation
        formula: String,
        /// IDs of dimensions this depends on
        dependencies: Vec<String>,
    },
}

/// Emergent patterns from concept interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPattern {
    /// Unique identifier for this pattern
    pub pattern_id: String,
    /// Type classification of the pattern
    pub pattern_type: PatternType,
    /// Set of concepts involved in this pattern
    pub involved_concepts: HashSet<String>,
    /// Stability measure (0.0 to 1.0)
    pub stability: f64,
    /// Energy level of the pattern
    pub energy: f64,
    /// Unix timestamp when pattern formed
    pub formation_timestamp: u64,
    /// Events that triggered pattern formation
    pub triggering_events: Vec<String>,
}

/// Types of emergent patterns in conceptual space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Stable cluster of related concepts
    ConceptCluster { 
        /// Center point of the cluster
        centroid: Point3<f64>, 
        /// Radius of the cluster sphere
        radius: f64 
    },
    /// Oscillating relationship strength
    RelationshipOscillation { 
        /// Period of oscillation in time units
        period: f64, 
        /// Amplitude of the oscillation
        amplitude: f64 
    },
    /// Hierarchical structure
    ConceptHierarchy {
        /// Root concept at the top of hierarchy
        root_concept: String,
        /// Nested levels of the hierarchy
        levels: Vec<Vec<String>>,
    },
    /// Traveling activation wave
    ActivationWave {
        /// Wavelength of the activation wave
        wavelength: f64,
        /// Propagation speed
        speed: f64,
        /// Direction of wave propagation
        direction: Vector3<f64>,
    },
    /// Spiral arrangement of concepts
    ConceptSpiral { 
        /// Center point of the spiral
        center: Point3<f64>, 
        /// Pitch of the spiral (vertical spacing)
        pitch: f64 
    },
}

/// Validation result for consistency checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the validation passed
    pub is_valid: bool,
    /// List of validation violations found
    pub violations: Vec<String>,
    /// List of validation warnings
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Point3 tests
    // ========================================================================

    #[test]
    fn test_point3_creation() {
        let point = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn test_point3_coords() {
        let point = Point3::new(1.0, 2.0, 3.0);
        let coords = point.coords();
        assert_eq!(coords.x, 1.0);
        assert_eq!(coords.y, 2.0);
        assert_eq!(coords.z, 3.0);
    }

    #[test]
    fn test_point3_subtraction() {
        let p1 = Point3::new(5.0, 8.0, 3.0);
        let p2 = Point3::new(2.0, 3.0, 1.0);
        let diff = p1 - p2;
        assert_eq!(diff.x, 3.0);
        assert_eq!(diff.y, 5.0);
        assert_eq!(diff.z, 2.0);
    }

    #[test]
    fn test_point3_from_nalgebra() {
        let na_point = nalgebra::Point3::new(1.0_f64, 2.0, 3.0);
        let point: Point3<f64> = na_point.into();
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn test_point3_to_nalgebra() {
        let point = Point3::new(1.0_f64, 2.0, 3.0);
        let na_point: nalgebra::Point3<f64> = point.into();
        assert_eq!(na_point.x, 1.0);
        assert_eq!(na_point.y, 2.0);
        assert_eq!(na_point.z, 3.0);
    }

    #[test]
    fn test_point3_equality() {
        let p1 = Point3::new(1.0, 2.0, 3.0);
        let p2 = Point3::new(1.0, 2.0, 3.0);
        let p3 = Point3::new(1.0, 2.0, 4.0);
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_point3_clone() {
        let p1 = Point3::new(1.0, 2.0, 3.0);
        let p2 = p1.clone();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_point3_serialization() {
        let point = Point3::new(1.5, 2.5, 3.5);
        let json = serde_json::to_string(&point).unwrap();
        let deserialized: Point3<f64> = serde_json::from_str(&json).unwrap();
        assert_eq!(point, deserialized);
    }

    // ========================================================================
    // Vector3 tests
    // ========================================================================

    #[test]
    fn test_vector3_creation() {
        let vec = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }

    #[test]
    fn test_vector3_magnitude() {
        let vec = Vector3::new(3.0_f64, 4.0, 0.0);
        assert!((vec.magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector3_magnitude_unit() {
        let vec = Vector3::new(1.0_f64, 0.0, 0.0);
        assert!((vec.magnitude() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector3_magnitude_zero() {
        let vec = Vector3::new(0.0_f64, 0.0, 0.0);
        assert!(vec.magnitude() < 1e-10);
    }

    #[test]
    fn test_vector3_magnitude_3d() {
        // sqrt(1^2 + 2^2 + 2^2) = sqrt(9) = 3
        let vec = Vector3::new(1.0_f64, 2.0, 2.0);
        assert!((vec.magnitude() - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector3_from_nalgebra() {
        let na_vec = nalgebra::Vector3::new(1.0_f64, 2.0, 3.0);
        let vec: Vector3<f64> = na_vec.into();
        assert_eq!(vec.x, 1.0);
        assert_eq!(vec.y, 2.0);
        assert_eq!(vec.z, 3.0);
    }

    #[test]
    fn test_vector3_to_nalgebra() {
        let vec = Vector3::new(1.0_f64, 2.0, 3.0);
        let na_vec: nalgebra::Vector3<f64> = vec.into();
        assert_eq!(na_vec.x, 1.0);
        assert_eq!(na_vec.y, 2.0);
        assert_eq!(na_vec.z, 3.0);
    }

    #[test]
    fn test_vector3_serialization() {
        let vec = Vector3::new(1.5, 2.5, 3.5);
        let json = serde_json::to_string(&vec).unwrap();
        let deserialized: Vector3<f64> = serde_json::from_str(&json).unwrap();
        assert_eq!(vec, deserialized);
    }

    // ========================================================================
    // UnitVector3 tests
    // ========================================================================

    #[test]
    fn test_unit_vector3_normalize() {
        let vec = Vector3::new(3.0_f64, 4.0, 0.0);
        let unit = UnitVector3::new_normalize(vec);
        // Should be normalized to length 1
        let mag = unit.vector.magnitude();
        assert!((mag - 1.0).abs() < 1e-10);
        // Direction should be preserved
        assert!((unit.vector.x - 0.6).abs() < 1e-10);
        assert!((unit.vector.y - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_unit_vector3_already_unit() {
        let vec = Vector3::new(1.0_f64, 0.0, 0.0);
        let unit = UnitVector3::new_normalize(vec);
        assert!((unit.vector.x - 1.0).abs() < 1e-10);
        assert!(unit.vector.y.abs() < 1e-10);
        assert!(unit.vector.z.abs() < 1e-10);
    }

    #[test]
    fn test_unit_vector3_diagonal() {
        // (1,1,1) normalized should be (1/sqrt(3), 1/sqrt(3), 1/sqrt(3))
        let vec = Vector3::new(1.0_f64, 1.0, 1.0);
        let unit = UnitVector3::new_normalize(vec);
        let expected = 1.0 / 3.0_f64.sqrt();
        assert!((unit.vector.x - expected).abs() < 1e-10);
        assert!((unit.vector.y - expected).abs() < 1e-10);
        assert!((unit.vector.z - expected).abs() < 1e-10);
    }

    #[test]
    fn test_unit_vector3_serialization() {
        let vec = Vector3::new(1.0_f64, 0.0, 0.0);
        let unit = UnitVector3::new_normalize(vec);
        let json = serde_json::to_string(&unit).unwrap();
        let deserialized: UnitVector3<f64> = serde_json::from_str(&json).unwrap();
        assert!((unit.vector.x - deserialized.vector.x).abs() < 1e-10);
    }

    // ========================================================================
    // ConceptNode tests
    // ========================================================================

    #[test]
    fn test_concept_node_creation() {
        let mut properties = HashMap::new();
        properties.insert("type".to_string(), serde_json::json!("animal"));

        let node = ConceptNode {
            id: "cat".to_string(),
            properties,
            edges: vec!["edge1".to_string()],
        };

        assert_eq!(node.id, "cat");
        assert_eq!(node.edges.len(), 1);
        assert!(node.properties.contains_key("type"));
    }

    #[test]
    fn test_concept_node_serialization() {
        let node = ConceptNode {
            id: "test".to_string(),
            properties: HashMap::new(),
            edges: Vec::new(),
        };

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: ConceptNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node.id, deserialized.id);
    }

    // ========================================================================
    // ConceptEdge tests
    // ========================================================================

    #[test]
    fn test_concept_edge_creation() {
        let edge = ConceptEdge {
            id: "edge1".to_string(),
            from_node: "cat".to_string(),
            to_node: "dog".to_string(),
            edge_type: "similar".to_string(),
            properties: HashMap::new(),
        };

        assert_eq!(edge.id, "edge1");
        assert_eq!(edge.from_node, "cat");
        assert_eq!(edge.to_node, "dog");
        assert_eq!(edge.edge_type, "similar");
    }

    #[test]
    fn test_concept_edge_with_properties() {
        let mut properties = HashMap::new();
        properties.insert("strength".to_string(), serde_json::json!(0.85));

        let edge = ConceptEdge {
            id: "edge1".to_string(),
            from_node: "a".to_string(),
            to_node: "b".to_string(),
            edge_type: "related".to_string(),
            properties,
        };

        let strength = edge.properties.get("strength")
            .and_then(|v| v.as_f64())
            .unwrap();
        assert!((strength - 0.85).abs() < 1e-10);
    }

    // ========================================================================
    // QualityDimension tests
    // ========================================================================

    #[test]
    fn test_quality_dimension_scalar() {
        let dim = QualityDimension {
            dimension_id: "temp".to_string(),
            dimension_type: QualityType::Scalar { range: (0.0, 100.0) },
            origin_concept_id: "cold".to_string(),
            target_concept_id: "hot".to_string(),
            direction: UnitVector3::new_normalize(Vector3::new(1.0, 0.0, 0.0)),
            magnitude: 1.0,
            is_emergent: false,
            stability: 0.95,
        };

        assert_eq!(dim.dimension_id, "temp");
        assert!((dim.stability - 0.95).abs() < 1e-10);
    }

    #[test]
    fn test_quality_dimension_categorical() {
        let dim = QualityDimension {
            dimension_id: "color".to_string(),
            dimension_type: QualityType::Categorical {
                categories: vec!["red".to_string(), "green".to_string(), "blue".to_string()]
            },
            origin_concept_id: "object".to_string(),
            target_concept_id: "perception".to_string(),
            direction: UnitVector3::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
            magnitude: 1.0,
            is_emergent: true,
            stability: 0.8,
        };

        if let QualityType::Categorical { categories } = &dim.dimension_type {
            assert_eq!(categories.len(), 3);
        } else {
            panic!("Expected Categorical type");
        }
    }

    // ========================================================================
    // PatternType tests
    // ========================================================================

    #[test]
    fn test_pattern_type_cluster() {
        let pattern = EmergentPattern {
            pattern_id: "cluster1".to_string(),
            pattern_type: PatternType::ConceptCluster {
                centroid: Point3::new(0.0, 0.0, 1.0),
                radius: 0.5,
            },
            involved_concepts: ["a".to_string(), "b".to_string()].into_iter().collect(),
            stability: 0.9,
            energy: 1.5,
            formation_timestamp: 12345678,
            triggering_events: vec!["event1".to_string()],
        };

        assert_eq!(pattern.pattern_id, "cluster1");
        assert_eq!(pattern.involved_concepts.len(), 2);
    }

    #[test]
    fn test_pattern_type_hierarchy() {
        let pattern = EmergentPattern {
            pattern_id: "hier1".to_string(),
            pattern_type: PatternType::ConceptHierarchy {
                root_concept: "root".to_string(),
                levels: vec![
                    vec!["child1".to_string(), "child2".to_string()],
                    vec!["grandchild1".to_string()],
                ],
            },
            involved_concepts: HashSet::new(),
            stability: 0.75,
            energy: 2.0,
            formation_timestamp: 12345678,
            triggering_events: Vec::new(),
        };

        if let PatternType::ConceptHierarchy { root_concept, levels } = &pattern.pattern_type {
            assert_eq!(root_concept, "root");
            assert_eq!(levels.len(), 2);
        } else {
            panic!("Expected ConceptHierarchy type");
        }
    }

    // ========================================================================
    // ValidationResult tests
    // ========================================================================

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult {
            is_valid: true,
            violations: Vec::new(),
            warnings: Vec::new(),
        };
        assert!(result.is_valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_validation_result_invalid() {
        let result = ValidationResult {
            is_valid: false,
            violations: vec!["Missing required field".to_string()],
            warnings: vec!["Consider adding description".to_string()],
        };
        assert!(!result.is_valid);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            is_valid: true,
            violations: Vec::new(),
            warnings: vec!["test warning".to_string()],
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ValidationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.is_valid, deserialized.is_valid);
        assert_eq!(result.warnings.len(), deserialized.warnings.len());
    }
}