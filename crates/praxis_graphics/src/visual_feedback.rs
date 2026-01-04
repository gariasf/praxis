//! Visual feedback utilities for editor and debug visualization.
//!
//! This module provides helper functions for creating common visual feedback elements
//! such as grids, axis indicators, selection outlines, and bounding boxes.

use crate::line_renderer::LineBatch;
use praxis_math::{Mat4, Vec3};

/// Configuration for grid floor display.
#[derive(Clone, Debug)]
pub struct GridConfig {
    /// Size of the grid in world units (one side).
    pub size: f32,
    /// Number of divisions per axis.
    pub divisions: u32,
    /// Color for normal grid lines.
    pub line_color: Vec3,
    /// Color for axis-aligned lines (X and Z axes).
    pub axis_color: Vec3,
    /// Height (Y coordinate) of the grid.
    pub height: f32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            size: 20.0,
            divisions: 20,
            line_color: Vec3::new(0.3, 0.3, 0.3),
            axis_color: Vec3::new(0.5, 0.5, 0.5),
            height: 0.0,
        }
    }
}

/// Configuration for axis indicator display.
#[derive(Clone, Debug)]
pub struct AxisIndicatorConfig {
    /// Length of each axis line.
    pub length: f32,
    /// Position of the axis indicator origin.
    pub position: Vec3,
    /// Whether to show labels (not implemented here, would need text rendering).
    pub show_labels: bool,
}

impl Default for AxisIndicatorConfig {
    fn default() -> Self {
        Self {
            length: 1.0,
            position: Vec3::ZERO,
            show_labels: false,
        }
    }
}

/// Creates a grid floor for spatial reference.
///
/// The grid is centered at the origin and extends in the X and Z directions.
/// Grid lines are drawn at regular intervals, with axis-aligned lines highlighted.
///
/// # Arguments
///
/// * `config` - Grid configuration parameters
///
/// # Returns
///
/// A `LineBatch` containing all grid lines
pub fn create_grid(config: &GridConfig) -> LineBatch {
    let mut batch = LineBatch::with_capacity((config.divisions * 4 + 4) as usize);
    
    let half_size = config.size / 2.0;
    let step = config.size / config.divisions as f32;
    
    // Draw lines parallel to X axis (along Z)
    for i in 0..=config.divisions {
        let z = -half_size + i as f32 * step;
        let color = if i == config.divisions / 2 {
            config.axis_color // Center line (Z=0)
        } else {
            config.line_color
        };
        
        batch.add(
            Vec3::new(-half_size, config.height, z),
            Vec3::new(half_size, config.height, z),
            color,
        );
    }
    
    // Draw lines parallel to Z axis (along X)
    for i in 0..=config.divisions {
        let x = -half_size + i as f32 * step;
        let color = if i == config.divisions / 2 {
            config.axis_color // Center line (X=0)
        } else {
            config.line_color
        };
        
        batch.add(
            Vec3::new(x, config.height, -half_size),
            Vec3::new(x, config.height, half_size),
            color,
        );
    }
    
    batch
}

/// Creates an axis indicator showing X, Y, Z directions.
///
/// - X axis: Red
/// - Y axis: Green
/// - Z axis: Blue
///
/// # Arguments
///
/// * `config` - Axis indicator configuration
///
/// # Returns
///
/// A `LineBatch` containing the three axis lines
pub fn create_axis_indicator(config: &AxisIndicatorConfig) -> LineBatch {
    let mut batch = LineBatch::with_capacity(3);
    
    let origin = config.position;
    
    // X axis - Red
    batch.add(
        origin,
        origin + Vec3::X * config.length,
        Vec3::new(1.0, 0.0, 0.0),
    );
    
    // Y axis - Green
    batch.add(
        origin,
        origin + Vec3::Y * config.length,
        Vec3::new(0.0, 1.0, 0.0),
    );
    
    // Z axis - Blue
    batch.add(
        origin,
        origin + Vec3::Z * config.length,
        Vec3::new(0.0, 0.0, 1.0),
    );
    
    batch
}

/// Creates a bounding box outline around an object.
///
/// # Arguments
///
/// * `center` - Center position of the bounding box
/// * `size` - Size of the bounding box (half-extents)
/// * `color` - Color of the bounding box lines
///
/// # Returns
///
/// A `LineBatch` containing the 12 edges of the bounding box
pub fn create_bounding_box(center: Vec3, size: Vec3, color: Vec3) -> LineBatch {
    let mut batch = LineBatch::with_capacity(12);
    
    let min = center - size;
    let max = center + size;
    
    // Bottom face (4 edges)
    batch.add(Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z), color);
    batch.add(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, min.y, max.z), color);
    batch.add(Vec3::new(max.x, min.y, max.z), Vec3::new(min.x, min.y, max.z), color);
    batch.add(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, min.y, min.z), color);
    
    // Top face (4 edges)
    batch.add(Vec3::new(min.x, max.y, min.z), Vec3::new(max.x, max.y, min.z), color);
    batch.add(Vec3::new(max.x, max.y, min.z), Vec3::new(max.x, max.y, max.z), color);
    batch.add(Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z), color);
    batch.add(Vec3::new(min.x, max.y, max.z), Vec3::new(min.x, max.y, min.z), color);
    
    // Vertical edges (4 edges)
    batch.add(Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, max.y, min.z), color);
    batch.add(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, max.y, min.z), color);
    batch.add(Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, max.y, max.z), color);
    batch.add(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, max.y, max.z), color);
    
    batch
}

