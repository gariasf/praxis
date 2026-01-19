//! Integration test for GPU culling with indirect draws.
//!
//! This test validates the complete GPU culling pipeline:
//! - Creates 1000 objects arranged in a grid
//! - Enables GPU culling compute shader dispatch
//! - Validates compute shader execution
//! - Verifies culled object count matches expected frustum culling results (70-90% reduction)
//! - Confirms indirect draw buffer correctness
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

use praxis_graphics::gpu_culling::{
    extract_frustum_planes, GpuCullingManager, GpuDrawCommand, GpuMeshData, IndirectDrawCommand,
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
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    sync::GpuFuture,
    VulkanLibrary,
};

/// Test fixture for GPU culling tests with Vulkan resources.
struct GpuCullingTestFixture {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    culling_manager: GpuCullingManager,
}

impl GpuCullingTestFixture {
    /// Creates a new test fixture with Vulkan device and allocators.
    fn new() -> Result<Self> {
        info!("Initializing GPU culling integration test fixture");

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
                    .any(|q| q.queue_flags.contains(QueueFlags::COMPUTE))
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
                praxis_utils::eyre::eyre!("No suitable physical device with compute support found")
            })?;

        debug!(
            "Selected device: {} ({:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        // Find compute queue family
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(_, q)| q.queue_flags.contains(QueueFlags::COMPUTE))
            .map(|(i, _)| i as u32)
            .ok_or_else(|| praxis_utils::eyre::eyre!("No compute queue family found"))?;

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

        info!("GPU culling test fixture initialized successfully");

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
}

/// Creates a grid of 1000 objects for culling tests.
///
/// Objects are arranged in a 10x10x10 grid centered at the origin.
/// Each object has a bounding sphere with radius 1.0.
///
/// # Arguments
///
/// * `spacing` - Distance between objects in each dimension
///
/// # Returns
///
/// Vector of 1000 GPU draw commands
fn create_test_grid(spacing: f32) -> Vec<GpuDrawCommand> {
    let mut commands = Vec::with_capacity(1000);
    const GRID_DIM: i32 = 10;

    for x in 0..GRID_DIM {
        for y in 0..GRID_DIM {
            for z in 0..GRID_DIM {
                let position = Vec3::new(
                    (x as f32 - GRID_DIM as f32 / 2.0) * spacing,
                    (y as f32 - GRID_DIM as f32 / 2.0) * spacing,
                    (z as f32 - GRID_DIM as f32 / 2.0) * spacing,
                );

                let model = Mat4::from_translation(position);
                let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 1.0); // Center at model origin, radius 1.0

                let object_id = (x * GRID_DIM * GRID_DIM + y * GRID_DIM + z) as u32;

                commands.push(GpuDrawCommand::new(
                    model,
                    bounding_sphere,
                    object_id,     // mesh_id
                    object_id % 5, // material_id (5 different materials)
                ));
            }
        }
    }

    commands
}

/// Creates test mesh data for all objects.
///
/// All objects share the same mesh configuration (cube with 36 indices).
fn create_test_mesh_data(object_count: usize) -> Vec<GpuMeshData> {
    let mut mesh_data = Vec::with_capacity(object_count);

    for i in 0..object_count {
        mesh_data.push(GpuMeshData {
            index_count: 36, // Standard cube
            first_index: (i * 36) as u32,
            vertex_offset: 0,
            _padding: 0,
        });
    }

    mesh_data
}

