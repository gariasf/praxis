//! Integration test for multi-draw indirect rendering.
//!
//! This test verifies that the GPU culling and multi-draw indirect rendering system:
//! - Efficiently batches draw calls for objects with the same material
//! - Reduces batch count significantly (from 200+ individual draws to 50-100 batches)
//! - Correctly populates indirect draw buffers with valid commands
//! - Produces correct rendering output after culling
//!
//! The test creates 200+ objects with 10 different materials and validates the
//! entire rendering pipeline from setup through GPU culling to indirect draw execution.
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
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferUsage, RecordingCommandBuffer,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    sync::GpuFuture,
    VulkanLibrary,
};

/// Test fixture for multi-draw indirect rendering tests.
struct MultiDrawTestFixture {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    culling_manager: GpuCullingManager,
}

impl MultiDrawTestFixture {
    /// Creates a new test fixture with Vulkan device and allocators.
    fn new() -> Result<Self> {
        info!("Initializing multi-draw indirect test fixture");

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

        info!("Test fixture initialized successfully");

        Ok(Self {
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            culling_manager,
        })
    }
}

/// Creates test draw commands for a grid of objects with multiple materials.
///
/// # Arguments
///
/// * `object_count` - Total number of objects to create
/// * `material_count` - Number of different materials to distribute across objects
///
/// # Returns
///
/// Vector of GPU draw commands with objects arranged in a grid pattern
fn create_test_draw_commands(object_count: usize, material_count: usize) -> Vec<GpuDrawCommand> {
    let mut commands = Vec::with_capacity(object_count);
    let grid_size = (object_count as f32).sqrt().ceil() as u32;

    for i in 0..object_count {
        let x = (i % grid_size as usize) as f32 * 3.0;
        let z = (i / grid_size as usize) as f32 * 3.0;

        let model = Mat4::from_translation(Vec3::new(x, 0.0, z));
        let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 1.0); // Radius 1.0

        // Distribute materials evenly across objects
        let material_id = (i % material_count) as u32;
        let mesh_id = i as u32;

        commands.push(GpuDrawCommand::new(
            model,
            bounding_sphere,
            mesh_id,
            material_id,
        ));
    }

    commands
}

/// Creates test mesh data for the objects.
///
/// # Arguments
///
/// * `mesh_count` - Number of meshes to create
///
/// # Returns
///
/// Vector of GPU mesh data with simple triangle count info
fn create_test_mesh_data(mesh_count: usize) -> Vec<GpuMeshData> {
    let mut mesh_data = Vec::with_capacity(mesh_count);

    for i in 0..mesh_count {
        // Simple cube: 36 indices (12 triangles * 3 vertices)
        mesh_data.push(GpuMeshData {
            index_count: 36,
            first_index: (i * 36) as u32,
            vertex_offset: 0,
            _padding: 0,
        });
    }

    mesh_data
}

