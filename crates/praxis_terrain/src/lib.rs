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
//! **Heightmap Representation:**
//! A heightmap is a 2D grid where each cell stores a single height value (f32), representing
//! the terrain elevation at that point. The grid provides a memory-efficient way to represent
//! large terrains: a 512x512 heightmap requires only ~1MB of memory but can define millions
//! of triangles when tessellated.
//!
//! Key characteristics:
//! - **Grid-based**: Regular 2D array indexed by (x, z) coordinates
//! - **Height values**: Typically normalized [0.0, 1.0] then scaled by max_height
//! - **Bilinear interpolation**: Smooth height queries between grid points
//! - **Single-height limitation**: Cannot represent overhangs or caves
//!
//! - **`TerrainHeightmap`**: CPU-side heightmap data storage with bilinear interpolation
//! - **`TerrainChunk`**: Individual terrain chunk with mesh data and LOD state
//! - **`TerrainMesh`**: GPU-side terrain mesh generation from heightmap
//!
//! ## LOD System
//!
//! **Chunked LOD (Level of Detail):**
//! The terrain is divided into fixed-size chunks (e.g., 64x64 meters), each with multiple
//! pre-generated meshes at different detail levels. LOD reduces triangle count for distant
//! terrain, dramatically improving performance.
//!
//! How it works:
//! - **Distance calculation**: Each frame, measure distance from camera to chunk center
//! - **LOD selection**: Choose appropriate detail level based on distance thresholds
//!   - LOD 0 (0-50m): Full detail, 1 meter per vertex
//!   - LOD 1 (50-100m): Half detail, 2 meters per vertex  
//!   - LOD 2 (100-200m): Quarter detail, 4 meters per vertex
//!   - LOD 3 (200m+): Eighth detail, 8 meters per vertex
//! - **Mesh switching**: Swap vertex/index buffers based on current LOD level
//! - **Skirt geometry**: Vertical skirts around chunk edges prevent cracks between LOD levels
//!
//! Performance impact: LOD 3 renders 64x fewer triangles than LOD 0, enabling massive worlds.
//!
//! - **`TerrainLodManager`**: Manages LOD levels based on camera distance
//! - **`ChunkLod`**: Per-chunk LOD configuration and transition state
//! - Seamless transitions between LOD levels with skirt geometry
//! - Distance-based chunk activation and deactivation
//!
//! ## Texture Splatting
//!
//! **Texture Splatting with Blend Weights:**
//! Instead of a single texture, terrain uses multiple material layers (grass, rock, snow, etc.)
//! blended together based on a "splat map" - a texture where each channel stores blend weights.
//!
//! The process:
//! 1. **Splat map**: RGBA texture where each channel is a blend weight [0.0, 1.0]
//!    - Red channel = grass weight
//!    - Green channel = rock weight
//!    - Blue channel = sand weight
//!    - Alpha channel = snow weight
//! 2. **Material layers**: Each layer has albedo, normal, roughness, metallic textures
//! 3. **Fragment shader blending**: For each pixel, sample all layers and blend:
//!    ```glsl
//!    vec4 weights = texture(splatMap, uv);
//!    vec3 finalColor = weights.r * grassColor 
//!                    + weights.g * rockColor
//!                    + weights.b * sandColor
//!                    + weights.a * snowColor;
//!    ```
//! 4. **Weight normalization**: Weights sum to 1.0 for energy conservation
//!
//! Supports up to 8 layers using two RGBA splat maps (4 channels each).
//!
//! - **`TerrainMaterial`**: Material layer definition with textures and properties
//! - **`SplatMap`**: Control maps for blending up to 8 material layers
//! - **`TerrainRenderer`**: Specialized renderer for terrain with texture splatting
//! - Height and slope-based automatic material distribution
//!
//! ## Vegetation Instancing
//!
//! **GPU-Instanced Vegetation:**
//! Efficiently render millions of vegetation instances (grass, trees, rocks) by sending
//! transform data to the GPU and using instanced rendering to draw all copies in one call.
//!
//! Traditional rendering problem:
//! - Drawing 100,000 grass blades with individual draw calls = 100,000 CPU-GPU roundtrips
//! - Each draw call has overhead, severely limiting vegetation density
//!
//! Instancing solution:
//! 1. **Upload base mesh once**: Single grass blade mesh stays in GPU memory
//! 2. **Instance buffer**: Upload array of per-instance data (position, rotation, scale, color)
//! 3. **Single draw call**: GPU draws mesh N times, applying different transforms
//! 4. **Vertex shader**: Reads instance data via `gl_InstanceID` and applies transform
//!
//! Performance: 100,000 instances in ~1ms vs. ~100ms with individual draws.
//!
//! Additional optimizations:
//! - **View frustum culling**: Only render visible instances (done CPU-side)
//! - **Distance culling**: Fade out distant vegetation
//! - **Billboards**: Switch to 2D quads for very distant instances
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
pub use renderer::{
    TerrainRenderer, VegetationInstanceData, VegetationPushConstants, VegetationRenderer,
};
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
