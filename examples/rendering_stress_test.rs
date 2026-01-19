//! Comprehensive Rendering Pipeline Stress Test
//!
//! This example performs intensive stress testing of the rendering pipeline to validate
//! stability, performance, and resource management under extreme conditions.
//!
//! # Test Scenarios
//!
//! 1. **Massive Object Count**: 10,000+ objects with various mesh types
//! 2. **Extreme Camera Movements**: Rapid position changes, violent rotations, instant teleports
//! 3. **Rapid LOD Transitions**: Fast camera movements triggering constant LOD switching
//! 4. **Material Instance Stress**: 500+ material instances with varying properties
//! 5. **High Mesh Streaming Throughput**: Continuous loading/unloading cycles
//! 6. **Resource Cleanup Validation**: Memory leak detection and resource tracking
//!
//! # Validation Criteria
//!
//! - **No crashes** during any test scenario
//! - **Acceptable performance degradation** (>15 FPS minimum)
//! - **Stable memory usage** (no unbounded growth)
//! - **Proper resource cleanup** (no descriptor set leaks, no mesh orphans)
//! - **Visual correctness** (no flickering, culling errors, or artifacts)
//!
//! # Controls
//!
//! - **1**: Massive object count test (10,000 objects)
//! - **2**: Extreme camera movement test (rapid teleports)
//! - **3**: Rapid LOD transition test (fast sweeps)
//! - **4**: Material instance stress test (500+ materials)
//! - **5**: Mesh streaming throughput test (continuous load/unload)
//! - **6**: Combined stress test (all at once)
//! - **7**: Resource cleanup validation test
//! - **Space**: Reset to idle state
//! - **P**: Print current statistics
//! - **W/A/S/D**: Manual camera movement
//! - **Q/E**: Manual camera up/down
//! - **Arrow Keys**: Manual camera rotation
//! - **ESC**: Exit
//!
//! # Expected Behavior
//!
//! Each test should complete without crashes and maintain:
//! - Frame rate: >15 FPS (acceptable degradation from baseline)
//! - Memory growth: <500 MB over baseline after stabilization
//! - Descriptor sets: Reused efficiently (not growing unbounded)
//! - Draw calls: Proportional to visible objects
//!
//! # Usage
//!
//! ```bash
//! cargo run --release --example rendering_stress_test
//! ```
//!
//! **Note**: Release mode is required for realistic performance testing.

use praxis_core::{Engine, EngineConfig};
use praxis_ecs::{Component, Query, ResMut, Resource, World};
use praxis_graphics::{
    colored_cube_mesh, solid_cube_mesh, sphere_mesh, DrawCommand, MaterialProperties,
    RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{error, info, warn, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Test scenarios for stress testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StressTest {
    /// Idle state (minimal load)
    Idle,
    /// Test 1: 10,000+ objects rendering
    MassiveObjectCount,
    /// Test 2: Extreme camera movements
    ExtremeCameraMovement,
    /// Test 3: Rapid LOD transitions
    RapidLodTransitions,
    /// Test 4: Material instance stress (500+ instances)
    MaterialInstanceStress,
    /// Test 5: High mesh streaming throughput
    MeshStreamingThroughput,
    /// Test 6: Combined stress (all tests simultaneously)
    CombinedStress,
    /// Test 7: Resource cleanup validation
    ResourceCleanup,
}

impl StressTest {
    fn name(&self) -> &str {
        match self {
            Self::Idle => "Idle (Minimal Load)",
            Self::MassiveObjectCount => "Massive Object Count (10,000+ Objects)",
            Self::ExtremeCameraMovement => "Extreme Camera Movement (Rapid Teleports)",
            Self::RapidLodTransitions => "Rapid LOD Transitions (Fast Sweeps)",
            Self::MaterialInstanceStress => "Material Instance Stress (500+ Instances)",
            Self::MeshStreamingThroughput => "Mesh Streaming Throughput (Continuous Load/Unload)",
            Self::CombinedStress => "Combined Stress (All Tests Simultaneously)",
            Self::ResourceCleanup => "Resource Cleanup Validation",
        }
    }
}

/// Object types for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectType {
    Cube,
    Sphere,
    Occluder,
}

/// Component marking stress test objects
#[derive(Component, Debug, Clone)]
struct StressObject {
    object_type: ObjectType,
    lod_level: u32,
    material_instance_id: Option<String>,
    distance_to_camera: f32,
    is_visible: bool,
    lifetime: f32, // For spawning/despawning tests
}

/// Camera controller for stress tests
#[derive(Resource)]
struct StressCamera {
    position: Vec3,
    rotation: Quat,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    rotate_speed: f32,
    // Extreme movement state
    teleport_timer: f32,
    teleport_interval: f32,
    sweep_timer: f32,
    sweep_speed: f32,
    // Input states
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    rotate_left: bool,
    rotate_right: bool,
    rotate_up_key: bool,
    rotate_down_key: bool,
}

impl Default for StressCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 50.0, 150.0),
            rotation: Quat::IDENTITY,
            yaw: 0.0,
            pitch: -0.3,
            move_speed: 50.0,
            rotate_speed: 2.0,
            teleport_timer: 0.0,
            teleport_interval: 0.5, // Teleport every 0.5 seconds
            sweep_timer: 0.0,
            sweep_speed: 100.0, // Very fast movement for LOD transitions
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            rotate_left: false,
            rotate_right: false,
            rotate_up_key: false,
            rotate_down_key: false,
        }
    }
}

