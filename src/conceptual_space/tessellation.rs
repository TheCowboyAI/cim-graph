/*
 * Copyright (c) 2025 - Cowboy AI, LLC.
 */

//! Spherical Voronoi Tessellation for Conceptual Spaces
//!
//! Implements the mathematical algorithms for computing Voronoi cells
//! on the surface of a unit sphere.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{Point3, ConceptNode};

/// Voronoi tessellation computed from concept node positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalVoronoiTessellation {
    /// Unique identifier for this tessellation
    pub tessellation_id: String,
    /// Voronoi cells around each concept
    pub cells: Vec<VoronoiCell>,
    /// Edges of the dual (Delaunay) graph
    pub dual_graph_edges: Vec<DualEdge>,
    /// Total surface area of the sphere
    pub total_surface_area: f64,
}

/// Individual Voronoi cell around a concept
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoronoiCell {
    /// Unique identifier for this cell
    pub cell_id: String,
    /// ID of the concept at the center
    pub concept_node_id: String,
    /// Position of the generating concept
    pub generator_position: Point3<f64>,
    /// Vertices defining the cell boundary
    pub vertices: Vec<Point3<f64>>,
    /// Edges of the cell
    pub edges: Vec<SphericalEdge>,
    /// Surface area of the cell
    pub area: f64,
    /// Influence strength of the concept
    pub concept_influence_strength: f64,
}

/// Edge in dual graph connecting adjacent Voronoi cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualEdge {
    /// Unique identifier for this edge
    pub edge_id: String,
    /// First cell ID
    pub cell1_id: String,
    /// Second cell ID
    pub cell2_id: String,
    /// The shared boundary between cells
    pub shared_boundary: SphericalEdge,
    /// Strength of relationship between concepts
    pub concept_relationship_strength: f64,
}

/// Edge on spherical surface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalEdge {
    /// Starting point of the edge
    pub start: Point3<f64>,
    /// Ending point of the edge
    pub end: Point3<f64>,
    /// Length of the arc on the sphere
    pub arc_length: f64,
    /// Whether this is a geodesic (shortest path)
    pub is_geodesic: bool,
}

impl SphericalVoronoiTessellation {
    /// Compute Voronoi tessellation from concept positions
    pub fn compute(
        space_id: &str,
        positions: &[Point3<f64>],
        concepts: &HashMap<String, ConceptNode>,
    ) -> Result<Self> {
        let mut cells = Vec::new();
        let mut dual_edges = Vec::new();

        // Create Voronoi cell for each concept
        for (i, position) in positions.iter().enumerate() {
            let concept_id = concepts.values()
                .nth(i)
                .map(|c| c.id.clone())
                .unwrap_or_else(|| format!("concept_{}", i));

            let cell = VoronoiCell::compute_for_position(
                &format!("cell_{}_{}", space_id, i),
                &concept_id,
                position,
                positions,
                concepts.get(&concept_id),
            )?;
            cells.push(cell);
        }

        // Compute dual graph edges between adjacent cells
        for i in 0..cells.len() {
            for j in (i + 1)..cells.len() {
                if VoronoiCell::are_adjacent(&cells[i], &cells[j], positions) {
                    let edge = DualEdge {
                        edge_id: format!("dual_{}_{}", i, j),
                        cell1_id: cells[i].cell_id.clone(),
                        cell2_id: cells[j].cell_id.clone(),
                        shared_boundary: SphericalEdge::compute_geodesic(
                            &cells[i].generator_position,
                            &cells[j].generator_position,
                        )?,
                        concept_relationship_strength: 1.0 / 
                            SphericalEdge::compute_geodesic_distance(
                                &cells[i].generator_position,
                                &cells[j].generator_position,
                            ),
                    };
                    dual_edges.push(edge);
                }
            }
        }

        Ok(Self {
            tessellation_id: format!("tessellation_{}", uuid::Uuid::new_v4()),
            cells,
            dual_graph_edges: dual_edges,
            total_surface_area: 4.0 * std::f64::consts::PI, // Unit sphere surface area
        })
    }

