/// Common vertex layouts for basic shapes.
///
/// These are provided as examples and for testing. Real applications
/// would typically load vertex data from files or generate it procedurally.
use crate::vertex::{Vertex2D, Vertex3D};

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
