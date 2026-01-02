//! Vertex data structures and utilities for the graphics system.
//!
//! This module defines the vertex formats used by the graphics pipeline to render geometry.
//! Supports both 2D vertices (position + color) and 3D vertices (position + color + UV coordinates).

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
    #[allow(dead_code)]
    pub fn new(position: [f32; 2], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}
/// Vertex data for 3D rendering with texture support.
///
/// Each vertex contains:
/// - A 3D position in model/world space
/// - An RGB color value
/// - UV texture coordinates
///
/// # Memory Layout
///
/// The struct is marked with `#[repr(C)]` to ensure predictable memory layout:
///
/// ```text
/// Vertex3D (32 bytes total):
/// ┌──────────────────┬──────────────────┬──────────────┐
/// │ position (12b)   │ color (12b)      │ uv (8b)      │
/// ├──────┬──────┬────┼──────┬──────┬────┼──────┬───────┤
/// │ x:f32│ y:f32│z:f32│ r:f32│ g:f32│b:f32│ u:f32│ v:f32│
/// └──────┴──────┴────┴──────┴──────┴────┴──────┴───────┘
/// ```
///
/// # Shader Binding
///
/// This vertex format maps to the following shader inputs:
/// - `location = 0`: position (vec3)
/// - `location = 1`: color (vec3)
/// - `location = 2`: uv (vec2)
///
/// # Texture Coordinates
///
/// UV coordinates follow standard OpenGL/Vulkan conventions:
/// ```text
///   v
///   ^
/// 1 │ (0,1)────(1,1)
///   │   │        │
///   │   │        │
/// 0 │ (0,0)────(1,0)
///   └──────────────> u
///     0            1
/// ```
///
/// # Example
///
/// ```rust
/// use praxis_graphics::vertex::Vertex3D;
///
/// // Create a textured vertex at origin with white color
/// let vertex = Vertex3D::with_uv(
///     [0.0, 0.0, 0.0],    // position
///     [1.0, 1.0, 1.0],    // color (white)
///     [0.5, 0.5]          // UV (center of texture)
/// );
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
pub struct Vertex3D {
    /// 3D position in model/world space.
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    
    /// RGB color values [0.0, 1.0].
    ///
    /// This color is multiplied with the texture sample in the fragment shader.
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
    
    /// UV texture coordinates [0.0, 1.0].
    ///
    /// These coordinates determine which part of the texture is sampled
    /// for this vertex. Values outside [0,1] will wrap or clamp depending
    /// on the sampler configuration.
    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

impl Vertex3D {
    /// Creates a new vertex with the given position and color.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position in world space
    /// * `color` - RGB color values in range [0.0, 1.0]
    ///
    /// # Example
    ///
    /// ```rust
    /// // Create a white vertex at the origin
    /// let vertex = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    /// ```
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            color,
            uv: [0.0, 0.0],
        }
    }

    /// Creates a new vertex with position, color, and texture coordinates.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position in world space
    /// * `color` - RGB color values in range [0.0, 1.0]
    /// * `uv` - Texture coordinates in range [0.0, 1.0]
    pub fn with_uv(position: [f32; 3], color: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            color,
            uv,
        }
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

    #[test]
    fn test_vertex3d_creation() {
        let vertex = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(vertex.position, [0.0, 0.0, 0.0]);
        assert_eq!(vertex.color, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_vertex3d_size() {
        // Ensure our vertex struct has the expected size
        assert_eq!(std::mem::size_of::<Vertex3D>(), 32); // 3*4 + 3*4 + 2*4 = 32 bytes
    }
}
