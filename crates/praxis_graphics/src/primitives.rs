/// Common vertex layouts for basic shapes.
///
/// These are provided as examples and for testing. Real applications
/// would typically load vertex data from files or generate it procedurally.
use crate::{
    mesh::MeshData,
    vertex::{Vertex2D, Vertex3D},
};

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
#[allow(dead_code)]
pub fn colored_triangle() -> [Vertex2D; 3] {
    [
        Vertex2D::new([-0.5, -0.5], [1.0, 0.0, 0.0]), // Bottom-left (red)
        Vertex2D::new([0.5, -0.5], [0.0, 1.0, 0.0]),  // Bottom-right (green)
        Vertex2D::new([0.0, 0.5], [0.0, 0.0, 1.0]),   // Top (blue)
    ]
}

/// Creates the unique vertices and index list for a coloured unit cube centred at the origin.
///
/// The cube is defined in model-space coordinates (range 0.5..0.5 on each axis).  It returns
/// a tuple `(vertices, indices)` where `vertices` contains the eight unique `Vertex3D`s and
/// `indices` defines 12 triangles (36 indices) that reference those vertices.
pub fn colored_cube() -> (Vec<Vertex3D>, Vec<u16>) {
    // 8 unique cube vertices
    let vertices = vec![
        // Back face (z = -0.5)
        Vertex3D::new([-0.5, -0.5, -0.5], [1.0, 0.0, 0.0]), // 0
        Vertex3D::new([0.5, -0.5, -0.5], [0.0, 1.0, 0.0]),  // 1
        Vertex3D::new([0.5, 0.5, -0.5], [0.0, 0.0, 1.0]),   // 2
        Vertex3D::new([-0.5, 0.5, -0.5], [1.0, 1.0, 0.0]),  // 3
        // Front face (z = 0.5)
        Vertex3D::new([-0.5, -0.5, 0.5], [1.0, 0.0, 1.0]), // 4
        Vertex3D::new([0.5, -0.5, 0.5], [0.0, 1.0, 1.0]),  // 5
        Vertex3D::new([0.5, 0.5, 0.5], [1.0, 1.0, 1.0]),   // 6
        Vertex3D::new([-0.5, 0.5, 0.5], [0.0, 0.0, 0.0]),  // 7
    ];

    // 12 triangles (two per face) expressed as 36 indices
    let indices: Vec<u16> = vec![
        // Back face
        0, 1, 2, 2, 3, 0, // Front face
        4, 6, 5, 4, 7, 6, // Bottom face
        4, 5, 1, 4, 1, 0, // Top face
        3, 2, 6, 3, 6, 7, // Left face
        4, 0, 3, 4, 3, 7, // Right face
        1, 5, 6, 1, 6, 2,
    ];

    (vertices, indices)
}

/// Creates mesh data for a colored cube.
///
/// Returns a `MeshData` structure ready to be uploaded to the GPU.
pub fn colored_cube_mesh() -> MeshData {
    let (vertices, indices) = colored_cube();

    let positions: Vec<[f32; 3]> = vertices.iter().map(|v| v.position).collect();
    let colors: Vec<[f32; 3]> = vertices.iter().map(|v| v.color).collect();

    MeshData::with_colors(positions, colors, indices)
}

/// Creates mesh data for a unit cube with a single color.
///
/// # Arguments
///
/// * `color` - RGB color for all vertices
pub fn solid_cube_mesh(color: [f32; 3]) -> MeshData {
    let positions = vec![
        // Back face (z = -0.5)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        // Front face (z = 0.5)
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ];

    let indices: Vec<u16> = vec![
        // Back face
        0, 1, 2, 2, 3, 0, // Front face
        4, 6, 5, 4, 7, 6, // Bottom face
        4, 5, 1, 4, 1, 0, // Top face
        3, 2, 6, 3, 6, 7, // Left face
        4, 0, 3, 4, 3, 7, // Right face
        1, 5, 6, 1, 6, 2,
    ];

    let colors = vec![color; 8];

    MeshData::with_colors(positions, colors, indices)
}

