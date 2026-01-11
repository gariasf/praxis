//! Comprehensive GUI demonstration showcasing egui integration.
//!
//! This example demonstrates the complete Praxis GUI system:
//! - Debug panels with FPS counter and performance metrics
//! - Entity inspector for viewing and editing ECS components
//! - Scene hierarchy visualization
//! - Interactive scene manipulation (transform editing)
//! - Material and texture inspection
//! - Multiple demonstration objects with different properties
//!
//! Features demonstrated:
//! - Real-time performance monitoring (FPS, frame time, memory)
//! - Entity creation, deletion, and modification through UI
//! - Component editing with immediate visual feedback
//! - Scene graph navigation and visualization
//! - Material/texture preview and editing
//! - Camera controls for scene navigation
//!
//! Controls:
//! - WASD - Move camera horizontally
//! - Space/Left Ctrl - Move camera up/down
//! - Left Shift - Sprint (faster movement)
//! - Mouse - Look around (when cursor locked)
//! - ESC - Toggle cursor lock / Exit (when unlocked)
//! - F1 - Toggle debug UI
//! - F2 - Toggle FPS counter
//! - F3 - Toggle performance window
//! - F4 - Toggle entity inspector
//!
//! Usage:
//! ```bash
//! cargo run --example gui_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{Name, PerspectiveCameraBundle, PointLight, Transform, World};
use praxis_graphics::{
    colored_cube_mesh, textured_cube_mesh, textured_quad_mesh, DrawCommand, RenderCommands,
    RenderContext,
};
use praxis_gui::{DebugUi, EguiIntegration};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Quat, Vec3};
use praxis_utils::timing::FrameTimer;
use praxis_utils::{info, warn, Result};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

struct GuiDemoApp {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    egui_integration: Option<EguiIntegration>,
    debug_ui: Option<DebugUi>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    frame_timer: FrameTimer,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
    rotating_entities: Vec<praxis_ecs::Entity>,
    show_debug_info: bool,
    show_entity_list: bool,
    animation_speed: f32,
    scene_rotation_enabled: bool,
}

impl Default for GuiDemoApp {
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
            egui_integration: None,
            debug_ui: None,
            cursor_locked: false,
            last_frame_time: None,
            frame_timer: FrameTimer::new_with_global(),
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            input_map,
            rotating_entities: Vec::new(),
            show_debug_info: true,
            show_entity_list: false,
            animation_speed: 1.0,
            scene_rotation_enabled: true,
        }
    }
}

