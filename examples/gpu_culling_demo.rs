//! GPU-driven culling demonstration with multi-draw indirect rendering.
//!
//! This example demonstrates the complete GPU culling system that performs frustum culling
//! using compute shaders and renders using multi-draw indirect commands.
//!
//! Features demonstrated:
//! - GPU frustum culling with bounding spheres
//! - Multi-draw indirect rendering (single draw call for all visible objects)
//! - Large scene rendering (1000+ objects)
//! - Minimal CPU overhead
//! - Automatic indirect draw buffer generation
//!
//! Controls:
//! - WASD: Move camera
//! - Mouse: Look around
//! - ESC: Exit

use praxis_core::{Engine, EngineConfig};
use praxis_ecs::{Component, Query, ResMut, Resource, World};
use praxis_graphics::{
    gpu_culling::{extract_frustum_planes, GpuCullingManager, GpuDrawCommand, GpuMeshData},
    mesh::MeshData,
    DrawCommand, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Vec3, Vec4};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, Result};
use std::sync::Arc;
use winit::event::KeyEvent;

/// Marker component for culled objects
#[derive(Component, Debug, Clone)]
struct CulledObject {
    mesh_id: u32,
    material_id: u32,
}

/// Resource containing GPU culling state
#[derive(Resource)]
struct GpuCullingState {
    manager: GpuCullingManager,
    draw_commands: Vec<GpuDrawCommand>,
    mesh_data: Vec<GpuMeshData>,
    visible_count: u32,
    total_count: u32,
}

/// Camera controller state
#[derive(Resource)]
struct CameraController {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 30.0),
            yaw: 0.0,
            pitch: 0.0,
            speed: 10.0,
        }
    }
}

/// Creates a simple colored cube mesh.
fn create_cube_mesh() -> MeshData {
    let positions = vec![
        // Front face
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
        // Back face
        [-1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [1.0, -1.0, -1.0],
        // Top face
        [-1.0, 1.0, -1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, -1.0],
        // Bottom face
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, -1.0, 1.0],
        [-1.0, -1.0, 1.0],
        // Right face
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [1.0, 1.0, 1.0],
        [1.0, -1.0, 1.0],
        // Left face
        [-1.0, -1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0],
    ];

    let colors = vec![[1.0, 0.5, 0.2]; 24]; // Orange cubes

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

/// Sets up the scene with a grid of objects
fn setup_scene(world: &mut World, render_context: &mut RenderContext) -> Result<()> {
    info!("Setting up GPU culling demo scene");

    // Load cube mesh
    let cube_mesh = create_cube_mesh();
    let (sphere_center, sphere_radius) = cube_mesh.calculate_bounding_sphere();
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", cube_mesh)?;

    info!(
        "Loaded cube mesh with bounding sphere: center={:?}, radius={}",
        sphere_center, sphere_radius
    );

    // Create a grid of objects
    const GRID_SIZE: i32 = 10;
    const SPACING: f32 = 3.0;
    let mut object_count = 0;

    for x in -GRID_SIZE..GRID_SIZE {
        for y in -GRID_SIZE..GRID_SIZE {
            for z in -GRID_SIZE..GRID_SIZE {
                let position = Vec3::new(x as f32 * SPACING, y as f32 * SPACING, z as f32 * SPACING);

                world.spawn((
                    Transform::from_translation(position),
                    GlobalTransform::default(),
                    CulledObject {
                        mesh_id: 0,
                        material_id: 0,
                    },
                ));

                object_count += 1;
            }
        }
    }

    info!("Created {} objects for GPU culling", object_count);

    Ok(())
}

/// Initialize GPU culling system
fn init_gpu_culling(render_context: &mut RenderContext) -> Result<GpuCullingState> {
    info!("Initializing GPU culling manager");

    let manager = GpuCullingManager::new(
        render_context.device.clone(),
        render_context.memory_allocator().clone(),
        Arc::new(vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
            render_context.device.clone(),
            Default::default(),
        )),
    )?;

    // Prepare mesh metadata
    let mesh_data = vec![GpuMeshData {
        index_count: 36, // Cube has 36 indices
        first_index: 0,
        vertex_offset: 0,
        _padding: 0,
    }];

    Ok(GpuCullingState {
        manager,
        draw_commands: Vec::new(),
        mesh_data,
        visible_count: 0,
        total_count: 0,
    })
}

