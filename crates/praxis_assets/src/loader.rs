//! Asset loader traits and implementations.
//!
//! This module provides the core asset loading functionality, including
//! generic traits for loading assets and specific implementations for
//! different file formats.

use praxis_graphics::MeshData;
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::{debug, eyre, info, Result};
use std::collections::HashMap;
use std::path::Path;

/// Generic trait for loading assets from files.
///
/// This trait defines a common interface for loading different types of assets.
/// Implementations handle format-specific parsing and conversion to engine types.
///
/// # Type Parameters
///
/// * `T` - The output type produced by this loader (e.g., `MeshData`, `Texture`, etc.)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::{AssetLoader, MeshLoader};
/// use praxis_graphics::MeshData;
///
/// fn load_asset<L: AssetLoader<MeshData>>(loader: &L, path: &str) -> praxis_utils::Result<MeshData> {
///     loader.load(path)
/// }
///
/// let loader = MeshLoader::new();
/// let mesh = load_asset(&loader, "model.obj")?;
/// # Ok::<(), praxis_utils::eyre::Report>(())
/// ```
pub trait AssetLoader<T> {
    /// Loads an asset from a file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the asset file to load
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist or cannot be read
    /// - The file format is invalid or unsupported
    /// - Parsing fails for any reason
    fn load(&self, path: impl AsRef<Path>) -> Result<T>;

    /// Returns a list of file extensions this loader supports.
    ///
    /// # Example
    ///
    /// ```rust
    /// use praxis_assets::{AssetLoader, MeshLoader};
    ///
    /// let loader = MeshLoader::new();
    /// assert!(loader.supported_extensions().contains(&"obj"));
    /// ```
    fn supported_extensions(&self) -> &[&str];
}

/// OBJ file loader for 3D meshes.
///
/// This loader parses Wavefront OBJ files and converts them to `MeshData`
/// that can be uploaded to the GPU via `MeshAssetManager`.
///
/// # Supported Features
///
/// - Vertex positions (required)
/// - Vertex normals (optional)
/// - Texture coordinates (optional)
/// - Triangulated faces
/// - Multiple models/objects per file (merged into single mesh)
///
/// # Limitations
///
/// - Only triangulated meshes are supported (faces with 3 vertices)
/// - Materials are not loaded (MTL files are ignored)
/// - All models in a file are merged into a single mesh
/// - All models must have consistent attributes (all with or without normals/UVs)
/// - Vertex colors are not supported in OBJ format
/// - Maximum 65536 vertices in merged mesh (u16 index limit)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::{MeshLoader, AssetLoader};
///
/// let loader = MeshLoader::new();
/// let mesh = loader.load("assets/models/cube.obj")?;
///
/// println!("Loaded {} vertices", mesh.positions.len());
/// # Ok::<(), praxis_utils::eyre::Report>(())
/// ```
pub struct MeshLoader {}

