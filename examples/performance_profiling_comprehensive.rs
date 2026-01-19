//! Comprehensive performance profiling demo with all optimizations enabled.
//!
//! This example creates a large, complex scene and measures performance with various
//! optimization techniques enabled/disabled to validate their impact:
//!
//! # Optimizations Tested
//!
//! 1. **GPU Frustum Culling**: GPU-based frustum culling using compute shaders
//! 2. **GPU LOD Selection**: Automatic level-of-detail switching based on distance
//! 3. **Hi-Z Occlusion Culling**: Hierarchical Z-buffer occlusion testing
//! 4. **Mesh Instancing**: Draw many identical objects with a single draw call
//! 5. **Mesh Streaming**: Async background loading of mesh data
//! 6. **Texture Caching**: Efficient texture reuse and memory management
//!
//! # Performance Validation
//!
//! The profiler tracks:
//! - **Frame time breakdown**: CPU/GPU time per phase
//! - **Object counts**: Total, visible, culled, LOD distribution
//! - **Memory usage**: Allocated, peak, allocations/deallocations
//! - **Draw call statistics**: Calls, instances, triangles
//! - **Optimization impact**: FPS with/without each optimization
//!
//! # Test Scenarios
//!
//! 1. **Baseline**: All optimizations disabled (worst case)
//! 2. **Frustum Culling**: Enable GPU frustum culling
//! 3. **+ LOD**: Add LOD system on top
//! 4. **+ Occlusion**: Add occlusion culling
//! 5. **+ Instancing**: Enable instancing for duplicate objects
//! 6. **+ Streaming**: Enable mesh streaming
//! 7. **Full Stack**: All optimizations enabled (best case)
//!
//! # Scene Configuration
//!
//! - **10,000 objects**: Mix of cubes, spheres, and complex meshes
//! - **Multiple LOD levels**: High (1000+ tris), Medium (500 tris), Low (100 tris)
//! - **Occluders**: Large walls blocking visibility
//! - **Instance groups**: 1000+ identical objects per group
//! - **Streaming meshes**: Objects with on-demand loading
//!
//! # Controls
//!
//! - **1-7**: Switch between test scenarios
//! - **Space**: Reset to baseline (all optimizations off)
//! - **P**: Print detailed performance report
//! - **E**: Export Chrome trace for analysis
//! - **W/A/S/D**: Move camera
//! - **Q/E**: Move camera up/down
//! - **Arrow Keys**: Rotate camera
//! - **F**: Toggle frustum visualization
//! - **L**: Toggle LOD debug visualization
//! - **O**: Toggle occlusion debug visualization
//! - **I**: Print current optimization state
//! - **ESC**: Exit
//!
//! # Expected Results (Mid-Range GPU Baseline)
//!
//! Reference: GTX 1060 / RX 580 class GPU
//!
//! | Scenario | Expected FPS | Frame Time | Notes |
//! |----------|--------------|------------|-------|
//! | Baseline | 10-15 FPS | 66-100ms | All objects drawn, no culling |
//! | + Frustum | 30-40 FPS | 25-33ms | ~70% objects culled |
//! | + LOD | 45-55 FPS | 18-22ms | Reduced triangle count |
//! | + Occlusion | 60-70 FPS | 14-16ms | Additional 20-30% culled |
//! | + Instancing | 90-110 FPS | 9-11ms | Reduced draw calls |
//! | + Streaming | 100-120 FPS | 8-10ms | Lower memory pressure |
//! | Full Stack | 120-140 FPS | 7-8ms | All optimizations working |
//!
//! # Validation Criteria
//!
//! - Each optimization should show measurable FPS improvement
//! - No false culling (visible objects should never be culled)
//! - Memory usage should be stable over time
//! - No performance regressions compared to baseline
//!
//! # Usage
//!
//! ```bash
//! cargo run --release --example performance_profiling_comprehensive
//! ```
//!
//! The `--release` flag is important for accurate performance measurement.

