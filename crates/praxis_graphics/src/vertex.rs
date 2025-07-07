//! Vertex data structures and utilities for the graphics system.
//!
//! This module defines the vertex format used by the graphics pipeline to render geometry.
//! Currently supports 2D positions with RGB colors for basic rendering.

use vulkano::pipeline::graphics::vertex_input::Vertex;

/// Vertex data for rendering geometry.
///
/// Each vertex contains:
/// - A 2D position in normalized device coordinates (NDC)
/// - An RGB color value
///
/// # Memory Layout
///
/// The struct is marked with `#[repr(C)]` to ensure predictable memory layout
/// matching what the GPU expects:
///
/// ```text
/// VertexData (20 bytes total):
/// ┌─────────────────┬─────────────────────────┐
/// │ position (8b)   │ color (12b)             │
/// ├────────┬────────┼────────┬────────┬───────┤
/// │ x: f32 │ y: f32 │ r: f32 │ g: f32 │ b: f32│
/// └────────┴────────┴────────┴────────┴───────┘
/// ```
///
/// # Coordinate System
///
/// Positions use Vulkan's normalized device coordinates:
/// ```text
///              +Y (1.0)
///               │
///               │
/// (-1.0) ───────┼─────── +X (1.0)
///               │
///               │
///              -Y (-1.0)
/// ```
///
/// # Example
///
/// ```rust
/// // Create a red vertex at the top of the screen
/// let vertex = VertexData::new([0.0, 1.0], [1.0, 0.0, 0.0]);
///
/// // Create vertices for a triangle
/// let triangle = [
///     VertexData::new([-0.5, -0.5], [1.0, 0.0, 0.0]), // Bottom-left (red)
///     VertexData::new([ 0.5, -0.5], [0.0, 1.0, 0.0]), // Bottom-right (green)
///     VertexData::new([ 0.0,  0.5], [0.0, 0.0, 1.0]), // Top (blue)
/// ];
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
pub struct Vertex2D {
    /// Position in 2D normalized device coordinates.
    ///
    /// Range: [-1.0, 1.0] for both x and y components.
    /// - (-1, -1) is bottom-left of the screen
    /// - (1, 1) is top-right of the screen
    /// - (0, 0) is the center
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    /// RGB color values.
    ///
    /// Range: [0.0, 1.0] for each component.
    /// - (1, 0, 0) is pure red
    /// - (0, 1, 0) is pure green
    /// - (0, 0, 1) is pure blue
    /// - (1, 1, 1) is white
    /// - (0, 0, 0) is black
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

impl Vertex2D {
    /// Creates a new vertex with the given position and color.
    ///
    /// # Arguments
    ///
    /// * `position` - 2D position in normalized device coordinates [-1.0, 1.0]
    /// * `color` - RGB color values in range [0.0, 1.0]
    ///
    /// # Example
    ///
    /// ```rust
    /// // Create a white vertex at the origin
    /// let vertex = VertexData::new([0.0, 0.0], [1.0, 1.0, 1.0]);
    /// ```
    pub fn new(position: [f32; 2], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = Vertex2D::new([0.5, -0.5], [1.0, 0.0, 0.5]);
        assert_eq!(vertex.position, [0.5, -0.5]);
        assert_eq!(vertex.color, [1.0, 0.0, 0.5]);
    }

    #[test]
    fn test_vertex_size() {
        // Ensure our vertex struct has the expected size
        assert_eq!(std::mem::size_of::<Vertex2D>(), 20); // 2*4 + 3*4 = 20 bytes
    }
}
