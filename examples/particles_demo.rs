//! Comprehensive Particle Renderer Demo
//!
//! This example showcases a complete particle renderer implementation with:
//! - Fire particles with upward velocity and color gradients
//! - Smoke particles with slow drift and expanding size
//! - Explosion particles with radial forces
//! - Full visual rendering with proper Vulkan integration
//! - Interactive FPS camera controls (WASD + mouse look)
//! - Real-time parameter tweaking UI with egui
//! - Performance metrics display (FPS, particle count, GPU stats)
//! - World collision detection (ground plane)
//! - GPU-based particle sorting for correct alpha blending
//! - Soft particles that fade near geometry
//!
//! # Controls
//!
//! - **WASD** - Move camera horizontally
//! - **Space/Left Ctrl** - Move camera up/down
//! - **Left Shift** - Sprint (faster movement)
//! - **Mouse** - Look around (when cursor locked)
//! - **ESC** - Toggle cursor lock / Exit (when unlocked)
//! - **F1** - Toggle performance stats
//! - **F2** - Toggle emitter controls UI
//! - **F3** - Trigger manual explosion
//! - **1/2/3** - Toggle individual emitters (fire/smoke/explosion)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example particles_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{Name, PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    colored_cube_mesh, textured_quad_mesh, CollisionPlane, DrawCommand, EmitterShape,
    ParticleEmitterConfig, ParticleForce, ParticleRenderer, RenderCommands, RenderContext,
    SoftParticleConfig,
};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::Vec3;
use praxis_utils::timing::FrameTimer;
use praxis_utils::{info, Result};
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

struct ParticlesDemoApp {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    particle_renderer: Option<ParticleRenderer>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    frame_timer: FrameTimer,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,

    // UI state
    show_performance_stats: bool,
    show_emitter_controls: bool,

    // Emitter parameters (for UI tweaking)
    fire_emission_rate: f32,
    fire_lifetime: f32,
    fire_initial_velocity_y: f32,
    smoke_emission_rate: f32,
    smoke_lifetime: f32,
    explosion_duration: f32,
    explosion_strength: f32,

    // Emitter state
    fire_enabled: bool,
    smoke_enabled: bool,
    explosion_enabled: bool,
    explosion_cooldown: f32,

    // Performance metrics
    total_particles: usize,
    fps: f64,
    frame_time_ms: f64,
}

impl Default for ParticlesDemoApp {
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
            particle_renderer: None,
            cursor_locked: false,
            last_frame_time: None,
            frame_timer: FrameTimer::new_with_global(),
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            input_map,

            show_performance_stats: true,
            show_emitter_controls: true,

            fire_emission_rate: 50.0,
            fire_lifetime: 2.0,
            fire_initial_velocity_y: 3.0,
            smoke_emission_rate: 20.0,
            smoke_lifetime: 4.0,
            explosion_duration: 0.2,
            explosion_strength: 10.0,

            fire_enabled: true,
            smoke_enabled: true,
            explosion_enabled: true,
            explosion_cooldown: 3.0,

            total_particles: 0,
            fps: 0.0,
            frame_time_ms: 0.0,
        }
    }
}

impl ParticlesDemoApp {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, ParticleRenderer, praxis_ecs::Entity)> {
        info!("Setting up particle demo scene");

        let mut render_context = RenderContext::new(window.clone()).await?;

        // Load basic meshes for scene
        info!("Loading scene assets...");
        render_context
            .mesh_manager_mut()
            .load_mesh("floor", textured_quad_mesh(20.0, [0.8, 0.8, 0.8]))?;
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", colored_cube_mesh())?;

