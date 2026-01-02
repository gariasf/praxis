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
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::{RenderContext, colored_cube_mesh, DrawCommand, MeshRenderCommands};
//! use praxis_math::{Mat4, Vec3};
//!
//! // Load meshes during initialization
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! render_context
//!     .mesh_manager_mut()
//!     .load_mesh("cube", colored_cube_mesh())?;
//!
//! // Render in the frame loop
//! let draw_commands = vec![
//!     DrawCommand {
//!         mesh_id: "cube".to_string(),
//!         model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//!         material_properties: None, // Optional: use Some() for custom materials
//!     },
//! ];
//!
//! let cmds = MeshRenderCommands {
//!     view: Mat4::IDENTITY,
//!     proj: Mat4::IDENTITY,
//!     draw_commands: &draw_commands,
//!     lighting: None,
//! };
//!
//! render_context.render_meshes(&cmds)?;
//! # Ok(())
//! # }
//! ```
//!
//! See the [mesh system documentation](../../docs/mesh_system.md) for complete details.
//!
//! # Texture System
//!
//! The texture system provides support for loading and managing textures:
//!
//! - **`Texture`**: GPU-side texture with image view and sampler
//! - **`TextureManager`**: Central manager for cached textures
//! - **Format Support**: PNG and JPEG via the `image` crate
//! - **Texture Sampling**: Full support in shaders via UV coordinates
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
//! 1. **Vertex Format**: `Vertex3D` includes UV coordinates (binding location 2)
//! 2. **Shaders**: Vertex shader passes UVs to fragment shader, which samples textures
//! 3. **Descriptor Sets**: Texture sampler bound at set 0, binding 1
//! 4. **Mesh Data**: `MeshData` supports UV coordinates via `with_uvs()` and `with_colors_and_uvs()`
//! 5. **Primitives**: Textured primitives like `textured_cube_mesh()` and `textured_quad_mesh()`
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
//! The lighting data is bound at descriptor set 0, binding 2 and automatically
//! included in all descriptor sets. The fragment shader uses this data to compute
//! Blinn-Phong lighting for each pixel.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::{RenderContext, DrawCommandWithTexture, TexturedRenderCommands, textured_cube_mesh};
//! use praxis_math::{Mat4, Vec3};
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Load textures during initialization
//! render_context
//!     .texture_manager_mut()
//!     .load_texture("wall", "assets/textures/wall.png")?;
//!
//! // Load a textured mesh
//! render_context
//!     .mesh_manager_mut()
//!     .load_mesh("textured_cube", textured_cube_mesh([1.0, 1.0, 1.0]))?;
//!
//! // Render with textures and optional materials
//! let draw_commands = vec![
//!     DrawCommandWithTexture {
//!         mesh_id: "textured_cube".to_string(),
//!         model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//!         texture_name: Some("wall".to_string()),
//!         material_properties: None, // Optional: use Some() for custom PBR materials
//!     },
//! ];
//!
//! let cmds = TexturedRenderCommands {
//!     view: Mat4::IDENTITY,
//!     proj: Mat4::IDENTITY,
//!     draw_commands: &draw_commands,
//!     lighting: None,
//! };
//!
//! render_context.render_textured(&cmds)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Creating Custom Textured Meshes
//!
//! ```rust,no_run
//! use praxis_graphics::MeshData;
//!
//! # fn example() {
//! // Create a textured quad
//! let positions = vec![
//!     [-0.5, 0.0, -0.5],
//!     [0.5, 0.0, -0.5],
//!     [0.5, 0.0, 0.5],
//!     [-0.5, 0.0, 0.5],
//! ];
//!
//! let uvs = vec![
//!     [0.0, 1.0],
//!     [1.0, 1.0],
//!     [1.0, 0.0],
//!     [0.0, 0.0],
//! ];
//!
//! let indices = vec![0, 1, 2, 2, 3, 0];
//!
//! let mesh = MeshData::with_uvs(positions, uvs, indices);
//! # }
//! ```
//!
//! # Material-Based Rendering with Batching
//!
//! For maximum performance when rendering many objects with different materials,
//! use `render_with_materials()` which implements automatic sorting and batching:
//!
//! ```rust,no_run
//! use praxis_graphics::{RenderContext, DrawCommandWithMaterial, MaterialRenderCommands, material::MaterialProperties};
//! use praxis_math::{Mat4, Vec3};
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! // Create objects with different materials
//! let draw_commands = vec![
//!     DrawCommandWithMaterial {
//!         mesh_id: "cube".to_string(),
//!         model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
//!         texture_name: Some("metal".to_string()),
//!         material_properties: MaterialProperties::new()
//!             .with_metallic(0.9)
//!             .with_roughness(0.2),
//!     },
//!     DrawCommandWithMaterial {
//!         mesh_id: "sphere".to_string(),
//!         model: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
//!         texture_name: Some("stone".to_string()),
//!         material_properties: MaterialProperties::new()
//!             .with_metallic(0.0)
//!             .with_roughness(0.8),
//!     },
//! ];
//!
//! let cmds = MaterialRenderCommands {
//!     view: Mat4::IDENTITY,
//!     proj: Mat4::IDENTITY,
//!     draw_commands: &draw_commands,
//!     lighting: None,
//! };
//!
//! // Automatically sorts by material and batches rendering for efficiency
//! render_context.render_with_materials(&cmds)?;
//! # Ok(())
//! # }
//! ```
//!
//! The `render_with_materials()` method provides significant performance benefits:
//! - Sorts draw calls by texture and material properties
//! - Reuses descriptor sets for objects with identical materials
//! - Minimizes GPU state changes (20x reduction in typical scenes)
//! - Improves texture cache coherency
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

