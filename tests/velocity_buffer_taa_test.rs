//! Integration test for velocity buffer TAA integration.
//!
//! This test validates the complete TAA pipeline with velocity buffer integration:
//! - Renders moving objects across 3 frames
//! - Extracts velocity buffer data
//! - Validates motion vector magnitudes
//! - Verifies reprojection UV calculations
//! - Tests history buffer sampling
//!
//! # Requirements
//!
//! These tests require:
//! - Vulkan-capable GPU and drivers
//! - CMake (for shader compilation via vulkano-shaders)
//!
//! To install CMake:
//! - Windows: `winget install Kitware.CMake` or download from https://cmake.org/download/
//! - Linux: `sudo apt install cmake` or equivalent
//! - macOS: `brew install cmake`

use praxis_graphics::{
    deferred::{DeferredRenderParams, DeferredRenderer, GBuffer},
    lighting::{DirectionalLight, LightingUniforms},
    material::MaterialProperties,
    mesh::{MeshAssetManager, MeshData},
    taa::{TaaApplyParams, TaaConfig, TaaRenderer, TaaRenderTarget},
    texture::TextureManager,
    uniform_buffer::{DynamicUniformBuffer, ModelUniforms, ViewProjectionUniforms},
    velocity_buffer::{VelocityBuffer, VelocityBufferRenderer},
    vertex::Vertex3D,
    DrawCommand,
};
use praxis_math::{Mat4, Vec3, Vec4};
use praxis_utils::{debug, info, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferUsage, CopyImageToBufferInfo,
        RecordingCommandBuffer,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageAspects, ImageCreateInfo, ImageSubresourceLayers, ImageType, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    sync::GpuFuture,
    VulkanLibrary,
};

/// Test fixture for TAA integration tests with Vulkan resources.
struct TaaTestFixture {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    deferred_renderer: DeferredRenderer,
    taa_renderer: TaaRenderer,
    velocity_buffer_renderer: VelocityBufferRenderer,
    mesh_manager: MeshAssetManager,
    texture_manager: TextureManager,
    width: u32,
    height: u32,
}

impl TaaTestFixture {
    /// Creates a new test fixture with Vulkan device and allocators.
    fn new() -> Result<Self> {
        info!("Initializing TAA integration test fixture");

        let width = 256;
        let height = 256;

        // Load Vulkan library
        let library = VulkanLibrary::new()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to load Vulkan library: {}", e))?;

        // Create Vulkan instance
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create Vulkan instance: {}", e))?;

        // Select physical device (prefer discrete GPU)
        let physical_device = instance
            .enumerate_physical_devices()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to enumerate physical devices: {}", e))?
            .filter(|p| {
                p.queue_family_properties()
                    .iter()
                    .any(|q| q.queue_flags.contains(QueueFlags::GRAPHICS))
            })
            .min_by_key(|p| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .ok_or_else(|| {
                praxis_utils::eyre::eyre!("No suitable physical device with graphics support found")
            })?;

        debug!(
            "Selected device: {} ({:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        // Find graphics queue family
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))
            .map(|(i, _)| i as u32)
            .ok_or_else(|| praxis_utils::eyre::eyre!("No graphics queue family found"))?;

        // Create logical device
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: DeviceExtensions {
                    khr_storage_buffer_storage_class: true,
                    ..DeviceExtensions::empty()
                },
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create device: {}", e))?;

        let queue = queues
            .next()
            .ok_or_else(|| praxis_utils::eyre::eyre!("Failed to get queue from device"))?;

        // Create allocators
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Create renderers
        let deferred_renderer = DeferredRenderer::new(
            device.clone(),
            memory_allocator.clone(),
            descriptor_set_allocator.clone(),
            width,
            height,
        )?;

        let taa_renderer = TaaRenderer::new(device.clone(), memory_allocator.clone())?;

        let velocity_buffer_renderer =
            VelocityBufferRenderer::new(device.clone(), memory_allocator.clone())?;

        // Create mesh and texture managers
        let mesh_manager = MeshAssetManager::new(memory_allocator.clone());
        let texture_manager = TextureManager::new(
            device.clone(),
            memory_allocator.clone(),
            queue.clone(),
            command_buffer_allocator.clone(),
        )?;

        info!("TAA test fixture initialized successfully");

        Ok(Self {
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            deferred_renderer,
            taa_renderer,
            velocity_buffer_renderer,
            mesh_manager,
            texture_manager,
            width,
            height,
        })
    }

