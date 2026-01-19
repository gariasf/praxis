//! Unified Optimization Showcase Demo
//!
//! This comprehensive demo consolidates all optimization-related examples:
//! - Runtime configuration and A/B testing (from optimization_config_demo)
//! - Debug visualization modes (from optimization_debug_demo)
//! - Large-scale performance demonstration (original showcase)
//!
//! # Features
//!
//! ## Optimization Configuration
//! - Multi-Draw Indirect batching
//! - GPU-driven culling (frustum + occlusion)
//! - GPU LOD selection
//! - Descriptor set caching
//! - Hi-Z occlusion culling
//! - Mesh streaming
//!
//! ## Debug Visualization
//! - Wireframe bounding spheres (culling results)
//! - LOD level heat map
//! - Mesh streaming state indicators
//! - Performance HUD overlay
//!
//! ## Scene
//! - 10,000+ objects (buildings, vegetation, props, occluders)
//! - Material instancing with color variations
//! - Multi-level LOD groups
//! - Real-time performance statistics
//!
//! # Controls
//!
//! ## Camera Movement
//! - **W/A/S/D**: Move forward/left/back/right
//! - **Q/E**: Move down/up
//! - **Left Shift**: Sprint (faster movement)
//! - **Space**: Reset camera to default position
//! - **1-9**: Jump to preset viewpoints
//!
//! ## Optimization Toggles
//! - **F1**: Toggle Multi-Draw Indirect
//! - **F2**: Toggle GPU Culling
//! - **F3**: Toggle GPU LOD Selection
//! - **F4**: Toggle Descriptor Caching
//! - **F5**: Toggle Hi-Z Occlusion
//! - **F6**: Toggle Mesh Streaming
//! - **F7**: Toggle optimization panel visibility
//! - **F8**: Reset to default settings
//!
//! ## Debug Visualization
//! - **Num1**: Toggle culling debug visualization
//! - **Num2**: Toggle LOD heat map
//! - **Num3**: Toggle mesh streaming state
//! - **H**: Toggle performance HUD
//! - **V**: Cycle visualization modes
//! - **P**: Print detailed statistics to console
//!
//! ## Other
//! - **ESC**: Exit

use praxis_core::{Engine, EngineConfig};
use praxis_ecs::{Component, Query, ResMut, Resource, World};
use praxis_graphics::optimization_config::RenderingOptimizationConfig;
use praxis_graphics::{
    debug_rendering::{
        helpers, CullingDebugInfo, DebugRenderMode, DebugRenderer, LodDebugInfo,
        StreamingDebugInfo, StreamingState,
    },
    lod::{LodGroup, LodLevel},
    material::MaterialProperties,
    mesh::MeshData,
    DrawCommand, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, warn, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

const GRID_SIZE: i32 = 25;
const GRID_SPACING: f32 = 8.0;
const TOTAL_OBJECT_TARGET: usize = 12000;

#[derive(Clone, Copy, Debug, PartialEq)]
enum ObjectType {
    Building,
    Vegetation,
    Prop,
    Occluder,
}

#[derive(Component, Clone)]
struct OptimizedObject {
    object_type: ObjectType,
    lod_group: LodGroup,
    base_color: [f32; 3],
    metallic: f32,
    roughness: f32,
    material_instance_id: String,
    mesh_id: String,
    bounding_radius: f32,
    is_visible: bool,
    streaming_state: StreamingState,
    load_progress: f32,
}

#[derive(Resource)]
struct CameraController {
    position: Vec3,
    rotation: Quat,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    sprint_multiplier: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    sprint: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 30.0, 80.0),
            rotation: Quat::IDENTITY,
            yaw: 0.0,
            pitch: -0.3,
            move_speed: 30.0,
            sprint_multiplier: 3.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            sprint: false,
        }
    }
}

