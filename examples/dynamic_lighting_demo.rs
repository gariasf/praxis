//! Dynamic lighting demonstration with multiple moving lights affecting multiple meshes.
//!
//! This example demonstrates the complete lighting system with:
//! - Multiple directional lights (simulating sun, moon, etc.)
//! - Multiple point lights moving in various patterns
//! - Multiple meshes with different materials affected by the lights
//! - ECS-based light management using components
//! - Real-time lighting updates via the gather_lighting_system
//! - Camera navigation to view lighting from different angles
//!
//! The example shows:
//! 1. How to set up light entities using DirectionalLight and PointLight components
//! 2. How the gather_lighting_system collects light data from the ECS
//! 3. How lighting data flows from ECS components to GPU uniforms
//! 4. How multiple lights combine to illuminate the scene
//! 5. How to animate lights by modifying Transform and light components
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
//! cargo run --example dynamic_lighting_demo
//! ```

use praxis_ecs::{
    DirectionalLight, LightingData, PerspectiveCameraBundle, PointLight, Transform, World,
};
use praxis_graphics::{
    textured_cube_mesh, textured_quad_mesh, DrawCommand, RenderContext,
    RenderCommands,
};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Quat, Vec3};
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

/// Camera controller for FPS-style navigation
struct CameraController {
    move_speed: f32,
    sprint_multiplier: f32,
    mouse_sensitivity: f32,
    pitch: f32,
    yaw: f32,
    max_pitch: f32,
    camera_entity: Option<praxis_ecs::Entity>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 8.0,
            sprint_multiplier: 2.0,
            mouse_sensitivity: 0.002,
            pitch: 0.0,
            yaw: std::f32::consts::PI,
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.01,
            camera_entity: None,
        }
    }
}

impl CameraController {
    fn update_rotation(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw -= delta_x * self.mouse_sensitivity;
        self.pitch -= delta_y * self.mouse_sensitivity;
        self.pitch = self.pitch.clamp(-self.max_pitch, self.max_pitch);
    }

    fn get_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
    }
}

struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    cursor_locked: bool,
    last_frame_time: Option<Instant>,
    start_time: Instant,
    camera_controller: CameraController,
    input_state: InputState,
    input_map: InputMap,
    // Store light entities for animation
    point_light_entities: Vec<praxis_ecs::Entity>,
}

impl Default for App {
    fn default() -> Self {
        // Set up input mapping for camera controls
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
            start_time: Instant::now(),
            camera_controller: CameraController::default(),
            input_state: InputState::default(),
            input_map,
            point_light_entities: Vec::new(),
        }
    }
}