    /// Find which cell contains a given point on the sphere
    pub fn find_containing_cell(&self, point: &Point3<f64>) -> Option<&VoronoiCell> {
        // Normalize point to sphere surface
        let normalized = Self::normalize_to_sphere(point);
        
        // Find cell whose generator is closest
        self.cells.iter()
            .min_by(|a, b| {
                let dist_a = SphericalEdge::compute_geodesic_distance(
                    &a.generator_position,
                    &normalized,
                );
                let dist_b = SphericalEdge::compute_geodesic_distance(
                    &b.generator_position,
                    &normalized,
                );
                dist_a.partial_cmp(&dist_b).unwrap()
            })
    }

    /// Normalize a point to lie on the unit sphere
    fn normalize_to_sphere(point: &Point3<f64>) -> Point3<f64> {
        let magnitude = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        Point3::new(
            point.x / magnitude,
            point.y / magnitude,
            point.z / magnitude,
        )
    }
}

impl VoronoiCell {
    /// Compute a single Voronoi cell
    fn compute_for_position(
        cell_id: &str,
        concept_id: &str,
        position: &Point3<f64>,
        all_positions: &[Point3<f64>],
        concept: Option<&ConceptNode>,
    ) -> Result<Self> {
        let vertices = Self::compute_vertices(position, all_positions)?;
        let edges = Self::compute_edges(&vertices)?;
        let area = Self::compute_area(position, all_positions)?;
        let influence = Self::compute_influence(concept);

        Ok(Self {
            cell_id: cell_id.to_string(),
            concept_node_id: concept_id.to_string(),
            generator_position: *position,
            vertices,
            edges,
            area,
            concept_influence_strength: influence,
        })
    }

    /// Compute vertices of the Voronoi cell
    fn compute_vertices(
        center: &Point3<f64>,
        all_positions: &[Point3<f64>],
    ) -> Result<Vec<Point3<f64>>> {
        let mut vertices = Vec::new();

        // For each pair of neighboring generators, compute the perpendicular bisector
        for other in all_positions {
            if other != center {
                // Midpoint between the two generators
                let midpoint = Point3::new(
                    (center.x + other.x) * 0.5,
                    (center.y + other.y) * 0.5,
                    (center.z + other.z) * 0.5,
                );

                // Project midpoint to sphere surface
                let magnitude = (midpoint.x * midpoint.x + 
                               midpoint.y * midpoint.y + 
                               midpoint.z * midpoint.z).sqrt();
                
                if magnitude > 0.0 {
                    let normalized = Point3::new(
                        midpoint.x / magnitude,
                        midpoint.y / magnitude,
                        midpoint.z / magnitude,
                    );

                    // Use vector math to ensure we only keep vertices that are not collinear
                    // with the center→other great-circle direction (robustness against duplicate points).
                    let c = nalgebra::Vector3::new(center.x, center.y, center.z);
                    let o = nalgebra::Vector3::new(other.x, other.y, other.z);
                    let normal_na = c.cross(&o);
                    let normal: Vector3<f64> = normal_na.into();
                    if normal.magnitude() > 1e-10 {
                        // Also ensure this vertex is locally distinct from existing ones
                        if !vertices.iter().any(|v: &Point3<f64>| {
                            let dv = Vector3::new(v.x - normalized.x, v.y - normalized.y, v.z - normalized.z);
                            dv.magnitude() < 1e-8
                        }) {
                            vertices.push(normalized);
                        }
                    }
                }
            }
        }

        Ok(vertices)
    }

    /// Compute edges of the Voronoi cell
    fn compute_edges(vertices: &[Point3<f64>]) -> Result<Vec<SphericalEdge>> {
        let mut edges = Vec::new();

        // Connect adjacent vertices with geodesic edges
        for i in 0..vertices.len() {
            let j = (i + 1) % vertices.len();
            edges.push(SphericalEdge::compute_geodesic(&vertices[i], &vertices[j])?);
        }

        Ok(edges)
    }

    /// Compute area of the Voronoi cell on the sphere
    fn compute_area(
        _center: &Point3<f64>,
        all_positions: &[Point3<f64>],
    ) -> Result<f64> {
        // Approximate local Voronoi area by spherical cap with radius r = 0.5 * d_min,
        // where d_min is the geodesic distance to the nearest neighbor.
        let mut d_min = std::f64::consts::PI; // upper bound (antipode)
        for other in all_positions {
            if other != center {
                let d = SphericalEdge::compute_geodesic_distance(center, other);
                if d < d_min { d_min = d; }
            }
        }

        // If isolated or single point, distribute evenly
        if !d_min.is_finite() || all_positions.len() <= 1 {
            let total_area = 4.0 * std::f64::consts::PI;
            return Ok(total_area / all_positions.len().max(1) as f64);
        }

        let r = 0.5 * d_min;
        // Spherical cap area A = 2π(1 - cos r)
        let area = 2.0 * std::f64::consts::PI * (1.0 - r.cos());
        Ok(area)
    }