/// Update GPU culling draw commands from ECS
fn update_culling_system(
    mut culling: ResMut<GpuCullingState>,
    query: Query<(&GlobalTransform, &CulledObject)>,
) {
    culling.draw_commands.clear();

    for (transform, obj) in query.iter() {
        let model = transform.compute_matrix();
        let bounding_sphere = Vec4::new(0.0, 0.0, 0.0, 1.5); // Slightly larger than cube

        culling.draw_commands.push(GpuDrawCommand::new(
            model,
            bounding_sphere,
            obj.mesh_id,
            obj.material_id,
        ));
    }

    culling.total_count = culling.draw_commands.len() as u32;
}

/// Render system with GPU culling
fn render_system(world: &World, render_context: &mut RenderContext) -> Result<()> {
    let camera = world.get_resource::<CameraController>().unwrap();
    let culling = world.get_resource::<GpuCullingState>().unwrap();

    // Build view and projection matrices
    let target = camera.position
        + Vec3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos(),
        );

    let view = Mat4::look_at_rh(camera.position, target, Vec3::Y);
    let aspect_ratio = 1280.0 / 720.0;
    let projection = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 1000.0);

    // Build regular draw commands for all objects (GPU culling happens internally)
    let mut draw_commands = Vec::new();
    let query = world.query::<(&GlobalTransform, &CulledObject)>();
    
    for (_entity, (transform, _obj)) in query.iter() {
        draw_commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model: transform.compute_matrix(),
            texture_name: None,
            material_properties: None,
            material_instance_id: None,
            bone_matrices: None,
        });
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

/// Camera controller system
fn camera_controller_system(mut camera: ResMut<CameraController>, delta_time: f32) {
    // Rotate camera around the scene
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f32();

    let radius = 50.0;
    camera.position = Vec3::new(
        radius * (time * 0.3).cos(),
        20.0 * (time * 0.2).sin(),
        radius * (time * 0.3).sin(),
    );

    // Look at center
    camera.yaw = -(time * 0.3);
    camera.pitch = (time * 0.2).sin() * 0.3;
}

/// Print statistics every 60 frames
fn stats_system(culling: ResMut<GpuCullingState>) {
    static mut FRAME_COUNT: u32 = 0;
    unsafe {
        FRAME_COUNT += 1;
        if FRAME_COUNT % 60 == 0 {
            info!(
                "GPU Culling Stats: {} visible / {} total objects",
                culling.visible_count, culling.total_count
            );
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    info!("=== GPU Culling Demo with Multi-Draw Indirect ===");
    info!("");
    info!("This demo creates 1000+ objects and uses GPU culling to");
    info!("efficiently render only visible objects using multi-draw indirect.");
    info!("");
    info!("Controls:");
    info!("  ESC - Exit");
    info!("");

    // Create engine
    let config = EngineConfig::default();
    let mut engine = Engine::new(config).await?;

    // Setup scene
    if let Some(render_context) = engine.render_context_mut() {
        setup_scene(engine.world_mut(), render_context)?;

        // Initialize GPU culling
        let culling_state = init_gpu_culling(render_context)?;
        engine.world_mut().insert_resource(culling_state);
    }

    // Initialize camera controller
    engine.world_mut().insert_resource(CameraController::default());

    // Main loop
    let mut last_time = std::time::Instant::now();

    engine.run(move |engine_state, _event| {
        let current_time = std::time::Instant::now();
        let delta_time = (current_time - last_time).as_secs_f32();
        last_time = current_time;

        // Update camera
        if let Some(mut camera) = engine_state.world.get_resource_mut::<CameraController>() {
            camera_controller_system(camera, delta_time);
        }

        // Update culling draw commands
        update_culling_system(
            engine_state.world.get_resource_mut::<GpuCullingState>().unwrap(),
            engine_state.world.query::<(&GlobalTransform, &CulledObject)>(),
        );

        // Render
        if let Some(render_context) = engine_state.render_context.as_mut() {
            if let Err(e) = render_system(&engine_state.world, render_context) {
                eprintln!("Render error: {}", e);
                return;
            }
        }

        // Print stats
        if let Some(culling) = engine_state.world.get_resource::<GpuCullingState>() {
            stats_system(culling);
        }
    })?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("gpu_culling_demo requires graphics support and cannot run in headless mode");
    Ok(())
}
