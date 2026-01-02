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
//! - Integration with `praxis_graphics::MeshAssetManager` for GPU upload
//!
//! # Example
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
//! # Ok(())
//! # }
//! ```

pub mod loader;

use praxis_graphics::mesh::MeshAssetManager;
use praxis_graphics::MeshData;
use praxis_utils::Result;
use std::path::Path;

pub use loader::{AssetLoader, MeshLoader};

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

/// Initializes the asset system.
pub fn init() {
    println!("Asset system initialized");
}