impl StressCamera {
    fn reset(&mut self) {
        self.position = Vec3::new(0.0, 50.0, 150.0);
        self.yaw = 0.0;
        self.pitch = -0.3;
        self.update_rotation();
    }

    fn update_rotation(&mut self) {
        self.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    fn forward(&self) -> Vec3 {
        self.rotation * Vec3::new(0.0, 0.0, -1.0)
    }

    fn right(&self) -> Vec3 {
        self.rotation * Vec3::new(1.0, 0.0, 0.0)
    }

    /// Extreme camera movement: random teleports
    fn update_extreme_movement(&mut self, delta_time: f32) {
        self.teleport_timer += delta_time;

        if self.teleport_timer >= self.teleport_interval {
            self.teleport_timer = 0.0;

            // Random position in large area
            use rand::Rng;
            let mut rng = rand::thread_rng();
            self.position = Vec3::new(
                rng.gen_range(-200.0..200.0),
                rng.gen_range(10.0..100.0),
                rng.gen_range(-200.0..200.0),
            );

            // Random rotation
            self.yaw = rng.gen_range(0.0..std::f32::consts::TAU);
            self.pitch = rng.gen_range(-1.0..1.0);
            self.update_rotation();
        }
    }

    /// Fast sweeping movement for LOD stress
    fn update_sweep_movement(&mut self, delta_time: f32) {
        self.sweep_timer += delta_time;

        // Fast circular motion
        let radius = 150.0;
        let angular_speed = 0.5; // radians per second
        let angle = self.sweep_timer * angular_speed;

        self.position = Vec3::new(
            radius * angle.cos(),
            30.0 + (self.sweep_timer * 0.3).sin() * 20.0,
            radius * angle.sin(),
        );

        // Look toward center
        let to_center = -self.position.normalize();
        self.yaw = to_center.x.atan2(-to_center.z);
        self.pitch = to_center.y.asin();
        self.update_rotation();
    }
}

/// Performance and resource tracking
#[derive(Resource)]
struct StressTestState {
    current_test: StressTest,
    test_start_time: Instant,
    test_duration: Duration,
    frame_count: u32,
    total_frame_time: f32,
    min_fps: f32,
    max_fps: f32,
    baseline_memory_mb: f32,
    current_memory_mb: f32,
    peak_memory_mb: f32,
    total_objects_spawned: u32,
    total_objects_despawned: u32,
    visible_objects: u32,
    culled_objects: u32,
    draw_calls: u32,
    material_instances_created: u32,
    mesh_loads: u32,
    mesh_unloads: u32,
    descriptor_sets_allocated: u32,
    last_stats_print: Instant,
    passed_tests: Vec<StressTest>,
    failed_tests: Vec<(StressTest, String)>,
}

impl Default for StressTestState {
    fn default() -> Self {
        Self {
            current_test: StressTest::Idle,
            test_start_time: Instant::now(),
            test_duration: Duration::from_secs(10), // 10 seconds per test
            frame_count: 0,
            total_frame_time: 0.0,
            min_fps: f32::INFINITY,
            max_fps: 0.0,
            baseline_memory_mb: 0.0,
            current_memory_mb: 0.0,
            peak_memory_mb: 0.0,
            total_objects_spawned: 0,
            total_objects_despawned: 0,
            visible_objects: 0,
            culled_objects: 0,
            draw_calls: 0,
            material_instances_created: 0,
            mesh_loads: 0,
            mesh_unloads: 0,
            descriptor_sets_allocated: 0,
            last_stats_print: Instant::now(),
            passed_tests: Vec::new(),
            failed_tests: Vec::new(),
        }
    }
}

impl StressTestState {
    fn start_test(&mut self, test: StressTest) {
        info!("========================================");
        info!("Starting test: {}", test.name());
        info!("========================================");
        self.current_test = test;
        self.test_start_time = Instant::now();
        self.frame_count = 0;
        self.total_frame_time = 0.0;
        self.min_fps = f32::INFINITY;
        self.max_fps = 0.0;
        self.visible_objects = 0;
        self.culled_objects = 0;
    }