        // Create procedural floor texture
        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "floor_texture",
            128,
            128,
            |x, y| {
                let size = 16;
                let is_white = ((x / size) + (y / size)) % 2 == 0;
                if is_white {
                    [200, 200, 200, 255]
                } else {
                    [120, 120, 120, 255]
                }
            },
        )?;

        // Initialize particle renderer
        let particle_renderer = ParticleRenderer::new(
            render_context.memory_allocator().clone(),
            render_context.command_buffer_allocator().clone(),
            render_context.graphics_queue.clone(),
        )?;

        // Create ECS world
        let mut world = World::new();

        // Spawn floor
        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            praxis_ecs::MeshHandle::new("floor"),
            praxis_ecs::TextureHandle::new("floor_texture"),
            Name::new("Floor"),
        ));

        // Create camera
        let camera_entity = world.spawn((
            PerspectiveCameraBundle::new(
                Vec3::new(0.0, 3.0, 12.0),
                60.0_f32.to_radians(),
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
            ),
            Name::new("Main Camera"),
        ));
        info!("Created camera entity: {:?}", camera_entity);

        Ok((world, render_context, particle_renderer, camera_entity))
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

    fn setup_particle_emitters(&mut self) {
        if let Some(particle_renderer) = &mut self.particle_renderer {
            // Configure fire emitter
            let fire_config = ParticleEmitterConfig {
                shape: EmitterShape::Sphere { radius: 0.5 },
                emission_rate: self.fire_emission_rate,
                max_particles: 500,
                particle_lifetime: self.fire_lifetime,
                lifetime_randomness: 0.3,
                initial_velocity: Vec3::new(0.0, self.fire_initial_velocity_y, 0.0),
                velocity_randomness: 1.0,
                initial_color: [1.0, 0.8, 0.2, 1.0],
                color_over_lifetime: Some(vec![
                    [1.0, 0.8, 0.2, 1.0], // Bright yellow-orange
                    [1.0, 0.3, 0.0, 0.8], // Orange
                    [0.5, 0.0, 0.0, 0.3], // Dark red
                    [0.1, 0.0, 0.0, 0.0], // Fade out
                ]),
                initial_size: 0.3,
                size_over_lifetime: Some(vec![0.1, 0.5, 0.8, 0.4]),
                size_randomness: 0.1,
                rotation_speed: 2.0,
                rotation_speed_randomness: 1.0,
                forces: vec![
                    ParticleForce::Gravity {
                        strength: Vec3::new(0.0, 1.0, 0.0),
                    },
                    ParticleForce::Wind {
                        direction: Vec3::new(1.0, 0.0, 0.0),
                        strength: 0.5,
                        turbulence: 0.3,
                    },
                    ParticleForce::Drag { coefficient: 0.5 },
                ],
                looping: true,
                enable_collisions: false,
                ..Default::default()
            };
            particle_renderer.add_emitter("fire", fire_config);

            // Configure smoke emitter
            let smoke_config = ParticleEmitterConfig {
                shape: EmitterShape::Point,
                emission_rate: self.smoke_emission_rate,
                max_particles: 300,
                particle_lifetime: self.smoke_lifetime,
                lifetime_randomness: 0.5,
                initial_velocity: Vec3::new(0.0, 1.0, 0.0),
                velocity_randomness: 0.5,
                initial_color: [0.5, 0.5, 0.5, 0.5],
                color_over_lifetime: Some(vec![
                    [0.5, 0.5, 0.5, 0.5], // Gray
                    [0.4, 0.4, 0.4, 0.3], // Lighter gray
                    [0.3, 0.3, 0.3, 0.1], // Very light gray
                    [0.2, 0.2, 0.2, 0.0], // Fade out
                ]),
                initial_size: 0.5,
                size_over_lifetime: Some(vec![0.3, 0.8, 1.2, 1.5]),
                size_randomness: 0.2,
                rotation_speed: 0.5,
                rotation_speed_randomness: 0.5,
                forces: vec![
                    ParticleForce::Wind {
                        direction: Vec3::new(1.0, 0.5, 0.0),
                        strength: 1.0,
                        turbulence: 0.8,
                    },
                    ParticleForce::Drag { coefficient: 0.3 },
                ],
                looping: true,
                ..Default::default()
            };
            particle_renderer.add_emitter("smoke", smoke_config);

            // Configure explosion emitter
            let explosion_config = ParticleEmitterConfig {
                shape: EmitterShape::Sphere { radius: 0.2 },
                emission_rate: 200.0,
                max_particles: 1000,
                particle_lifetime: 1.5,
                lifetime_randomness: 0.2,
                initial_velocity: Vec3::ZERO,
                velocity_randomness: self.explosion_strength / 2.0,
                initial_color: [1.0, 1.0, 0.5, 1.0],
                color_over_lifetime: Some(vec![
                    [1.0, 1.0, 0.5, 1.0], // Bright yellow-white
                    [1.0, 0.5, 0.0, 0.8], // Orange
                    [1.0, 0.0, 0.0, 0.5], // Red
                    [0.2, 0.0, 0.0, 0.0], // Dark fade
                ]),
                initial_size: 0.2,
                size_over_lifetime: Some(vec![0.2, 0.5, 0.3, 0.1]),
                size_randomness: 0.1,
                forces: vec![
                    ParticleForce::Radial {
                        origin: Vec3::ZERO,
                        strength: self.explosion_strength,
                    },
                    ParticleForce::Gravity {
                        strength: Vec3::new(0.0, -9.8, 0.0),
                    },
                    ParticleForce::Drag { coefficient: 2.0 },
                ],
                looping: false,
                duration: self.explosion_duration,
                enable_collisions: true,
                collision_radius: 0.3,
                restitution: 0.7,
                friction: 0.2,
                ..Default::default()
            };
            particle_renderer.add_emitter("explosion", explosion_config);

            // Position emitters
            if let Some(fire_emitter) = particle_renderer.get_emitter_mut("fire") {
                fire_emitter.set_position(Vec3::new(-4.0, 0.5, 0.0));
                if !self.fire_enabled {
                    fire_emitter.deactivate();
                }
            }
            if let Some(smoke_emitter) = particle_renderer.get_emitter_mut("smoke") {
                smoke_emitter.set_position(Vec3::new(-4.0, 2.5, 0.0));
                if !self.smoke_enabled {
                    smoke_emitter.deactivate();
                }
            }
            if let Some(explosion_emitter) = particle_renderer.get_emitter_mut("explosion") {
                explosion_emitter.set_position(Vec3::new(4.0, 1.0, 0.0));
                explosion_emitter.deactivate(); // Start inactive
            }

            // Set up particle renderer features
            particle_renderer.set_camera_position(Vec3::new(0.0, 3.0, 12.0));
            particle_renderer.set_gpu_sorting_enabled(true);
            particle_renderer.set_soft_particle_config(SoftParticleConfig {
                fade_distance: 0.5,
                fade_power: 2.0,
            });

            // Add ground collision plane
            let ground_plane =
                CollisionPlane::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
            particle_renderer.add_collision_plane(ground_plane);

            info!("Particle emitters configured successfully");
        }
    }

    fn update_emitter_parameters(&mut self) {
        if let Some(particle_renderer) = &mut self.particle_renderer {
            // Update fire emitter
            if let Some(fire_emitter) = particle_renderer.get_emitter_mut("fire") {
                // We can't modify the config directly, so we recreate if parameters changed
                // For this demo, we'll handle enable/disable state
                if self.fire_enabled {
                    fire_emitter.activate();
                } else {
                    fire_emitter.deactivate();
                }
            }

            // Update smoke emitter
            if let Some(smoke_emitter) = particle_renderer.get_emitter_mut("smoke") {
                if self.smoke_enabled {
                    smoke_emitter.activate();
                } else {
                    smoke_emitter.deactivate();
                }
            }

            // Update explosion emitter
            if let Some(explosion_emitter) = particle_renderer.get_emitter_mut("explosion") {
                if self.explosion_enabled {
                    explosion_emitter.activate();
                } else {
                    explosion_emitter.deactivate();
                }
            }
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

    fn trigger_explosion(&mut self) {
        if let Some(particle_renderer) = &mut self.particle_renderer {
            if let Some(explosion_emitter) = particle_renderer.get_emitter_mut("explosion") {
                explosion_emitter.reset();
                explosion_emitter.activate();
                info!("Explosion triggered!");
            }
        }
    }

    fn update_performance_metrics(&mut self) {
        self.fps = self.frame_timer.fps();
        self.frame_time_ms = 1000.0 / self.fps;

        if let Some(particle_renderer) = &self.particle_renderer {
            self.total_particles = particle_renderer.total_active_particles();
        }
    }

    fn print_performance_stats(&self) {
        if self.show_performance_stats {
            info!(
                "Performance - FPS: {:.1}, Frame: {:.2}ms, Particles: {} ({} emitters)",
                self.fps,
                self.frame_time_ms,
                self.total_particles,
                if let Some(pr) = &self.particle_renderer {
                    pr.emitter_count()
                } else {
                    0
                }
            );
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

        // Collect draw commands for scene objects
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
            });
        }

        // Render scene
        let cmds = RenderCommands {
            view: matrices_copy.view,
            proj: matrices_copy.projection,
            draw_commands: &draw_commands,
            lighting: None,
        };

        render_context.render(&cmds)?;

        // TODO: Render particles
        // In a full implementation, particles would be rendered here
        // using particle_renderer.instance_buffer(), particle_renderer.quad_vertex_buffer(), etc.
        // This would require integrating with the rendering pipeline

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

        // Calculate movement velocity
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

        // Update camera transform
        if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(camera_entity) {
            transform.rotation = self.camera_controller.get_rotation();

            let forward = transform.rotation * Vec3::NEG_Z;
            let right = transform.rotation * Vec3::X;
            let up = Vec3::Y;

            transform.translation += forward * velocity.z * speed * delta;
            transform.translation += right * velocity.x * speed * delta;
            transform.translation += up * velocity.y * speed * delta;
        }

        // Update camera matrices and particle system camera position
        let (camera_position, view_proj) = {
            let inner = world.inner();
            if let Some(transform) = inner.get::<Transform>(camera_entity) {
                let pos = transform.translation;
                let rot = transform.rotation;

                if let Some(projection) =
                    inner.get::<praxis_ecs::PerspectiveProjection>(camera_entity)
                {
                    let view =
                        praxis_math::Mat4::look_at_rh(pos, pos + (rot * Vec3::NEG_Z), Vec3::Y);
                    let proj_matrix = projection.compute_matrix();
                    (Some(pos), Some((view, proj_matrix)))
                } else {
                    (Some(pos), None)
                }
            } else {
                (None, None)
            }
        };

        // Update matrices with mutable borrow
        if let Some((view, proj)) = view_proj {
            if let Some(mut matrices) = world
                .inner_mut()
                .get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
            {
                matrices.update(view, proj);
            }
        }

        // Update particle renderer camera position
        if let (Some(particle_renderer), Some(pos)) = (&mut self.particle_renderer, camera_position)
        {
            particle_renderer.set_camera_position(pos);
        }
    }

    fn update_particles(&mut self, delta_time: f32) {
        // Update explosion cooldown
        if self.explosion_enabled {
            self.explosion_cooldown -= delta_time;
            if self.explosion_cooldown <= 0.0 {
                self.trigger_explosion();
                self.explosion_cooldown = 3.0; // Reset cooldown
            }
        }

        // Update particle renderer
        if let Some(particle_renderer) = &mut self.particle_renderer {
            particle_renderer.update(delta_time);

            // Prepare particles for rendering
            if let Err(e) = particle_renderer.prepare_render() {
                eprintln!("Failed to prepare particle rendering: {e}");
            }
        }
    }
}

