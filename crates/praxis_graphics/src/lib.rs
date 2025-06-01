//! Graphics system for the Praxis engine.
//!
//! This crate provides functionality for rendering and managing graphics using Vulkan via vulkano.

use praxis_utils::{Result, eyre, info};

use std::sync::Arc;
use vulkano::{
    VulkanLibrary,
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
        SubpassEndInfo, allocator::StandardCommandBufferAllocator,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateInfo},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo},
    sync::{self, GpuFuture},
};
use winit::window::Window;

/// Core graphics context containing the Vulkan state.
///
/// This struct holds the main graphics backend components including the instance, device,
/// queues, surface, swapchain, and other Vulkan objects needed for rendering.
pub struct RenderContext {
    pub instance: Arc<Instance>,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,

    present_queue: Arc<Queue>,
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    swapchain_images: Vec<Arc<Image>>,
    swapchain_image_views: Vec<Arc<ImageView>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl RenderContext {
    /// Creates a new `RenderContext` for a given window.
    ///
    /// Initializes the Vulkan instance, selects a suitable physical device,
    /// creates a logical device with required queues, sets up the surface and swapchain,
    /// and prepares the render pass and framebuffers.
    ///
    /// # Arguments
    ///
    /// * `window` - An `Arc<Window>` representing the window to render onto.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Self>` containing the initialized context or an error.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        info!("Initializing Vulkan graphics context...");

        // Create Vulkan instance
        let library = VulkanLibrary::new()
            .map_err(|e| eyre::eyre!("Failed to load Vulkan library: {}", e))?;

        let required_extensions = Surface::required_extensions(&window);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create Vulkan instance: {}", e))?;

        info!("Created Vulkan instance");

        // Create surface
        let surface = Surface::from_window(instance.clone(), window.clone())
            .map_err(|e| eyre::eyre!("Failed to create window surface: {}", e))?;

        info!("Created window surface");

        // Select physical device
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_graphics, queue_family_present) =
            Self::select_physical_device(&instance, &surface, &device_extensions)?;

