//! Vertex data structures and utilities for the graphics system.
//!
//! This module defines the vertex formats used by the graphics pipeline to render geometry.
//!
//! # Zero-Copy Data Conversion with bytemuck
//!
//! Vertex data uses `bytemuck::Pod` (Plain Old Data) for safe, zero-copy conversion
//! between Rust types and GPU memory. This eliminates serialization overhead and
//! allows direct memory uploads.
//!
//! ## Why bytemuck?
//!
//! ```rust
//! use praxis_graphics::Vertex3D;
//!
//! let vertex = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
//!
//! // Zero-copy conversion to byte slice
//! let bytes: &[u8] = bytemuck::bytes_of(&vertex);
//!
//! // Can be directly uploaded to GPU buffer
//! // No serialization, no copying, just reinterpretation
//! ```
//!
//! ## Requirements for Pod
//!
//! For a type to implement `bytemuck::Pod`, it must be:
//!
//! - `#[repr(C)]`: Stable, predictable memory layout
//! - `Copy`: Bitwise copyable
//! - `Zeroable`: Safe to zero-initialize
//! - No padding bits with undefined values
//! - No references, pointers, or non-Pod fields
//!
//! ## Memory Layout Guarantees
//!
//! ```text
//! Vertex3D (92 bytes total):
//! Offset  Field           Size  Type
//! 0       position        12    [f32; 3]
//! 12      normal          12    [f32; 3]
//! 24      color           12    [f32; 3]
//! 36      uv              8     [f32; 2]
//! 44      tangent         16    [f32; 4]
//! 60      bone_indices    16    [i32; 4]
//! 76      bone_weights    16    [f32; 4]
//! ```
//!
//! This layout is guaranteed stable by `#[repr(C)]` and matches Vulkan's
//! expectations for vertex input.

use vulkano::pipeline::graphics::vertex_input::Vertex;