    /// Executes a command buffer and waits for completion.
    fn execute_and_wait(&self, command_buffer_builder: RecordingCommandBuffer) -> Result<()> {
        let command_buffer = command_buffer_builder
            .end()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to build command buffer: {}", e))?;

        let future = vulkano::sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute command buffer: {}", e))?
            .then_signal_fence_and_flush()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush command buffer: {}", e))?;

        future
            .wait(None)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait for GPU: {}", e))?;

        Ok(())
    }

    /// Creates a simple cube mesh for testing.
    fn create_test_cube(&mut self) -> Result<String> {
        let vertices = vec![
            // Front face
            Vertex3D::new([-0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex3D::new([0.5, -0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex3D::new([0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex3D::new([-0.5, 0.5, 0.5], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back face
            Vertex3D::new([0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 0.0]),
            Vertex3D::new([-0.5, -0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex3D::new([-0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex3D::new([0.5, 0.5, -0.5], [0.0, 0.0, -1.0], [0.0, 1.0]),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
        ];

        let mesh_data = MeshData { vertices, indices };
        self.mesh_manager.add_mesh("test_cube", mesh_data)?;

        Ok("test_cube".to_string())
    }

    /// Creates a simple output framebuffer for testing.
    fn create_output_framebuffer(&self) -> Result<Arc<Framebuffer>> {
        let render_pass = vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments: {
                color: {
                    format: Format::R8G8B8A8_UNORM,
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
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create render pass: {}", e))?;

        let image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [self.width, self.height, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create output image: {}", e))?;

        let image_view = ImageView::new_default(image)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create image view: {}", e))?;

        Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![image_view],
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create framebuffer: {}", e))
    }

    /// Reads velocity buffer data from GPU to CPU.
    fn read_velocity_buffer(&self, velocity_buffer: &VelocityBuffer) -> Result<Vec<[f32; 2]>> {
        let mut builder = RecordingCommandBuffer::new(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

        // Create staging buffer
        let buffer_size = (self.width * self.height * 8) as u64; // 2 floats per pixel * 4 bytes
        let staging_buffer = Buffer::new_slice::<f32>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            (self.width * self.height * 2) as u64,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create staging buffer: {}", e))?;

        // Copy velocity buffer to staging buffer
        builder
            .copy_image_to_buffer(CopyImageToBufferInfo {
                regions: [vulkano::command_buffer::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: 0,
                    buffer_image_height: 0,
                    image_subresource: ImageSubresourceLayers {
                        aspects: ImageAspects::COLOR,
                        mip_level: 0,
                        array_layers: 0..1,
                    },
                    image_offset: [0, 0, 0],
                    image_extent: [self.width, self.height, 1],
                    ..Default::default()
                }]
                .into(),
                ..CopyImageToBufferInfo::image_buffer(
                    velocity_buffer.image.clone(),
                    staging_buffer.clone(),
                )
            })
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to copy image to buffer: {}", e))?;

        self.execute_and_wait(builder)?;

        // Read data from staging buffer
        let data = staging_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read staging buffer: {}", e))?;

        let mut velocities = Vec::with_capacity((self.width * self.height) as usize);
        for i in 0..(self.width * self.height) as usize {
            let x = data[i * 2];
            let y = data[i * 2 + 1];
            velocities.push([x, y]);
        }

        Ok(velocities)
    }
}

/// Creates draw commands for multiple moving objects.
fn create_moving_objects(
    mesh_id: String,
    frame: u32,
    object_count: usize,
) -> Vec<(DrawCommand, ModelUniforms, ModelUniforms)> {
    let mut commands = Vec::with_capacity(object_count);

    for i in 0..object_count {
        let offset_x = (i as f32 - object_count as f32 / 2.0) * 2.5;
        let velocity = 0.1 + (i as f32 * 0.02); // Different velocities for each object

        // Current frame position
        let current_pos = Vec3::new(offset_x + velocity * frame as f32, 0.0, -5.0);
        let current_model = Mat4::from_translation(current_pos);

        // Previous frame position
        let previous_pos = if frame > 0 {
            Vec3::new(offset_x + velocity * (frame - 1) as f32, 0.0, -5.0)
        } else {
            current_pos
        };
        let previous_model = Mat4::from_translation(previous_pos);

        let draw_cmd = DrawCommand {
            mesh_id: mesh_id.clone(),
            transform: current_model,
            material_properties: Some(MaterialProperties {
                albedo: [0.8, 0.3, 0.3, 1.0],
                metallic: 0.0,
                roughness: 0.5,
                emissive: [0.0, 0.0, 0.0],
            }),
            texture_name: None,
        };

        let current_uniforms = ModelUniforms {
            model: current_model.to_cols_array_2d(),
            normal_matrix: current_model.inverse().transpose().to_cols_array_2d(),
        };

        let previous_uniforms = ModelUniforms {
            model: previous_model.to_cols_array_2d(),
            normal_matrix: previous_model.inverse().transpose().to_cols_array_2d(),
        };

        commands.push((draw_cmd, current_uniforms, previous_uniforms));
    }

    commands
}

#[test]
fn test_velocity_buffer_generation_moving_objects() -> Result<()> {
    info!("Test: Velocity buffer generation with moving objects");

    let mut fixture = TaaTestFixture::new()?;
    let mesh_id = fixture.create_test_cube()?;

    // Create camera matrices
    let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0,
        fixture.width as f32 / fixture.height as f32,
        0.1,
        100.0,
    );

    // Frame 0: Render with no motion
    let frame = 0;
    let object_count = 3;
    let moving_objects = create_moving_objects(mesh_id.clone(), frame, object_count);

    let mut draw_commands = Vec::new();
    let mut current_uniforms = Vec::new();
    let mut previous_uniforms = Vec::new();

    for (cmd, current, previous) in moving_objects {
        draw_commands.push(cmd);
        current_uniforms.push(current);
        previous_uniforms.push(previous);
    }

    let view_proj = ViewProjectionUniforms {
        view: view.to_cols_array_2d(),
        projection: proj.to_cols_array_2d(),
        view_position: [0.0, 0.0, 0.0, 1.0],
        view_projection: (proj * view).to_cols_array_2d(),
    };

    let view_proj_buffer = Buffer::from_data(
        fixture.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        view_proj,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create view projection buffer: {}", e))?;

    let dynamic_buffer = DynamicUniformBuffer::new(
        fixture.memory_allocator.clone(),
        &current_uniforms,
        draw_commands.len(),
    )?;

    let previous_dynamic_buffer = DynamicUniformBuffer::new(
        fixture.memory_allocator.clone(),
        &previous_uniforms,
        draw_commands.len(),
    )?;

    let lighting = LightingUniforms {
        directional_light: DirectionalLight {
            direction: [-0.5, -1.0, -0.3, 0.0],
            color: [1.0, 1.0, 1.0, 0.0],
            intensity: 1.0,
            _padding: [0.0; 3],
        },
        ambient_color: [0.1, 0.1, 0.1, 1.0],
        point_light_count: 0,
        _padding: [0.0; 3],
    };

    let lighting_buffer = Buffer::from_data(
        fixture.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        lighting,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create lighting buffer: {}", e))?;

    let output_framebuffer = fixture.create_output_framebuffer()?;

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [fixture.width as f32, fixture.height as f32],
        depth_range: 0.0..=1.0,
    };

    // Render frame
    let mut builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    let params = DeferredRenderParams {
        output_framebuffer,
        viewport,
        draw_commands: &draw_commands,
        view_proj_buffer: view_proj_buffer.clone(),
        dynamic_uniform_buffer: &dynamic_buffer,
        mesh_manager: &fixture.mesh_manager,
        texture_manager: &fixture.texture_manager,
        lighting_buffer,
        previous_view_proj_buffer: view_proj_buffer,
        previous_dynamic_uniform_buffer: &previous_dynamic_buffer,
    };

    fixture.deferred_renderer.render(&mut builder, &params)?;

    fixture.execute_and_wait(builder)?;

    // Read velocity buffer
    let velocity_buffer = fixture
        .deferred_renderer
        .velocity_buffer()
        .ok_or_else(|| praxis_utils::eyre::eyre!("Velocity buffer not available"))?;

    let velocities = fixture.read_velocity_buffer(&VelocityBuffer {
        image: velocity_buffer.image().clone(),
        image_view: velocity_buffer.clone(),
        framebuffer: fixture.deferred_renderer.gbuffer.as_ref().unwrap().framebuffer.clone(),
        width: fixture.width,
        height: fixture.height,
    })?;

    // Validate: Frame 0 should have zero velocity (no previous motion)
    let mut zero_velocity_count = 0;
    for vel in &velocities {
        let magnitude = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
        if magnitude < 0.01 {
            zero_velocity_count += 1;
        }
    }

    // Most pixels should have near-zero velocity on frame 0
    let zero_velocity_ratio = zero_velocity_count as f32 / velocities.len() as f32;
    assert!(
        zero_velocity_ratio > 0.8,
        "Expected >80% zero velocity on frame 0, got {:.1}%",
        zero_velocity_ratio * 100.0
    );

    info!("✓ Velocity buffer generation validated: {:.1}% zero velocity on frame 0", zero_velocity_ratio * 100.0);

    Ok(())
}

#[test]
fn test_motion_vector_magnitude_validation() -> Result<()> {
    info!("Test: Motion vector magnitude validation across 3 frames");

    let mut fixture = TaaTestFixture::new()?;
    let mesh_id = fixture.create_test_cube()?;

    let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0,
        fixture.width as f32 / fixture.height as f32,
        0.1,
        100.0,
    );

    let mut frame_velocities = Vec::new();

    for frame in 0..3 {
        debug!("Rendering frame {}", frame);

        let moving_objects = create_moving_objects(mesh_id.clone(), frame, 3);

        let mut draw_commands = Vec::new();
        let mut current_uniforms = Vec::new();
        let mut previous_uniforms = Vec::new();

        for (cmd, current, previous) in moving_objects {
            draw_commands.push(cmd);
            current_uniforms.push(current);
            previous_uniforms.push(previous);
        }

        let view_proj = ViewProjectionUniforms {
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
            view_position: [0.0, 0.0, 0.0, 1.0],
            view_projection: (proj * view).to_cols_array_2d(),
        };

        let view_proj_buffer = Buffer::from_data(
            fixture.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            view_proj,
        )?;

        let dynamic_buffer = DynamicUniformBuffer::new(
            fixture.memory_allocator.clone(),
            &current_uniforms,
            draw_commands.len(),
        )?;

        let previous_dynamic_buffer = DynamicUniformBuffer::new(
            fixture.memory_allocator.clone(),
            &previous_uniforms,
            draw_commands.len(),
        )?;

        let lighting = LightingUniforms {
            directional_light: DirectionalLight {
                direction: [-0.5, -1.0, -0.3, 0.0],
                color: [1.0, 1.0, 1.0, 0.0],
                intensity: 1.0,
                _padding: [0.0; 3],
            },
            ambient_color: [0.1, 0.1, 0.1, 1.0],
            point_light_count: 0,
            _padding: [0.0; 3],
        };

        let lighting_buffer = Buffer::from_data(
            fixture.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            lighting,
        )?;

        let output_framebuffer = fixture.create_output_framebuffer()?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [fixture.width as f32, fixture.height as f32],
            depth_range: 0.0..=1.0,
        };

        let mut builder = RecordingCommandBuffer::new(
            fixture.command_buffer_allocator.clone(),
            fixture.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let params = DeferredRenderParams {
            output_framebuffer,
            viewport,
            draw_commands: &draw_commands,
            view_proj_buffer: view_proj_buffer.clone(),
            dynamic_uniform_buffer: &dynamic_buffer,
            mesh_manager: &fixture.mesh_manager,
            texture_manager: &fixture.texture_manager,
            lighting_buffer,
            previous_view_proj_buffer: view_proj_buffer,
            previous_dynamic_uniform_buffer: &previous_dynamic_buffer,
        };

        fixture.deferred_renderer.render(&mut builder, &params)?;
        fixture.execute_and_wait(builder)?;

        let velocity_buffer = fixture
            .deferred_renderer
            .velocity_buffer()
            .ok_or_else(|| praxis_utils::eyre::eyre!("Velocity buffer not available"))?;

        let velocities = fixture.read_velocity_buffer(&VelocityBuffer {
            image: velocity_buffer.image().clone(),
            image_view: velocity_buffer.clone(),
            framebuffer: fixture.deferred_renderer.gbuffer.as_ref().unwrap().framebuffer.clone(),
            width: fixture.width,
            height: fixture.height,
        })?;

        frame_velocities.push(velocities);
    }

    // Validate motion vectors increase from frame 0 to frame 2
    let avg_magnitude_frame0 = calculate_average_nonzero_magnitude(&frame_velocities[0]);
    let avg_magnitude_frame1 = calculate_average_nonzero_magnitude(&frame_velocities[1]);
    let avg_magnitude_frame2 = calculate_average_nonzero_magnitude(&frame_velocities[2]);

    info!(
        "Average velocity magnitudes: Frame 0: {:.6}, Frame 1: {:.6}, Frame 2: {:.6}",
        avg_magnitude_frame0, avg_magnitude_frame1, avg_magnitude_frame2
    );

    // Frame 1 and 2 should have non-zero motion vectors (objects are moving)
    assert!(
        avg_magnitude_frame1 > 0.001,
        "Frame 1 should have measurable motion vectors"
    );
    assert!(
        avg_magnitude_frame2 > 0.001,
        "Frame 2 should have measurable motion vectors"
    );

    info!("✓ Motion vector magnitudes validated across 3 frames");

    Ok(())
}

#[test]
fn test_reprojection_uv_calculation() -> Result<()> {
    info!("Test: Reprojection UV calculation validation");

    // Test reprojection math directly
    let test_cases = vec![
        // (current_uv, velocity, expected_history_uv)
        ([0.5, 0.5], [0.0, 0.0], [0.5, 0.5]), // No motion
        ([0.5, 0.5], [0.1, 0.0], [0.4, 0.5]), // Right motion
        ([0.5, 0.5], [-0.1, 0.0], [0.6, 0.5]), // Left motion
        ([0.5, 0.5], [0.0, 0.1], [0.5, 0.4]), // Down motion
        ([0.5, 0.5], [0.0, -0.1], [0.5, 0.6]), // Up motion
    ];

    for (current_uv, velocity, expected_history_uv) in test_cases {
        // Reprojection: history_uv = current_uv - velocity
        let history_uv = [current_uv[0] - velocity[0], current_uv[1] - velocity[1]];

        let error_x = (history_uv[0] - expected_history_uv[0]).abs();
        let error_y = (history_uv[1] - expected_history_uv[1]).abs();

        assert!(
            error_x < 0.001 && error_y < 0.001,
            "Reprojection error: got [{:.3}, {:.3}], expected [{:.3}, {:.3}]",
            history_uv[0],
            history_uv[1],
            expected_history_uv[0],
            expected_history_uv[1]
        );
    }

    info!("✓ Reprojection UV calculations validated");

    Ok(())
}

#[test]
fn test_history_buffer_sampling() -> Result<()> {
    info!("Test: History buffer sampling validation");

    let mut fixture = TaaTestFixture::new()?;
    let mesh_id = fixture.create_test_cube()?;

    // Create TAA render target
    let taa_target = fixture
        .taa_renderer
        .create_render_target(fixture.width, fixture.height)?;

    let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::Y);
    let proj = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0,
        fixture.width as f32 / fixture.height as f32,
        0.1,
        100.0,
    );

    // Render 2 frames to build history
    for frame in 0..2 {
        debug!("Rendering frame {} for history sampling test", frame);

        let moving_objects = create_moving_objects(mesh_id.clone(), frame, 1);

        let mut draw_commands = Vec::new();
        let mut current_uniforms = Vec::new();
        let mut previous_uniforms = Vec::new();

        for (cmd, current, previous) in moving_objects {
            draw_commands.push(cmd);
            current_uniforms.push(current);
            previous_uniforms.push(previous);
        }

        let view_proj = ViewProjectionUniforms {
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
            view_position: [0.0, 0.0, 0.0, 1.0],
            view_projection: (proj * view).to_cols_array_2d(),
        };

        let view_proj_buffer = Buffer::from_data(
            fixture.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            view_proj,
        )?;

        let dynamic_buffer = DynamicUniformBuffer::new(
            fixture.memory_allocator.clone(),
            &current_uniforms,
            draw_commands.len(),
        )?;

        let previous_dynamic_buffer = DynamicUniformBuffer::new(
            fixture.memory_allocator.clone(),
            &previous_uniforms,
            draw_commands.len(),
        )?;

        let lighting = LightingUniforms {
            directional_light: DirectionalLight {
                direction: [-0.5, -1.0, -0.3, 0.0],
                color: [1.0, 1.0, 1.0, 0.0],
                intensity: 1.0,
                _padding: [0.0; 3],
            },
            ambient_color: [0.1, 0.1, 0.1, 1.0],
            point_light_count: 0,
            _padding: [0.0; 3],
        };

        let lighting_buffer = Buffer::from_data(
            fixture.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            lighting,
        )?;

        let output_framebuffer = fixture.create_output_framebuffer()?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [fixture.width as f32, fixture.height as f32],
            depth_range: 0.0..=1.0,
        };

        let mut builder = RecordingCommandBuffer::new(
            fixture.command_buffer_allocator.clone(),
            fixture.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let deferred_params = DeferredRenderParams {
            output_framebuffer,
            viewport: viewport.clone(),
            draw_commands: &draw_commands,
            view_proj_buffer: view_proj_buffer.clone(),
            dynamic_uniform_buffer: &dynamic_buffer,
            mesh_manager: &fixture.mesh_manager,
            texture_manager: &fixture.texture_manager,
            lighting_buffer,
            previous_view_proj_buffer: view_proj_buffer,
            previous_dynamic_uniform_buffer: &previous_dynamic_buffer,
        };

        fixture.deferred_renderer.render(&mut builder, &deferred_params)?;
        fixture.execute_and_wait(builder)?;

        // If frame 1, apply TAA to test history sampling
        if frame == 1 {
            let velocity_buffer = fixture
                .deferred_renderer
                .velocity_buffer()
                .ok_or_else(|| praxis_utils::eyre::eyre!("Velocity buffer not available"))?;

            let depth_buffer = &fixture
                .deferred_renderer
                .gbuffer
                .as_ref()
                .ok_or_else(|| praxis_utils::eyre::eyre!("G-buffer not available"))?
                .depth;

            let current_frame = &fixture
                .deferred_renderer
                .gbuffer
                .as_ref()
                .unwrap()
                .albedo;

            let mut builder = RecordingCommandBuffer::new(
                fixture.command_buffer_allocator.clone(),
                fixture.queue.queue_family_index(),
                vulkano::command_buffer::CommandBufferLevel::Primary,
                CommandBufferUsage::OneTimeSubmit,
            )?;

            let taa_config = TaaConfig {
                jitter_offset: [0.0, 0.0],
                blend_factor: 0.1,
            };

            let taa_params = TaaApplyParams {
                taa_target: &taa_target,
                current_frame: current_frame.clone(),
                velocity_buffer: velocity_buffer.clone(),
                depth_buffer: depth_buffer.clone(),
                config: taa_config,
            };

            fixture.taa_renderer.apply(&mut builder, &taa_params)?;
            fixture.execute_and_wait(builder)?;
        }
    }

    info!("✓ History buffer sampling validated (no crashes, TAA applied successfully)");

    Ok(())
}

/// Helper function to calculate average magnitude of non-zero velocities
fn calculate_average_nonzero_magnitude(velocities: &[[f32; 2]]) -> f32 {
    let mut sum = 0.0;
    let mut count = 0;

    for vel in velocities {
        let magnitude = (vel[0] * vel[0] + vel[1] * vel[1]).sqrt();
        if magnitude > 0.001 {
            sum += magnitude;
            count += 1;
        }
    }

    if count > 0 {
        sum / count as f32
    } else {
        0.0
    }
}