        info!(
            "Selected physical device: {} ({})",
            physical_device.properties().device_name,
            match physical_device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => "Discrete GPU",
                PhysicalDeviceType::IntegratedGpu => "Integrated GPU",
                PhysicalDeviceType::VirtualGpu => "Virtual GPU",
                PhysicalDeviceType::Cpu => "CPU",
                _ => "Other",
            }
        );

        // Create logical device and queues
        let mut queue_create_infos = vec![QueueCreateInfo {
            queue_family_index: queue_family_graphics,
            ..Default::default()
        }];

        if queue_family_present != queue_family_graphics {
            queue_create_infos.push(QueueCreateInfo {
                queue_family_index: queue_family_present,
                ..Default::default()
            });
        }

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos,
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create logical device: {}", e))?;

        let graphics_queue = queues.next().unwrap();
        let present_queue = if queue_family_present != queue_family_graphics {
            queues.next().unwrap()
        } else {
            graphics_queue.clone()
        };

        info!("Created logical device and queues");

        // Create swapchain
        let (swapchain, swapchain_images) =
            Self::create_swapchain(&device, &physical_device, &surface, &window)?;

        info!("Created swapchain with {} images", swapchain_images.len());

        // Create image views
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| ImageView::new_default(image.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| eyre::eyre!("Failed to create image views: {}", e))?;

        // Create render pass
        let render_pass = Self::create_render_pass(&device, swapchain.image_format())?;

        // Create framebuffers
        let framebuffers = Self::create_framebuffers(&swapchain_image_views, &render_pass)?;

        // Create command buffer allocator
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Initialize synchronization
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        info!("Graphics context initialization complete");

        Ok(Self {
            // Public
            instance,
            physical_device,
            device,
            graphics_queue,

            // Private
            present_queue,
            surface,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            render_pass,
            framebuffers,
            command_buffer_allocator,
            recreate_swapchain: false,
            previous_frame_end,
        })
    }

    /// Configures the surface for the given dimensions.
    ///
    /// For vulkano, this marks that the swapchain needs recreation.
    pub fn configure_surface(&mut self, _width: u32, _height: u32) {
        self.recreate_swapchain = true;
    }

    /// Renders a single frame to the configured surface.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if rendering fails.
    pub fn render(&mut self) -> Result<()> {
        if self.recreate_swapchain {
            // Flush any pending operations before recreating
            if let Some(previous_frame_end) = self.previous_frame_end.take() {
                let _ = previous_frame_end.flush();
            }
            self.recreate_swapchain_and_framebuffers()?;
            self.recreate_swapchain = false;
            self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
        }

        let mut previous_frame_end = match self.previous_frame_end.take() {
            Some(future) => future,
            None => sync::now(self.device.clone()).boxed(),
        };

        previous_frame_end.cleanup_finished();

        // Acquire next image
        let (image_index, suboptimal, acquire_future) =
            vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(|e| eyre::eyre!("Failed to acquire next image: {}", e))?;

        if suboptimal {
            self.recreate_swapchain = true;
        }

        // Create command buffer
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| eyre::eyre!("Failed to create command buffer: {}", e))?;

        // Begin render pass
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

        // End render pass
        command_buffer_builder
            .end_render_pass(SubpassEndInfo::default())
            .map_err(|e| eyre::eyre!("Failed to end render pass: {}", e))?;

        // Build command buffer
        let command_buffer = command_buffer_builder
            .build()
            .map_err(|e| eyre::eyre!("Failed to build command buffer: {}", e))?;

        // Execute command buffer
        let execution = previous_frame_end
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|e| eyre::eyre!("Failed to execute command buffer: {}", e))?;

        // Present
        let future = execution
            .then_swapchain_present(
                self.present_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        let future = future.map_err(|e| eyre::eyre!("Failed to present frame: {}", e))?;
        {
            self.previous_frame_end = Some(future.boxed());
        }
        Ok(())
    }

    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> Result<(Arc<PhysicalDevice>, u32, u32)> {
        let suitable_device = instance
            .enumerate_physical_devices()
            .map_err(|e| eyre::eyre!("Failed to enumerate physical devices: {}", e))?
            .filter(|device| device.supported_extensions().contains(device_extensions))
            .filter(|device| device.properties().device_type == PhysicalDeviceType::DiscreteGpu)
            .filter_map(|device| {
                // Find graphics queue family
                let graphics_queue_family = device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .find(|(_, properties)| properties.queue_flags.intersects(QueueFlags::GRAPHICS))
                    .map(|(index, _)| index as u32);

                // Find present queue family
                let present_queue_family = device
                    .queue_family_properties()
                    .iter()
                    .enumerate()
                    .find(|(index, _)| {
                        device
                            .surface_support(*index as u32, surface)
                            .unwrap_or(false)
                    })
                    .map(|(index, _)| index as u32);

                match (graphics_queue_family, present_queue_family) {
                    (Some(graphics), Some(present)) => Some((device, graphics, present)),
                    _ => None,
                }
            })
            .next()
            .ok_or_else(|| eyre::eyre!("No suitable discrete GPU found"))?;

        Ok(suitable_device)
    }

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

    fn recreate_swapchain_and_framebuffers(&mut self) -> Result<()> {
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

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images;
        self.swapchain_image_views = new_image_views;
        self.framebuffers = new_framebuffers;

        info!("Recreated swapchain and framebuffers");
        Ok(())
    }

    /// Returns a reference to the command buffer allocator.
    pub fn command_buffer_allocator(&self) -> &Arc<StandardCommandBufferAllocator> {
        &self.command_buffer_allocator
    }

    /// Returns a reference to the main render pass.
    pub fn render_pass(&self) -> &Arc<RenderPass> {
        &self.render_pass
    }

    /// Returns a reference to the present queue.
    pub fn present_queue(&self) -> &Arc<Queue> {
        &self.present_queue
    }

    /// Returns the current swapchain image format.
    pub fn swapchain_format(&self) -> vulkano::format::Format {
        self.swapchain.image_format()
    }

    /// Returns the current swapchain extent (dimensions).
    pub fn swapchain_extent(&self) -> [u32; 2] {
        self.swapchain.image_extent()
    }
}