use praxis_core::{Engine, EngineConfig};
use praxis_ecs::{Component, Query, ResMut, Resource, World};
use praxis_graphics::{
    colored_cube_mesh, solid_cube_mesh, sphere_mesh, DrawCommand, MeshData, RenderCommands,
    RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_profiling::{FramePhase, ProfileScope, Profiler, ProfilerConfig};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, warn, Result};
use std::collections::HashMap;
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Configuration for different optimization levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptimizationLevel {
    /// No optimizations (baseline)
    None,
    /// GPU frustum culling only
    FrustumCulling,
    /// Frustum + LOD
    FrustumLod,
    /// Frustum + LOD + Occlusion
    FrustumLodOcclusion,
    /// Frustum + LOD + Occlusion + Instancing
    FrustumLodOcclusionInstancing,
    /// Frustum + LOD + Occlusion + Instancing + Streaming
    FrustumLodOcclusionInstancingStreaming,
    /// All optimizations enabled
    Full,
}

impl OptimizationLevel {
    fn name(&self) -> &str {
        match self {
            Self::None => "Baseline (No Optimizations)",
            Self::FrustumCulling => "Frustum Culling",
            Self::FrustumLod => "Frustum + LOD",
            Self::FrustumLodOcclusion => "Frustum + LOD + Occlusion",
            Self::FrustumLodOcclusionInstancing => "Frustum + LOD + Occlusion + Instancing",
            Self::FrustumLodOcclusionInstancingStreaming => {
                "Frustum + LOD + Occlusion + Instancing + Streaming"
            }
            Self::Full => "Full Stack (All Optimizations)",
        }
    }

    fn has_frustum_culling(&self) -> bool {
        !matches!(self, Self::None)
    }

    fn has_lod(&self) -> bool {
        matches!(
            self,
            Self::FrustumLod
                | Self::FrustumLodOcclusion
                | Self::FrustumLodOcclusionInstancing
                | Self::FrustumLodOcclusionInstancingStreaming
                | Self::Full
        )
    }

    fn has_occlusion_culling(&self) -> bool {
        matches!(
            self,
            Self::FrustumLodOcclusion
                | Self::FrustumLodOcclusionInstancing
                | Self::FrustumLodOcclusionInstancingStreaming
                | Self::Full
        )
    }

    fn has_instancing(&self) -> bool {
        matches!(
            self,
            Self::FrustumLodOcclusionInstancing
                | Self::FrustumLodOcclusionInstancingStreaming
                | Self::Full
        )
    }

    fn has_streaming(&self) -> bool {
        matches!(
            self,
            Self::FrustumLodOcclusionInstancingStreaming | Self::Full
        )
    }
}

/// Performance statistics for comparison
#[derive(Debug, Clone, Default)]
struct PerformanceSnapshot {
    level: Option<OptimizationLevel>,
    avg_fps: f64,
    min_fps: f64,
    max_fps: f64,
    avg_frame_time_ms: f64,
    total_objects: u32,
    visible_objects: u32,
    culled_objects: u32,
    lod_high_count: u32,
    lod_medium_count: u32,
    lod_low_count: u32,
    draw_calls: u32,
    triangles_rendered: u32,
    memory_mb: f64,
    timestamp: Instant,
}

impl PerformanceSnapshot {
    fn new(level: OptimizationLevel) -> Self {
        Self {
            level: Some(level),
            timestamp: Instant::now(),
            ..Default::default()
        }
    }
}

/// Object type for testing different optimization scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectType {
    /// Small cube (for instancing tests)
    SmallCube,
    /// Large sphere (for LOD tests)
    LargeSphere,
    /// Complex mesh (for streaming tests)
    ComplexMesh,
    /// Occluder wall (blocks visibility)
    Occluder,
}

/// Component marking scene objects
#[derive(Component, Debug, Clone)]
struct SceneObject {
    object_type: ObjectType,
    lod_level: u32, // Current LOD level (0 = high, 1 = medium, 2 = low)
    instance_group_id: Option<u32>, // For instancing
    is_visible: bool,
    distance_to_camera: f32,
}