/// Vertex data for 3D rendering with texture support, lighting, and skeletal animation.
///
/// Each vertex contains:
/// - A 3D position in model/world space
/// - A normal vector for lighting calculations
/// - An RGB color value
/// - UV texture coordinates
/// - A tangent vector for normal mapping
/// - Bone indices for skeletal animation (up to 4 bones per vertex)
/// - Bone weights for skeletal animation (sum should equal 1.0)
///
/// # Memory Layout
///
/// The struct is marked with `#[repr(C)]` to ensure predictable memory layout:
///
/// ```text
/// Vertex3D (92 bytes total):
/// ┌──────────────┬──────────────┬──────────────┬──────────┬──────────────┬────────────────┬────────────────┐
/// │ position(12b)│ normal(12b)  │ color(12b)   │ uv(8b)   │ tangent(16b) │ bone_indices(16b)│ bone_weights(16b)│
/// └──────────────┴──────────────┴──────────────┴──────────┴──────────────┴────────────────┴────────────────┘
/// ```
///
/// # Shader Binding
///
/// This vertex format maps to the following shader inputs:
/// - `location = 0`: position (vec3)
/// - `location = 1`: normal (vec3)
/// - `location = 2`: color (vec3)
/// - `location = 3`: uv (vec2)
/// - `location = 4`: tangent (vec4, w component indicates handedness for bitangent)
/// - `location = 5`: bone_indices (ivec4, 4 bone indices per vertex)
/// - `location = 6`: bone_weights (vec4, 4 weights per vertex)
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
/// # Tangent Space
///
/// The tangent vector (with normal) forms the TBN (Tangent-Bitangent-Normal) matrix
/// for normal mapping. The tangent.w component stores the handedness (+1 or -1) for
/// computing the bitangent: `bitangent = cross(normal, tangent.xyz) * tangent.w`
///
/// # Example
///
/// ```rust
/// use praxis_graphics::Vertex3D;
///
/// // Create a textured vertex with lighting at origin
/// let vertex = Vertex3D::with_all(
///     [0.0, 0.0, 0.0],    // position
///     [0.0, 1.0, 0.0],    // normal (pointing up)
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

    /// Normal vector for lighting calculations (should be normalized).
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],

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

    /// Tangent vector for normal mapping (xyz) with handedness (w).
    ///
    /// The tangent vector, along with the normal, forms the tangent space basis.
    /// The w component stores the handedness (+1.0 or -1.0) used to compute the
    /// bitangent: `bitangent = cross(normal, tangent.xyz) * tangent.w`
    #[format(R32G32B32A32_SFLOAT)]
    pub tangent: [f32; 4],

    /// Bone indices for skeletal animation (up to 4 bones per vertex).
    ///
    /// Each vertex can be influenced by up to 4 bones. The indices reference
    /// bones in the skeleton's bone array. If fewer than 4 bones are needed,
    /// unused indices should be set to 0 and their weights to 0.0.
    #[format(R32G32B32A32_SINT)]
    pub bone_indices: [i32; 4],

    /// Bone weights for skeletal animation (must sum to 1.0).
    ///
    /// Each weight corresponds to the influence of the bone at the same index
    /// in bone_indices. For example, bone_weights[0] is the weight for the bone
    /// at bone_indices[0]. The sum of all weights should equal 1.0.
    #[format(R32G32B32A32_SFLOAT)]
    pub bone_weights: [f32; 4],
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
    /// use praxis_graphics::Vertex3D;
    ///
    /// // Create a white vertex at the origin
    /// let vertex = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    /// ```
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            normal: [0.0, 1.0, 0.0],
            color,
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            bone_indices: [0, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
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
            normal: [0.0, 1.0, 0.0],
            color,
            uv,
            tangent: [1.0, 0.0, 0.0, 1.0],
            bone_indices: [0, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Creates a new vertex with all attributes.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position in world space
    /// * `normal` - Normal vector (should be normalized)
    /// * `color` - RGB color values in range [0.0, 1.0]
    /// * `uv` - Texture coordinates in range [0.0, 1.0]
    pub fn with_all(position: [f32; 3], normal: [f32; 3], color: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            color,
            uv,
            tangent: [1.0, 0.0, 0.0, 1.0],
            bone_indices: [0, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Creates a new vertex with all attributes including tangent.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position in world space
    /// * `normal` - Normal vector (should be normalized)
    /// * `color` - RGB color values in range [0.0, 1.0]
    /// * `uv` - Texture coordinates in range [0.0, 1.0]
    /// * `tangent` - Tangent vector (xyz) with handedness (w: +1 or -1)
    pub fn with_tangent(
        position: [f32; 3],
        normal: [f32; 3],
        color: [f32; 3],
        uv: [f32; 2],
        tangent: [f32; 4],
    ) -> Self {
        Self {
            position,
            normal,
            color,
            uv,
            tangent,
            bone_indices: [0, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Creates a new vertex with all attributes including skeletal animation data.
    ///
    /// # Arguments
    ///
    /// * `position` - 3D position in model space
    /// * `normal` - Normal vector (should be normalized)
    /// * `color` - RGB color values in range [0.0, 1.0]
    /// * `uv` - Texture coordinates in range [0.0, 1.0]
    /// * `tangent` - Tangent vector (xyz) with handedness (w: +1 or -1)
    /// * `bone_indices` - Indices of bones that influence this vertex (up to 4)
    /// * `bone_weights` - Weights for each bone influence (should sum to 1.0)
    pub fn with_skinning(
        position: [f32; 3],
        normal: [f32; 3],
        color: [f32; 3],
        uv: [f32; 2],
        tangent: [f32; 4],
        bone_indices: [i32; 4],
        bone_weights: [f32; 4],
    ) -> Self {
        Self {
            position,
            normal,
            color,
            uv,
            tangent,
            bone_indices,
            bone_weights,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex3d_creation() {
        let vertex = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(vertex.position, [0.0, 0.0, 0.0]);
        assert_eq!(vertex.color, [1.0, 1.0, 1.0]);
        assert_eq!(vertex.normal, [0.0, 1.0, 0.0]);
        assert_eq!(vertex.uv, [0.0, 0.0]);
    }

    #[test]
    fn test_vertex3d_with_uv() {
        let vertex = Vertex3D::with_uv([1.0, 2.0, 3.0], [0.5, 0.6, 0.7], [0.25, 0.75]);
        assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex.color, [0.5, 0.6, 0.7]);
        assert_eq!(vertex.uv, [0.25, 0.75]);
        assert_eq!(vertex.normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_vertex3d_with_all() {
        let vertex = Vertex3D::with_all(
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 1.0],
            [0.5, 0.6, 0.7],
            [0.25, 0.75],
        );
        assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex.normal, [0.0, 0.0, 1.0]);
        assert_eq!(vertex.color, [0.5, 0.6, 0.7]);
        assert_eq!(vertex.uv, [0.25, 0.75]);
    }

    #[test]
    fn test_vertex3d_default() {
        let vertex = Vertex3D::default();
        assert_eq!(vertex.position, [0.0, 0.0, 0.0]);
        assert_eq!(vertex.normal, [0.0, 0.0, 0.0]);
        assert_eq!(vertex.color, [0.0, 0.0, 0.0]);
        assert_eq!(vertex.uv, [0.0, 0.0]);
    }

    #[test]
    fn test_vertex3d_size() {
        // Vertex3D layout: position(12) + normal(12) + color(12) + uv(8) + tangent(16)
        // + bone_indices(16) + bone_weights(16) = 92 bytes
        assert_eq!(std::mem::size_of::<Vertex3D>(), 92);
    }

    #[test]
    fn test_vertex3d_alignment() {
        assert_eq!(std::mem::align_of::<Vertex3D>(), 4);
    }

    #[test]
    fn test_vertex3d_clone() {
        let vertex1 = Vertex3D::with_all(
            [1.0, 2.0, 3.0],
            [0.0, 1.0, 0.0],
            [0.8, 0.9, 1.0],
            [0.5, 0.5],
        );
        let vertex2 = vertex1;
        assert_eq!(vertex1.position, vertex2.position);
        assert_eq!(vertex1.normal, vertex2.normal);
        assert_eq!(vertex1.color, vertex2.color);
        assert_eq!(vertex1.uv, vertex2.uv);
    }

    #[test]
    fn test_vertex3d_copy() {
        let vertex1 = Vertex3D::new([1.0, 2.0, 3.0], [0.5, 0.5, 0.5]);
        let vertex2 = vertex1;
        assert_eq!(vertex1.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex2.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_vertex3d_position_extremes() {
        let vertex = Vertex3D::new([f32::MAX, f32::MIN, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(vertex.position[0], f32::MAX);
        assert_eq!(vertex.position[1], f32::MIN);
        assert_eq!(vertex.position[2], 0.0);
    }

    #[test]
    fn test_vertex3d_color_values() {
        let vertex = Vertex3D::new([0.0, 0.0, 0.0], [0.0, 0.5, 1.0]);
        assert_eq!(vertex.color, [0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_vertex3d_uv_range() {
        let vertex = Vertex3D::with_uv([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0]);
        assert_eq!(vertex.uv, [0.0, 1.0]);

        let vertex2 = Vertex3D::with_uv([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5]);
        assert_eq!(vertex2.uv, [0.5, 0.5]);
    }

    #[test]
    fn test_vertex3d_normal_vectors() {
        let up = Vertex3D::with_all(
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.0, 0.0],
        );
        assert_eq!(up.normal, [0.0, 1.0, 0.0]);

        let right = Vertex3D::with_all(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.0, 0.0],
        );
        assert_eq!(right.normal, [1.0, 0.0, 0.0]);

        let forward = Vertex3D::with_all(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 0.0],
        );
        assert_eq!(forward.normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_vertex3d_bytemuck_pod() {
        let vertex = Vertex3D::new([1.0, 2.0, 3.0], [0.5, 0.6, 0.7]);
        let bytes = bytemuck::bytes_of(&vertex);
        assert_eq!(bytes.len(), 92);
    }

    #[test]
    fn test_vertex3d_bytemuck_cast() {
        let vertex = Vertex3D::with_all(
            [1.0, 2.0, 3.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.6, 0.7],
            [0.25, 0.75],
        );
        let bytes = bytemuck::bytes_of(&vertex);
        let vertex_back: &Vertex3D = bytemuck::from_bytes(bytes);
        assert_eq!(vertex_back.position, vertex.position);
        assert_eq!(vertex_back.normal, vertex.normal);
        assert_eq!(vertex_back.color, vertex.color);
        assert_eq!(vertex_back.uv, vertex.uv);
    }
}
