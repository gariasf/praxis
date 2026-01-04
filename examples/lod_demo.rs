//! Demonstrates the LOD (Level of Detail) system with distance-based mesh switching.
//!
//! This example shows:
//! - Multiple LOD levels per entity
//! - Distance-based LOD selection using squared distance
//! - Smooth alpha-blended transitions between LOD levels
//! - LOD group management per entity
//!
//! Controls:
//! - W/S: Move camera forward/backward to see LOD transitions
//! - A/D: Strafe camera left/right
//! - Mouse: Look around
//! - ESC: Exit

use praxis::{
    praxis_core::Engine,
    praxis_ecs::{
        systems::{update_lod_system, DeltaTime},
        LodGroupComponent, PerspectiveCameraBundle, Query, Transform, World,
    },
    praxis_graphics::{
        colored_cube_mesh,
        lod::{LodGroup, LodLevel, LodManager},
        sphere_mesh, DrawCommand, RenderCommands, RenderContext,
    },
    praxis_input::{is_key_pressed, Key},
    praxis_math::{Mat4, Vec3},
    praxis_utils::{info, Result},
    praxis_window::WindowConfig,
};
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

fn main() -> Result<()> {
    praxis_utils::init_logger()?;
    info!("Starting LOD demo");

    let window_config = WindowConfig {
        title: "LOD System Demo".to_string(),
        width: 1280,
        height: 720,
        ..Default::default()
    };

    pollster::block_on(run(window_config))
}

async fn run(window_config: WindowConfig) -> Result<()> {
    let mut engine = Engine::new(window_config).await?;
    let mut world = World::new();

    // Initialize LOD manager
    let mut lod_manager = LodManager::new();

    // Create different mesh LOD levels
    // High detail: 10 sphere subdivisions
    let high_detail_mesh = sphere_mesh(10);
    // Medium detail: 5 sphere subdivisions
    let medium_detail_mesh = sphere_mesh(5);
    // Low detail: 2 sphere subdivisions
    let low_detail_mesh = sphere_mesh(2);
    // Very low detail: cube
    let very_low_detail_mesh = colored_cube_mesh();

    // Load meshes into graphics system
    {
        let render_context = engine.render_context_mut();
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere_high", high_detail_mesh)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere_medium", medium_detail_mesh)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere_low", low_detail_mesh)?;
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere_very_low", very_low_detail_mesh)?;
    }

    // Spawn camera
    world.spawn(PerspectiveCameraBundle::new(
        Vec3::new(0.0, 5.0, 20.0),
        70.0_f32.to_radians(),
        1280.0 / 720.0,
    ));

    // Spawn multiple entities with LOD groups at different distances
    for i in 0..10 {
        let z_pos = -10.0 - (i as f32) * 10.0;
        let x_pos = (i as f32 % 3.0 - 1.0) * 5.0;

        // Create LOD group with 4 levels
        let lod_group = LodGroup::new(vec![
            LodLevel::new("sphere_high", 0.0, 15.0),        // 0-15 units
            LodLevel::new("sphere_medium", 15.0, 35.0),     // 15-35 units
            LodLevel::new("sphere_low", 35.0, 70.0),        // 35-70 units
            LodLevel::new("sphere_very_low", 70.0, 150.0),  // 70-150 units
        ]);

        world.spawn((
            Transform::from_xyz(x_pos, 0.0, z_pos),
            LodGroupComponent::new(lod_group),
        ));
    }

    // Insert delta time resource
    world.insert_resource(DeltaTime(0.016));

    let start_time = Instant::now();
    let mut last_frame_time = start_time;
    let mut camera_pos = Vec3::new(0.0, 5.0, 20.0);
    let mut camera_rot = Vec3::ZERO;

    info!("Starting main loop");

    engine.run(move |events, window, render_context| {
        // Calculate delta time
        let now = Instant::now();
        let delta_time = (now - last_frame_time).as_secs_f32();
        last_frame_time = now;

        // Update delta time resource
        world.insert_resource(DeltaTime(delta_time));

        // Handle input
        for event in events {
            if is_key_pressed(event, Key::Escape) {
                info!("ESC pressed, exiting");
                return Ok(false);
            }
        }

        // Camera movement
        let move_speed = 10.0 * delta_time;
        if is_key_pressed(&events[0], Key::KeyW) {
            camera_pos.z -= move_speed;
        }
        if is_key_pressed(&events[0], Key::KeyS) {
            camera_pos.z += move_speed;
        }
        if is_key_pressed(&events[0], Key::KeyA) {
            camera_pos.x -= move_speed;
        }
        if is_key_pressed(&events[0], Key::KeyD) {
            camera_pos.x += move_speed;
        }

        // Update camera transform
        let mut camera_query = world.query::<(&mut Transform,)>();
        for (mut transform,) in camera_query.iter_mut(world.inner_mut()) {
            transform.translation = camera_pos;
        }

        // Update LOD system
        update_lod_system(
            world.query::<(&mut LodGroupComponent, &crate::praxis_ecs::GlobalTransform)>(),
            world.query::<(&crate::praxis_ecs::Camera, &crate::praxis_ecs::GlobalTransform)>(),
            Some(world.resource::<DeltaTime>().copied()),
        );

        // Collect draw commands based on active LOD levels
        let mut draw_commands = Vec::new();
        
        let lod_query = world.query::<(&LodGroupComponent, &Transform)>();
        for (lod_group, transform) in lod_query.iter(&world) {
            // Get meshes to render (may be multiple during transition)
            let render_meshes = lod_group.get_render_meshes();
            
            for (mesh_id, alpha) in render_meshes {
                draw_commands.push(DrawCommand {
                    mesh_id: mesh_id.to_string(),
                    model: transform.compute_matrix(),
                    texture_name: None,
                    material_properties: None,
                });
            }
        }

        // Setup view and projection matrices
        let view = Mat4::look_at_rh(
            camera_pos,
            camera_pos + Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
        );
        let proj = Mat4::perspective_rh(70.0_f32.to_radians(), 1280.0 / 720.0, 0.1, 1000.0);

        // Render
        let render_commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };

        render_context.render(&render_commands)?;

        Ok(true)
    })
}
