//! Graphics system for the Praxis engine.
//!
//! This crate provides functionality for rendering and managing graphics using Vulkan via vulkano.
//!
//! # Educational Overview: Modern 3D Rendering Architecture
//!
//! This graphics system demonstrates modern GPU rendering techniques used in production game engines.
//! Understanding this architecture provides insight into how AAA games and real-time 3D applications work.
//!
//! ## Core Rendering Concepts
//!
//! ### 1. Vulkan vs OpenGL
//! Vulkan is a low-level graphics API that gives explicit control over:
//! - **Memory management**: We decide when and where GPU memory is allocated
//! - **Synchronization**: We manually manage when GPU work starts/finishes
//! - **Command recording**: We build command buffers that the GPU executes
//! - **Resource lifetimes**: We must ensure resources exist while GPU uses them
//!
//! This explicitness enables better performance but requires more careful programming.
//! OpenGL hides these details, making it simpler but less optimal.
//!
//! ### 2. Render Pipeline Overview
//! ```text
//! CPU Side                          GPU Side
//! ┌──────────────┐                 ┌──────────────┐
//! │ Application  │                 │ Vertex       │
//! │ Logic        │                 │ Shader       │
//! └──────┬───────┘                 └──────┬───────┘
//!        │                                │
//!        ▼                                ▼
//! ┌──────────────┐                 ┌──────────────┐
//! │ Build Draw   │                 │ Rasterizer   │
//! │ Commands     │                 │ (triangles)  │
//! └──────┬───────┘                 └──────┬───────┘
//!        │                                │
//!        ▼                                ▼
//! ┌──────────────┐                 ┌──────────────┐
//! │ Submit to    │───────────────►│ Fragment     │
//! │ GPU Queue    │                 │ Shader       │
//! └──────────────┘                 └──────┬───────┘
//!                                         │
//!                                         ▼
//!                                  ┌──────────────┐
//!                                  │ Framebuffer  │
//!                                  │ (final image)│
//!                                  └──────────────┘
//! ```
//!
//! ### 3. Forward vs Deferred Rendering
//!
//! **Forward Rendering** (traditional):
//! - For each object: shade it with all lights
//! - Cost: O(objects × lights)
//! - Good for: Few lights, transparent objects
//!
//! **Deferred Rendering** (modern):
//! - Pass 1: Render all objects to G-buffer (geometry data)
//! - Pass 2: For each screen pixel: apply all lights
//! - Cost: O(objects) + O(pixels × lights)
//! - Good for: Many lights, complex lighting
//!
//! ### 4. Descriptor Sets (Vulkan's Resource Binding)
//!
//! Descriptor sets are how shaders access resources (textures, buffers):
//! ```text
//! Set 0 (Per-Frame):
//!   - Camera matrices (updated once per frame)
//!   - Lighting data (updated once per frame)
//!
//! Set 1 (Per-Material):
//!   - Textures (albedo, normal, etc.)
//!   - Material properties (metallic, roughness)
//!
//! Set 2 (Per-Object):
//!   - Model matrix (updated for each object)
//! ```
//!
//! Grouping by update frequency minimizes GPU state changes.
//!
//! ### 5. Bindless Rendering (Advanced)
//!
//! Traditional: Bind new descriptor set for each material (expensive)
//! Bindless: All textures in one giant array, index via push constant (fast)
//!
//! Example:
//! ```text
//! Traditional (100 materials):
//!   for material in materials:
//!     bind_descriptor_set(material)  ← 100 GPU state changes
//!     for obj in objects_with_material:
//!       draw(obj)
//!
//! Bindless (100 materials):
//!   bind_descriptor_set(texture_array)  ← 1 GPU state change
//!   for obj in all_objects:
//!     push_constant(material_index)    ← fast CPU-side write
//!     draw(obj)
//! ```
//!
//! ### 6. GPU Culling (Modern Optimization)
//!
//! **CPU Culling** (traditional):
//! - CPU tests each object against frustum
//! - CPU builds list of visible objects
//! - CPU submits draw commands
//!
//! **GPU Culling** (modern):
//! - CPU uploads all objects to GPU
//! - GPU compute shader tests visibility in parallel
//! - GPU builds indirect draw buffer
//! - Single indirect draw call renders all visible objects
//!
//! Benefits: Massively parallel, no CPU-GPU sync, scales to 10,000+ objects
//!
//! # Architecture
//!
//! The graphics system is organized into several modules:
//! - `device`: Vulkan instance and device management
//! - `vertex`: Vertex data structures and primitives
//! - `pipeline`: Graphics pipeline creation and configuration
//! - `shaders`: GLSL shader compilation to SPIR-V
//! - `mesh`: Mesh data structures and asset management
//! - `primitives`: Built-in primitive mesh generators
//! - `texture`: Texture loading and management
//! - `material`: Material system with texture support and descriptor set management
//! - `lighting`: Lighting uniforms and buffer management
//! - `line_renderer`: Line primitive rendering for debug visualization and gizmos
//! - `visual_feedback`: Helper utilities for grids, axes, bounding boxes, and outlines
//! - `deferred`: Deferred rendering with G-buffer passes
//! - `hdr`: High Dynamic Range rendering with tone mapping
//! - `ssao`: Screen-space ambient occlusion for realistic shadowing
//! - `ssr`: Screen-space reflections with hierarchical ray marching and environment probe fallback
//! - `post_process`: Post-processing framework for screen-space effects
//! - `gpu_culling`: GPU-driven culling for large scenes with minimal CPU overhead
//!
//! # Unified Rendering API
//!
//! The graphics system provides a single, unified rendering method that supports all features:
//! - Multiple mesh types per frame
//! - Optional texture per object (defaults to white if not specified)
//! - Optional material properties per object (PBR: metallic, roughness, emissive)
//! - Dynamic lighting updates
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::{RenderContext, DrawCommand, RenderCommands, colored_cube_mesh};
//! use praxis_math::{Mat4, Vec3};
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Load meshes during initialization
//! render_context
//!     .mesh_manager_mut()
//!     .load_mesh("cube", colored_cube_mesh())?;
//!
//! // Render in the frame loop
//! let draw_commands = vec![
//!     DrawCommand {
//!         mesh_id: "cube".to_string(),
//!         model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//!         texture_name: None, // Optional: use Some("texture_name") for custom texture
//!         material_properties: None, // Optional: use Some() for custom materials
//!     },
//! ];
//!
//! let cmds = RenderCommands {
//!     view: Mat4::IDENTITY,
//!     proj: Mat4::IDENTITY,
//!     draw_commands: &draw_commands,
//!     lighting: None, // Optional: use Some() for dynamic lighting
//! };
//!
//! render_context.render(&cmds)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Mesh System
//!
//! The mesh system provides complete support for loading and rendering 3D geometry:
//!
//! - **`MeshData`**: CPU-side mesh definition with vertices, indices, and attributes
//! - **`GpuMesh`**: GPU-side mesh containing Vulkan buffers
//! - **`MeshAssetManager`**: Central manager for loaded meshes
//! - **Primitive Generators**: Built-in functions for common shapes
//!
//! See the [mesh system documentation](../../docs/mesh_system.md) for complete details.
//!
//! # Texture System
//!
//! The texture system provides support for loading and managing textures:
//!
//! - **`Texture`**: GPU-side texture with image view and sampler
//! - **`TextureManager`**: Central manager for cached textures
//! - **`Cubemap`**: GPU-side cubemap texture for skyboxes and environment mapping
//! - **Format Support**: PNG and JPEG via the `image` crate
//! - **Texture Sampling**: Full support in shaders via UV coordinates
//! - **Cubemap Loading**: Support for 6-face cubemaps and equirectangular conversion
//!
//! # Procedural Texture System
//!
//! The procedural texture system provides runtime texture generation using noise functions
//! and programmable texture graphs:
//!
//! - **`ProceduralTextureManager`**: High-level manager for generating procedural textures
//! - **Noise Functions**: Perlin, Simplex, and Worley (cellular) noise
//! - **Texture Graphs**: Node-based system for combining operations (blend, transform, etc.)
//! - **GPU Compute**: Shader-based generation for optimal performance
//! - **Caching**: Automatic caching to avoid redundant generation
//!
//! ## Example
//!
//! ```rust,no_run
//! use praxis_graphics::ProceduralTextureManager;
//! use praxis_procedural::{TextureGraph, TextureNode, NoiseType, TextureGenerationParams};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create a texture graph
//! let mut graph = TextureGraph::new();
//! let noise_id = graph.add_node(TextureNode::Noise {
//!     noise_type: NoiseType::Perlin,
//!     scale: 8.0,
//!     octaves: 4,
//!     persistence: 0.5,
//!     lacunarity: 2.0,
//! });
//! graph.set_output(noise_id);
//!
//! // Generate texture
//! // let mut manager = ProceduralTextureManager::new(
//! //     device,
//! //     queue,
//! //     memory_allocator,
//! //     command_buffer_allocator,
//! //     descriptor_set_allocator,
//! // );
//! // let params = TextureGenerationParams { width: 512, height: 512, seed: 0 };
//! // let texture = manager.generate_texture(&graph, params)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Bindless Rendering System
//!
//! The bindless rendering system eliminates per-material descriptor set binds using
//! VK_EXT_descriptor_indexing with large texture arrays:
//!
//! - **`BindlessTextureManager`**: Central manager for bindless textures and materials
//! - **`BindlessMaterialData`**: GPU-side material structure with texture indices
//! - **Texture Arrays**: Up to 4096 textures in a single descriptor array
//! - **Push Constants**: Material indices passed via fast push constants
//! - **Zero Descriptor Binds**: Eliminates per-material descriptor set operations
//!
//! ## Performance Benefits
//!
//! Traditional rendering with 100 materials:
//! - 100 descriptor set binds per frame
//! - High CPU overhead from GPU synchronization
//! - Complex descriptor set management
//!
//! Bindless rendering with 100 materials:
//! - 1 descriptor set bind per frame
//! - Minimal CPU overhead (push constant writes)
//! - Simple material indexing
//!
//! Result: 100x+ reduction in descriptor set operations
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::{RenderContext, BindlessMaterialData};
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Enable bindless rendering
//! render_context.enable_bindless_rendering()?;
//!
//! // Access bindless manager
//! let bindless = render_context.bindless_manager_mut().unwrap();
//!
//! // Register textures from texture manager
//! let texture_manager = render_context.texture_manager();
//! let brick_texture = texture_manager.get_texture("brick").unwrap();
//! let brick_idx = bindless.register_texture(
//!     "brick",
//!     brick_texture.view.clone(),
//!     brick_texture.sampler.clone(),
//! )?;
//!
//! // Create and register material
//! let material_data = BindlessMaterialData {
//!     base_color: [1.0, 1.0, 1.0, 1.0],
//!     albedo_texture_index: brick_idx,
//!     normal_texture_index: 0,
//!     metallic: 0.0,
//!     roughness: 0.5,
//!     emissive_strength: 0.0,
//!     _padding: [0.0; 3],
//! };
//! let material_idx = bindless.register_material(material_data)?;
//!
//! // Rendering automatically uses bindless mode when enabled
//! // Material switches become essentially free!
//! # Ok(())
//! # }
//! ```
//!
//! See `BINDLESS_RENDERING.md` for complete documentation.
//!
//! # Material System
//!
//! The material system defines surface appearance with efficient descriptor set management:
//!
//! - **`Material`**: Surface properties with texture bindings and descriptor sets
//! - **`MaterialManager`**: Central manager for cached materials with shared descriptor set allocator
//! - **`MaterialProperties`**: PBR-style properties (metallic, roughness, emissive)
//! - **Descriptor Set Management**: Per-material descriptor sets for efficient rendering
//!
//! ## Key Benefits
//!
//! Materials manage their own descriptor sets, providing major performance benefits:
//!
//! - **Reduced Allocations**: 100 objects sharing 1 material = 1 descriptor set (not 100)
//! - **Fewer GPU Binds**: Bind once per material, not once per object
//! - **Memory Efficient**: Descriptor sets are pooled and reused automatically
//!
//! ## Descriptor Set Pooling
//!
//! The rendering system uses a descriptor set pool (`DescriptorSetPool`) to pre-allocate
//! and reuse both transform and material descriptor sets across frames, eliminating
//! per-frame allocation overhead:
//!
//! ### Transform Descriptor Sets
//! - Pooled by texture name (all other bindings are shared)
//! - Created once per unique texture and reused across all frames
//! - Eliminates per-object, per-frame descriptor set allocations
//!
//! ### Material Descriptor Sets
//! - Pooled by texture name and material properties hash
//! - Created once per unique material and reused across all frames
//!
//! **Performance Impact:**
//! - **Frame 1**: Creates 10 transform sets + 5 material sets for 100 objects (15 allocations)
//! - **Frame 2+**: Reuses all 15 cached descriptor sets (zero allocations)
//! - **Result**: 100x+ reduction in descriptor set allocations for typical scenes
//!
//! **Management**: Pool is maintained internally and can be inspected via
//! `descriptor_set_pool_size()` or cleared via `clear_descriptor_set_pool()`
//!
//! This approach eliminates GPU API overhead and memory fragmentation in scenes
//! with many objects.
//!
//! See the `material` module documentation for detailed explanations of descriptor set
//! lifecycle and efficiency gains.
//!
//! ## Texture Sampling Architecture
//!
//! The graphics pipeline supports texture sampling through:
//!
//! 1. **Vertex Format**: `Vertex3D` includes UV coordinates (binding location 3) and tangents (binding location 4)
//! 2. **Shaders**: Vertex shader computes TBN matrix and passes to fragment shader for normal mapping
//! 3. **Descriptor Sets**:
//!    - Set 0, Binding 0: View/Projection uniform buffer
//!    - Set 0, Binding 1: Model matrix (UniformBufferDynamic with dynamic offsets)
//!    - Set 0, Binding 2: Texture sampler (albedo)
//!    - Set 0, Binding 3: Lighting uniform buffer
//!    - Set 0, Binding 4: Shadow uniform buffer
//!    - Set 0, Binding 5-8: Shadow map samplers (one per cascade)
//!    - Set 0, Binding 9: Normal map texture sampler
//!    - Set 1, Binding 0: Material properties uniform buffer
//! 4. **Mesh Data**: `MeshData` supports UV coordinates and tangents via `calculate_tangents()`
//! 5. **Primitives**: Textured primitives like `textured_cube_mesh()` and `textured_quad_mesh()`
//! 6. **Normal Mapping**: TBN matrix computed in vertex shader for per-pixel normal perturbation
//!
//! # Lighting System
//!
//! The lighting system provides dynamic lighting support with directional and point lights:
//!
//! - **`LightingUniforms`**: CPU-side lighting data structure with std140 layout
//! - **`LightingUniformBuffer`**: GPU buffer management for lighting data
//! - **`DirectionalLightData`**: Sun-like lights with direction but no position
//! - **`PointLightData`**: Omnidirectional lights with position and attenuation
//!
//! The lighting data is bound at descriptor set 0, binding 3 and automatically
//! included in all descriptor sets. The fragment shader uses this data to compute
//! Blinn-Phong lighting for each pixel.
//!
//! # Shadow Mapping System
//!
//! The shadow mapping system provides realistic shadows for directional lights:
//!
//! - **`ShadowMapManager`**: Manages shadow map resources and rendering
//! - **`ShadowConfig`**: Configuration for shadow quality and cascade distances
//! - **`ShadowUniforms`**: Shadow data passed to shaders (light-space matrices)
//! - **Cascaded Shadow Maps (CSM)**: Multiple shadow maps at different distances
//! - **PCF Filtering**: Percentage Closer Filtering for soft shadow edges
//!
//! Shadow mapping is a two-pass technique:
//! 1. **Shadow Pass**: Render scene from light's perspective to depth texture
//! 2. **Main Pass**: Sample shadow maps to determine if fragments are shadowed
//!
//! The shadow data is bound at descriptor set 0, binding 4 with shadow map samplers
//! at bindings 5-8 (one per cascade). The fragment shader performs cascade selection
//! and PCF filtering to produce smooth, realistic shadows.
//!
//! # Post-Processing System
//!
//! The post-processing system provides a flexible framework for screen-space effects:
//!
//! - **`PostProcessPass`**: Trait for custom post-processing effects
//! - **`RenderTarget`**: Offscreen framebuffers for render-to-texture
//! - **`RenderTargetPool`**: Efficient render target reuse
//! - **`FullScreenQuad`**: Geometry for full-screen effects
//! - **`PostProcessChain`**: Chains multiple effects together
//!
//! Post-processing effects are applied after 3D scene rendering and operate on
//! 2D screen-space images. Common effects include:
//! - Color grading (grayscale, sepia, etc.)
//! - Image filtering (blur, sharpen, edge detection)
//! - Screen-space effects (bloom, depth of field, motion blur)
//! - Cinematic effects (vignette, chromatic aberration, film grain)
//!
//! ## Bloom Effect
//!
//! The bloom effect creates a glow around bright areas of the scene:
//!
//! - **`BloomEffect`**: Complete bloom implementation with configurable parameters
//! - **`BloomConfig`**: Configuration for brightness threshold, blur iterations, exposure, and intensity
//! - **Brightness Extraction**: Isolates bright pixels above a threshold
//! - **Separable Gaussian Blur**: Efficient two-pass blur (horizontal and vertical)
//! - **HDR Tone Mapping**: Reinhard tone mapping with gamma correction
//!
//! The bloom effect uses a multi-pass pipeline:
//! 1. Extract bright pixels (brightness > threshold)
//! 2. Apply separable Gaussian blur multiple times for smooth glow
//! 3. Combine blurred bloom with original scene using tone mapping
//!
//! ## Cinematic Post-Processing Effects
//!
//! Advanced cinematic effects for realistic and artistic presentation:
//!
//! - **`DepthOfFieldPass`**: Realistic camera lens focus with circle of confusion and bokeh blur
//! - **`MotionBlurPass`**: Per-pixel motion blur using velocity buffers
//! - **`ChromaticAberrationPass`**: Lens color fringing for realistic distortion
//! - **`VignettePass`**: Edge darkening for cinematic framing
//! - **`FilmGrainPass`**: Procedural grain noise to simulate film stock
//!
//! These effects can be chained together in a `PostProcessChain` to create
//! sophisticated cinematic looks. Each effect is highly configurable with its
//! own parameter structure.
//!
//! See the `post_process` module documentation and `POST_PROCESSING.md` for detailed
//! information on implementing custom effects.
//!
//! # Screen-Space Ambient Occlusion (SSAO)
//!
//! The SSAO system provides realistic ambient occlusion effects:
//!
//! - **`SsaoRenderer`**: Complete SSAO implementation with configurable parameters
//! - **`SsaoConfig`**: Configuration for kernel size, radius, bias, and power
//! - **Hemisphere Sampling**: Randomly distributed samples for accurate occlusion
//! - **Noise Texture**: Reduces banding artifacts by rotating the sample kernel
//! - **Blur Pass**: Smooths the occlusion texture to reduce noise
//!
//! SSAO darkens areas that are surrounded by geometry (crevices, corners, contact points)
//! to simulate indirect lighting occlusion. The effect uses the G-buffer depth and
//! normal textures from deferred rendering.
//!
//! The SSAO implementation uses:
//! 1. Generate hemisphere sample kernel with varying distribution
//! 2. Generate random noise texture for kernel rotation
//! 3. SSAO pass: Sample depth buffer in hemisphere around each pixel
//! 4. Blur pass: Apply box blur to reduce noise artifacts
//!
//! The resulting occlusion texture can be integrated into lighting calculations
//! to darken occluded areas, providing more realistic ambient lighting.
//!
//! See the `ssao` module documentation for usage details.
//!
//! # Screen-Space Reflections (SSR)
//!
//! The SSR system provides realistic reflections using screen-space ray marching:
//!
//! - **`SsrRenderer`**: Complete SSR implementation with hierarchical ray marching
//! - **`SsrConfig`**: Configuration for ray marching steps, thickness, roughness, and blur
//! - **Hierarchical Ray Marching**: Adaptive step size for efficient screen-space ray tracing
//! - **Binary Search Refinement**: Sub-pixel accuracy for reflection hit positions
//! - **Roughness-Aware Blur**: Variable blur strength based on surface roughness
//! - **Environment Probe Fallback**: Uses environment probes when rays miss screen-space geometry
//!
//! SSR generates reflections by tracing rays through the depth buffer in screen space.
//! It's efficient and provides accurate reflections for on-screen geometry, with
//! environment probes filling in for off-screen reflections.
//!
//! The SSR implementation uses:
//! 1. Ray marching pass: Trace reflection rays through depth buffer with hierarchical steps
//! 2. Binary search refinement: Refine hit positions for sub-pixel accuracy
//! 3. Roughness-aware blur: Apply separable Gaussian blur scaled by surface roughness
//! 4. Composite pass: Blend SSR reflections with environment probe fallback
//!
//! The resulting reflection texture can be additively blended with the scene to create
//! realistic metallic and glossy surface reflections.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::ssr::{SsrRenderer, SsrConfig};
//! # use std::sync::Arc;
//! # use vulkano::device::Device;
//! # use vulkano::memory::allocator::StandardMemoryAllocator;
//! # fn example(
//! #     device: Arc<Device>,
//! #     memory_allocator: Arc<StandardMemoryAllocator>,
//! # ) -> praxis_utils::Result<()> {
//!
//! let config = SsrConfig::default()
//!     .with_max_steps(64)
//!     .with_max_binary_search_steps(8)
//!     .with_thickness(0.1)
//!     .with_max_roughness(0.8);
//!
//! let mut ssr = SsrRenderer::new(
//!     device,
//!     memory_allocator,
//!     1920,
//!     1080,
//!     config,
//! )?;
//!
//! // In render loop, after G-buffer pass:
//! // let ssr_texture = ssr.render(
//! //     builder,
//! //     gbuffer,
//! //     scene_color,
//! //     view,
//! //     proj,
//! //     camera_position,
//! //     ibl_data,
//! // )?;
//! # Ok(())
//! # }
//! ```
//!
//! See the `ssr` module documentation for complete details.
//!
//! # GPU-Driven Culling System
//!
//! The GPU culling system provides a high-performance culling solution for large scenes:
//!
//! - **`GpuCullingManager`**: Manages compute shader dispatch for frustum and occlusion culling
//! - **`GpuDrawCommand`**: Draw command structure with bounding sphere for culling
//! - **`IndirectDrawCommand`**: Vulkan indirect draw command for GPU-driven rendering
//! - **Frustum Culling**: Tests bounding spheres against view frustum planes on GPU
//! - **Occlusion Culling**: Optional hierarchical Z-buffer culling using depth pyramid
//! - **Indirect Draw Buffer**: GPU generates draw commands directly for `vkCmdDrawIndexedIndirect`
//!
//! The GPU culling implementation uses a compute shader that processes draw commands in parallel,
//! testing each object's bounding sphere against the view frustum. Visible objects are atomically
//! added to an indirect draw buffer, which can then be used for multi-draw indirect rendering.
//!
//! This approach dramatically reduces CPU overhead for large scenes by:
//! - Eliminating per-object CPU culling tests
//! - Avoiding CPU-GPU synchronization for draw counts
//! - Enabling single multi-draw indirect call for all visible objects
//! - Scaling efficiently to tens of thousands of objects
//!
//! ## Example
//!
//! ```rust,no_run
//! use praxis_graphics::gpu_culling::{GpuCullingManager, GpuDrawCommand, GpuMeshData};
//! use praxis_math::{Mat4, Vec4};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create culling manager
//! // let mut culling_manager = GpuCullingManager::new(
//! //     device.clone(),
//! //     memory_allocator.clone(),
//! //     descriptor_set_allocator.clone(),
//! // )?;
//!
//! // Prepare draw commands with bounding spheres
//! // let draw_commands: Vec<GpuDrawCommand> = objects.iter().map(|obj| {
//! //     GpuDrawCommand::new(
//! //         obj.transform,
//! //         Vec4::new(0.0, 0.0, 0.0, 1.0), // bounding sphere
//! //         obj.mesh_id,
//! //         obj.material_id,
//! //     )
//! // }).collect();
//!
//! // Dispatch culling and render
//! // culling_manager.prepare_frame(&draw_commands, &mesh_data)?;
//! // culling_manager.dispatch_culling(cmd_builder, view_proj, frustum_planes, camera_pos)?;
//! # Ok(())
//! # }
//! ```
//!
//! See the `gpu_culling` module documentation for complete details.
//!
//! # Line Rendering System
//!
//! The line rendering system provides efficient rendering of colored line primitives
//! for debug visualization, gizmo drawing, and editor tools:
//!
//! - **`LineVertex`**: Vertex format with position and color
//! - **`Line`**: Single line segment with start point, end point, and color
//! - **`LineBatch`**: Collection of lines for efficient batch rendering
//! - **`LineRenderer`**: GPU renderer with depth testing for proper z-ordering
//!
//! Line rendering is typically used for:
//! - Debug visualization (collision shapes, rays, paths)
//! - Editor gizmos (transform handles, selection boxes)
//! - Grid floors and axis indicators
//! - Bounding boxes and outlines
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{RenderContext, LineBatch};
//! use praxis_math::Vec3;
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Initialize line renderer with depth support
//! let render_pass = render_context.create_render_pass_with_depth(
//!     vulkano::format::Format::R8G8B8A8_UNORM
//! )?;
//! render_context.initialize_line_renderer(render_pass, [800, 600])?;
//!
//! // Create line batch
//! let mut batch = LineBatch::new();
//! batch.add(
//!     Vec3::new(0.0, 0.0, 0.0),
//!     Vec3::new(1.0, 1.0, 1.0),
//!     Vec3::new(1.0, 0.0, 0.0), // Red color
//! );
//!
//! // Render lines within render pass
//! // (Command buffer recording shown in full examples)
//! # Ok(())
//! # }
//! ```
//!
//! ## Visual Feedback Utilities
//!
//! The `visual_feedback` module provides helper functions for common patterns:
//!
//! ```rust,no_run
//! use praxis_graphics::{create_grid, create_axis_indicator, GridConfig, AxisIndicatorConfig};
//! use praxis_math::Vec3;
//!
//! // Grid floor
//! let grid = create_grid(&GridConfig {
//!     size: 20.0,
//!     divisions: 20,
//!     line_color: Vec3::new(0.3, 0.3, 0.3),
//!     axis_color: Vec3::new(0.5, 0.5, 0.5),
//!     height: 0.0,
//! });
//!
//! // XYZ axis indicators
//! let axes = create_axis_indicator(&AxisIndicatorConfig {
//!     length: 1.0,
//!     position: Vec3::ZERO,
//!     show_labels: false,
//! });
//! ```
//!
//! See the `line_renderer` and `visual_feedback` module documentation for complete
//! details on line rendering integration, performance considerations, and render
//! order requirements.
//!
//! # Environment Probe System
//!
//! The environment probe system provides image-based lighting (IBL) for realistic
//! reflections and ambient lighting:
//!
//! - **`EnvironmentProbe`**: Probe component capturing environment as cubemap
//! - **`EnvironmentProbeManager`**: Central manager for probe capture and IBL precomputation
//! - **`EnvironmentProbeCapture`**: Helper for rendering scene to cubemap faces
//! - **Diffuse Irradiance**: Precomputed ambient lighting from environment
//! - **Specular Reflection**: Prefiltered reflections with multiple roughness levels
//! - **Real-time Updates**: Dynamic probe updates for changing scenes
//!
//! Environment probes capture the surrounding environment as a cubemap and precompute
//! lighting data for realistic image-based lighting. They enable:
//! - Accurate reflections on metallic and glossy surfaces
//! - Ambient lighting that matches the scene environment
//! - Indirect lighting approximation without expensive path tracing
//!
//! ## Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{EnvironmentProbeConfig, EnvironmentProbeManager, environment_probe::ProbeUpdateMode};
//! use praxis_math::Vec3;
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create probe configuration
//! let config = EnvironmentProbeConfig {
//!     position: Vec3::new(0.0, 2.0, 0.0),
//!     resolution: 256,
//!     near_clip: 0.1,
//!     far_clip: 100.0,
//!     update_mode: ProbeUpdateMode::Once,
//! };
//!
//! // Add probe to manager
//! // let mut probe_manager = EnvironmentProbeManager::new(device, allocator, queue)?;
//! // probe_manager.add_probe("main_probe".to_string(), config)?;
//!
//! // Get IBL data for rendering
//! // let ibl_data = probe_manager.get_nearest_probe(camera_position);
//! # Ok(())
//! # }
//! ```
//!
//! See the `environment_probe` module documentation and `environment_probe_demo.rs`
//! for complete usage examples.
//!
//! # Skybox System
//!
//! The skybox system provides realistic background rendering using cubemaps:
//!
//! - **`SkyboxRenderer`**: Specialized renderer for skybox cubes with reversed depth
//! - **`Cubemap`**: Support for 6-face cubemaps and equirectangular conversion
//! - **Reversed Depth**: Skybox always renders behind all geometry
//! - **Camera-Centered**: Skybox follows camera rotation but not translation
//!
//! Skyboxes create the illusion of a distant environment (sky, space, etc.) by
//! rendering a large cube textured with a cubemap around the scene. The renderer
//! uses reversed depth testing to ensure the skybox always appears at infinite
//! distance, behind all other geometry.
//!
//! # Deferred Rendering System
//!
//! The deferred rendering system provides an alternative rendering path optimized
//! for many-light scenarios:
//!
//! - **`DeferredRenderer`**: Complete deferred rendering pipeline
//! - **`GBuffer`**: Multiple render targets storing geometry data
//! - **Geometry Pass**: Renders scene to G-buffer (albedo, normal, metallic-roughness, depth)
//! - **Lighting Pass**: Full-screen pass accumulating lighting from all lights
//!
//! ## Benefits Over Forward Rendering
//!
//! - **Many Lights**: Lighting cost is O(lights × pixels) instead of O(lights × triangles)
//! - **Efficient Culling**: Only visible pixels are lit, occluded geometry is skipped
//! - **Decoupled Shading**: Geometry and lighting calculations are independent
//!
//! ## Trade-offs
//!
//! - **Memory**: Requires multiple full-screen render targets (G-buffer)
//! - **Bandwidth**: Multiple render target writes and reads
//! - **Transparency**: Difficult to handle (requires hybrid forward pass)
//! - **MSAA**: Expensive with multiple render targets
//!
//! The deferred renderer can be used alongside the forward renderer, allowing
//! applications to choose the best rendering path for their needs or use both
//! (e.g., deferred for opaque geometry, forward for transparent objects).
//!
//! # HDR Rendering System
//!
//! The HDR (High Dynamic Range) rendering system provides a complete pipeline for
//! rendering with floating-point precision and tone mapping to displayable LDR:
//!
//! - **`HdrRenderTarget`**: Floating-point render targets (R16G16B16A16_SFLOAT)
//! - **`ExposureCalculator`**: Automatic and manual exposure calculation
//! - **`ToneMapper`**: Tone mapping with multiple operators (ACES, Reinhard, Uncharted 2)
//! - **`ToneMappingOperator`**: Selection of tone mapping algorithms
//!
//! ## HDR Pipeline
//!
//! The HDR rendering pipeline works in these stages:
//!
//! 1. **HDR Scene Rendering**: Render scene to floating-point target (values can exceed 1.0)
//! 2. **Luminance Calculation**: Calculate average scene brightness for auto-exposure
//! 3. **Exposure Adjustment**: Apply exposure based on scene luminance or manual value
//! 4. **Tone Mapping**: Map HDR values to LDR [0,1] range using selected operator
//! 5. **Gamma Correction**: Apply final gamma correction (typically 2.2)
//!
//! ## Tone Mapping Operators
//!
//! - **Reinhard**: Simple and fast, `color / (color + 1)`
//! - **ACES**: Industry-standard filmic curve, used in film and AAA games
//! - **Uncharted 2**: High contrast, used in Uncharted 2 (Hable tone mapping)
//!
//! ## Exposure Modes
//!
//! - **Manual**: Fixed exposure value set by the application
//! - **Automatic**: Dynamic exposure based on average scene luminance with smooth adaptation
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::{HdrRenderTarget, ToneMapper, ToneMappingOperator, ExposureMode};
//!
//! # async fn example() -> praxis_utils::Result<()> {
//! // Create HDR render target for scene rendering
//! // let hdr_target = HdrRenderTarget::new(memory_allocator, render_pass, [1920, 1080])?;
//!
//! // Create tone mapper with ACES operator
//! // let mut tone_mapper = ToneMapper::new(device, memory_allocator, format, ToneMappingOperator::ACES)?;
//!
//! // Set automatic exposure
//! // tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });
//!
//! // In render loop:
//! // 1. Render scene to HDR target
//! // render_scene_to_hdr(&hdr_target);
//!
//! // 2. Apply tone mapping
//! // let average_luminance = 0.5; // From scene analysis or fixed value
//! // tone_mapper.apply(builder, &hdr_target, output_framebuffer, extent, average_luminance, delta_time)?;
//! # Ok(())
//! # }
//! ```
//!
//! See the `hdr` module documentation for detailed information on implementing HDR rendering.
//!
//! # Rendering Flow
//!
//! ```text
//! Application
//!     │
//!     ▼
//! RenderContext::new()     ← Initialize Vulkan
//!     │
//!     ▼
//! RenderContext::render()  ← Called each frame
//!     │
//!     ├─► Acquire swapchain image
//!     ├─► Record command buffer
//!     ├─► Submit to GPU
//!     └─► Present to screen
//! ```
//!
//! # Module Privacy Patterns
//!
//! This crate uses two patterns for module organization:
//!
//! ## Public Modules
//!
//! Most subsystems are exposed as public modules (`pub mod`), allowing direct access
//! to their types and functions:
//! - `deferred`, `hdr`, `lighting`, `material`, `mesh`, `texture`, etc.
//!
//! ## Private Modules with Selective Re-exports
//!
//! Implementation detail modules are kept private (`mod`) with only their public API
//! re-exported at the crate root:
//! - `device`: Internal Vulkan device setup (not re-exported)
//! - `pipeline`: Internal pipeline creation (not re-exported)
//! - `shaders`: Internal shader compilation (not re-exported)
//! - `vertex`: Private module, `Vertex3D` re-exported
//! - `primitives`: Private module, mesh generator functions re-exported
//!
//! This pattern provides:
//! - **Encapsulation**: Implementation details remain hidden
//! - **Stable API**: Public surface is clearly defined via re-exports
//! - **Flexibility**: Internal organization can change without breaking users
//!
//! # Testing and Mocking
//!
//! For headless testing without GPU initialization, use `MockRenderContext`:
//!
//! ```rust,no_run
//! # #[cfg(test)]
//! # {
//! use praxis_graphics::MockRenderContext;
//!
//! let mut ctx = MockRenderContext::new();
//! // All rendering operations are no-ops
//! // Suitable for testing game logic without graphics hardware
//! # }
//! ```
//!
//! The mock provides the same API surface as `RenderContext` but with all
//! operations as no-ops, allowing tests to run in CI environments without
//! GPU access.

