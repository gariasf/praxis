/// Common vertex layouts for basic shapes.
///
/// These are provided as examples and for testing. Real applications
/// would typically load vertex data from files or generate it procedurally.
use crate::vertex::Vertex2D;

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