impl App {
    /// Sets up the entire scene including meshes, textures, lights, and camera
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(
        World,
        RenderContext,
        praxis_ecs::Entity,
        Vec<praxis_ecs::Entity>,
    )> {
        info!("Setting up dynamic lighting demo scene");

        // Initialize render context and load assets
        let mut render_context = RenderContext::new(window.clone()).await?;
        Self::load_assets(&mut render_context)?;

        // Create ECS world and add the lighting data resource
        // This resource is required by the gather_lighting_system to store collected light data
        let mut world = World::new();
        world.insert_resource(LightingData::default());

        // Spawn scene objects (floor and cubes)
        Self::spawn_scene_objects(&mut world);

        // Spawn light entities and store their IDs for animation
        let point_light_entities = Self::spawn_lights(&mut world);

        // Create camera entity
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 15.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));
        info!("Created camera entity: {:?}", camera_entity);

        Ok((world, render_context, camera_entity, point_light_entities))
    }

    /// Loads all assets (meshes and textures) into the render context
    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading meshes and textures...");

        // Load mesh geometry
        render_context
            .mesh_manager_mut()
            .load_mesh("floor_quad", textured_quad_mesh(20.0, [1.0, 1.0, 1.0]))?;
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", textured_cube_mesh([1.0, 1.0, 1.0]))?;

        // Create procedural textures for visual variety
        Self::create_checker_texture(render_context, "floor_checker")?;
        Self::create_brick_texture(render_context, "brick")?;
        Self::create_metal_texture(render_context, "metal")?;
        Self::create_wood_texture(render_context, "wood")?;

        info!(
            "Assets loaded: {} meshes, 4 textures",
            render_context.mesh_manager().mesh_count()
        );

        Ok(())
    }

    /// Creates a checkered texture procedurally
    fn create_checker_texture(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 64;
        let height = 64;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let checker_size = 8;
                let is_white = ((x / checker_size) + (y / checker_size)) % 2 == 0;
                let color = if is_white {
                    [240, 240, 240, 255]
                } else {
                    [60, 60, 60, 255]
                };
                pixels.extend_from_slice(&color);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a brick texture procedurally
    fn create_brick_texture(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 64;
        let height = 64;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let brick_height = 16;
                let brick_width = 32;
                let row = y / brick_height;
                let offset = if row % 2 == 0 { 0 } else { brick_width / 2 };
                let col = (x + offset) / brick_width;

                let is_mortar_h = y % brick_height < 2;
                let is_mortar_v = (x + offset) % brick_width < 2;

                let color = if is_mortar_h || is_mortar_v {
                    [180, 180, 180, 255]
                } else {
                    let variation = ((x + y + col * 13) % 20) as u8;
                    [160 + variation, 80 + variation / 2, 60 + variation / 3, 255]
                };
                pixels.extend_from_slice(&color);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a metallic texture procedurally
    fn create_metal_texture(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 64;
        let height = 64;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let noise = ((x * 7 + y * 13) % 40) as u8;
                let base = 160 + noise;
                let color = [base, base, base + 20, 255];
                pixels.extend_from_slice(&color);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a wood texture procedurally
    fn create_wood_texture(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 64;
        let height = 64;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let grain = ((x as f32 * 0.3).sin() * 20.0) as i32;
                let base = 139 + grain.clamp(-20, 20);
                let color = [base as u8, (base - 30).max(0) as u8, 19, 255];
                pixels.extend_from_slice(&color);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Spawns the static scene objects (floor and cubes)
    fn spawn_scene_objects(world: &mut World) {
        info!("Spawning scene objects...");

        // Floor with checkered pattern
        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            praxis_ecs::MeshHandle::new("floor_quad"),
            praxis_ecs::TextureHandle::new("floor_checker"),
            praxis_ecs::Name::new("Floor"),
        ));

        // Row of cubes at different positions with different materials
        // These will all be affected by the moving lights
        world.spawn((
            Transform::from_xyz(-6.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("brick"),
            praxis_ecs::Name::new("Cube 1"),
        ));

        world.spawn((
            Transform::from_xyz(-3.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("metal"),
            praxis_ecs::Name::new("Cube 2"),
        ));

        world.spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("wood"),
            praxis_ecs::Name::new("Cube 3"),
        ));

        world.spawn((
            Transform::from_xyz(3.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("brick"),
            praxis_ecs::Name::new("Cube 4"),
        ));

        world.spawn((
            Transform::from_xyz(6.0, 1.0, 0.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("metal"),
            praxis_ecs::Name::new("Cube 5"),
        ));

        // Back row of cubes
        world.spawn((
            Transform::from_xyz(-4.5, 1.0, -5.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("wood"),
            praxis_ecs::Name::new("Cube 6"),
        ));

        world.spawn((
            Transform::from_xyz(0.0, 1.0, -5.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("brick"),
            praxis_ecs::Name::new("Cube 7"),
        ));

        world.spawn((
            Transform::from_xyz(4.5, 1.0, -5.0),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("metal"),
            praxis_ecs::Name::new("Cube 8"),
        ));

        info!("Spawned 9 scene objects (1 floor + 8 cubes)");
    }

    /// Spawns light entities that will be animated
    /// Returns the entity IDs so we can update them each frame
    fn spawn_lights(world: &mut World) -> Vec<praxis_ecs::Entity> {
        info!("Spawning light entities...");

        let mut light_entities = Vec::new();

        // Directional light 1: Main sun-like light from above-right
        // This simulates outdoor sunlight
        world.spawn((
            DirectionalLight::new(
                Vec3::new(0.3, -0.8, 0.5).normalize(), // Direction toward ground
                Vec3::new(1.0, 0.95, 0.85),             // Warm white color
                0.6,                                    // Moderate intensity
            ),
            praxis_ecs::Name::new("Sun Light"),
        ));

        // Directional light 2: Fill light from the side
        // This adds some bounce lighting for a more realistic look
        world.spawn((
            DirectionalLight::new(
                Vec3::new(-0.5, -0.3, 0.0).normalize(),
                Vec3::new(0.4, 0.5, 0.7), // Cool blue-ish color
                0.3,                      // Lower intensity for fill
            ),
            praxis_ecs::Name::new("Fill Light"),
        ));

        // Point light 1: Red light that circles around the scene
        // This will move in a circular pattern at medium height
        let red_light = world.spawn((
            Transform::from_xyz(5.0, 3.0, 0.0), // Starting position
            PointLight::new(
                Vec3::new(1.0, 0.2, 0.2), // Red color
                25.0,                     // High intensity for dramatic effect
                15.0,                     // Large range to affect multiple objects
            ),
            praxis_ecs::Name::new("Red Point Light"),
        ));
        light_entities.push(red_light);

        // Point light 2: Green light that moves in a different pattern
        // This will move in a figure-eight pattern
        let green_light = world.spawn((
            Transform::from_xyz(-5.0, 3.0, 0.0),
            PointLight::new(
                Vec3::new(0.2, 1.0, 0.2), // Green color
                25.0,
                15.0,
            ),
            praxis_ecs::Name::new("Green Point Light"),
        ));
        light_entities.push(green_light);

        // Point light 3: Blue light that bobs up and down
        // This will oscillate vertically
        let blue_light = world.spawn((
            Transform::from_xyz(0.0, 5.0, -3.0),
            PointLight::new(
                Vec3::new(0.3, 0.3, 1.0), // Blue color
                30.0,                     // Even higher intensity
                12.0,
            ),
            praxis_ecs::Name::new("Blue Point Light"),
        ));
        light_entities.push(blue_light);

        // Point light 4: White light that spirals
        // This combines circular and vertical motion
        let white_light = world.spawn((
            Transform::from_xyz(0.0, 2.0, 3.0),
            PointLight::new(
                Vec3::new(1.0, 1.0, 1.0), // White color
                20.0,
                10.0,
            ),
            praxis_ecs::Name::new("White Point Light"),
        ));
        light_entities.push(white_light);

        info!(
            "Spawned {} lights (2 directional + 4 point lights)",
            2 + light_entities.len()
        );

        light_entities
    }

    /// Animates the point lights by updating their Transform components
    /// This demonstrates how moving entities with light components creates dynamic lighting
    fn animate_lights(&mut self, elapsed_time: f32) {
        let world = self.world.as_mut().unwrap();

        // Red light: Circle around the scene horizontally
        if let Some(&entity) = self.point_light_entities.get(0) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let radius = 7.0;
                let speed = 0.8;
                let angle = elapsed_time * speed;
                transform.translation.x = angle.cos() * radius;
                transform.translation.z = angle.sin() * radius;
                transform.translation.y = 3.0;
            }
        }

        // Green light: Figure-eight pattern
        if let Some(&entity) = self.point_light_entities.get(1) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let speed = 1.0;
                let angle = elapsed_time * speed;
                transform.translation.x = (angle * 2.0).sin() * 6.0;
                transform.translation.z = angle.sin() * 4.0;
                transform.translation.y = 3.5;
            }
        }

        // Blue light: Vertical bobbing motion
        if let Some(&entity) = self.point_light_entities.get(2) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let speed = 1.5;
                let bob = (elapsed_time * speed).sin();
                transform.translation.y = 3.0 + bob * 2.5; // Oscillate between 0.5 and 5.5
                transform.translation.x = 0.0;
                transform.translation.z = -3.0;
            }
        }

        // White light: Spiral motion (circular + vertical)
        if let Some(&entity) = self.point_light_entities.get(3) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let radius = 5.0;
                let speed = 1.2;
                let angle = elapsed_time * speed;
                transform.translation.x = angle.cos() * radius;
                transform.translation.z = 3.0 + angle.sin() * radius;
                // Spiral up and down
                transform.translation.y = 2.0 + ((elapsed_time * 0.7).sin() + 1.0) * 2.0;
            }
        }
    }

    /// Locks the cursor to the window for mouse look
    fn lock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(false);
            let _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
            self.cursor_locked = true;
        }
    }

    /// Unlocks the cursor from the window
    fn unlock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(true);
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            self.cursor_locked = false;
        }
    }

    /// Main render function that orchestrates the entire rendering process
    /// This shows the complete data flow from ECS to GPU
    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();

        // Step 1: Run the gather_lighting_system to collect light data from ECS
        // This system queries all DirectionalLight and PointLight entities and
        // populates the LightingData resource with their current state
        praxis_ecs::systems::gather_lighting_system(
            world.resource_mut::<LightingData>(),
            world.query::<(&DirectionalLight, Option<&Transform>)>(),
            world.query::<(&PointLight, Option<&praxis_ecs::GlobalTransform>, Option<&Transform>)>(),
        );

        // Step 2: Get the collected lighting data from the resource
        let lighting_data = world.resource::<LightingData>();

        // Step 3: Get camera matrices for the view and projection
        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        // Step 4: Build draw commands by querying all renderable entities
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

        let cmds = RenderCommands {
            view: matrices_copy.view,
            proj: matrices_copy.projection,
            draw_commands: &draw_commands,
            lighting: Some(lighting_data),
        };

        render_context.render(&cmds)?;

        Ok(())
    }

    /// Updates the camera based on input state
    fn update_camera(
        camera_entity: praxis_ecs::Entity,
        camera_controller: &CameraController,
        input_state: &InputState,
        input_map: &InputMap,
        world: &mut World,
    ) {
        // Calculate velocity based on input
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

        // Apply sprint multiplier if shift is held
        let mut speed = camera_controller.move_speed;
        if input_map.is_action_pressed(&Action::new("sprint"), input_state) {
            speed *= camera_controller.sprint_multiplier;
        }

        let dt = 1.0 / 60.0;

        // Update transform
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

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing...");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Dynamic Lighting Demo")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, point_light_entities) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {}", e);
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);
        self.point_light_entities = point_light_entities;

        println!("\n=== Praxis Dynamic Lighting Demo ===");
        println!("Demonstrating:");
        println!("  • Multiple directional lights (sun, fill light)");
        println!("  • Multiple animated point lights (4 moving lights)");
        println!("  • Multiple meshes affected by all lights");
        println!("  • ECS-based light management");
        println!("  • gather_lighting_system collecting light data");
        println!("  • Real-time lighting data flow from ECS to GPU");
        println!("\nLights in Scene:");
        println!("  🌞 Sun: Warm directional light from above");
        println!("  🌙 Fill: Cool directional light from side");
        println!("  🔴 Red Point: Circles horizontally");
        println!("  🟢 Green Point: Figure-eight pattern");
        println!("  🔵 Blue Point: Vertical bobbing");
        println!("  ⚪ White Point: Spiral motion");
        println!("\nControls:");
        println!("  WASD - Move camera horizontally");
        println!("  Space - Move up");
        println!("  Left Ctrl - Move down");
        println!("  Left Shift - Sprint (hold)");
        println!("  Mouse - Look around");
        println!("  ESC - Toggle cursor lock / Exit (when unlocked)");
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

                // Animate lights based on elapsed time
                let elapsed_time = self.start_time.elapsed().as_secs_f32();
                self.animate_lights(elapsed_time);

                // Update input and camera
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

                    // Manually update camera matrices
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

                // Render the scene
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

fn main() -> Result<()> {
    // Initialize engine subsystems
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;

    // Create event loop and run application
    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}