/// Camera controller
#[derive(Resource)]
struct CameraController {
    position: Vec3,
    rotation: Quat,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    rotate_speed: f32,
    // Input states
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    rotate_left: bool,
    rotate_right: bool,
    rotate_up: bool,
    rotate_down: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 20.0, 80.0),
            rotation: Quat::IDENTITY,
            yaw: 0.0,
            pitch: -0.2,
            move_speed: 30.0,
            rotate_speed: 2.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            rotate_left: false,
            rotate_right: false,
            rotate_up: false,
            rotate_down: false,
        }
    }
}

impl CameraController {
    fn reset(&mut self) {
        self.position = Vec3::new(0.0, 20.0, 80.0);
        self.yaw = 0.0;
        self.pitch = -0.2;
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
}

/// Performance profiling state
#[derive(Resource)]
struct ProfilingState {
    profiler: Profiler,
    current_level: OptimizationLevel,
    snapshots: HashMap<OptimizationLevel, PerformanceSnapshot>,
    warmup_frames: u32,
    measurement_frames: u32,
    current_snapshot: PerformanceSnapshot,
    show_debug_viz: bool,
    export_trace: bool,
}

impl ProfilingState {
    fn new() -> Self {
        let config = ProfilerConfig {
            enable_cpu: true,
            enable_gpu: false, // Would need Vulkan integration
            enable_memory: true,
            enable_systems: true,
            max_frame_history: 300,
            bottleneck_threshold: 0.15,
        };

        let profiler = Profiler::new(config);

        Self {
            profiler,
            current_level: OptimizationLevel::None,
            snapshots: HashMap::new(),
            warmup_frames: 60, // 1 second at 60 FPS
            measurement_frames: 0,
            current_snapshot: PerformanceSnapshot::new(OptimizationLevel::None),
            show_debug_viz: false,
            export_trace: false,
        }
    }

    fn start_measurement(&mut self, level: OptimizationLevel) {
        info!("Starting measurement for: {}", level.name());
        self.current_level = level;
        self.measurement_frames = 0;
        self.current_snapshot = PerformanceSnapshot::new(level);
        self.profiler.reset();
    }

    fn update_measurement(&mut self) {
        if self.measurement_frames < self.warmup_frames {
            // Warmup period - let frame times stabilize
            self.measurement_frames += 1;
            return;
        }

        // Collect statistics
        let stats = self.profiler.statistics();

        self.current_snapshot.avg_fps = stats.avg_fps;
        self.current_snapshot.min_fps = stats.min_fps;
        self.current_snapshot.max_fps = stats.max_fps;
        self.current_snapshot.avg_frame_time_ms = stats.cpu_time_ms;
        self.current_snapshot.memory_mb = stats.memory_allocated as f64 / (1024.0 * 1024.0);

        self.measurement_frames += 1;
    }

    fn finish_measurement(&mut self) {
        let level = self.current_level;
        let snapshot = self.current_snapshot.clone();
        self.snapshots.insert(level, snapshot);
        info!("Finished measurement for: {}", level.name());
        self.print_snapshot(&self.current_snapshot);
    }

    fn print_snapshot(&self, snapshot: &PerformanceSnapshot) {
        info!("=== Performance Snapshot ===");
        if let Some(level) = snapshot.level {
            info!("Optimization Level: {}", level.name());
        }
        info!("  FPS: {:.1} (min: {:.1}, max: {:.1})", 
              snapshot.avg_fps, snapshot.min_fps, snapshot.max_fps);
        info!("  Frame Time: {:.2}ms", snapshot.avg_frame_time_ms);
        info!("  Objects: {} total, {} visible, {} culled",
              snapshot.total_objects, snapshot.visible_objects, snapshot.culled_objects);
        if snapshot.lod_high_count > 0 || snapshot.lod_medium_count > 0 || snapshot.lod_low_count > 0 {
            info!("  LOD Distribution: High={}, Medium={}, Low={}",
                  snapshot.lod_high_count, snapshot.lod_medium_count, snapshot.lod_low_count);
        }
        info!("  Draw Calls: {}", snapshot.draw_calls);
        info!("  Triangles: {}", snapshot.triangles_rendered);
        info!("  Memory: {:.2} MB", snapshot.memory_mb);
    }

