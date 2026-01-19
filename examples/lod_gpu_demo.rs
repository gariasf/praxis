//! GPU-driven LOD selection demo with smooth transitions and debug visualization.
//!
//! This example demonstrates the GPU-driven LOD (Level of Detail) system that uses
//! compute shaders to calculate appropriate LOD levels for objects based on their
//! distance from the camera. All LOD calculations happen in parallel on the GPU,
//! enabling efficient LOD management for large numbers of objects.
//!
//! # Features Demonstrated
//!
//! - GPU-driven LOD selection using compute shaders
//! - Multiple LOD levels per object (high, medium, low detail)
//! - Distance-based LOD switching with configurable thresholds
//! - LOD bias for forcing higher/lower detail globally
//! - Smooth LOD transitions to avoid popping artifacts
//! - Real-time debug visualization showing:
//!   - Selected LOD levels with color coding
//!   - Distance from camera
//!   - LOD transition zones
//! - Interactive camera controls for testing LOD behavior
//! - Performance statistics (LOD distribution, frame time)
//!
//! # Controls
//!
//! - **W/A/S/D**: Move camera forward/left/back/right
//! - **Q/E**: Move camera down/up
//! - **Arrow Keys**: Rotate camera
//! - **+/=**: Increase LOD bias (forces higher detail)
//! - **-/_**: Decrease LOD bias (forces lower detail)
//! - **L**: Toggle LOD system on/off (test with/without LOD)
//! - **V**: Toggle debug visualization (LOD color coding)
//! - **T**: Toggle smooth transitions on/off
//! - **Space**: Reset camera to default position
//! - **ESC**: Exit
//!
//! # LOD Distance Thresholds
//!
//! - **LOD 0 (High)**: 0-15 units (green spheres)
//! - **LOD 1 (Medium)**: 15-35 units (yellow cubes)
//! - **LOD 2 (Low)**: 35+ units (red pyramids)
//!
//! # Debug Visualization
//!
//! When enabled (V key), objects are colored based on their active LOD level:
//! - Green: High detail (LOD 0)
//! - Yellow: Medium detail (LOD 1)
//! - Red: Low detail (LOD 2)
//! - Cyan: Transitioning between levels

use praxis_core::{Engine, EngineConfig};
use praxis_ecs::{Component, Query, ResMut, Resource, World};
use praxis_graphics::{
    lod::{GpuLodLevel, GpuLodSelector, GpuObjectData, LodGroup, LodLevel},
    mesh::MeshData,
    DrawCommand, RenderCommands,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, Result};
use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Component marking an object with LOD support
#[derive(Component, Debug, Clone)]
struct LodObject {
    /// LOD group defining levels and thresholds
    lod_group: LodGroup,
    /// Index into GPU LOD data arrays
    gpu_index: usize,
    /// Color for debug visualization
    debug_color: [f32; 3],
}

/// Resource managing the GPU LOD system
#[derive(Resource)]
struct LodSystem {
    /// GPU LOD selector for compute shader-based selection
    selector: GpuLodSelector,
    /// Object data for GPU (transforms, bounding spheres, LOD metadata)
    objects: Vec<GpuObjectData>,
    /// LOD level definitions (distance thresholds, mesh IDs)
    lod_levels: Vec<GpuLodLevel>,
    /// Selected LOD indices (output from GPU)
    selected_lods: Vec<u32>,
    /// Squared distances from camera (for visualization)
    distances: Vec<f32>,
    /// Global LOD bias (-1.0 to 1.0)
    lod_bias: f32,
    /// Enable/disable LOD system
    enable_lod: bool,
    /// Enable smooth transitions between LOD levels
    enable_transitions: bool,
    /// Show debug visualization
    show_debug: bool,
}

/// Camera controller with keyboard/mouse input
#[derive(Resource)]
struct CameraController {
    position: Vec3,
    rotation: Quat,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    rotate_speed: f32,
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
            position: Vec3::new(0.0, 10.0, 50.0),
            rotation: Quat::IDENTITY,
            yaw: 0.0,
            pitch: -0.2,
            move_speed: 20.0,
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
        self.position = Vec3::new(0.0, 10.0, 50.0);
        self.yaw = 0.0;
        self.pitch = -0.2;
    }
}

