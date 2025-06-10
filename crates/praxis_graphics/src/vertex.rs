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
pub struct VertexData {
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

impl VertexData {
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

    /// Creates a vertex with a position and white color.
    ///
    /// Convenience method for when you don't need colored vertices.
    ///
    /// # Example
    ///
    /// ```rust
    /// let vertex = VertexData::with_position([0.5, 0.5]);
    /// assert_eq!(vertex.color, [1.0, 1.0, 1.0]);
    /// ```
    pub fn with_position(position: [f32; 2]) -> Self {
        Self {
            position,
            color: [1.0, 1.0, 1.0],
        }
    }

    /// Creates a vertex at the origin with the specified color.
    ///
    /// Convenience method for when position will be set later or
    /// transformed by a matrix.
    ///
    /// # Example
    ///
    /// ```rust
    /// let vertex = VertexData::with_color([1.0, 0.0, 0.0]); // Red vertex at origin
    /// assert_eq!(vertex.position, [0.0, 0.0]);
    /// ```
    pub fn with_color(color: [f32; 3]) -> Self {
        Self {
            position: [0.0, 0.0],
            color,
        }
    }
}

/// Common vertex layouts for basic shapes.
///
/// These are provided as examples and for testing. Real applications
/// would typically load vertex data from files or generate it procedurally.
pub mod primitives {
    use super::VertexData;

    /// Creates vertices for a triangle with one vertex of each primary color.
    ///
    /// The triangle is centered at the origin with vertices at:
    /// ```text
    ///        (0, 0.5) Blue
    ///           /\
    ///          /  \
    ///         /    \
    ///        /      \
    ///       /________\
    /// (-0.5, -0.5)  (0.5, -0.5)
    ///     Red          Green
    /// ```
    pub fn colored_triangle() -> [VertexData; 3] {
        [
            VertexData::new([-0.5, -0.5], [1.0, 0.0, 0.0]), // Bottom-left (red)
            VertexData::new([0.5, -0.5], [0.0, 1.0, 0.0]),  // Bottom-right (green)
            VertexData::new([0.0, 0.5], [0.0, 0.0, 1.0]),   // Top (blue)
        ]
    }

    /// Creates vertices for a unit square centered at the origin.
    ///
    /// The square extends from -0.5 to 0.5 in both dimensions:
    /// ```text
    /// (-0.5, 0.5) ┌─────┐ (0.5, 0.5)
    ///             │     │
    ///             │  ·  │ (0, 0)
    ///             │     │
    /// (-0.5,-0.5) └─────┘ (0.5,-0.5)
    /// ```
    ///
    /// All vertices are white by default.
    pub fn unit_square() -> [VertexData; 4] {
        [
            VertexData::with_position([-0.5, -0.5]), // Bottom-left
            VertexData::with_position([0.5, -0.5]),  // Bottom-right
            VertexData::with_position([0.5, 0.5]),   // Top-right
            VertexData::with_position([-0.5, 0.5]),  // Top-left
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = VertexData::new([0.5, -0.5], [1.0, 0.0, 0.5]);
        assert_eq!(vertex.position, [0.5, -0.5]);
        assert_eq!(vertex.color, [1.0, 0.0, 0.5]);
    }

    #[test]
    fn test_vertex_with_position() {
        let vertex = VertexData::with_position([0.25, 0.75]);
        assert_eq!(vertex.position, [0.25, 0.75]);
        assert_eq!(vertex.color, [1.0, 1.0, 1.0]); // Should be white
    }

    #[test]
    fn test_vertex_with_color() {
        let vertex = VertexData::with_color([0.5, 0.5, 0.5]);
        assert_eq!(vertex.position, [0.0, 0.0]); // Should be at origin
        assert_eq!(vertex.color, [0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_vertex_size() {
        // Ensure our vertex struct has the expected size
        assert_eq!(std::mem::size_of::<VertexData>(), 20); // 2*4 + 3*4 = 20 bytes
    }
}