    /// Compute influence strength based on concept properties
    fn compute_influence(concept: Option<&ConceptNode>) -> f64 {
        match concept {
            Some(c) => {
                // Base influence on number of edges and properties
                let edge_count = c.edges.len() as f64;
                let property_count = c.properties.len() as f64;
                (edge_count + property_count + 1.0).ln() // Logarithmic scaling
            }
            None => 1.0,
        }
    }

    /// Check if two cells are adjacent
    fn are_adjacent(cell1: &VoronoiCell, cell2: &VoronoiCell, _positions: &[Point3<f64>]) -> bool {
        // Simplified: cells are adjacent if their generators are close enough
        let distance = SphericalEdge::compute_geodesic_distance(
            &cell1.generator_position,
            &cell2.generator_position,
        );
        distance < std::f64::consts::PI / 2.0 // Less than 90 degrees apart
    }
}

impl SphericalEdge {
    /// Compute geodesic edge between two points on sphere
    pub fn compute_geodesic(start: &Point3<f64>, end: &Point3<f64>) -> Result<Self> {
        let arc_length = Self::compute_geodesic_distance(start, end);
        
        Ok(Self {
            start: *start,
            end: *end,
            arc_length,
            is_geodesic: true,
        })
    }

    /// Compute geodesic distance on unit sphere
    pub fn compute_geodesic_distance(p1: &Point3<f64>, p2: &Point3<f64>) -> f64 {
        // Using spherical law of cosines
        let dot_product = p1.x * p2.x + p1.y * p2.y + p1.z * p2.z;
        // Clamp to avoid numerical errors with acos
        let clamped = dot_product.max(-1.0).min(1.0);
        clamped.acos()
    }

    /// Interpolate point along geodesic
    pub fn interpolate(&self, t: f64) -> Point3<f64> {
        // Spherical linear interpolation (slerp)
        let omega = self.arc_length;
        
        if omega < 1e-10 {
            // Points are too close, use linear interpolation
            return Point3::new(
                self.start.x * (1.0 - t) + self.end.x * t,
                self.start.y * (1.0 - t) + self.end.y * t,
                self.start.z * (1.0 - t) + self.end.z * t,
            );
        }

        let sin_omega = omega.sin();
        let a = ((1.0 - t) * omega).sin() / sin_omega;
        let b = (t * omega).sin() / sin_omega;

        Point3::new(
            self.start.x * a + self.end.x * b,
            self.start.y * a + self.end.y * b,
            self.start.z * a + self.end.z * b,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geodesic_distance() {
        // North and south poles
        let north = Point3::new(0.0, 0.0, 1.0);
        let south = Point3::new(0.0, 0.0, -1.0);
        let distance = SphericalEdge::compute_geodesic_distance(&north, &south);
        assert!((distance - std::f64::consts::PI).abs() < 1e-10);

        // Same point
        let distance = SphericalEdge::compute_geodesic_distance(&north, &north);
        assert!(distance < 1e-10);

        // 90 degree separation
        let equator = Point3::new(1.0, 0.0, 0.0);
        let distance = SphericalEdge::compute_geodesic_distance(&north, &equator);
        assert!((distance - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_spherical_interpolation() {
        let start = Point3::new(1.0, 0.0, 0.0);
        let end = Point3::new(0.0, 1.0, 0.0);
        let edge = SphericalEdge::compute_geodesic(&start, &end).unwrap();

        // Check midpoint
        let midpoint = edge.interpolate(0.5);
        let expected = Point3::new(
            (2.0_f64).sqrt() / 2.0,
            (2.0_f64).sqrt() / 2.0,
            0.0
        );
        assert!((midpoint.x - expected.x).abs() < 1e-10);
        assert!((midpoint.y - expected.y).abs() < 1e-10);
        assert!(midpoint.z.abs() < 1e-10);

        // Check endpoints
        let start_check = edge.interpolate(0.0);
        assert!((start_check.x - start.x).abs() < 1e-10);
        
        let end_check = edge.interpolate(1.0);
        assert!((end_check.x - end.x).abs() < 1e-10);
    }
}