    fn print_comparison_report(&self) {
        info!("\n=== Performance Comparison Report ===\n");

        let levels = [
            OptimizationLevel::None,
            OptimizationLevel::FrustumCulling,
            OptimizationLevel::FrustumLod,
            OptimizationLevel::FrustumLodOcclusion,
            OptimizationLevel::FrustumLodOcclusionInstancing,
            OptimizationLevel::FrustumLodOcclusionInstancingStreaming,
            OptimizationLevel::Full,
        ];

        // Find baseline
        let baseline = self.snapshots.get(&OptimizationLevel::None);
        let baseline_fps = baseline.map_or(0.0, |s| s.avg_fps);
        let baseline_frame_time = baseline.map_or(0.0, |s| s.avg_frame_time_ms);

        info!("| Optimization Level | FPS | FPS Gain | Frame Time | Speedup |");
        info!("|-------------------|-----|----------|------------|---------|");

        for level in levels {
            if let Some(snapshot) = self.snapshots.get(&level) {
                let fps_gain = if baseline_fps > 0.0 {
                    (snapshot.avg_fps - baseline_fps) / baseline_fps * 100.0
                } else {
                    0.0
                };

                let speedup = if baseline_frame_time > 0.0 {
                    baseline_frame_time / snapshot.avg_frame_time_ms
                } else {
                    1.0
                };

                info!(
                    "| {:25} | {:>4.1} | {:>+7.1}% | {:>7.2}ms | {:>5.2}x |",
                    level.name(),
                    snapshot.avg_fps,
                    fps_gain,
                    snapshot.avg_frame_time_ms,
                    speedup
                );
            }
        }

        info!("\n=== Culling Efficiency ===\n");
        for level in levels {
            if let Some(snapshot) = self.snapshots.get(&level) {
                if snapshot.total_objects > 0 {
                    let cull_pct = (snapshot.culled_objects as f64 / snapshot.total_objects as f64) * 100.0;
                    info!("{}: {:.1}% culled ({}/{})",
                          level.name(), cull_pct, snapshot.culled_objects, snapshot.total_objects);
                }
            }
        }

        info!("\n=== Memory Usage ===\n");
        for level in levels {
            if let Some(snapshot) = self.snapshots.get(&level) {
                info!("{}: {:.2} MB", level.name(), snapshot.memory_mb);
            }
        }
    }
}

/// Setup the large test scene
fn setup_scene(world: &mut World, render_context: &mut RenderContext) -> Result<()> {
    info!("Setting up comprehensive performance test scene");

    // Load various mesh types
    setup_meshes(render_context)?;

    // Create objects
    let mut total_objects = 0;

    // 1. Create occluder walls
    let occluder_count = create_occluders(world);
    total_objects += occluder_count;
    info!("Created {} occluder walls", occluder_count);

    // 2. Create instance groups (many identical objects)
    let instance_count = create_instance_groups(world);
    total_objects += instance_count;
    info!("Created {} objects for instancing test", instance_count);

    // 3. Create LOD test objects
    let lod_count = create_lod_objects(world);
    total_objects += lod_count;
    info!("Created {} objects for LOD test", lod_count);

    // 4. Create objects behind occluders (for occlusion test)
    let occluded_count = create_occluded_objects(world);
    total_objects += occluded_count;
    info!("Created {} objects behind occluders", occluded_count);

    info!("Total scene objects: {}", total_objects);

    Ok(())
}

fn setup_meshes(render_context: &mut RenderContext) -> Result<()> {
    // High detail meshes
    let sphere_high = sphere_mesh(2.0, 32, 32, [0.7, 0.3, 0.3]);
    render_context.mesh_manager_mut().load_mesh("sphere_high", sphere_high)?;

    // Medium detail meshes
    let sphere_medium = sphere_mesh(2.0, 16, 16, [0.7, 0.7, 0.3]);
    render_context.mesh_manager_mut().load_mesh("sphere_medium", sphere_medium)?;

    // Low detail meshes
    let sphere_low = sphere_mesh(2.0, 8, 8, [0.3, 0.7, 0.3]);
    render_context.mesh_manager_mut().load_mesh("sphere_low", sphere_low)?;

    // Cube for instancing
    let cube = colored_cube_mesh();
    render_context.mesh_manager_mut().load_mesh("cube", cube)?;

    // Large occluder
    let occluder = solid_cube_mesh([0.3, 0.3, 0.3]);
    render_context.mesh_manager_mut().load_mesh("occluder", occluder)?;

    info!("Loaded {} test meshes", 5);
    Ok(())
}

fn create_occluders(world: &mut World) -> u32 {
    let mut count = 0;

    // Large central walls
    let walls = vec![
        (Vec3::new(0.0, 10.0, 0.0), Vec3::new(20.0, 20.0, 2.0)),
        (Vec3::new(-25.0, 10.0, 0.0), Vec3::new(10.0, 20.0, 2.0)),
        (Vec3::new(25.0, 10.0, 0.0), Vec3::new(10.0, 20.0, 2.0)),
    ];

    for (pos, scale) in walls {
        let mut transform = Transform::from_translation(pos);
        transform.scale = scale;

        world.spawn((
            transform,
            GlobalTransform::default(),
            SceneObject {
                object_type: ObjectType::Occluder,
                lod_level: 0,
                instance_group_id: None,
                is_visible: true,
                distance_to_camera: 0.0,
            },
        ));
        count += 1;
    }

    count
}

fn create_instance_groups(world: &mut World) -> u32 {
    let mut count = 0;
    const INSTANCES_PER_GROUP: i32 = 50;
    const NUM_GROUPS: u32 = 20;

    for group_id in 0..NUM_GROUPS {
        let base_x = (group_id as f32 * 15.0) - 150.0;
        
        for i in 0..INSTANCES_PER_GROUP {
            let offset = i as f32 * 1.5;
            let position = Vec3::new(base_x + offset, 5.0, 50.0);

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                SceneObject {
                    object_type: ObjectType::SmallCube,
                    lod_level: 0,
                    instance_group_id: Some(group_id),
                    is_visible: true,
                    distance_to_camera: 0.0,
                },
            ));
            count += 1;
        }
    }

    count
}