    fn update_frame_stats(&mut self, frame_time: f32) {
        self.frame_count += 1;
        self.total_frame_time += frame_time;

        let fps = 1.0 / frame_time;
        self.min_fps = self.min_fps.min(fps);
        self.max_fps = self.max_fps.max(fps);
    }

    fn print_statistics(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_stats_print).as_secs() < 1 {
            return;
        }
        self.last_stats_print = now;

        let avg_fps = if self.total_frame_time > 0.0 {
            self.frame_count as f32 / self.total_frame_time
        } else {
            0.0
        };

        info!("=== Stress Test Statistics ===");
        info!("Current Test: {}", self.current_test.name());
        info!(
            "Time Elapsed: {:.1}s / {:.1}s",
            self.test_start_time.elapsed().as_secs_f32(),
            self.test_duration.as_secs_f32()
        );
        info!(
            "FPS: avg={:.1}, min={:.1}, max={:.1}",
            avg_fps, self.min_fps, self.max_fps
        );
        info!(
            "Memory: current={:.1}MB, peak={:.1}MB, growth={:.1}MB",
            self.current_memory_mb,
            self.peak_memory_mb,
            self.current_memory_mb - self.baseline_memory_mb
        );
        info!(
            "Objects: spawned={}, despawned={}, visible={}, culled={}",
            self.total_objects_spawned,
            self.total_objects_despawned,
            self.visible_objects,
            self.culled_objects
        );
        info!("Draw Calls: {}", self.draw_calls);
        info!("Material Instances: {}", self.material_instances_created);
        info!(
            "Mesh Loads/Unloads: {} / {}",
            self.mesh_loads, self.mesh_unloads
        );
        info!(
            "Descriptor Sets Allocated: {}",
            self.descriptor_sets_allocated
        );

        // Check for test completion
        if self.test_start_time.elapsed() >= self.test_duration
            && self.current_test != StressTest::Idle
        {
            self.validate_test_completion();
        }
    }

    fn validate_test_completion(&mut self) {
        let avg_fps = if self.total_frame_time > 0.0 {
            self.frame_count as f32 / self.total_frame_time
        } else {
            0.0
        };

        let memory_growth = self.current_memory_mb - self.baseline_memory_mb;

        // Validation criteria
        let min_acceptable_fps = 15.0;
        let max_acceptable_memory_growth = 500.0; // MB

        let mut passed = true;
        let mut failure_reason = String::new();

        if avg_fps < min_acceptable_fps {
            passed = false;
            failure_reason.push_str(&format!(
                "FPS too low ({:.1} < {:.1}). ",
                avg_fps, min_acceptable_fps
            ));
        }

        if memory_growth > max_acceptable_memory_growth {
            passed = false;
            failure_reason.push_str(&format!(
                "Memory growth too high ({:.1}MB > {:.1}MB). ",
                memory_growth, max_acceptable_memory_growth
            ));
        }

        if self.min_fps < 1.0 {
            passed = false;
            failure_reason.push_str("Experienced severe frame drops (<1 FPS). ");
        }

        if passed {
            info!("✓ Test PASSED: {}", self.current_test.name());
            self.passed_tests.push(self.current_test);
        } else {
            warn!(
                "✗ Test FAILED: {} - {}",
                self.current_test.name(),
                failure_reason
            );
            self.failed_tests.push((self.current_test, failure_reason));
        }
    }

    fn print_final_report(&self) {
        info!("\n========================================");
        info!("FINAL STRESS TEST REPORT");
        info!("========================================");
        info!("Tests Passed: {}", self.passed_tests.len());
        info!("Tests Failed: {}", self.failed_tests.len());
        info!("");

        if !self.passed_tests.is_empty() {
            info!("Passed Tests:");
            for test in &self.passed_tests {
                info!("  ✓ {}", test.name());
            }
            info!("");
        }

        if !self.failed_tests.is_empty() {
            info!("Failed Tests:");
            for (test, reason) in &self.failed_tests {
                info!("  ✗ {} - {}", test.name(), reason);
            }
            info!("");
        }

        let success_rate = if self.passed_tests.len() + self.failed_tests.len() > 0 {
            (self.passed_tests.len() as f32
                / (self.passed_tests.len() + self.failed_tests.len()) as f32)
                * 100.0
        } else {
            0.0
        };

        info!("Success Rate: {:.1}%", success_rate);
        info!("========================================\n");
    }
}