impl CameraController {
    fn update_rotation(&mut self) {
        self.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    fn set_preset(&mut self, preset: u32) {
        match preset {
            1 => {
                self.position = Vec3::new(0.0, 30.0, 80.0);
                self.yaw = 0.0;
                self.pitch = -0.3;
                info!("Camera preset 1: Overview");
            }
            2 => {
                self.position = Vec3::new(50.0, 10.0, 50.0);
                self.yaw = -std::f32::consts::PI / 4.0;
                self.pitch = 0.0;
                info!("Camera preset 2: Street level");
            }
            3 => {
                self.position = Vec3::new(0.0, 100.0, 0.0);
                self.yaw = 0.0;
                self.pitch = -std::f32::consts::PI / 3.0;
                info!("Camera preset 3: Top-down view");
            }
            4 => {
                self.position = Vec3::new(-80.0, 20.0, -80.0);
                self.yaw = std::f32::consts::PI / 4.0;
                self.pitch = -0.2;
                info!("Camera preset 4: Corner view");
            }
            5 => {
                self.position = Vec3::new(0.0, 5.0, 100.0);
                self.yaw = 0.0;
                self.pitch = 0.0;
                info!("Camera preset 5: Distant view (LOD test)");
            }
            6 => {
                self.position = Vec3::new(0.0, 5.0, 10.0);
                self.yaw = 0.0;
                self.pitch = 0.0;
                info!("Camera preset 6: Close-up view");
            }
            7 => {
                self.position = Vec3::new(30.0, 15.0, 30.0);
                self.yaw = -std::f32::consts::PI / 2.0;
                self.pitch = -0.1;
                info!("Camera preset 7: Dense area");
            }
            8 => {
                self.position = Vec3::new(-50.0, 50.0, 50.0);
                self.yaw = std::f32::consts::PI / 6.0;
                self.pitch = -0.5;
                info!("Camera preset 8: Elevated angle");
            }
            9 => {
                self.position = Vec3::new(0.0, 2.0, 0.0);
                self.yaw = 0.0;
                self.pitch = 0.0;
                info!("Camera preset 9: Ground level (occlusion test)");
            }
            _ => {}
        }
        self.update_rotation();
    }
}

#[derive(Resource)]
struct PerformanceStats {
    frame_times: Vec<f32>,
    current_fps: f32,
    average_fps: f32,
    total_objects: u32,
    visible_objects: u32,
    frustum_culled: u32,
    occlusion_culled: u32,
    lod_counts: [u32; 4],
    streaming_loaded: u32,
    streaming_loading: u32,
    streaming_queued: u32,
    draw_calls_traditional: u32,
    draw_calls_optimized: u32,
    last_update_time: Instant,
    show_hud: bool,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            frame_times: Vec::with_capacity(120),
            current_fps: 0.0,
            average_fps: 0.0,
            total_objects: 0,
            visible_objects: 0,
            frustum_culled: 0,
            occlusion_culled: 0,
            lod_counts: [0; 4],
            streaming_loaded: 0,
            streaming_loading: 0,
            streaming_queued: 0,
            draw_calls_traditional: 0,
            draw_calls_optimized: 0,
            last_update_time: Instant::now(),
            show_hud: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VisualizationMode {
    Normal,
    LodColors,
    CullingStatus,
    StreamingStatus,
}

#[derive(Resource)]
struct DemoState {
    debug_renderer: Option<DebugRenderer>,
    visualization_mode: VisualizationMode,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            debug_renderer: None,
            visualization_mode: VisualizationMode::Normal,
        }
    }
}

fn create_lod_meshes() -> HashMap<String, MeshData> {
    let mut meshes = HashMap::new();

    meshes.insert("building_lod0".to_string(), create_building_mesh(32));
    meshes.insert("building_lod1".to_string(), create_building_mesh(16));
    meshes.insert("building_lod2".to_string(), create_building_mesh(8));

    meshes.insert("vegetation_lod0".to_string(), create_vegetation_mesh(24));
    meshes.insert("vegetation_lod1".to_string(), create_vegetation_mesh(12));
    meshes.insert("vegetation_lod2".to_string(), create_vegetation_mesh(6));

    meshes.insert("prop_lod0".to_string(), create_prop_mesh(16));
    meshes.insert("prop_lod1".to_string(), create_prop_mesh(8));

    meshes.insert("occluder".to_string(), create_occluder_mesh());

    meshes
}

fn create_building_mesh(segments: u32) -> MeshData {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let size = 3.0;
    let height = 8.0;

    let vertices = [
        [-size, 0.0, size],
        [size, 0.0, size],
        [size, height, size],
        [-size, height, size],
        [-size, 0.0, -size],
        [-size, height, -size],
        [size, height, -size],
        [size, 0.0, -size],
        [-size, height, -size],
        [-size, height, size],
        [size, height, size],
        [size, height, -size],
        [-size, 0.0, -size],
        [size, 0.0, -size],
        [size, 0.0, size],
        [-size, 0.0, size],
        [size, 0.0, -size],
        [size, height, -size],
        [size, height, size],
        [size, 0.0, size],
        [-size, 0.0, -size],
        [-size, 0.0, size],
        [-size, height, size],
        [-size, height, -size],
    ];

    for v in &vertices {
        positions.push(*v);
        colors.push([0.7, 0.7, 0.8]);
    }

    let face_indices = vec![
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17,
        18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];

    indices.extend(face_indices);
    MeshData::with_colors(positions, colors, indices)
}

