//! Integration test for GPU LOD selection system.
//!
//! This test validates the complete GPU LOD pipeline:
//! - Creates objects at varying distances from camera
//! - Dispatches LOD compute shader
//! - Validates selected LOD levels match distance thresholds
//! - Tests LOD bias effects on selection
//! - Verifies integration with indirect draw generation
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

use praxis_graphics::lod::{GpuLodLevel, GpuLodSelector, GpuObjectData, LodUniforms};
use praxis_math::{Mat4, Vec3};
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

/// Test fixture for GPU LOD tests with Vulkan resources.
struct GpuLodTestFixture {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    lod_selector: GpuLodSelector,
}

impl GpuLodTestFixture {
    /// Creates a new test fixture with Vulkan device and allocators.
    fn new() -> Result<Self> {
        info!("Initializing GPU LOD integration test fixture");

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

        // Create GPU LOD selector
        let lod_selector = GpuLodSelector::new(
            device.clone(),
            memory_allocator.clone(),
            descriptor_set_allocator.clone(),
        )?;

        info!("GPU LOD test fixture initialized successfully");

        Ok(Self {
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            lod_selector,
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

/// Creates objects at varying distances along the X-axis for LOD testing.
///
/// Objects are positioned at: 5, 15, 30, 45, 60 units from origin.
/// Camera is at origin looking along X-axis.
///
/// Expected LOD selection with thresholds (0-10, 10-25, 25+):
/// - Object at 5:  LOD 0 (high detail)
/// - Object at 15: LOD 1 (medium detail)
/// - Object at 30: LOD 2 (low detail)
/// - Object at 45: LOD 2 (low detail)
/// - Object at 60: LOD 2 (low detail)
fn create_distance_test_objects() -> Vec<GpuObjectData> {
    let distances = [5.0, 15.0, 30.0, 45.0, 60.0];
    let mut objects = Vec::new();

    for (i, &distance) in distances.iter().enumerate() {
        let position = Vec3::new(distance, 0.0, 0.0);
        let model = Mat4::from_translation(position);
        let bounding_sphere = [0.0, 0.0, 0.0, 1.0]; // Center at origin, radius 1.0

        objects.push(GpuObjectData::new(
            model,
            bounding_sphere,
            (i * 3) as u32, // Base mesh ID (high detail)
            3,              // 3 LOD levels per object
            (i * 3) as u32, // LOD offset in array
        ));
    }

    objects
}

/// Creates LOD level definitions for the test objects.
///
/// Each object has 3 LOD levels:
/// - LOD 0 (high):   0-10 units (0-100 squared)
/// - LOD 1 (medium): 10-25 units (100-625 squared)
/// - LOD 2 (low):    25+ units (625+ squared)
fn create_lod_level_definitions() -> Vec<GpuLodLevel> {
    let mut lod_levels = Vec::new();

    // Create LOD definitions for 5 objects (3 levels each = 15 total)
    for object_id in 0..5 {
        let base_mesh_id = object_id * 3;

        // LOD 0: High detail (0-10 units)
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id,
            min_distance_sq: 0.0,
            max_distance_sq: 100.0, // 10^2
            padding: 0,
        });

        // LOD 1: Medium detail (10-25 units)
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id + 1,
            min_distance_sq: 100.0,
            max_distance_sq: 625.0, // 25^2
            padding: 0,
        });

        // LOD 2: Low detail (25+ units)
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id + 2,
            min_distance_sq: 625.0,
            max_distance_sq: f32::MAX,
            padding: 0,
        });
    }

    lod_levels
}