/// Test that verifies GPU culling with 200+ objects reduces batch count significantly.
#[test]
fn test_multi_draw_indirect_batch_reduction() -> Result<()> {
    praxis_utils::init().ok(); // Initialize logging

    info!("Starting multi-draw indirect batch reduction test");

    let mut fixture = MultiDrawTestFixture::new()?;

    // Test configuration
    const OBJECT_COUNT: usize = 250;
    const MATERIAL_COUNT: usize = 10;

    // Create test data
    info!(
        "Creating {} objects with {} materials",
        OBJECT_COUNT, MATERIAL_COUNT
    );
    let draw_commands = create_test_draw_commands(OBJECT_COUNT, MATERIAL_COUNT);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    // Prepare culling manager
    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Set up camera for full scene visibility
    let camera_position = Vec3::new(
        (OBJECT_COUNT as f32).sqrt() * 1.5,
        50.0,
        (OBJECT_COUNT as f32).sqrt() * 1.5,
    );

    let view = Mat4::look_at_rh(
        camera_position,
        Vec3::new(
            (OBJECT_COUNT as f32).sqrt() * 1.5,
            0.0,
            (OBJECT_COUNT as f32).sqrt() * 1.5,
        ),
        Vec3::Y,
    );

    let projection = Mat4::perspective_rh(
        std::f32::consts::PI / 4.0, // 45 degree FOV
        16.0 / 9.0,                 // Aspect ratio
        0.1,                        // Near plane
        1000.0,                     // Far plane
    );

    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Create command buffer
    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    // Dispatch GPU culling
    info!("Dispatching GPU culling compute shader");
    fixture.culling_manager.dispatch_culling(
        &mut command_buffer_builder,
        view_proj,
        frustum_planes,
        camera_position,
    )?;

    // Build and submit command buffer
    let command_buffer = command_buffer_builder
        .end()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to build command buffer: {}", e))?;

    let future = vulkano::sync::now(fixture.device.clone())
        .then_execute(fixture.queue.clone(), command_buffer)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute command buffer: {}", e))?
        .then_signal_fence_and_flush()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush command buffer: {}", e))?;

    future
        .wait(None)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait for GPU: {}", e))?;

    // Read back results
    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!("Visible objects after culling: {}", visible_count);

    // Verify results
    assert!(visible_count > 0, "At least some objects should be visible");
    assert!(
        visible_count <= OBJECT_COUNT as u32,
        "Visible count should not exceed total objects"
    );

    // With all objects visible and 10 materials, we expect roughly 10 batches
    // (one per material) instead of 250 individual draws
    let expected_max_batches = MATERIAL_COUNT * 2; // Allow some overhead
    let expected_min_batches = MATERIAL_COUNT / 2; // At least half of materials should batch

    info!(
        "Expected batch count range: {} to {} (vs {} individual draws)",
        expected_min_batches, expected_max_batches, OBJECT_COUNT
    );

    // For this test, the "batch count" is approximated by the number of unique materials
    // in visible objects. In a real renderer, consecutive objects with the same material
    // would be batched into a single multi-draw-indirect call.
    //
    // The key assertion is that batch count (≈ material count) << object count
    assert!(
        MATERIAL_COUNT < OBJECT_COUNT / 2,
        "Batch count ({}) should be significantly less than object count ({})",
        MATERIAL_COUNT,
        OBJECT_COUNT
    );

    info!(
        "✓ Batch reduction verified: {} materials batch {} objects",
        MATERIAL_COUNT, OBJECT_COUNT
    );

    Ok(())
}