impl MeshLoader {
    /// Creates a new mesh loader with default configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use praxis_assets::MeshLoader;
    ///
    /// let loader = MeshLoader::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MeshLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLoader<MeshData> for MeshLoader {
    fn load(&self, path: impl AsRef<Path>) -> Result<MeshData> {
        let path = path.as_ref();
        info!("Loading mesh from: {}", path.display());

        // Load the OBJ file using tobj
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to load OBJ file '{}': {}", path.display(), e))?;

        if models.is_empty() {
            return Err(eyre::eyre!(
                "OBJ file '{}' contains no models",
                path.display()
            ));
        }

        if models.len() > 1 {
            info!(
                "OBJ file '{}' contains {} models, merging into single mesh",
                path.display(),
                models.len()
            );
        }

        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut has_normals = false;
        let mut has_uvs = false;

        for model in &models {
            let mesh = &model.mesh;
            let vertex_offset = positions.len() as u32;

            debug!(
                "Processing model '{}' with {} vertices, {} indices",
                model.name,
                mesh.positions.len() / 3,
                mesh.indices.len()
            );

            positions.extend(
                mesh.positions
                    .chunks_exact(3)
                    .map(|chunk| [chunk[0], chunk[1], chunk[2]]),
            );

            for &i in &mesh.indices {
                let adjusted_index = i + vertex_offset;
                if adjusted_index > u16::MAX as u32 {
                    return Err(eyre::eyre!(
                        "Merged mesh has too many vertices for u16 indices (vertex index: {})",
                        adjusted_index
                    ));
                }
                indices.push(adjusted_index as u16);
            }

            if !mesh.normals.is_empty() {
                has_normals = true;
                normals.extend(
                    mesh.normals
                        .chunks_exact(3)
                        .map(|chunk| [chunk[0], chunk[1], chunk[2]]),
                );
            } else if has_normals {
                return Err(eyre::eyre!(
                    "Model '{}' is missing normals while previous models had them. All models must have consistent attributes.",
                    model.name
                ));
            }

            if !mesh.texcoords.is_empty() {
                has_uvs = true;
                uvs.extend(
                    mesh.texcoords
                        .chunks_exact(2)
                        .map(|chunk| [chunk[0], chunk[1]]),
                );
            } else if has_uvs {
                return Err(eyre::eyre!(
                    "Model '{}' is missing texture coordinates while previous models had them. All models must have consistent attributes.",
                    model.name
                ));
            }
        }

        info!(
            "Successfully loaded {} model(s) from {} ({} total vertices, {} total indices)",
            models.len(),
            path.display(),
            positions.len(),
            indices.len()
        );

        Ok(MeshData {
            positions,
            colors: None,
            normals: if has_normals { Some(normals) } else { None },
            uvs: if has_uvs { Some(uvs) } else { None },
            tangents: None,
            indices,
        })
    }

    fn supported_extensions(&self) -> &[&str] {
        &["obj"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_mesh_loader_creation() {
        let loader = MeshLoader::new();
        assert!(loader.supported_extensions().contains(&"obj"));
    }

    #[test]
    fn test_mesh_loader_default() {
        let _loader = MeshLoader::default();
    }

    #[test]
    fn test_supported_extensions() {
        let loader = MeshLoader::new();
        let extensions = loader.supported_extensions();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0], "obj");
    }

    #[test]
    fn test_load_simple_triangle() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_triangle.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Failed to load simple triangle");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 3, "Should have 3 vertices");
        assert_eq!(mesh.indices.len(), 3, "Should have 3 indices");
        assert!(mesh.normals.is_none(), "Should not have normals");
        assert!(mesh.uvs.is_none(), "Should not have UVs");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_mesh_with_normals() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
f 1//1 2//2 3//3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_normals.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Failed to load mesh with normals");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 3);
        assert!(mesh.normals.is_some(), "Should have normals");
        let normals = mesh.normals.unwrap();
        assert_eq!(normals.len(), 3);

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_mesh_with_uvs() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
f 1/1 2/2 3/3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_uvs.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Failed to load mesh with UVs");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 3);
        assert!(mesh.uvs.is_some(), "Should have UVs");
        let uvs = mesh.uvs.unwrap();
        assert_eq!(uvs.len(), 3);

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_mesh_with_normals_and_uvs() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/2 3/3/3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_complete.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Failed to load mesh with normals and UVs");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 3);
        assert!(mesh.normals.is_some(), "Should have normals");
        assert!(mesh.uvs.is_some(), "Should have UVs");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_nonexistent_file() {
        let loader = MeshLoader::new();
        let result = loader.load("nonexistent_file_12345.obj");
        assert!(result.is_err(), "Should fail to load nonexistent file");
    }

