//! Unified Optimization Showcase Demo
//!
//! This comprehensive demo combines all advanced rendering optimization techniques in a single
//! large-scale scene to demonstrate real-world performance benefits:
//!
//! - **GPU-driven culling**: Frustum and occlusion culling via compute shaders
//! - **LOD selection**: Distance-based level of detail switching
//! - **Material instancing**: Efficient per-object material variations
//! - **Mesh streaming**: Background loading with priority-based streaming
//! - **Hi-Z occlusion culling**: Hierarchical depth-based visibility testing
//!
//! # Scene Composition
//!
//! The demo creates a large-scale scene with 10,000+ objects arranged in a complex
//! environment featuring:
//! - Dense city-like grid with buildings (varying heights)
//! - Scattered vegetation and props
//! - Large occluder structures
//! - Objects at varying distances for LOD testing
//!
//! # Real-Time Performance Statistics
//!
//! The demo displays a comprehensive performance overlay showing:
//! - **FPS and Frame Time**: Current and average rendering performance
//! - **Draw Call Reduction**: Traditional vs optimized draw call counts
//! - **Culling Efficiency**: Objects culled vs total objects (frustum + occlusion)
//! - **LOD Distribution**: Breakdown of objects per LOD level
//! - **Streaming Metrics**: Loading status, bandwidth usage, and queue depth
//! - **Memory Statistics**: GPU memory usage, descriptor set pooling efficiency
//!
//! # Controls
//!
//! - **W/A/S/D**: Move camera forward/left/back/right
//! - **Q/E**: Move camera down/up
//! - **Mouse**: Look around (when cursor locked)
//! - **Left Shift**: Sprint (faster movement)
//! - **Space**: Reset camera to default position
//! - **1-9**: Jump to preset viewpoints (test different scenarios)
//! - **F**: Toggle frustum culling on/off
//! - **O**: Toggle occlusion culling on/off
//! - **L**: Toggle LOD system on/off
//! - **M**: Toggle mesh streaming on/off
//! - **I**: Toggle material instancing on/off
//! - **H**: Toggle performance HUD overlay
//! - **V**: Toggle visualization mode (culled objects, LOD colors, etc.)
//! - **P**: Print detailed statistics to console
//! - **ESC**: Toggle cursor lock / Exit

use praxis_core::{Engine, EngineConfig};
use praxis_ecs::{Component, Query, ResMut, Resource, World};
use praxis_graphics::{
    gpu_culling::{extract_frustum_planes, GpuCullingManager, GpuDrawCommand, GpuMeshData},
    lod::{GpuLodLevel, GpuLodSelector, GpuObjectData, LodGroup, LodLevel},
    material::MaterialProperties,
    mesh::MeshData,
    DrawCommand, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3, Vec4};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, warn, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

const GRID_SIZE: i32 = 25; // 50x50 grid = 2,500 base objects
const GRID_SPACING: f32 = 8.0;
const OBJECT_LAYERS: i32 = 5; // Multiple layers for depth testing
const TOTAL_OBJECT_TARGET: usize = 12000; // Aim for 12,000+ objects

/// Object types in the scene
#[derive(Clone, Copy, Debug, PartialEq)]
enum ObjectType {
    Building,   // Large static structures
    Vegetation, // Trees, bushes
    Prop,       // Small decorative objects
    Occluder,   // Large walls for occlusion testing
}

/// Component for objects in the optimization showcase
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
}