/// Main integration test: GPU LOD selection with objects at varying distances.
///
/// This test validates:
/// 1. Compute shader executes successfully
/// 2. Selected LOD levels match expected values based on distance thresholds
/// 3. LOD selection is consistent and correct
#[test]
fn test_gpu_lod_selection_varying_distances() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU LOD Integration Test: Varying Distances ===");

    let mut fixture = GpuLodTestFixture::new()?;

    // Create test objects at different distances
    let objects = create_distance_test_objects();
    let lod_levels = create_lod_level_definitions();

    info!(
        "Created {} objects with {} LOD level definitions",
        objects.len(),
        lod_levels.len()
    );

    // Upload data to GPU
    info!("Uploading object data and LOD definitions to GPU");
    fixture.lod_selector.prepare_frame(&objects, &lod_levels)?;

    // Set up camera at origin looking along X-axis
    let camera_position = Vec3::ZERO;
    let lod_bias = 0.0;
    let enable_lod = true;

    info!("Camera position: {:?}", camera_position);
    info!("LOD bias: {}", lod_bias);

    // Create command buffer for compute dispatch
    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    // Dispatch GPU LOD selection compute shader
    info!("Dispatching GPU LOD selection compute shader");
    fixture.lod_selector.dispatch_lod_selection(
        &mut command_buffer_builder,
        camera_position,
        lod_bias,
        enable_lod,
    )?;

    // Execute and wait for completion
    info!("Executing command buffer and waiting for GPU");
    fixture.execute_and_wait(command_buffer_builder)?;

    // Read back selected LOD levels
    let selected_lods = fixture.lod_selector.read_selected_lods()?;
    let distances = fixture.lod_selector.read_distances()?;

    info!("LOD selection results:");
    for (i, (&selected, &distance_sq)) in selected_lods.iter().zip(distances.iter()).enumerate() {
        let distance = distance_sq.sqrt();
        info!(
            "  Object {}: distance={:.1}, distance_sq={:.1}, selected_mesh_id={}",
            i, distance, distance_sq, selected
        );
    }

    // Validation: Check each object's LOD selection matches expected value
    let expected_selections = [
        (0, 0),  // Object 0 at 5 units: LOD 0 (mesh_id 0)
        (1, 4),  // Object 1 at 15 units: LOD 1 (mesh_id 4)
        (2, 8),  // Object 2 at 30 units: LOD 2 (mesh_id 8)
        (3, 11), // Object 3 at 45 units: LOD 2 (mesh_id 11)
        (4, 14), // Object 4 at 60 units: LOD 2 (mesh_id 14)
    ];

    for (object_idx, expected_mesh_id) in expected_selections {
        let selected_mesh_id = selected_lods[object_idx];
        assert_eq!(
            selected_mesh_id, expected_mesh_id,
            "Object {} LOD mismatch: expected mesh_id {}, got {}",
            object_idx, expected_mesh_id, selected_mesh_id
        );
    }

    info!("✓ All LOD selections match expected values");

    // Validation: Check distance calculations are correct
    let expected_distances_sq = [25.0, 225.0, 900.0, 2025.0, 3600.0]; // 5^2, 15^2, 30^2, 45^2, 60^2

    for (i, &expected_dist_sq) in expected_distances_sq.iter().enumerate() {
        let actual_dist_sq = distances[i];
        let diff = (actual_dist_sq - expected_dist_sq).abs();
        assert!(
            diff < 0.01,
            "Object {} distance calculation error: expected {:.1}, got {:.1}",
            i,
            expected_dist_sq,
            actual_dist_sq
        );
    }

    info!("✓ All distance calculations are correct");

    info!("=== GPU LOD Integration Test PASSED ===");
    info!("Summary:");
    info!("  ✓ Compute shader dispatch executed successfully");
    info!("  ✓ All LOD selections match distance thresholds");
    info!("  ✓ Distance calculations are accurate");

    Ok(())
}