    #[test]
    fn test_load_empty_obj_file() {
        let loader = MeshLoader::new();
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_empty.obj");
        fs::write(&test_file, "").expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_err(), "Should fail to load empty OBJ file");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_quad_mesh_triangulated() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v -1.0 -1.0 0.0
v  1.0 -1.0 0.0
v  1.0  1.0 0.0
v -1.0  1.0 0.0
f 1 2 3 4
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_quad.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Should load quad and triangulate");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 4);
        assert_eq!(
            mesh.indices.len(),
            6,
            "Quad should be triangulated to 6 indices"
        );

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_multiple_models_merged() {
        let loader = MeshLoader::new();
        let obj_content = r#"
o Model1
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3

o Model2
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.5 1.0 0.0
f 4 5 6
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_multi.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Should load and merge multiple models");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 6, "Should have 6 total vertices");
        assert_eq!(mesh.indices.len(), 6, "Should have 6 total indices");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_inconsistent_normals() {
        let loader = MeshLoader::new();
        let obj_content = r#"
o Model1
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
f 1//1 2//2 3//3

o Model2
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.5 1.0 0.0
f 4 5 6
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_inconsistent_normals.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_err(), "Should fail with inconsistent normals");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("missing normals") || error_msg.contains("consistent attributes")
        );

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_inconsistent_uvs() {
        let loader = MeshLoader::new();
        let obj_content = r#"
o Model1
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
f 1/1 2/2 3/3

o Model2
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.5 1.0 0.0
f 4 5 6
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_inconsistent_uvs.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_err(), "Should fail with inconsistent UVs");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("missing texture coordinates")
                || error_msg.contains("consistent attributes")
        );

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_vertex_data_correctness() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v 1.0 2.0 3.0
v 4.0 5.0 6.0
v 7.0 8.0 9.0
f 1 2 3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_vertex_data.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok());

        let mesh = result.unwrap();
        assert_eq!(mesh.positions[0], [1.0, 2.0, 3.0]);
        assert_eq!(mesh.positions[1], [4.0, 5.0, 6.0]);
        assert_eq!(mesh.positions[2], [7.0, 8.0, 9.0]);

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_index_offset_in_merged_models() {
        let loader = MeshLoader::new();
        let obj_content = r#"
o Model1
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3

o Model2
v 2.0 0.0 0.0
v 3.0 0.0 0.0
v 2.5 1.0 0.0
f 4 5 6
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_index_offset.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok());

        let mesh = result.unwrap();
        assert_eq!(mesh.indices[0], 0);
        assert_eq!(mesh.indices[1], 1);
        assert_eq!(mesh.indices[2], 2);
        assert_eq!(mesh.indices[3], 3);
        assert_eq!(mesh.indices[4], 4);
        assert_eq!(mesh.indices[5], 5);

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_path_as_ref_trait() {
        let loader = MeshLoader::new();
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_path_trait.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let path_str = test_file.to_str().unwrap();
        let result1 = loader.load(path_str);
        assert!(result1.is_ok(), "Should work with &str");

        let path_string = test_file.to_string_lossy().to_string();
        let result2 = loader.load(path_string);
        assert!(result2.is_ok(), "Should work with String");

        let result3 = loader.load(&test_file);
        assert!(result3.is_ok(), "Should work with &Path");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_asset_loader_trait() {
        fn generic_load<L: AssetLoader<MeshData>>(loader: &L, path: &str) -> Result<MeshData> {
            loader.load(path)
        }

        let loader = MeshLoader::new();
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_trait.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = generic_load(&loader, test_file.to_str().unwrap());
        assert!(result.is_ok(), "Should work through generic trait");

        fs::remove_file(&test_file).ok();
    }
}

/// Node in a GLTF scene hierarchy.
///
/// Represents a node in the GLTF scene graph with transform, mesh, and children.
#[derive(Debug, Clone)]
pub struct GltfNode {
    /// Node name from GLTF file (if present).
    pub name: Option<String>,
    /// Local transform matrix.
    pub transform: Mat4,
    /// Indices of mesh primitives associated with this node.
    /// GLTF meshes can have multiple primitives; each becomes a separate mesh
    /// in the meshes array. This vec contains all primitive indices for this node's mesh.
    pub mesh_indices: Vec<usize>,
    /// Indices of child nodes.
    pub children: Vec<usize>,
}

impl GltfNode {
    /// Checks if this node has any associated meshes.
    ///
    /// # Returns
    ///
    /// `true` if the node has at least one mesh primitive, `false` otherwise.
    pub fn has_mesh(&self) -> bool {
        !self.mesh_indices.is_empty()
    }