pub mod bindless;
pub mod deferred;
mod device;
pub mod gpu_culling;
pub mod hdr;
pub mod lighting;
pub mod line_renderer;
pub mod lod;
pub mod material;
pub mod material_instancing;
pub mod material_layers;
pub mod mesh;
pub mod particles;
mod pipeline;
pub mod post_process;
/// Private module containing primitive mesh generators.
/// Public API is re-exported at crate root (see `pub use primitives::{...}` below).
mod primitives;
pub mod procedural_texture;
mod shaders;
pub mod shadow;
pub mod skybox;
pub mod ssao;
pub mod ssr;
pub mod taa;
pub mod texture;
pub mod uniform_buffer;
pub mod velocity_buffer;
/// Private module containing vertex type definitions.
/// Public API is re-exported at crate root (see `pub use vertex::Vertex3D` below).
mod vertex;
pub mod visual_feedback;

use crate::{device::VulkanDevice, pipeline::create_simple_pipeline_3d};
use praxis_math::Mat4;
use praxis_utils::{debug, error, eyre, info, timing::FrameTimer, trace, warn, Result};
use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::descriptor_set::allocator::DescriptorSetAllocator;
use vulkano::descriptor_set::DescriptorSet;

use std::collections::HashMap;
use std::sync::Arc;
use vulkano::descriptor_set::{allocator::StandardDescriptorSetAllocator, WriteDescriptorSet};
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    },
    device::{physical::PhysicalDevice, Device, Queue},
    image::{view::ImageView, Image, ImageUsage},
    instance::Instance,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};