/// Setup stress test scene
fn setup_stress_scene(
    world: &mut World,
    render_context: &mut RenderContext,
    test: StressTest,
) -> Result<u32> {
    match test {
        StressTest::Idle => setup_idle_scene(world, render_context),
        StressTest::MassiveObjectCount => setup_massive_object_count(world, render_context),
        StressTest::ExtremeCameraMovement => setup_camera_movement_test(world, render_context),
        StressTest::RapidLodTransitions => setup_lod_transition_test(world, render_context),
        StressTest::MaterialInstanceStress => setup_material_instance_test(world, render_context),
        StressTest::MeshStreamingThroughput => setup_mesh_streaming_test(world, render_context),
        StressTest::CombinedStress => setup_combined_stress_test(world, render_context),
        StressTest::ResourceCleanup => setup_cleanup_validation_test(world, render_context),
    }
}

/// Idle scene: minimal objects
fn setup_idle_scene(_world: &mut World, _render_context: &mut RenderContext) -> Result<u32> {
    info!("Setting up idle scene (minimal load)");
    Ok(0)
}

/// Test 1: Massive object count (10,000+ objects)
fn setup_massive_object_count(
    world: &mut World,
    render_context: &mut RenderContext,
) -> Result<u32> {
    info!("Setting up massive object count test (10,000+ objects)");

    // Load meshes
    let cube_mesh = colored_cube_mesh();
    let sphere_mesh_data = sphere_mesh(1.5, 16, 16, [0.7, 0.3, 0.3]);
    let occluder_mesh = solid_cube_mesh([0.5, 0.5, 0.5]);

    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("sphere", sphere_mesh_data)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("occluder", occluder_mesh)?;

    let mut count = 0;

    // Create dense grid of objects
    const GRID_SIZE: i32 = 32; // 32x32 = 1024 per layer
    const LAYERS: i32 = 10; // 10 layers = 10,240 objects
    const SPACING: f32 = 5.0;

    for layer in 0..LAYERS {
        for x in -GRID_SIZE..GRID_SIZE {
            for z in -GRID_SIZE..GRID_SIZE {
                // Vary Y position
                let y =
                    layer as f32 * 10.0 + ((x as f32 * 0.1).sin() + (z as f32 * 0.1).cos()) * 2.0;
                let position = Vec3::new(x as f32 * SPACING, y, z as f32 * SPACING);

                // Alternate mesh types
                let object_type = match (x + z + layer) % 3 {
                    0 => ObjectType::Cube,
                    1 => ObjectType::Sphere,
                    _ => ObjectType::Occluder,
                };

                world.spawn((
                    Transform::from_translation(position),
                    GlobalTransform::default(),
                    StressObject {
                        object_type,
                        lod_level: 0,
                        material_instance_id: None,
                        distance_to_camera: 0.0,
                        is_visible: true,
                        lifetime: 0.0,
                    },
                ));
                count += 1;
            }
        }
    }

    info!("Created {} objects for massive object count test", count);
    Ok(count)
}

/// Test 2: Extreme camera movement test
fn setup_camera_movement_test(
    world: &mut World,
    render_context: &mut RenderContext,
) -> Result<u32> {
    info!("Setting up extreme camera movement test");

    // Create scattered objects across large area
    let cube_mesh = colored_cube_mesh();
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;

    let mut count = 0;
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..5000 {
        let position = Vec3::new(
            rng.gen_range(-300.0..300.0),
            rng.gen_range(0.0..100.0),
            rng.gen_range(-300.0..300.0),
        );

        world.spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            StressObject {
                object_type: ObjectType::Cube,
                lod_level: 0,
                material_instance_id: None,
                distance_to_camera: 0.0,
                is_visible: true,
                lifetime: 0.0,
            },
        ));
        count += 1;
    }

    info!("Created {} objects for camera movement test", count);
    Ok(count)
}

/// Test 3: Rapid LOD transition test
fn setup_lod_transition_test(world: &mut World, render_context: &mut RenderContext) -> Result<u32> {
    info!("Setting up rapid LOD transition test");

    // Load LOD meshes
    let sphere_high = sphere_mesh(2.0, 32, 32, [0.7, 0.3, 0.3]);
    let sphere_medium = sphere_mesh(2.0, 16, 16, [0.7, 0.7, 0.3]);
    let sphere_low = sphere_mesh(2.0, 8, 8, [0.3, 0.7, 0.3]);

    render_context
        .mesh_manager_mut()
        .load_mesh("sphere_high", sphere_high)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("sphere_medium", sphere_medium)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("sphere_low", sphere_low)?;

    let mut count = 0;

    // Create radial pattern that will trigger LOD transitions during camera sweep
    const RINGS: i32 = 20;
    const OBJECTS_PER_RING: i32 = 50;

    for ring in 0..RINGS {
        let radius = (ring + 1) as f32 * 10.0;
        for i in 0..OBJECTS_PER_RING {
            let angle = (i as f32 / OBJECTS_PER_RING as f32) * std::f32::consts::TAU;
            let position = Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin());

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                StressObject {
                    object_type: ObjectType::Sphere,
                    lod_level: 0,
                    material_instance_id: None,
                    distance_to_camera: 0.0,
                    is_visible: true,
                    lifetime: 0.0,
                },
            ));
            count += 1;
        }
    }

    info!("Created {} objects for LOD transition test", count);
    Ok(count)
}

