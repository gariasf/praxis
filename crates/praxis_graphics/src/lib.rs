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
pub mod uniform_buffer;
mod vertex;

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
    #[allow(dead_code)]
    dynamic_uniform_buffer: uniform_buffer::DynamicUniformBuffer,

    /// Descriptor set layout for per-frame view/projection data.
    #[allow(dead_code)]
    view_proj_descriptor_set_layout: Arc<vulkano::descriptor_set::layout::DescriptorSetLayout>,

    /// Buffer for per-frame view/projection uniforms.
    #[allow(dead_code)]
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

        // Create dynamic uniform buffer with 3 frames in flight and 1024 max objects
        debug!("Creating dynamic uniform buffer");
        let dynamic_uniform_buffer = uniform_buffer::DynamicUniformBuffer::new(
            &device,
            memory_allocator.clone(),
            3,
            1024,
        )?;

        // Create view/projection descriptor set layout (same as descriptor_set_layout for now)
        let view_proj_descriptor_set_layout = descriptor_set_layout.clone();

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
            view_proj_descriptor_set_layout,
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

        if let Some(lighting) = cmds.lighting {
            trace!("Uploading lighting data to GPU");
            self.lighting_buffer.update(lighting)?;
        }

        let mut indexed_commands: Vec<(usize, &DrawCommand)> = 
            cmds.draw_commands.iter().enumerate().collect();
        
        indexed_commands.sort_by(|(_, a), (_, b)| {
            let tex_a = a.texture_name.as_deref().unwrap_or("_default_white");
            let tex_b = b.texture_name.as_deref().unwrap_or("_default_white");
            
            match tex_a.cmp(tex_b) {
                std::cmp::Ordering::Equal => {
                    let props_a = a.material_properties.unwrap_or_else(material::MaterialProperties::default);
                    let props_b = b.material_properties.unwrap_or_else(material::MaterialProperties::default);
                    
                    let bytes_a = bytemuck::bytes_of(&props_a);
                    let bytes_b = bytemuck::bytes_of(&props_b);
                    bytes_a.cmp(bytes_b)
                }
                other => other,
            }
        });

        let default_texture = self
            .texture_manager
            .get_texture("_default_white")
            .ok_or_else(|| eyre::eyre!("Default white texture not found"))?;

        let mut draw_list: Vec<(Arc<DescriptorSet>, Arc<DescriptorSet>, &mesh::GpuMesh)> =
            Vec::with_capacity(indexed_commands.len());

        let mut current_texture_name: Option<String> = None;
        let mut current_material_props: Option<material::MaterialProperties> = None;
        let mut current_material_set: Option<Arc<DescriptorSet>> = None;

        for (_original_index, draw_cmd) in indexed_commands.iter() {
            let mesh = self
                .mesh_manager
                .get_mesh(&draw_cmd.mesh_id)
                .ok_or_else(|| eyre::eyre!("Mesh '{}' not found", draw_cmd.mesh_id))?;

            let texture_name = draw_cmd.texture_name
                .as_deref()
                .unwrap_or("_default_white")
                .to_string();

            let material_props = draw_cmd.material_properties
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

            // Extract camera position from view matrix inverse
            let view_inverse = cmds.view.inverse();
            let camera_position = [
                view_inverse.col(3).x,
                view_inverse.col(3).y,
                view_inverse.col(3).z,
            ];

            // Create per-frame view-projection uniform buffer
            let view_proj_uniforms = uniform_buffer::ViewProjectionUniforms {
                view: cmds.view.to_cols_array_2d(),
                proj: cmds.proj.to_cols_array_2d(),
                camera_position,
                _padding: 0.0,
            };

            let view_proj_buffer = Buffer::from_data(
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
                view_proj_uniforms,
            )
            .map_err(|e| eyre::eyre!("Failed to create view-projection uniform buffer: {}", e))?;

            // Create per-object model uniform buffer
            let model_uniforms = uniform_buffer::ModelUniforms {
                model: draw_cmd.model.to_cols_array_2d(),
            };

            let model_buffer = Buffer::from_data(
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
                model_uniforms,
            )
            .map_err(|e| eyre::eyre!("Failed to create model uniform buffer: {}", e))?;

            let transform_set = DescriptorSet::new(
                self.descriptor_set_allocator.clone(),
                self.descriptor_set_layout.clone(),
                [
                    WriteDescriptorSet::buffer(0, view_proj_buffer.clone()),
                    WriteDescriptorSet::buffer(1, model_buffer.clone()),
                    WriteDescriptorSet::image_view_sampler(
                        2,
                        texture.view.clone(),
                        texture.sampler.clone(),
                    ),
                    WriteDescriptorSet::buffer(3, self.lighting_buffer.buffer().clone()),
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

        let mut last_material_set: Option<Arc<DescriptorSet>> = None;

        for (transform_set, material_set, mesh) in draw_list.iter() {
            command_buffer_builder
                .bind_vertex_buffers(0, mesh.vertex_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?
                .bind_index_buffer(mesh.index_buffer.clone())
                .map_err(|e| eyre::eyre!("Failed to bind index buffer: {}", e))?;

            unsafe {
                command_buffer_builder
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.graphics_pipeline.layout().clone(),
                        0,
                        transform_set.clone(),
                    )
                    .map_err(|e| eyre::eyre!("Failed to bind transform descriptor set: {}", e))?;

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
    colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh, sphere_mesh, textured_cube_mesh,
    textured_quad_mesh,
};
pub use texture::{Texture, TextureManager};
pub use uniform_buffer::{DynamicUniformBuffer, ModelUniforms, ViewProjectionUniforms};
pub use vertex::Vertex2D;
