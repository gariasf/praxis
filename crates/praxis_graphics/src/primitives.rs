/// Common vertex layouts for basic shapes.
///
/// These are provided as examples and for testing. Real applications
/// would typically load vertex data from files or generate it procedurally.
use crate::mesh::MeshData;

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
        let stack_angle =
            std::f32::consts::PI / 2.0 - (i as f32) * std::f32::consts::PI / (stacks as f32);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_cube_mesh_structure() {
        let mesh = colored_cube_mesh();
        assert_eq!(mesh.positions.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());
        assert_eq!(mesh.colors.as_ref().unwrap().len(), 24);
        assert_eq!(mesh.normals.as_ref().unwrap().len(), 24);
    }

    #[test]
    fn test_colored_cube_mesh_colors() {
        let mesh = colored_cube_mesh();
        let colors = mesh.colors.as_ref().unwrap();

        assert_eq!(colors[0], [1.0, 0.0, 0.0]);
        assert_eq!(colors[4], [0.0, 1.0, 0.0]);
        assert_eq!(colors[8], [0.0, 0.0, 1.0]);
        assert_eq!(colors[12], [1.0, 1.0, 0.0]);
        assert_eq!(colors[16], [1.0, 0.0, 1.0]);
        assert_eq!(colors[20], [0.0, 1.0, 1.0]);
    }

    #[test]
    fn test_colored_cube_mesh_normals() {
        let mesh = colored_cube_mesh();
        let normals = mesh.normals.as_ref().unwrap();

        assert_eq!(normals[0], [0.0, 0.0, -1.0]);
        assert_eq!(normals[4], [0.0, 0.0, 1.0]);
        assert_eq!(normals[8], [0.0, -1.0, 0.0]);
        assert_eq!(normals[12], [0.0, 1.0, 0.0]);
        assert_eq!(normals[16], [-1.0, 0.0, 0.0]);
        assert_eq!(normals[20], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_colored_cube_mesh_indices_valid() {
        let mesh = colored_cube_mesh();
        for &index in &mesh.indices {
            assert!((index as usize) < mesh.positions.len());
        }
    }

    #[test]
    fn test_solid_cube_mesh_structure() {
        let color = [0.5, 0.5, 0.5];
        let mesh = solid_cube_mesh(color);
        assert_eq!(mesh.positions.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());
    }

    #[test]
    fn test_solid_cube_mesh_uniform_color() {
        let color = [0.8, 0.2, 0.4];
        let mesh = solid_cube_mesh(color);
        let colors = mesh.colors.as_ref().unwrap();

        for c in colors {
            assert_eq!(*c, color);
        }
    }

    #[test]
    fn test_textured_cube_mesh_structure() {
        let color = [1.0, 1.0, 1.0];
        let mesh = textured_cube_mesh(color);
        assert_eq!(mesh.positions.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());
        assert!(mesh.uvs.is_some());
        assert_eq!(mesh.uvs.as_ref().unwrap().len(), 24);
    }

    #[test]
    fn test_textured_cube_mesh_uvs() {
        let mesh = textured_cube_mesh([1.0, 1.0, 1.0]);
        let uvs = mesh.uvs.as_ref().unwrap();

        for uv in uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
        }
    }

    #[test]
    fn test_quad_mesh_structure() {
        let mesh = quad_mesh(2.0, [1.0, 1.0, 1.0]);
        assert_eq!(mesh.positions.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());
    }

    #[test]
    fn test_quad_mesh_size() {
        let size = 5.0;
        let mesh = quad_mesh(size, [1.0, 1.0, 1.0]);
        let half_size = size / 2.0;

        assert_eq!(mesh.positions[0], [-half_size, 0.0, -half_size]);
        assert_eq!(mesh.positions[1], [half_size, 0.0, -half_size]);
        assert_eq!(mesh.positions[2], [half_size, 0.0, half_size]);
        assert_eq!(mesh.positions[3], [-half_size, 0.0, half_size]);
    }

    #[test]
    fn test_quad_mesh_normals() {
        let mesh = quad_mesh(1.0, [1.0, 1.0, 1.0]);
        let normals = mesh.normals.as_ref().unwrap();

        for normal in normals {
            assert_eq!(*normal, [0.0, 1.0, 0.0]);
        }
    }

    #[test]
    fn test_textured_quad_mesh_structure() {
        let mesh = textured_quad_mesh(2.0, [1.0, 1.0, 1.0]);
        assert_eq!(mesh.positions.len(), 4);
        assert_eq!(mesh.indices.len(), 6);
        assert!(mesh.uvs.is_some());
        assert_eq!(mesh.uvs.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn test_textured_quad_mesh_uvs() {
        let mesh = textured_quad_mesh(1.0, [1.0, 1.0, 1.0]);
        let uvs = mesh.uvs.as_ref().unwrap();

        assert_eq!(uvs[0], [0.0, 1.0]);
        assert_eq!(uvs[1], [1.0, 1.0]);
        assert_eq!(uvs[2], [1.0, 0.0]);
        assert_eq!(uvs[3], [0.0, 0.0]);
    }

    #[test]
    fn test_pyramid_mesh_structure() {
        let mesh = pyramid_mesh([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(mesh.positions.len(), 16);
        assert_eq!(mesh.indices.len(), 18);
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());
    }

    #[test]
    fn test_pyramid_mesh_tip_position() {
        let mesh = pyramid_mesh([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

        assert_eq!(mesh.positions[6], [0.0, 1.0, 0.0]);
        assert_eq!(mesh.positions[9], [0.0, 1.0, 0.0]);
        assert_eq!(mesh.positions[12], [0.0, 1.0, 0.0]);
        assert_eq!(mesh.positions[15], [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_pyramid_mesh_colors() {
        let base_color = [1.0, 0.0, 0.0];
        let tip_color = [0.0, 1.0, 0.0];
        let mesh = pyramid_mesh(base_color, tip_color);
        let colors = mesh.colors.as_ref().unwrap();

        for i in 0..4 {
            assert_eq!(colors[i], base_color);
        }

        assert_eq!(colors[6], tip_color);
        assert_eq!(colors[9], tip_color);
        assert_eq!(colors[12], tip_color);
        assert_eq!(colors[15], tip_color);
    }

    #[test]
    fn test_sphere_mesh_structure() {
        let mesh = sphere_mesh(1.0, 36, 18, [1.0, 1.0, 1.0]);

        let expected_vertices = (18 + 1) * (36 + 1);
        assert_eq!(mesh.positions.len(), expected_vertices);
        assert!(mesh.colors.is_some());
        assert!(mesh.normals.is_some());
        assert!(mesh.uvs.is_some());
    }

    #[test]
    fn test_sphere_mesh_radius() {
        let radius = 2.5;
        let mesh = sphere_mesh(radius, 36, 18, [1.0, 1.0, 1.0]);

        for pos in &mesh.positions {
            let length = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
            assert!((length - radius).abs() < 0.01);
        }
    }

    #[test]
    fn test_sphere_mesh_normals() {
        let mesh = sphere_mesh(1.0, 36, 18, [1.0, 1.0, 1.0]);
        let normals = mesh.normals.as_ref().unwrap();

        for normal in normals {
            let length =
                (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
            assert!((length - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_sphere_mesh_uvs() {
        let mesh = sphere_mesh(1.0, 36, 18, [1.0, 1.0, 1.0]);
        let uvs = mesh.uvs.as_ref().unwrap();

        for uv in uvs {
            assert!(uv[0] >= 0.0 && uv[0] <= 1.0);
            assert!(uv[1] >= 0.0 && uv[1] <= 1.0);
        }
    }

    #[test]
    fn test_sphere_mesh_small_detail() {
        let mesh = sphere_mesh(1.0, 8, 4, [1.0, 1.0, 1.0]);
        let expected_vertices = (4 + 1) * (8 + 1);
        assert_eq!(mesh.positions.len(), expected_vertices);
    }

    #[test]
    fn test_sphere_mesh_color() {
        let color = [0.3, 0.7, 0.9];
        let mesh = sphere_mesh(1.0, 36, 18, color);
        let colors = mesh.colors.as_ref().unwrap();

        for c in colors {
            assert_eq!(*c, color);
        }
    }

    #[test]
    fn test_mesh_indices_form_triangles() {
        let meshes = vec![
            colored_cube_mesh(),
            solid_cube_mesh([1.0, 1.0, 1.0]),
            textured_cube_mesh([1.0, 1.0, 1.0]),
            quad_mesh(1.0, [1.0, 1.0, 1.0]),
            textured_quad_mesh(1.0, [1.0, 1.0, 1.0]),
            pyramid_mesh([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            sphere_mesh(1.0, 36, 18, [1.0, 1.0, 1.0]),
        ];

        for mesh in meshes {
            assert_eq!(mesh.indices.len() % 3, 0);
        }
    }

    #[test]
    fn test_colored_cube_vertices_conversion() {
        let mesh = colored_cube_mesh();
        let vertices = mesh.to_vertices();
        assert_eq!(vertices.len(), 24);
    }

    #[test]
    fn test_cube_positions_within_bounds() {
        let mesh = colored_cube_mesh();
        for pos in &mesh.positions {
            assert!(pos[0].abs() <= 0.5);
            assert!(pos[1].abs() <= 0.5);
            assert!(pos[2].abs() <= 0.5);
        }
    }

    #[test]
    fn test_quad_on_y_plane() {
        let mesh = quad_mesh(2.0, [1.0, 1.0, 1.0]);
        for pos in &mesh.positions {
            assert_eq!(pos[1], 0.0);
        }
    }

    #[test]
    fn test_pyramid_base_at_y_zero() {
        let mesh = pyramid_mesh([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        for i in 0..4 {
            assert_eq!(mesh.positions[i][1], 0.0);
        }
    }
}