/// Performance and LOD statistics
#[derive(Resource, Default)]
struct Stats {
    frame_count: u32,
    lod_counts: [u32; 3],
    total_objects: u32,
    last_print_time: std::time::Instant,
}

/// Creates a sphere mesh for LOD 0 (high detail)
fn create_sphere_mesh(radius: f32, segments: u32) -> MeshData {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for lat in 0..=segments {
        let theta = lat as f32 * std::f32::consts::PI / segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=segments {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;

            positions.push([x, y, z]);
            colors.push([0.2, 0.8, 0.3]); // Green for high detail
        }
    }

    // Generate indices
    for lat in 0..segments {
        for lon in 0..segments {
            let first = lat * (segments + 1) + lon;
            let second = first + segments + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    MeshData::with_colors(positions, colors, indices)
}

/// Creates a cube mesh for LOD 1 (medium detail)
fn create_cube_mesh(size: f32) -> MeshData {
    let h = size / 2.0;
    let positions = vec![
        // Front face
        [-h, -h, h],
        [h, -h, h],
        [h, h, h],
        [-h, h, h],
        // Back face
        [-h, -h, -h],
        [-h, h, -h],
        [h, h, -h],
        [h, -h, -h],
        // Top face
        [-h, h, -h],
        [-h, h, h],
        [h, h, h],
        [h, h, -h],
        // Bottom face
        [-h, -h, -h],
        [h, -h, -h],
        [h, -h, h],
        [-h, -h, h],
        // Right face
        [h, -h, -h],
        [h, h, -h],
        [h, h, h],
        [h, -h, h],
        // Left face
        [-h, -h, -h],
        [-h, -h, h],
        [-h, h, h],
        [-h, h, -h],
    ];

    let colors = vec![[0.9, 0.8, 0.2]; 24]; // Yellow for medium detail

    let indices = vec![
        0, 1, 2, 0, 2, 3, // Front
        4, 5, 6, 4, 6, 7, // Back
        8, 9, 10, 8, 10, 11, // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];

    MeshData::with_colors(positions, colors, indices)
}

/// Creates a pyramid mesh for LOD 2 (low detail)
fn create_pyramid_mesh(size: f32) -> MeshData {
    let h = size / 2.0;
    let positions = vec![
        // Base
        [-h, -h, -h],
        [h, -h, -h],
        [h, -h, h],
        [-h, -h, h],
        // Apex
        [0.0, h * 2.0, 0.0],
    ];

    let colors = vec![
        [0.9, 0.2, 0.2], // Red for low detail
        [0.9, 0.2, 0.2],
        [0.9, 0.2, 0.2],
        [0.9, 0.2, 0.2],
        [0.9, 0.2, 0.2],
    ];

    let indices = vec![
        // Base
        0, 2, 1, 0, 3, 2, // Sides
        0, 1, 4, 1, 2, 4, 2, 3, 4, 3, 0, 4,
    ];

    MeshData::with_colors(positions, colors, indices)
}

/// Setup the scene with LOD objects
fn setup_scene(
    world: &mut World,
    render_context: &mut praxis_graphics::RenderContext,
) -> Result<()> {
    info!("Setting up LOD scene");

    // Load meshes for different LOD levels
    let sphere_mesh = create_sphere_mesh(1.5, 16);
    let cube_mesh = create_cube_mesh(2.5);
    let pyramid_mesh = create_pyramid_mesh(3.0);

    render_context
        .mesh_manager_mut()
        .load_mesh("lod_high", sphere_mesh)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("lod_medium", cube_mesh)?;
    render_context
        .mesh_manager_mut()
        .load_mesh("lod_low", pyramid_mesh)?;

    info!("Loaded 3 LOD meshes: sphere (high), cube (medium), pyramid (low)");

    // Create LOD level definitions
    let lod_levels = vec![
        LodLevel::new("lod_high", 0.0, 15.0), // High detail: 0-15 units
        LodLevel::new("lod_medium", 15.0, 35.0), // Medium detail: 15-35 units
        LodLevel::new("lod_low", 35.0, 1000.0), // Low detail: 35+ units
    ];

    // Create a grid of objects
    const GRID_SIZE: i32 = 8;
    const SPACING: f32 = 8.0;
    let mut object_count = 0;

    for x in -GRID_SIZE..=GRID_SIZE {
        for z in -GRID_SIZE..=GRID_SIZE {
            // Vary height based on position for visual interest
            let y = ((x as f32 * 0.5).sin() + (z as f32 * 0.5).cos()) * 2.0;
            let position = Vec3::new(x as f32 * SPACING, y, z as f32 * SPACING);

            // Create LOD group for this object
            let mut lod_group = LodGroup::new(lod_levels.clone());
            lod_group.enable_transitions(true);
            lod_group.set_transition_duration(0.5); // Half-second transitions

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                LodObject {
                    lod_group,
                    gpu_index: object_count,
                    debug_color: [1.0, 1.0, 1.0], // White by default
                },
            ));

            object_count += 1;
        }
    }

    info!(
        "Created {} LOD objects in a {}x{} grid",
        object_count,
        GRID_SIZE * 2 + 1,
        GRID_SIZE * 2 + 1
    );

    Ok(())
}

/// Initialize the GPU LOD system
fn init_lod_system(render_context: &mut praxis_graphics::RenderContext) -> Result<LodSystem> {
    info!("Initializing GPU LOD selector");

    let selector = GpuLodSelector::new(
        render_context.device.clone(),
        render_context.memory_allocator().clone(),
        Arc::new(
            vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
                render_context.device.clone(),
                Default::default(),
            ),
        ),
    )?;

    Ok(LodSystem {
        selector,
        objects: Vec::new(),
        lod_levels: Vec::new(),
        selected_lods: Vec::new(),
        distances: Vec::new(),
        lod_bias: 0.0,
        enable_lod: true,
        enable_transitions: true,
        show_debug: true,
    })
}