    /// Decomposes the transform matrix into translation, rotation, and scale.
    ///
    /// This is useful for converting GLTF node transforms into engine-friendly
    /// transform components.
    ///
    /// # Returns
    ///
    /// A tuple of (translation, rotation, scale) where:
    /// - translation is a Vec3
    /// - rotation is a Quat
    /// - scale is a Vec3
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfLoader;
    ///
    /// let loader = GltfLoader::new();
    /// let asset = loader.load_gltf("assets/models/scene.gltf")?;
    ///
    /// for node in &asset.nodes {
    ///     let (translation, rotation, scale) = node.decompose_transform();
    ///     println!("Node {:?}: pos={:?}, rot={:?}, scale={:?}",
    ///         node.name, translation, rotation, scale);
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn decompose_transform(&self) -> (Vec3, Quat, Vec3) {
        let (scale, rotation, translation) = self.transform.to_scale_rotation_translation();
        (translation, rotation, scale)
    }
}

/// Material data from GLTF.
///
/// Contains material properties extracted from GLTF materials.
#[derive(Debug, Clone)]
pub struct GltfMaterial {
    /// Material name from GLTF file (if present).
    pub name: Option<String>,
    /// Base color factor (RGBA).
    pub base_color: [f32; 4],
    /// Metallic factor [0.0, 1.0].
    pub metallic: f32,
    /// Roughness factor [0.0, 1.0].
    pub roughness: f32,
    /// Index of base color texture (if any).
    pub base_color_texture_index: Option<usize>,
    /// Index of normal map texture (if any).
    pub normal_texture_index: Option<usize>,
}

impl Default for GltfMaterial {
    fn default() -> Self {
        Self {
            name: None,
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            base_color_texture_index: None,
            normal_texture_index: None,
        }
    }
}

impl GltfMaterial {
    /// Converts this GLTF material to graphics material properties.
    ///
    /// This is useful for uploading material properties to the GPU for rendering.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfLoader;
    ///
    /// let loader = GltfLoader::new();
    /// let asset = loader.load_gltf("assets/models/scene.gltf")?;
    ///
    /// for material in &asset.materials {
    ///     let props = material.to_material_properties();
    ///     println!("Material: metallic={}, roughness={}",
    ///         props.metallic, props.roughness);
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn to_material_properties(&self) -> praxis_graphics::MaterialProperties {
        praxis_graphics::MaterialProperties::new()
            .with_base_color(self.base_color)
            .with_metallic(self.metallic)
            .with_roughness(self.roughness)
            .with_emissive_strength(0.0)
    }
}

/// Texture data from GLTF.
///
/// Contains raw image data and format information.
#[derive(Debug, Clone)]
pub struct GltfTexture {
    /// Image data in the format specified by `format`.
    pub data: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image format (e.g., R8G8B8A8).
    pub format: GltfTextureFormat,
}

/// Texture format for GLTF textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GltfTextureFormat {
    /// 8-bit RGBA format.
    R8G8B8A8,
    /// 8-bit RGB format.
    R8G8B8,
}

/// Complete GLTF asset data.
///
/// Contains all data loaded from a GLTF file, including meshes, materials,
/// textures, and the scene hierarchy.
#[derive(Debug, Clone)]
pub struct GltfAsset {
    /// All meshes in the GLTF file.
    pub meshes: Vec<MeshData>,
    /// All materials in the GLTF file.
    pub materials: Vec<GltfMaterial>,
    /// All textures in the GLTF file.
    pub textures: Vec<GltfTexture>,
    /// Scene graph nodes.
    pub nodes: Vec<GltfNode>,
    /// Root node indices (nodes without parents).
    pub root_nodes: Vec<usize>,
}