/// Test LOD bias effects on selection.
///
/// Tests that positive bias (prefer higher detail) and negative bias
/// (prefer lower detail) correctly affect LOD selection.
#[test]
fn test_gpu_lod_bias_effects() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU LOD Test: Bias Effects ===");

    let mut fixture = GpuLodTestFixture::new()?;

    // Create objects at distances where bias will matter
    // Object at 15 units is in medium range (10-25)
    // With positive bias, should select high detail
    // With negative bias, should select low detail
    let objects = vec![GpuObjectData::new(
        Mat4::from_translation(Vec3::new(15.0, 0.0, 0.0)),
        [0.0, 0.0, 0.0, 1.0],
        0, // Base mesh ID (high)
        3, // 3 LOD levels
        0, // LOD offset
    )];

    let lod_levels = vec![
        GpuLodLevel {
            mesh_id: 0,
            min_distance_sq: 0.0,
            max_distance_sq: 100.0,
            padding: 0,
        },
        GpuLodLevel {
            mesh_id: 1,
            min_distance_sq: 100.0,
            max_distance_sq: 625.0,
            padding: 0,
        },
        GpuLodLevel {
            mesh_id: 2,
            min_distance_sq: 625.0,
            max_distance_sq: f32::MAX,
            padding: 0,
        },
    ];

    fixture.lod_selector.prepare_frame(&objects, &lod_levels)?;

    let camera_position = Vec3::ZERO;

    // Test cases: (bias, expected_mesh_id, description)
    let test_cases = vec![
        (0.0, 1, "No bias"),
        (1.0, 0, "Max positive bias (prefer high detail)"),
        (-1.0, 1, "Max negative bias (prefer low detail)"),
        (0.5, 0, "Moderate positive bias"),
    ];

    for (lod_bias, expected_mesh_id, description) in test_cases {
        info!("Testing: {} (bias: {:.1})", description, lod_bias);

        let mut command_buffer_builder = RecordingCommandBuffer::new(
            fixture.command_buffer_allocator.clone(),
            fixture.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

        fixture.lod_selector.dispatch_lod_selection(
            &mut command_buffer_builder,
            camera_position,
            lod_bias,
            true,
        )?;

        fixture.execute_and_wait(command_buffer_builder)?;

        let selected_lods = fixture.lod_selector.read_selected_lods()?;
        let selected_mesh_id = selected_lods[0];

        info!(
            "  Expected mesh_id: {}, got: {}",
            expected_mesh_id, selected_mesh_id
        );

        assert_eq!(
            selected_mesh_id, expected_mesh_id,
            "{}: expected mesh_id {}, got {}",
            description, expected_mesh_id, selected_mesh_id
        );

        info!("  ✓ LOD bias test passed");
    }

    info!("✓ All LOD bias tests passed");
    Ok(())
}