/// Creates mesh data for a textured unit cube.
///
/// Each face is mapped to the full UV range [0,0] to [1,1].
/// The cube is centered at the origin with size 1.0.
///
/// # Arguments
///
/// * `color` - RGB color multiplier for all vertices (typically white [1.0, 1.0, 1.0])
pub fn textured_cube_mesh(color: [f32; 3]) -> MeshData {
    let positions = vec![
        // Back face (-Z, 4 vertices)
        [-0.5, -0.5, -0.5], // 0
        [0.5, -0.5, -0.5],  // 1
        [0.5, 0.5, -0.5],   // 2
        [-0.5, 0.5, -0.5],  // 3
        // Front face (+Z, 4 vertices)
        [-0.5, -0.5, 0.5], // 4
        [0.5, -0.5, 0.5],  // 5
        [0.5, 0.5, 0.5],   // 6
        [-0.5, 0.5, 0.5],  // 7
        // Bottom face (-Y, 4 vertices)
        [-0.5, -0.5, -0.5], // 8
        [0.5, -0.5, -0.5],  // 9
        [0.5, -0.5, 0.5],   // 10
        [-0.5, -0.5, 0.5],  // 11
        // Top face (+Y, 4 vertices)
        [-0.5, 0.5, -0.5], // 12
        [0.5, 0.5, -0.5],  // 13
        [0.5, 0.5, 0.5],   // 14
        [-0.5, 0.5, 0.5],  // 15
        // Left face (-X, 4 vertices)
        [-0.5, -0.5, -0.5], // 16
        [-0.5, -0.5, 0.5],  // 17
        [-0.5, 0.5, 0.5],   // 18
        [-0.5, 0.5, -0.5],  // 19
        // Right face (+X, 4 vertices)
        [0.5, -0.5, -0.5], // 20
        [0.5, -0.5, 0.5],  // 21
        [0.5, 0.5, 0.5],   // 22
        [0.5, 0.5, -0.5],  // 23
    ];

    let uvs = vec![
        // Back face
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        // Front face
        [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        // Bottom face
        [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        // Top face
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        // Left face
        [1.0, 1.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        // Right face
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    ];

    let indices: Vec<u16> = vec![
        // Back face
        0, 1, 2, 2, 3, 0,
        // Front face
        4, 5, 6, 6, 7, 4,
        // Bottom face
        8, 9, 10, 10, 11, 8,
        // Top face
        12, 13, 14, 14, 15, 12,
        // Left face
        16, 17, 18, 18, 19, 16,
        // Right face
        20, 21, 22, 22, 23, 20,
    ];

    let colors = vec![color; positions.len()];

    MeshData {
        positions,
        colors: Some(colors),
        normals: None,
        uvs: Some(uvs),
        indices,
    }
}

/// Creates mesh data for a quad (plane) facing up (along Y axis).
///
/// # Arguments
///
/// * `size` - Size of the quad in world units
/// * `color` - RGB color for all vertices
pub fn quad_mesh(size: f32, color: [f32; 3]) -> MeshData {
    let half_size = size / 2.0;

    let positions = vec![
        [-half_size, 0.0, -half_size], // Bottom-left
        [half_size, 0.0, -half_size],  // Bottom-right
        [half_size, 0.0, half_size],   // Top-right
        [-half_size, 0.0, half_size],  // Top-left
    ];

    let indices = vec![
        0, 1, 2, // First triangle
        2, 3, 0, // Second triangle
    ];

    let colors = vec![color; 4];

    MeshData::with_colors(positions, colors, indices)
}

/// Creates mesh data for a textured quad (plane) facing up (along Y axis).
///
/// The quad is centered at the origin and mapped with standard UV coordinates.
///
/// # Arguments
///
/// * `size` - Size of the quad in world units
/// * `color` - RGB color multiplier for all vertices (typically white [1.0, 1.0, 1.0])
pub fn textured_quad_mesh(size: f32, color: [f32; 3]) -> MeshData {
    let half_size = size / 2.0;

    let positions = vec![
        [-half_size, 0.0, -half_size], // Bottom-left
        [half_size, 0.0, -half_size],  // Bottom-right
        [half_size, 0.0, half_size],   // Top-right
        [-half_size, 0.0, half_size],  // Top-left
    ];

    let uvs = vec![
        [0.0, 1.0], // Bottom-left
        [1.0, 1.0], // Bottom-right
        [1.0, 0.0], // Top-right
        [0.0, 0.0], // Top-left
    ];

    let indices = vec![
        0, 1, 2, // First triangle
        2, 3, 0, // Second triangle
    ];

    let colors = vec![color; 4];

    MeshData {
        positions,
        colors: Some(colors),
        normals: None,
        uvs: Some(uvs),
        indices,
    }
}

/// Creates mesh data for a pyramid.
///
/// # Arguments
///
/// * `base_color` - RGB color for the base vertices
/// * `tip_color` - RGB color for the tip vertex
pub fn pyramid_mesh(base_color: [f32; 3], tip_color: [f32; 3]) -> MeshData {
    let positions = vec![
        // Base vertices
        [-0.5, 0.0, -0.5],
        [0.5, 0.0, -0.5],
        [0.5, 0.0, 0.5],
        [-0.5, 0.0, 0.5],
        // Tip
        [0.0, 1.0, 0.0],
    ];

    let colors = vec![base_color, base_color, base_color, base_color, tip_color];

    let indices = vec![
        // Base
        0, 2, 1, 0, 3, 2, // Sides
        0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4,
    ];

    MeshData::with_colors(positions, colors, indices)
}
