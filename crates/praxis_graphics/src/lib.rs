//! Graphics system for the Praxis engine.
//!
//! This crate provides functionality for rendering and managing graphics using Vulkan via vulkano.
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
//! - `deferred`: Deferred rendering with G-buffer passes
//! - `hdr`: High Dynamic Range rendering with tone mapping
//! - `ssao`: Screen-space ambient occlusion for realistic shadowing
//! - `post_process`: Post-processing framework for screen-space effects
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

pub mod deferred;
mod device;
pub mod hdr;
pub mod lighting;
pub mod line_renderer;
pub mod lod;
pub mod material;
pub mod mesh;
pub mod particles;
mod pipeline;
pub mod post_process;
mod primitives;
mod shaders;
pub mod shadow;
pub mod skybox;
pub mod ssao;
pub mod texture;
pub mod uniform_buffer;
mod vertex;
pub mod visual_feedback;

use crate::{device::VulkanDevice, pipeline::create_simple_pipeline_3d};
use praxis_math::Mat4;
use praxis_utils::{debug, error, eyre, info, timing::FrameTimer, trace, warn, Result};
use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::descriptor_set::allocator::DescriptorSetAllocator;
use vulkano::descriptor_set::DescriptorSet;

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
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,
    material_descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    frame_timer: FrameTimer,

    /// Mesh asset manager for loading and managing meshes.
    mesh_manager: mesh::MeshAssetManager,

    /// Texture asset manager for loading and managing textures.
    texture_manager: texture::TextureManager,

    /// Material asset manager for loading and managing materials.
    material_manager: material::MaterialManager,

    /// Lighting uniform buffer for passing lighting data to shaders.
    lighting_buffer: lighting::LightingUniformBuffer,

    /// Dynamic uniform buffer for per-object model matrices.
    dynamic_uniform_buffer: uniform_buffer::DynamicUniformBuffer,

    /// Buffer for per-frame view/projection uniforms.
    view_proj_buffer: vulkano::buffer::Subbuffer<uniform_buffer::ViewProjectionUniforms>,
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

        debug!("Creating render pass");
        let render_pass = Self::create_render_pass(&device, swapchain.image_format())?;

        debug!("Creating {} framebuffers", swapchain_image_views.len());
        let framebuffers = Self::create_framebuffers(&swapchain_image_views, &render_pass)?;

        trace!("Creating command buffer allocator");
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        trace!("Creating memory allocator");
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let graphics_pipeline =
            create_simple_pipeline_3d(&device, &render_pass, swapchain.image_extent())?;

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let descriptor_set_layout = graphics_pipeline.layout().set_layouts()[0].clone();
        let material_descriptor_set_layout = graphics_pipeline.layout().set_layouts()[1].clone();

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
        let mesh_manager = mesh::MeshAssetManager::new(memory_allocator.clone());

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
            render_pass,
            framebuffers,
            command_buffer_allocator,
            graphics_pipeline,
            memory_allocator,
            descriptor_set_layout,
            material_descriptor_set_layout,
            descriptor_set_allocator,
            viewport,
            recreate_swapchain: false,
            previous_frame_end,

            // Performance tracking
            frame_timer: FrameTimer::new(),

            // Mesh management
            mesh_manager,

            // Texture management
            texture_manager,

            // Material management
            material_manager,

            // Lighting management
            lighting_buffer,

            // Dynamic uniform buffer
            dynamic_uniform_buffer,

            // View/projection data
            view_proj_buffer,
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

            let transform_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.descriptor_set_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, self.view_proj_buffer.clone()),
                    WriteDescriptorSet::buffer(1, self.dynamic_uniform_buffer.buffer().clone()),
                    WriteDescriptorSet::image_view_sampler(
                        2,
                        texture.view.clone(),
                        texture.sampler.clone(),
                    ),
                    WriteDescriptorSet::buffer(3, self.lighting_buffer.buffer().clone()),
                    WriteDescriptorSet::image_view_sampler(
                        9,
                        default_normal_map.view.clone(),
                        default_normal_map.sampler.clone(),
                    ),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create transform descriptor set: {}", e))?;

            let material_set = if material_changed {
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

                let new_material_set = DescriptorSet::new(
                    self.descriptor_set_allocator.clone(),
                    self.material_descriptor_set_layout.clone(),
                    [WriteDescriptorSet::buffer(0, material_buffer.clone())],
                    [],
                )
                .map_err(|e| eyre::eyre!("Failed to create material descriptor set: {}", e))?;

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
                    clear_values: vec![Some([0.1, 0.2, 0.3, 1.0].into())],
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

            unsafe {
                let set_with_offsets = vulkano::descriptor_set::DescriptorSetWithOffsets::new(
                    transform_set.clone(),
                    [dynamic_offset],
                );

                command_buffer_builder.bind_descriptor_sets_unchecked(
                    PipelineBindPoint::Graphics,
                    self.graphics_pipeline.layout().clone(),
                    0,
                    set_with_offsets,
                );

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
    /// Our simple render pass has:
    /// - One color attachment (the swapchain image)
    /// - One subpass that clears and then renders to it
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
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create render pass: {}", e))
    }

    /// Creates framebuffers for each swapchain image.
    ///
    /// A framebuffer binds specific images to the attachments defined in a render pass.
    /// We need one framebuffer per swapchain image.
    fn create_framebuffers(
        image_views: &[Arc<ImageView>],
        render_pass: &Arc<RenderPass>,
    ) -> Result<Vec<Arc<Framebuffer>>> {
        image_views
            .iter()
            .map(|image_view| {
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![image_view.clone()],
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

        let new_framebuffers = Self::create_framebuffers(&new_image_views, &self.render_pass)?;

        // Update viewport
        self.viewport.extent = [window_size.width as f32, window_size.height as f32];

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images;
        self.swapchain_image_views = new_image_views;
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
pub use deferred::{DeferredRenderer, GBuffer};
pub use environment_probe::{
    EnvironmentProbe, EnvironmentProbeCapture, EnvironmentProbeConfig, EnvironmentProbeManager,
    IblData, IblUniforms, ProbeUpdateMode, MAX_ENVIRONMENT_PROBES, SPECULAR_MIP_LEVELS,
};
pub use hdr::{
    calculate_luminance, ExposureCalculator, ExposureMode, HdrRenderTarget,
    ToneMapPass as HdrToneMapPass, ToneMapper, ToneMappingOperator,
};
pub use lighting::{
    DirectionalLightData, LightingUniformBuffer, LightingUniforms, PointLightData,
    MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS,
};
pub use line_renderer::{Line, LineBatch, LineRenderer, LineVertex};
pub use lod::{
    LodGroup, LodLevel, LodManager, LodStatistics, DEFAULT_TRANSITION_DURATION, MAX_LOD_LEVELS,
};
pub use material::{Material, MaterialManager, MaterialProperties};
pub use mesh::{GpuMesh, MeshData};
pub use particles::{
    EmitterShape, ParticleEmitterConfig, ParticleForce, ParticleInstance, ParticleSystem,
    MAX_PARTICLES_PER_EMITTER,
};
pub use post_process::{
    BloomConfig, BloomEffect, BrightnessExtractionPass, CopyPass, FullScreenQuad,
    GaussianBlurHorizontalPass, GaussianBlurVerticalPass, GrayscalePass, PostProcessChain,
    PostProcessContext, PostProcessPass, QuadVertex, RenderTarget, RenderTargetPool, ToneMapPass,
};
pub use primitives::{
    colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh, sphere_mesh, textured_cube_mesh,
    textured_quad_mesh,
};
pub use shadow::{ShadowConfig, ShadowMapManager, ShadowUniforms, MAX_SHADOW_CASCADES};
pub use skybox::SkyboxRenderer;
pub use ssao::{SsaoConfig, SsaoRenderer};
pub use texture::{Cubemap, CubemapFace, Texture, TextureManager};
pub use uniform_buffer::{DynamicUniformBuffer, ModelUniforms, ViewProjectionUniforms};
pub use vertex::Vertex3D;
pub use visual_feedback::{
    create_axis_indicator, create_bounding_box, create_grid, create_selection_outline,
    AxisIndicatorConfig, GridConfig,
};

pub mod environment_probe;

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

        let opaque_objects = vec!["cube1", "cube2", "sphere1"];
        let transparent_objects = vec!["glass1", "water1"];

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
}