impl GltfAsset {
    /// Gets all nodes that have meshes.
    ///
    /// # Returns
    ///
    /// An iterator over pairs of (node_index, node) for all nodes with meshes.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfLoader;
    ///
    /// let loader = GltfLoader::new();
    /// let asset = loader.load_gltf("assets/models/scene.gltf")?;
    ///
    /// for (node_index, node) in asset.nodes_with_meshes() {
    ///     println!("Node {} has {} mesh primitives", node_index, node.mesh_indices.len());
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn nodes_with_meshes(&self) -> impl Iterator<Item = (usize, &GltfNode)> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.mesh_indices.is_empty())
    }

    /// Traverses the scene hierarchy depth-first starting from root nodes.
    ///
    /// Calls the provided function for each node in depth-first order,
    /// passing the node index, node reference, and current depth.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to call for each node (node_index, node, depth)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfLoader;
    ///
    /// let loader = GltfLoader::new();
    /// let asset = loader.load_gltf("assets/models/scene.gltf")?;
    ///
    /// asset.traverse_depth_first(|node_index, node, depth| {
    ///     let indent = "  ".repeat(depth);
    ///     println!("{}{}: {:?}", indent, node_index, node.name);
    /// });
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn traverse_depth_first<F>(&self, mut f: F)
    where
        F: FnMut(usize, &GltfNode, usize),
    {
        fn traverse_node<F>(
            asset: &GltfAsset,
            node_index: usize,
            depth: usize,
            f: &mut F,
        )
        where
            F: FnMut(usize, &GltfNode, usize),
        {
            let node = &asset.nodes[node_index];
            f(node_index, node, depth);

            for &child_index in &node.children {
                traverse_node(asset, child_index, depth + 1, f);
            }
        }

        for &root_index in &self.root_nodes {
            traverse_node(self, root_index, 0, &mut f);
        }
    }
}

/// GLTF file loader.
///
/// This loader parses GLTF/GLB files and converts them to engine-compatible data structures.
///
/// # Supported Features
///
/// - Meshes with positions, normals, UVs, and tangents
/// - Node hierarchies with transforms
/// - PBR materials (base color, metallic, roughness)
/// - Embedded and external textures
/// - Multiple primitives per mesh
/// - Multiple scenes (uses default scene)
///
/// # Limitations
///
/// - Animations are not supported
/// - Skins/skeletal animation not supported
/// - Morph targets not supported
/// - Only triangulated meshes (non-triangle primitives ignored)
/// - Maximum 65536 vertices per mesh (u16 index limit)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets::GltfLoader;
///
/// let loader = GltfLoader::new();
/// let asset = loader.load_gltf("assets/models/scene.gltf")?;
///
/// println!("Loaded {} meshes", asset.meshes.len());
/// println!("Loaded {} materials", asset.materials.len());
/// println!("Loaded {} textures", asset.textures.len());
/// # Ok::<(), praxis_utils::eyre::Report>(())
/// ```
pub struct GltfLoader {}

impl GltfLoader {
    /// Creates a new GLTF loader with default configuration.
    pub fn new() -> Self {
        Self {}
    }