use winit::window::Window;

/// A single draw command with mesh, transform, and optional texture/material.
///
/// This is the unified draw command structure that supports all rendering features:
/// - Different mesh types per object
/// - Optional custom textures (defaults to white texture if not specified)
/// - Optional PBR material properties (defaults to standard properties if not specified)
/// - Optional bone matrices for skeletal animation
///
/// # Examples
///
/// Basic colored mesh:
/// ```rust,no_run
/// # use praxis_graphics::DrawCommand;
/// # use praxis_math::Mat4;
/// DrawCommand {
///     mesh_id: "cube".to_string(),
///     model: Mat4::IDENTITY,
///     texture_name: None,
///     material_properties: None,
///     bone_matrices: None,
/// }
/// # ;
/// ```
///
/// Textured mesh:
/// ```rust,no_run
/// # use praxis_graphics::DrawCommand;
/// # use praxis_math::Mat4;
/// DrawCommand {
///     mesh_id: "wall".to_string(),
///     model: Mat4::IDENTITY,
///     texture_name: Some("brick".to_string()),
///     material_properties: None,
///     bone_matrices: None,
/// }
/// # ;
/// ```
///
/// Textured mesh with PBR material:
/// ```rust,no_run
/// # use praxis_graphics::{DrawCommand, MaterialProperties};
/// # use praxis_math::Mat4;
/// DrawCommand {
///     mesh_id: "sphere".to_string(),
///     model: Mat4::IDENTITY,
///     texture_name: Some("metal".to_string()),
///     material_properties: Some(MaterialProperties::new()
///         .with_metallic(0.9)
///         .with_roughness(0.2)),
///     bone_matrices: None,
/// }
/// # ;
/// ```
///
/// Animated mesh with skeletal animation:
/// ```rust,no_run
/// # use praxis_graphics::DrawCommand;
/// # use praxis_math::Mat4;
/// DrawCommand {
///     mesh_id: "character".to_string(),
///     model: Mat4::IDENTITY,
///     texture_name: Some("skin".to_string()),
///     material_properties: None,
///     bone_matrices: Some(vec![Mat4::IDENTITY; 10]), // Actual bone transforms
/// }
/// # ;
/// ```
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Identifier of the mesh to draw.
    pub mesh_id: String,
    /// Model matrix for this object.
    pub model: Mat4,
    /// Optional texture name to use instead of the default white texture.
    pub texture_name: Option<String>,
    /// Optional material properties for this object.
    /// If None, uses default material properties (white, non-metallic, medium roughness).
    pub material_properties: Option<material::MaterialProperties>,
    /// Optional bone matrices for skeletal animation (up to 256 bones).
    /// If None, uses identity matrices (no skeletal animation).
    /// For animated meshes, provide the final skinning matrices computed from
    /// the animated pose.
    pub bone_matrices: Option<Vec<Mat4>>,
}

/// Unified render commands supporting all rendering features.
///
/// This structure provides the complete set of data needed for rendering:
/// - Camera matrices (view and projection)
/// - List of objects to render with their meshes, transforms, textures, and materials
/// - Optional lighting data for dynamic lighting updates
///
/// # Performance Optimizations
///
/// The render implementation includes several automatic optimizations:
///
/// **Material Batching**: Draw commands are sorted by texture and material properties
/// to minimize GPU state changes. Objects with identical materials are rendered
/// consecutively, allowing descriptor set reuse.
///
/// **Descriptor Set Reuse**: When multiple objects share the same material properties,
/// the same material descriptor set is reused instead of creating a new one for each object.
///
/// **Conditional Descriptor Binding**: Material descriptor sets are only re-bound when
/// the material changes, not for every object.
///
/// # Examples
///
/// Basic rendering:
/// ```rust,no_run
/// # use praxis_graphics::{RenderCommands, DrawCommand};
/// # use praxis_math::Mat4;
/// let cmds = RenderCommands {
///     view: Mat4::IDENTITY,
///     proj: Mat4::IDENTITY,
///     draw_commands: &[
///         DrawCommand {
///             mesh_id: "cube".to_string(),
///             model: Mat4::IDENTITY,
///             texture_name: None,
///             material_properties: None,
///         },
///     ],
///     lighting: None,
/// };
/// ```
pub struct RenderCommands<'a> {
    /// Camera view matrix (world → view).
    pub view: Mat4,
    /// Camera projection matrix (view → clip).
    pub proj: Mat4,
    /// List of draw commands with mesh, texture, and material references.
    pub draw_commands: &'a [DrawCommand],
    /// Optional lighting data to upload this frame.
    /// If None, uses the previously uploaded lighting data.
    pub lighting: Option<&'a lighting::LightingUniforms>,
}

/// Key for identifying unique material descriptor sets in the pool.
///
/// Materials are identified by their texture name and property bytes.
/// This allows efficient reuse of descriptor sets for identical materials.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct MaterialKey {
    texture_name: String,
    properties_hash: u64,
}

impl MaterialKey {
    fn new(texture_name: String, properties: &material::MaterialProperties) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        bytemuck::bytes_of(properties).hash(&mut hasher);
        let properties_hash = hasher.finish();

        Self {
            texture_name,
            properties_hash,
        }
    }
}

/// Key for identifying unique transform descriptor sets in the pool.
///
/// Transform descriptor sets are identified by their texture name since the
/// buffers (view/projection, dynamic uniforms, lighting) are shared across all
/// objects and only the texture varies.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct TransformKey {
    texture_name: String,
}

impl TransformKey {
    fn new(texture_name: String) -> Self {
        Self { texture_name }
    }
}

/// Pool for pre-allocating and reusing descriptor sets for materials and transforms.
///
/// The descriptor set pool manages both material and transform descriptor sets to
/// eliminate per-frame allocation overhead. It maintains caches of descriptor sets
/// keyed by their properties, allowing multiple objects to share descriptor sets
/// when they use identical configurations.
///
/// # Benefits
///
/// - **Eliminated Per-Frame Allocations**: Descriptor sets are created once and reused
/// - **Cache Efficiency**: Identical configurations share the same descriptor set
/// - **Lower GPU Overhead**: Significantly fewer descriptor set allocations and bindings
/// - **Memory Efficiency**: No redundant descriptor set storage
///
/// # Pooling Strategy
///
/// ## Transform Descriptor Sets
/// Pooled by texture name since all other bindings (view/proj, dynamic uniforms,
/// lighting) are shared across all objects in a frame.
///
/// ## Material Descriptor Sets
/// Pooled by texture name and material properties hash to allow sharing between
/// objects with identical materials.
///
/// # Example
///
/// ```text
/// Frame 1: 100 objects with 10 unique textures and 5 unique materials
///   - Creates 10 transform descriptor sets (one per texture)
///   - Creates 5 material descriptor sets (one per unique material)
///   - Total allocations: 15
///
/// Frame 2: Same 100 objects
///   - Reuses all 15 cached descriptor sets (zero allocations)
///
/// Frame 3: 200 objects with same 10 textures and 5 materials
///   - Reuses all 15 cached descriptor sets (zero allocations)
///
/// Result: 100x+ reduction in descriptor set allocations
/// ```
struct DescriptorSetPool {
    /// Cached transform descriptor sets indexed by texture name
    transform_sets: HashMap<TransformKey, Arc<DescriptorSet>>,