/// Test LOD system enable/disable functionality.
///
/// Verifies that when LOD is disabled, all objects use their base mesh_id.
#[test]
fn test_gpu_lod_enable_disable() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU LOD Test: Enable/Disable ===");

    let mut fixture = GpuLodTestFixture::new()?;

    let objects = create_distance_test_objects();
    let lod_levels = create_lod_level_definitions();

    fixture.lod_selector.prepare_frame(&objects, &lod_levels)?;

    let camera_position = Vec3::ZERO;

    // Test with LOD disabled
    info!("Testing with LOD disabled");
    {
        let mut command_buffer_builder = RecordingCommandBuffer::new(
            fixture.command_buffer_allocator.clone(),
            fixture.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

        fixture.lod_selector.dispatch_lod_selection(
            &mut command_buffer_builder,
            camera_position,
            0.0,
            false, // LOD disabled
        )?;

        fixture.execute_and_wait(command_buffer_builder)?;

        let selected_lods = fixture.lod_selector.read_selected_lods()?;

        // All objects should use their base mesh_id (0, 3, 6, 9, 12)
        for (i, &selected) in selected_lods.iter().enumerate() {
            let expected_base_mesh = (i * 3) as u32;
            assert_eq!(
                selected, expected_base_mesh,
                "With LOD disabled, object {} should use base mesh_id {}, got {}",
                i, expected_base_mesh, selected
            );
        }

        info!("✓ All objects use base mesh when LOD disabled");
    }

    // Test with LOD enabled (should use distance-based selection)
    info!("Testing with LOD enabled");
    {
        let mut command_buffer_builder = RecordingCommandBuffer::new(
            fixture.command_buffer_allocator.clone(),
            fixture.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

        fixture.lod_selector.dispatch_lod_selection(
            &mut command_buffer_builder,
            camera_position,
            0.0,
            true, // LOD enabled
        )?;

        fixture.execute_and_wait(command_buffer_builder)?;

        let selected_lods = fixture.lod_selector.read_selected_lods()?;

        // Should not all be base mesh IDs
        let all_base_mesh = selected_lods
            .iter()
            .enumerate()
            .all(|(i, &mesh_id)| mesh_id == (i * 3) as u32);

        assert!(
            !all_base_mesh,
            "With LOD enabled, not all objects should use base mesh"
        );

        info!("✓ Distance-based LOD selection active when enabled");
    }

    info!("✓ Enable/disable functionality works correctly");
    Ok(())
}

/// Test integration with indirect draw generation (buffer validation).
///
/// Verifies that selected LOD buffer can be used for indirect draw generation.
#[test]
fn test_gpu_lod_indirect_draw_integration() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU LOD Test: Indirect Draw Integration ===");

    let mut fixture = GpuLodTestFixture::new()?;

    const OBJECT_COUNT: usize = 100;
    let mut objects = Vec::with_capacity(OBJECT_COUNT);
    let mut lod_levels = Vec::with_capacity(OBJECT_COUNT * 3);

    // Create 100 objects at various distances
    for i in 0..OBJECT_COUNT {
        let distance = 5.0 + (i as f32 * 2.0); // 5, 7, 9, 11, ...
        let position = Vec3::new(distance, 0.0, 0.0);
        let model = Mat4::from_translation(position);

        objects.push(GpuObjectData::new(
            model,
            [0.0, 0.0, 0.0, 1.0],
            (i * 3) as u32,
            3,
            (i * 3) as u32,
        ));

        // Add LOD levels for this object
        let base_mesh_id = (i * 3) as u32;
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id,
            min_distance_sq: 0.0,
            max_distance_sq: 100.0,
            padding: 0,
        });
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id + 1,
            min_distance_sq: 100.0,
            max_distance_sq: 625.0,
            padding: 0,
        });
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id + 2,
            min_distance_sq: 625.0,
            max_distance_sq: f32::MAX,
            padding: 0,
        });
    }

    info!(
        "Created {} objects with {} LOD definitions",
        objects.len(),
        lod_levels.len()
    );

    fixture.lod_selector.prepare_frame(&objects, &lod_levels)?;

    let camera_position = Vec3::ZERO;

    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    fixture.lod_selector.dispatch_lod_selection(
        &mut command_buffer_builder,
        camera_position,
        0.0,
        true,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    // Verify selected LOD buffer is available and valid
    let selected_lod_buffer = fixture
        .lod_selector
        .selected_lod_buffer()
        .ok_or_else(|| praxis_utils::eyre::eyre!("Selected LOD buffer not available"))?;

    info!("✓ Selected LOD buffer is available");

    let selected_lods = fixture.lod_selector.read_selected_lods()?;
    assert_eq!(
        selected_lods.len(),
        OBJECT_COUNT,
        "Selected LODs count mismatch"
    );

    // Verify all selected mesh IDs are valid (within expected range)
    for (i, &mesh_id) in selected_lods.iter().enumerate() {
        let base_mesh_id = (i * 3) as u32;
        let max_mesh_id = base_mesh_id + 2;

        assert!(
            mesh_id >= base_mesh_id && mesh_id <= max_mesh_id,
            "Object {} has invalid mesh_id {}, expected range [{}, {}]",
            i,
            mesh_id,
            base_mesh_id,
            max_mesh_id
        );
    }

    info!("✓ All {} selected mesh IDs are valid", OBJECT_COUNT);

    // Verify distance buffer is available
    let distance_buffer = fixture
        .lod_selector
        .distance_buffer()
        .ok_or_else(|| praxis_utils::eyre::eyre!("Distance buffer not available"))?;

    info!("✓ Distance buffer is available");

    let distances = fixture.lod_selector.read_distances()?;
    assert_eq!(distances.len(), OBJECT_COUNT, "Distances count mismatch");

    // Verify distances are monotonically increasing (objects placed at increasing distances)
    for i in 1..distances.len() {
        assert!(
            distances[i] >= distances[i - 1],
            "Distance at index {} ({:.1}) should be >= distance at index {} ({:.1})",
            i,
            distances[i],
            i - 1,
            distances[i - 1]
        );
    }

    info!("✓ Distance buffer contains valid sorted distances");

    // Count LOD distribution
    let mut lod_counts = [0u32; 3];
    for (i, &mesh_id) in selected_lods.iter().enumerate() {
        let base_mesh_id = (i * 3) as u32;
        let lod_level = (mesh_id - base_mesh_id) as usize;
        if lod_level < 3 {
            lod_counts[lod_level] += 1;
        }
    }

    info!("LOD distribution:");
    info!("  LOD 0 (high):   {} objects", lod_counts[0]);
    info!("  LOD 1 (medium): {} objects", lod_counts[1]);
    info!("  LOD 2 (low):    {} objects", lod_counts[2]);

    // Verify we have objects at multiple LOD levels (not all at same level)
    let non_zero_lods = lod_counts.iter().filter(|&&c| c > 0).count();
    assert!(
        non_zero_lods >= 2,
        "Expected objects at multiple LOD levels, got {} levels",
        non_zero_lods
    );

    info!("✓ Objects distributed across {} LOD levels", non_zero_lods);

    info!("=== Indirect Draw Integration Test PASSED ===");
    info!("Summary:");
    info!("  ✓ Selected LOD buffer available for indirect draws");
    info!(
        "  ✓ All {} mesh IDs valid and in expected range",
        OBJECT_COUNT
    );
    info!("  ✓ Distance buffer contains valid sorted distances");
    info!("  ✓ Objects distributed across multiple LOD levels");

    Ok(())
}

