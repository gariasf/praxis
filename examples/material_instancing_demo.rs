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
//! - **Mouse**: Look around (when cursor locked)
//! - **W/A/S/D**: Move camera
//! - **Space**: Move camera up
//! - **Left Ctrl**: Move camera down
//! - **Left Shift**: Sprint
//! - **ESC**: Toggle cursor lock / Exit
//!
//! # Usage
//!
//! ```bash
//! cargo run --example material_instancing_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    colored_cube_mesh, DrawCommand, MaterialProperties, RenderCommands, RenderContext,
};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const NUM_INSTANCES: usize = 100;

/// Demo state containing scene data and camera controller.
struct DemoState {
    base_material_id: String,
    instance_ids: Vec<String>,
}

impl DemoState {
    fn new() -> Self {
        Self {
            base_material_id: String::from("metal_base"),
            instance_ids: Vec::new(),
        }
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    demo_state: Option<DemoState>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
    frame_count: usize,
    stats_last_printed: Instant,
}

impl App {
    fn new() -> Self {
        let mut input_map = InputMap::default();
        input_map.bind_key(&Action::new("forward"), winit::keyboard::KeyCode::KeyW);
        input_map.bind_key(&Action::new("backward"), winit::keyboard::KeyCode::KeyS);
        input_map.bind_key(&Action::new("left"), winit::keyboard::KeyCode::KeyA);
        input_map.bind_key(&Action::new("right"), winit::keyboard::KeyCode::KeyD);
        input_map.bind_key(&Action::new("up"), winit::keyboard::KeyCode::Space);
        input_map.bind_key(&Action::new("down"), winit::keyboard::KeyCode::ControlLeft);
        input_map.bind_key(&Action::new("sprint"), winit::keyboard::KeyCode::ShiftLeft);

        Self {
            window: None,
            world: None,
            render_context: None,
            demo_state: None,
            cursor_locked: false,
            last_frame_time: None,
            camera_controller: CameraController {
                move_speed: 8.0,
                ..CameraController::default()
            },
            input_state: InputState::default(),
            input_map,
            frame_count: 0,
            stats_last_printed: Instant::now(),
        }
    }

    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, DemoState, praxis_ecs::Entity)> {
        info!("Setting up material instancing demo scene");

        let mut render_context = RenderContext::new(window.clone()).await?;
        let mut demo_state = DemoState::new();

        // Initialize scene with base material and instances
        Self::initialize_scene(&mut render_context, &mut demo_state)?;

        let mut world = World::new();
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 15.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        Ok((world, render_context, demo_state, camera_entity))
    }

    /// Initialize the scene with base material and material instances.
    fn initialize_scene(
        render_context: &mut RenderContext,
        demo_state: &mut DemoState,
    ) -> Result<()> {
        info!("Initializing material instancing demo scene");

        // Load mesh
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", colored_cube_mesh())?;

        // Create a default white texture for the base material
        let white_texture = render_context
            .texture_manager()
            .get_texture("_default_white")
            .ok_or_else(|| praxis_utils::eyre::eyre!("Default white texture not found"))?;

        // Create base material with shared textures
        render_context
            .material_manager_mut()
            .create_material(&demo_state.base_material_id, white_texture.clone());

        info!("Created base material: '{}'", demo_state.base_material_id);

        // Create material instances with different colors
        info!(
            "Creating {} material instances with color variants",
            NUM_INSTANCES
        );

        for i in 0..NUM_INSTANCES {
            let instance_id = format!("color_variant_{}", i);

            // Generate unique color based on index using HSV to RGB conversion
            let hue = (i as f32 / NUM_INSTANCES as f32) * 360.0;
            let color = hsv_to_rgb(hue, 0.8, 0.9);

            // Vary metallic and roughness properties
            let metallic = 0.7 + (i as f32 / NUM_INSTANCES as f32) * 0.3;
            let roughness = 0.1 + (i as f32 / NUM_INSTANCES as f32) * 0.4;

            // Create instance with property overrides
            render_context
                .create_material_instance(&instance_id, &demo_state.base_material_id)?
                .override_properties(
                    MaterialProperties::new()
                        .with_base_color([color.0, color.1, color.2, 1.0])
                        .with_metallic(metallic)
                        .with_roughness(roughness),
                );

            demo_state.instance_ids.push(instance_id);
        }

        // Print instancing statistics
        let stats = render_context.material_instance_stats();
        info!("Material Instancing Statistics:");
        info!("  Total instances: {}", stats.total_instances);
        info!("  Unique base materials: {}", stats.unique_base_materials);
        info!(
            "  Instances with overrides: {}",
            stats.instances_with_overrides
        );
        info!(
            "  Avg instances per base: {:.2}",
            stats.avg_instances_per_base
        );

        info!("Scene initialization complete");
        Ok(())
    }

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();
        let demo_state = self.demo_state.as_ref().unwrap();

