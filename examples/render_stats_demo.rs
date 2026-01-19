//! Render statistics collection and visualization demo.
//!
//! This example demonstrates the render statistics system, which tracks:
//! - Total objects submitted for rendering
//! - Visible objects after culling
//! - Objects culled by frustum
//! - Objects culled by occlusion
//! - Draw calls issued to GPU
//! - Descriptor set allocations
//! - Active LOD levels
//! - Mesh streaming queue depth
//!
//! The demo creates a scene with many objects and displays real-time statistics
//! with historical graphs. It also demonstrates CSV export for external analysis.
//!
//! # Features Demonstrated
//!
//! - **Real-time Statistics**: Per-frame metrics updated live
//! - **Historical Graphs**: Rolling window of frame statistics
//! - **Statistical Analysis**: Min, max, average computations
//! - **CSV Export**: Save statistics for spreadsheet analysis
//! - **Performance Monitoring**: Track culling efficiency and draw call batching
//!
//! # Controls
//!
//! - **W/A/S/D**: Move camera
//! - **Mouse**: Look around
//! - **Space**: Export statistics to CSV
//! - **Tab**: Toggle stats display
//! - **Escape**: Exit

use praxis_core::{App, CoreStage};
use praxis_ecs::{Commands, Query, Res, ResMut, Transform};
use praxis_graphics::{
    colored_cube_mesh, DrawCommand, MaterialProperties, RenderCommands, RenderContext,
};
use praxis_input::{Input, KeyCode};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use praxis_window::WindowSettings;
use std::time::Instant;

/// Application state for the render stats demo.
struct DemoState {
    /// Camera position
    camera_position: Vec3,
    /// Camera rotation (yaw, pitch)
    camera_rotation: (f32, f32),
    /// Movement speed
    move_speed: f32,
    /// Look sensitivity
    look_sensitivity: f32,
    /// Number of objects in the scene
    object_count: usize,
    /// Grid size for object placement
    grid_size: usize,
    /// Whether to show stats overlay
    show_stats: bool,
    /// Time since last CSV export
    last_export_time: Instant,
}

impl Default for DemoState {
    fn default() -> Self {
        Self {
            camera_position: Vec3::new(0.0, 5.0, 20.0),
            camera_rotation: (0.0, 0.0),
            move_speed: 5.0,
            look_sensitivity: 0.002,
            object_count: 500,
            grid_size: 20,
            show_stats: true,
            last_export_time: Instant::now(),
        }
    }
}

/// Setup the demo scene with many objects.
fn setup_system(mut commands: Commands, mut render_context: ResMut<RenderContext>) {
    info!("Setting up render stats demo scene");

    // Load cube mesh
    let _ = render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh());

    // Create demo state resource
    commands.insert_resource(DemoState::default());

    info!("Demo scene setup complete");
}

/// Update camera based on input.
fn camera_system(
    input: Res<Input>,
    mut state: ResMut<DemoState>,
    time: Res<praxis_utils::timing::FrameTimer>,
) {
    let delta = time.delta_seconds();

    // Camera movement
    let mut movement = Vec3::ZERO;

    if input.key_pressed(KeyCode::KeyW) {
        movement.z -= 1.0;
    }
    if input.key_pressed(KeyCode::KeyS) {
        movement.z += 1.0;
    }
    if input.key_pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if input.key_pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    if input.key_pressed(KeyCode::Space) {
        movement.y += 1.0;
    }
    if input.key_pressed(KeyCode::ShiftLeft) {
        movement.y -= 1.0;
    }

    if movement.length_squared() > 0.0 {
        movement = movement.normalize();

        // Apply rotation to movement
        let (yaw, _) = state.camera_rotation;
        let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        let right = Vec3::new(forward.z, 0.0, -forward.x);

        let rotated_movement =
            right * movement.x + Vec3::new(0.0, movement.y, 0.0) + forward * movement.z;

        state.camera_position += rotated_movement * state.move_speed * delta;
    }

    // Camera rotation (simple mouse-look simulation with arrow keys)
    if input.key_pressed(KeyCode::ArrowLeft) {
        state.camera_rotation.0 -= 1.0 * delta;
    }
    if input.key_pressed(KeyCode::ArrowRight) {
        state.camera_rotation.0 += 1.0 * delta;
    }
    if input.key_pressed(KeyCode::ArrowUp) {
        state.camera_rotation.1 += 1.0 * delta;
    }
    if input.key_pressed(KeyCode::ArrowDown) {
        state.camera_rotation.1 -= 1.0 * delta;
    }

    // Clamp pitch to avoid gimbal lock
    state.camera_rotation.1 = state.camera_rotation.1.clamp(-1.5, 1.5);

    // Toggle stats display
    if input.key_just_pressed(KeyCode::Tab) {
        state.show_stats = !state.show_stats;
        info!(
            "Stats display: {}",
            if state.show_stats { "ON" } else { "OFF" }
        );
    }

    // Export stats to CSV
    if input.key_just_pressed(KeyCode::KeyE) {
        state.last_export_time = Instant::now();
    }
}

