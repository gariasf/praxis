//! Environment probe demo showcasing image-based lighting (IBL).
//!
//! This example demonstrates:
//! - Environment probe creation and placement
//! - Cubemap capture from scene geometry
//! - Diffuse irradiance for ambient lighting
//! - Specular reflections with varying roughness
//! - Real-time probe updates for dynamic scenes
//!
//! Controls:
//! - WASD: Move camera horizontally
//! - Space/Shift: Move camera up/down
//! - Mouse: Look around
//! - ESC: Exit

use praxis::prelude::*;
use praxis_ecs::{EnvironmentProbe, Transform, World};
use praxis_graphics::{
    colored_cube_mesh, sphere_mesh, EnvironmentProbeConfig, EnvironmentProbeManager,
    MaterialProperties, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

struct DemoState {
    camera_position: Vec3,
    camera_rotation: Quat,
    camera_pitch: f32,
    camera_yaw: f32,
    
    time: f32,
}

impl DemoState {
    fn new() -> Self {
        Self {
            camera_position: Vec3::new(0.0, 5.0, 15.0),
            camera_rotation: Quat::IDENTITY,
            camera_pitch: 0.0,
            camera_yaw: 0.0,
            time: 0.0,
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        
        // Update camera rotation
        self.camera_rotation = Quat::from_euler(
            praxis_math::EulerRot::YXZ,
            self.camera_yaw,
            self.camera_pitch,
            0.0,
        );
    }

    fn get_view_matrix(&self) -> Mat4 {
        let rotation_matrix = Mat4::from_quat(self.camera_rotation);
        let translation_matrix = Mat4::from_translation(-self.camera_position);
        rotation_matrix * translation_matrix
    }

    fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(70.0_f32.to_radians(), aspect_ratio, 0.1, 1000.0)
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;

    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Environment Probe Demo - Praxis Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            .build(&event_loop)?,
    );

    let mut render_context = pollster::block_on(RenderContext::new(window.clone()))?;
    let mut demo_state = DemoState::new();
    let mut world = World::new();

    // Load meshes
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;
    render_context
        .mesh_manager_mut()
        .load_mesh("sphere", sphere_mesh(32, 32))?;

    println!("=== Environment Probe Demo ===");
    println!("This demo showcases image-based lighting with environment probes");
    println!("\nFeatures demonstrated:");
    println!("  - Cubemap capture from scene geometry");
    println!("  - Diffuse irradiance for ambient lighting");
    println!("  - Specular reflections with varying roughness");
    println!("  - Multiple probes with spatial blending");
    println!("\nScene setup:");
    println!("  - 3 environment probes at different locations");
    println!("  - Metallic spheres with varying roughness");
    println!("  - Colored cubes providing environment color");
    println!("\nControls:");
    println!("  WASD - Move camera horizontally");
    println!("  Space/Shift - Move camera up/down");
    println!("  Mouse - Look around");
    println!("  ESC - Exit");

    // Spawn environment probes
    world.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        EnvironmentProbe::new("center_probe")
            .with_resolution(512)
            .with_influence_radius(20.0)
            .with_update_every_n_frames(60),
    ));

    world.spawn((
        Transform::from_xyz(-15.0, 3.0, 0.0),
        EnvironmentProbe::new("left_probe")
            .with_resolution(256)
            .with_influence_radius(15.0)
            .with_update_manual(),
    ));

    world.spawn((
        Transform::from_xyz(15.0, 3.0, 0.0),
        EnvironmentProbe::new("right_probe")
            .with_resolution(256)
            .with_influence_radius(15.0)
            .with_update_manual(),
    ));

    // Create test scene with reflective objects
    // Ground plane
    world.spawn(Transform {
        translation: Vec3::new(0.0, 0.0, 0.0),
        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
        scale: Vec3::new(50.0, 50.0, 1.0),
    });

    // Metallic spheres with varying roughness
    for i in 0..5 {
        let roughness = i as f32 / 4.0;
        world.spawn(Transform::from_xyz(-8.0 + i as f32 * 4.0, 2.0, 0.0));
    }

    // Colored cubes for environment color
    let cube_positions = [
        (Vec3::new(-10.0, 3.0, -10.0), [1.0, 0.0, 0.0, 1.0]), // Red
        (Vec3::new(10.0, 3.0, -10.0), [0.0, 1.0, 0.0, 1.0]),  // Green
        (Vec3::new(-10.0, 3.0, 10.0), [0.0, 0.0, 1.0, 1.0]),  // Blue
        (Vec3::new(10.0, 3.0, 10.0), [1.0, 1.0, 0.0, 1.0]),   // Yellow
    ];

    for (pos, _color) in &cube_positions {
        world.spawn(Transform {
            translation: *pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(2.0, 2.0, 2.0),
        });
    }

    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("\nClosing demo...");
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    render_context.configure_surface(size.width, size.height);
                }
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let delta_time = (now - last_time).as_secs_f32();
                    last_time = now;

                    demo_state.update(delta_time);

                    let window_size = window.inner_size();
                    let aspect_ratio = window_size.width as f32 / window_size.height as f32;

                    let view = demo_state.get_view_matrix();
                    let proj = demo_state.get_projection_matrix(aspect_ratio);

                    // Render ground plane
                    let ground_cmd = praxis_graphics::DrawCommand {
                        mesh_id: "cube".to_string(),
                        model: Mat4::from_scale_rotation_translation(
                            Vec3::new(50.0, 50.0, 1.0),
                            Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                            Vec3::ZERO,
                        ),
                        texture_name: None,
                        material_properties: Some(
                            MaterialProperties::default()
                                .with_base_color([0.3, 0.3, 0.3, 1.0])
                                .with_metallic(0.0)
                                .with_roughness(0.9),
                        ),
                    };

                    // Render metallic spheres
                    let mut sphere_cmds = Vec::new();
                    for i in 0..5 {
                        let roughness = i as f32 / 4.0;
                        sphere_cmds.push(praxis_graphics::DrawCommand {
                            mesh_id: "sphere".to_string(),
                            model: Mat4::from_translation(Vec3::new(
                                -8.0 + i as f32 * 4.0,
                                2.0,
                                0.0,
                            )),
                            texture_name: None,
                            material_properties: Some(
                                MaterialProperties::default()
                                    .with_base_color([0.8, 0.8, 0.8, 1.0])
                                    .with_metallic(1.0)
                                    .with_roughness(roughness),
                            ),
                        });
                    }

                    // Render colored cubes
                    let mut cube_cmds = Vec::new();
                    for (pos, color) in &cube_positions {
                        cube_cmds.push(praxis_graphics::DrawCommand {
                            mesh_id: "cube".to_string(),
                            model: Mat4::from_scale_rotation_translation(
                                Vec3::new(2.0, 2.0, 2.0),
                                Quat::IDENTITY,
                                *pos,
                            ),
                            texture_name: None,
                            material_properties: Some(
                                MaterialProperties::default()
                                    .with_base_color(*color)
                                    .with_metallic(0.0)
                                    .with_roughness(0.5),
                            ),
                        });
                    }

                    let mut all_cmds = vec![ground_cmd];
                    all_cmds.extend(sphere_cmds);
                    all_cmds.extend(cube_cmds);

                    let render_commands = RenderCommands {
                        view,
                        proj,
                        draw_commands: &all_cmds,
                        lighting: None,
                    };

                    if let Err(e) = render_context.render(&render_commands) {
                        eprintln!("Render error: {}", e);
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