fn create_vegetation_mesh(segments: u32) -> MeshData {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let radius = 1.5;
    let height = 4.0;
    let segments = segments as usize;

    positions.push([0.0, 0.0, 0.0]);
    colors.push([0.2, 0.6, 0.2]);

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        positions.push([radius * angle.cos(), 0.0, radius * angle.sin()]);
        colors.push([0.2, 0.6, 0.2]);
    }

    positions.push([0.0, height, 0.0]);
    colors.push([0.1, 0.5, 0.1]);

    for i in 0..segments {
        indices.push(0);
        indices.push(((i + 1) % segments + 1) as u32);
        indices.push((i + 1) as u32);
    }

    let top_idx = (segments + 1) as u32;
    for i in 0..segments {
        indices.push((i + 1) as u32);
        indices.push(((i + 1) % segments + 1) as u32);
        indices.push(top_idx);
    }

    MeshData::with_colors(positions, colors, indices)
}

fn create_prop_mesh(segments: u32) -> MeshData {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let radius = 0.8;

    for lat in 0..=segments {
        let theta = (lat as f32 / segments as f32) * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=segments {
            let phi = (lon as f32 / segments as f32) * std::f32::consts::TAU;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;

            positions.push([x, y, z]);
            colors.push([0.8, 0.6, 0.3]);
        }
    }

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

fn create_occluder_mesh() -> MeshData {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let width = 15.0;
    let height = 12.0;

    positions.push([-width, 0.0, 0.0]);
    positions.push([width, 0.0, 0.0]);
    positions.push([width, height, 0.0]);
    positions.push([-width, height, 0.0]);

    for _ in 0..4 {
        colors.push([0.5, 0.5, 0.5]);
    }

    indices.extend(&[0, 1, 2, 0, 2, 3]);
    MeshData::with_colors(positions, colors, indices)
}

fn setup_scene(world: &mut World, render_context: &mut RenderContext) -> Result<()> {
    info!("Setting up optimization showcase scene");

    let meshes = create_lod_meshes();
    for (name, mesh_data) in &meshes {
        render_context
            .mesh_manager_mut()
            .load_mesh(name, mesh_data.clone())?;
    }

    info!("Loaded {} unique meshes", meshes.len());

    let building_lods = vec![
        LodLevel::new("building_lod0", 0.0, 30.0),
        LodLevel::new("building_lod1", 30.0, 80.0),
        LodLevel::new("building_lod2", 80.0, 1000.0),
    ];

    let vegetation_lods = vec![
        LodLevel::new("vegetation_lod0", 0.0, 20.0),
        LodLevel::new("vegetation_lod1", 20.0, 60.0),
        LodLevel::new("vegetation_lod2", 60.0, 1000.0),
    ];

    let prop_lods = vec![
        LodLevel::new("prop_lod0", 0.0, 15.0),
        LodLevel::new("prop_lod1", 15.0, 1000.0),
    ];

    let mut object_count = 0;
    let mut material_instance_count = 0;

    for x in -GRID_SIZE..GRID_SIZE {
        for z in -GRID_SIZE..GRID_SIZE {
            let height_variation = ((x * 7 + z * 13) % 20) as f32 * 0.5;
            let y = height_variation;
            let position = Vec3::new(x as f32 * GRID_SPACING, y, z as f32 * GRID_SPACING);

            let hue = ((x + z) as f32 * 17.0) % 360.0;
            let (r, g, b) = hsv_to_rgb(hue, 0.3, 0.8);

            let material_instance_id = format!("building_mat_{}", material_instance_count);
            material_instance_count += 1;

            let mut lod_group = LodGroup::new(building_lods.clone());
            lod_group.enable_transitions(true);

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                OptimizedObject {
                    object_type: ObjectType::Building,
                    lod_group,
                    base_color: [r, g, b],
                    metallic: 0.1,
                    roughness: 0.8,
                    material_instance_id,
                    mesh_id: "building_lod0".to_string(),
                    bounding_radius: 5.0,
                    is_visible: true,
                    streaming_state: StreamingState::Loaded,
                    load_progress: 1.0,
                },
            ));

            object_count += 1;
        }
    }

    info!("Created {} buildings", object_count);

    let vegetation_count = 2000;
    for i in 0..vegetation_count {
        let angle = (i as f32 / vegetation_count as f32) * std::f32::consts::TAU;
        let radius = 20.0 + (i as f32 / vegetation_count as f32) * 150.0;
        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let position = Vec3::new(x, 0.0, z);

        let material_instance_id = format!("vegetation_mat_{}", material_instance_count);
        material_instance_count += 1;

        let mut lod_group = LodGroup::new(vegetation_lods.clone());
        lod_group.enable_transitions(true);

        world.spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            OptimizedObject {
                object_type: ObjectType::Vegetation,
                lod_group,
                base_color: [0.2, 0.6, 0.2],
                metallic: 0.0,
                roughness: 0.9,
                material_instance_id,
                mesh_id: "vegetation_lod0".to_string(),
                bounding_radius: 2.0,
                is_visible: true,
                streaming_state: StreamingState::Loaded,
                load_progress: 1.0,
            },
        ));

        object_count += 1;
    }

    info!("Created {} vegetation objects", vegetation_count);

    let prop_count = 3000;
    for i in 0..prop_count {
        let x = ((i * 17) % 400 - 200) as f32;
        let z = ((i * 23) % 400 - 200) as f32;
        let y = ((i * 7) % 10) as f32 * 0.5;
        let position = Vec3::new(x * 0.5, y, z * 0.5);

        let hue = (i as f32 * 137.5) % 360.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.6, 0.9);

        let material_instance_id = format!("prop_mat_{}", material_instance_count);
        material_instance_count += 1;

        let mut lod_group = LodGroup::new(prop_lods.clone());
        lod_group.enable_transitions(false);

        world.spawn((
            Transform::from_translation(position),
            GlobalTransform::default(),
            OptimizedObject {
                object_type: ObjectType::Prop,
                lod_group,
                base_color: [r, g, b],
                metallic: 0.5,
                roughness: 0.4,
                material_instance_id,
                mesh_id: "prop_lod0".to_string(),
                bounding_radius: 1.0,
                is_visible: true,
                streaming_state: StreamingState::Loaded,
                load_progress: 1.0,
            },
        ));

        object_count += 1;
    }

    info!("Created {} props", prop_count);

    let occluder_positions = vec![
        Vec3::new(0.0, 6.0, -30.0),
        Vec3::new(40.0, 6.0, 0.0),
        Vec3::new(-40.0, 6.0, 0.0),
        Vec3::new(0.0, 6.0, 40.0),
    ];

    for (idx, position) in occluder_positions.iter().enumerate() {
        let material_instance_id = format!("occluder_mat_{}", idx);

        world.spawn((
            Transform::from_translation(*position),
            GlobalTransform::default(),
            OptimizedObject {
                object_type: ObjectType::Occluder,
                lod_group: LodGroup::new(vec![LodLevel::new("occluder", 0.0, 1000.0)]),
                base_color: [0.4, 0.4, 0.4],
                metallic: 0.0,
                roughness: 1.0,
                material_instance_id,
                mesh_id: "occluder".to_string(),
                bounding_radius: 15.0,
                is_visible: true,
                streaming_state: StreamingState::Loaded,
                load_progress: 1.0,
            },
        ));

        object_count += 1;
    }

    info!("Created {} occluders", occluder_positions.len());
    info!(
        "Total scene objects: {} (target: {}+)",
        object_count, TOTAL_OBJECT_TARGET
    );
    info!(
        "Created {} unique material instances",
        material_instance_count
    );

    Ok(())
}

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
        5 | _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}

