//! Procedural texture generation system for the Praxis engine.
//!
//! This crate provides runtime texture synthesis using noise functions and programmable
//! texture graphs. Textures are generated on the GPU using compute shaders compiled at
//! runtime from GLSL to SPIR-V, providing optimal performance with a caching system to
//! avoid redundant regeneration.
//!
//! # Overview
//!
//! The procedural texture system enables artists and developers to create complex, dynamic
//! textures without storing large image files. Instead, textures are described as a graph
//! of operations (nodes) that are executed on the GPU to produce pixel data on-demand.
//!
//! # Features
//!
//! - **Noise Functions**: Perlin, Simplex, Worley (cellular) noise
//! - **Texture Graph**: Node-based system for combining operations
//! - **GPU Compute**: Runtime shader compilation and GPU dispatch (5-10ms for 512x512)
//! - **Caching**: Automatic LRU caching of generated textures
//!
//! # Architecture
//!
//! The system is organized into several modules:
//!
//! ## 1. Texture Graph (Node-Based Composition)
//!
//! The `graph` module provides a **node-based system** for composing textures from simple
//! operations. Think of it like visual node editors in tools like Blender or Substance Designer.
//!
//! Each **TextureNode** represents an operation:
//! - **Source nodes**: Generate base patterns (noise, constants)
//! - **Transform nodes**: Modify coordinates (scale, rotate, offset)
//! - **Blend nodes**: Combine multiple inputs (add, multiply, mix)
//! - **Effect nodes**: Apply filters (invert, contrast, threshold)
//!
//! Nodes are connected in a **directed acyclic graph (DAG)** where:
//! - Nodes read from their inputs and produce output
//! - The graph is evaluated recursively from the output node
//! - Each node becomes a function in the generated shader
//!
//! Example: `Noise → Power → Contrast → ColorRamp → Output`
//!
//! ## 2. Generator (Runtime GLSL-to-SPIR-V Compilation)
//!
//! The `generator` module performs **runtime shader compilation**:
//!
//! **Why runtime compilation?** Each texture graph is unique and can be changed dynamically.
//! Pre-compiling all possible combinations is impossible, so we compile shaders on-demand.
//!
//! **The compilation pipeline:**
//! 1. **Graph → GLSL**: Convert the node graph into GLSL compute shader source
//!    - Each node becomes a `vec4 eval_node_N(vec2 uv)` function
//!    - Nodes call their input nodes recursively
//! 2. **GLSL → SPIR-V**: Use the `shaderc` library to compile to Vulkan bytecode
//!    - SPIR-V is the binary format Vulkan understands
//!    - Compilation takes ~1-5ms per unique graph
//! 3. **SPIR-V → Pipeline**: Create a Vulkan compute pipeline
//!    - The pipeline is cached and reused for the same graph
//!
//! ## 3. Compute Shader Execution and Dispatch
//!
//! GPU compute shaders work differently than traditional graphics shaders:
//!
//! **Compute shader structure:**
//! - Runs in **workgroups** (16×16 threads per group in our case)
//! - Each thread processes one pixel: `gl_GlobalInvocationID.xy` = pixel coordinates
//! - Writes directly to an output image: `imageStore(outputImage, pixel, color)`
//!
//! **Dispatch calculation:**
//! ```
//! dispatch_x = texture_width  ÷ 16 (rounded up)
//! dispatch_y = texture_height ÷ 16 (rounded up)
//! ```
//! For a 512×512 texture: dispatch 32×32 workgroups = 1024 workgroups total
//!
//! **Execution flow:**
//! 1. Allocate output image on GPU (VRAM)
//! 2. Bind image to compute shader
//! 3. Dispatch workgroups (GPU executes all threads in parallel)
//! 4. Copy image data to CPU-accessible buffer
//! 5. Read back RGBA8 pixel data
//!
//! ## 4. Noise Function Implementations
//!
//! The `noise` module provides three types of coherent noise:
//!
//! **Perlin Noise:**
//! - Gradient-based noise with smooth transitions
//! - Uses interpolated gradients at grid corners
//! - Good for natural textures like clouds, terrain
//!
//! **Simplex Noise:**
//! - Improved version of Perlin with better isotropy (no directional bias)
//! - Uses a simplex grid (triangles in 2D, not squares)
//! - Faster and more natural looking than Perlin
//!
//! **Worley Noise (Cellular):**
//! - Distance to nearest random point in each cell
//! - Creates cellular/voronoi patterns
//! - Great for stone, water caustics, organic structures
//!
//! **Fractal Brownian Motion (fBm):**
//! All noise types support fBm by layering multiple **octaves**:
//! - Each octave doubles frequency (lacunarity = 2.0)
//! - Each octave halves amplitude (persistence = 0.5)
//! - More octaves = more detail but slower computation
//!
//! ## 5. LRU Cache Strategy
//!
//! The `cache` module implements a **Least Recently Used (LRU)** cache:
//!
//! **Why caching?** Generating a 512×512 texture takes 5-10ms. If the same texture is
//! needed repeatedly (e.g., for multiple objects), we can reuse the generated data.
//!
//! **Cache key:** Graph structure hash + dimensions + seed
//! - Identical graphs with identical parameters produce the same key
//! - Hash is computed from node types, connections, and parameters
//!
//! **Eviction policy:**
//! - When cache is full (max entries or max memory), evict least-used texture
//! - Each texture tracks its `access_count`
//! - Texture with lowest access count is evicted first
//! - This keeps frequently-used textures in memory
//!
//! **Default limits:** 1000 textures, 512 MB memory
//!
//! # Performance Characteristics
//!
//! - **Shader compilation**: 1-5ms per unique graph (one-time cost)
//! - **GPU generation**: 5-10ms for 512×512 texture
//! - **Cache lookup**: <0.1ms (hash table lookup)
//! - **Memory usage**: ~1 MB per 512×512 RGBA8 texture
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_procedural::{TextureGraph, TextureNode, NoiseType, ProceduralTextureGenerator};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Build a texture graph (node-based composition)
//! let mut graph = TextureGraph::new();
//!
//! // Add a Perlin noise node with 4 octaves
//! let noise_id = graph.add_node(TextureNode::Noise {
//!     noise_type: NoiseType::Perlin,
//!     scale: 8.0,        // Frequency of the noise pattern
//!     octaves: 4,        // Number of detail layers (fBm)
//!     persistence: 0.5,  // Amplitude decay per octave
//!     lacunarity: 2.0,   // Frequency multiplier per octave
//! });
//!
//! // Set the noise node as the graph output
//! graph.set_output(noise_id);
//!
//! // Generate texture on GPU using compute shader
//! // - Compiles GLSL → SPIR-V at runtime
//! // - Dispatches 32×32 workgroups (16×16 threads each)
//! // - Returns RGBA8 pixel data
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
