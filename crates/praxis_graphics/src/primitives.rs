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
/// The cube is defined in model-space coordinates (range -0.5..0.5 on each axis). It returns
/// a tuple `(vertices, indices)` where `vertices` contains the eight unique `Vertex3D`s and
/// `indices` defines 12 triangles (36 indices) that reference those vertices.
///
/// Note: This version does not include per-face normals. For proper lighting, use `colored_cube_mesh()`.
#[allow(dead_code)]
pub fn colored_cube() -> (Vec<Vertex3D>, Vec<u16>) {
    // 8 unique cube vertices (normals default to up)
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

/// Creates mesh data for a colored cube with proper per-face normals.
///
/// Returns a `MeshData` structure ready to be uploaded to the GPU.
/// Each face has its own vertices with correct normals for lighting.
pub fn colored_cube_mesh() -> MeshData {
    // Cube with 24 vertices (4 per face) for proper per-face normals
    let positions = vec![
        // Back face (-Z)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        // Front face (+Z)
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Bottom face (-Y)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // Top face (+Y)
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Left face (-X)
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        // Right face (+X)
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
    ];

    let normals = vec![
        // Back face
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // Front face
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Bottom face
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Top face
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Left face
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Right face
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];

    let colors = vec![
        // Back face (red)
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Front face (green)
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Bottom face (blue)
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Top face (yellow)
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        // Left face (magenta)
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        // Right face (cyan)
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];

    let indices: Vec<u16> = vec![
        // Back face
        0, 1, 2, 2, 3, 0, // Front face
        4, 5, 6, 6, 7, 4, // Bottom face
        8, 9, 10, 10, 11, 8, // Top face
        12, 13, 14, 14, 15, 12, // Left face
        16, 17, 18, 18, 19, 16, // Right face
        20, 21, 22, 22, 23, 20,
    ];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        indices,
    }
}

/// Creates mesh data for a unit cube with a single color and proper normals.
///
/// # Arguments
///
/// * `color` - RGB color for all vertices
pub fn solid_cube_mesh(color: [f32; 3]) -> MeshData {
    let positions = vec![
        // Back face (-Z)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        // Front face (+Z)
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Bottom face (-Y)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // Top face (+Y)
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Left face (-X)
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        // Right face (+X)
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
    ];

    let normals = vec![
        // Back face
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // Front face
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Bottom face
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Top face
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Left face
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Right face
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];

    let indices: Vec<u16> = vec![
        // Back face
        0, 1, 2, 2, 3, 0, // Front face
        4, 5, 6, 6, 7, 4, // Bottom face
        8, 9, 10, 10, 11, 8, // Top face
        12, 13, 14, 14, 15, 12, // Left face
        16, 17, 18, 18, 19, 16, // Right face
        20, 21, 22, 22, 23, 20,
    ];

    let colors = vec![color; positions.len()];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        indices,
    }
}

/// Creates mesh data for a textured unit cube with proper normals.
///
/// Each face is mapped to the full UV range [0,0] to [1,1].
/// The cube is centered at the origin with size 1.0.
///
/// # Arguments
///
/// * `color` - RGB color multiplier for all vertices (typically white [1.0, 1.0, 1.0])
pub fn textured_cube_mesh(color: [f32; 3]) -> MeshData {
    let positions = vec![
        // Back face (-Z)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        // Front face (+Z)
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Bottom face (-Y)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // Top face (+Y)
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // Left face (-X)
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        // Right face (+X)
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
    ];

    let normals = vec![
        // Back face
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // Front face
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Bottom face
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Top face
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Left face
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Right face
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
    ];

    let uvs = vec![
        // Back face
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        // Front face
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        // Bottom face
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        // Top face
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
        // Left face
        [1.0, 1.0],
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        // Right face
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
        [0.0, 0.0],
    ];

    let indices: Vec<u16> = vec![
        // Back face
        0, 1, 2, 2, 3, 0, // Front face
        4, 5, 6, 6, 7, 4, // Bottom face
        8, 9, 10, 10, 11, 8, // Top face
        12, 13, 14, 14, 15, 12, // Left face
        16, 17, 18, 18, 19, 16, // Right face
        20, 21, 22, 22, 23, 20,
    ];

    let colors = vec![color; positions.len()];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
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

    let normals = vec![
        [0.0, 1.0, 0.0], // Up
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let indices = vec![
        0, 1, 2, // First triangle
        2, 3, 0, // Second triangle
    ];

    let colors = vec![color; 4];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        indices,
    }
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

    let normals = vec![
        [0.0, 1.0, 0.0], // Up
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
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
        normals: Some(normals),
        uvs: Some(uvs),
        indices,
    }
}

