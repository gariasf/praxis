//! Material Instancing Demo
//!
//! Demonstrates the material instancing system for efficient per-object material property
//! overrides without full material duplication. This is ideal for scenes with hundreds of
//! material variants sharing the same base textures.
//!
//! # Features Demonstrated
//!
//! - Creating a base material with shared textures
//! - Creating material instances with per-object property overrides
//! - Rendering scenes with 100+ material variants efficiently
//! - Monitoring instancing statistics
//! - Comparing memory usage: traditional vs instancing
//!
//! # Performance Benefits
//!
//! For 100 objects with 100 different colors:
//! - **Traditional**: 100 full materials (textures + properties) = high memory + setup overhead
//! - **Instancing**: 1 base material + 100 property overrides = minimal memory + instant creation
//!
//! # Controls
//!
//! - Mouse: Rotate camera
//! - W/A/S/D: Move camera
//! - Space/Shift: Move camera up/down
//! - ESC: Exit

use praxis_core::{Engine, EngineState};
use praxis_ecs::{
    components::{Camera, Transform},
    World,
};
use praxis_graphics::{
    colored_cube_mesh, DrawCommand, MaterialProperties, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use std::sync::Arc;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
};

mod common;
mod fps_camera_controller;

use fps_camera_controller::FpsCameraController;

/// Demo state containing scene data and camera controller.
struct DemoState {
    camera_controller: FpsCameraController,
    base_material_id: String,
    instance_ids: Vec<String>,
}

impl DemoState {
    fn new() -> Self {
        Self {
            camera_controller: FpsCameraController::new(
                Vec3::new(0.0, 5.0, 15.0),
                Vec3::new(0.0, 0.0, 0.0),
            ),
            base_material_id: String::from("metal_base"),
            instance_ids: Vec::new(),
        }
    }
}

/// Initialize the scene with base material and material instances.
fn initialize_scene(render_context: &mut RenderContext, demo_state: &mut DemoState) -> Result<()> {
    info!("Initializing material instancing demo scene");

    // Load mesh
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;

    // Create a default white texture for the base material
    // In a real application, this would load actual textures
    let white_texture = render_context
        .texture_manager()
        .get_texture("_default_white")
        .ok_or_else(|| praxis_utils::eyre::eyre!("Default white texture not found"))?;

    // Create base material with shared textures
    render_context
        .material_manager_mut()
        .create_material(&demo_state.base_material_id, white_texture.clone());

    info!(
        "Created base material: '{}'",
        demo_state.base_material_id
    );

    // Create 100 material instances with different colors
    // This demonstrates efficient per-object material property overrides
    let num_instances = 100;
    info!("Creating {} material instances with color variants", num_instances);

    for i in 0..num_instances {
        let instance_id = format!("color_variant_{}", i);

        // Generate unique color based on index
        let hue = (i as f32 / num_instances as f32) * 360.0;
        let color = hsv_to_rgb(hue, 0.8, 0.9);

        // Create instance with property overrides
        render_context
            .create_material_instance(&instance_id, &demo_state.base_material_id)?
            .override_properties(
                MaterialProperties::new()
                    .with_base_color([color.0, color.1, color.2, 1.0])
                    .with_metallic(0.7 + (i as f32 / num_instances as f32) * 0.3)
                    .with_roughness(0.1 + (i as f32 / num_instances as f32) * 0.4),
            );

        demo_state.instance_ids.push(instance_id);
    }

    // Print instancing statistics
    let stats = render_context.material_instance_stats();
    info!("Material Instancing Statistics:");
    info!("  Total instances: {}", stats.total_instances);
    info!("  Unique base materials: {}", stats.unique_base_materials);
    info!("  Instances with overrides: {}", stats.instances_with_overrides);
    info!("  Avg instances per base: {:.2}", stats.avg_instances_per_base);

    info!("Scene initialization complete");
    Ok(())
}

/// Convert HSV color to RGB.
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

/// Render the scene with material instances.
fn render_scene(
    render_context: &mut RenderContext,
    demo_state: &DemoState,
    camera_transform: &Transform,
) -> Result<()> {
    // Build draw commands for all instances
    // Each instance uses the same mesh but references a different material instance
    let mut draw_commands = Vec::new();

    // Arrange instances in a 10x10 grid
    let grid_size = 10;
    let spacing = 2.5;

    for (i, instance_id) in demo_state.instance_ids.iter().enumerate() {
        let x = (i % grid_size) as f32 - (grid_size as f32 / 2.0);
        let z = (i / grid_size) as f32 - (grid_size as f32 / 2.0);

        let position = Vec3::new(x * spacing, 0.0, z * spacing);
        let rotation_angle = (i as f32 * 0.1).sin() * 0.5;

        let model = Mat4::from_translation(position)
            * Mat4::from_rotation_y(rotation_angle)
            * Mat4::from_scale(Vec3::splat(0.8));

        draw_commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model,
            texture_name: None,
            material_properties: None,
            material_instance_id: Some(instance_id.clone()),
            bone_matrices: None,
        });
    }

    // Set up camera matrices
    let view = camera_transform.compute_view_matrix();
    let aspect_ratio = 1920.0 / 1080.0;
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0);

    // Render the scene
    let render_commands = RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&render_commands)?;

    Ok(())
}

fn main() -> Result<()> {
    // Initialize logging
    praxis_utils::init_logging();

    info!("Starting Material Instancing Demo");
    info!("This demo shows efficient per-object material property overrides");

    // Create window and event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = common::create_window(&event_loop, "Material Instancing Demo");
    let window = Arc::new(window);

    // Initialize engine components
    let mut world = World::default();
    let mut render_context =
        pollster::block_on(RenderContext::new(window.clone())).expect("Failed to create renderer");

    // Create demo state
    let mut demo_state = DemoState::new();

    // Initialize scene
    initialize_scene(&mut render_context, &mut demo_state)
        .expect("Failed to initialize scene");

    // Create camera entity
    let camera_entity = world.spawn((
        Camera,
        Transform::from_translation(demo_state.camera_controller.position),
    ));

    let mut engine_state = EngineState::new();

    // Run event loop
    info!("Entering render loop");
    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        info!("Close requested, exiting");
                        target.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        info!("Escape pressed, exiting");
                        target.exit();
                    }
                    WindowEvent::Resized(size) => {
                        info!("Window resized to: {}x{}", size.width, size.height);
                        render_context.handle_resize();
                    }
                    WindowEvent::RedrawRequested => {
                        // Update camera controller
                        let dt = engine_state.time.delta_seconds();
                        if let Some(mut transform) = world.get_mut::<Transform>(camera_entity) {
                            demo_state
                                .camera_controller
                                .update(&window, dt, &mut transform);

                            // Render scene
                            if let Err(e) = render_scene(&mut render_context, &demo_state, &transform) {
                                eprintln!("Render error: {}", e);
                                target.exit();
                            }
                        }

                        window.request_redraw();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");

    Ok(())
}