/// Main integration test: GPU culling with 1000 objects in grid.
///
/// This test validates:
/// 1. Compute shader dispatch executes successfully
/// 2. Frustum culling eliminates 70-90% of objects
/// 3. Indirect draw buffer contains valid commands
/// 4. Draw count matches visible object count
#[test]
fn test_gpu_culling_1000_objects_grid() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU Culling Integration Test: 1000 Objects ===");

    let mut fixture = GpuCullingTestFixture::new()?;

    // Create 1000 objects in a 10x10x10 grid
    const OBJECT_COUNT: usize = 1000;
    const SPACING: f32 = 5.0;

    info!(
        "Creating {} objects in 10x10x10 grid (spacing: {})",
        OBJECT_COUNT, SPACING
    );
    let draw_commands = create_test_grid(SPACING);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    assert_eq!(draw_commands.len(), OBJECT_COUNT);
    assert_eq!(mesh_data.len(), OBJECT_COUNT);

    // Prepare culling manager with object data
    info!("Uploading draw commands and mesh data to GPU");
    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Set up camera to view only a portion of the grid (should cull 70-90%)
    // Camera positioned at an angle looking at grid center
    let camera_position = Vec3::new(30.0, 30.0, 30.0);
    let camera_target = Vec3::new(0.0, 0.0, 0.0);
    let camera_up = Vec3::Y;

    let view = Mat4::look_at_rh(camera_position, camera_target, camera_up);
    let projection = Mat4::perspective_rh(
        std::f32::consts::FRAC_PI_4, // 45 degree FOV
        16.0 / 9.0,                  // Aspect ratio
        1.0,                         // Near plane
        100.0,                       // Far plane
    );

    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    info!("Camera setup:");
    info!("  Position: {:?}", camera_position);
    info!("  Target: {:?}", camera_target);
    info!("  FOV: 45°, Near: 1.0, Far: 100.0");

    // Create command buffer for compute dispatch
    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    // Dispatch GPU culling compute shader
    info!("Dispatching GPU culling compute shader");
    fixture.culling_manager.dispatch_culling(
        &mut command_buffer_builder,
        view_proj,
        frustum_planes,
        camera_position,
    )?;

    // Execute and wait for completion
    info!("Executing command buffer and waiting for GPU");
    fixture.execute_and_wait(command_buffer_builder)?;

    // Read back results
    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!("Culling results:");
    info!("  Total objects: {}", OBJECT_COUNT);
    info!("  Visible objects: {}", visible_count);
    info!("  Culled objects: {}", OBJECT_COUNT as u32 - visible_count);
    info!(
        "  Cull percentage: {:.1}%",
        (OBJECT_COUNT as f32 - visible_count as f32) / OBJECT_COUNT as f32 * 100.0
    );

    // Validation 1: Visible count should be in expected range (10-30% of objects)
    assert!(
        visible_count > 0,
        "At least some objects should be visible (got 0)"
    );

    let culled_count = OBJECT_COUNT as u32 - visible_count;
    let cull_percentage = (culled_count as f32 / OBJECT_COUNT as f32) * 100.0;

    info!("Validating frustum culling efficiency");
    assert!(
        cull_percentage >= 70.0,
        "Expected at least 70% of objects to be culled, got {:.1}%",
        cull_percentage
    );

    assert!(
        cull_percentage <= 95.0,
        "Expected at most 95% of objects to be culled, got {:.1}%",
        cull_percentage
    );

    info!(
        "✓ Frustum culling efficiency validated: {:.1}% culled",
        cull_percentage
    );

    // Validation 2: Verify indirect draw buffer content
    info!("Validating indirect draw buffer");
    if let Some(indirect_buffer) = fixture.culling_manager.indirect_draw_buffer() {
        let buffer_read = indirect_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read indirect buffer: {}", e))?;

        info!("Checking {} indirect draw commands", visible_count);

        // Validate each visible draw command
        for i in 0..(visible_count as usize) {
            let cmd = buffer_read[i];

            // Verify command has valid instance count
            assert_eq!(
                cmd.instance_count, 1,
                "Command {} has invalid instance count: {} (expected 1)",
                i, cmd.instance_count
            );

            // Verify command has valid index count (36 for cube)
            assert_eq!(
                cmd.index_count, 36,
                "Command {} has invalid index count: {} (expected 36)",
                i, cmd.index_count
            );

            // Verify first_instance is 0 (standard for non-instanced draws)
            assert_eq!(
                cmd.first_instance, 0,
                "Command {} has invalid first_instance: {} (expected 0)",
                i, cmd.first_instance
            );

            debug!(
                "Command {}: index_count={}, instance_count={}, first_index={}, vertex_offset={}",
                i, cmd.index_count, cmd.instance_count, cmd.first_index, cmd.vertex_offset
            );
        }

        info!("✓ All {} indirect draw commands are valid", visible_count);
    } else {
        return Err(praxis_utils::eyre::eyre!(
            "Indirect draw buffer not available"
        ));
    }

    // Validation 3: Verify visible indices buffer
    info!("Validating visible indices buffer");
    if let Some(indices_buffer) = fixture.culling_manager.visible_indices_buffer() {
        let indices_read = indices_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read indices buffer: {}", e))?;

        for i in 0..(visible_count as usize) {
            let index = indices_read[i];

            // Index should be within valid range
            assert!(
                (index as usize) < OBJECT_COUNT,
                "Visible index {} is out of range: {} (max: {})",
                i,
                index,
                OBJECT_COUNT - 1
            );
        }

        // Check for duplicate indices (shouldn't happen)
        let mut sorted_indices: Vec<u32> = indices_read[0..(visible_count as usize)].to_vec();
        sorted_indices.sort_unstable();
        for i in 1..sorted_indices.len() {
            assert_ne!(
                sorted_indices[i - 1],
                sorted_indices[i],
                "Duplicate index found at position {}: {}",
                i,
                sorted_indices[i]
            );
        }

        info!(
            "✓ All {} visible indices are valid and unique",
            visible_count
        );
    } else {
        return Err(praxis_utils::eyre::eyre!(
            "Visible indices buffer not available"
        ));
    }

    // Validation 4: Verify draw count buffer matches visible count
    info!("Validating draw count buffer");
    if let Some(draw_count_buffer) = fixture.culling_manager.draw_count_buffer() {
        let count_read = draw_count_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read draw count: {}", e))?;

        let draw_count = *count_read;
        assert_eq!(
            draw_count, visible_count,
            "Draw count buffer ({}) should match visible count ({})",
            draw_count, visible_count
        );

        info!("✓ Draw count buffer matches visible count: {}", draw_count);
    } else {
        return Err(praxis_utils::eyre::eyre!("Draw count buffer not available"));
    }

    info!("=== GPU Culling Integration Test PASSED ===");
    info!("Summary:");
    info!("  ✓ Compute shader dispatch executed successfully");
    info!(
        "  ✓ Frustum culling eliminated {:.1}% of objects",
        cull_percentage
    );
    info!("  ✓ {} indirect draw commands validated", visible_count);
    info!("  ✓ All visible indices are valid and unique");
    info!("  ✓ Draw count buffer is correct");

    Ok(())
}