    /// Loads a GLTF file and returns the complete asset data.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the GLTF or GLB file to load
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist or cannot be read
    /// - The GLTF format is invalid
    /// - Required data is missing or malformed
    pub fn load_gltf(&self, path: impl AsRef<Path>) -> Result<GltfAsset> {
        let path = path.as_ref();
        info!("Loading GLTF from: {}", path.display());

        let (document, buffers, images) = gltf::import(path)
            .map_err(|e| eyre::eyre!("Failed to load GLTF file '{}': {}", path.display(), e))?;

        let mut meshes = Vec::new();
        let mut materials = Vec::new();
        let mut textures = Vec::new();
        let mut nodes = Vec::new();

        debug!("Processing {} textures", document.textures().len());
        for texture in document.textures() {
            let image = &images[texture.source().index()];
            let gltf_texture = GltfTexture {
                data: image.pixels.clone(),
                width: image.width,
                height: image.height,
                format: match image.format {
                    gltf::image::Format::R8G8B8A8 => GltfTextureFormat::R8G8B8A8,
                    gltf::image::Format::R8G8B8 => GltfTextureFormat::R8G8B8,
                    _ => {
                        return Err(eyre::eyre!(
                            "Unsupported texture format: {:?}",
                            image.format
                        ))
                    }
                },
            };
            textures.push(gltf_texture);
        }

        debug!("Processing {} materials", document.materials().len());
        for material in document.materials() {
            let pbr = material.pbr_metallic_roughness();
            let base_color = pbr.base_color_factor();
            let metallic = pbr.metallic_factor();
            let roughness = pbr.roughness_factor();

            let base_color_texture_index = pbr
                .base_color_texture()
                .map(|info| info.texture().index());

            let normal_texture_index = material
                .normal_texture()
                .map(|info| info.texture().index());

            let gltf_material = GltfMaterial {
                name: material.name().map(String::from),
                base_color,
                metallic,
                roughness,
                base_color_texture_index,
                normal_texture_index,
            };
            materials.push(gltf_material);
        }

        debug!("Processing {} meshes", document.meshes().len());
        let mut mesh_primitive_map: HashMap<usize, Vec<usize>> = HashMap::new();
        
        for mesh in document.meshes() {
            let mut primitive_indices = Vec::new();
            
            for primitive in mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    debug!(
                        "Skipping non-triangle primitive in mesh '{}'",
                        mesh.name().unwrap_or("unnamed")
                    );
                    continue;
                }

                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or_else(|| eyre::eyre!("Mesh primitive missing positions"))?
                    .collect();

                let normals: Option<Vec<[f32; 3]>> =
                    reader.read_normals().map(|iter| iter.collect());

                let uvs: Option<Vec<[f32; 2]>> = reader
                    .read_tex_coords(0)
                    .map(|iter| iter.into_f32().collect());

                let tangents: Option<Vec<[f32; 4]>> =
                    reader.read_tangents().map(|iter| iter.collect());

                let indices: Vec<u16> = reader
                    .read_indices()
                    .ok_or_else(|| eyre::eyre!("Mesh primitive missing indices"))?
                    .into_u32()
                    .map(|i| {
                        if i > u16::MAX as u32 {
                            Err(eyre::eyre!("Index {} exceeds u16::MAX", i))
                        } else {
                            Ok(i as u16)
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;

                let mesh_data = MeshData {
                    positions,
                    colors: None,
                    normals,
                    uvs,
                    tangents,
                    indices,
                };

                primitive_indices.push(meshes.len());
                meshes.push(mesh_data);
            }
            
            mesh_primitive_map.insert(mesh.index(), primitive_indices);
        }

        let default_scene = document
            .default_scene()
            .or_else(|| document.scenes().next())
            .ok_or_else(|| eyre::eyre!("GLTF file contains no scenes"))?;

        debug!("Processing {} nodes", document.nodes().len());
        let mut node_map: HashMap<usize, usize> = HashMap::new();

        for (new_index, node) in document.nodes().enumerate() {
            node_map.insert(node.index(), new_index);
        }

        for node in document.nodes() {
            let transform = Mat4::from_cols_array_2d(&node.transform().matrix());

            let mesh_indices = node
                .mesh()
                .and_then(|mesh| mesh_primitive_map.get(&mesh.index()))
                .cloned()
                .unwrap_or_default();

            let children: Vec<usize> = node
                .children()
                .map(|child| {
                    *node_map
                        .get(&child.index())
                        .expect("Child node should be in node_map")
                })
                .collect();

            let gltf_node = GltfNode {
                name: node.name().map(String::from),
                transform,
                mesh_indices,
                children,
            };

            nodes.push(gltf_node);
        }

        let root_nodes: Vec<usize> = default_scene
            .nodes()
            .map(|node| {
                *node_map
                    .get(&node.index())
                    .expect("Root node should be in node_map")
            })
            .collect();

        info!(
            "Successfully loaded GLTF from {} ({} meshes, {} materials, {} textures, {} nodes)",
            path.display(),
            meshes.len(),
            materials.len(),
            textures.len(),
            nodes.len()
        );

        Ok(GltfAsset {
            meshes,
            materials,
            textures,
            nodes,
            root_nodes,
        })
    }
}

impl Default for GltfLoader {
    fn default() -> Self {
        Self::new()
    }
}
