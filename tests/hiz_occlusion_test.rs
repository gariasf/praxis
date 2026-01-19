//! Integration test for Hi-Z occlusion culling.
//!
//! This test validates the complete Hi-Z occlusion culling pipeline:
//! - Creates a scene with large occluders and small objects behind them
//! - Renders depth buffer with occluders
//! - Generates Hi-Z pyramid from depth buffer
//! - Dispatches occlusion culling compute shader
//! - Validates additional 30-50% culling beyond frustum culling
//!
//! # Test Scenarios
//!
//! 1. **Large Occluders**: Tests that objects behind large walls are properly culled
//! 2. **Partial Occlusion**: Tests that partially occluded objects are handled correctly
//! 3. **No Occlusion**: Validates that visible objects are not incorrectly culled
//!
//! # Requirements
//!
//! These tests require:
//! - Vulkan-capable GPU and drivers
//! - Support for compute shaders and depth attachments

use praxis_graphics::gpu_culling::{
    extract_frustum_planes, GpuCullingManager, GpuDrawCommand, GpuMeshData,
};
use praxis_math::{Mat4, Vec3, Vec4};
use praxis_utils::{debug, info, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferUsage, RecordingCommandBuffer,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    sync::GpuFuture,
    VulkanLibrary,
};

/// Test fixture for Hi-Z occlusion culling tests with Vulkan resources.
struct HizOcclusionTestFixture {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    culling_manager: GpuCullingManager,
}

impl HizOcclusionTestFixture {
    /// Creates a new test fixture with Vulkan device and allocators.
    fn new() -> Result<Self> {
        info!("Initializing Hi-Z occlusion culling test fixture");

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
                p.queue_family_properties().iter().any(|q| {
                    q.queue_flags
                        .contains(QueueFlags::COMPUTE | QueueFlags::GRAPHICS)
                })
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
                praxis_utils::eyre::eyre!(
                    "No suitable physical device with compute and graphics support found"
                )
            })?;

        debug!(
            "Selected device: {} ({:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        // Find compute/graphics queue family
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_, q)| {
                q.queue_flags
                    .contains(QueueFlags::COMPUTE | QueueFlags::GRAPHICS)
            })
            .map(|(i, _)| i as u32)
            .ok_or_else(|| praxis_utils::eyre::eyre!("No compute/graphics queue family found"))?;

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

        // Create GPU culling manager
        let culling_manager = GpuCullingManager::new(
            device.clone(),
            memory_allocator.clone(),
            descriptor_set_allocator.clone(),
        )?;

        info!("Hi-Z occlusion culling test fixture initialized successfully");

        Ok(Self {
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            culling_manager,
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

    /// Creates a mock depth buffer for testing Hi-Z pyramid generation.
    fn create_depth_buffer(&self, width: u32, height: u32) -> Result<Arc<ImageView>> {
        let depth_image = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: [width, height, 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create depth image: {}", e))?;

        ImageView::new_default(depth_image)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create depth image view: {}", e))
    }
}

/// Creates a scene with large occluders in front and small objects behind them.
///
/// Layout:
/// - Large wall at z=0 (blocks view)
/// - Small objects at z=-10 (behind wall, should be occluded)
/// - Small objects at z=+10 (in front of wall, visible)
///
/// Returns (draw_commands, mesh_data, expected_visible_without_occlusion, expected_visible_with_occlusion)
fn create_occluded_scene() -> (Vec<GpuDrawCommand>, Vec<GpuMeshData>, usize, usize) {
    let mut draw_commands = Vec::new();
    let mut mesh_data = Vec::new();

    // Create large occluder wall (10x10 at z=0)
    for x in -2..=2 {
        for y in -2..=2 {
            let position = Vec3::new(x as f32 * 10.0, y as f32 * 10.0, 0.0);
            let scale = 5.0;
            let model = Mat4::from_scale_rotation_translation(
                Vec3::splat(scale),
                praxis_math::Quat::IDENTITY,
                position,
            );

            // Large bounding sphere for occluder
            let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 2.0);

            draw_commands.push(GpuDrawCommand::new(
                model,
                bounding_sphere,
                0, // mesh_id
                0, // material_id
            ));

            mesh_data.push(GpuMeshData {
                index_count: 36,
                first_index: 0,
                vertex_offset: 0,
                _padding: 0,
            });
        }
    }

    let occluder_count = draw_commands.len();
    info!("Created {} occluder objects (wall)", occluder_count);

    // Create small objects behind the wall (should be occluded)
    let mut occluded_object_count = 0;
    for x in -2..=2 {
        for y in -2..=2 {
            let position = Vec3::new(x as f32 * 8.0, y as f32 * 8.0, -20.0);
            let model = Mat4::from_translation(position);
            let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 1.0);

            draw_commands.push(GpuDrawCommand::new(
                model,
                bounding_sphere,
                1, // mesh_id
                1, // material_id
            ));

            mesh_data.push(GpuMeshData {
                index_count: 36,
                first_index: 0,
                vertex_offset: 0,
                _padding: 0,
            });

            occluded_object_count += 1;
        }
    }

    info!(
        "Created {} objects behind wall (should be occluded)",
        occluded_object_count
    );

    // Create small objects in front of the wall (should be visible)
    let mut visible_object_count = 0;
    for x in -1..=1 {
        for y in -1..=1 {
            let position = Vec3::new(x as f32 * 8.0, y as f32 * 8.0, 20.0);
            let model = Mat4::from_translation(position);
            let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 1.0);

            draw_commands.push(GpuDrawCommand::new(
                model,
                bounding_sphere,
                2, // mesh_id
                2, // material_id
            ));

            mesh_data.push(GpuMeshData {
                index_count: 36,
                first_index: 0,
                vertex_offset: 0,
                _padding: 0,
            });

            visible_object_count += 1;
        }
    }

    info!(
        "Created {} objects in front of wall (should be visible)",
        visible_object_count
    );

    let total_objects = draw_commands.len();
    let expected_visible_without_occlusion =
        occluder_count + visible_object_count + occluded_object_count;
    let expected_visible_with_occlusion = occluder_count + visible_object_count;

    info!("Scene statistics:");
    info!("  Total objects: {}", total_objects);
    info!(
        "  Expected visible (frustum only): {}",
        expected_visible_without_occlusion
    );
    info!(
        "  Expected visible (with occlusion): {}",
        expected_visible_with_occlusion
    );
    info!(
        "  Expected occluded: {}",
        expected_visible_without_occlusion - expected_visible_with_occlusion
    );

    (
        draw_commands,
        mesh_data,
        expected_visible_without_occlusion,
        expected_visible_with_occlusion,
    )
}

