//! Comprehensive scene demonstration with textured OBJ meshes and camera navigation.
//!
//! This example demonstrates the complete asset loading pipeline from disk to screen:
//! - Loading OBJ mesh files from disk using praxis_assets
//! - Loading texture files from disk using praxis_graphics::TextureManager
//! - ECS-based scene management with Transform hierarchy
//! - FPS camera controller with mouse look and WASD movement
//! - Multiple textured objects in the scene
//! - Camera system integration for view/projection matrices
//!
//! Controls:
//! - WASD - Move camera horizontally
//! - Space/Left Ctrl - Move camera up/down
//! - Left Shift - Sprint (faster movement)
//! - Mouse - Look around (when cursor locked)
//! - ESC - Toggle cursor lock / Exit (when unlocked)
//!
//! Usage:
//! ```bash
//! cargo run --example comprehensive_scene_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_assets::load_obj_mesh;
use praxis_ecs::{PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{DrawCommand, RenderCommands, RenderContext};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::Vec3;
use praxis_utils::{info, Result};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
}

impl Default for App {
    fn default() -> Self {
        let mut input_map = InputMap::default();
        input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
        input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
        input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
        input_map.bind_key(&Action::new("right"), KeyCode::KeyD);
        input_map.bind_key(&Action::new("up"), KeyCode::Space);
        input_map.bind_key(&Action::new("down"), KeyCode::ControlLeft);
        input_map.bind_key(&Action::new("sprint"), KeyCode::ShiftLeft);

        Self {
            window: None,
            world: None,
            render_context: None,
            cursor_locked: false,
            last_frame_time: None,
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            input_map,
        }
    }
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, praxis_ecs::Entity)> {
        info!("Setting up comprehensive scene demo");

        let mut render_context = RenderContext::new(window.clone()).await?;

        info!("Loading assets...");

        Self::load_assets(&mut render_context)?;

        let mut world = World::new();

        Self::spawn_scene_objects(&mut world);

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 2.0, 8.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));
        info!("Created camera entity: {:?}", camera_entity);

        Ok((world, render_context, camera_entity))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading meshes from disk...");

        match load_obj_mesh(
            render_context.mesh_manager_mut(),
            "cube_obj",
            "assets/models/cube.obj",
        ) {
            Ok(_) => info!("Successfully loaded cube.obj"),
            Err(e) => {
                info!("Could not load cube.obj: {}, using procedural cube", e);
                render_context
                    .mesh_manager_mut()
                    .load_mesh("cube_obj", praxis_graphics::colored_cube_mesh())?;
            }
        }

        render_context.mesh_manager_mut().load_mesh(
            "floor_quad",
            praxis_graphics::textured_quad_mesh(10.0, [1.0, 1.0, 1.0]),
        )?;

        render_context.mesh_manager_mut().load_mesh(
            "textured_cube",
            praxis_graphics::textured_cube_mesh([1.0, 1.0, 1.0]),
        )?;

        info!("Creating procedural textures...");

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "checker",
            64,
            64,
            |x, y| {
                let checker_size = 8;
                let is_white = ((x / checker_size) + (y / checker_size)) % 2 == 0;
                if is_white {
                    [220, 220, 220, 255]
                } else {
                    [80, 80, 80, 255]
                }
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "brick",
            64,
            64,
            |x, y| {
                let brick_height = 16;
                let brick_width = 32;
                let row = y / brick_height;
                let offset = if row % 2 == 0 { 0 } else { brick_width / 2 };
                let col = (x + offset) / brick_width;

                let is_mortar_h = y % brick_height < 2;
                let is_mortar_v = (x + offset) % brick_width < 2;

                if is_mortar_h || is_mortar_v {
                    [180, 180, 180, 255]
                } else {
                    let variation = ((x + y + col * 13) % 20) as u8;
                    [160 + variation, 80 + variation / 2, 60 + variation / 3, 255]
                }
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "metal",
            64,
            64,
            |x, y| {
                let noise = ((x * 7 + y * 13) % 40) as u8;
                let base = 160 + noise;
                [base, base, base + 20, 255]
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "wood",
            64,
            64,
            |x, _y| {
                let grain = ((x as f32 * 0.3).sin() * 20.0) as i32;
                let base = 139 + grain.clamp(-20, 20);
                [base as u8, (base - 30).max(0) as u8, 19, 255]
            },
        )?;

        info!(
            "Assets loaded: {} meshes, {} textures",
            render_context.mesh_manager().mesh_count(),
            4
        );

        Ok(())
    }

    fn create_procedural_texture<F>(
        texture_manager: &mut praxis_graphics::TextureManager,
        name: &str,
        width: u32,
        height: u32,
        pixel_fn: F,
    ) -> Result<()>
    where
        F: Fn(u32, u32) -> [u8; 4],
    {
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let color = pixel_fn(x, y);
                pixels.extend_from_slice(&color);
            }
        }

        texture_manager.load_texture_from_bytes(name, &pixels, width, height)?;
        info!("Created procedural texture: {}", name);
        Ok(())
    }

    fn spawn_scene_objects(world: &mut World) {
        info!("Spawning scene objects...");

        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            praxis_ecs::MeshHandle::new("floor_quad"),
            praxis_ecs::TextureHandle::new("checker"),
        ));

        world.spawn((
            Transform::from_xyz(-3.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("textured_cube"),
            praxis_ecs::TextureHandle::new("brick"),
        ));

        world.spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("textured_cube"),
            praxis_ecs::TextureHandle::new("metal"),
        ));

        world.spawn((
            Transform::from_xyz(3.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("textured_cube"),
            praxis_ecs::TextureHandle::new("wood"),
        ));

        world.spawn((
            Transform::from_xyz(-3.0, 1.0, -5.0),
            praxis_ecs::MeshHandle::new("cube_obj"),
            praxis_ecs::TextureHandle::new("brick"),
        ));

        world.spawn((
            Transform::from_xyz(0.0, 1.0, -5.0),
            praxis_ecs::MeshHandle::new("cube_obj"),
            praxis_ecs::TextureHandle::new("metal"),
        ));

        world.spawn((
            Transform::from_xyz(3.0, 1.0, -5.0),
            praxis_ecs::MeshHandle::new("cube_obj"),
            praxis_ecs::TextureHandle::new("wood"),
        ));

        info!("Spawned 7 scene objects");
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

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();

        // Get camera matrices
        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        let mut draw_commands = Vec::new();

        // Query all renderable entities
        let mut query = world.inner_mut().query::<(
            &Transform,
            &praxis_ecs::MeshHandle,
            &praxis_ecs::TextureHandle,
        )>();

        for (transform, mesh_handle, texture_handle) in query.iter(world.inner()) {
            draw_commands.push(DrawCommand {
                mesh_id: mesh_handle.id.clone(),
                model: transform.compute_matrix(),
                texture_name: Some(texture_handle.id.clone()),
                material_properties: None,
            });
        }

        let cmds = RenderCommands {
            view: matrices_copy.view,
            proj: matrices_copy.projection,
            draw_commands: &draw_commands,
            lighting: None,
        };

        render_context.render(&cmds)?;

        Ok(())
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
                .with_title("Praxis - Comprehensive Scene Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {}", e);
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);

        println!("\n=== Praxis Comprehensive Scene Demo ===");
        println!("Demonstrating:");
        println!("  • OBJ mesh loading from disk");
        println!("  • Texture loading and procedural generation");
        println!("  • ECS-based scene management");
        println!("  • FPS camera controller with mouse look");
        println!("  • Multiple textured objects in scene");
        println!("\nControls:");
        println!("  WASD - Move camera horizontally");
        println!("  Space - Move up");
        println!("  Left Ctrl - Move down");
        println!("  Left Shift - Sprint (hold)");
        println!("  Mouse - Look around");
        println!("  ESC - Toggle cursor lock / Exit (when unlocked)");
        println!("\nCamera will start locked. Press ESC to unlock cursor.");
        println!();

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.last_frame_time = Some(Instant::now());

        self.lock_cursor();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let world = match self.world.as_mut() {
            Some(world) => world,
            None => return,
        };

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

                {
                    self.input_state.update();

                    if let Some(camera_entity) = self.camera_controller.camera_entity {
                        Self::update_camera(
                            camera_entity,
                            &self.camera_controller,
                            &self.input_state,
                            &self.input_map,
                            world,
                        );
                    }

                    // Manually update camera matrices since we're not using systems
                    if let Some(camera_entity) = self.camera_controller.camera_entity {
                        let inner = world.inner_mut();
                        if let Some(transform) = inner.get::<Transform>(camera_entity) {
                            if let Some(projection) =
                                inner.get::<praxis_ecs::PerspectiveProjection>(camera_entity)
                            {
                                let view = praxis_math::Mat4::look_at_rh(
                                    transform.translation,
                                    transform.translation
                                        + (transform.rotation * praxis_math::Vec3::NEG_Z),
                                    praxis_math::Vec3::Y,
                                );
                                let proj_matrix = projection.compute_matrix();

                                if let Some(mut matrices) =
                                    inner.get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
                                {
                                    matrices.update(view, proj_matrix);
                                }
                            }
                        }
                    }
                }

                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {}", e);
                }

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

        if let Some(window) = &self.window {
            window.request_redraw();
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

impl App {
    fn update_camera(
        camera_entity: praxis_ecs::Entity,
        camera_controller: &CameraController,
        input_state: &InputState,
        input_map: &InputMap,
        world: &mut World,
    ) {
        let mut velocity = Vec3::ZERO;

        if input_map.is_action_pressed(&Action::new("forward"), input_state) {
            velocity.z -= 1.0;
        }
        if input_map.is_action_pressed(&Action::new("backward"), input_state) {
            velocity.z += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("left"), input_state) {
            velocity.x -= 1.0;
        }
        if input_map.is_action_pressed(&Action::new("right"), input_state) {
            velocity.x += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("up"), input_state) {
            velocity.y += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("down"), input_state) {
            velocity.y -= 1.0;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize();
        }

        let mut speed = camera_controller.move_speed;
        if input_map.is_action_pressed(&Action::new("sprint"), input_state) {
            speed *= camera_controller.sprint_multiplier;
        }

        let dt = 1.0 / 60.0;

        // Get mutable access to transform
        if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(camera_entity) {
            transform.rotation = camera_controller.get_rotation();

            let forward = transform.rotation * Vec3::NEG_Z;
            let right = transform.rotation * Vec3::X;
            let up = Vec3::Y;

            transform.translation += forward * velocity.z * speed * dt;
            transform.translation += right * velocity.x * speed * dt;
            transform.translation += up * velocity.y * speed * dt;
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}