        // Build draw commands for all instances
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

        // Get camera matrices
        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        // Render the scene
        let render_commands = RenderCommands {
            view: matrices_copy.view,
            proj: matrices_copy.projection,
            draw_commands: &draw_commands,
            lighting: None,
        };

        render_context.render(&render_commands)?;

        Ok(())
    }

    fn update_camera(&mut self) {
        if self.world.is_none() {
            return;
        }

        let world = self.world.as_mut().unwrap();
        let camera_entity = self.camera_controller.camera_entity.unwrap();

        let mut velocity = Vec3::ZERO;

        if self
            .input_map
            .is_action_pressed(&Action::new("forward"), &self.input_state)
        {
            velocity.z -= 1.0;
        }
        if self
            .input_map
            .is_action_pressed(&Action::new("backward"), &self.input_state)
        {
            velocity.z += 1.0;
        }
        if self
            .input_map
            .is_action_pressed(&Action::new("left"), &self.input_state)
        {
            velocity.x -= 1.0;
        }
        if self
            .input_map
            .is_action_pressed(&Action::new("right"), &self.input_state)
        {
            velocity.x += 1.0;
        }
        if self
            .input_map
            .is_action_pressed(&Action::new("up"), &self.input_state)
        {
            velocity.y += 1.0;
        }
        if self
            .input_map
            .is_action_pressed(&Action::new("down"), &self.input_state)
        {
            velocity.y -= 1.0;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize();
        }

        let mut speed = self.camera_controller.move_speed;
        if self
            .input_map
            .is_action_pressed(&Action::new("sprint"), &self.input_state)
        {
            speed *= self.camera_controller.sprint_multiplier;
        }

        let dt = 1.0 / 60.0;

        if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(camera_entity) {
            transform.rotation = self.camera_controller.get_rotation();

            let forward = transform.rotation * Vec3::NEG_Z;
            let right = transform.rotation * Vec3::X;
            let up = Vec3::Y;

            transform.translation += forward * velocity.z * speed * dt;
            transform.translation += right * velocity.x * speed * dt;
            transform.translation += up * velocity.y * speed * dt;
        }

        // Update camera matrices
        if let Some(transform) = world.inner().get::<Transform>(camera_entity) {
            if let Some(projection) = world
                .inner()
                .get::<praxis_ecs::PerspectiveProjection>(camera_entity)
            {
                let view = Mat4::look_at_rh(
                    transform.translation,
                    transform.translation + (transform.rotation * Vec3::NEG_Z),
                    Vec3::Y,
                );
                let proj_matrix = projection.compute_matrix();

                if let Some(mut matrices) = world
                    .inner_mut()
                    .get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
                {
                    matrices.update(view, proj_matrix);
                }
            }
        }
    }

    fn print_statistics(&mut self) {
        // Print statistics every 5 seconds
        if self.stats_last_printed.elapsed().as_secs() >= 5 {
            if let Some(render_context) = &self.render_context {
                let stats = render_context.material_instance_stats();
                let pool_size = render_context.descriptor_set_pool_size();

                info!("=== Runtime Statistics ===");
                info!("Frame: {}", self.frame_count);
                info!("Material Instances: {}", stats.total_instances);
                info!("Unique Base Materials: {}", stats.unique_base_materials);
                info!(
                    "Avg Instances per Base: {:.2}",
                    stats.avg_instances_per_base
                );
                info!("Descriptor Set Pool Size: {}", pool_size);
                info!(
                    "Memory Efficiency: {}x (vs {} traditional materials)",
                    NUM_INSTANCES / stats.unique_base_materials.max(1),
                    NUM_INSTANCES
                );
            }

            self.stats_last_printed = Instant::now();
        }
    }

    fn lock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(false);
            let _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
            self.cursor_locked = true;
        }
    }

    fn unlock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(true);
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            self.cursor_locked = false;
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing...");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Material Instancing Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, demo_state, camera_entity) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);

        println!("\n╔═══════════════════════════════════════════════════════════════════╗");
        println!("║           PRAXIS - MATERIAL INSTANCING DEMO                      ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝");
        println!("\n✨ FEATURES DEMONSTRATED:");
        println!(
            "  🎨 {} material instances sharing 1 base material",
            NUM_INSTANCES
        );
        println!("  🖼️  Per-object color, metallic, and roughness overrides");
        println!("  💾 Efficient texture sharing (no duplication)");
        println!("  📊 Real-time statistics display");
        println!("\n⌨️  CONTROLS:");
        println!("  WASD        - Move horizontally");
        println!("  Space       - Move up");
        println!("  Left Ctrl   - Move down");
        println!("  Left Shift  - Sprint");
        println!("  Mouse       - Look around");
        println!("  ESC         - Toggle cursor lock / Exit");
        println!("\n💡 WHAT TO LOOK FOR:");
        println!("  • 100 cubes in 10x10 grid, each with unique color");
        println!("  • Smooth rendering without stuttering");
        println!("  • Statistics printed every 5 seconds");
        println!("  • Descriptor set pool remains stable (efficient reuse)");
        println!("\n📈 PERFORMANCE COMPARISON:");
        println!(
            "  Traditional:  {} full materials (high memory)",
            NUM_INSTANCES
        );
        println!(
            "  Instancing:   1 base + {} overrides (minimal memory)",
            NUM_INSTANCES
        );
        println!("  Efficiency:   {}x memory reduction!", NUM_INSTANCES);
        println!();

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.demo_state = Some(demo_state);
        self.last_frame_time = Some(Instant::now());

        self.lock_cursor();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.world.is_none() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::Focused(focused) => {
                if focused && self.cursor_locked {
                    self.lock_cursor();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.configure_surface(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let _delta = if let Some(last_time) = self.last_frame_time {
                    now.duration_since(last_time)
                } else {
                    std::time::Duration::from_secs_f32(1.0 / 60.0)
                };
                self.last_frame_time = Some(now);

                self.input_state.update();
                self.update_camera();

                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {e}");
                }

                self.frame_count += 1;
                self.print_statistics();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
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
                if self.cursor_locked {
                    println!("Cursor unlocked. Press ESC again to exit.");
                    self.unlock_cursor();
                } else {
                    info!("Exiting...");
                    event_loop.exit();
                }
            }
            _ => {
                praxis_input::winit_integration::process_window_event(
                    &mut self.input_state,
                    &event,
                );
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.cursor_locked {
            return;
        }

        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera_controller
                .update_rotation(delta.0 as f32, delta.1 as f32);
        }
    }
}

/// Convert HSV color to RGB.
///
/// # Arguments
///
/// * `h` - Hue in degrees [0, 360]
/// * `s` - Saturation [0.0, 1.0]
/// * `v` - Value [0.0, 1.0]
///
/// # Returns
///
/// RGB color as (r, g, b) tuple [0.0, 1.0]
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

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("material_instancing_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}
