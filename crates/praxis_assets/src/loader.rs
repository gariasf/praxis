//! Asset loader traits and implementations.
//!
//! This module provides the core asset loading functionality, including
//! generic traits for loading assets and specific implementations for
//! different file formats.

use praxis_graphics::MeshData;
use praxis_utils::{Result, eyre, info, debug};
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
///
/// # Limitations
///
/// - Only triangulated meshes are supported (faces with 3 vertices)
/// - Materials are not loaded (MTL files are ignored)
/// - Groups and objects are merged into a single mesh
/// - Vertex colors are not supported in OBJ format
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
            return Err(eyre::eyre!("OBJ file '{}' contains no models", path.display()));
        }

        // For now, we only load the first model
        // TODO: Support multiple models/meshes per file
        if models.len() > 1 {
            debug!(
                "OBJ file '{}' contains {} models, only loading the first",
                path.display(),
                models.len()
            );
        }

        let model = &models[0];
        let mesh = &model.mesh;

        debug!(
            "Loaded model '{}' with {} vertices, {} indices",
            model.name,
            mesh.positions.len() / 3,
            mesh.indices.len()
        );

        // Convert positions from flat array to Vec<[f32; 3]>
        let positions: Vec<[f32; 3]> = mesh
            .positions
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect();

        // Convert indices to u16
        // Note: This assumes the mesh has fewer than 65536 vertices
        let indices: Vec<u16> = mesh
            .indices
            .iter()
            .map(|&i| {
                if i > u16::MAX as u32 {
                    Err(eyre::eyre!(
                        "Mesh has too many vertices for u16 indices (vertex index: {})",
                        i
                    ))
                } else {
                    Ok(i as u16)
                }
            })
            .collect::<Result<Vec<_>>>()?;

        // Convert normals if present
        let normals = if !mesh.normals.is_empty() {
            let normals: Vec<[f32; 3]> = mesh
                .normals
                .chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect();
            Some(normals)
        } else {
            None
        };

        // Convert texture coordinates if present
        let uvs = if !mesh.texcoords.is_empty() {
            let uvs: Vec<[f32; 2]> = mesh
                .texcoords
                .chunks_exact(2)
                .map(|chunk| [chunk[0], chunk[1]])
                .collect();
            Some(uvs)
        } else {
            None
        };

        info!(
            "Successfully loaded mesh '{}' from {}",
            model.name,
            path.display()
        );

        Ok(MeshData {
            positions,
            colors: None, // OBJ doesn't support per-vertex colors
            normals,
            uvs,
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
