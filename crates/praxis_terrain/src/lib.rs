//! Terrain rendering system for the Praxis engine.
//!
//! This crate provides a comprehensive terrain rendering system with:
//! - Heightmap-based terrain generation and rendering
//! - Chunked LOD system for large-scale landscapes
//! - Texture splatting with multiple material layers
//! - Grass and vegetation instancing using GPU instancing
//! - Terrain editing tools integrated with the editor
//!
//! # Architecture
//!
//! The terrain system consists of several key components:
//!
//! ## Heightmap Terrain
//!
//! - **`TerrainHeightmap`**: CPU-side heightmap data storage with bilinear interpolation
//! - **`TerrainChunk`**: Individual terrain chunk with mesh data and LOD state
//! - **`TerrainMesh`**: GPU-side terrain mesh generation from heightmap
//!
//! ## LOD System
//!
//! - **`TerrainLodManager`**: Manages LOD levels based on camera distance
//! - **`ChunkLod`**: Per-chunk LOD configuration and transition state
//! - Seamless transitions between LOD levels with skirt geometry
//! - Distance-based chunk activation and deactivation
//!
//! ## Texture Splatting
//!
//! - **`TerrainMaterial`**: Material layer definition with textures and properties
//! - **`SplatMap`**: Control maps for blending up to 8 material layers
//! - **`TerrainRenderer`**: Specialized renderer for terrain with texture splatting
//! - Height and slope-based automatic material distribution
//!
//! ## Vegetation Instancing
//!
//! - **`VegetationLayer`**: Definition of vegetation type (grass, trees, rocks, etc.)
//! - **`VegetationInstance`**: Individual vegetation instance placement with transform
//! - **`VegetationRenderer`**: GPU instancing renderer for millions of instances
//! - Poisson disc sampling for natural distribution
//! - Wind animation and color variation
//!
//! ## Terrain Editing
//!
//! - **`TerrainEditTool`**: Editor tools for terrain sculpting and painting
//! - **`HeightmapBrush`**: Brush-based height editing with various shapes and falloff
//! - **`PaintBrush`**: Paint splat maps to blend materials
//! - **`VegetationPainter`**: Place and remove vegetation instances interactively
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use praxis_terrain::{TerrainConfig, TerrainSystem, TerrainHeightmap};
//! use praxis_math::Vec3;
//!
//! # fn example() -> praxis_utils::Result<()> {
//! // Create terrain from procedural noise
//! let heightmap = TerrainHeightmap::from_noise(512, 512, 100.0, 4.0, 6);
//! 
//! // Configure terrain system
//! let config = TerrainConfig {
//!     chunk_size: 64.0,
//!     vertices_per_chunk: 65,
//!     max_height: 100.0,
//!     lod_levels: 4,
//!     lod_distances: vec![50.0, 100.0, 200.0, 400.0],
//!     world_size: 1024.0,
//!     world_scale: 1.0,
//!     enable_frustum_culling: true,
//!     enable_occlusion_culling: false,
//! };
//!
//! // Create terrain system
//! let mut terrain = TerrainSystem::new(config, heightmap)?;
//!
//! // Add material layers
//! terrain.add_material_layer("grass", 0.0, 30.0)?;
//! terrain.add_material_layer("rock", 30.0, 70.0)?;
//! terrain.add_material_layer("snow", 70.0, 100.0)?;
//!
//! // Add vegetation layers
//! terrain.add_vegetation_layer("grass", 5.0, 0.0, 40.0)?;
//! terrain.add_vegetation_layer("trees", 0.5, 20.0, 60.0)?;
//!
//! // Generate vegetation (uses parallel processing)
//! terrain.generate_vegetation()?;
//!
//! // Update and render (called each frame)
//! let camera_pos = Vec3::new(0.0, 50.0, 0.0);
//! terrain.update(camera_pos);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Features
//!
//! - **Parallel Processing**: Chunk generation and vegetation distribution use Rayon
//! - **GPU Instancing**: Render millions of vegetation instances efficiently
//! - **Frustum Culling**: Only visible chunks and vegetation are rendered
//! - **LOD System**: Reduces triangle count by ~75% at distance
//! - **Streaming**: Chunks are loaded/unloaded based on camera position
//!
//! # Integration
//!
//! The terrain system integrates with:
//! - **Praxis Graphics**: Vulkan-based rendering with specialized shaders
//! - **Praxis Editor**: Visual editing tools for terrain sculpting and painting
//! - **Praxis ECS**: Components for terrain entities
//! - **Praxis Math**: Vector and matrix operations

pub mod chunk;
pub mod components;
pub mod editing;
pub mod heightmap;
pub mod lod;
pub mod material;
pub mod mesh;
pub mod renderer;
pub mod splatmap;
pub mod system;
pub mod vegetation;

pub use chunk::{TerrainChunk, TerrainChunkId};
pub use components::{Terrain, TerrainMaterialLayers, VegetationInstances};
pub use editing::{
    BrushFalloff, BrushShape, HeightmapBrush, PaintBrush, TerrainEditOperation, TerrainEditTool,
    VegetationPainter,
};
pub use heightmap::TerrainHeightmap;
pub use lod::{ChunkLod, TerrainLodManager};
pub use material::{TerrainMaterial, TerrainMaterialLayer};
pub use mesh::TerrainMesh;
pub use renderer::{TerrainRenderer, VegetationInstanceData, VegetationPushConstants, VegetationRenderer};
pub use splatmap::SplatMap;
pub use system::{TerrainConfig, TerrainSystem};
pub use vegetation::{VegetationDistributor, VegetationInstance, VegetationLayer};

use praxis_utils::{info, Result};

/// Initializes the terrain system.
pub fn init() -> Result<()> {
    info!("Initializing terrain system");
    Ok(())
}

/// Maximum number of terrain material layers supported.
pub const MAX_TERRAIN_LAYERS: usize = 8;

/// Maximum number of vegetation layers supported.
pub const MAX_VEGETATION_LAYERS: usize = 16;

/// Default chunk size in vertices (must be power of 2 + 1).
pub const DEFAULT_CHUNK_SIZE: u32 = 65;

/// Default LOD level count.
pub const DEFAULT_LOD_LEVELS: usize = 4;

/// Default LOD distances in world units.
pub const DEFAULT_LOD_DISTANCES: [f32; 4] = [50.0, 100.0, 200.0, 400.0];
