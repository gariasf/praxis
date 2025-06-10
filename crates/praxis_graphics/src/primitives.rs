/// Common vertex layouts for basic shapes.
///
/// These are provided as examples and for testing. Real applications
/// would typically load vertex data from files or generate it procedurally.
use crate::vertex::VertexData;

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