/// Test LOD selection at boundary conditions.
///
/// Verifies correct behavior at exact distance thresholds.
#[test]
fn test_gpu_lod_boundary_conditions() -> Result<()> {
    praxis_utils::init().ok();

    info!("=== GPU LOD Test: Boundary Conditions ===");

    let mut fixture = GpuLodTestFixture::new()?;

    // Create objects exactly at LOD boundaries
    // Boundaries: 10.0 (100 squared), 25.0 (625 squared)
    let boundary_distances = [10.0, 25.0];
    let mut objects = Vec::new();
    let mut lod_levels = Vec::new();

    for (i, &distance) in boundary_distances.iter().enumerate() {
        let position = Vec3::new(distance, 0.0, 0.0);
        let model = Mat4::from_translation(position);

        objects.push(GpuObjectData::new(
            model,
            [0.0, 0.0, 0.0, 1.0],
            (i * 3) as u32,
            3,
            (i * 3) as u32,
        ));

        let base_mesh_id = (i * 3) as u32;
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id,
            min_distance_sq: 0.0,
            max_distance_sq: 100.0,
            padding: 0,
        });
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id + 1,
            min_distance_sq: 100.0,
            max_distance_sq: 625.0,
            padding: 0,
        });
        lod_levels.push(GpuLodLevel {
            mesh_id: base_mesh_id + 2,
            min_distance_sq: 625.0,
            max_distance_sq: f32::MAX,
            padding: 0,
        });
    }

    fixture.lod_selector.prepare_frame(&objects, &lod_levels)?;

    let camera_position = Vec3::ZERO;

    let mut command_buffer_builder = RecordingCommandBuffer::new(
        fixture.command_buffer_allocator.clone(),
        fixture.queue.queue_family_index(),
        vulkano::command_buffer::CommandBufferLevel::Primary,
        CommandBufferUsage::OneTimeSubmit,
    )
    .map_err(|e| praxis_utils::eyre::eyre!("Failed to create command buffer: {}", e))?;

    fixture.lod_selector.dispatch_lod_selection(
        &mut command_buffer_builder,
        camera_position,
        0.0,
        true,
    )?;

    fixture.execute_and_wait(command_buffer_builder)?;

    let selected_lods = fixture.lod_selector.read_selected_lods()?;

    info!("Boundary test results:");
    for (i, &mesh_id) in selected_lods.iter().enumerate() {
        let distance = boundary_distances[i];
        info!("  Object at {:.1} units: mesh_id {}", distance, mesh_id);
    }

    // At boundary 10.0 (100 squared), should select LOD 1 (mesh_id 1)
    // because condition is: distance_sq >= min_distance_sq && distance_sq < max_distance_sq
    // At exactly 100, it's >= 100 and < max, so LOD 1
    assert_eq!(
        selected_lods[0], 1,
        "Object at 10.0 boundary should select LOD 1 (mesh_id 1)"
    );

    // At boundary 25.0 (625 squared), should select LOD 2 (mesh_id 5)
    assert_eq!(
        selected_lods[1], 5,
        "Object at 25.0 boundary should select LOD 2 (mesh_id 5)"
    );

    info!("✓ Boundary conditions handled correctly");
    Ok(())
}