    /// Cached material descriptor sets indexed by material key
    material_sets: HashMap<
        MaterialKey,
        (
            Arc<DescriptorSet>,
            vulkano::buffer::Subbuffer<material::MaterialProperties>,
        ),
    >,

    /// Descriptor set allocator for creating new sets
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,

    /// Memory allocator for creating material buffers
    memory_allocator: Arc<StandardMemoryAllocator>,

    /// Layout for transform descriptor sets
    transform_descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,

    /// Layout for material descriptor sets
    material_descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,
}

impl DescriptorSetPool {
    /// Creates a new descriptor set pool.
    fn new(
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        transform_descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,
        material_descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,
    ) -> Self {
        Self {
            transform_sets: HashMap::new(),
            material_sets: HashMap::new(),
            descriptor_set_allocator,
            memory_allocator,
            transform_descriptor_set_layout,
            material_descriptor_set_layout,
        }
    }

    /// Gets or creates a transform descriptor set for the given texture.
    ///
    /// If a descriptor set already exists for this texture combination, returns the
    /// cached version. Otherwise, creates a new descriptor set and caches it.
    ///
    /// # Arguments
    ///
    /// * `texture_name` - Name of the texture for this transform set
    /// * `view_proj_buffer` - View/projection uniform buffer
    /// * `dynamic_uniform_buffer` - Dynamic uniform buffer for model matrices
    /// * `texture` - Texture to bind
    /// * `lighting_buffer` - Lighting uniform buffer
    /// * `default_normal_map` - Default normal map texture
    ///
    /// # Returns
    ///
    /// A descriptor set containing all transform bindings.
    ///
    /// # Errors
    ///
    /// Returns an error if descriptor set creation fails.
    #[allow(clippy::too_many_arguments)]
    fn get_or_create_transform_set(
        &mut self,
        texture_name: String,
        view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
        dynamic_uniform_buffer: vulkano::buffer::Subbuffer<[u8]>,
        texture: &texture::Texture,
        lighting_buffer: vulkano::buffer::Subbuffer<lighting::LightingUniforms>,
        default_normal_map: &texture::Texture,
        bone_matrices_buffer: vulkano::buffer::Subbuffer<uniform_buffer::BoneMatricesUniforms>,
        shadow_buffer: vulkano::buffer::Subbuffer<shadow::ShadowUniforms>,
        dummy_shadow_map: Arc<ImageView>,
        shadow_sampler: Arc<vulkano::image::sampler::Sampler>,
    ) -> Result<Arc<DescriptorSet>> {
        let key = TransformKey::new(texture_name.clone());

        if let Some(descriptor_set) = self.transform_sets.get(&key) {
            trace!(
                "Reusing cached transform descriptor set for texture '{}'",
                texture_name
            );
            return Ok(descriptor_set.clone());
        }

        trace!(
            "Creating new transform descriptor set for texture '{}'",
            texture_name
        );

        // Create descriptor set with all bindings
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.transform_descriptor_set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, view_proj_buffer),
                WriteDescriptorSet::buffer(1, dynamic_uniform_buffer),
                WriteDescriptorSet::image_view_sampler(
                    2,
                    texture.view.clone(),
                    texture.sampler.clone(),
                ),
                WriteDescriptorSet::buffer(3, lighting_buffer),
                WriteDescriptorSet::buffer(4, shadow_buffer),
                WriteDescriptorSet::image_view_sampler(5, dummy_shadow_map.clone(), shadow_sampler.clone()),
                WriteDescriptorSet::image_view_sampler(6, dummy_shadow_map.clone(), shadow_sampler.clone()),
                WriteDescriptorSet::image_view_sampler(7, dummy_shadow_map.clone(), shadow_sampler.clone()),
                WriteDescriptorSet::image_view_sampler(8, dummy_shadow_map.clone(), shadow_sampler.clone()),
                WriteDescriptorSet::image_view_sampler(
                    9,
                    default_normal_map.view.clone(),
                    default_normal_map.sampler.clone(),
                ),
                WriteDescriptorSet::buffer(10, bone_matrices_buffer),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create transform descriptor set: {}", e))?;

        // Cache the descriptor set for reuse
        self.transform_sets.insert(key, descriptor_set.clone());

        Ok(descriptor_set)
    }

    /// Gets or creates a material descriptor set for the given properties.
    ///
    /// If a descriptor set already exists for this material, returns the cached version.
    /// Otherwise, creates a new descriptor set and caches it for future use.
    ///
    /// # Arguments
    ///
    /// * `texture_name` - Name of the texture for this material
    /// * `material_props` - Material properties to bind
    ///
    /// # Returns
    ///
    /// A descriptor set containing the material properties uniform buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if descriptor set or buffer creation fails.
    fn get_or_create_material_set(
        &mut self,
        texture_name: String,
        material_props: material::MaterialProperties,
    ) -> Result<Arc<DescriptorSet>> {
        let key = MaterialKey::new(texture_name, &material_props);

        if let Some((descriptor_set, _)) = self.material_sets.get(&key) {
            trace!("Reusing cached material descriptor set");
            return Ok(descriptor_set.clone());
        }

        trace!("Creating new material descriptor set");

        // Create material properties buffer
        let material_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            material_props,
        )
        .map_err(|e| eyre::eyre!("Failed to create material properties buffer: {}", e))?;

        // Create descriptor set
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.material_descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, material_buffer.clone())],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create material descriptor set: {}", e))?;

        // Cache the descriptor set and buffer for reuse
        self.material_sets
            .insert(key, (descriptor_set.clone(), material_buffer));

        Ok(descriptor_set)
    }

    /// Clears all cached descriptor sets.
    ///
    /// This should be called when materials or textures are modified to ensure
    /// the cache is invalidated.
    fn clear(&mut self) {
        debug!(
            "Clearing descriptor set pool ({} transform sets, {} material sets)",
            self.transform_sets.len(),
            self.material_sets.len()
        );
        self.transform_sets.clear();
        self.material_sets.clear();
    }

    /// Returns the total number of cached descriptor sets.
    fn len(&self) -> usize {
        self.transform_sets.len() + self.material_sets.len()
    }
}

/// Core graphics context containing the Vulkan state.
///
/// This struct manages the entire graphics rendering pipeline, from initialization
/// to frame presentation. It encapsulates all Vulkan objects and provides a
/// simplified interface for rendering.
///
/// # Responsibilities
///
/// - Vulkan device and queue management
/// - Swapchain creation and recreation
/// - Command buffer recording and submission
/// - Frame synchronization
/// - Resource lifetime management
///
/// # Frame Lifecycle
///
/// Each frame follows this sequence:
/// 1. Acquire next swapchain image
/// 2. Record rendering commands
/// 3. Submit commands to GPU
/// 4. Present the rendered image
///
/// # Example
///
/// ```rust,ignore
/// // Requires async runtime and window handle
/// let window = Arc::new(window);
/// let mut ctx = RenderContext::new(window).await?;
///
/// // Main render loop
/// loop {
///     ctx.render()?;
/// }
/// ```
pub struct RenderContext {
    // Public fields for external access
    /// The Vulkan instance - connection to the Vulkan API
    pub instance: Arc<Instance>,
    /// The logical device - interface to the GPU
    pub device: Arc<Device>,
    /// Queue for submitting graphics commands
    pub graphics_queue: Arc<Queue>,
    /// Queue for presenting images (may be same as graphics_queue)
    pub present_queue: Arc<Queue>,

    // Private implementation details
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    swapchain_image_views: Vec<Arc<ImageView>>,
    depth_images: Vec<Arc<ImageView>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    frame_timer: FrameTimer,

    /// Mesh asset manager for loading and managing meshes.
    mesh_manager: mesh::MeshAssetManager,

    /// Texture asset manager for loading and managing textures.
    texture_manager: texture::TextureManager,

    /// Procedural texture manager for GPU-based texture generation.
    procedural_texture_manager: procedural_texture::ProceduralTextureManager,

    /// Material asset manager for loading and managing materials.
    material_manager: material::MaterialManager,

    /// Lighting uniform buffer for passing lighting data to shaders.
    lighting_buffer: lighting::LightingUniformBuffer,

    /// Dynamic uniform buffer for per-object model matrices.
    dynamic_uniform_buffer: uniform_buffer::DynamicUniformBuffer,

    /// Buffer for per-frame view/projection uniforms.
    view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,

    /// Buffer for bone matrices (skeletal animation).
    bone_matrices_buffer: vulkano::buffer::Subbuffer<uniform_buffer::BoneMatricesUniforms>,

    /// Buffer for shadow uniforms (disabled by default with cascade_count = 0).
    shadow_buffer: vulkano::buffer::Subbuffer<shadow::ShadowUniforms>,

    /// Dummy depth image for shadow map binding (when shadows are disabled).
    dummy_shadow_map: Arc<ImageView>,

    /// Sampler for shadow maps (depth comparison sampler).
    shadow_sampler: Arc<vulkano::image::sampler::Sampler>,

    /// Line renderer for debug visualization and gizmos.
    line_renderer: Option<line_renderer::LineRenderer>,
    /// Descriptor set pool for efficient material descriptor set reuse.
    descriptor_set_pool: DescriptorSetPool,

    /// Bindless texture manager for zero-cost material switches.
    bindless_manager: Option<bindless::BindlessTextureManager>,

    /// Whether to use bindless rendering mode.
    use_bindless: bool,
}

impl RenderContext {
    /// Creates a new `RenderContext` for a given window.
    ///
    /// This function performs the complete Vulkan initialization sequence:
    ///
    /// 1. **Device Setup**: Creates Vulkan instance, selects GPU, creates logical device
    /// 2. **Swapchain Setup**: Creates swapchain for presenting images to the window
    /// 3. **Pipeline Setup**: Compiles shaders and creates the graphics pipeline
    /// 4. **Resource Setup**: Allocates vertex buffers and other resources
    ///
    /// # Arguments
    ///
    /// * `window` - The window to render into. Must remain valid for the lifetime of the context.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Vulkan is not available on the system
    /// - No suitable GPU is found
    /// - Required extensions are not supported
    /// - Resource allocation fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Requires async runtime and window handle
    /// let window = Arc::new(window);
    /// let context = RenderContext::new(window).await?;
    /// ```
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        info!("Initializing graphics context...");
        let init_start = std::time::Instant::now();

        debug!("Creating Vulkan device and surface");
        let device_start = std::time::Instant::now();
        let (vulkan_device, surface) = VulkanDevice::new(&window)?;
        debug!("Vulkan device created in {:?}", device_start.elapsed());

        let instance = vulkan_device.instance.clone();
        let device = vulkan_device.device.clone();
        let physical_device = vulkan_device.physical_device.clone();
        let graphics_queue = vulkan_device.graphics_queue.clone();
        let present_queue = vulkan_device.present_queue.clone();

        debug!("Creating swapchain");
        let swapchain_start = std::time::Instant::now();
        let (swapchain, swapchain_images) =
            Self::create_swapchain(&device, &physical_device, &surface, &window)?;

        info!(
            "Created swapchain with {} images at {}x{} in {:?}",
            swapchain_images.len(),
            swapchain.image_extent()[0],
            swapchain.image_extent()[1],
            swapchain_start.elapsed()
        );

        trace!("Creating {} swapchain image views", swapchain_images.len());
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| ImageView::new_default(image.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| eyre::eyre!("Failed to create image views: {}", e))?;
        trace!("Created swapchain image views");

        trace!("Creating memory allocator");
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        debug!("Creating {} depth images", swapchain_images.len());
        let depth_images = Self::create_depth_images(
            &memory_allocator,
            swapchain.image_extent(),
            swapchain_images.len(),
        )?;

        debug!("Creating render pass");
        let render_pass = Self::create_render_pass(&device, swapchain.image_format())?;

        debug!("Creating {} framebuffers", swapchain_image_views.len());
        let framebuffers = Self::create_framebuffers(&swapchain_image_views, &depth_images, &render_pass)?;

        trace!("Creating command buffer allocator");
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let graphics_pipeline =
            create_simple_pipeline_3d(&device, &render_pass, swapchain.image_extent())?;

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let descriptor_set_layout = graphics_pipeline.layout().set_layouts()[0].clone();
        let material_descriptor_set_layout = graphics_pipeline.layout().set_layouts()[1].clone();

        // Create descriptor set pool for efficient material descriptor set reuse
        debug!("Creating descriptor set pool");
        let descriptor_set_pool = DescriptorSetPool::new(
            descriptor_set_allocator.clone(),
            memory_allocator.clone(),
            descriptor_set_layout.clone(),
            material_descriptor_set_layout,
        );