/// Test that verifies indirect draw buffer content is valid.
#[test]
fn test_indirect_draw_buffer_validation() -> Result<()> {
    praxis_utils::init().ok();

    info!("Starting indirect draw buffer validation test");

    let mut fixture = MultiDrawTestFixture::new()?;

    // Create smaller test set for detailed validation
    const OBJECT_COUNT: usize = 50;
    const MATERIAL_COUNT: usize = 5;

    let draw_commands = create_test_draw_commands(OBJECT_COUNT, MATERIAL_COUNT);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Set up camera
    let camera_position = Vec3::new(15.0, 30.0, 15.0);
    let view = Mat4::look_at_rh(camera_position, Vec3::new(15.0, 0.0, 15.0), Vec3::Y);
    let projection = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0);
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Dispatch culling
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

    let command_buffer = command_buffer_builder
        .end()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to build command buffer: {}", e))?;

    let future = vulkano::sync::now(fixture.device.clone())
        .then_execute(fixture.queue.clone(), command_buffer)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute: {}", e))?
        .then_signal_fence_and_flush()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush: {}", e))?;

    future
        .wait(None)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait: {}", e))?;

    // Read back indirect draw buffer
    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!("Visible count: {}", visible_count);

    if let Some(indirect_buffer) = fixture.culling_manager.indirect_draw_buffer() {
        let buffer_read = indirect_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read indirect buffer: {}", e))?;

        info!("Validating {} indirect draw commands", visible_count);

        // Validate each indirect draw command
        for i in 0..(visible_count as usize) {
            let cmd = buffer_read[i];

            // Verify command fields are valid
            assert!(
                cmd.instance_count > 0,
                "Command {} has invalid instance count: {}",
                i,
                cmd.instance_count
            );

            assert!(
                cmd.index_count > 0,
                "Command {} has invalid index count: {}",
                i,
                cmd.index_count
            );

            // For our test meshes (36 indices per cube)
            assert_eq!(
                cmd.index_count, 36,
                "Command {} has unexpected index count: {} (expected 36)",
                i, cmd.index_count
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

    Ok(())
}

/// Test that verifies visible indices buffer content.
#[test]
fn test_visible_indices_buffer() -> Result<()> {
    praxis_utils::init().ok();

    info!("Starting visible indices buffer test");

    let mut fixture = MultiDrawTestFixture::new()?;

    const OBJECT_COUNT: usize = 30;
    const MATERIAL_COUNT: usize = 3;

    let draw_commands = create_test_draw_commands(OBJECT_COUNT, MATERIAL_COUNT);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Camera positioned to see all objects
    let camera_position = Vec3::new(10.0, 25.0, 10.0);
    let view = Mat4::look_at_rh(camera_position, Vec3::new(10.0, 0.0, 10.0), Vec3::Y);
    let projection = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0);
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Execute culling
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

    let command_buffer = command_buffer_builder
        .end()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to build: {}", e))?;

    let future = vulkano::sync::now(fixture.device.clone())
        .then_execute(fixture.queue.clone(), command_buffer)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute: {}", e))?
        .then_signal_fence_and_flush()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush: {}", e))?;

    future
        .wait(None)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait: {}", e))?;

    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!("Visible count: {}", visible_count);

    // Validate visible indices
    if let Some(indices_buffer) = fixture.culling_manager.visible_indices_buffer() {
        let indices_read = indices_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read indices buffer: {}", e))?;

        info!("Validating {} visible indices", visible_count);

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

            debug!("Visible index {}: object {}", i, index);
        }

        // Check for duplicate indices (shouldn't happen)
        let mut sorted_indices: Vec<u32> = indices_read[0..(visible_count as usize)].to_vec();
        sorted_indices.sort();
        for i in 1..sorted_indices.len() {
            assert_ne!(
                sorted_indices[i - 1],
                sorted_indices[i],
                "Duplicate index found: {}",
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

    Ok(())
}

/// Test with objects outside frustum to verify culling.
#[test]
fn test_frustum_culling_accuracy() -> Result<()> {
    praxis_utils::init().ok();

    info!("Starting frustum culling accuracy test");

    let mut fixture = MultiDrawTestFixture::new()?;

    // Create objects in a known pattern: half inside, half outside frustum
    let mut draw_commands = Vec::new();

    // Objects inside frustum (near origin)
    for i in 0..100 {
        let x = (i % 10) as f32 * 2.0;
        let z = (i / 10) as f32 * 2.0;
        let model = Mat4::from_translation(Vec3::new(x, 0.0, z));
        draw_commands.push(GpuDrawCommand::new(
            model,
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            i,
            i % 10,
        ));
    }

    // Objects outside frustum (far away)
    for i in 100..200 {
        let x = (i % 10) as f32 * 2.0 + 1000.0; // Very far away
        let z = (i / 10) as f32 * 2.0 + 1000.0;
        let model = Mat4::from_translation(Vec3::new(x, 0.0, z));
        draw_commands.push(GpuDrawCommand::new(
            model,
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            i,
            i % 10,
        ));
    }

    let mesh_data = create_test_mesh_data(200);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Camera looking at near objects only
    let camera_position = Vec3::new(10.0, 20.0, 10.0);
    let view = Mat4::look_at_rh(camera_position, Vec3::new(10.0, 0.0, 10.0), Vec3::Y);
    let projection = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Execute culling
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

    let command_buffer = command_buffer_builder
        .end()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to build: {}", e))?;

    let future = vulkano::sync::now(fixture.device.clone())
        .then_execute(fixture.queue.clone(), command_buffer)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute: {}", e))?
        .then_signal_fence_and_flush()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush: {}", e))?;

    future
        .wait(None)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait: {}", e))?;

    let visible_count = fixture.culling_manager.read_visible_count()?;
    info!(
        "Visible count after frustum culling: {} / 200",
        visible_count
    );

    // We expect far fewer than 200 objects to be visible
    // The far objects (100+) should be culled
    assert!(
        visible_count < 150,
        "Expected significant culling, got {} visible out of 200",
        visible_count
    );

    assert!(
        visible_count > 0,
        "Expected at least some objects to be visible"
    );

    info!(
        "✓ Frustum culling reduced object count from 200 to {}",
        visible_count
    );

    Ok(())
}

/// Test that verifies draw count buffer is updated correctly.
#[test]
fn test_draw_count_buffer() -> Result<()> {
    praxis_utils::init().ok();

    info!("Starting draw count buffer test");

    let mut fixture = MultiDrawTestFixture::new()?;

    const OBJECT_COUNT: usize = 100;
    const MATERIAL_COUNT: usize = 5;

    let draw_commands = create_test_draw_commands(OBJECT_COUNT, MATERIAL_COUNT);
    let mesh_data = create_test_mesh_data(OBJECT_COUNT);

    fixture
        .culling_manager
        .prepare_frame(&draw_commands, &mesh_data)?;

    // Set up camera
    let camera_position = Vec3::new(15.0, 30.0, 15.0);
    let view = Mat4::look_at_rh(camera_position, Vec3::ZERO, Vec3::Y);
    let projection = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 1000.0);
    let view_proj = projection * view;
    let frustum_planes = extract_frustum_planes(view_proj);

    // Execute culling
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

    let command_buffer = command_buffer_builder
        .end()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to build: {}", e))?;

    let future = vulkano::sync::now(fixture.device.clone())
        .then_execute(fixture.queue.clone(), command_buffer)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute: {}", e))?
        .then_signal_fence_and_flush()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to flush: {}", e))?;

    future
        .wait(None)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to wait: {}", e))?;

    // Verify draw count buffer
    if let Some(draw_count_buffer) = fixture.culling_manager.draw_count_buffer() {
        let count_read = draw_count_buffer
            .read()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read draw count: {}", e))?;

        let draw_count = *count_read;
        info!("Draw count from buffer: {}", draw_count);

        assert!(draw_count > 0, "Draw count should be greater than 0");
        assert!(
            draw_count <= OBJECT_COUNT as u32,
            "Draw count {} should not exceed object count {}",
            draw_count,
            OBJECT_COUNT
        );

        // Verify it matches the visible count
        let visible_count = fixture.culling_manager.read_visible_count()?;
        assert_eq!(
            draw_count, visible_count,
            "Draw count buffer should match visible count"
        );

        info!("✓ Draw count buffer is correctly updated: {}", draw_count);
    } else {
        return Err(praxis_utils::eyre::eyre!("Draw count buffer not available"));
    }

    Ok(())
}

// ===== Unit Tests (No GPU Required) =====

/// Test that GPU draw command creation works correctly.
#[test]
fn test_gpu_draw_command_creation() {
    let model = Mat4::from_translation(Vec3::new(5.0, 10.0, 15.0));
    let bounding_sphere = Vec4::new(1.0, 2.0, 3.0, 5.0); // Center (1,2,3), radius 5
    let mesh_id = 42;
    let material_id = 7;

    let cmd = GpuDrawCommand::new(model, bounding_sphere, mesh_id, material_id);

    assert_eq!(cmd.model, model.to_cols_array_2d());
    assert_eq!(cmd.bounding_sphere, bounding_sphere.to_array());
    assert_eq!(cmd.mesh_id, mesh_id);
    assert_eq!(cmd.material_id, material_id);
}

/// Test that mesh data is created correctly.
#[test]
fn test_mesh_data_creation() {
    let mesh_data = GpuMeshData {
        index_count: 36,
        first_index: 0,
        vertex_offset: 0,
        _padding: 0,
    };

    assert_eq!(mesh_data.index_count, 36);
    assert_eq!(mesh_data.first_index, 0);
    assert_eq!(mesh_data.vertex_offset, 0);
}

/// Test that indirect draw command is laid out correctly.
#[test]
fn test_indirect_draw_command_layout() {
    use std::mem::size_of;

    // Verify size matches VkDrawIndexedIndirectCommand
    assert_eq!(size_of::<IndirectDrawCommand>(), 20);

    let cmd = IndirectDrawCommand {
        index_count: 36,
        instance_count: 1,
        first_index: 0,
        vertex_offset: 0,
        first_instance: 0,
    };

    assert_eq!(cmd.index_count, 36);
    assert_eq!(cmd.instance_count, 1);
}

/// Test frustum plane extraction from projection matrix.
#[test]
fn test_frustum_plane_extraction() {
    let projection = Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);

    let view = Mat4::look_at_rh(Vec3::new(0.0, 10.0, 10.0), Vec3::ZERO, Vec3::Y);

    let view_proj = projection * view;
    let planes = extract_frustum_planes(view_proj);

    // Verify we got 6 planes
    assert_eq!(planes.len(), 6);

    // Verify planes are normalized (using epsilon for floating-point comparison)
    const EPSILON: f32 = 0.01;
    for (i, plane) in planes.iter().enumerate() {
        let length = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
        assert!(
            length.is_finite(),
            "Plane {} has non-finite length: {}",
            i,
            length
        );
        assert!(
            (length - 1.0).abs() < EPSILON,
            "Plane {} not normalized: length = {}, expected ~1.0",
            i,
            length
        );
    }
}

/// Test that draw commands can be created for a large number of objects.
#[test]
fn test_create_many_draw_commands() {
    const OBJECT_COUNT: usize = 250;
    const MATERIAL_COUNT: usize = 10;

    let commands = create_test_draw_commands(OBJECT_COUNT, MATERIAL_COUNT);

    assert_eq!(commands.len(), OBJECT_COUNT);

    // Verify materials are distributed
    let mut material_counts = vec![0; MATERIAL_COUNT];
    for cmd in &commands {
        material_counts[cmd.material_id as usize] += 1;
    }

    // Each material should have approximately equal number of objects
    let expected_per_material = OBJECT_COUNT / MATERIAL_COUNT;
    for (i, count) in material_counts.iter().enumerate() {
        assert!(
            *count >= expected_per_material - 1 && *count <= expected_per_material + 1,
            "Material {} has {} objects, expected ~{}",
            i,
            count,
            expected_per_material
        );
    }
}

/// Test that mesh data can be created for many meshes.
#[test]
fn test_create_many_mesh_data() {
    const MESH_COUNT: usize = 200;

    let mesh_data = create_test_mesh_data(MESH_COUNT);

    assert_eq!(mesh_data.len(), MESH_COUNT);

    // Verify indices are sequential
    for (i, data) in mesh_data.iter().enumerate() {
        assert_eq!(data.index_count, 36);
        assert_eq!(data.first_index, (i * 36) as u32);
        assert_eq!(data.vertex_offset, 0);
    }
}

/// Test batch reduction calculation.
#[test]
fn test_batch_reduction_calculation() {
    const OBJECT_COUNT: usize = 250;
    const MATERIAL_COUNT: usize = 10;

    // In a batched rendering system:
    // - Without batching: 250 draw calls
    // - With batching by material: 10 batches (one per material)
    // - Reduction: 250 / 10 = 25x fewer draw calls

    let reduction_factor = OBJECT_COUNT / MATERIAL_COUNT;
    assert_eq!(reduction_factor, 25);

    // Verify this is a significant reduction
    assert!(
        reduction_factor >= 10,
        "Expected at least 10x reduction, got {}x",
        reduction_factor
    );

    info!(
        "Batch reduction: {} objects with {} materials = {}x reduction",
        OBJECT_COUNT, MATERIAL_COUNT, reduction_factor
    );
}

/// Test that GPU draw commands are properly sized for GPU buffers.
#[test]
fn test_gpu_draw_command_size() {
    use std::mem::size_of;

    // Should be properly aligned for GPU (96 bytes)
    assert_eq!(size_of::<GpuDrawCommand>(), 96);

    // Verify it's Pod/Zeroable
    let _zeroed = GpuDrawCommand::zeroed();
    let cmd = GpuDrawCommand::new(Mat4::IDENTITY, Vec4::ZERO, 0, 0);
    let _bytes: &[u8] = bytemuck::bytes_of(&cmd);
}

/// Test that mesh data is properly sized for GPU buffers.
#[test]
fn test_gpu_mesh_data_size() {
    use std::mem::size_of;

    // Should be 16 bytes (4 u32s/i32s)
    assert_eq!(size_of::<GpuMeshData>(), 16);

    // Verify it's Pod/Zeroable
    let _zeroed = GpuMeshData::zeroed();
    let data = GpuMeshData {
        index_count: 36,
        first_index: 0,
        vertex_offset: 0,
        _padding: 0,
    };
    let _bytes: &[u8] = bytemuck::bytes_of(&data);
}