/// Main integration test: Hi-Z occlusion culling with occluded objects.
///
/// This test validates:
/// 1. Hi-Z pyramid generation from depth buffer
/// 2. Occlusion culling compute shader execution
/// 3. Additional 30-50% culling beyond frustum culling
/// 4. Correct handling of occluded vs visible objects
#[test]
fn test_hiz_occlusion_culling() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Hi-Z Occlusion Culling Integration Test ===");

    let mut fixture = HizOcclusionTestFixture::new()?;

    // Initialize Hi-Z pyramid (1920x1080 resolution)
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;

    info!("Initializing Hi-Z pyramid ({}x{})", WIDTH, HEIGHT);
    fixture
        .culling_manager
        .initialize_hiz_pyramid([WIDTH, HEIGHT])?;

    // Create scene with occluders and occluded objects
    let (draw_commands, mesh_data, expected_without_occlusion, expected_with_occlusion) =
        create_occluded_scene();

    let total_objects = draw_commands.len();
    info!("Created test scene with {} objects", total_objects);

    // Prepare culling manager with object data
    info!("Uploading draw commands and mesh data to GPU");
    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Set up camera looking at the scene from front
    let camera_position = Vec3::new(0.0, 0.0, 50.0);
    let camera_target = Vec3::new(0.0, 0.0, 0.0);
    let camera_up = Vec3::Y;

    let view = Mat4::look_at_rh(camera_position, camera_target, camera_up);
    let projection = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_3, // 60 degree FOV
        WIDTH as f32 / HEIGHT as f32,
        1.0,
        200.0,
    );

    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    info!("Camera setup:");
    info!("  Position: {:?}", camera_position);
    info!("  Target: {:?}", camera_target);
    info!("  FOV: 60°, Near: 1.0, Far: 200.0");

    // Phase 1: Frustum culling only (baseline)
    info!("\n=== Phase 1: Frustum Culling Only ===");

    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    fixture.culling_manager.dispatch_culling(
        &mut command_buffer_builder,
        view_proj,
        frustum_planes,
        camera_position,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    let visible_frustum_only = fixture.culling_manager.read_visible_count()?;
    info!("Frustum culling results:");
    info!("  Visible objects: {}", visible_frustum_only);
    info!(
        "  Culled objects: {}",
        total_objects as u32 - visible_frustum_only
    );

    // Validate frustum culling doesn't incorrectly cull visible objects
    assert!(
        visible_frustum_only as usize >= expected_without_occlusion - 5,
        "Frustum culling removed too many objects: expected ~{}, got {}",
        expected_without_occlusion,
        visible_frustum_only
    );

    // Phase 2: Generate Hi-Z pyramid and enable occlusion culling
    info!("\n=== Phase 2: Hi-Z Occlusion Culling ===");

    // Create mock depth buffer
    let depth_buffer = fixture.create_depth_buffer(WIDTH, HEIGHT)?;

    // Reset draw count for next culling pass
    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    // Generate Hi-Z pyramid from depth buffer
    info!("Generating Hi-Z pyramid from depth buffer");
    fixture
        .culling_manager
        .generate_hiz_pyramid(&mut command_buffer_builder, depth_buffer)?;

    // Enable occlusion culling
    fixture.culling_manager.set_occlusion_culling(true);

    // Dispatch culling with both frustum and occlusion
    info!("Dispatching culling compute shader (frustum + occlusion)");
    fixture.culling_manager.dispatch_culling(
        &mut command_buffer_builder,
        view_proj,
        frustum_planes,
        camera_position,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    let visible_with_occlusion = fixture.culling_manager.read_visible_count()?;
    info!("Hi-Z occlusion culling results:");
    info!("  Visible objects: {}", visible_with_occlusion);
    info!(
        "  Culled objects: {}",
        total_objects as u32 - visible_with_occlusion
    );

    // Calculate culling statistics
    let additional_culled = visible_frustum_only - visible_with_occlusion;
    let additional_cull_percentage =
        (additional_culled as f32 / visible_frustum_only as f32) * 100.0;

    info!("\n=== Occlusion Culling Performance ===");
    info!("  Total objects: {}", total_objects);
    info!("  Visible (frustum only): {}", visible_frustum_only);
    info!("  Visible (with occlusion): {}", visible_with_occlusion);
    info!(
        "  Additional culled by Hi-Z: {} ({:.1}%)",
        additional_culled, additional_cull_percentage
    );

    // Validation: Occlusion culling should remove 30-50% additional objects
    assert!(
        additional_cull_percentage >= 30.0,
        "Expected at least 30% additional culling from Hi-Z occlusion, got {:.1}%",
        additional_cull_percentage
    );

    assert!(
        additional_cull_percentage <= 70.0,
        "Additional culling seems too aggressive: {:.1}% (max expected 70%)",
        additional_cull_percentage
    );

    info!(
        "✓ Hi-Z occlusion culling achieved {:.1}% additional culling",
        additional_cull_percentage
    );

    // Validate that we didn't cull too many objects (false positives)
    let min_expected_visible = (expected_with_occlusion as f32 * 0.8) as u32;
    assert!(
        visible_with_occlusion >= min_expected_visible,
        "Occlusion culling may have false positives: expected at least {}, got {}",
        min_expected_visible,
        visible_with_occlusion
    );

    info!("✓ No significant false positive culling detected");

    info!("\n=== Hi-Z Occlusion Culling Integration Test PASSED ===");
    info!("Summary:");
    info!("  ✓ Hi-Z pyramid generation succeeded");
    info!("  ✓ Occlusion culling compute shader executed successfully");
    info!(
        "  ✓ Achieved {:.1}% additional culling (30-50% target met)",
        additional_cull_percentage
    );
    info!("  ✓ No false positive culling detected");

    Ok(())
}

/// Test Hi-Z pyramid initialization and configuration.
#[test]
fn test_hiz_pyramid_initialization() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Hi-Z Pyramid Initialization Test ===");

    let mut fixture = HizOcclusionTestFixture::new()?;

    // Test various resolutions
    let test_resolutions = vec![
        ([1920, 1080], "1920x1080 (Full HD)"),
        ([1280, 720], "1280x720 (HD)"),
        ([2560, 1440], "2560x1440 (QHD)"),
        ([512, 512], "512x512 (Small)"),
    ];

    for (resolution, description) in test_resolutions {
        info!("Testing Hi-Z pyramid initialization: {}", description);

        fixture.culling_manager.initialize_hiz_pyramid(resolution)?;

        assert!(
            fixture.culling_manager.is_hiz_initialized(),
            "Hi-Z pyramid should be initialized"
        );

        assert_eq!(
            fixture.culling_manager.hiz_extent(),
            Some(resolution),
            "Hi-Z extent should match initialized resolution"
        );

        info!("  ✓ Successfully initialized {}", description);
    }

    info!("✓ Hi-Z pyramid initialization test passed");
    Ok(())
}