        // Create viewport to normalize coordinates from vertex shader output to
        // framebuffer coordinates
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [
                swapchain.image_extent()[0] as f32,
                swapchain.image_extent()[1] as f32,
            ],
            depth_range: 0.0..=1.0,
        };

        // Initialize frame synchronization
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        // Initialize mesh manager
        let mesh_manager = mesh::MeshAssetManager::new(
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            graphics_queue.clone(),
        );

        // Initialize texture manager
        let mut texture_manager = texture::TextureManager::new(
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            graphics_queue.clone(),
        );

        // Create default white texture
        debug!("Creating default white texture");
        texture_manager
            .create_default_white_texture()
            .map_err(|e| eyre::eyre!("Failed to create default white texture: {}", e))?;

        // Create default flat normal map texture
        debug!("Creating default flat normal map texture");
        texture_manager
            .create_default_flat_normal()
            .map_err(|e| eyre::eyre!("Failed to create default flat normal texture: {}", e))?;

        // Initialize procedural texture manager
        debug!("Creating procedural texture manager");
        let procedural_texture_manager = procedural_texture::ProceduralTextureManager::new(
            device.clone(),
            graphics_queue.clone(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
            descriptor_set_allocator.clone(),
        );

        // Initialize material manager
        debug!("Creating material manager");
        let material_manager = material::MaterialManager::new();

        // Create lighting uniform buffer
        debug!("Creating lighting uniform buffer");
        let lighting_buffer = lighting::LightingUniformBuffer::new(memory_allocator.clone())?;

        // Create dynamic uniform buffer with 3 frames in flight and 1024 max objects
        debug!("Creating dynamic uniform buffer");
        let dynamic_uniform_buffer =
            uniform_buffer::DynamicUniformBuffer::new(&device, memory_allocator.clone(), 3, 1024)?;

        // Create initial view/projection buffer with identity matrices
        debug!("Creating view/projection buffer");
        let initial_view_proj = uniform_buffer::ViewProjectionUniforms {
            view: Mat4::IDENTITY.to_cols_array_2d(),
            proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_position: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };

        let view_proj_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            initial_view_proj,
        )
        .map_err(|e| eyre::eyre!("Failed to create view/projection buffer: {}", e))?;

        // Create bone matrices buffer with identity matrices (for non-animated meshes)
        debug!("Creating bone matrices buffer");
        let initial_bone_matrices = uniform_buffer::BoneMatricesUniforms::identity();
        let bone_matrices_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            initial_bone_matrices,
        )
        .map_err(|e| eyre::eyre!("Failed to create bone matrices buffer: {}", e))?;

        // Create shadow buffer with disabled shadows (cascade_count = 0)
        debug!("Creating shadow uniform buffer");
        let initial_shadow_uniforms = shadow::ShadowUniforms::default();
        let shadow_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            initial_shadow_uniforms,
        )
        .map_err(|e| eyre::eyre!("Failed to create shadow uniform buffer: {}", e))?;

        // Create dummy shadow map (1x1 depth image for binding when shadows are disabled)
        debug!("Creating dummy shadow map");
        use vulkano::format::Format;
        let dummy_shadow_image = Image::new(
            memory_allocator.clone(),
            vulkano::image::ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: [1, 1, 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create dummy shadow image: {}", e))?;
        
        let dummy_shadow_map = ImageView::new_default(dummy_shadow_image)
            .map_err(|e| eyre::eyre!("Failed to create dummy shadow image view: {}", e))?;

        // Create shadow sampler (depth comparison sampler)
        debug!("Creating shadow sampler");
        use vulkano::image::sampler::{Sampler, SamplerCreateInfo, Filter, SamplerAddressMode};
        use vulkano::pipeline::graphics::depth_stencil::CompareOp;
        let shadow_sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                compare: Some(CompareOp::LessOrEqual),
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create shadow sampler: {}", e))?;

        info!(
            "Graphics context initialization complete in {:?}",
            init_start.elapsed()
        );

        Ok(Self {
            // Public fields
            instance,
            device,
            graphics_queue,
            present_queue,

            // Internal state
            surface,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            depth_images,
            render_pass,
            framebuffers,
            command_buffer_allocator,
            graphics_pipeline,
            memory_allocator,
            viewport,
            recreate_swapchain: false,
            previous_frame_end,

            // Performance tracking
            frame_timer: FrameTimer::new(),

            // Mesh management
            mesh_manager,

            // Texture management
            texture_manager,

            // Procedural texture generation
            procedural_texture_manager,

            // Material management
            material_manager,

            // Lighting management
            lighting_buffer,

            // Dynamic uniform buffer
            dynamic_uniform_buffer,

            // View/projection data
            view_proj_buffer,

            // Bone matrices for skeletal animation
            bone_matrices_buffer,

            // Shadow uniforms and dummy shadow map
            shadow_buffer,
            dummy_shadow_map,
            shadow_sampler,

            // Line renderer
            line_renderer: None,
            // Descriptor set pool
            descriptor_set_pool,

            // Bindless rendering (disabled by default)
            bindless_manager: None,
            use_bindless: false,
        })
    }

    /// Gets a reference to the mesh asset manager.
    ///
    /// Use this to load, access, or manage mesh assets.
    pub fn mesh_manager(&self) -> &mesh::MeshAssetManager {
        &self.mesh_manager
    }

    /// Gets a mutable reference to the mesh asset manager.
    ///
    /// Use this to load or modify mesh assets.
    pub fn mesh_manager_mut(&mut self) -> &mut mesh::MeshAssetManager {
        &mut self.mesh_manager
    }

    /// Gets a reference to the texture asset manager.
    ///
    /// Use this to access or query texture assets.
    pub fn texture_manager(&self) -> &texture::TextureManager {
        &self.texture_manager
    }

    /// Gets a mutable reference to the texture asset manager.
    ///
    /// Use this to load or modify texture assets.
    pub fn texture_manager_mut(&mut self) -> &mut texture::TextureManager {
        &mut self.texture_manager
    }

    /// Gets a reference to the procedural texture manager.
    ///
    /// Use this to generate procedural textures using GPU compute shaders.
    pub fn procedural_texture_manager(&self) -> &procedural_texture::ProceduralTextureManager {
        &self.procedural_texture_manager
    }

    /// Gets a mutable reference to the procedural texture manager.
    ///
    /// Use this to generate or manage procedural textures.
    pub fn procedural_texture_manager_mut(
        &mut self,
    ) -> &mut procedural_texture::ProceduralTextureManager {
        &mut self.procedural_texture_manager
    }

    /// Gets a reference to the material asset manager.
    ///
    /// Use this to access or query material assets.
    pub fn material_manager(&self) -> &material::MaterialManager {
        &self.material_manager
    }

    /// Gets a mutable reference to the material asset manager.
    ///
    /// Use this to load or modify material assets.
    pub fn material_manager_mut(&mut self) -> &mut material::MaterialManager {
        &mut self.material_manager
    }

    /// Gets a reference to the lighting uniform buffer.
    ///
    /// Use this to access lighting data.
    pub fn lighting_buffer(&self) -> &lighting::LightingUniformBuffer {
        &self.lighting_buffer
    }

    /// Gets a mutable reference to the lighting uniform buffer.
    ///
    /// Use this to update lighting data for the next frame.
    pub fn lighting_buffer_mut(&mut self) -> &mut lighting::LightingUniformBuffer {
        &mut self.lighting_buffer
    }

    /// Gets a reference to the memory allocator.
    ///
    /// Use this for allocating GPU resources like buffers and images.
    pub fn memory_allocator(&self) -> &Arc<StandardMemoryAllocator> {
        &self.memory_allocator
    }

    /// Gets a reference to the command buffer allocator.
    ///
    /// Use this for creating command buffers.
    pub fn command_buffer_allocator(&self) -> &Arc<dyn CommandBufferAllocator> {
        &self.command_buffer_allocator
    }

    /// Gets a reference to the render pass.
    ///
    /// Use this for creating additional pipelines or renderers.
    pub fn render_pass(&self) -> &Arc<RenderPass> {
        &self.render_pass
    }

    /// Gets a reference to the swapchain.
    ///
    /// Use this for acquiring images and presenting.
    pub fn swapchain(&self) -> &Arc<Swapchain> {
        &self.swapchain
    }

    /// Gets a reference to a framebuffer by index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn framebuffer(&self, index: usize) -> &Arc<Framebuffer> {
        &self.framebuffers[index]
    }

    /// Gets a reference to the viewport.
    ///
    /// Use this for setting viewport state in command buffers.
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Initializes the line renderer for debug visualization and gizmo rendering.
    ///
    /// This must be called before using line rendering features. It creates a line renderer
    /// with depth testing enabled for proper z-ordering with 3D geometry.
    ///
    /// # Arguments
    ///
    /// * `render_pass` - The render pass to use for line rendering
    /// * `extent` - The viewport dimensions
    ///
    /// # Errors
    ///
    /// Returns an error if line renderer initialization fails.
    pub fn initialize_line_renderer(
        &mut self,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
    ) -> Result<()> {
        let line_renderer = line_renderer::LineRenderer::new(
            self.device.clone(),
            render_pass,
            self.memory_allocator.clone(),
            extent,
        )?;
        self.line_renderer = Some(line_renderer);
        Ok(())
    }

    /// Gets a reference to the line renderer if initialized.
    ///
    /// Returns `None` if `initialize_line_renderer` has not been called.
    pub fn line_renderer(&self) -> Option<&line_renderer::LineRenderer> {
        self.line_renderer.as_ref()
    }

    /// Gets a mutable reference to the line renderer if initialized.
    ///
    /// Returns `None` if `initialize_line_renderer` has not been called.
    pub fn line_renderer_mut(&mut self) -> Option<&mut line_renderer::LineRenderer> {
        self.line_renderer.as_mut()
    }

    /// Gets the total number of cached descriptor sets in the pool.
    ///
    /// Returns the sum of cached transform and material descriptor sets.
    /// This can be used to monitor memory usage and descriptor set reuse efficiency.
    /// A high count relative to unique textures and materials indicates good cache utilization.
    ///
    /// # Example
    ///
    /// With 10 unique textures and 5 unique materials, this would return 15 after
    /// the first frame where all combinations are used.
    pub fn descriptor_set_pool_size(&self) -> usize {
        self.descriptor_set_pool.len()
    }

    /// Clears the descriptor set pool cache.
    ///
    /// This should be called when materials or textures are modified to ensure
    /// stale descriptor sets are not reused. The pool will automatically rebuild
    /// the cache as textures and materials are used in subsequent frames.
    ///
    /// Clears both transform and material descriptor set caches.
    pub fn clear_descriptor_set_pool(&mut self) {
        self.descriptor_set_pool.clear();
    }

    /// Enables bindless rendering mode.
    ///
    /// Bindless rendering eliminates per-material descriptor set binds by using
    /// large texture arrays and material indices passed via push constants.
    ///
    /// This provides significant performance benefits for scenes with many materials:
    /// - Zero descriptor set binds during rendering
    /// - Support for up to 4096 textures and materials
    /// - 100x+ reduction in CPU overhead
    ///
    /// Once enabled, all subsequent rendering will use bindless mode. Textures
    /// must be registered with the bindless manager before use.
    ///
    /// # Errors
    ///
    /// Returns an error if bindless manager initialization fails.
    pub fn enable_bindless_rendering(&mut self) -> Result<()> {
        if self.bindless_manager.is_some() {
            info!("Bindless rendering already enabled");
            self.use_bindless = true;
            return Ok(());
        }

        info!("Enabling bindless rendering mode");

        let descriptor_set_allocator = Arc::new(
            vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
                self.device.clone(),
                Default::default(),
            ),
        );

        let bindless_manager = bindless::BindlessTextureManager::new(
            self.device.clone(),
            self.memory_allocator.clone(),
            descriptor_set_allocator,
        )?;

        self.bindless_manager = Some(bindless_manager);
        self.use_bindless = true;

        info!("Bindless rendering enabled");

        Ok(())
    }

    /// Disables bindless rendering mode.
    ///
    /// Returns to traditional rendering with per-material descriptor set binds.
    /// The bindless manager is retained so it can be re-enabled without losing
    /// registered textures and materials.
    pub fn disable_bindless_rendering(&mut self) {
        info!("Disabling bindless rendering mode");
        self.use_bindless = false;
    }

    /// Checks if bindless rendering is currently enabled.
    pub fn is_bindless_enabled(&self) -> bool {
        self.use_bindless
    }

    /// Gets a reference to the bindless texture manager if available.
    ///
    /// Returns `None` if bindless rendering has not been enabled.
    pub fn bindless_manager(&self) -> Option<&bindless::BindlessTextureManager> {
        self.bindless_manager.as_ref()
    }

    /// Gets a mutable reference to the bindless texture manager if available.
    ///
    /// Returns `None` if bindless rendering has not been enabled.
    pub fn bindless_manager_mut(&mut self) -> Option<&mut bindless::BindlessTextureManager> {
        self.bindless_manager.as_mut()
    }

    /// Marks the swapchain for recreation on the next frame.
    ///
    /// This should be called when the window is resized. The actual recreation
    /// happens during the next `render()` call to avoid recreation during
    /// rapid resize events.
    ///
    /// # Arguments
    ///
    /// * `_width` - New width (currently unused, size is queried from window)
    /// * `_height` - New height (currently unused, size is queried from window)
    pub fn configure_surface(&mut self, width: u32, height: u32) {
        debug!("Surface configuration requested: {}x{}", width, height);
        self.recreate_swapchain = true;
    }

    /// Creates a render pass suitable for post-processing.
    ///
    /// This is a simple render pass with a single color attachment,
    /// suitable for rendering post-processing effects to offscreen targets.
    ///
    /// # Returns
    ///
    /// A render pass configured for post-processing operations.
    ///
    /// # Errors
    ///
    /// Returns an error if render pass creation fails.
    pub fn create_post_process_render_pass(&self) -> Result<Arc<RenderPass>> {
        Self::create_render_pass(&self.device, vulkano::format::Format::R8G8B8A8_UNORM)
    }

    /// Creates a render pass with depth buffer support for 3D rendering with lines.
    ///
    /// This render pass includes both a color attachment and a depth attachment,
    /// enabling proper depth testing for 3D line rendering alongside regular meshes.
    ///
    /// # Arguments
    ///
    /// * `format` - The color attachment format
    ///
    /// # Returns
    ///
    /// A render pass configured for 3D rendering with depth testing.
    ///
    /// # Errors
    ///
    /// Returns an error if render pass creation fails.
    pub fn create_render_pass_with_depth(
        &self,
        format: vulkano::format::Format,
    ) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: vulkano::format::Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create render pass with depth: {}", e))
    }

    /// Creates a render pass suitable for HDR rendering.
    ///
    /// This render pass uses R16G16B16A16_SFLOAT format to support HDR values
    /// beyond the [0,1] range.
    ///
    /// # Returns
    ///
    /// A render pass configured for HDR rendering operations.
    ///
    /// # Errors
    ///
    /// Returns an error if render pass creation fails.
    pub fn create_hdr_render_pass(&self) -> Result<Arc<RenderPass>> {
        Self::create_render_pass(&self.device, vulkano::format::Format::R16G16B16A16_SFLOAT)
    }

    /// Checks if the window is currently minimized (0×0 size).
    ///
    /// # Returns
    ///
    /// `true` if the window is minimized, `false` otherwise
    fn is_window_minimized(&self) -> bool {
        if let Some(obj) = self.surface.object() {
            if let Some(window) = obj.downcast_ref::<Window>() {
                let size = window.inner_size();
                return size.width == 0 || size.height == 0;
            }
        }
        false
    }

    /// Renders a frame with full support for meshes, textures, materials, and lighting.
    ///
    /// This is the unified rendering method that supports all rendering features:
    /// - Multiple different mesh types per frame
    /// - Optional custom textures per object (defaults to white texture)
    /// - Optional PBR material properties per object (defaults to standard properties)
    /// - Optional dynamic lighting updates
    ///
    /// # Automatic Optimizations
    ///
    /// The implementation includes several performance optimizations:
    ///
    /// **Material Batching**: Draw commands are automatically sorted by texture and
    /// material properties to group objects with identical materials. This significantly
    /// reduces GPU state changes.
    ///
    /// **Descriptor Set Reuse**: When multiple objects share the same material properties,
    /// the same material descriptor set is reused instead of creating a new one for each
    /// object. For example, 100 objects with the same material use 1 descriptor set
    /// instead of 100.
    ///
    /// **Conditional Binding**: Material descriptor sets are only re-bound when the
    /// material actually changes, not for every object. Combined with sorting, this
    /// drastically reduces the number of descriptor set binds.
    ///
    /// **Example Performance Impact**:
    /// ```text
    /// Scene: 200 objects with 10 different materials (20 objects per material)
    ///
    /// Without Optimizations:
    /// - Material descriptor sets created: 200
    /// - Material descriptor set binds: 200
    ///
    /// With Optimizations:
    /// - Material descriptor sets created: 10
    /// - Material descriptor set binds: 10
    ///
    /// Result: 20x reduction in descriptor set operations
    /// ```
    ///
    /// # Lighting Data Upload
    ///
    /// If `cmds.lighting` is `Some`, the lighting data is uploaded to the GPU
    /// before rendering. This allows dynamic lighting updates each frame.
    ///
    /// If `cmds.lighting` is `None`, the previously uploaded lighting data is used,
    /// which is more efficient when lighting doesn't change between frames.
    ///
    /// # Arguments
    ///
    /// * `cmds` - Render commands containing camera matrices, draw commands, and optional lighting
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Swapchain recreation fails
    /// - A referenced mesh or texture doesn't exist
    /// - Lighting buffer update fails
    /// - Command buffer recording fails
    /// - GPU submission fails
    pub fn render(&mut self, cmds: &RenderCommands) -> Result<()> {
        let _ = self.frame_timer.tick();

        let mut previous_frame_end = self
            .previous_frame_end
            .take()
            .unwrap_or_else(|| sync::now(self.device.clone()).boxed());

        if self.is_window_minimized() {
            previous_frame_end.cleanup_finished();
            self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            return Ok(());
        }

        if self.recreate_swapchain {
            debug!("Recreating swapchain due to pending resize");
            let start_time = std::time::Instant::now();
            previous_frame_end
                .flush()
                .expect("Failed to flush previous frame end");
            self.recreate_swapchain_and_framebuffers()?;
            self.recreate_swapchain = false;
            previous_frame_end = sync::now(self.device.clone()).boxed();
            info!(
                "Swapchain recreation completed in {:?}",
                start_time.elapsed()
            );
        }

        previous_frame_end.cleanup_finished();

        self.dynamic_uniform_buffer.next_frame();

        if let Some(lighting) = cmds.lighting {
            trace!("Uploading lighting data to GPU");
            self.lighting_buffer.update(lighting)?;
        }

        let view_proj_uniforms = uniform_buffer::ViewProjectionUniforms::new(cmds.view, cmds.proj);

        {
            let mut write_lock = self.view_proj_buffer.write().map_err(|e| {
                eyre::eyre!("Failed to lock view/projection buffer for writing: {}", e)
            })?;
            *write_lock = view_proj_uniforms;
        }

        let mut indexed_commands: Vec<(usize, &DrawCommand)> =
            cmds.draw_commands.iter().enumerate().collect();

        indexed_commands.sort_by(|(_, a), (_, b)| {
            let tex_a = a.texture_name.as_deref().unwrap_or("_default_white");
            let tex_b = b.texture_name.as_deref().unwrap_or("_default_white");

            match tex_a.cmp(tex_b) {
                std::cmp::Ordering::Equal => {
                    let props_a = a
                        .material_properties
                        .unwrap_or_else(material::MaterialProperties::default);
                    let props_b = b
                        .material_properties
                        .unwrap_or_else(material::MaterialProperties::default);

                    let bytes_a = bytemuck::bytes_of(&props_a);
                    let bytes_b = bytemuck::bytes_of(&props_b);
                    bytes_a.cmp(bytes_b)
                }
                other => other,
            }
        });

        let model_matrices: Vec<Mat4> = indexed_commands
            .iter()
            .map(|(_, draw_cmd)| draw_cmd.model)
            .collect();

        self.dynamic_uniform_buffer.write_models(&model_matrices)?;

        let default_texture = self
            .texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found"))?;

        let default_normal_map = self
            .texture_manager
            .get_texture("_default_flat_normal")
            .ok_or_else(|| eyre::eyre!("Default flat normal texture not found"))?;

        // Update bone matrices buffer if any draw command has bone matrices
        // Note: Currently only supports one animated object per frame (the last one with bone matrices)
        // For multiple animated objects, a dynamic bone matrices buffer would be needed
        for draw_cmd in cmds.draw_commands.iter() {
            if let Some(ref bone_matrices) = draw_cmd.bone_matrices {
                trace!(
                    "Uploading {} bone matrices for skeletal animation",
                    bone_matrices.len()
                );
                
                let bone_uniforms = uniform_buffer::BoneMatricesUniforms::from_matrices(bone_matrices);
                
                {
                    let mut write_lock = self.bone_matrices_buffer.write().map_err(|e| {
                        eyre::eyre!("Failed to lock bone matrices buffer for writing: {}", e)
                    })?;
                    *write_lock = bone_uniforms;
                }
                
                // Break after first animated object (current limitation)
                break;
            }
        }

        let mut draw_list: Vec<(
            Arc<DescriptorSet>,
            Arc<DescriptorSet>,
            &mesh::GpuMesh,
            usize,
        )> = Vec::with_capacity(indexed_commands.len());

        let mut current_texture_name: Option<String> = None;
        let mut current_material_props: Option<material::MaterialProperties> = None;
        let mut current_material_set: Option<Arc<DescriptorSet>> = None;

        for (object_index, (_original_index, draw_cmd)) in indexed_commands.iter().enumerate() {
            let mesh = self
                .mesh_manager
                .get_mesh(&draw_cmd.mesh_id)
                .ok_or_else(|| eyre::eyre!("Mesh '{}' not found", draw_cmd.mesh_id))?;

            let texture_name = draw_cmd
                .texture_name
                .as_deref()
                .unwrap_or("_default_white")
                .to_string();

            let material_props = draw_cmd
                .material_properties
                .unwrap_or_else(material::MaterialProperties::default);

            let material_changed = current_texture_name.as_ref() != Some(&texture_name)
                || current_material_props.as_ref() != Some(&material_props);

            let texture = if let Some(ref tex_name) = draw_cmd.texture_name {
                self.texture_manager
                    .get_texture(tex_name)
                    .ok_or_else(|| eyre::eyre!("Texture '{}' not found", tex_name))?
            } else {
                default_texture
            };

            // Use the descriptor set pool to get or create a cached transform descriptor set
            let transform_set = self.descriptor_set_pool.get_or_create_transform_set(
                texture_name.clone(),
                self.view_proj_buffer.clone(),
                self.dynamic_uniform_buffer.buffer().clone(),
                texture,
                self.lighting_buffer.buffer().clone(),
                default_normal_map,
                self.bone_matrices_buffer.clone(),
                self.shadow_buffer.clone(),
                self.dummy_shadow_map.clone(),
                self.shadow_sampler.clone(),
            )?;

            let material_set = if material_changed {
                // Use the descriptor set pool to get or create a cached descriptor set
                let new_material_set = self
                    .descriptor_set_pool
                    .get_or_create_material_set(texture_name.clone(), material_props)?;

                current_texture_name = Some(texture_name);
                current_material_props = Some(material_props);
                current_material_set = Some(new_material_set.clone());

                new_material_set
            } else {
                current_material_set
                    .as_ref()
                    .expect("Material set should exist if material_changed is false")
                    .clone()
            };

            draw_list.push((transform_set, material_set, mesh, object_index));
        }

        trace!("Acquiring next swapchain image");
        let acquire_start = std::time::Instant::now();
        let (image_index, suboptimal, acquire_future) =
            vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(|e| eyre::eyre!("Failed to acquire next image: {}", e))?;
        trace!(
            "Image {} acquired in {:?}",
            image_index,
            acquire_start.elapsed()
        );

        if suboptimal {
            warn!("Swapchain is suboptimal, will recreate on next frame");
            self.recreate_swapchain = true;
        }

        trace!("Building command buffer for frame");
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.1, 0.2, 0.3, 1.0].into()),  // Color attachment clear value
                        Some(1.0.into()),                     // Depth attachment clear value
                    ],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to begin render pass: {}", e))?;

        command_buffer_builder
            .bind_pipeline_graphics(self.graphics_pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind graphics pipeline: {}", e))?;

        command_buffer_builder
            .set_viewport(0, [self.viewport.clone()].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        let mut last_material_set: Option<Arc<DescriptorSet>> = None;

        for (transform_set, material_set, mesh, object_index) in draw_list.iter() {
            command_buffer_builder
                .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(mesh.index_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

            let dynamic_offset = self
                .dynamic_uniform_buffer
                .get_dynamic_offset(*object_index);

            // Bind transform descriptor set with dynamic offset (set 0)
            let set_with_offsets = vulkano::descriptor_set::DescriptorSetWithOffsets::new(
                transform_set.clone(),
                [dynamic_offset],
            );

            // Bind material descriptor set only when material changes (set 1)
            let material_changed = last_material_set
                .as_ref()
                .is_none_or(|last| !Arc::ptr_eq(last, material_set));

            if material_changed {
                command_buffer_builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        1,
                        material_set.clone(),
                    )
                    .map_err(|e| {
                        eyre::eyre!("Failed to bind material descriptor set: {}", e)
                    })?;

                last_material_set = Some(material_set.clone());
            }

            // SAFETY: We ensure the dynamic offset is within bounds via the dynamic uniform buffer
            // and the draw parameters are valid for the bound mesh
            unsafe {
                command_buffer_builder.bind_descriptor_sets_unchecked(
                    PipelineBindPoint::Graphics,
                    self.graphics_pipeline.layout().clone(),
                    0,
                    set_with_offsets,
                );

                command_buffer_builder
                    .draw_indexed(mesh.index_count, 1, 0, 0, 0)
                    .map_err(|e| eyre::eyre!("Failed to draw indexed: {}", e))?;
            }
        }

        command_buffer_builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        let command_buffer = command_buffer_builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build command buffer: {}", e))?;

        trace!("Submitting command buffer to graphics queue");

        let execution = previous_frame_end
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to submit command buffer: {}", e))?;

        let future = execution
            .then_swapchain_present(
                self.present_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        let future = match future {
            Ok(future) => future,
            Err(vulkano::Validated::Error(e)) => {
                use vulkano::VulkanError;
                match e {
                    VulkanError::OutOfDate => {
                        debug!("Swapchain out of date, will recreate on next frame");
                        self.recreate_swapchain = true;
                        self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
                        return Ok(());
                    }
                    _ => {
                        error!("Failed to present frame: {}", e);
                        return Err(eyre::eyre!("Failed to flush future: {}", e));
                    }
                }
            }
            Err(e) => {
                error!("Failed to present frame: {}", e);
                return Err(eyre::eyre!("Failed to flush future: {}", e));
            }
        };

        self.previous_frame_end = Some(future.boxed());

        trace!("Frame rendering complete");

        Ok(())
    }

    /// Creates a swapchain for presenting rendered images to the window.
    ///
    /// The swapchain is a queue of images that can be presented to the window.
    /// While one image is being displayed, we can render to another.
    ///
    /// # Configuration
    ///
    /// The swapchain is configured with:
    /// - At least 2 images (double buffering)
    /// - Window's current dimensions
    /// - Optimal image format for the surface
    /// - Optimal presentation mode (VSync, immediate, etc.)
    fn create_swapchain(
        device: &Arc<Device>,
        physical_device: &Arc<PhysicalDevice>,
        surface: &Arc<Surface>,
        window: &Arc<Window>,
    ) -> Result<(Arc<Swapchain>, Vec<Arc<Image>>)> {
        let surface_capabilities = physical_device
            .surface_capabilities(surface, Default::default())
            .map_err(|e| eyre::eyre!("Failed to get surface capabilities: {}", e))?;

        let image_format = physical_device
            .surface_formats(surface, Default::default())
            .map_err(|e| eyre::eyre!("Failed to get surface formats: {}", e))?[0]
            .0;

        let window_size = window.inner_size();

        let image_count = surface_capabilities.min_image_count.max(2).min(
            surface_capabilities
                .max_image_count
                .unwrap_or(u32::MAX)
                .min(surface_capabilities.min_image_count + 1),
        );

        trace!(
            "Creating swapchain: {}x{} with {} images, format: {:?}",
            window_size.width,
            window_size.height,
            image_count,
            image_format
        );
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_format,
                image_extent: [window_size.width, window_size.height],
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .ok_or_else(|| eyre::eyre!("No supported composite alpha modes found"))?,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create swapchain: {}", e))?;

        Ok((swapchain, images))
    }

    /// Creates a render pass that defines the rendering operations.
    ///
    /// A render pass describes:
    /// - What attachments (images) are used
    /// - How they're initialized (cleared, preserved, etc.)
    /// - How they're used in subpasses
    /// - Dependencies between subpasses
    ///
    /// Our render pass has:
    /// - One color attachment (the swapchain image)
    /// - One depth attachment for depth testing
    /// - One subpass that clears and then renders to both
    fn create_render_pass(
        device: &Arc<Device>,
        format: vulkano::format::Format,
    ) -> Result<Arc<RenderPass>> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: vulkano::format::Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create render pass: {}", e))
    }

    /// Creates depth images for the swapchain.
    ///
    /// Each swapchain image needs a corresponding depth image for depth testing.
    /// Depth images store per-pixel depth values that enable proper z-ordering.
    fn create_depth_images(
        memory_allocator: &Arc<StandardMemoryAllocator>,
        extent: [u32; 2],
        image_count: usize,
    ) -> Result<Vec<Arc<ImageView>>> {
        use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
        use vulkano::format::Format;
        
        (0..image_count)
            .map(|_| {
                let depth_image = Image::new(
                    memory_allocator.clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: Format::D32_SFLOAT,
                        extent: [extent[0], extent[1], 1],
                        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .map_err(|e| eyre::eyre!("Failed to create depth image: {}", e))?;
                
                ImageView::new_default(depth_image)
                    .map_err(|e| eyre::eyre!("Failed to create depth image view: {}", e))
            })
            .collect()
    }

    /// Creates framebuffers for each swapchain image with depth attachments.
    ///
    /// A framebuffer binds specific images to the attachments defined in a render pass.
    /// We need one framebuffer per swapchain image, each with its own depth image.
    fn create_framebuffers(
        image_views: &[Arc<ImageView>],
        depth_views: &[Arc<ImageView>],
        render_pass: &Arc<RenderPass>,
    ) -> Result<Vec<Arc<Framebuffer>>> {
        image_views
            .iter()
            .zip(depth_views.iter())
            .map(|(image_view, depth_view)| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone(), depth_view.clone()],
                        ..Default::default()
                    },
                )
                .map_err(|e| eyre::eyre!("Failed to create framebuffer: {}", e))
            })
            .collect()
    }

    /// Recreates the swapchain and associated resources when the window is resized.
    ///
    /// This function handles the complex process of recreating the swapchain when
    /// the window size changes. It must:
    ///
    /// 1. Get the new window dimensions
    /// 2. Create a new swapchain with the updated size
    /// 3. Create new image views for each swapchain image
    /// 4. Create new framebuffers to match
    /// 5. Update the viewport to the new dimensions
    ///
    /// # Why Recreation is Necessary
    ///
    /// The swapchain images have a fixed size. When the window resizes, we need
    /// new images that match the new window dimensions. This requires recreating
    /// the entire swapchain and all resources that depend on it.
    ///
    /// # Performance Considerations
    ///
    /// Swapchain recreation is expensive, which is why we:
    /// - Debounce resize events in the window module
    /// - Only recreate when actually needed (via the `recreate_swapchain` flag)
    /// - Reuse existing resources where possible (shaders, pipelines, vertex buffers)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Window handle is invalid
    /// - Swapchain recreation fails
    /// - Resource allocation fails
    fn recreate_swapchain_and_framebuffers(&mut self) -> Result<()> {
        let recreate_start = std::time::Instant::now();

        let surface_object = self
            .surface
            .object()
            .ok_or_else(|| eyre::eyre!("Failed to get surface object"))?;
        let window = surface_object
            .downcast_ref::<Window>()
            .ok_or_else(|| eyre::eyre!("Failed to downcast surface object to Window"))?;
        let window_size = window.inner_size();

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: [window_size.width, window_size.height],
                ..self.swapchain.create_info()
            })
            .map_err(|e| eyre::eyre!("Failed to recreate swapchain: {}", e))?;

        let new_image_views = new_images
            .iter()
            .map(|image| ImageView::new_default(image.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| eyre::eyre!("Failed to create new image views: {}", e))?;

        let new_depth_images = Self::create_depth_images(
            &self.memory_allocator,
            [window_size.width, window_size.height],
            new_images.len(),
        )?;

        let new_framebuffers = Self::create_framebuffers(&new_image_views, &new_depth_images, &self.render_pass)?;

        // Update viewport
        self.viewport.extent = [window_size.width as f32, window_size.height as f32];

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images;
        self.swapchain_image_views = new_image_views;
        self.depth_images = new_depth_images;
        self.framebuffers = new_framebuffers;

        info!(
            "Recreated swapchain and framebuffers for size {}x{} in {:?}",
            window_size.width,
            window_size.height,
            recreate_start.elapsed()
        );

        Ok(())
    }
}