/// Update LOD object data from ECS
fn update_lod_objects(
    mut lod_system: ResMut<LodSystem>,
    camera: ResMut<CameraController>,
    mut query: Query<(&GlobalTransform, &mut LodObject)>,
    delta_time: f32,
) {
    lod_system.objects.clear();
    lod_system.lod_levels.clear();

    let camera_position = camera.position;

    // Build GPU LOD data
    for (transform, mut lod_obj) in query.iter_mut() {
        let model = transform.compute_matrix();
        let position = Vec3::new(model.w_axis.x, model.w_axis.y, model.w_axis.z);

        // Calculate distance for CPU-side LOD group updates
        let distance_squared = (position - camera_position).length_squared();

        // Update CPU LOD group with smooth transitions
        lod_obj
            .lod_group
            .set_transition_duration(if lod_system.enable_transitions {
                0.5
            } else {
                0.0
            });
        lod_obj.lod_group.update(distance_squared, delta_time);

        // Set debug color based on current LOD level
        if lod_system.show_debug {
            let current_level = lod_obj.lod_group.current_level();
            let is_transitioning = lod_obj.lod_group.is_transitioning();

            lod_obj.debug_color = if is_transitioning {
                [0.2, 0.8, 0.8] // Cyan during transitions
            } else {
                match current_level {
                    0 => [0.2, 0.8, 0.3], // Green for high detail
                    1 => [0.9, 0.8, 0.2], // Yellow for medium detail
                    2 => [0.9, 0.2, 0.2], // Red for low detail
                    _ => [1.0, 0.0, 1.0], // Magenta for unknown
                }
            };
        } else {
            lod_obj.debug_color = [0.7, 0.7, 0.7]; // Gray when debug off
        }

        // Bounding sphere (center in model space, radius)
        let bounding_sphere = [0.0, 0.0, 0.0, 2.0];

        // Add LOD levels for this object
        let lod_offset = lod_system.lod_levels.len() as u32;
        let lod_count = lod_obj.lod_group.level_count() as u32;

        for i in 0..lod_obj.lod_group.level_count() {
            let level = lod_obj.lod_group.get_level(i);
            lod_system.lod_levels.push(GpuLodLevel {
                mesh_id: i as u32, // Use index as mesh ID
                min_distance_sq: level.min_distance_squared,
                max_distance_sq: level.max_distance_squared,
                padding: 0,
            });
        }

        // Add object data for GPU
        lod_system.objects.push(GpuObjectData::new(
            model,
            bounding_sphere,
            0, // Base mesh ID
            lod_count,
            lod_offset,
        ));
    }
}