/// Creates a selection outline for an entity using its transform.
///
/// This creates a wireframe box around the entity, optionally transformed by a matrix.
///
/// # Arguments
///
/// * `transform` - Transform matrix for the entity
/// * `size` - Size of the selection box (typically 1.0 for unit cubes)
/// * `color` - Color of the selection outline (typically bright for visibility)
///
/// # Returns
///
/// A `LineBatch` containing the selection outline
pub fn create_selection_outline(transform: &Mat4, size: Vec3, color: Vec3) -> LineBatch {
    let mut batch = LineBatch::with_capacity(12);
    
    // Define the 8 corners of a unit cube
    let corners = [
        Vec3::new(-size.x, -size.y, -size.z),
        Vec3::new(size.x, -size.y, -size.z),
        Vec3::new(size.x, -size.y, size.z),
        Vec3::new(-size.x, -size.y, size.z),
        Vec3::new(-size.x, size.y, -size.z),
        Vec3::new(size.x, size.y, -size.z),
        Vec3::new(size.x, size.y, size.z),
        Vec3::new(-size.x, size.y, size.z),
    ];
    
    // Transform corners by the entity's transform
    let transformed_corners: Vec<Vec3> = corners
        .iter()
        .map(|&corner| transform.transform_point3(corner))
        .collect();
    
    // Bottom face (4 edges)
    batch.add(transformed_corners[0], transformed_corners[1], color);
    batch.add(transformed_corners[1], transformed_corners[2], color);
    batch.add(transformed_corners[2], transformed_corners[3], color);
    batch.add(transformed_corners[3], transformed_corners[0], color);
    
    // Top face (4 edges)
    batch.add(transformed_corners[4], transformed_corners[5], color);
    batch.add(transformed_corners[5], transformed_corners[6], color);
    batch.add(transformed_corners[6], transformed_corners[7], color);
    batch.add(transformed_corners[7], transformed_corners[4], color);
    
    // Vertical edges (4 edges)
    batch.add(transformed_corners[0], transformed_corners[4], color);
    batch.add(transformed_corners[1], transformed_corners[5], color);
    batch.add(transformed_corners[2], transformed_corners[6], color);
    batch.add(transformed_corners[3], transformed_corners[7], color);
    
    batch
}

/// Creates gizmo lines for a transform gizmo.
///
/// This is a helper to convert gizmo line data from the editor into a `LineBatch`.
///
/// # Arguments
///
/// * `lines` - Iterator of (start, end, color) tuples
///
/// # Returns
///
/// A `LineBatch` containing all gizmo lines
pub fn create_gizmo_lines<I>(lines: I) -> LineBatch
where
    I: IntoIterator<Item = (Vec3, Vec3, Vec3)>,
{
    let lines_vec: Vec<_> = lines.into_iter().collect();
    let mut batch = LineBatch::with_capacity(lines_vec.len());
    
    for (start, end, color) in lines_vec {
        batch.add(start, end, color);
    }
    
    batch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_creation() {
        let config = GridConfig::default();
        let batch = create_grid(&config);
        
        let expected_lines = (config.divisions + 1) * 2;
        assert_eq!(batch.len(), expected_lines as usize);
    }

    #[test]
    fn test_axis_indicator_creation() {
        let config = AxisIndicatorConfig::default();
        let batch = create_axis_indicator(&config);
        
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_bounding_box_creation() {
        let center = Vec3::ZERO;
        let size = Vec3::ONE;
        let color = Vec3::new(1.0, 1.0, 0.0);
        
        let batch = create_bounding_box(center, size, color);
        
        assert_eq!(batch.len(), 12);
    }

    #[test]
    fn test_selection_outline_creation() {
        let transform = Mat4::IDENTITY;
        let size = Vec3::new(0.5, 0.5, 0.5);
        let color = Vec3::new(1.0, 0.5, 0.0);
        
        let batch = create_selection_outline(&transform, size, color);
        
        assert_eq!(batch.len(), 12);
    }

    #[test]
    fn test_gizmo_lines_creation() {
        let lines = vec![
            (Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0)),
            (Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 1.0, 0.0)),
            (Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 1.0)),
        ];
        
        let batch = create_gizmo_lines(lines);
        
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_empty_gizmo_lines() {
        let batch = create_gizmo_lines(vec![]);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_grid_config_default() {
        let config = GridConfig::default();
        assert_eq!(config.size, 20.0);
        assert_eq!(config.divisions, 20);
        assert_eq!(config.height, 0.0);
    }

    #[test]
    fn test_axis_indicator_config_default() {
        let config = AxisIndicatorConfig::default();
        assert_eq!(config.length, 1.0);
        assert_eq!(config.position, Vec3::ZERO);
        assert!(!config.show_labels);
    }

    #[test]
    fn test_custom_grid_config() {
        let config = GridConfig {
            size: 10.0,
            divisions: 10,
            line_color: Vec3::new(0.5, 0.5, 0.5),
            axis_color: Vec3::new(1.0, 1.0, 1.0),
            height: 0.5,
        };
        
        let batch = create_grid(&config);
        assert_eq!(batch.len(), 22); // (10+1)*2 = 22 lines
    }

    #[test]
    fn test_bounding_box_with_offset() {
        let center = Vec3::new(5.0, 10.0, 15.0);
        let size = Vec3::new(2.0, 3.0, 4.0);
        let color = Vec3::ONE;
        
        let batch = create_bounding_box(center, size, color);
        assert_eq!(batch.len(), 12);
    }

    #[test]
    fn test_selection_outline_with_transform() {
        let translation = Mat4::from_translation(Vec3::new(10.0, 5.0, 0.0));
        let size = Vec3::ONE;
        let color = Vec3::new(1.0, 0.5, 0.0);
        
        let batch = create_selection_outline(&translation, size, color);
        assert_eq!(batch.len(), 12);
    }
}