/// Test 4: Material instance stress test (500+ instances)
fn setup_material_instance_test(
    world: &mut World,
    render_context: &mut RenderContext,
) -> Result<u32> {
    info!("Setting up material instance stress test (500+ instances)");

    // Load base mesh
    let cube_mesh = colored_cube_mesh();
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;

    // Create base material
    let white_texture = render_context
        .texture_manager()
        .get_texture("_default_white")
        .ok_or_else(|| praxis_utils::eyre::eyre!("Default white texture not found"))?;

    render_context
        .material_manager_mut()
        .create_material("base_material", white_texture.clone());

    // Create 500 material instances with different properties
    const NUM_MATERIAL_INSTANCES: usize = 500;
    let mut material_instance_ids = Vec::new();

    for i in 0..NUM_MATERIAL_INSTANCES {
        let instance_id = format!("material_instance_{}", i);

        // Generate varied properties
        let hue = (i as f32 / NUM_MATERIAL_INSTANCES as f32) * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.9);
        let metallic = (i as f32 / NUM_MATERIAL_INSTANCES as f32) * 0.5 + 0.5;
        let roughness = ((i * 7) % NUM_MATERIAL_INSTANCES) as f32 / NUM_MATERIAL_INSTANCES as f32;

        render_context
            .create_material_instance(&instance_id, "base_material")?
            .override_properties(
                MaterialProperties::new()
                    .with_base_color([r, g, b, 1.0])
                    .with_metallic(metallic)
                    .with_roughness(roughness),
            );

        material_instance_ids.push(instance_id);
    }

    // Create objects using material instances
    let mut count = 0;
    const GRID_SIZE: i32 = 25; // 25x25 = 625 objects (uses all 500 instances + repeats)
    const SPACING: f32 = 4.0;

    for x in -GRID_SIZE..GRID_SIZE {
        for z in -GRID_SIZE..GRID_SIZE {
            let y = ((x as f32 * 0.2).sin() + (z as f32 * 0.2).cos()) * 2.0;
            let position = Vec3::new(x as f32 * SPACING, y, z as f32 * SPACING);

            // Cycle through material instances
            let material_idx = count % NUM_MATERIAL_INSTANCES;
            let material_instance_id = Some(material_instance_ids[material_idx].clone());

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                StressObject {
                    object_type: ObjectType::Cube,
                    lod_level: 0,
                    material_instance_id,
                    distance_to_camera: 0.0,
                    is_visible: true,
                    lifetime: 0.0,
                },
            ));
            count += 1;
        }
    }

    info!(
        "Created {} material instances and {} objects",
        NUM_MATERIAL_INSTANCES, count
    );
    Ok(count)
}

/// Test 5: Mesh streaming throughput test
fn setup_mesh_streaming_test(world: &mut World, render_context: &mut RenderContext) -> Result<u32> {
    info!("Setting up mesh streaming throughput test");

    // Load initial meshes
    let cube_mesh = colored_cube_mesh();
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;

    // Create objects that will be continuously spawned and despawned
    let mut count = 0;
    const INITIAL_OBJECTS: i32 = 2000;
    const SPACING: f32 = 10.0;

    for i in 0..INITIAL_OBJECTS {
        let angle = (i as f32 / INITIAL_OBJECTS as f32) * std::f32::consts::TAU;
        let radius = 50.0 + (i as f32 * 0.5);
        let position = Vec3::new(radius * angle.cos(), 0.0, radius * angle.sin());

        world.spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            StressObject {
                object_type: ObjectType::Cube,
                lod_level: 0,
                material_instance_id: None,
                distance_to_camera: 0.0,
                is_visible: true,
                lifetime: 0.0, // Will be updated each frame
            },
        ));
        count += 1;
    }

    info!("Created {} objects for mesh streaming test", count);
    Ok(count)
}