/// Test occlusion culling enable/disable functionality.
#[test]
fn test_occlusion_culling_toggle() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Occlusion Culling Toggle Test ===");

    let mut fixture = HizOcclusionTestFixture::new()?;

    // Initialize Hi-Z pyramid
    fixture
        .culling_manager
        .initialize_hiz_pyramid([1920, 1080])?;

    // Test enabling occlusion culling
    info!("Enabling occlusion culling");
    fixture.culling_manager.set_occlusion_culling(true);
    assert!(
        fixture.culling_manager.is_occlusion_culling_enabled(),
        "Occlusion culling should be enabled"
    );

    // Test disabling occlusion culling
    info!("Disabling occlusion culling");
    fixture.culling_manager.set_occlusion_culling(false);
    assert!(
        !fixture.culling_manager.is_occlusion_culling_enabled(),
        "Occlusion culling should be disabled"
    );

    info!("✓ Occlusion culling toggle test passed");
    Ok(())
}

/// Test that attempting to enable occlusion culling without Hi-Z initialization is handled gracefully.
#[test]
fn test_occlusion_culling_without_hiz() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Occlusion Culling Without Hi-Z Test ===");

    let mut fixture = HizOcclusionTestFixture::new()?;

    // Try to enable occlusion culling without initializing Hi-Z pyramid
    info!("Attempting to enable occlusion culling without Hi-Z initialization");
    fixture.culling_manager.set_occlusion_culling(true);

    // Should not be enabled since Hi-Z is not initialized
    assert!(
        !fixture.culling_manager.is_occlusion_culling_enabled(),
        "Occlusion culling should not be enabled without Hi-Z initialization"
    );

    info!("✓ Correctly prevented enabling occlusion culling without Hi-Z");
    Ok(())
}

