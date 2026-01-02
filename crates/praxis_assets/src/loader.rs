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
pub struct MeshLoader {
    /// Configuration options for the loader
    _config: MeshLoaderConfig,
}

/// Configuration options for mesh loading.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MeshLoaderConfig {
    /// Whether to load normals from the OBJ file
    load_normals: bool,
    /// Whether to load texture coordinates from the OBJ file
    load_uvs: bool,
}

impl Default for MeshLoaderConfig {
    fn default() -> Self {
        Self {
            load_normals: true,
            load_uvs: true,
        }
    }
}

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
        Self {
            _config: MeshLoaderConfig::default(),
        }
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
}