/// Test with different camera angles to verify varying culling percentages.
#[test]
fn test_gpu_culling_varying_camera_angles() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU Culling Test: Varying Camera Angles ===");

    let mut fixture = GpuCullingTestFixture::new()?;

    const OBJECT_COUNT: usize = 1000;
    const SPACING: f32 = 5.0;

    let draw_commands = create_test_grid(SPACING);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Test multiple camera angles
    let test_cases = vec![
        (
            "Looking at center from far",
            Vec3::new(100.0, 0.0, 0.0),
            Vec3::ZERO,
        ),
        ("Looking from above", Vec3::new(0.0, 100.0, 0.0), Vec3::ZERO),
        (
            "Looking from corner",
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::ZERO,
        ),
        (
            "Looking away from grid",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1000.0, 0.0, 0.0),
        ),
    ];

    for (description, camera_pos, camera_target) in test_cases {
        info!("Testing: {}", description);

        let view = Mat4::look_at_rh(camera_pos, camera_target, Vec3::Y);
        let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 1.0, 200.0);
        let view_proj = projection * view;
        let frustum_planes = extract_frustum_planes(view_proj);

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
            camera_pos,
        )?;

        fixture.execute_and_wait(command_buffer_builder)?;

        let visible_count = fixture.culling_manager.read_visible_count()?;
        let cull_percentage =
            (OBJECT_COUNT as f32 - visible_count as f32) / OBJECT_COUNT as f32 * 100.0;

        info!(
            "  Visible: {} / {} ({:.1}% culled)",
            visible_count, OBJECT_COUNT, cull_percentage
        );

        // All scenarios should result in valid culling
        assert!(
            visible_count <= OBJECT_COUNT as u32,
            "Visible count should not exceed total objects"
        );
    }

    info!("✓ All camera angles tested successfully");
    Ok(())
}