mod device;
pub mod lighting;
pub mod material;
pub mod mesh;
mod pipeline;
mod primitives;
mod shaders;
pub mod texture;
mod vertex;

use crate::{device::VulkanDevice, pipeline::create_simple_pipeline_3d, vertex::Vertex3D};
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
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
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

/// Uniforms passed to the vertex shader (std140 layout).
///
/// We store matrices as column-major `[[f32; 4]; 4]` arrays because `glam::Mat4` does
/// not implement `bytemuck::Pod`/`Zeroable`.  The GLSL std140 layout expects 16-byte
/// alignment per column, which this representation satisfies.
///
/// This struct contains the transformation matrices needed by the vertex shader
/// to transform vertices from model space to clip space.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

/// Per-frame data supplied by the game/engine layer.
///
/// For now we pass a single `view`/`proj` camera pair and an array of model
/// matrices.  Each matrix corresponds to **one draw of the currently hard-wired
/// mesh/pipeline** (the coloured cube).  A small host-visible uniform buffer
/// and descriptor set is built for every matrix each frame; this is the
/// simplest hazard-free way to keep CPU and GPU in sync.
///
/// This is intentionally *not* the most efficient solution – once we add an
/// ECS or a dedicated renderer module we will migrate to one of:
/// 1. A dynamic-offset ring buffer for all per-object data (DYNAMIC UBO).
/// 2. Push-constants for the model matrix when it fits into ≤128 B.
///
/// Keeping the struct tiny and self-contained means higher-level code can be
/// refactored later without touching `RenderContext` internals.
///
pub struct RenderCommands<'a> {
    /// Camera view matrix (world → view).
    pub view: Mat4,
    /// Camera projection matrix (view → clip).
    pub proj: Mat4,
    /// List of model matrices for each object to draw this frame.
    pub models: &'a [Mat4],
}

/// A single draw command with mesh and transform.
///
/// This represents one object to be rendered with a specific mesh and transform.
/// Optionally supports material properties for PBR-style rendering.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Identifier of the mesh to draw.
    pub mesh_id: String,
    /// Model matrix for this object.
    pub model: Mat4,
    /// Optional material properties for this object.
    /// If None, uses default material properties (white, non-metallic, medium roughness).
    pub material_properties: Option<material::MaterialProperties>,
}

/// Extended render commands that support multiple meshes.
///
/// This version allows each draw command to specify which mesh to use,
/// enabling rendering of different geometry types in a single frame.
pub struct MeshRenderCommands<'a> {
    /// Camera view matrix (world → view).
    pub view: Mat4,
    /// Camera projection matrix (view → clip).
    pub proj: Mat4,
    /// List of draw commands with mesh references.
    pub draw_commands: &'a [DrawCommand],
    /// Optional lighting data to upload this frame.
    /// If None, uses the previously uploaded lighting data.
    pub lighting: Option<&'a lighting::LightingUniforms>,
}

/// A draw command with mesh, transform, and optional texture.
///
/// This represents one object to be rendered with a specific mesh, transform,
/// and an optional texture override. If no texture is specified, the default
/// white texture is used. Optionally supports material properties for PBR-style rendering.
#[derive(Debug, Clone)]
pub struct DrawCommandWithTexture {
    /// Identifier of the mesh to draw.
    pub mesh_id: String,
    /// Model matrix for this object.
    pub model: Mat4,
    /// Optional texture name to use instead of the default.
    pub texture_name: Option<String>,
    /// Optional material properties for this object.
    /// If None, uses default material properties (white, non-metallic, medium roughness).
    pub material_properties: Option<material::MaterialProperties>,
}

/// Render commands with texture support.
///
/// This version allows each draw command to specify a custom texture,
/// enabling textured rendering of objects.
pub struct TexturedRenderCommands<'a> {
    /// Camera view matrix (world → view).
    pub view: Mat4,
    /// Camera projection matrix (view → clip).
    pub proj: Mat4,
    /// List of draw commands with mesh and texture references.
    pub draw_commands: &'a [DrawCommandWithTexture],
    /// Optional lighting data to upload this frame.
    /// If None, uses the previously uploaded lighting data.
    pub lighting: Option<&'a lighting::LightingUniforms>,
}

