//! Asset loader traits and implementations.
//!
//! This module provides the core asset loading functionality, including
//! generic traits for loading assets and specific implementations for
//! different file formats.

use praxis_graphics::MeshData;
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{AnimationClip, Bone, Skeleton};
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

        // Validate that we have actual vertex data
        if positions.is_empty() {
            return Err(eyre::eyre!(
                "OBJ file '{}' contains no vertex data",
                path.display()
            ));
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

/// Skeletal animation data from GLTF.
///
/// Contains keyframe animation data for bone transforms.
#[derive(Debug, Clone)]
pub struct GltfAnimation {
    /// Animation name from GLTF file (if present).
    pub name: Option<String>,
    /// Duration of the animation in seconds.
    pub duration: f32,
    /// Animation clip data ready for use with the engine's animation system.
    pub clip: praxis_scene::AnimationClip,
}

/// Skin data from GLTF.
///
/// Contains skeleton hierarchy and inverse bind matrices for skinning.
#[derive(Debug, Clone)]
pub struct GltfSkin {
    /// Skin name from GLTF file (if present).
    pub name: Option<String>,
    /// Skeleton component ready for use with the engine's animation system.
    pub skeleton: praxis_scene::Skeleton,
    /// Node indices that are joints in this skin (for bone mapping).
    pub joint_nodes: Vec<usize>,
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
    /// Skeletal animation data.
    pub animations: Vec<GltfAnimation>,
    /// Skin/skeleton data for skeletal animation.
    pub skins: Vec<GltfSkin>,
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
        fn traverse_node<F>(asset: &GltfAsset, node_index: usize, depth: usize, f: &mut F)
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

    /// Gets an animation by name.
    ///
    /// # Returns
    ///
    /// The animation if found, or None if no animation with that name exists.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfLoader;
    ///
    /// let loader = GltfLoader::new();
    /// let asset = loader.load_gltf("assets/models/character.gltf")?;
    ///
    /// if let Some(walk_anim) = asset.find_animation("Walk") {
    ///     println!("Walk animation duration: {}", walk_anim.duration);
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn find_animation(&self, name: &str) -> Option<&GltfAnimation> {
        self.animations
            .iter()
            .find(|anim| anim.name.as_ref().is_some_and(|n| n == name))
    }

    /// Gets a skin by name.
    ///
    /// # Returns
    ///
    /// The skin if found, or None if no skin with that name exists.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_assets::GltfLoader;
    ///
    /// let loader = GltfLoader::new();
    /// let asset = loader.load_gltf("assets/models/character.gltf")?;
    ///
    /// if let Some(skin) = asset.find_skin("CharacterSkin") {
    ///     println!("Skeleton has {} bones", skin.skeleton.bone_count());
    /// }
    /// # Ok::<(), praxis_utils::eyre::Report>(())
    /// ```
    pub fn find_skin(&self, name: &str) -> Option<&GltfSkin> {
        self.skins
            .iter()
            .find(|skin| skin.name.as_ref().is_some_and(|n| n == name))
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
/// - Skeletal animations with keyframe interpolation
/// - Skins/skeletons with bone hierarchies
///
/// # Limitations
///
/// - Morph targets not supported
/// - Only triangulated meshes (non-triangle primitives ignored)
/// - Maximum 65536 vertices per mesh (u16 index limit)
/// - Animation interpolation modes other than linear are treated as linear
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
/// println!("Loaded {} animations", asset.animations.len());
/// println!("Loaded {} skins", asset.skins.len());
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

            let base_color_texture_index =
                pbr.base_color_texture().map(|info| info.texture().index());

            let normal_texture_index = material.normal_texture().map(|info| info.texture().index());

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

        // Load skins and skeletons
        debug!("Processing {} skins", document.skins().len());
        let mut skins = Vec::new();

        for skin in document.skins() {
            let gltf_skin = Self::load_skin(&skin, &buffers, &node_map)?;
            skins.push(gltf_skin);
        }

        // Load animations
        debug!("Processing {} animations", document.animations().len());
        let mut animations = Vec::new();

        for animation in document.animations() {
            let gltf_animation = Self::load_animation(&animation, &buffers, &node_map, &skins)?;
            animations.push(gltf_animation);
        }

        info!(
            "Successfully loaded GLTF from {} ({} meshes, {} materials, {} textures, {} nodes, {} skins, {} animations)",
            path.display(),
            meshes.len(),
            materials.len(),
            textures.len(),
            nodes.len(),
            skins.len(),
            animations.len()
        );

        Ok(GltfAsset {
            meshes,
            materials,
            textures,
            nodes,
            root_nodes,
            animations,
            skins,
        })
    }

    /// Loads a skin from GLTF data.
    fn load_skin(
        skin: &gltf::Skin,
        buffers: &[gltf::buffer::Data],
        node_map: &HashMap<usize, usize>,
    ) -> Result<GltfSkin> {
        let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));

        // Get inverse bind matrices
        let _inverse_bind_matrices: Vec<Mat4> =
            if let Some(matrices) = reader.read_inverse_bind_matrices() {
                matrices.map(|m| Mat4::from_cols_array_2d(&m)).collect()
            } else {
                // If no inverse bind matrices provided, use identity matrices
                vec![Mat4::IDENTITY; skin.joints().len()]
            };

        // Build bone hierarchy
        let joint_nodes: Vec<usize> = skin
            .joints()
            .map(|joint| {
                *node_map
                    .get(&joint.index())
                    .expect("Joint node should be in node_map")
            })
            .collect();

        // Create a mapping from GLTF node index to bone index in our skeleton
        let mut gltf_node_to_bone: HashMap<usize, usize> = HashMap::new();
        for (bone_idx, joint) in skin.joints().enumerate() {
            gltf_node_to_bone.insert(joint.index(), bone_idx);
        }

        // First pass: collect all joints and their GLTF parents
        let joints: Vec<gltf::Node> = skin.joints().collect();
        let mut parent_relationships: Vec<Option<usize>> = vec![None; joints.len()];

        // Build a map of all nodes to their parents by traversing the scene
        let mut node_parents: HashMap<usize, usize> = HashMap::new();
        for joint in &joints {
            for child in joint.children() {
                node_parents.insert(child.index(), joint.index());
            }
        }

        // Determine parent bone index for each joint
        for (bone_idx, joint) in joints.iter().enumerate() {
            // Look up this joint's parent node in GLTF
            if let Some(&parent_gltf_index) = node_parents.get(&joint.index()) {
                // Check if the parent is also a joint in this skin
                if let Some(&parent_bone_idx) = gltf_node_to_bone.get(&parent_gltf_index) {
                    parent_relationships[bone_idx] = Some(parent_bone_idx);
                }
            }
        }

        // Build bones with proper parent relationships
        let mut bones = Vec::new();
        for (bone_idx, joint) in joints.iter().enumerate() {
            // Get node transform
            let transform = Mat4::from_cols_array_2d(&joint.transform().matrix());
            let (translation, rotation, scale) = transform.to_scale_rotation_translation();

            let bone = Bone::with_bind_pose(
                joint.name().unwrap_or("Unnamed").to_string(),
                parent_relationships[bone_idx],
                translation,
                rotation,
                scale,
            );

            bones.push(bone);
        }

        // Create skeleton
        let skeleton = Skeleton::new(bones);

        Ok(GltfSkin {
            name: skin.name().map(String::from),
            skeleton,
            joint_nodes,
        })
    }

    /// Loads an animation from GLTF data.
    fn load_animation(
        animation: &gltf::Animation,
        buffers: &[gltf::buffer::Data],
        node_map: &HashMap<usize, usize>,
        skins: &[GltfSkin],
    ) -> Result<GltfAnimation> {
        // Type alias for bone track data: (translations, rotations, scales)
        type BoneTrackData = (Vec<(f32, Vec3)>, Vec<(f32, Quat)>, Vec<(f32, Vec3)>);

        let mut max_time = 0.0f32;
        let mut bone_tracks: HashMap<usize, BoneTrackData> = HashMap::new();

        // Process all animation channels
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
            let target = channel.target();
            let target_node_index = *node_map
                .get(&target.node().index())
                .ok_or_else(|| eyre::eyre!("Animation target node not found in node map"))?;

            // Find which bone this node corresponds to (if any)
            let mut bone_index: Option<usize> = None;
            for skin in skins {
                if let Some(idx) = skin
                    .joint_nodes
                    .iter()
                    .position(|&n| n == target_node_index)
                {
                    bone_index = Some(idx);
                    break;
                }
            }

            // If this node isn't part of a skeleton, skip it
            let bone_index = match bone_index {
                Some(idx) => idx,
                None => continue,
            };

            // Read input timestamps
            let inputs: Vec<f32> = reader
                .read_inputs()
                .ok_or_else(|| eyre::eyre!("Animation channel missing input timestamps"))?
                .collect();

            // Update max time
            if let Some(&last_time) = inputs.last() {
                max_time = max_time.max(last_time);
            }

            // Get or create bone track entry
            let track = bone_tracks
                .entry(bone_index)
                .or_insert_with(|| (Vec::new(), Vec::new(), Vec::new()));

            // Read outputs based on property type
            match target.property() {
                gltf::animation::Property::Translation => {
                    if let Some(outputs) = reader.read_outputs() {
                        match outputs {
                            gltf::animation::util::ReadOutputs::Translations(translations) => {
                                for (time, translation) in inputs.iter().zip(translations) {
                                    track.0.push((*time, Vec3::from(translation)));
                                }
                            }
                            _ => return Err(eyre::eyre!("Unexpected output type for translation")),
                        }
                    }
                }
                gltf::animation::Property::Rotation => {
                    if let Some(outputs) = reader.read_outputs() {
                        match outputs {
                            gltf::animation::util::ReadOutputs::Rotations(rotations) => {
                                for (time, rotation) in inputs.iter().zip(rotations.into_f32()) {
                                    let quat = Quat::from_array(rotation);
                                    track.1.push((*time, quat));
                                }
                            }
                            _ => return Err(eyre::eyre!("Unexpected output type for rotation")),
                        }
                    }
                }
                gltf::animation::Property::Scale => {
                    if let Some(outputs) = reader.read_outputs() {
                        match outputs {
                            gltf::animation::util::ReadOutputs::Scales(scales) => {
                                for (time, scale) in inputs.iter().zip(scales) {
                                    track.2.push((*time, Vec3::from(scale)));
                                }
                            }
                            _ => return Err(eyre::eyre!("Unexpected output type for scale")),
                        }
                    }
                }
                gltf::animation::Property::MorphTargetWeights => {
                    // Morph targets not supported yet, skip
                    continue;
                }
            }
        }

        // Create animation clip
        let name = animation.name().unwrap_or("Unnamed");
        let mut clip = AnimationClip::new(name.to_string(), max_time);

        // Add all bone tracks to the clip
        for (bone_index, (translations, rotations, scales)) in bone_tracks {
            // Add translation keyframes
            for (time, translation) in translations {
                clip.add_translation_keyframe(bone_index, time, translation);
            }

            // Add rotation keyframes
            for (time, rotation) in rotations {
                clip.add_rotation_keyframe(bone_index, time, rotation);
            }

            // Add scale keyframes
            for (time, scale) in scales {
                clip.add_scale_keyframe(bone_index, time, scale);
            }
        }

        Ok(GltfAnimation {
            name: animation.name().map(String::from),
            duration: max_time,
            clip,
        })
    }
}

