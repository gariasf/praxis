//! Asset management system for the Praxis engine.
//!
//! This crate provides functionality for loading, managing and accessing game assets.
//!
//! # Architecture
//!
//! The asset system is built around a flexible trait-based architecture:
//!
//! - **`AssetLoader`**: Generic trait for loading any asset type from files
//! - **`MeshLoader`**: OBJ file loader implementation
//! - **`GltfLoader`**: GLTF/GLB file loader implementation
//! - **`GltfAssetManager`**: Caching manager for GLTF assets
//! - Integration with `praxis_graphics::MeshAssetManager` for GPU upload
//!
//! # OBJ File Loading
//!
//! ```rust,no_run
//! use praxis_assets::{MeshLoader, AssetLoader};
//! use praxis_graphics::RenderContext;
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Create a mesh loader
//! let loader = MeshLoader::new();
//!
//! // Load an OBJ file
//! let mesh_data = loader.load("assets/models/cube.obj")?;
//!
//! // Upload to GPU via MeshAssetManager
//! render_context
//!     .mesh_manager_mut()
//!     .load_mesh("cube", mesh_data)?;
//! # Ok(())
//! # }
//! ```
//!
//! # GLTF File Loading
//!
//! ```rust,no_run
//! use praxis_assets::GltfLoader;
//!
//! # fn example() -> praxis_utils::Result<()> {
//! // Create a GLTF loader
//! let loader = GltfLoader::new();
//!
//! // Load a GLTF file
//! let asset = loader.load_gltf("assets/models/scene.gltf")?;
//!
//! // Access loaded data
//! println!("Loaded {} meshes", asset.meshes.len());
//! println!("Loaded {} materials", asset.materials.len());
//! println!("Loaded {} textures", asset.textures.len());
//! println!("Scene has {} nodes", asset.nodes.len());
//! # Ok(())
//! # }
//! ```
//!
//! # GLTF Asset Management
//!
//! For caching and reusing GLTF assets:
//!
//! ```rust,no_run
//! use praxis_assets::GltfAssetManager;
//!
//! # fn example() -> praxis_utils::Result<()> {
//! let mut manager = GltfAssetManager::new();
//!
//! // Load and cache a GLTF file
//! let asset = manager.load("assets/models/scene.gltf")?;
//!
//! // Subsequent loads return the cached version
//! let asset2 = manager.load("assets/models/scene.gltf")?;
//! # Ok(())
//! # }
//! ```
//!
//! # High-Level Integration
//!
//! For convenience, you can also use the integration helpers:
//!
//! ```rust,no_run
//! use praxis_assets;
//! use praxis_graphics::RenderContext;
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Load and upload an OBJ mesh in one step
//! praxis_assets::load_obj_mesh(
//!     render_context.mesh_manager_mut(),
//!     "cube",
//!     "assets/models/cube.obj"
//! )?;
//!
//! // Load a GLTF file
//! let gltf_asset = praxis_assets::load_gltf("assets/models/scene.gltf")?;
//! # Ok(())
//! # }
//! ```

pub mod gltf_manager;
pub mod loader;

use praxis_graphics::mesh::MeshAssetManager;
use praxis_graphics::MeshData;
use praxis_utils::Result;
use std::path::Path;

pub use gltf_manager::GltfAssetManager;
pub use loader::{
    AssetLoader, GltfAsset, GltfLoader, GltfMaterial, GltfNode, GltfTexture, GltfTextureFormat,
    MeshLoader,
};

/// Loads an OBJ mesh file and uploads it to the GPU via a mesh asset manager.
///
/// This is a convenience function that combines mesh loading and GPU upload
/// into a single operation. It's equivalent to manually creating a `MeshLoader`,
/// loading the file, and then calling `mesh_manager.load_mesh()`.
///
/// # Arguments
///
/// * `mesh_manager` - Mesh asset manager to upload the mesh to
/// * `id` - Unique identifier for the loaded mesh
/// * `path` - Path to the OBJ file to load
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist or cannot be read
/// - The OBJ format is invalid
/// - GPU buffer creation fails
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets;
/// use praxis_graphics::RenderContext;
///
/// # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
/// praxis_assets::load_obj_mesh(
///     render_context.mesh_manager_mut(),
///     "spaceship",
///     "assets/models/spaceship.obj"
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn load_obj_mesh(
    mesh_manager: &mut MeshAssetManager,
    id: impl Into<String>,
    path: impl AsRef<Path>,
) -> Result<()> {
    let loader = MeshLoader::new();
    let mesh_data = loader.load(path)?;
    mesh_manager.load_mesh(id, mesh_data)?;
    Ok(())
}

/// Loads an OBJ mesh file and returns the mesh data without uploading to GPU.
///
/// This function is useful when you need to process or modify the mesh data
/// before uploading it to the GPU.
///
/// # Arguments
///
/// * `path` - Path to the OBJ file to load
///
/// # Errors
///
/// Returns an error if:
/// - The file doesn't exist or cannot be read
/// - The OBJ format is invalid
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets;
///
/// # fn example() -> praxis_utils::Result<()> {
/// let mesh_data = praxis_assets::load_obj("assets/models/cube.obj")?;
/// println!("Loaded {} vertices", mesh_data.positions.len());
/// # Ok(())
/// # }
/// ```
pub fn load_obj(path: impl AsRef<Path>) -> Result<MeshData> {
    let loader = MeshLoader::new();
    loader.load(path)
}

/// Loads a GLTF file and returns the complete asset data.
///
/// This is a convenience function that creates a loader and loads the GLTF file.
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
///
/// # Example
///
/// ```rust,no_run
/// use praxis_assets;
///
/// let asset = praxis_assets::load_gltf("assets/models/scene.gltf")?;
/// println!("Loaded {} meshes", asset.meshes.len());
/// println!("Loaded {} materials", asset.materials.len());
/// println!("Loaded {} textures", asset.textures.len());
/// # Ok::<(), praxis_utils::eyre::Report>(())
/// ```
pub fn load_gltf(path: impl AsRef<Path>) -> Result<GltfAsset> {
    let loader = GltfLoader::new();
    loader.load_gltf(path)
}

/// Initializes the asset system.
///
/// This function sets up any necessary global state for the asset system.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for asset
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., asset cache setup,
///   thread pool configuration for parallel loading)
///
/// # Example
///
/// ```rust,no_run
/// praxis_assets::init().expect("Failed to initialize asset system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    use praxis_utils::info;
    info!("Initializing asset system");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_init() {
        let result = init();
        assert!(result.is_ok(), "Initialization should succeed");
    }

    #[test]
    fn test_load_obj_function() {
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_load_obj.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = load_obj(&test_file);
        assert!(result.is_ok(), "load_obj should succeed");

        let mesh = result.unwrap();
        assert_eq!(mesh.positions.len(), 3);
        assert_eq!(mesh.indices.len(), 3);

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_obj_with_string_path() {
        let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_load_obj_string.obj");
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let path_string = test_file.to_string_lossy().to_string();
        let result = load_obj(path_string);
        assert!(result.is_ok(), "load_obj should work with String");

        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_load_obj_nonexistent() {
        let result = load_obj("nonexistent_test_file_99999.obj");
        assert!(result.is_err(), "load_obj should fail for nonexistent file");
    }

    #[test]
    fn test_multiple_init_calls() {
        let result1 = init();
        let result2 = init();
        let result3 = init();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }

    #[test]
    fn test_load_gltf_nonexistent() {
        let result = load_gltf("nonexistent_test_file_99999.gltf");
        assert!(
            result.is_err(),
            "load_gltf should fail for nonexistent file"
        );
    }
}