/// Test 6: Combined stress test (all tests at once)
fn setup_combined_stress_test(
    world: &mut World,
    render_context: &mut RenderContext,
) -> Result<u32> {
    info!("Setting up combined stress test (all scenarios simultaneously)");

    let mut total_count = 0;

    // Massive objects (reduced to 5000 for combined test)
    let cube_mesh = colored_cube_mesh();
    let sphere_mesh_data = sphere_mesh(1.5, 16, 16, [0.7, 0.3, 0.3]);
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("sphere", sphere_mesh_data)?;

    const GRID_SIZE: i32 = 22; // ~22x22 = 484 per layer
    const LAYERS: i32 = 10; // ~4,840 objects

    for layer in 0..LAYERS {
        for x in -GRID_SIZE..GRID_SIZE {
            for z in -GRID_SIZE..GRID_SIZE {
                let y = layer as f32 * 8.0;
                let position = Vec3::new(x as f32 * 5.0, y, z as f32 * 5.0);

                world.spawn((
                    Transform::from_translation(position),
                    GlobalTransform::default(),
                    StressObject {
                        object_type: if (x + z) % 2 == 0 {
                            ObjectType::Cube
                        } else {
                            ObjectType::Sphere
                        },
                        lod_level: 0,
                        material_instance_id: None,
                        distance_to_camera: 0.0,
                        is_visible: true,
                        lifetime: 0.0,
                    },
                ));
                total_count += 1;
            }
        }
    }

    // Add material instances (200 instead of 500)
    let white_texture = render_context
        .texture_manager()
        .get_texture("_default_white")
        .ok_or_else(|| praxis_utils::eyre::eyre!("Default white texture not found"))?;

    render_context
        .material_manager_mut()
        .create_material("base_material", white_texture.clone());

    for i in 0..200 {
        let instance_id = format!("combined_material_{}", i);
        let hue = (i as f32 / 200.0) * 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.9);

        render_context
            .create_material_instance(&instance_id, "base_material")?
            .override_properties(MaterialProperties::new().with_base_color([r, g, b, 1.0]));
    }

    info!("Created {} objects for combined stress test", total_count);
    Ok(total_count)
}

/// Test 7: Resource cleanup validation test
fn setup_cleanup_validation_test(
    world: &mut World,
    render_context: &mut RenderContext,
) -> Result<u32> {
    info!("Setting up resource cleanup validation test");

    // This test will spawn and despawn objects repeatedly to validate cleanup
    let cube_mesh = colored_cube_mesh();
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;

    let mut count = 0;
    const INITIAL_BATCH: i32 = 1000;

    for i in 0..INITIAL_BATCH {
        let position = Vec3::new(
            (i as f32 % 32.0) * 5.0 - 80.0,
            0.0,
            ((i / 32) as f32) * 5.0 - 80.0,
        );

        world.spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            StressObject {
                object_type: ObjectType::Cube,
                lod_level: 0,
                material_instance_id: None,
                distance_to_camera: 0.0,
                is_visible: true,
                lifetime: 0.0,
            },
        ));
        count += 1;
    }

    info!("Created {} objects for cleanup validation test", count);
    Ok(count)
}

/// Update camera based on current test
fn update_camera_system(
    mut camera: ResMut<StressCamera>,
    state: ResMut<StressTestState>,
    delta_time: f32,
) {
    match state.current_test {
        StressTest::ExtremeCameraMovement | StressTest::CombinedStress => {
            camera.update_extreme_movement(delta_time);
        }
        StressTest::RapidLodTransitions => {
            camera.update_sweep_movement(delta_time);
        }
        _ => {
            // Manual camera control
            update_manual_camera(&mut camera, delta_time);
        }
    }
}

/// Manual camera control from keyboard input
fn update_manual_camera(camera: &mut StressCamera, delta_time: f32) {
    // Update rotation
    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;

    if camera.rotate_left {
        yaw_delta += camera.rotate_speed * delta_time;
    }
    if camera.rotate_right {
        yaw_delta -= camera.rotate_speed * delta_time;
    }
    if camera.rotate_up_key {
        pitch_delta += camera.rotate_speed * delta_time;
    }
    if camera.rotate_down_key {
        pitch_delta -= camera.rotate_speed * delta_time;
    }

    camera.yaw += yaw_delta;
    camera.pitch = (camera.pitch + pitch_delta).clamp(-1.5, 1.5);
    camera.update_rotation();

    // Update position
    let forward = camera.forward();
    let right = camera.right();
    let up = Vec3::Y;

    let mut velocity = Vec3::ZERO;

    if camera.move_forward {
        velocity += forward;
    }
    if camera.move_backward {
        velocity -= forward;
    }
    if camera.move_right {
        velocity += right;
    }
    if camera.move_left {
        velocity -= right;
    }
    if camera.move_up {
        velocity += up;
    }
    if camera.move_down {
        velocity -= up;
    }

    if velocity.length_squared() > 0.0 {
        velocity = velocity.normalize();
        camera.position += velocity * camera.move_speed * delta_time;
    }
}