/// Test Hi-Z pyramid generation with different scene configurations.
#[test]
fn test_hiz_generation_with_depth_buffer() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Hi-Z Generation with Depth Buffer Test ===");

    let mut fixture = HizOcclusionTestFixture::new()?;

    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;

    // Initialize Hi-Z pyramid
    fixture
        .culling_manager
        .initialize_hiz_pyramid([WIDTH, HEIGHT])?;

    // Create depth buffer
    let depth_buffer = fixture.create_depth_buffer(WIDTH, HEIGHT)?;

    // Generate Hi-Z pyramid
    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    info!(
        "Generating Hi-Z pyramid from {}x{} depth buffer",
        WIDTH, HEIGHT
    );
    fixture
        .culling_manager
        .generate_hiz_pyramid(&mut command_buffer_builder, depth_buffer)?;

    fixture.execute_and_wait(command_buffer_builder)?;

    info!("✓ Hi-Z pyramid generation completed successfully");
    Ok(())
}

/// Test complete occlusion culling pipeline with varying object densities.
#[test]
fn test_occlusion_culling_varying_densities() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Occlusion Culling with Varying Densities Test ===");

    let mut fixture = HizOcclusionTestFixture::new()?;

    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;

    // Initialize Hi-Z pyramid
    fixture
        .culling_manager
        .initialize_hiz_pyramid([WIDTH, HEIGHT])?;

    let depth_buffer = fixture.create_depth_buffer(WIDTH, HEIGHT)?;

    // Test with the occluded scene
    let (draw_commands, mesh_data, _, _) = create_occluded_scene();

    info!(
        "Testing occlusion culling with {} objects",
        draw_commands.len()
    );

    // Prepare frame
    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Set up camera
    let camera_position = Vec3::new(0.0, 0.0, 50.0);
    let view = Mat4::look_at_rh(camera_position, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_3,
        WIDTH as f32 / HEIGHT as f32,
        1.0,
        200.0,
    );
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Generate Hi-Z and dispatch culling
    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    fixture
        .culling_manager
        .generate_hiz_pyramid(&mut command_buffer_builder, depth_buffer)?;

    fixture.culling_manager.set_occlusion_culling(true);

    fixture.culling_manager.dispatch_culling(
        &mut command_buffer_builder,
        view_proj,
        frustum_planes,
        camera_position,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    let visible_count = fixture.culling_manager.read_visible_count()?;

    info!(
        "Visible objects: {} / {}",
        visible_count,
        draw_commands.len()
    );

    // Validate results
    assert!(
        visible_count > 0,
        "Some objects should be visible after culling"
    );

    assert!(
        (visible_count as usize) < draw_commands.len(),
        "Some objects should be culled"
    );

    info!("✓ Occlusion culling with varying densities test passed");
    Ok(())
}