fn update_objects(
    camera: &CameraController,
    mut query: Query<(&GlobalTransform, &mut OptimizedObject)>,
    delta_time: f32,
) {
    for (transform, mut obj) in query.iter_mut() {
        let obj_pos = Vec3::new(
            transform.compute_matrix().w_axis.x,
            transform.compute_matrix().w_axis.y,
            transform.compute_matrix().w_axis.z,
        );
        let distance_sq = (obj_pos - camera.position).length_squared();
        obj.lod_group.update(distance_sq, delta_time);

        let distance = distance_sq.sqrt();
        if distance > 100.0 {
            obj.streaming_state = StreamingState::NotLoaded;
            obj.load_progress = 0.0;
        } else if distance > 80.0 {
            obj.streaming_state = StreamingState::Loading;
            obj.load_progress = ((100.0 - distance) / 20.0).clamp(0.0, 1.0);
        } else {
            obj.streaming_state = StreamingState::Loaded;
            obj.load_progress = 1.0;
        }

        obj.is_visible = distance < 120.0;
    }
}

fn update_stats(
    mut stats: ResMut<PerformanceStats>,
    query: Query<&OptimizedObject>,
    delta_time: f32,
) {
    let frame_time_ms = delta_time * 1000.0;
    stats.frame_times.push(frame_time_ms);
    if stats.frame_times.len() > 120 {
        stats.frame_times.remove(0);
    }

    if frame_time_ms > 0.0 {
        stats.current_fps = 1000.0 / frame_time_ms;
    }

    if !stats.frame_times.is_empty() {
        let avg_frame_time: f32 =
            stats.frame_times.iter().sum::<f32>() / stats.frame_times.len() as f32;
        stats.average_fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        };
    }

    stats.lod_counts = [0; 4];
    stats.total_objects = 0;
    stats.visible_objects = 0;

    for obj in query.iter() {
        stats.total_objects += 1;
        if obj.is_visible {
            stats.visible_objects += 1;
        }
        let lod_level = obj.lod_group.current_level();
        if lod_level < 4 {
            stats.lod_counts[lod_level] += 1;
        }
    }

    if stats.last_update_time.elapsed().as_secs() >= 2 {
        info!(
            "Performance: {:.1} FPS | Objects: {}/{} visible | LOD: L0={} L1={} L2={} L3={}",
            stats.current_fps,
            stats.visible_objects,
            stats.total_objects,
            stats.lod_counts[0],
            stats.lod_counts[1],
            stats.lod_counts[2],
            stats.lod_counts[3]
        );
        stats.last_update_time = Instant::now();
    }
}