/// Camera controller update system
fn camera_controller_system(mut camera: ResMut<CameraController>, delta_time: f32) {
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
    camera.rotation = Quat::from_rotation_y(camera.yaw) * Quat::from_rotation_x(camera.pitch);

    // Update position
    let forward = camera.rotation * Vec3::new(0.0, 0.0, -1.0);
    let right = camera.rotation * Vec3::new(1.0, 0.0, 0.0);
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

/// Input handling system
fn handle_input(event: &WindowEvent, camera: &mut CameraController, lod_system: &mut LodSystem) {
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
                // Camera movement
                KeyCode::KeyW => camera.move_forward = pressed,
                KeyCode::KeyS => camera.move_backward = pressed,
                KeyCode::KeyA => camera.move_left = pressed,
                KeyCode::KeyD => camera.move_right = pressed,
                KeyCode::KeyQ => camera.move_down = pressed,
                KeyCode::KeyE => camera.move_up = pressed,

                // Camera rotation
                KeyCode::ArrowLeft => camera.rotate_left = pressed,
                KeyCode::ArrowRight => camera.rotate_right = pressed,
                KeyCode::ArrowUp => camera.rotate_up = pressed,
                KeyCode::ArrowDown => camera.rotate_down = pressed,

                // LOD bias controls
                KeyCode::Equal | KeyCode::NumpadAdd if pressed => {
                    lod_system.lod_bias = (lod_system.lod_bias + 0.1).min(1.0);
                    info!(
                        "LOD bias increased to {:.2} (higher detail)",
                        lod_system.lod_bias
                    );
                }
                KeyCode::Minus | KeyCode::NumpadSubtract if pressed => {
                    lod_system.lod_bias = (lod_system.lod_bias - 0.1).max(-1.0);
                    info!(
                        "LOD bias decreased to {:.2} (lower detail)",
                        lod_system.lod_bias
                    );
                }

                // Toggle LOD system
                KeyCode::KeyL if pressed => {
                    lod_system.enable_lod = !lod_system.enable_lod;
                    info!(
                        "LOD system: {}",
                        if lod_system.enable_lod {
                            "ENABLED"
                        } else {
                            "DISABLED (all objects use highest detail)"
                        }
                    );
                }

                // Toggle debug visualization
                KeyCode::KeyV if pressed => {
                    lod_system.show_debug = !lod_system.show_debug;
                    info!(
                        "Debug visualization: {}",
                        if lod_system.show_debug {
                            "ENABLED (objects colored by LOD level)"
                        } else {
                            "DISABLED"
                        }
                    );
                }

                // Toggle smooth transitions
                KeyCode::KeyT if pressed => {
                    lod_system.enable_transitions = !lod_system.enable_transitions;
                    info!(
                        "Smooth transitions: {}",
                        if lod_system.enable_transitions {
                            "ENABLED"
                        } else {
                            "DISABLED (instant LOD switching)"
                        }
                    );
                }

                // Reset camera
                KeyCode::Space if pressed => {
                    camera.reset();
                    info!("Camera reset to default position");
                }

                _ => {}
            }
        }
        _ => {}
    }
}

/// Statistics and performance monitoring system
fn stats_system(mut stats: ResMut<Stats>, query: Query<&LodObject>) {
    stats.frame_count += 1;

    // Count objects per LOD level
    stats.lod_counts = [0, 0, 0];
    stats.total_objects = 0;

    for lod_obj in query.iter() {
        stats.total_objects += 1;
        let level = lod_obj.lod_group.current_level();
        if level < 3 {
            stats.lod_counts[level] += 1;
        }
    }

    // Print statistics every second
    let now = std::time::Instant::now();
    let elapsed = now.duration_since(stats.last_print_time).as_secs_f32();

    if elapsed >= 1.0 {
        let fps = stats.frame_count as f32 / elapsed;
        info!(
            "FPS: {:.1} | LOD Distribution: L0={} L1={} L2={} | Total: {}",
            fps, stats.lod_counts[0], stats.lod_counts[1], stats.lod_counts[2], stats.total_objects
        );

        stats.frame_count = 0;
        stats.last_print_time = now;
    }
}