impl Default for GltfLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod gltf_tests {
    use super::*;

    #[test]
    fn test_gltf_node_has_mesh() {
        let node_with_mesh = GltfNode {
            name: Some("MeshNode".to_string()),
            transform: Mat4::IDENTITY,
            mesh_indices: vec![0, 1],
            children: vec![],
        };
        assert!(node_with_mesh.has_mesh());

        let node_without_mesh = GltfNode {
            name: Some("EmptyNode".to_string()),
            transform: Mat4::IDENTITY,
            mesh_indices: vec![],
            children: vec![],
        };
        assert!(!node_without_mesh.has_mesh());
    }

    #[test]
    fn test_gltf_node_decompose_transform_identity() {
        let node = GltfNode {
            name: None,
            transform: Mat4::IDENTITY,
            mesh_indices: vec![],
            children: vec![],
        };

        let (translation, rotation, scale) = node.decompose_transform();
        assert!((translation.x - 0.0).abs() < 0.001);
        assert!((translation.y - 0.0).abs() < 0.001);
        assert!((translation.z - 0.0).abs() < 0.001);
        assert!((rotation.w - 1.0).abs() < 0.001);
        assert!((scale.x - 1.0).abs() < 0.001);
        assert!((scale.y - 1.0).abs() < 0.001);
        assert!((scale.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gltf_node_decompose_transform_translation() {
        let transform = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
        let node = GltfNode {
            name: None,
            transform,
            mesh_indices: vec![],
            children: vec![],
        };

        let (translation, _rotation, _scale) = node.decompose_transform();
        assert!((translation.x - 10.0).abs() < 0.001);
        assert!((translation.y - 20.0).abs() < 0.001);
        assert!((translation.z - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_gltf_node_decompose_transform_scale() {
        let transform = Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let node = GltfNode {
            name: None,
            transform,
            mesh_indices: vec![],
            children: vec![],
        };

        let (_translation, _rotation, scale) = node.decompose_transform();
        assert!((scale.x - 2.0).abs() < 0.001);
        assert!((scale.y - 3.0).abs() < 0.001);
        assert!((scale.z - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_gltf_node_decompose_transform_rotation() {
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let transform = Mat4::from_quat(rotation);
        let node = GltfNode {
            name: None,
            transform,
            mesh_indices: vec![],
            children: vec![],
        };

        let (_translation, extracted_rotation, _scale) = node.decompose_transform();
        assert!((extracted_rotation.x - rotation.x).abs() < 0.001);
        assert!((extracted_rotation.y - rotation.y).abs() < 0.001);
        assert!((extracted_rotation.z - rotation.z).abs() < 0.001);
        assert!((extracted_rotation.w - rotation.w).abs() < 0.001);
    }

    #[test]
    fn test_gltf_node_children() {
        let node = GltfNode {
            name: Some("Parent".to_string()),
            transform: Mat4::IDENTITY,
            mesh_indices: vec![],
            children: vec![1, 2, 3],
        };

        assert_eq!(node.children.len(), 3);
        assert_eq!(node.children[0], 1);
        assert_eq!(node.children[1], 2);
        assert_eq!(node.children[2], 3);
    }

    #[test]
    fn test_gltf_material_default() {
        let material = GltfMaterial::default();

        assert_eq!(material.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(material.metallic, 0.0);
        assert_eq!(material.roughness, 0.5);
        assert!(material.base_color_texture_index.is_none());
        assert!(material.normal_texture_index.is_none());
        assert!(material.name.is_none());
    }

    #[test]
    fn test_gltf_material_to_material_properties() {
        let material = GltfMaterial {
            name: Some("TestMaterial".to_string()),
            base_color: [0.8, 0.2, 0.1, 1.0],
            metallic: 0.7,
            roughness: 0.3,
            base_color_texture_index: Some(0),
            normal_texture_index: Some(1),
        };

        let props = material.to_material_properties();
        assert_eq!(props.base_color, [0.8, 0.2, 0.1, 1.0]);
        assert_eq!(props.metallic, 0.7);
        assert_eq!(props.roughness, 0.3);
    }

    #[test]
    fn test_gltf_texture_format() {
        assert_eq!(GltfTextureFormat::R8G8B8A8, GltfTextureFormat::R8G8B8A8);
        assert_eq!(GltfTextureFormat::R8G8B8, GltfTextureFormat::R8G8B8);
        assert_ne!(GltfTextureFormat::R8G8B8A8, GltfTextureFormat::R8G8B8);
    }

    #[test]
    fn test_gltf_texture_creation() {
        let texture = GltfTexture {
            data: vec![255, 0, 0, 255],
            width: 1,
            height: 1,
            format: GltfTextureFormat::R8G8B8A8,
        };

        assert_eq!(texture.data.len(), 4);
        assert_eq!(texture.width, 1);
        assert_eq!(texture.height, 1);
        assert_eq!(texture.format, GltfTextureFormat::R8G8B8A8);
    }

    #[test]
    fn test_gltf_asset_nodes_with_meshes() {
        let nodes = vec![
            GltfNode {
                name: Some("Node0".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![0],
                children: vec![],
            },
            GltfNode {
                name: Some("Node1".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
            GltfNode {
                name: Some("Node2".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![1, 2],
                children: vec![],
            },
        ];

        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes,
            root_nodes: vec![0, 1, 2],
            animations: vec![],
            skins: vec![],
        };

        let nodes_with_meshes: Vec<_> = asset.nodes_with_meshes().collect();
        assert_eq!(nodes_with_meshes.len(), 2);
        assert_eq!(nodes_with_meshes[0].0, 0);
        assert_eq!(nodes_with_meshes[1].0, 2);
    }

    #[test]
    fn test_gltf_asset_traverse_depth_first_single_level() {
        let nodes = vec![GltfNode {
            name: Some("Root".to_string()),
            transform: Mat4::IDENTITY,
            mesh_indices: vec![],
            children: vec![],
        }];

        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes,
            root_nodes: vec![0],
            animations: vec![],
            skins: vec![],
        };

        let mut visited = Vec::new();
        asset.traverse_depth_first(|index, _node, depth| {
            visited.push((index, depth));
        });

        assert_eq!(visited.len(), 1);
        assert_eq!(visited[0], (0, 0));
    }

    #[test]
    fn test_gltf_asset_traverse_depth_first_hierarchy() {
        let nodes = vec![
            GltfNode {
                name: Some("Root".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![1, 2],
            },
            GltfNode {
                name: Some("Child1".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![3],
            },
            GltfNode {
                name: Some("Child2".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
            GltfNode {
                name: Some("GrandChild".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
        ];

        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes,
            root_nodes: vec![0],
            animations: vec![],
            skins: vec![],
        };

        let mut visited = Vec::new();
        asset.traverse_depth_first(|index, _node, depth| {
            visited.push((index, depth));
        });

        assert_eq!(visited.len(), 4);
        assert_eq!(visited[0], (0, 0));
        assert_eq!(visited[1], (1, 1));
        assert_eq!(visited[2], (3, 2));
        assert_eq!(visited[3], (2, 1));
    }

    #[test]
    fn test_gltf_asset_traverse_depth_first_multiple_roots() {
        let nodes = vec![
            GltfNode {
                name: Some("Root1".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![2],
            },
            GltfNode {
                name: Some("Root2".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![3],
            },
            GltfNode {
                name: Some("Child1".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
            GltfNode {
                name: Some("Child2".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
        ];

        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes,
            root_nodes: vec![0, 1],
            animations: vec![],
            skins: vec![],
        };

        let mut visited = Vec::new();
        asset.traverse_depth_first(|index, _node, depth| {
            visited.push((index, depth));
        });

        assert_eq!(visited.len(), 4);
        assert_eq!(visited[0], (0, 0));
        assert_eq!(visited[1], (2, 1));
        assert_eq!(visited[2], (1, 0));
        assert_eq!(visited[3], (3, 1));
    }

    #[test]
    fn test_gltf_asset_traverse_depth_first_deep_hierarchy() {
        let nodes = vec![
            GltfNode {
                name: Some("Level0".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![1],
            },
            GltfNode {
                name: Some("Level1".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![2],
            },
            GltfNode {
                name: Some("Level2".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![3],
            },
            GltfNode {
                name: Some("Level3".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
        ];

        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes,
            root_nodes: vec![0],
            animations: vec![],
            skins: vec![],
        };

        let mut max_depth = 0;
        asset.traverse_depth_first(|_index, _node, depth| {
            max_depth = max_depth.max(depth);
        });

        assert_eq!(max_depth, 3);
    }

    #[test]
    fn test_gltf_asset_traverse_with_node_names() {
        let nodes = vec![
            GltfNode {
                name: Some("Parent".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![1, 2],
            },
            GltfNode {
                name: Some("LeftChild".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
            GltfNode {
                name: Some("RightChild".to_string()),
                transform: Mat4::IDENTITY,
                mesh_indices: vec![],
                children: vec![],
            },
        ];

        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes,
            root_nodes: vec![0],
            animations: vec![],
            skins: vec![],
        };

        let mut names = Vec::new();
        asset.traverse_depth_first(|_index, node, _depth| {
            if let Some(name) = &node.name {
                names.push(name.clone());
            }
        });

        assert_eq!(names.len(), 3);
        assert_eq!(names[0], "Parent");
        assert_eq!(names[1], "LeftChild");
        assert_eq!(names[2], "RightChild");
    }

    #[test]
    fn test_gltf_loader_creation() {
        let _loader = GltfLoader::new();
        let _default_loader = GltfLoader::default();
    }

    #[test]
    fn test_gltf_node_multiple_mesh_indices() {
        let node = GltfNode {
            name: Some("MultiMesh".to_string()),
            transform: Mat4::IDENTITY,
            mesh_indices: vec![0, 1, 2, 3],
            children: vec![],
        };

        assert!(node.has_mesh());
        assert_eq!(node.mesh_indices.len(), 4);
    }

    #[test]
    fn test_gltf_material_with_textures() {
        let material = GltfMaterial {
            name: Some("TexturedMaterial".to_string()),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 1.0,
            roughness: 0.0,
            base_color_texture_index: Some(5),
            normal_texture_index: Some(7),
        };

        assert_eq!(material.base_color_texture_index, Some(5));
        assert_eq!(material.normal_texture_index, Some(7));
    }

    #[test]
    fn test_gltf_asset_empty() {
        let asset = GltfAsset {
            meshes: vec![],
            materials: vec![],
            textures: vec![],
            nodes: vec![],
            root_nodes: vec![],
            animations: vec![],
            skins: vec![],
        };

        assert_eq!(asset.meshes.len(), 0);
        assert_eq!(asset.materials.len(), 0);
        assert_eq!(asset.textures.len(), 0);
        assert_eq!(asset.nodes.len(), 0);
        assert_eq!(asset.root_nodes.len(), 0);
        assert_eq!(asset.animations.len(), 0);
        assert_eq!(asset.skins.len(), 0);

        let nodes_with_meshes: Vec<_> = asset.nodes_with_meshes().collect();
        assert_eq!(nodes_with_meshes.len(), 0);
    }

    #[test]
    fn test_gltf_node_transform_combined() {
        let translation = Vec3::new(5.0, 10.0, 15.0);
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let scale = Vec3::new(2.0, 3.0, 4.0);
        let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);

        let node = GltfNode {
            name: None,
            transform,
            mesh_indices: vec![],
            children: vec![],
        };

        let (extracted_translation, _extracted_rotation, extracted_scale) =
            node.decompose_transform();

        assert!((extracted_translation.x - translation.x).abs() < 0.01);
        assert!((extracted_translation.y - translation.y).abs() < 0.01);
        assert!((extracted_translation.z - translation.z).abs() < 0.01);

        assert!((extracted_scale.x - scale.x).abs() < 0.01);
        assert!((extracted_scale.y - scale.y).abs() < 0.01);
        assert!((extracted_scale.z - scale.z).abs() < 0.01);
    }
}