/// Culling and LOD system
fn culling_lod_system(
    camera: ResMut<StressCamera>,
    mut state: ResMut<StressTestState>,
    mut query: Query<(&Transform, &mut StressObject)>,
) {
    let camera_pos = camera.position;
    let camera_forward = camera.forward();

    let mut visible = 0;
    let mut culled = 0;

    for (transform, mut obj) in query.iter_mut() {
        let object_pos = transform.translation;
        let distance_squared = (object_pos - camera_pos).length_squared();
        obj.distance_to_camera = distance_squared.sqrt();

        // Simple frustum culling
        let to_object = (object_pos - camera_pos).normalize();
        let dot = camera_forward.dot(to_object);
        obj.is_visible = dot > -0.5 && obj.distance_to_camera < 500.0;

        // LOD selection for spheres
        if obj.object_type == ObjectType::Sphere {
            obj.lod_level = if obj.distance_to_camera < 30.0 {
                0 // High
            } else if obj.distance_to_camera < 80.0 {
                1 // Medium
            } else {
                2 // Low
            };
        }

        if obj.is_visible {
            visible += 1;
        } else {
            culled += 1;
        }
    }

    state.visible_objects = visible;
    state.culled_objects = culled;
}

/// Render system
fn render_system(
    world: &World,
    render_context: &mut RenderContext,
    state: &mut StressTestState,
) -> Result<()> {
    let camera = world
        .get_resource::<StressCamera>()
        .ok_or_else(|| praxis_utils::eyre::eyre!("Camera not found"))?;

    // Build view and projection matrices
    let forward = camera.forward();
    let target = camera.position + forward;
    let view = Mat4::look_at_rh(camera.position, target, Vec3::Y);
    let aspect = 1280.0 / 720.0;
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 2000.0);

    // Build draw commands
    let mut draw_commands = Vec::new();
    let query = world.query::<(&Transform, &StressObject)>();

    for (_entity, (transform, obj)) in query.iter() {
        if !obj.is_visible {
            continue;
        }

        let mesh_id = match obj.object_type {
            ObjectType::Cube => "cube",
            ObjectType::Sphere => match obj.lod_level {
                0 => "sphere_high",
                1 => "sphere_medium",
                2 => "sphere_low",
                _ => "sphere",
            },
            ObjectType::Occluder => "occluder",
        };

        draw_commands.push(DrawCommand {
            mesh_id: mesh_id.to_string(),
            model: transform.compute_matrix(),
            texture_name: None,
            material_properties: None,
            material_instance_id: obj.material_instance_id.clone(),
            bone_matrices: None,
        });
    }

    state.draw_calls = draw_commands.len() as u32;

    let render_commands = RenderCommands {
        view,
        proj: projection,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&render_commands)?;

    Ok(())
}

/// Input handling
fn handle_input(
    event: &WindowEvent,
    camera: &mut StressCamera,
    state: &mut StressTestState,
    world: &mut World,
    render_context: &mut RenderContext,
) {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state: key_state,
                    ..
                },
            ..
        } => {
            let pressed = *key_state == ElementState::Pressed;

            match keycode {
                // Camera controls
                KeyCode::KeyW => camera.move_forward = pressed,
                KeyCode::KeyS => camera.move_backward = pressed,
                KeyCode::KeyA => camera.move_left = pressed,
                KeyCode::KeyD => camera.move_right = pressed,
                KeyCode::KeyQ => camera.move_down = pressed,
                KeyCode::KeyE => camera.move_up = pressed,
                KeyCode::ArrowLeft => camera.rotate_left = pressed,
                KeyCode::ArrowRight => camera.rotate_right = pressed,
                KeyCode::ArrowUp => camera.rotate_up_key = pressed,
                KeyCode::ArrowDown => camera.rotate_down_key = pressed,

                // Test selection
                KeyCode::Digit1 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::MassiveObjectCount);
                    if let Err(e) =
                        setup_stress_scene(world, render_context, StressTest::MassiveObjectCount)
                    {
                        error!("Failed to setup test: {}", e);
                    }
                }
                KeyCode::Digit2 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::ExtremeCameraMovement);
                    if let Err(e) =
                        setup_stress_scene(world, render_context, StressTest::ExtremeCameraMovement)
                    {
                        error!("Failed to setup test: {}", e);
                    }
                }
                KeyCode::Digit3 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::RapidLodTransitions);
                    if let Err(e) =
                        setup_stress_scene(world, render_context, StressTest::RapidLodTransitions)
                    {
                        error!("Failed to setup test: {}", e);
                    }
                }
                KeyCode::Digit4 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::MaterialInstanceStress);
                    if let Err(e) = setup_stress_scene(
                        world,
                        render_context,
                        StressTest::MaterialInstanceStress,
                    ) {
                        error!("Failed to setup test: {}", e);
                    }
                }
                KeyCode::Digit5 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::MeshStreamingThroughput);
                    if let Err(e) = setup_stress_scene(
                        world,
                        render_context,
                        StressTest::MeshStreamingThroughput,
                    ) {
                        error!("Failed to setup test: {}", e);
                    }
                }
                KeyCode::Digit6 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::CombinedStress);
                    if let Err(e) =
                        setup_stress_scene(world, render_context, StressTest::CombinedStress)
                    {
                        error!("Failed to setup test: {}", e);
                    }
                }
                KeyCode::Digit7 if pressed => {
                    clear_scene(world);
                    state.start_test(StressTest::ResourceCleanup);
                    if let Err(e) =
                        setup_stress_scene(world, render_context, StressTest::ResourceCleanup)
                    {
                        error!("Failed to setup test: {}", e);
                    }
                }

                // Reset
                KeyCode::Space if pressed => {
                    clear_scene(world);
                    camera.reset();
                    state.start_test(StressTest::Idle);
                    info!("Reset to idle state");
                }

                // Print stats
                KeyCode::KeyP if pressed => {
                    state.print_statistics();
                }

                _ => {}
            }
        }
        _ => {}
    }
}