/// Render the scene with many objects.
fn render_system(
    state: Res<DemoState>,
    mut render_context: ResMut<RenderContext>,
    input: Res<Input>,
) -> Result<()> {
    // Export stats on 'E' key press
    if input.key_just_pressed(KeyCode::KeyE) {
        let filename = format!(
            "render_stats_{}.csv",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        render_context.export_render_stats_csv(&filename)?;
        info!("Exported render statistics to: {}", filename);
    }

    // Build view and projection matrices
    let (yaw, pitch) = state.camera_rotation;
    let forward = Vec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    );
    let target = state.camera_position + forward;
    let view = Mat4::look_at_rh(state.camera_position, target, Vec3::Y);

    let aspect_ratio = 1920.0 / 1080.0;
    let proj = Mat4::perspective_rh(std::f32::consts::PI / 4.0, aspect_ratio, 0.1, 1000.0);

    // Create draw commands for grid of cubes
    let mut draw_commands = Vec::with_capacity(state.object_count);
    let spacing = 3.0;
    let half_grid = state.grid_size as f32 * spacing / 2.0;

    for x in 0..state.grid_size {
        for z in 0..state.grid_size {
            if draw_commands.len() >= state.object_count {
                break;
            }

            let x_pos = x as f32 * spacing - half_grid;
            let z_pos = z as f32 * spacing - half_grid;
            let y_pos = ((x + z) as f32 * 0.5).sin() * 2.0;

            let position = Vec3::new(x_pos, y_pos, z_pos);
            let rotation_angle = (x + z) as f32 * 0.1;
            let rotation = Mat4::from_rotation_y(rotation_angle);
            let translation = Mat4::from_translation(position);
            let model = translation * rotation;

            // Vary materials for different objects
            let hue = (x + z) as f32 / (state.grid_size * 2) as f32;
            let color = hsv_to_rgb(hue * 360.0, 0.7, 0.9);

            draw_commands.push(DrawCommand {
                mesh_id: "cube".to_string(),
                model,
                texture_name: None,
                material_properties: Some(
                    MaterialProperties::new()
                        .with_base_color([color.0, color.1, color.2, 1.0])
                        .with_metallic(0.3)
                        .with_roughness(0.6),
                ),
                material_instance_id: None,
                bone_matrices: None,
            });
        }
    }

    let cmds = RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&cmds)?;

    Ok(())
}

/// Display render statistics overlay.
fn stats_display_system(state: Res<DemoState>, render_context: Res<RenderContext>) {
    if !state.show_stats {
        return;
    }

    let stats = render_context.render_stats();
    let history = render_context.render_stats_history();

    println!("\n=== Render Statistics (Frame {}) ===", stats.frame_number);
    println!("Total Objects:     {}", stats.total_objects);
    println!(
        "Visible Objects:   {} ({:.1}%)",
        stats.visible_objects,
        stats.visibility_ratio()
    );
    println!("Frustum Culled:    {}", stats.frustum_culled);
    println!("Occlusion Culled:  {}", stats.occlusion_culled);
    println!("Draw Calls:        {}", stats.draw_calls);
    println!("Descriptor Allocs: {}", stats.descriptor_allocations);
    println!("Culling Efficiency: {:.1}%", stats.culling_efficiency());

    println!(
        "\n=== Historical Averages ({} frames) ===",
        history.frame_count()
    );
    println!("Avg Visible:       {:.1}", history.avg_visible_objects());
    println!("Avg Draw Calls:    {:.1}", history.avg_draw_calls());
    println!(
        "Avg Culling:       {:.1}%",
        history.avg_culling_efficiency()
    );
    println!("Peak Visible:      {}", history.max_visible_objects());
    println!("Peak Draw Calls:   {}", history.max_draw_calls());

    println!("\n(Press Tab to toggle stats, E to export CSV, Esc to exit)");
}

/// Convert HSV to RGB color.
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
        _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}

fn main() -> Result<()> {
    praxis_utils::init_logging();
    info!("Starting render statistics demo");

    let mut app = App::new(WindowSettings {
        title: "Praxis Engine - Render Statistics Demo".to_string(),
        width: 1920,
        height: 1080,
        ..Default::default()
    })?;

    // Add systems
    app.add_startup_system(setup_system);
    app.add_system_to_stage(CoreStage::Update, camera_system);
    app.add_system_to_stage(CoreStage::Render, render_system);
    app.add_system_to_stage(CoreStage::PostRender, stats_display_system);

    app.run()
}
