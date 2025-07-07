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
mod pipeline;
mod primitives;
mod shaders;
mod vertex;

use crate::{device::VulkanDevice, pipeline::create_simple_pipeline, vertex::Vertex2D};
use praxis_utils::{Result, debug, error, eyre, info, timing::FrameTimer, trace, warn};

use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassEndInfo, allocator::StandardCommandBufferAllocator,
    },
    device::{Device, Queue, physical::PhysicalDevice},
    image::{Image, ImageUsage, view::ImageView},
    instance::Instance,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{GraphicsPipeline, graphics::viewport::Viewport},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};
use winit::window::Window;

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
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Subbuffer<[Vertex2D]>,
    viewport: Viewport,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    // Performance tracking
    frame_timer: FrameTimer,
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
            create_simple_pipeline(&device, &render_pass, swapchain.image_extent())?;

        // Create vertex buffer with a colored triangle
        // This uses the primitive helper for a standard test triangle
        let vertices = primitives::colored_triangle();
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
            vertices,
        )
        .map_err(|e| eyre::eyre!("Failed to create vertex buffer: {}", e))?;
        debug!("Created vertex buffer");

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
            viewport,
            recreate_swapchain: false,
            previous_frame_end,

            // Performance tracking
            frame_timer: FrameTimer::new(),
        })
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
    pub fn render(&mut self) -> Result<()> {
        self.frame_timer.tick();

        let mut previous_frame_end = self
            .previous_frame_end
            .take()
            .unwrap_or_else(|| sync::now(self.device.clone()).boxed());

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
            &self.command_buffer_allocator,
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
            .map_err(|e| eyre::eyre!("Failed to bind vertex buffer: {}", e))?;

        command_buffer_builder
            .set_viewport(0, [self.viewport.clone()].into_iter().collect())
            .map_err(|e| eyre::eyre!("Failed to set viewport: {}", e))?;

        // Draw the triangle (3 vertices, 1 instance)
        command_buffer_builder
            .draw(
                3, // vertex_count - we have 3 vertices in our triangle
                1, // instance_count - draw one instance
                0, // first_vertex - start at vertex 0
                0, // first_instance - start at instance 0
            )
            .map_err(|e| eyre::eyre!("Failed to draw: {}", e))?;

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
            window_size.width, window_size.height, image_count, image_format
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