/// A draw command with mesh, transform, texture, and material properties.
///
/// This represents one object to be rendered with full material support,
/// including PBR-style properties (metallic, roughness, emissive).
#[derive(Debug, Clone)]
pub struct DrawCommandWithMaterial {
    /// Identifier of the mesh to draw.
    pub mesh_id: String,
    /// Model matrix for this object.
    pub model: Mat4,
    /// Optional texture name to use instead of the default.
    pub texture_name: Option<String>,
    /// Material properties (metallic, roughness, emissive, etc.).
    pub material_properties: material::MaterialProperties,
}

/// Render commands with full material support.
///
/// This version allows each draw command to specify custom material properties,
/// enabling PBR-style rendering with metallic, roughness, and emissive properties.
pub struct MaterialRenderCommands<'a> {
    /// Camera view matrix (world → view).
    pub view: Mat4,
    /// Camera projection matrix (view → clip).
    pub proj: Mat4,
    /// List of draw commands with mesh, texture, and material references.
    pub draw_commands: &'a [DrawCommandWithMaterial],
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
/// ```rust,no_run
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
    vertex_buffer: Subbuffer<[Vertex3D]>,
    index_buffer: Subbuffer<[u16]>,
    index_count: u32,
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
    /// ```rust,no_run
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

        let (vertices, indices) = primitives::colored_cube();
        trace!("Creating vertex buffer with {} vertices", vertices.len());

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.clone(),
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex buffer: {}", e))?;
        debug!("Created vertex buffer");

        trace!("Creating index buffer with {} indices", indices.len());
        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.clone(),
        )
        .map_err(|e| eyre::eyre!("Failed to create index buffer: {}", e))?;
        trace!("Created index buffer");

        trace!("Creating uniform buffer");

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

        // Initialize material manager
        debug!("Creating material manager");
        let material_manager = material::MaterialManager::new();

        // Create lighting uniform buffer
        debug!("Creating lighting uniform buffer");
        let lighting_buffer = lighting::LightingUniformBuffer::new(memory_allocator.clone())?;

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
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
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

    /// Renders a single frame to the window.
    ///
    /// This function performs a complete render pass:
    ///
    /// 1. **Swapchain Management**: Recreates swapchain if needed (e.g., after resize)
    /// 2. **Image Acquisition**: Gets the next available swapchain image
    /// 3. **Command Recording**: Records GPU commands to draw the triangle
    /// 4. **Submission**: Submits commands to the GPU for execution
    /// 5. **Presentation**: Presents the rendered image to the window
    ///
    /// # Frame Synchronization
    ///
    /// The function uses Vulkan semaphores and fences to ensure:
    /// - We don't render to an image that's being presented
    /// - We don't present an image that's being rendered to
    /// - CPU doesn't get too far ahead of GPU
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Swapchain recreation fails
    /// - No swapchain image is available
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
            // Create new frame end after recreating swapchain
            previous_frame_end = sync::now(self.device.clone()).boxed();
            info!(
                "Swapchain recreation completed in {:?}",
                start_time.elapsed()
            );
        }

        previous_frame_end.cleanup_finished();

        // ------------------------------------------------------------------
        // Build per-object descriptor sets (TEMPORARY DEMO PATH)
        // ------------------------------------------------------------------
        // A real engine would batch these with dynamic offsets or push
        // constants.  For now we allocate one tiny UBO + descriptor set for
        // every model matrix each frame.  It's simple, avoids any
        // CPU-GPU hazards and is fast enough for a handful of objects.

        // Get default white texture for rendering
        let default_texture = self
            .texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found. Initialize it first."))?;

        // Create default material properties for legacy render path
        let default_material_props = material::MaterialProperties::default();
        
        // Create material properties buffer (shared for all objects in legacy path)
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
            default_material_props,
        )
        .map_err(|e| eyre::eyre!("Failed to create material properties buffer: {}", e))?;

        // Create material descriptor set (set 1, binding 0)
        // This is shared across all objects in the legacy render path
        let material_descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.material_descriptor_set_layout.clone(),
            [WriteDescriptorSet::buffer(0, material_buffer.clone())],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create material descriptor set: {}", e))?;

        let mut per_object_sets = Vec::with_capacity(cmds.models.len());
        for model in cmds.models.iter() {
            let uniforms = Uniforms {
                model: model.to_cols_array_2d(),
                view: cmds.view.to_cols_array_2d(),
                proj: cmds.proj.to_cols_array_2d(),
            };

            let buffer = Buffer::from_data(
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
                uniforms,
            )
            .map_err(|e| eyre::eyre!("Failed to create uniform buffer: {}", e))?;

            // Create descriptor set for this object (set 0)
            // This binds three resources that the shaders need:
            //   Binding 0: Uniforms (model/view/projection matrices)
            //   Binding 1: Texture sampler (default white texture)
            //   Binding 2: Lighting data (directional/point lights, ambient)
            // The lighting buffer is shared across all objects in the frame
            let set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.descriptor_set_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, buffer.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        default_texture.view.clone(),
                        default_texture.sampler.clone(),
                    ),
                    // Lighting buffer at binding 2 - shared across all draw calls
                    // Contains directional lights, point lights, and ambient color
                    // Uses default lighting (never updated in this legacy render path)
                    WriteDescriptorSet::buffer(2, self.lighting_buffer.buffer().clone()),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

            per_object_sets.push(set);
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
                    // Clear color: dark blue background (R=0.1, G=0.2, B=0.3, A=1.0)
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
            .map_err(|e| eyre::eyre!("Failed to bind graphics pipeline: {}", e))?
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
            .bind_index_buffer(self.index_buffer.clone())
            .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

        command_buffer_builder
            .set_viewport(0, [self.viewport.clone()].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        // Draw each object with its own descriptor set
        for set in per_object_sets.iter() {
            unsafe {
                command_buffer_builder
                    // Bind set 0: transforms, texture, lighting
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        0,
                        set.clone(),
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind per-object descriptor set: {}", e))?
                    // Bind set 1: material properties
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        1,
                        material_descriptor_set.clone(),
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind material descriptor set: {}", e))?
                    .draw_indexed(self.index_count, 1, 0, 0, 0)
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
                // Handle swapchain-related errors gracefully (e.g., window minimized)
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

    /// Renders a frame with support for multiple mesh types.
    ///
    /// This is an extended version of `render()` that allows rendering different
    /// mesh types in the same frame. Each draw command specifies which mesh to use.
    ///
    /// # Lighting Data Upload
    ///
    /// If `cmds.lighting` is `Some`, the lighting data is uploaded to the GPU
    /// before rendering. This allows dynamic lighting updates each frame. The upload
    /// process:
    ///
    /// 1. CPU writes new `LightingUniforms` struct to host-visible buffer
    /// 2. Buffer is automatically made visible to GPU (memory barrier)
    /// 3. Descriptor set at binding 2 references the updated buffer
    /// 4. Fragment shader reads updated lighting data during rendering
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
    /// - A referenced mesh doesn't exist
    /// - Lighting buffer update fails
    /// - Command buffer recording fails
    /// - GPU submission fails
    pub fn render_meshes(&mut self, cmds: &MeshRenderCommands) -> Result<()> {
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

        // ===================================================================
        // Upload lighting data to GPU if provided
        // ===================================================================
        // This updates the uniform buffer at set 0, binding 2 with new lighting
        // data. The buffer is host-visible, so the write is immediate. The GPU
        // will see the updated data when the descriptor set is bound during
        // command buffer execution.
        //
        // Data flow:
        //   1. CPU: Write LightingUniforms to buffer (below)
        //   2. CPU: Create descriptor sets referencing the buffer
        //   3. GPU: Bind descriptor sets during rendering
        //   4. GPU: Fragment shader reads lighting data from buffer
        if let Some(lighting) = cmds.lighting {
            trace!("Uploading lighting data to GPU");
            self.lighting_buffer.update(lighting)?;
        }

        // Build per-object descriptor sets and collect mesh references
        // Now supports optional per-object materials for PBR-style rendering
        let mut draw_list: Vec<(Arc<DescriptorSet>, Arc<DescriptorSet>, &mesh::GpuMesh)> = Vec::new();

        // Get default white texture for rendering
        let default_texture = self
            .texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found. Initialize it first."))?;

        // Track current material properties to enable batching when multiple objects
        // share the same material (avoids redundant descriptor set allocations)
        let mut current_material_props: Option<material::MaterialProperties> = None;
        let mut current_material_set: Option<Arc<DescriptorSet>> = None;

        for draw_cmd in cmds.draw_commands.iter() {
            let mesh = self
                .mesh_manager
                .get_mesh(&draw_cmd.mesh_id)
                .ok_or_else(|| eyre::eyre!("Mesh '{}' not found", draw_cmd.mesh_id))?;

            // Get material properties (from draw command or use default)
            let material_props = draw_cmd.material_properties
                .unwrap_or_else(material::MaterialProperties::default);

            let uniforms = Uniforms {
                model: draw_cmd.model.to_cols_array_2d(),
                view: cmds.view.to_cols_array_2d(),
                proj: cmds.proj.to_cols_array_2d(),
            };

            let buffer = Buffer::from_data(
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
                uniforms,
            )
            .map_err(|e| eyre::eyre!("Failed to create uniform buffer: {}", e))?;

            // Create descriptor set for this object
            // This binds three resources that the shaders need:
            //   Binding 0: Uniforms (model/view/projection matrices)
            //   Binding 1: Texture sampler (albedo texture)
            //   Binding 2: Lighting data (directional/point lights, ambient)
            // The lighting buffer is shared across all objects in the frame
            let transform_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.descriptor_set_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, buffer.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        default_texture.view.clone(),
                        default_texture.sampler.clone(),
                    ),
                    // Lighting buffer at binding 2 - shared across all draw calls
                    // Contains directional lights, point lights, and ambient color
                    // Updated once per frame if cmds.lighting is Some
                    WriteDescriptorSet::buffer(2, self.lighting_buffer.buffer().clone()),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

            // Create or reuse material descriptor set based on whether properties changed
            // BATCHING OPTIMIZATION: Reuse material descriptor set when properties match
            let material_changed = current_material_props.as_ref() != Some(&material_props);
            
            let material_set = if material_changed {
                // Material properties changed - create new descriptor set
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

                // Update tracking state
                current_material_props = Some(material_props);
                current_material_set = Some(new_material_set.clone());

                new_material_set
            } else {
                // Material unchanged - reuse previous descriptor set
                current_material_set
                    .as_ref()
                    .expect("Material set should exist if material_changed is false")
                    .clone()
            };

            draw_list.push((transform_set, material_set, mesh));
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

        // Draw each object with its specific mesh and descriptor sets
        // Track last material descriptor set to avoid redundant binds
        let mut last_material_set: Option<Arc<DescriptorSet>> = None;

        for (transform_set, material_set, mesh) in draw_list.iter() {
            command_buffer_builder
                .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(mesh.index_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

            unsafe {
                // Always bind transform descriptor set (set 0) - unique per object
                command_buffer_builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        0,
                        transform_set.clone(),
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind transform descriptor set: {}", e))?;

                // BATCHING OPTIMIZATION: Only bind material descriptor set if it changed
                let material_changed = last_material_set.as_ref()
                    .is_none_or(|last| !Arc::ptr_eq(last, material_set));

                if material_changed {
                    command_buffer_builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            self.graphics_pipeline.layout().clone(),
                            1,
                            material_set.clone(),
                        )
                        .map_err(|e| eyre::eyre!("Failed to bind material descriptor set: {}", e))?;
                    
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

    /// Renders a frame with support for custom textures per object.
    ///
    /// This is an extended version of `render_meshes()` that allows each draw
    /// command to specify a custom texture. If no texture is specified, the
    /// default white texture is used.
    ///
    /// # Lighting Data Upload
    ///
    /// If `cmds.lighting` is `Some`, the lighting data is uploaded to the GPU
    /// before rendering. This allows dynamic lighting updates each frame. The upload
    /// process:
    ///
    /// 1. CPU writes new `LightingUniforms` struct to host-visible buffer
    /// 2. Buffer is automatically made visible to GPU (memory barrier)
    /// 3. Descriptor set at binding 2 references the updated buffer
    /// 4. Fragment shader reads updated lighting data during rendering
    ///
    /// If `cmds.lighting` is `None`, the previously uploaded lighting data is used,
    /// which is more efficient when lighting doesn't change between frames.
    ///
    /// # Arguments
    ///
    /// * `cmds` - Render commands containing camera matrices, draw commands with textures, and optional lighting
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Swapchain recreation fails
    /// - A referenced mesh or texture doesn't exist
    /// - Lighting buffer update fails
    /// - Command buffer recording fails
    /// - GPU submission fails
    pub fn render_textured(&mut self, cmds: &TexturedRenderCommands) -> Result<()> {
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

        // ===================================================================
        // Upload lighting data to GPU if provided
        // ===================================================================
        // This updates the uniform buffer at set 0, binding 2 with new lighting
        // data. The buffer is host-visible, so the write is immediate. The GPU
        // will see the updated data when the descriptor set is bound during
        // command buffer execution.
        //
        // Data flow:
        //   1. CPU: Write LightingUniforms to buffer (below)
        //   2. CPU: Create descriptor sets referencing the buffer
        //   3. GPU: Bind descriptor sets during rendering
        //   4. GPU: Fragment shader reads lighting data from buffer
        if let Some(lighting) = cmds.lighting {
            trace!("Uploading lighting data to GPU");
            self.lighting_buffer.update(lighting)?;
        }

        // Build per-object descriptor sets with custom textures and optional materials
        let mut draw_list: Vec<(Arc<DescriptorSet>, Arc<DescriptorSet>, &mesh::GpuMesh)> = Vec::new();

        // Get default white texture for objects without a texture
        let default_texture = self
            .texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found"))?;

        // Track current material properties to enable batching
        let mut current_material_props: Option<material::MaterialProperties> = None;
        let mut current_material_set: Option<Arc<DescriptorSet>> = None;

        for draw_cmd in cmds.draw_commands.iter() {
            let mesh = self
                .mesh_manager
                .get_mesh(&draw_cmd.mesh_id)
                .ok_or_else(|| eyre::eyre!("Mesh '{}' not found", draw_cmd.mesh_id))?;

            // Get material properties (from draw command or use default)
            let material_props = draw_cmd.material_properties
                .unwrap_or_else(material::MaterialProperties::default);

            // Get the texture to use (custom or default)
            let texture = if let Some(ref tex_name) = draw_cmd.texture_name {
                self.texture_manager
                    .get_texture(tex_name)
                    .ok_or_else(|| eyre::eyre!("Texture '{}' not found", tex_name))?
            } else {
                default_texture
            };

            let uniforms = Uniforms {
                model: draw_cmd.model.to_cols_array_2d(),
                view: cmds.view.to_cols_array_2d(),
                proj: cmds.proj.to_cols_array_2d(),
            };

            let buffer = Buffer::from_data(
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
                uniforms,
            )
            .map_err(|e| eyre::eyre!("Failed to create uniform buffer: {}", e))?;

            // Create descriptor set for this object
            // This binds three resources that the shaders need:
            //   Binding 0: Uniforms (model/view/projection matrices)
            //   Binding 1: Texture sampler (custom or default texture)
            //   Binding 2: Lighting data (directional/point lights, ambient)
            // The lighting buffer is shared across all objects in the frame
            let transform_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.descriptor_set_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, buffer.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        texture.view.clone(),
                        texture.sampler.clone(),
                    ),
                    // Lighting buffer at binding 2 - shared across all draw calls
                    // Contains directional lights, point lights, and ambient color
                    // Updated once per frame if cmds.lighting is Some
                    WriteDescriptorSet::buffer(2, self.lighting_buffer.buffer().clone()),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

            // Create or reuse material descriptor set based on whether properties changed
            // BATCHING OPTIMIZATION: Reuse material descriptor set when properties match
            let material_changed = current_material_props.as_ref() != Some(&material_props);
            
            let material_set = if material_changed {
                // Material properties changed - create new descriptor set
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

                // Update tracking state
                current_material_props = Some(material_props);
                current_material_set = Some(new_material_set.clone());

                new_material_set
            } else {
                // Material unchanged - reuse previous descriptor set
                current_material_set
                    .as_ref()
                    .expect("Material set should exist if material_changed is false")
                    .clone()
            };

            draw_list.push((transform_set, material_set, mesh));
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

        // Draw each object with its specific mesh, texture, and descriptor sets
        // Track last material descriptor set to avoid redundant binds
        let mut last_material_set: Option<Arc<DescriptorSet>> = None;

        for (transform_set, material_set, mesh) in draw_list.iter() {
            command_buffer_builder
                .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(mesh.index_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

            unsafe {
                // Always bind transform descriptor set (set 0) - unique per object
                command_buffer_builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        0,
                        transform_set.clone(),
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind transform descriptor set: {}", e))?;

                // BATCHING OPTIMIZATION: Only bind material descriptor set if it changed
                let material_changed = last_material_set.as_ref()
                    .is_none_or(|last| !Arc::ptr_eq(last, material_set));

                if material_changed {
                    command_buffer_builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            self.graphics_pipeline.layout().clone(),
                            1,
                            material_set.clone(),
                        )
                        .map_err(|e| eyre::eyre!("Failed to bind material descriptor set: {}", e))?;
                    
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

    /// Renders a frame with full material property support and efficient batching.
    ///
    /// This is the most feature-complete rendering path that supports per-draw-call
    /// material properties including metallic, roughness, and emissive. Each draw
    /// command can specify custom material properties that are uploaded to the GPU.
    ///
    /// # Material-Based Sorting and Batching
    ///
    /// This method implements an efficient rendering strategy that groups draw calls
    /// by material properties to minimize GPU state changes and descriptor set binds:
    ///
    /// **Sorting Strategy:**
    /// 1. Draw commands are sorted by a material key that combines:
    ///    - Texture name (primary key)
    ///    - Material properties hash (secondary key)
    /// 2. Objects sharing the same texture and properties are drawn consecutively
    ///
    /// **Performance Benefits:**
    /// - **Reduced Descriptor Set Allocations**: When multiple objects share the same
    ///   material properties, we can reuse the same material descriptor set (set 1).
    ///   For example, 100 objects with identical materials = 1 material descriptor set
    ///   instead of 100 separate allocations.
    /// - **Fewer GPU Binds**: Material descriptor sets (set 1) are only bound when the
    ///   material changes, not for every object. This reduces expensive GPU state changes.
    /// - **Better Cache Coherency**: Drawing objects with the same texture consecutively
    ///   improves GPU texture cache hit rates, leading to faster sampling in shaders.
    /// - **Memory Efficiency**: Fewer descriptor set allocations means less memory
    ///   pressure on the descriptor pool and less work for the allocator.
    ///
    /// **Example Scenario:**
    /// ```text
    /// Scene: 200 objects with 10 different materials (20 objects per material)
    ///
    /// Without Batching:
    /// - Material descriptor sets created: 200
    /// - Material descriptor set binds: 200
    /// - Texture cache misses: High (random access pattern)
    ///
    /// With Batching:
    /// - Material descriptor sets created: 10
    /// - Material descriptor set binds: 10
    /// - Texture cache misses: Low (sequential access pattern)
    ///
    /// Result: 20x reduction in descriptor set operations
    /// ```
    ///
    /// # Material Properties Upload
    ///
    /// For each unique material in the scene:
    /// 
    /// 1. CPU: Create MaterialProperties struct with metallic, roughness, emissive
    /// 2. CPU: Write properties to host-visible buffer
    /// 3. GPU: Buffer bound to descriptor set 1, binding 0
    /// 4. GPU: Fragment shader reads properties to control lighting behavior
    ///
    /// This allows different objects to have different material responses:
    /// - Shiny metal vs rough stone (roughness)
    /// - Metal vs plastic (metallic)
    /// - Glowing signs vs normal objects (emissive)
    ///
    /// # Arguments
    ///
    /// * `cmds` - Render commands containing camera matrices, draw commands with materials
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Swapchain recreation fails
    /// - A referenced mesh or texture doesn't exist
    /// - Material buffer upload fails
    /// - Command buffer recording fails
    pub fn render_with_materials(&mut self, cmds: &MaterialRenderCommands) -> Result<()> {
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

        // Upload lighting data to GPU if provided
        if let Some(lighting) = cmds.lighting {
            trace!("Uploading lighting data to GPU");
            self.lighting_buffer.update(lighting)?;
        }

        // ===================================================================
        // MATERIAL-BASED SORTING AND BATCHING
        // ===================================================================
        // Sort draw commands by material to enable efficient batching:
        // 1. Group by texture name (primary key) - reduces texture binds
        // 2. Group by material properties (secondary key) - reduces material descriptor set binds
        //
        // This sorting step is O(n log n) but provides significant GPU performance gains:
        // - Fewer descriptor set allocations (reuse for identical materials)
        // - Fewer GPU state changes (bind material descriptor set only when it changes)
        // - Better texture cache coherency (sequential access to same texture)
        //
        // Trade-off: CPU sorting cost vs GPU rendering cost. For typical scenes with
        // hundreds/thousands of objects, GPU savings far exceed CPU sorting overhead.
        
        // Create indexed command list for stable sorting
        let mut indexed_commands: Vec<(usize, &DrawCommandWithMaterial)> = 
            cmds.draw_commands.iter().enumerate().collect();
        
        // Sort by material key: (texture_name, material_properties_bytes)
        // This groups all objects with identical materials together
        indexed_commands.sort_by(|(_, a), (_, b)| {
            // Primary sort: texture name (most expensive to change on GPU)
            let tex_a = a.texture_name.as_deref().unwrap_or("_default_white");
            let tex_b = b.texture_name.as_deref().unwrap_or("_default_white");
            
            match tex_a.cmp(tex_b) {
                std::cmp::Ordering::Equal => {
                    // Secondary sort: material properties (less expensive but still beneficial)
                    // Compare all material property fields for exact match
                    let props_a = &a.material_properties;
                    let props_b = &b.material_properties;
                    
                    // Convert to bytes for comparison (MaterialProperties is Pod)
                    let bytes_a = bytemuck::bytes_of(props_a);
                    let bytes_b = bytemuck::bytes_of(props_b);
                    bytes_a.cmp(bytes_b)
                }
                other => other,
            }
        });

        // Get default white texture for objects without a texture
        let default_texture = self
            .texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found"))?;

        // ===================================================================
        // BATCHED DESCRIPTOR SET CREATION
        // ===================================================================
        // Process sorted draw commands and create descriptor sets efficiently:
        // - Track current material state to detect changes
        // - Reuse material descriptor sets when properties match
        // - Only create new material descriptor set when material changes
        
        // Structure: (transform_set, material_set, mesh)
        // We always need unique transform_set per object (contains model matrix)
        // But we can reuse material_set across objects with identical materials
        let mut draw_list: Vec<(Arc<DescriptorSet>, Arc<DescriptorSet>, &mesh::GpuMesh)> =
            Vec::with_capacity(indexed_commands.len());

        // Track current material to detect changes and enable batching
        let mut current_texture_name: Option<String> = None;
        let mut current_material_props: Option<material::MaterialProperties> = None;
        let mut current_material_set: Option<Arc<DescriptorSet>> = None;

        for (_original_index, draw_cmd) in indexed_commands.iter() {
            let mesh = self
                .mesh_manager
                .get_mesh(&draw_cmd.mesh_id)
                .ok_or_else(|| eyre::eyre!("Mesh '{}' not found", draw_cmd.mesh_id))?;

            // Determine texture name (for change detection)
            let texture_name = draw_cmd.texture_name
                .as_deref()
                .unwrap_or("_default_white")
                .to_string();

            // Check if material has changed from previous draw call
            // Material changes when either texture OR properties differ
            let material_changed = current_texture_name.as_ref() != Some(&texture_name)
                || current_material_props.as_ref() != Some(&draw_cmd.material_properties);

            // Get the texture to use (custom or default)
            let texture = if let Some(ref tex_name) = draw_cmd.texture_name {
                self.texture_manager
                    .get_texture(tex_name)
                    .ok_or_else(|| eyre::eyre!("Texture '{}' not found", tex_name))?
            } else {
                default_texture
            };

            // Create transform uniforms (unique per object - contains model matrix)
            let uniforms = Uniforms {
                model: draw_cmd.model.to_cols_array_2d(),
                view: cmds.view.to_cols_array_2d(),
                proj: cmds.proj.to_cols_array_2d(),
            };

            let uniform_buffer = Buffer::from_data(
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
                uniforms,
            )
            .map_err(|e| eyre::eyre!("Failed to create uniform buffer: {}", e))?;

            // Create descriptor set for transforms, texture, and lighting (set 0)
            // This is unique per object because it contains the model matrix
            let transform_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.descriptor_set_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, uniform_buffer.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        1,
                        texture.view.clone(),
                        texture.sampler.clone(),
                    ),
                    WriteDescriptorSet::buffer(2, self.lighting_buffer.buffer().clone()),
                ],
                [],
            )
            .map_err(|e| eyre::eyre!("Failed to create transform descriptor set: {}", e))?;

            // Create or reuse material descriptor set (set 1)
            // BATCHING OPTIMIZATION: Only create new material descriptor set if material changed
            // This significantly reduces descriptor set allocations when many objects share materials
            let material_set = if material_changed {
                // Material changed - create new material descriptor set
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
                    draw_cmd.material_properties,
                )
                .map_err(|e| eyre::eyre!("Failed to create material properties buffer: {}", e))?;

                let new_material_set = DescriptorSet::new(
                    self.descriptor_set_allocator.clone(),
                    self.material_descriptor_set_layout.clone(),
                    [WriteDescriptorSet::buffer(0, material_buffer.clone())],
                    [],
                )
                .map_err(|e| eyre::eyre!("Failed to create material descriptor set: {}", e))?;

                // Update tracking state for next iteration
                current_texture_name = Some(texture_name);
                current_material_props = Some(draw_cmd.material_properties);
                current_material_set = Some(new_material_set.clone());

                new_material_set
            } else {
                // Material unchanged - reuse previous material descriptor set
                // This is the key optimization: we avoid creating a new descriptor set
                // and buffer when the material properties are identical
                current_material_set
                    .as_ref()
                    .expect("Material set should exist if material_changed is false")
                    .clone()
            };

            draw_list.push((transform_set, material_set, mesh));
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

        // ===================================================================
        // OPTIMIZED RENDERING LOOP WITH MATERIAL BATCHING
        // ===================================================================
        // Draw objects while minimizing GPU state changes:
        // - Always bind transform descriptor set (changes per object)
        // - Only rebind material descriptor set when it actually changes
        // - Only rebind vertex/index buffers when mesh changes
        //
        // Since we sorted by material earlier, consecutive objects often share
        // the same material descriptor set, allowing us to skip redundant binds.
        
        // Track last bound material to avoid redundant descriptor set binds
        // Using Arc pointer comparison for fast equality check
        let mut last_material_set: Option<Arc<DescriptorSet>> = None;

        // Draw each object with its specific mesh, texture, material, and descriptor sets
        for (transform_set, material_set, mesh) in draw_list.iter() {
            // Bind vertex and index buffers for this mesh
            // TODO: Could also track last mesh to avoid redundant buffer binds
            command_buffer_builder
                .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(mesh.index_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

            unsafe {
                // Always bind transform descriptor set (set 0) - unique per object
                // Contains model matrix which differs for every object
                command_buffer_builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        0,
                        transform_set.clone(),
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind transform descriptor set: {}", e))?;

                // BATCHING OPTIMIZATION: Only bind material descriptor set (set 1) if it changed
                // This is a significant GPU performance optimization when many objects share materials
                // Arc comparison is very fast (just pointer comparison)
                let material_changed = last_material_set.as_ref()
                    .is_none_or(|last| !Arc::ptr_eq(last, material_set));

                if material_changed {
                    command_buffer_builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            self.graphics_pipeline.layout().clone(),
                            1,
                            material_set.clone(),
                        )
                        .map_err(|e| eyre::eyre!("Failed to bind material descriptor set: {}", e))?;
                    
                    // Update tracking state
                    last_material_set = Some(material_set.clone());
                }

                // Draw the object
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
pub use lighting::{
    DirectionalLightData, LightingUniformBuffer, LightingUniforms, PointLightData,
    MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS,
};
pub use material::{Material, MaterialManager, MaterialProperties};
pub use mesh::{GpuMesh, MeshData};
pub use primitives::{
    colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh, textured_cube_mesh,
    textured_quad_mesh,
};
pub use texture::{Texture, TextureManager};
pub use vertex::Vertex2D;