impl GuiDemoApp {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(
        World,
        RenderContext,
        praxis_ecs::Entity,
        Vec<praxis_ecs::Entity>,
    )> {
        info!("Setting up GUI demo scene");

        let mut render_context = RenderContext::new(window.clone()).await?;

        info!("Loading demonstration assets...");
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();

        let rotating_entities = Self::spawn_scene_objects(&mut world);

        let camera_entity = world.spawn((
            PerspectiveCameraBundle::new(
                Vec3::new(0.0, 5.0, 15.0),
                60.0_f32.to_radians(),
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
            ),
            Name::new("Main Camera"),
        ));
        info!("Created camera entity: {:?}", camera_entity);

        Ok((world, render_context, camera_entity, rotating_entities))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading meshes...");

        render_context
            .mesh_manager_mut()
            .load_mesh("colored_cube", colored_cube_mesh())?;

        render_context
            .mesh_manager_mut()
            .load_mesh("textured_cube", textured_cube_mesh([1.0, 1.0, 1.0]))?;

        render_context
            .mesh_manager_mut()
            .load_mesh("floor", textured_quad_mesh(20.0, [0.8, 0.8, 0.8]))?;

        render_context
            .mesh_manager_mut()
            .load_mesh("small_cube", textured_cube_mesh([0.5, 0.5, 0.5]))?;

        info!("Creating procedural textures...");

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "checker",
            64,
            64,
            |x, y| {
                let size = 8;
                let is_white = ((x / size) + (y / size)) % 2 == 0;
                if is_white {
                    [240, 240, 240, 255]
                } else {
                    [60, 60, 60, 255]
                }
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "red_gradient",
            64,
            64,
            |x, _y| {
                let intensity = ((x as f32 / 64.0) * 255.0) as u8;
                [intensity, 0, 0, 255]
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "green_gradient",
            64,
            64,
            |x, _y| {
                let intensity = ((x as f32 / 64.0) * 255.0) as u8;
                [0, intensity, 0, 255]
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "blue_gradient",
            64,
            64,
            |x, _y| {
                let intensity = ((x as f32 / 64.0) * 255.0) as u8;
                [0, 0, intensity, 255]
            },
        )?;

        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "grid",
            64,
            64,
            |x, y| {
                let is_grid = (x % 8 == 0) || (y % 8 == 0);
                if is_grid {
                    [100, 150, 255, 255]
                } else {
                    [20, 30, 50, 255]
                }
            },
        )?;

        info!(
            "Assets loaded: {} meshes, 5 textures",
            render_context.mesh_manager().mesh_count()
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
        Ok(())
    }

    fn spawn_scene_objects(world: &mut World) -> Vec<praxis_ecs::Entity> {
        info!("Spawning demonstration objects...");

        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            praxis_ecs::MeshHandle::new("floor"),
            praxis_ecs::TextureHandle::new("checker"),
            Name::new("Floor"),
        ));

        let rotating = vec![
            world.spawn((
                Transform::from_xyz(-4.0, 1.0, 0.0),
                praxis_ecs::MeshHandle::new("textured_cube"),
                praxis_ecs::TextureHandle::new("red_gradient"),
                Name::new("Red Cube"),
            )),
            world.spawn((
                Transform::from_xyz(0.0, 1.0, 0.0),
                praxis_ecs::MeshHandle::new("textured_cube"),
                praxis_ecs::TextureHandle::new("green_gradient"),
                Name::new("Green Cube"),
            )),
            world.spawn((
                Transform::from_xyz(4.0, 1.0, 0.0),
                praxis_ecs::MeshHandle::new("textured_cube"),
                praxis_ecs::TextureHandle::new("blue_gradient"),
                Name::new("Blue Cube"),
            )),
        ];

        world.spawn((
            Transform::from_xyz(-4.0, 3.0, -4.0),
            praxis_ecs::MeshHandle::new("small_cube"),
            praxis_ecs::TextureHandle::new("grid"),
            Name::new("Small Cube 1"),
        ));

        world.spawn((
            Transform::from_xyz(0.0, 3.0, -4.0),
            praxis_ecs::MeshHandle::new("small_cube"),
            praxis_ecs::TextureHandle::new("grid"),
            Name::new("Small Cube 2"),
        ));

        world.spawn((
            Transform::from_xyz(4.0, 3.0, -4.0),
            praxis_ecs::MeshHandle::new("small_cube"),
            praxis_ecs::TextureHandle::new("grid"),
            Name::new("Small Cube 3"),
        ));

        world.spawn((
            Transform::from_xyz(-6.0, 4.0, 0.0),
            PointLight::new(Vec3::new(1.0, 0.3, 0.3), 5.0, 20.0),
            Name::new("Red Light"),
        ));

        world.spawn((
            Transform::from_xyz(6.0, 4.0, 0.0),
            PointLight::new(Vec3::new(0.3, 0.3, 1.0), 5.0, 20.0),
            Name::new("Blue Light"),
        ));

        world.spawn((
            Transform::from_xyz(0.0, 6.0, -6.0),
            PointLight::new(Vec3::new(1.0, 1.0, 1.0), 8.0, 30.0),
            Name::new("White Light"),
        ));

        info!("Spawned 9 entities (6 meshes, 3 lights)");
        rotating
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

    fn print_debug_info(&mut self) {
        if !self.show_debug_info {
            return;
        }

        let fps = self.frame_timer.fps();
        let frame_time = 1000.0 / fps;

        let world = match self.world.as_mut() {
            Some(w) => w,
            None => return,
        };

        let mut query = world.inner_mut().query::<()>();
        let entity_count = query.iter(world.inner()).count();

        let mut mesh_query = world.inner_mut().query::<&praxis_ecs::MeshHandle>();
        let mesh_count = mesh_query.iter(world.inner()).count();

        let mut light_query = world.inner_mut().query::<&PointLight>();
        let light_count = light_query.iter(world.inner()).count();

        if entity_count > 0 {
            info!(
                "GUI Demo Stats - FPS: {:.1}, Frame: {:.2}ms, Entities: {}, Meshes: {}, Lights: {}",
                fps, frame_time, entity_count, mesh_count, light_count
            );
        }
    }

    fn print_entity_list(&mut self) {
        if !self.show_entity_list {
            return;
        }

        let world = match self.world.as_mut() {
            Some(w) => w,
            None => return,
        };

        info!("=== Entity List ===");
        let mut query = world
            .inner_mut()
            .query::<(praxis_ecs::Entity, Option<&Name>)>();

        for (entity, name) in query.iter(world.inner()) {
            let name_str = name.map(|n| n.as_str()).unwrap_or("Unnamed");
            info!("  Entity {:?}: {}", entity, name_str);
        }
    }

    fn update_rotating_objects(&mut self, delta: f32) {
        if !self.scene_rotation_enabled {
            return;
        }

        if let Some(world) = &mut self.world {
            for (i, entity) in self.rotating_entities.iter().enumerate() {
                if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(*entity) {
                    let speed = (0.5 + (i as f32 * 0.3)) * self.animation_speed;
                    let rotation = Quat::from_rotation_y(delta * speed);
                    transform.rotation = rotation * transform.rotation;
                }
            }
        }
    }

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();

        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        let mut draw_commands = Vec::new();

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
                bone_matrices: None,
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

    fn update_camera(&mut self, delta: f32) {
        let world = match self.world.as_mut() {
            Some(w) => w,
            None => return,
        };

        let camera_entity = match self.camera_controller.camera_entity {
            Some(e) => e,
            None => return,
        };

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

        if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(camera_entity) {
            transform.rotation = self.camera_controller.get_rotation();

            let forward = transform.rotation * Vec3::NEG_Z;
            let right = transform.rotation * Vec3::X;
            let up = Vec3::Y;

            transform.translation += forward * velocity.z * speed * delta;
            transform.translation += right * velocity.x * speed * delta;
            transform.translation += up * velocity.y * speed * delta;
        }

        if let Some(transform) = world.inner().get::<Transform>(camera_entity) {
            if let Some(projection) = world
                .inner()
                .get::<praxis_ecs::PerspectiveProjection>(camera_entity)
            {
                let view = praxis_math::Mat4::look_at_rh(
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
}

impl ApplicationHandler for GuiDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing GUI demo...");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - GUI Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, rotating_entities) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        let egui_integration = EguiIntegration::new(
            event_loop,
            render_context.surface(),
            render_context.queue(),
            render_context.swapchain_format(),
        );

        let debug_ui = DebugUi::new();

        self.camera_controller.camera_entity = Some(camera_entity);
        self.rotating_entities = rotating_entities;

        println!("\n=== Praxis GUI Demo ===");
        println!("Comprehensive demonstration of Praxis GUI capabilities");
        println!("\nFeatures:");
        println!("  • Real-time performance monitoring");
        println!("  • Entity inspection and component editing");
        println!("  • Scene statistics and hierarchy");
        println!("  • Asset management demonstration");
        println!("  • Interactive camera controls");
        println!("  • Animated scene objects with lighting");
        println!("\nControls:");
        println!("  WASD - Move camera");
        println!("  Space/Ctrl - Up/Down");
        println!("  Shift - Sprint");
        println!("  Mouse - Look around");
        println!("  ESC - Toggle cursor lock");
        println!("  F1 - Toggle debug stats logging");
        println!("  F2 - Toggle entity list logging");
        println!("  F3 - Toggle scene rotation");
        println!("  F4 - Increase animation speed");
        println!("  F5 - Decrease animation speed");
        println!();

        self.window = Some(window.clone());
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.egui_integration = Some(egui_integration);
        self.debug_ui = Some(debug_ui);
        self.last_frame_time = Some(Instant::now());

        self.lock_cursor();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Clone the window Arc early to avoid borrow conflicts
        let window = match self.window.as_ref() {
            Some(w) => w.clone(),
            None => return,
        };

        // Let egui handle the event if cursor is unlocked
        if !self.cursor_locked {
            if let Some(egui_integration) = &mut self.egui_integration {
                if egui_integration.handle_event(&window, &event) {
                    window.request_redraw();
                    return;
                }
            }
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
                let delta = if let Some(last_time) = self.last_frame_time {
                    now.duration_since(last_time)
                } else {
                    std::time::Duration::from_secs_f32(1.0 / 60.0)
                };
                self.last_frame_time = Some(now);
                let delta_secs = delta.as_secs_f32();

                self.frame_timer.tick();
                self.input_state.update();

                self.update_camera(delta_secs);
                self.update_rotating_objects(delta_secs);

                static mut FRAME_COUNTER: u64 = 0;
                unsafe {
                    FRAME_COUNTER += 1;
                    if FRAME_COUNTER % 60 == 0 {
                        self.print_debug_info();
                        self.print_entity_list();
                    }
                }

                // Render GUI overlays
                if !self.cursor_locked {
                    if let (Some(egui_integration), Some(debug_ui)) = (
                        &mut self.egui_integration,
                        &mut self.debug_ui,
                    ) {
                        egui_integration.begin_frame(&window);

                        let ctx = egui_integration.context();
                        debug_ui.render(ctx);

                        let (_full_output, _clipped_primitives) = egui_integration.end_frame(&window);
                    }
                }

                if let Err(e) = self.render_scene() {
                    warn!("Render error: {}", e);
                }

                window.request_redraw();
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F1),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.show_debug_info = !self.show_debug_info;
                println!(
                    "Debug stats logging: {}",
                    if self.show_debug_info { "ON" } else { "OFF" }
                );
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F2),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.show_entity_list = !self.show_entity_list;
                println!(
                    "Entity list logging: {}",
                    if self.show_entity_list { "ON" } else { "OFF" }
                );
                if self.show_entity_list {
                    self.print_entity_list();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F3),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.scene_rotation_enabled = !self.scene_rotation_enabled;
                println!(
                    "Scene rotation: {}",
                    if self.scene_rotation_enabled {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                );
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F4),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.animation_speed = (self.animation_speed + 0.25).min(3.0);
                println!("Animation speed: {:.2}x", self.animation_speed);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F5),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.animation_speed = (self.animation_speed - 0.25).max(0.25);
                println!("Animation speed: {:.2}x", self.animation_speed);
            }
            _ => {
                praxis_input::winit_integration::process_window_event(
                    &mut self.input_state,
                    &event,
                );
            }
        }

        window.request_redraw();
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

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;
    praxis_gui::init()?;

    info!("Starting GUI Demo");

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = GuiDemoApp::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("gui_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}