/// Creates mesh data for a pyramid with proper normals.
///
/// # Arguments
///
/// * `base_color` - RGB color for the base vertices
/// * `tip_color` - RGB color for the tip vertex
pub fn pyramid_mesh(base_color: [f32; 3], tip_color: [f32; 3]) -> MeshData {
    // Pyramid with separate vertices for each face to have proper normals
    let positions = vec![
        // Base (4 vertices facing down)
        [-0.5, 0.0, -0.5],
        [0.5, 0.0, -0.5],
        [0.5, 0.0, 0.5],
        [-0.5, 0.0, 0.5],
        // Back face (3 vertices)
        [-0.5, 0.0, -0.5],
        [0.5, 0.0, -0.5],
        [0.0, 1.0, 0.0],
        // Front face (3 vertices)
        [0.5, 0.0, 0.5],
        [-0.5, 0.0, 0.5],
        [0.0, 1.0, 0.0],
        // Left face (3 vertices)
        [-0.5, 0.0, 0.5],
        [-0.5, 0.0, -0.5],
        [0.0, 1.0, 0.0],
        // Right face (3 vertices)
        [0.5, 0.0, -0.5],
        [0.5, 0.0, 0.5],
        [0.0, 1.0, 0.0],
    ];

    let normals = vec![
        // Base normals (pointing down)
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Back face (calculated from cross product)
        [0.0, 0.447, -0.894],
        [0.0, 0.447, -0.894],
        [0.0, 0.447, -0.894],
        // Front face
        [0.0, 0.447, 0.894],
        [0.0, 0.447, 0.894],
        [0.0, 0.447, 0.894],
        // Left face
        [-0.894, 0.447, 0.0],
        [-0.894, 0.447, 0.0],
        [-0.894, 0.447, 0.0],
        // Right face
        [0.894, 0.447, 0.0],
        [0.894, 0.447, 0.0],
        [0.894, 0.447, 0.0],
    ];

    let colors = vec![
        // Base colors
        base_color, base_color, base_color, base_color, // Back face
        base_color, base_color, tip_color, // Front face
        base_color, base_color, tip_color, // Left face
        base_color, base_color, tip_color, // Right face
        base_color, base_color, tip_color,
    ];

    let indices = vec![
        // Base (two triangles)
        0, 2, 1, 0, 3, 2, // Back face
        4, 5, 6, // Front face
        7, 8, 9, // Left face
        10, 11, 12, // Right face
        13, 14, 15,
    ];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        indices,
    }
}

/// Creates mesh data for a UV sphere with proper normals.
///
/// A UV sphere is generated by sweeping a semicircle around the Y axis,
/// creating a sphere with latitude/longitude topology. This is ideal for
/// texturing since UV coordinates map naturally to the sphere surface.
///
/// # Arguments
///
/// * `radius` - Radius of the sphere
/// * `sectors` - Number of sectors (longitude divisions, typically 36)
/// * `stacks` - Number of stacks (latitude divisions, typically 18)
/// * `color` - RGB color for all vertices
///
/// # Performance
///
/// Vertex count = (stacks + 1) * (sectors + 1)
/// Triangle count = stacks * sectors * 2
///
/// For typical values (18 stacks, 36 sectors):
/// - Vertices: 703
/// - Triangles: 1296
pub fn sphere_mesh(radius: f32, sectors: u32, stacks: u32, color: [f32; 3]) -> MeshData {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0 - (i as f32) * std::f32::consts::PI / (stacks as f32);
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = (j as f32) * 2.0 * std::f32::consts::PI / (sectors as f32);

            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();

            positions.push([x, y, z]);

            // Normal is just the normalized position for a sphere centered at origin
            let length = (x * x + y * y + z * z).sqrt();
            normals.push([x / length, y / length, z / length]);

            colors.push(color);

            // UV coordinates: u = longitude [0,1], v = latitude [0,1]
            let u = j as f32 / sectors as f32;
            let v = i as f32 / stacks as f32;
            uvs.push([u, v]);
        }
    }

    // Generate indices
    for i in 0..stacks {
        let k1 = i * (sectors + 1);
        let k2 = k1 + sectors + 1;

        for j in 0..sectors {
            // Two triangles per quad
            if i != 0 {
                indices.push((k1 + j) as u16);
                indices.push((k2 + j) as u16);
                indices.push((k1 + j + 1) as u16);
            }

            if i != stacks - 1 {
                indices.push((k1 + j + 1) as u16);
                indices.push((k2 + j) as u16);
                indices.push((k2 + j + 1) as u16);
            }
        }
    }

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: Some(uvs),
        indices,
    }
}