/// Render system
fn render_system(world: &World, render_context: &mut praxis_graphics::RenderContext) -> Result<()> {
    let camera = world.get_resource::<CameraController>().unwrap();
    let lod_system = world.get_resource::<LodSystem>().unwrap();

    // Build view and projection matrices
    let forward = camera.rotation * Vec3::new(0.0, 0.0, -1.0);
    let target = camera.position + forward;
    let view = Mat4::look_at_rh(camera.position, target, Vec3::Y);
    let aspect_ratio = 1280.0 / 720.0;
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 1000.0);

    // Build draw commands
    let mut draw_commands = Vec::new();
    let query = world.query::<(&GlobalTransform, &LodObject)>();

    for (_entity, (transform, lod_obj)) in query.iter() {
        // Get meshes to render (may be multiple during transitions)
        let render_meshes = lod_obj.lod_group.get_render_meshes();

        for (mesh_id, alpha) in render_meshes {
            draw_commands.push(DrawCommand {
                mesh_id: mesh_id.to_string(),
                model: transform.compute_matrix(),
                texture_name: None,
                material_properties: None,
                material_instance_id: None,
                bone_matrices: None,
            });
        }
    }

    let render_commands = RenderCommands {
        view,
        proj: projection,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&render_commands)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    info!("=== GPU-Driven LOD Selection Demo ===");
    info!("");
    info!("This demo demonstrates GPU-accelerated LOD selection with smooth transitions.");
    info!("Move the camera around to see LOD levels change based on distance.");
    info!("");
    info!("LOD Distance Thresholds:");
    info!("  LOD 0 (High):   0-15 units  (green spheres)");
    info!("  LOD 1 (Medium): 15-35 units (yellow cubes)");
    info!("  LOD 2 (Low):    35+ units   (red pyramids)");
    info!("");
    info!("Controls:");
    info!("  W/A/S/D        - Move camera (forward/left/back/right)");
    info!("  Q/E            - Move camera down/up");
    info!("  Arrow Keys     - Rotate camera");
    info!("  +/=            - Increase LOD bias (force higher detail)");
    info!("  -/_            - Decrease LOD bias (force lower detail)");
    info!("  L              - Toggle LOD system on/off");
    info!("  V              - Toggle debug visualization (LOD color coding)");
    info!("  T              - Toggle smooth transitions");
    info!("  Space          - Reset camera");
    info!("  ESC            - Exit");
    info!("");

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    // Setup scene
    if let Some(render_context) = engine.render_context_mut() {
        setup_scene(engine.world_mut(), render_context)?;
        let lod_system = init_lod_system(render_context)?;
        engine.world_mut().insert_resource(lod_system);
    }

    // Initialize camera and stats
    engine
        .world_mut()
        .insert_resource(CameraController::default());
    engine.world_mut().insert_resource(Stats {
        last_print_time: std::time::Instant::now(),
        ..Default::default()
    });

    // Main loop
    let mut last_time = std::time::Instant::now();

    engine.run(move |engine_state, event| {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32().min(0.1); // Cap at 100ms
        last_time = current_time;

        // Handle input
        if let Some(window_event) = event {
            if let (Some(mut camera), Some(mut lod_system)) = (
                engine_state.world.get_resource_mut::<CameraController>(),
                engine_state.world.get_resource_mut::<LodSystem>(),
            ) {
                handle_input(window_event, &mut camera, &mut lod_system);
            }
        }

        // Update camera
        if let Some(mut camera) = engine_state.world.get_resource_mut::<CameraController>() {
            camera_controller_system(camera, delta_time);
        }

        // Update LOD objects
        if let (Some(lod_system), Some(camera)) = (
            engine_state.world.get_resource_mut::<LodSystem>(),
            engine_state.world.get_resource::<CameraController>(),
        ) {
            update_lod_objects(
                lod_system,
                camera,
                engine_state
                    .world
                    .query::<(&GlobalTransform, &mut LodObject)>(),
                delta_time,
            );
        }

        // Update statistics
        if let Some(stats) = engine_state.world.get_resource_mut::<Stats>() {
            stats_system(stats, engine_state.world.query::<&LodObject>());
        }

        // Render
        if let Some(render_context) = engine_state.render_context.as_mut() {
            if let Err(e) = render_system(&engine_state.world, render_context) {
                eprintln!("Render error: {}", e);
            }
        }
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("lod_gpu_demo requires graphics support and cannot run in headless mode");
    Ok(())
}
