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

use praxis_ecs::{EnvironmentProbe, Transform, World};
use praxis_graphics::{
    colored_cube_mesh, sphere_mesh, DrawCommand, MaterialProperties, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

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

struct App {
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    demo_state: DemoState,
    world: World,
    last_frame_time: Option<Instant>,
    cube_positions: Vec<(Vec3, [f32; 4])>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            render_context: None,
            demo_state: DemoState::new(),
            world: World::new(),
            last_frame_time: None,
            cube_positions: vec![
                (Vec3::new(-10.0, 3.0, -10.0), [1.0, 0.0, 0.0, 1.0]), // Red
                (Vec3::new(10.0, 3.0, -10.0), [0.0, 1.0, 0.0, 1.0]),  // Green
                (Vec3::new(-10.0, 3.0, 10.0), [0.0, 0.0, 1.0, 1.0]),  // Blue
                (Vec3::new(10.0, 3.0, 10.0), [1.0, 1.0, 0.0, 1.0]),   // Yellow
            ],
        }
    }
}

impl App {
    fn setup_scene(&mut self) -> Result<()> {
        // Load meshes
        if let Some(render_context) = &mut self.render_context {
            render_context
                .mesh_manager_mut()
                .load_mesh("cube", colored_cube_mesh())?;
            render_context
                .mesh_manager_mut()
                .load_mesh("sphere", sphere_mesh(1.0, 32, 32, [1.0, 1.0, 1.0]))?;
        }

        // Spawn environment probes
        self.world.spawn((
            Transform::from_xyz(0.0, 5.0, 0.0),
            EnvironmentProbe::new("center_probe")
                .with_resolution(512)
                .with_influence_radius(20.0)
                .with_update_every_n_frames(60),
        ));

        self.world.spawn((
            Transform::from_xyz(-15.0, 3.0, 0.0),
            EnvironmentProbe::new("left_probe")
                .with_resolution(256)
                .with_influence_radius(15.0)
                .with_update_manual(),
        ));

        self.world.spawn((
            Transform::from_xyz(15.0, 3.0, 0.0),
            EnvironmentProbe::new("right_probe")
                .with_resolution(256)
                .with_influence_radius(15.0)
                .with_update_manual(),
        ));

        // Create test scene with reflective objects
        // Ground plane
        self.world.spawn(Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::new(50.0, 50.0, 1.0),
        });

        // Metallic spheres with varying roughness
        for i in 0..5 {
            self.world
                .spawn(Transform::from_xyz(-8.0 + i as f32 * 4.0, 2.0, 0.0));
        }

        // Colored cubes for environment color
        for (pos, _color) in &self.cube_positions {
            self.world.spawn(Transform {
                translation: *pos,
                rotation: Quat::IDENTITY,
                scale: Vec3::new(2.0, 2.0, 2.0),
            });
        }

        Ok(())
    }

    fn render_frame(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        let Some(render_context) = &mut self.render_context else {
            return;
        };

        let now = Instant::now();
        let delta_time = self
            .last_frame_time
            .map(|t| (now - t).as_secs_f32())
            .unwrap_or(0.016);
        self.last_frame_time = Some(now);

        self.demo_state.update(delta_time);

        let window_size = window.inner_size();
        let aspect_ratio = window_size.width as f32 / window_size.height as f32;

        let view = self.demo_state.get_view_matrix();
        let proj = self.demo_state.get_projection_matrix(aspect_ratio);

        // Render ground plane
        let ground_cmd = DrawCommand {
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
            sphere_cmds.push(DrawCommand {
                mesh_id: "sphere".to_string(),
                model: Mat4::from_translation(Vec3::new(-8.0 + i as f32 * 4.0, 2.0, 0.0)),
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
        for (pos, color) in &self.cube_positions {
            cube_cmds.push(DrawCommand {
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
            eprintln!("Render error: {e}");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Environment Probe Demo - Praxis Engine"),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let render_context = match pollster::block_on(RenderContext::new(window.clone())) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Failed to create render context: {e}");
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window);
        self.render_context = Some(render_context);

        if let Err(e) = self.setup_scene() {
            eprintln!("Failed to setup scene: {e}");
            event_loop.exit();
            return;
        }

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

        self.last_frame_time = Some(Instant::now());

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("\nClosing demo...");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.configure_surface(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_frame();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!(
        "environment_probe_demo example requires graphics support and cannot run in headless mode"
    );
    Ok(())
}