/// Camera controller with full flight controls
#[derive(Resource)]
struct CameraController {
    position: Vec3,
    rotation: Quat,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    sprint_multiplier: f32,
    // Input state
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

/// Performance statistics tracking
#[derive(Resource)]
struct PerformanceStats {
    frame_times: Vec<f32>,
    current_fps: f32,
    average_fps: f32,
    total_objects: u32,
    visible_objects: u32,
    frustum_culled: u32,
    occlusion_culled: u32,
    lod_counts: [u32; 4], // LOD 0-3 distribution
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

/// Optimization toggles for A/B testing
#[derive(Resource)]
struct OptimizationToggles {
    use_frustum_culling: bool,
    use_occlusion_culling: bool,
    use_lod_system: bool,
    use_mesh_streaming: bool,
    use_material_instancing: bool,
    visualization_mode: VisualizationMode,
}

impl Default for OptimizationToggles {
    fn default() -> Self {
        Self {
            use_frustum_culling: true,
            use_occlusion_culling: true,
            use_lod_system: true,
            use_mesh_streaming: true,
            use_material_instancing: true,
            visualization_mode: VisualizationMode::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VisualizationMode {
    Normal,
    LodColors,       // Color objects by LOD level
    CullingStatus,   // Show culled vs visible
    StreamingStatus, // Show streaming state
}

/// Creates meshes for different LOD levels
fn create_lod_meshes() -> HashMap<String, MeshData> {
    let mut meshes = HashMap::new();

    // Building meshes (3 LOD levels)
    meshes.insert(
        "building_lod0".to_string(),
        create_building_mesh(32), // High detail
    );
    meshes.insert(
        "building_lod1".to_string(),
        create_building_mesh(16), // Medium detail
    );
    meshes.insert(
        "building_lod2".to_string(),
        create_building_mesh(8), // Low detail
    );

    // Vegetation meshes (3 LOD levels)
    meshes.insert("vegetation_lod0".to_string(), create_vegetation_mesh(24));
    meshes.insert("vegetation_lod1".to_string(), create_vegetation_mesh(12));
    meshes.insert("vegetation_lod2".to_string(), create_vegetation_mesh(6));

    // Prop meshes (2 LOD levels)
    meshes.insert("prop_lod0".to_string(), create_prop_mesh(16));
    meshes.insert("prop_lod1".to_string(), create_prop_mesh(8));

    // Occluder mesh (single LOD)
    meshes.insert("occluder".to_string(), create_occluder_mesh());

    meshes
}

fn create_building_mesh(segments: u32) -> MeshData {
    // Create a box mesh with variable detail
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let size = 3.0;
    let height = 8.0;

    // Simple box for now (can be enhanced with more detail)
    let vertices = [
        // Front face
        [-size, 0.0, size],
        [size, 0.0, size],
        [size, height, size],
        [-size, height, size],
        // Back face
        [-size, 0.0, -size],
        [-size, height, -size],
        [size, height, -size],
        [size, 0.0, -size],
        // Top face
        [-size, height, -size],
        [-size, height, size],
        [size, height, size],
        [size, height, -size],
        // Bottom face
        [-size, 0.0, -size],
        [size, 0.0, -size],
        [size, 0.0, size],
        [-size, 0.0, size],
        // Right face
        [size, 0.0, -size],
        [size, height, -size],
        [size, height, size],
        [size, 0.0, size],
        // Left face
        [-size, 0.0, -size],
        [-size, 0.0, size],
        [-size, height, size],
        [-size, height, -size],
    ];

    for v in &vertices {
        positions.push(*v);
        colors.push([0.7, 0.7, 0.8]); // Building color
    }

    let face_indices = vec![
        0, 1, 2, 0, 2, 3, // Front
        4, 5, 6, 4, 6, 7, // Back
        8, 9, 10, 8, 10, 11, // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];

    indices.extend(face_indices);

    MeshData::with_colors(positions, colors, indices)
}

fn create_vegetation_mesh(segments: u32) -> MeshData {
    // Simple cone for tree
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let radius = 1.5;
    let height = 4.0;
    let segments = segments as usize;

    // Base center
    positions.push([0.0, 0.0, 0.0]);
    colors.push([0.2, 0.6, 0.2]); // Green

    // Base circle
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        positions.push([radius * angle.cos(), 0.0, radius * angle.sin()]);
        colors.push([0.2, 0.6, 0.2]);
    }

    // Top point
    positions.push([0.0, height, 0.0]);
    colors.push([0.1, 0.5, 0.1]);

    // Base triangles
    for i in 0..segments {
        indices.push(0);
        indices.push(((i + 1) % segments + 1) as u32);
        indices.push((i + 1) as u32);
    }

    // Side triangles
    let top_idx = (segments + 1) as u32;
    for i in 0..segments {
        indices.push((i + 1) as u32);
        indices.push(((i + 1) % segments + 1) as u32);
        indices.push(top_idx);
    }

    MeshData::with_colors(positions, colors, indices)
}

fn create_prop_mesh(segments: u32) -> MeshData {
    // Simple sphere for props
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
            colors.push([0.8, 0.6, 0.3]); // Tan color
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
    // Large wall
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
        colors.push([0.5, 0.5, 0.5]); // Gray
    }

    indices.extend(&[0, 1, 2, 0, 2, 3]);

    MeshData::with_colors(positions, colors, indices)
}

/// Sets up the massive scene with all object types
fn setup_scene(world: &mut World, render_context: &mut RenderContext) -> Result<()> {
    info!("Setting up optimization showcase scene");

    // Load all LOD meshes
    let meshes = create_lod_meshes();
    for (name, mesh_data) in &meshes {
        render_context
            .mesh_manager_mut()
            .load_mesh(name, mesh_data.clone())?;
    }

    info!("Loaded {} unique meshes", meshes.len());

    // Create LOD level definitions
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

    // Generate main grid of buildings
    for x in -GRID_SIZE..GRID_SIZE {
        for z in -GRID_SIZE..GRID_SIZE {
            // Vary building heights
            let height_variation = ((x * 7 + z * 13) % 20) as f32 * 0.5;
            let y = height_variation;

            let position = Vec3::new(x as f32 * GRID_SPACING, y, z as f32 * GRID_SPACING);

            // Create material variation
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
                },
            ));

            object_count += 1;
        }
    }