fn update_camera(mut camera: ResMut<CameraController>, delta_time: f32) {
    let forward = camera.rotation * Vec3::NEG_Z;
    let right = camera.rotation * Vec3::X;
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
        let speed = if camera.sprint {
            camera.move_speed * camera.sprint_multiplier
        } else {
            camera.move_speed
        };
        camera.position += velocity * speed * delta_time;
    }
}

fn handle_input(
    event: &WindowEvent,
    camera: &mut CameraController,
    config: &mut RenderingOptimizationConfig,
    demo_state: &mut DemoState,
    stats: &mut PerformanceStats,
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
                KeyCode::KeyW => camera.move_forward = pressed,
                KeyCode::KeyS => camera.move_backward = pressed,
                KeyCode::KeyA => camera.move_left = pressed,
                KeyCode::KeyD => camera.move_right = pressed,
                KeyCode::KeyQ => camera.move_down = pressed,
                KeyCode::KeyE => camera.move_up = pressed,
                KeyCode::ShiftLeft => camera.sprint = pressed,

                KeyCode::Digit1 if pressed => camera.set_preset(1),
                KeyCode::Digit2 if pressed => camera.set_preset(2),
                KeyCode::Digit3 if pressed => camera.set_preset(3),
                KeyCode::Digit4 if pressed => camera.set_preset(4),
                KeyCode::Digit5 if pressed => camera.set_preset(5),
                KeyCode::Digit6 if pressed => camera.set_preset(6),
                KeyCode::Digit7 if pressed => camera.set_preset(7),
                KeyCode::Digit8 if pressed => camera.set_preset(8),
                KeyCode::Digit9 if pressed => camera.set_preset(9),

                KeyCode::Space if pressed => {
                    camera.position = Vec3::new(0.0, 30.0, 80.0);
                    camera.yaw = 0.0;
                    camera.pitch = -0.3;
                    camera.update_rotation();
                    info!("Camera reset");
                }

                KeyCode::F1 if pressed => {
                    config.set_multi_draw_indirect(!config.multi_draw_indirect());
                    info!(
                        "Multi-draw indirect: {}",
                        if config.multi_draw_indirect() {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                KeyCode::F2 if pressed => {
                    config.set_gpu_culling(!config.gpu_culling());
                    info!(
                        "GPU culling: {}",
                        if config.gpu_culling() { "ON" } else { "OFF" }
                    );
                }
                KeyCode::F3 if pressed => {
                    config.set_gpu_lod_selection(!config.gpu_lod_selection());
                    info!(
                        "GPU LOD selection: {}",
                        if config.gpu_lod_selection() {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                KeyCode::F4 if pressed => {
                    config.set_descriptor_caching(!config.descriptor_caching());
                    info!(
                        "Descriptor caching: {}",
                        if config.descriptor_caching() {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                KeyCode::F5 if pressed => {
                    config.set_hiz_occlusion(!config.hiz_occlusion());
                    info!(
                        "Hi-Z occlusion: {}",
                        if config.hiz_occlusion() { "ON" } else { "OFF" }
                    );
                }
                KeyCode::F6 if pressed => {
                    config.set_mesh_streaming(!config.mesh_streaming());
                    info!(
                        "Mesh streaming: {}",
                        if config.mesh_streaming() { "ON" } else { "OFF" }
                    );
                }
                KeyCode::F8 if pressed => {
                    config.reset_to_defaults();
                    info!(
                        "Reset to defaults: {}/{} optimizations enabled",
                        config.enabled_count(),
                        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
                    );
                }

                KeyCode::Numpad1 if pressed => {
                    if let Some(debug_renderer) = &mut demo_state.debug_renderer {
                        debug_renderer.toggle_mode(DebugRenderMode::CullingResults);
                        info!("Toggled culling debug visualization");
                    }
                }
                KeyCode::Numpad2 if pressed => {
                    if let Some(debug_renderer) = &mut demo_state.debug_renderer {
                        debug_renderer.toggle_mode(DebugRenderMode::LodHeatMap);
                        info!("Toggled LOD heat map");
                    }
                }
                KeyCode::Numpad3 if pressed => {
                    if let Some(debug_renderer) = &mut demo_state.debug_renderer {
                        debug_renderer.toggle_mode(DebugRenderMode::MeshStreamingState);
                        info!("Toggled streaming state visualization");
                    }
                }

                KeyCode::KeyH if pressed => {
                    stats.show_hud = !stats.show_hud;
                    info!(
                        "Performance HUD: {}",
                        if stats.show_hud { "ON" } else { "OFF" }
                    );
                }

                KeyCode::KeyV if pressed => {
                    demo_state.visualization_mode = match demo_state.visualization_mode {
                        VisualizationMode::Normal => VisualizationMode::LodColors,
                        VisualizationMode::LodColors => VisualizationMode::CullingStatus,
                        VisualizationMode::CullingStatus => VisualizationMode::StreamingStatus,
                        VisualizationMode::StreamingStatus => VisualizationMode::Normal,
                    };
                    info!("Visualization mode: {:?}", demo_state.visualization_mode);
                }

                KeyCode::KeyP if pressed => {
                    print_detailed_stats(stats, config);
                }

                _ => {}
            }
        }
        _ => {}
    }
}

fn print_detailed_stats(stats: &PerformanceStats, config: &RenderingOptimizationConfig) {
    info!("=== Detailed Performance Statistics ===");
    info!(
        "FPS: {:.1} current, {:.1} average",
        stats.current_fps, stats.average_fps
    );
    info!(
        "Objects: {} total, {} visible ({:.1}% culled)",
        stats.total_objects,
        stats.visible_objects,
        100.0 * (1.0 - stats.visible_objects as f32 / stats.total_objects as f32)
    );
    info!(
        "Culling: {} frustum, {} occlusion",
        stats.frustum_culled, stats.occlusion_culled
    );
    info!(
        "LOD Distribution: L0={} L1={} L2={} L3={}",
        stats.lod_counts[0], stats.lod_counts[1], stats.lod_counts[2], stats.lod_counts[3]
    );
    info!(
        "Streaming: {} loaded, {} loading, {} queued",
        stats.streaming_loaded, stats.streaming_loading, stats.streaming_queued
    );
    info!(
        "Draw Calls: {} traditional → {} optimized ({:.1}% reduction)",
        stats.draw_calls_traditional,
        stats.draw_calls_optimized,
        100.0
            * (1.0
                - stats.draw_calls_optimized as f32 / stats.draw_calls_traditional.max(1) as f32)
    );
    info!("=== Optimization Config ===");
    info!("{}", config.summary());
    info!(
        "Enabled: {}/{}",
        config.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );
}

fn render_system(world: &World, render_context: &mut RenderContext) -> Result<()> {
    let camera = world.get_resource::<CameraController>().unwrap();

    let forward = camera.rotation * Vec3::NEG_Z;
    let target = camera.position + forward;
    let view = Mat4::look_at_rh(camera.position, target, Vec3::Y);
    let aspect_ratio = 1280.0 / 720.0;
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 1000.0);

    let mut draw_commands = Vec::new();
    let query = world.query::<(&GlobalTransform, &OptimizedObject)>();

    for (_entity, (transform, obj)) in query.iter() {
        let current_mesh = obj.lod_group.get_render_meshes();

        for (mesh_id, _alpha) in current_mesh {
            draw_commands.push(DrawCommand {
                mesh_id: mesh_id.to_string(),
                model: transform.compute_matrix(),
                texture_name: None,
                material_properties: Some(
                    MaterialProperties::new()
                        .with_base_color([
                            obj.base_color[0],
                            obj.base_color[1],
                            obj.base_color[2],
                            1.0,
                        ])
                        .with_metallic(obj.metallic)
                        .with_roughness(obj.roughness),
                ),
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

    info!("=== Praxis Optimization Showcase Demo ===");
    info!("");
    info!("This demo combines:");
    info!("  • Runtime optimization configuration and A/B testing");
    info!("  • Debug visualization modes for culling, LOD, and streaming");
    info!("  • Large-scale scene with 12,000+ objects");
    info!("");
    info!("Camera Controls:");
    info!("  WASD/QE - Move camera");
    info!("  Shift - Sprint");
    info!("  Space - Reset camera");
    info!("  1-9 - Camera presets");
    info!("");
    info!("Optimization Toggles:");
    info!("  F1 - Multi-Draw Indirect");
    info!("  F2 - GPU Culling");
    info!("  F3 - GPU LOD Selection");
    info!("  F4 - Descriptor Caching");
    info!("  F5 - Hi-Z Occlusion");
    info!("  F6 - Mesh Streaming");
    info!("  F8 - Reset to defaults");
    info!("");
    info!("Debug Visualization:");
    info!("  Numpad 1 - Culling debug");
    info!("  Numpad 2 - LOD heat map");
    info!("  Numpad 3 - Streaming state");
    info!("  H - Toggle HUD");
    info!("  V - Cycle visualization modes");
    info!("  P - Print detailed stats");
    info!("");

    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    if let Some(render_context) = engine.render_context_mut() {
        setup_scene(engine.world_mut(), render_context)?;
    }

    engine
        .world_mut()
        .insert_resource(CameraController::default());
    engine
        .world_mut()
        .insert_resource(PerformanceStats::default());
    engine
        .world_mut()
        .insert_resource(RenderingOptimizationConfig::default());
    engine.world_mut().insert_resource(DemoState::default());

    info!("Scene setup complete, starting main loop");

    let mut last_time = std::time::Instant::now();

    engine.run(move |engine_state, event| {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32().min(0.1);
        last_time = current_time;

        if let Some(window_event) = event {
            if let (Some(mut camera), Some(mut config), Some(mut demo_state), Some(mut stats)) = (
                engine_state.world.get_resource_mut::<CameraController>(),
                engine_state
                    .world
                    .get_resource_mut::<RenderingOptimizationConfig>(),
                engine_state.world.get_resource_mut::<DemoState>(),
                engine_state.world.get_resource_mut::<PerformanceStats>(),
            ) {
                handle_input(
                    window_event,
                    &mut camera,
                    &mut config,
                    &mut demo_state,
                    &mut stats,
                );
            }
        }

        if let Some(camera) = engine_state.world.get_resource_mut::<CameraController>() {
            update_camera(camera, delta_time);
        }

        if let Some(camera) = engine_state.world.get_resource::<CameraController>() {
            update_objects(
                &camera,
                engine_state
                    .world
                    .query::<(&GlobalTransform, &mut OptimizedObject)>(),
                delta_time,
            );
        }

        if let Some(stats) = engine_state.world.get_resource_mut::<PerformanceStats>() {
            update_stats(
                stats,
                engine_state.world.query::<&OptimizedObject>(),
                delta_time,
            );
        }

        if let Some(render_context) = engine_state.render_context.as_mut() {
            if let Err(e) = render_system(&engine_state.world, render_context) {
                warn!("Render error: {}", e);
            }
        }
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!(
        "optimization_showcase_demo requires graphics support and cannot run in headless mode"
    );
    Ok(())
}