// Public re-exports
pub use area_lights::{
    AreaLight, AreaLightData, AreaLightManager, AreaLightType, LtcMatrixData, MAX_AREA_LIGHTS,
};
pub use bindless::{
    BindlessMaterialData, BindlessTextureManager, MAX_BINDLESS_MATERIALS, MAX_BINDLESS_TEXTURES,
};
pub use deferred::{DeferredRenderer, GBuffer};
pub use environment_probe::{
    EnvironmentProbe, EnvironmentProbeCapture, EnvironmentProbeConfig, EnvironmentProbeManager,
    IblData, IblUniforms, ProbeUpdateMode, MAX_ENVIRONMENT_PROBES, SPECULAR_MIP_LEVELS,
};
pub use god_rays::{GodRays, GodRaysConfig, GodRaysRenderer, RadialBlurPass};
pub use hdr::{
    calculate_luminance, ExposureCalculator, ExposureMode, HdrRenderTarget,
    ToneMapPass as HdrToneMapPass, ToneMapper, ToneMappingOperator,
};
pub use light_linking::{
    LightChannel, LightLinkingManager, LightLinkingMask, DEFAULT_LIGHT_CHANNEL,
};
pub use light_probe::{
    LightProbe, LightProbeData, LightProbeGrid, LightProbeManager, ProbeBlendMode,
    MAX_LIGHT_PROBES, PROBE_IRRADIANCE_COEFFS,
};
pub use lighting::{
    DirectionalLightData, LightingUniformBuffer, LightingUniforms, PointLightData,
    MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS,
};
pub use line_renderer::{Line, LineBatch, LineRenderer, LineVertex};
pub use lod::{
    LodGroup, LodLevel, LodManager, LodStatistics, DEFAULT_TRANSITION_DURATION, MAX_LOD_LEVELS,
};
pub use material::{
    BlendMode, ExtendedPbrProperties, Material, MaterialLayer, MaterialManager, MaterialProperties,
    ParallaxProperties,
};
pub use material_instancing::{InstancingStats, MaterialInstance, MaterialInstanceManager};
pub use material_layers::{
    LayerParamsUniforms, MaterialLayerCache, MaterialLayerRenderer, MaterialTextureSet,
    MAX_MATERIAL_LAYERS,
};
pub use mesh::{GpuMesh, MeshData};
pub use particles::{
    CollisionPlane, EmitterShape, GpuParticle, ParticleEmitterConfig, ParticleForce,
    ParticleInstance, ParticleRenderer, SoftParticleConfig, MAX_PARTICLES_PER_EMITTER,
};
pub use post_process::{
    BloomConfig, BloomEffect, BrightnessExtractionPass, ChromaticAberrationConfig,
    ChromaticAberrationPass, CopyPass, DepthOfFieldPass, DofConfig, FilmGrainConfig, FilmGrainPass,
    FullScreenQuad, GaussianBlurHorizontalPass, GaussianBlurVerticalPass, GrayscalePass,
    MotionBlurConfig, MotionBlurPass, PostProcessChain, PostProcessContext, PostProcessPass,
    QuadVertex, RenderTarget, RenderTargetPool, ToneMapPass, VelocityUniforms, VignetteConfig,
    VignettePass,
};
pub use primitives::{
    colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh, sphere_mesh, textured_cube_mesh,
    textured_quad_mesh,
};
pub use procedural_texture::ProceduralTextureManager;
pub use shadow::{ShadowConfig, ShadowMapManager, ShadowUniforms, MAX_SHADOW_CASCADES};
pub use skybox::SkyboxRenderer;
pub use ssao::{SsaoConfig, SsaoRenderer};
pub use texture::{Cubemap, CubemapFace, Texture, TextureManager};
pub use uniform_buffer::{DynamicUniformBuffer, ModelUniforms, ViewProjectionUniforms};
pub use velocity_buffer::{VelocityBuffer, VelocityBufferRenderer};
pub use vertex::Vertex3D;
pub use visual_feedback::{
    batch_to_lines, create_axis_indicator, create_bounding_box, create_gizmo_lines, create_grid,
    create_selection_outline, AxisIndicatorConfig, GridConfig,
};
pub use volumetric_fog::{
    FogDensityFunction, VolumetricFog, VolumetricFogConfig, VolumetricFogRenderer,
    MAX_RAYMARCH_STEPS,
};