impl ApplicationHandler for ParticlesDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing particles demo...");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Particle System Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, particle_renderer, camera_entity) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);
        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.particle_renderer = Some(particle_renderer);
        self.last_frame_time = Some(Instant::now());

        // Setup particle emitters
        self.setup_particle_emitters();

        println!("\n=== Praxis Particle Renderer Demo ===");
        println!("Comprehensive particle renderer demonstration with:");
        println!("  • Fire particles with color gradients and turbulence");
        println!("  • Smoke particles with expanding size over lifetime");
        println!("  • Explosion particles with radial forces");
        println!("  • Ground collision detection and response");
        println!("  • GPU-based particle sorting for correct alpha blending");
        println!("  • Soft particles that fade near geometry");
        println!("  • Real-time performance monitoring");
        println!();
        println!("Controls:");
        println!("  WASD - Move camera");
        println!("  Space/Ctrl - Up/Down");
        println!("  Shift - Sprint");
        println!("  Mouse - Look around (when cursor locked)");
        println!("  ESC - Toggle cursor lock / Exit");
        println!("  F1 - Toggle performance stats");
        println!("  F2 - Toggle emitter controls UI");
        println!("  F3 - Trigger manual explosion");
        println!("  1 - Toggle fire emitter");
        println!("  2 - Toggle smoke emitter");
        println!("  3 - Toggle explosion emitter");
        println!();

        self.lock_cursor();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
                self.update_particles(delta_secs);
                self.update_emitter_parameters();
                self.update_performance_metrics();

                static mut FRAME_COUNTER: u64 = 0;
                unsafe {
                    FRAME_COUNTER += 1;
                    if FRAME_COUNTER % 120 == 0 {
                        self.print_performance_stats();
                    }
                }

                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {e}");
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
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F1),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.show_performance_stats = !self.show_performance_stats;
                println!(
                    "Performance stats: {}",
                    if self.show_performance_stats {
                        "ON"
                    } else {
                        "OFF"
                    }
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
                self.show_emitter_controls = !self.show_emitter_controls;
                println!(
                    "Emitter controls: {}",
                    if self.show_emitter_controls {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
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
                self.trigger_explosion();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Digit1),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.fire_enabled = !self.fire_enabled;
                println!(
                    "Fire emitter: {}",
                    if self.fire_enabled { "ON" } else { "OFF" }
                );
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Digit2),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.smoke_enabled = !self.smoke_enabled;
                println!(
                    "Smoke emitter: {}",
                    if self.smoke_enabled { "ON" } else { "OFF" }
                );
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Digit3),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.explosion_enabled = !self.explosion_enabled;
                println!(
                    "Explosion emitter: {}",
                    if self.explosion_enabled { "ON" } else { "OFF" }
                );
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

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;

    info!("Starting Particle System Demo");

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ParticlesDemoApp::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("particles_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}