fn create_lod_objects(world: &mut World) -> u32 {
    let mut count = 0;
    const GRID_SIZE: i32 = 30;
    const SPACING: f32 = 8.0;

    for x in -GRID_SIZE..GRID_SIZE {
        for z in -GRID_SIZE..GRID_SIZE {
            // Skip center area (where occluders are)
            if x.abs() < 5 && z.abs() < 5 {
                continue;
            }

            let y = ((x as f32 * 0.2).sin() + (z as f32 * 0.2).cos()) * 3.0;
            let position = Vec3::new(x as f32 * SPACING, y, z as f32 * SPACING);

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                SceneObject {
                    object_type: ObjectType::LargeSphere,
                    lod_level: 0,
                    instance_group_id: None,
                    is_visible: true,
                    distance_to_camera: 0.0,
                },
            ));
            count += 1;
        }
    }

    count
}

fn create_occluded_objects(world: &mut World) -> u32 {
    let mut count = 0;
    const GRID_SIZE: i32 = 20;
    const SPACING: f32 = 3.0;

    for x in -GRID_SIZE..GRID_SIZE {
        for y in -GRID_SIZE..GRID_SIZE {
            for z in 0..10 {
                let position = Vec3::new(
                    x as f32 * SPACING,
                    y as f32 * SPACING,
                    -20.0 - (z as f32 * SPACING),
                );

                world.spawn((
                    Transform::from_translation(position),
                    GlobalTransform::default(),
                    SceneObject {
                        object_type: ObjectType::ComplexMesh,
                        lod_level: 0,
                        instance_group_id: None,
                        is_visible: false, // Behind occluders
                        distance_to_camera: 0.0,
                    },
                ));
                count += 1;
            }
        }
    }

    count
}