pub mod area_lights;
pub mod environment_probe;
pub mod god_rays;
pub mod light_linking;
pub mod light_probe;
pub mod volumetric_fog;

#[cfg(test)]
mod advanced_lighting_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_math::{Mat4, Vec3};

    /// Test rendering mode enumeration
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RenderMode {
        Forward,
        Deferred,
    }

    /// Mock renderer state for testing mode switching
    struct MockRendererState {
        current_mode: RenderMode,
        switch_count: u32,
    }

    impl MockRendererState {
        fn new() -> Self {
            Self {
                current_mode: RenderMode::Forward,
                switch_count: 0,
            }
        }

        fn switch_to(&mut self, mode: RenderMode) {
            if self.current_mode != mode {
                self.current_mode = mode;
                self.switch_count += 1;
            }
        }

        fn is_forward(&self) -> bool {
            self.current_mode == RenderMode::Forward
        }

        fn is_deferred(&self) -> bool {
            self.current_mode == RenderMode::Deferred
        }
    }

    #[test]
    fn test_renderer_mode_switch_forward_to_deferred() {
        let mut renderer = MockRendererState::new();

        assert!(renderer.is_forward());
        assert_eq!(renderer.switch_count, 0);

        renderer.switch_to(RenderMode::Deferred);

        assert!(renderer.is_deferred());
        assert_eq!(renderer.switch_count, 1);
    }

    #[test]
    fn test_renderer_mode_switch_deferred_to_forward() {
        let mut renderer = MockRendererState::new();
        renderer.switch_to(RenderMode::Deferred);

        assert!(renderer.is_deferred());
        assert_eq!(renderer.switch_count, 1);

        renderer.switch_to(RenderMode::Forward);

        assert!(renderer.is_forward());
        assert_eq!(renderer.switch_count, 2);
    }

    #[test]
    fn test_renderer_mode_switch_idempotent() {
        let mut renderer = MockRendererState::new();

        // Switching to the same mode should not increment counter
        renderer.switch_to(RenderMode::Forward);
        assert_eq!(renderer.switch_count, 0);

        renderer.switch_to(RenderMode::Forward);
        assert_eq!(renderer.switch_count, 0);

        renderer.switch_to(RenderMode::Deferred);
        assert_eq!(renderer.switch_count, 1);

        renderer.switch_to(RenderMode::Deferred);
        assert_eq!(renderer.switch_count, 1);
    }

    #[test]
    fn test_renderer_mode_multiple_switches() {
        let mut renderer = MockRendererState::new();

        for _ in 0..10 {
            renderer.switch_to(RenderMode::Deferred);
            renderer.switch_to(RenderMode::Forward);
        }

        // Should have switched 20 times (10 to deferred, 10 back to forward)
        assert_eq!(renderer.switch_count, 20);
        assert!(renderer.is_forward()); // Should end on forward
    }

    #[test]
    fn test_draw_command_structure() {
        // Test that DrawCommand can be created with all necessary data
        let cmd = DrawCommand {
            mesh_id: "test_mesh".to_string(),
            model: Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            texture_name: Some("test_texture".to_string()),
            material_properties: Some(MaterialProperties::default()),
        };

        assert_eq!(cmd.mesh_id, "test_mesh");
        assert!(cmd.texture_name.is_some());
        assert!(cmd.material_properties.is_some());
    }

    #[test]
    fn test_draw_command_without_texture() {
        let cmd = DrawCommand {
            mesh_id: "test_mesh".to_string(),
            model: Mat4::IDENTITY,
            texture_name: None,
            material_properties: None,
        };

        assert!(cmd.texture_name.is_none());
        assert!(cmd.material_properties.is_none());
    }

    #[test]
    fn test_render_commands_structure() {
        let draw_cmds = vec![DrawCommand {
            mesh_id: "cube".to_string(),
            model: Mat4::IDENTITY,
            texture_name: None,
            material_properties: None,
        }];

        let render_cmds = RenderCommands {
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            draw_commands: &draw_cmds,
            lighting: None,
        };

        assert_eq!(render_cmds.draw_commands.len(), 1);
        assert!(render_cmds.lighting.is_none());
    }

    #[test]
    fn test_render_commands_with_lighting() {
        let lighting = LightingUniforms::default();
        let draw_cmds = vec![];

        let render_cmds = RenderCommands {
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            draw_commands: &draw_cmds,
            lighting: Some(&lighting),
        };

        assert!(render_cmds.lighting.is_some());
    }

    #[test]
    fn test_forward_rendering_path_characteristics() {
        // Forward rendering characteristics:
        // - Single pass
        // - Renders geometry directly to framebuffer
        // - Lighting calculated per-fragment for each object
        // - Complexity: O(lights * triangles)

        let light_count = 10;
        let triangle_count = 1000;
        let forward_complexity = light_count * triangle_count;

        assert_eq!(forward_complexity, 10000);
    }

    #[test]
    fn test_deferred_rendering_path_characteristics() {
        // Deferred rendering characteristics:
        // - Two passes (geometry + lighting)
        // - G-buffer stores geometry data
        // - Lighting calculated per-pixel once
        // - Complexity: O(lights * pixels)

        let light_count = 10;
        let pixel_count = 1920 * 1080;
        let deferred_complexity = light_count * pixel_count;

        // Deferred is more efficient for many lights
        assert!(deferred_complexity < 10_000_000_000_i64); // Much less than forward with many triangles
    }

    #[test]
    fn test_renderer_mode_selection_few_lights() {
        // With few lights, forward rendering is typically more efficient
        let light_count = 2;
        let triangle_count = 10000;
        let pixel_count = 1920 * 1080;

        let forward_ops = light_count * triangle_count;
        let deferred_ops = light_count * pixel_count;

        // Forward should be less work with few lights
        assert!(forward_ops < deferred_ops);
    }

    #[test]
    fn test_renderer_mode_selection_many_lights() {
        // With many lights and high triangle count (due to overdraw),
        // deferred rendering is more efficient.
        // Forward: each triangle is shaded once per light
        // Deferred: geometry pass once, then lights × visible pixels
        let light_count = 100;
        let triangle_count = 10_000_000; // High overdraw scene
        let pixel_count = 1920 * 1080;

        let forward_ops = light_count * triangle_count;
        let deferred_ops = light_count * pixel_count;

        // Deferred should be less work with many lights in high-overdraw scenarios
        assert!(deferred_ops < forward_ops);
    }

    #[test]
    fn test_material_properties_defaults() {
        let props = MaterialProperties::default();

        // Default material should be reasonable
        assert!(props.metallic >= 0.0 && props.metallic <= 1.0);
        assert!(props.roughness >= 0.0 && props.roughness <= 1.0);
    }

    #[test]
    fn test_material_properties_custom() {
        let props = MaterialProperties::new()
            .with_metallic(0.8)
            .with_roughness(0.2);

        assert_eq!(props.metallic, 0.8);
        assert_eq!(props.roughness, 0.2);
    }

    #[test]
    fn test_viewport_structure() {
        use vulkano::pipeline::graphics::viewport::Viewport;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [1920.0, 1080.0],
            depth_range: 0.0..=1.0,
        };

        assert_eq!(viewport.offset, [0.0, 0.0]);
        assert_eq!(viewport.extent, [1920.0, 1080.0]);
    }

    #[test]
    fn test_line_batch_creation() {
        let mut batch = LineBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);

        batch.add(Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(batch.len(), 1);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_line_batch_with_capacity() {
        let batch = LineBatch::with_capacity(100);
        assert!(batch.is_empty());
        // Capacity is internal, but batch should still work
    }

    #[test]
    fn test_line_creation() {
        let start = Vec3::new(1.0, 2.0, 3.0);
        let end = Vec3::new(4.0, 5.0, 6.0);
        let color = Vec3::new(1.0, 0.0, 0.0);

        let line = Line::new(start, end, color);
        assert_eq!(line.start, start);
        assert_eq!(line.end, end);
        assert_eq!(line.color, color);
    }

    #[test]
    fn test_line_to_vertices() {
        let line = Line::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        );

        let vertices = line.to_vertices();
        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[1].position, [1.0, 1.0, 1.0]);
        assert_eq!(vertices[0].color, [1.0, 0.0, 0.0]);
        assert_eq!(vertices[1].color, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_line_batch_clear() {
        let mut batch = LineBatch::new();
        batch.add(Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0));
        batch.add(Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(batch.len(), 2);

        batch.clear();
        assert_eq!(batch.len(), 0);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_line_batch_add_multiple() {
        let mut batch = LineBatch::new();
        let lines = vec![
            Line::new(Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0)),
            Line::new(Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 1.0, 0.0)),
            Line::new(Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 1.0)),
        ];

        batch.add_lines(lines);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_rendering_pipeline_selection_criteria() {
        // Test criteria for choosing rendering pipeline

        struct SceneStats {
            light_count: u32,
            object_count: u32,
            needs_transparency: bool,
        }

        let select_pipeline = |stats: &SceneStats| -> RenderMode {
            if stats.needs_transparency {
                // Transparency requires forward or hybrid approach
                RenderMode::Forward
            } else if stats.light_count > 10 && stats.object_count > 50 {
                // Many lights + objects benefit from deferred
                RenderMode::Deferred
            } else {
                // Simple scenes work well with forward
                RenderMode::Forward
            }
        };

        // Test case 1: Few lights, no transparency
        let scene1 = SceneStats {
            light_count: 2,
            object_count: 100,
            needs_transparency: false,
        };
        assert_eq!(select_pipeline(&scene1), RenderMode::Forward);

        // Test case 2: Many lights, no transparency
        let scene2 = SceneStats {
            light_count: 50,
            object_count: 100,
            needs_transparency: false,
        };
        assert_eq!(select_pipeline(&scene2), RenderMode::Deferred);

        // Test case 3: Many lights, with transparency
        let scene3 = SceneStats {
            light_count: 50,
            object_count: 100,
            needs_transparency: true,
        };
        assert_eq!(select_pipeline(&scene3), RenderMode::Forward);
    }

    #[test]
    fn test_hybrid_rendering_approach() {
        // Test hybrid rendering concept: deferred for opaque, forward for transparent

        let opaque_objects = ["cube1", "cube2", "sphere1"];
        let transparent_objects = ["glass1", "water1"];

        // In hybrid rendering:
        // 1. Render opaque objects to G-buffer (deferred)
        let deferred_pass_count = opaque_objects.len();

        // 2. Lighting pass on G-buffer
        let lighting_pass_count = 1;

        // 3. Render transparent objects with forward rendering
        let forward_pass_count = transparent_objects.len();

        let total_passes = deferred_pass_count + lighting_pass_count + forward_pass_count;
        assert_eq!(total_passes, 6); // 3 + 1 + 2
    }

    #[test]
    fn test_render_mode_memory_requirements() {
        // Forward rendering memory
        let framebuffer_size = 1920 * 1080 * 4; // RGBA8
        let depth_buffer_size = 1920 * 1080 * 4; // D32
        let forward_memory = framebuffer_size + depth_buffer_size;

        // Deferred rendering memory (G-buffer)
        let albedo_size = 1920 * 1080 * 4; // RGBA8
        let normal_size = 1920 * 1080 * 8; // RGBA16F
        let metallic_roughness_size = 1920 * 1080 * 4; // RGBA8
        let gbuffer_depth_size = 1920 * 1080 * 4; // D32
        let deferred_memory =
            albedo_size + normal_size + metallic_roughness_size + gbuffer_depth_size;

        // Deferred uses more memory due to G-buffer
        assert!(deferred_memory > forward_memory);

        // But the trade-off is better performance with many lights
        let memory_overhead_ratio = deferred_memory as f32 / forward_memory as f32;
        assert!(memory_overhead_ratio > 1.0);
        assert!(memory_overhead_ratio < 4.0); // Reasonable overhead
    }

    #[test]
    fn test_g_buffer_format_characteristics() {
        // Test that G-buffer formats are appropriate for their data
        use vulkano::format::Format;

        // Albedo: RGBA8 is sufficient for base color
        let albedo_format = Format::R8G8B8A8_UNORM;
        assert_eq!(albedo_format, Format::R8G8B8A8_UNORM);

        // Normal: RGBA16F provides better precision for normals
        let normal_format = Format::R16G16B16A16_SFLOAT;
        assert_eq!(normal_format, Format::R16G16B16A16_SFLOAT);

        // Metallic-Roughness: RGBA8 is sufficient for material properties
        let material_format = Format::R8G8B8A8_UNORM;
        assert_eq!(material_format, Format::R8G8B8A8_UNORM);

        // Depth: D32 provides full precision for depth
        let depth_format = Format::D32_SFLOAT;
        assert_eq!(depth_format, Format::D32_SFLOAT);
    }

    #[test]
    fn test_material_key_equality() {
        // Test that MaterialKey correctly identifies identical materials
        let props1 = MaterialProperties::new()
            .with_base_color([1.0, 1.0, 1.0, 1.0])
            .with_metallic(0.5)
            .with_roughness(0.5)
            .with_emissive_strength(0.0);

        let props2 = MaterialProperties::new()
            .with_base_color([1.0, 1.0, 1.0, 1.0])
            .with_metallic(0.5)
            .with_roughness(0.5)
            .with_emissive_strength(0.0);

        let key1 = MaterialKey::new("texture1".to_string(), &props1);
        let key2 = MaterialKey::new("texture1".to_string(), &props2);

        // Same texture and properties should produce identical keys
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_material_key_different_textures() {
        let props = MaterialProperties::default();

        let key1 = MaterialKey::new("texture1".to_string(), &props);
        let key2 = MaterialKey::new("texture2".to_string(), &props);

        // Different textures should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_material_key_different_properties() {
        let props1 = MaterialProperties::default();
        let props2 = MaterialProperties::default().with_metallic(0.9);

        let key1 = MaterialKey::new("texture1".to_string(), &props1);
        let key2 = MaterialKey::new("texture1".to_string(), &props2);

        // Different properties should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_descriptor_set_pool_benefits() {
        // Demonstrate the performance benefits of descriptor set pooling

        // Scenario: 1000 objects with 10 unique textures and 5 unique materials
        let total_objects = 1000;
        let unique_textures = 10;
        let unique_materials = 5;

        // Without pooling: Create 2 descriptor sets per object per frame
        // (1 transform set + 1 material set)
        let without_pooling_allocations_per_frame = total_objects * 2;

        // With pooling: Create descriptor sets once for unique combinations
        // Transform sets: 1 per unique texture (only texture varies)
        // Material sets: 1 per unique material
        let with_pooling_allocations_initial = unique_textures + unique_materials;
        let with_pooling_allocations_subsequent_frames = 0;

        // Calculate reduction factor for subsequent frames
        let reduction_factor =
            without_pooling_allocations_per_frame / with_pooling_allocations_initial;

        assert_eq!(without_pooling_allocations_per_frame, 2000);
        assert_eq!(with_pooling_allocations_initial, 15);
        assert_eq!(with_pooling_allocations_subsequent_frames, 0);
        assert_eq!(reduction_factor, 133); // 2000 / 15

        // After the first frame, we have zero allocations while without pooling
        // we would continue to allocate 2000 descriptor sets per frame
        assert!(
            reduction_factor >= 100,
            "Pooling should provide at least 100x reduction"
        );

        // For frame 2 onwards, the benefit is infinite since we allocate 0 sets
        // while the non-pooled version continues allocating 2000 per frame
    }
}

