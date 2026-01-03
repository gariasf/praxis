//! Asset loader traits and implementations.
//!
//! This module provides the core asset loading functionality, including
//! generic traits for loading assets and specific implementations for
//! different file formats.

use praxis_graphics::MeshData;
use praxis_utils::{debug, eyre, info, Result};
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
