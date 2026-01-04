//! Procedural texture generation system for the Praxis engine.
//!
//! This crate provides runtime texture synthesis using noise functions and programmable
//! texture graphs. Textures are generated on the GPU using compute shaders for optimal
//! performance, with a caching system to avoid redundant regeneration.
//!
//! # Features
//!
//! - **Noise Functions**: Perlin, Simplex, Worley (cellular) noise
//! - **Texture Graph**: Node-based system for combining operations
//! - **GPU Compute**: Shader-based generation for performance
//! - **Caching**: Automatic caching of generated textures
//!
//! # Architecture
//!
//! The system is organized into several modules:
//! - `noise`: Noise function implementations
//! - `graph`: Texture graph nodes and evaluation
//! - `generator`: GPU compute shader-based generation
//! - `cache`: Caching system for generated textures
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_procedural::{TextureGraph, TextureNode, NoiseType, ProceduralTextureGenerator};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create a simple texture graph
//! let mut graph = TextureGraph::new();
//! let noise_id = graph.add_node(TextureNode::Noise {
//!     noise_type: NoiseType::Perlin,
//!     scale: 8.0,
//!     octaves: 4,
//!     persistence: 0.5,
//!     lacunarity: 2.0,
//! });
//!
//! graph.set_output(noise_id);
//!
//! // Generate texture on GPU
//! // let generator = ProceduralTextureGenerator::new(device, allocator)?;
//! // let texture = generator.generate(&graph, 512, 512)?;
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod generator;
pub mod graph;
pub mod noise;

#[cfg(test)]
mod integration_tests;

pub use cache::{CacheStatistics, ProceduralTextureCache, TextureCacheKey};
pub use generator::{ProceduralTextureGenerator, TextureGenerationParams};
pub use graph::{
    BlendMode, ColorRamp, ColorStop, NoiseType, TextureGraph, TextureNode, TextureNodeId,
    TransformParams,
};
pub use noise::{perlin_noise, simplex_noise, worley_noise};