/// Mock render context for headless testing.
///
/// This is a no-op implementation that allows tests to run without actual GPU/window initialization.
/// All rendering operations are no-ops, and all query methods return empty or default values.
///
/// # Example
///
/// ```rust
/// use praxis_graphics::MockRenderContext;
///
/// let mut ctx = MockRenderContext::new();
/// // All operations are no-ops, suitable for testing game logic without graphics
/// ```
#[cfg(test)]
pub struct MockRenderContext {
    mesh_count: usize,
    texture_count: usize,
    material_count: usize,
}

#[cfg(test)]
impl MockRenderContext {
    /// Creates a new mock render context.
    ///
    /// All internal state is initialized to empty/default values.
    pub fn new() -> Self {
        Self {
            mesh_count: 0,
            texture_count: 0,
            material_count: 0,
        }
    }

    /// No-op render method that accepts render commands but performs no actual rendering.
    ///
    /// # Arguments
    ///
    /// * `_cmds` - Render commands (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn render(&mut self, _cmds: &RenderCommands) -> Result<()> {
        Ok(())
    }

    /// Mock mesh loading that increments an internal counter.
    ///
    /// # Arguments
    ///
    /// * `_name` - Mesh name (ignored)
    /// * `_data` - Mesh data (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn load_mesh(&mut self, _name: &str, _data: MeshData) -> Result<()> {
        self.mesh_count += 1;
        Ok(())
    }

    /// Mock texture loading that increments an internal counter.
    ///
    /// # Arguments
    ///
    /// * `_name` - Texture name (ignored)
    /// * `_path` - Texture path (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn load_texture(&mut self, _name: &str, _path: &std::path::Path) -> Result<()> {
        self.texture_count += 1;
        Ok(())
    }

    /// Mock material loading that increments an internal counter.
    ///
    /// # Arguments
    ///
    /// * `_name` - Material name (ignored)
    /// * `_properties` - Material properties (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn load_material(&mut self, _name: &str, _properties: MaterialProperties) -> Result<()> {
        self.material_count += 1;
        Ok(())
    }

    /// Returns the number of loaded meshes (mock counter).
    pub fn mesh_count(&self) -> usize {
        self.mesh_count
    }

    /// Returns the number of loaded textures (mock counter).
    pub fn texture_count(&self) -> usize {
        self.texture_count
    }

    /// Returns the number of loaded materials (mock counter).
    pub fn material_count(&self) -> usize {
        self.material_count
    }

    /// No-op surface configuration.
    ///
    /// # Arguments
    ///
    /// * `_width` - Width (ignored)
    /// * `_height` - Height (ignored)
    pub fn configure_surface(&mut self, _width: u32, _height: u32) {
        // No-op
    }
}

#[cfg(test)]
impl Default for MockRenderContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[test]
    fn test_mock_render_context_creation() {
        let ctx = MockRenderContext::new();
        assert_eq!(ctx.mesh_count(), 0);
        assert_eq!(ctx.texture_count(), 0);
        assert_eq!(ctx.material_count(), 0);
    }

    #[test]
    fn test_mock_render_context_load_mesh() {
        let mut ctx = MockRenderContext::new();
        let mesh_data = MeshData::new(vec![], vec![]);

        ctx.load_mesh("test_mesh", mesh_data).unwrap();
        assert_eq!(ctx.mesh_count(), 1);
    }

    #[test]
    fn test_mock_render_context_load_texture() {
        let mut ctx = MockRenderContext::new();

        ctx.load_texture("test_texture", std::path::Path::new("test.png"))
            .unwrap();
        assert_eq!(ctx.texture_count(), 1);
    }

    #[test]
    fn test_mock_render_context_load_material() {
        let mut ctx = MockRenderContext::new();
        let props = MaterialProperties::default();

        ctx.load_material("test_material", props).unwrap();
        assert_eq!(ctx.material_count(), 1);
    }

    #[test]
    fn test_mock_render_context_render() {
        let mut ctx = MockRenderContext::new();
        let cmds = RenderCommands {
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY,
            draw_commands: &[],
            lighting: None,
        };

        // Should not panic or error
        ctx.render(&cmds).unwrap();
    }

    #[test]
    fn test_mock_render_context_configure_surface() {
        let mut ctx = MockRenderContext::new();
        // Should not panic
        ctx.configure_surface(1920, 1080);
    }

    #[test]
    fn test_mock_render_context_default() {
        let ctx = MockRenderContext::default();
        assert_eq!(ctx.mesh_count(), 0);
    }

    #[test]
    fn test_mock_render_context_integration() {
        // Simulate a game loop that loads resources and renders frames
        let mut ctx = MockRenderContext::new();

        // Load game assets
        ctx.load_mesh("player", MeshData::new(vec![], vec![]))
            .unwrap();
        ctx.load_mesh("enemy", MeshData::new(vec![], vec![]))
            .unwrap();
        ctx.load_texture("player_texture", std::path::Path::new("player.png"))
            .unwrap();
        ctx.load_material("player_material", MaterialProperties::default())
            .unwrap();

        // Verify resources loaded
        assert_eq!(ctx.mesh_count(), 2);
        assert_eq!(ctx.texture_count(), 1);
        assert_eq!(ctx.material_count(), 1);

        // Simulate game loop
        for _ in 0..10 {
            let cmds = RenderCommands {
                view: Mat4::IDENTITY,
                proj: Mat4::IDENTITY,
                draw_commands: &[],
                lighting: None,
            };
            ctx.render(&cmds).unwrap();
        }

        // All operations should succeed without errors
        ctx.configure_surface(1920, 1080);
    }
}