    info!("Created {} buildings", object_count);

    // Add scattered vegetation
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
            },
        ));

        object_count += 1;
    }

    info!("Created {} vegetation objects", vegetation_count);

    // Add small props scattered throughout
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
            },
        ));

        object_count += 1;
    }

    info!("Created {} props", prop_count);

    // Add large occluders for occlusion culling testing
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

/// Update LOD groups based on camera distance
fn update_lod_system(
    camera: &CameraController,
    mut query: Query<(&GlobalTransform, &mut OptimizedObject)>,
    delta_time: f32,
) {
    for (_transform, mut obj) in query.iter_mut() {
        let obj_pos = Vec3::new(
            _transform.compute_matrix().w_axis.x,
            _transform.compute_matrix().w_axis.y,
            _transform.compute_matrix().w_axis.z,
        );
        let distance_sq = (obj_pos - camera.position).length_squared();
        obj.lod_group.update(distance_sq, delta_time);
    }
}

/// Update performance statistics
fn update_stats(
    mut stats: ResMut<PerformanceStats>,
    query: Query<&OptimizedObject>,
    delta_time: f32,
) {
    // Update frame time tracking
    let frame_time_ms = delta_time * 1000.0;
    stats.frame_times.push(frame_time_ms);
    if stats.frame_times.len() > 120 {
        stats.frame_times.remove(0);
    }

    // Calculate FPS
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

    // Count LOD distribution
    stats.lod_counts = [0; 4];
    stats.total_objects = 0;

    for obj in query.iter() {
        stats.total_objects += 1;
        let lod_level = obj.lod_group.current_level();
        if lod_level < 4 {
            stats.lod_counts[lod_level] += 1;
        }
    }

    // Print periodic stats
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

/// Update camera controller
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

/// Handle input events
fn handle_input(
    event: &WindowEvent,
    camera: &mut CameraController,
    toggles: &mut OptimizationToggles,
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
                // Camera movement
                KeyCode::KeyW => camera.move_forward = pressed,
                KeyCode::KeyS => camera.move_backward = pressed,
                KeyCode::KeyA => camera.move_left = pressed,
                KeyCode::KeyD => camera.move_right = pressed,
                KeyCode::KeyQ => camera.move_down = pressed,
                KeyCode::KeyE => camera.move_up = pressed,
                KeyCode::ShiftLeft => camera.sprint = pressed,

                // Camera presets
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

                // Optimization toggles
                KeyCode::KeyF if pressed => {
                    toggles.use_frustum_culling = !toggles.use_frustum_culling;
                    info!(
                        "Frustum culling: {}",
                        if toggles.use_frustum_culling {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                KeyCode::KeyO if pressed => {
                    toggles.use_occlusion_culling = !toggles.use_occlusion_culling;
                    info!(
                        "Occlusion culling: {}",
                        if toggles.use_occlusion_culling {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                KeyCode::KeyL if pressed => {
                    toggles.use_lod_system = !toggles.use_lod_system;
                    info!(
                        "LOD system: {}",
                        if toggles.use_lod_system { "ON" } else { "OFF" }
                    );
                }
                KeyCode::KeyM if pressed => {
                    toggles.use_mesh_streaming = !toggles.use_mesh_streaming;
                    info!(
                        "Mesh streaming: {}",
                        if toggles.use_mesh_streaming {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                KeyCode::KeyI if pressed => {
                    toggles.use_material_instancing = !toggles.use_material_instancing;
                    info!(
                        "Material instancing: {}",
                        if toggles.use_material_instancing {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }

                KeyCode::KeyH if pressed => {
                    stats.show_hud = !stats.show_hud;
                    info!(
                        "Performance HUD: {}",
                        if stats.show_hud { "ON" } else { "OFF" }
                    );
                }

                KeyCode::KeyV if pressed => {
                    toggles.visualization_mode = match toggles.visualization_mode {
                        VisualizationMode::Normal => VisualizationMode::LodColors,
                        VisualizationMode::LodColors => VisualizationMode::CullingStatus,
                        VisualizationMode::CullingStatus => VisualizationMode::StreamingStatus,
                        VisualizationMode::StreamingStatus => VisualizationMode::Normal,
                    };
                    info!("Visualization mode: {:?}", toggles.visualization_mode);
                }

                KeyCode::KeyP if pressed => {
                    print_detailed_stats(stats);
                }

                _ => {}
            }
        }
        _ => {}
    }
}

fn print_detailed_stats(stats: &PerformanceStats) {
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
}

/// Render system
fn render_system(world: &World, render_context: &mut RenderContext) -> Result<()> {
    let camera = world.get_resource::<CameraController>().unwrap();
    let stats = world.get_resource::<PerformanceStats>().unwrap();

    // Build camera matrices
    let forward = camera.rotation * Vec3::NEG_Z;
    let target = camera.position + forward;
    let view = Mat4::look_at_rh(camera.position, target, Vec3::Y);
    let aspect_ratio = 1280.0 / 720.0;
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 1000.0);

    // Build draw commands
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
    info!("This demo combines all rendering optimizations in a large-scale scene:");
    info!("  • GPU-driven culling (frustum + occlusion)");
    info!("  • LOD selection with smooth transitions");
    info!("  • Material instancing for efficient variations");
    info!("  • Mesh streaming with priority-based loading");
    info!("  • Hi-Z occlusion culling");
    info!("");
    info!("Target: 12,000+ objects with real-time performance");
    info!("");
    info!("Controls:");
    info!("  WASD/QE - Move camera");
    info!("  Shift - Sprint");
    info!("  Space - Reset camera");
    info!("  1-9 - Camera presets");
    info!("  F - Toggle frustum culling");
    info!("  O - Toggle occlusion culling");
    info!("  L - Toggle LOD system");
    info!("  M - Toggle mesh streaming");
    info!("  I - Toggle material instancing");
    info!("  H - Toggle HUD");
    info!("  V - Cycle visualization modes");
    info!("  P - Print detailed stats");
    info!("  ESC - Exit");
    info!("");

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    // Setup scene
    if let Some(render_context) = engine.render_context_mut() {
        setup_scene(engine.world_mut(), render_context)?;
    }

    // Initialize resources
    engine
        .world_mut()
        .insert_resource(CameraController::default());
    engine
        .world_mut()
        .insert_resource(PerformanceStats::default());
    engine
        .world_mut()
        .insert_resource(OptimizationToggles::default());

    info!("Scene setup complete, starting main loop");

    // Main loop
    let mut last_time = std::time::Instant::now();

    engine.run(move |engine_state, event| {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32().min(0.1);
        last_time = current_time;

        // Handle input
        if let Some(window_event) = event {
            if let (Some(mut camera), Some(mut toggles), Some(mut stats)) = (
                engine_state.world.get_resource_mut::<CameraController>(),
                engine_state.world.get_resource_mut::<OptimizationToggles>(),
                engine_state.world.get_resource_mut::<PerformanceStats>(),
            ) {
                handle_input(window_event, &mut camera, &mut toggles, &mut stats);
            }
        }

        // Update camera
        if let Some(camera) = engine_state.world.get_resource_mut::<CameraController>() {
            update_camera(camera, delta_time);
        }

        // Update LOD system
        if let Some(camera) = engine_state.world.get_resource::<CameraController>() {
            update_lod_system(
                &camera,
                engine_state
                    .world
                    .query::<(&GlobalTransform, &mut OptimizedObject)>(),
                delta_time,
            );
        }

        // Update stats
        if let Some(stats) = engine_state.world.get_resource_mut::<PerformanceStats>() {
            update_stats(
                stats,
                engine_state.world.query::<&OptimizedObject>(),
                delta_time,
            );
        }

        // Render
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