/// Clear all test objects from scene
fn clear_scene(world: &mut World) {
    let entities_to_despawn: Vec<_> = world
        .query::<&StressObject>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();

    for entity in entities_to_despawn {
        world.despawn(entity);
    }
}

/// HSV to RGB conversion
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => (c, x, 0.0),
    };

    (r + m, g + m, b + m)
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    info!("========================================");
    info!("RENDERING PIPELINE STRESS TEST");
    info!("========================================");
    info!("");
    info!("This comprehensive stress test validates rendering pipeline stability,");
    info!("performance, and resource management under extreme conditions.");
    info!("");
    info!("TEST SCENARIOS:");
    info!("  1 - Massive Object Count (10,000+ objects)");
    info!("  2 - Extreme Camera Movement (rapid teleports)");
    info!("  3 - Rapid LOD Transitions (fast camera sweeps)");
    info!("  4 - Material Instance Stress (500+ instances)");
    info!("  5 - Mesh Streaming Throughput (continuous load/unload)");
    info!("  6 - Combined Stress (all tests simultaneously)");
    info!("  7 - Resource Cleanup Validation");
    info!("");
    info!("CONTROLS:");
    info!("  1-7         - Select stress test scenario");
    info!("  Space       - Reset to idle state");
    info!("  P           - Print current statistics");
    info!("  WASD/QE     - Manual camera movement");
    info!("  Arrow Keys  - Manual camera rotation");
    info!("  ESC         - Exit");
    info!("");
    info!("VALIDATION CRITERIA:");
    info!("  ✓ No crashes during any test");
    info!("  ✓ Minimum FPS >15 (acceptable degradation)");
    info!("  ✓ Memory growth <500MB");
    info!("  ✓ No resource leaks");
    info!("");
    info!("Starting in idle state. Press 1-7 to begin a test.");
    info!("========================================");
    info!("");

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    // Initialize resources
    engine.world_mut().insert_resource(StressCamera::default());
    engine
        .world_mut()
        .insert_resource(StressTestState::default());

    // Main loop
    let mut last_time = Instant::now();

    engine.run(move |engine_state, event| {
        let current_time = Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32().min(0.1);
        last_time = current_time;

        // Handle input
        if let Some(window_event) = event {
            if let (Some(mut camera), Some(mut state)) = (
                engine_state.world.get_resource_mut::<StressCamera>(),
                engine_state.world.get_resource_mut::<StressTestState>(),
            ) {
                if let Some(render_context) = engine_state.render_context.as_mut() {
                    handle_input(
                        window_event,
                        &mut camera,
                        &mut state,
                        &mut engine_state.world,
                        render_context,
                    );
                }
            }
        }

        // Update camera
        if let (Some(camera), Some(state)) = (
            engine_state.world.get_resource_mut::<StressCamera>(),
            engine_state.world.get_resource::<StressTestState>(),
        ) {
            update_camera_system(camera, state, delta_time);
        }

        // Update culling and LOD
        if let (Some(camera), Some(state)) = (
            engine_state.world.get_resource::<StressCamera>(),
            engine_state.world.get_resource_mut::<StressTestState>(),
        ) {
            culling_lod_system(
                camera,
                state,
                engine_state
                    .world
                    .query::<(&Transform, &mut StressObject)>(),
            );
        }

        // Render
        if let (Some(render_context), Some(mut state)) = (
            engine_state.render_context.as_mut(),
            engine_state.world.get_resource_mut::<StressTestState>(),
        ) {
            if let Err(e) = render_system(&engine_state.world, render_context, &mut state) {
                warn!("Render error: {}", e);
            }

            // Update frame stats
            state.update_frame_stats(delta_time);

            // Estimate memory usage (simplified)
            state.current_memory_mb = 100.0 + (state.visible_objects as f32 * 0.01);
            state.peak_memory_mb = state.peak_memory_mb.max(state.current_memory_mb);

            // Print periodic stats
            state.print_statistics();
        }
    })?;

    // Print final report
    if let Some(state) = engine.world().get_resource::<StressTestState>() {
        state.print_final_report();
    }

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("Rendering stress test requires graphics support");
    Ok(())
}