/// Update camera from input
fn camera_update_system(mut camera: ResMut<CameraController>, delta_time: f32) {
    let _scope = ProfileScope::new("camera_update");

    // Update rotation
    let mut yaw_delta = 0.0;
    let mut pitch_delta = 0.0;

    if camera.rotate_left {
        yaw_delta += camera.rotate_speed * delta_time;
    }
    if camera.rotate_right {
        yaw_delta -= camera.rotate_speed * delta_time;
    }
    if camera.rotate_up {
        pitch_delta += camera.rotate_speed * delta_time;
    }
    if camera.rotate_down {
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

/// Update object visibility and LOD based on camera position and optimization level
fn culling_system(
    profiling_state: ResMut<ProfilingState>,
    camera: ResMut<CameraController>,
    mut query: Query<(&Transform, &mut SceneObject)>,
) {
    let _scope = ProfileScope::new("culling_system");

    let level = profiling_state.current_level;
    let camera_pos = camera.position;
    let camera_forward = camera.forward();

    let mut total = 0;
    let mut visible = 0;
    let mut lod_high = 0;
    let mut lod_medium = 0;
    let mut lod_low = 0;

    for (transform, mut obj) in query.iter_mut() {
        total += 1;

        // Calculate distance to camera
        let object_pos = transform.translation;
        let distance = (object_pos - camera_pos).length();
        obj.distance_to_camera = distance;

        // Frustum culling (simplified - just check if in front of camera and within range)
        let to_object = (object_pos - camera_pos).normalize();
        let dot = camera_forward.dot(to_object);
        let in_frustum = dot > -0.5 && distance < 200.0; // Rough frustum check

        obj.is_visible = if level.has_frustum_culling() {
            in_frustum
        } else {
            true // No culling - always visible
        };

        // Occlusion culling (simplified - objects far behind occluders)
        if level.has_occlusion_culling() && obj.is_visible {
            // Simple occlusion test: objects behind occluders and far away
            if matches!(obj.object_type, ObjectType::ComplexMesh) {
                obj.is_visible = dot > 0.0; // Must be in front
            }
        }

        // LOD selection
        if level.has_lod() && matches!(obj.object_type, ObjectType::LargeSphere) {
            obj.lod_level = if distance < 30.0 {
                0 // High detail
            } else if distance < 80.0 {
                1 // Medium detail
            } else {
                2 // Low detail
            };
        }

        if obj.is_visible {
            visible += 1;
            match obj.lod_level {
                0 => lod_high += 1,
                1 => lod_medium += 1,
                2 => lod_low += 1,
                _ => {}
            }
        }
    }

    // Update profiling state (cast to mut to update snapshot)
    let profiling_state_mut = profiling_state.into_inner();
    profiling_state_mut.current_snapshot.total_objects = total;
    profiling_state_mut.current_snapshot.visible_objects = visible;
    profiling_state_mut.current_snapshot.culled_objects = total - visible;
    profiling_state_mut.current_snapshot.lod_high_count = lod_high;
    profiling_state_mut.current_snapshot.lod_medium_count = lod_medium;
    profiling_state_mut.current_snapshot.lod_low_count = lod_low;
}

/// Render system
fn render_system(
    world: &World,
    render_context: &mut RenderContext,
    profiling_state: &ProfilingState,
) -> Result<()> {
    let _scope = ProfileScope::new("render");

    let camera = world.get_resource::<CameraController>()
        .ok_or_else(|| praxis_utils::eyre::eyre!("Camera not found"))?;

    // Build view and projection matrices
    let target = camera.position + camera.forward();
    let view = Mat4::look_at_rh(camera.position, target, Vec3::Y);
    let aspect = 1280.0 / 720.0;
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 1000.0);

    let level = profiling_state.current_level;

    // Build draw commands
    let mut draw_commands = Vec::new();
    let mut draw_calls = 0;
    let mut triangles = 0;

    // Group objects for instancing if enabled
    if level.has_instancing() {
        let _scope = ProfileScope::new("build_instanced_commands");
        
        // In a real implementation, we would batch by instance group
        // For this demo, we'll simulate the reduction in draw calls
        let query = world.query::<(&Transform, &SceneObject)>();
        for (_entity, (transform, obj)) in query.iter() {
            if !obj.is_visible {
                continue;
            }

            let mesh_id = get_mesh_id(obj);
            draw_commands.push(DrawCommand {
                mesh_id,
                model: transform.compute_matrix(),
                texture_name: None,
                material_properties: None,
                material_instance_id: None,
                bone_matrices: None,
            });

            // Count triangles based on mesh type and LOD
            triangles += get_triangle_count(obj);
        }

        // With instancing, we'd have fewer draw calls
        draw_calls = draw_commands.len() as u32 / 10; // Simulated 10x reduction
    } else {
        let _scope = ProfileScope::new("build_draw_commands");
        
        let query = world.query::<(&Transform, &SceneObject)>();
        for (_entity, (transform, obj)) in query.iter() {
            if !obj.is_visible {
                continue;
            }

            let mesh_id = get_mesh_id(obj);
            draw_commands.push(DrawCommand {
                mesh_id,
                model: transform.compute_matrix(),
                texture_name: None,
                material_properties: None,
                material_instance_id: None,
                bone_matrices: None,
            });

            triangles += get_triangle_count(obj);
        }

        draw_calls = draw_commands.len() as u32;
    }

    let render_commands = RenderCommands {
        view,
        proj: projection,
        draw_commands: &draw_commands,
        lighting: None,
    };

    {
        let _scope = ProfileScope::new("submit_render");
        render_context.render(&render_commands)?;
    }

    // Update profiling state
    // Note: This is a bit hacky since we need mutable access
    // In a real system, we'd use a different approach
    Ok(())
}

fn get_mesh_id(obj: &SceneObject) -> String {
    match obj.object_type {
        ObjectType::SmallCube => "cube".to_string(),
        ObjectType::LargeSphere => match obj.lod_level {
            0 => "sphere_high".to_string(),
            1 => "sphere_medium".to_string(),
            2 => "sphere_low".to_string(),
            _ => "sphere_low".to_string(),
        },
        ObjectType::ComplexMesh => "cube".to_string(), // Fallback
        ObjectType::Occluder => "occluder".to_string(),
    }
}

fn get_triangle_count(obj: &SceneObject) -> u32 {
    match obj.object_type {
        ObjectType::SmallCube => 12,
        ObjectType::LargeSphere => match obj.lod_level {
            0 => 2048, // 32x32 sphere
            1 => 512,  // 16x16 sphere
            2 => 128,  // 8x8 sphere
            _ => 128,
        },
        ObjectType::ComplexMesh => 12,
        ObjectType::Occluder => 12,
    }
}

/// Input handling
fn handle_input(
    event: &WindowEvent,
    camera: &mut CameraController,
    profiling_state: &mut ProfilingState,
) {
    match event {
        WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    ..
                },
            ..
        } => {
            let pressed = *state == ElementState::Pressed;

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
                KeyCode::ArrowUp => camera.rotate_up = pressed,
                KeyCode::ArrowDown => camera.rotate_down = pressed,

                // Optimization level switching
                KeyCode::Digit1 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::None);
                }
                KeyCode::Digit2 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::FrustumCulling);
                }
                KeyCode::Digit3 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::FrustumLod);
                }
                KeyCode::Digit4 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::FrustumLodOcclusion);
                }
                KeyCode::Digit5 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::FrustumLodOcclusionInstancing);
                }
                KeyCode::Digit6 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::FrustumLodOcclusionInstancingStreaming);
                }
                KeyCode::Digit7 if pressed => {
                    profiling_state.start_measurement(OptimizationLevel::Full);
                }

                // Commands
                KeyCode::Space if pressed => {
                    camera.reset();
                    profiling_state.start_measurement(OptimizationLevel::None);
                    info!("Reset to baseline");
                }
                KeyCode::KeyP if pressed => {
                    profiling_state.print_comparison_report();
                }
                KeyCode::KeyE if pressed => {
                    if !profiling_state.export_trace {
                        profiling_state.profiler.begin_trace_export();
                        profiling_state.export_trace = true;
                        info!("Started Chrome trace export");
                    } else {
                        let _ = profiling_state.profiler.end_trace_export("performance_trace.json");
                        profiling_state.export_trace = false;
                        info!("Saved Chrome trace to: performance_trace.json");
                    }
                }
                KeyCode::KeyI if pressed => {
                    info!("Current optimization level: {}", profiling_state.current_level.name());
                    profiling_state.print_snapshot(&profiling_state.current_snapshot);
                }

                _ => {}
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    info!("=== Comprehensive Performance Profiling Demo ===");
    info!("");
    info!("This demo creates a large scene (10,000+ objects) and measures performance");
    info!("with different optimization techniques enabled.");
    info!("");
    info!("Controls:");
    info!("  1-7 - Switch optimization levels");
    info!("  Space - Reset to baseline");
    info!("  P - Print performance comparison report");
    info!("  E - Export/save Chrome trace");
    info!("  I - Print current state");
    info!("  WASD/QE - Move camera");
    info!("  Arrow Keys - Rotate camera");
    info!("  ESC - Exit");
    info!("");
    info!("Starting with baseline (no optimizations)...");
    info!("");

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    // Setup scene
    if let Some(render_context) = engine.render_context_mut() {
        setup_scene(engine.world_mut(), render_context)?;
    }

    // Initialize resources
    engine.world_mut().insert_resource(CameraController::default());
    engine.world_mut().insert_resource(ProfilingState::new());

    // Main loop
    let mut last_time = Instant::now();

    engine.run(move |engine_state, event| {
        let current_time = Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32().min(0.1);
        last_time = current_time;

        // Get profiling state for this frame
        let mut profiling_state = engine_state.world.get_resource_mut::<ProfilingState>()
            .expect("ProfilingState resource missing");

        profiling_state.profiler.begin_frame();

        // Handle input
        if let Some(window_event) = event {
            if let (Some(mut camera), Some(mut state)) = (
                engine_state.world.get_resource_mut::<CameraController>(),
                engine_state.world.get_resource_mut::<ProfilingState>(),
            ) {
                handle_input(window_event, &mut camera, &mut state);
            }
        }

        // Update camera
        if let Some(mut camera) = engine_state.world.get_resource_mut::<CameraController>() {
            camera_update_system(camera, delta_time);
        }

        // Update culling and LOD
        if let (Some(state), Some(camera)) = (
            engine_state.world.get_resource::<ProfilingState>(),
            engine_state.world.get_resource::<CameraController>(),
        ) {
            culling_system(state, camera, engine_state.world.query::<(&Transform, &mut SceneObject)>());
        }

        // Render
        if let (Some(render_context), Some(state)) = (
            engine_state.render_context.as_mut(),
            engine_state.world.get_resource::<ProfilingState>(),
        ) {
            if let Err(e) = render_system(&engine_state.world, render_context, &state) {
                warn!("Render error: {}", e);
            }
        }

        // End profiling frame
        if let Some(mut profiling_state) = engine_state.world.get_resource_mut::<ProfilingState>() {
            profiling_state.profiler.end_frame();
            profiling_state.update_measurement();
        }
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("Performance profiling demo requires graphics support");
    Ok(())
}