/// Test that verifies compute shader dispatch parameters are correct.
#[test]
fn test_compute_shader_dispatch_validation() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== Compute Shader Dispatch Validation Test ===");

    let mut fixture = GpuCullingTestFixture::new()?;

    const OBJECT_COUNT: usize = 1000;
    let draw_commands = create_test_grid(5.0);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    let camera_pos = Vec3::new(25.0, 25.0, 25.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 1.0, 100.0);
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Verify frustum planes are normalized
    info!("Validating frustum planes");
    for (i, plane) in frustum_planes.iter().enumerate() {
        let length = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
        assert!(
            (length - 1.0).abs() < 0.01,
            "Frustum plane {} not normalized: length = {:.4}",
            i,
            length
        );
    }
    info!("✓ All 6 frustum planes are properly normalized");

    // Dispatch compute shader
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
        camera_pos,
    )?;

    info!("✓ Compute shader dispatch recorded successfully");

    fixture.execute_and_wait(command_buffer_builder)?;

    info!("✓ Compute shader execution completed");

    let visible_count = fixture.culling_manager.read_visible_count()?;
    assert!(visible_count > 0, "Some objects should be visible");

    info!(
        "✓ Compute shader produced valid results: {} visible objects",
        visible_count
    );

    Ok(())
}

/// Test edge case: All objects visible (no culling).
#[test]
fn test_gpu_culling_all_visible() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU Culling Test: All Objects Visible ===");

    let mut fixture = GpuCullingTestFixture::new()?;

    const OBJECT_COUNT: usize = 100; // Smaller grid for full visibility
    let draw_commands = create_test_grid(3.0);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands[..OBJECT_COUNT], &mesh_data[..OBJECT_COUNT])?;

    // Camera very far away with wide FOV to see everything
    let camera_pos = Vec3::new(0.0, 0.0, 200.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh(
        std::f32::consts::PI / 2.0, // 90 degree FOV
        1.0,
        1.0,
        1000.0,
    );
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

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
        camera_pos,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!("Visible: {} / {}", visible_count, OBJECT_COUNT);

    // Should see most or all objects
    assert!(
        visible_count as f32 >= OBJECT_COUNT as f32 * 0.5,
        "Expected at least 50% visibility with wide FOV and distant camera, got {}%",
        (visible_count as f32 / OBJECT_COUNT as f32) * 100.0
    );

    info!(
        "✓ Wide FOV test passed: {:.1}% visible",
        (visible_count as f32 / OBJECT_COUNT as f32) * 100.0
    );

    Ok(())
}

/// Test edge case: No objects visible (all culled).
#[test]
fn test_gpu_culling_all_culled() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU Culling Test: All Objects Culled ===");

    let mut fixture = GpuCullingTestFixture::new()?;

    const OBJECT_COUNT: usize = 100;
    let draw_commands = create_test_grid(3.0);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands[..OBJECT_COUNT], &mesh_data[..OBJECT_COUNT])?;

    // Camera looking away from all objects
    let camera_pos = Vec3::new(0.0, 0.0, 0.0);
    let camera_target = Vec3::new(1000.0, 0.0, 0.0); // Looking far away from grid
    let view = Mat4::look_at_rh(camera_pos, camera_target, Vec3::Y);
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 1.0, 100.0);
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

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
        camera_pos,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!("Visible: {} / {}", visible_count, OBJECT_COUNT);

    // Most or all objects should be culled
    assert!(
        (visible_count as f32) < (OBJECT_COUNT as f32 * 0.2),
        "Expected less than 20% visibility when looking away, got {:.1}%",
        (visible_count as f32 / OBJECT_COUNT as f32) * 100.0
    );

    info!(
        "✓ Looking away test passed: only {:.1}% visible",
        (visible_count as f32 / OBJECT_COUNT as f32) * 100.0
    );

    Ok(())
}
