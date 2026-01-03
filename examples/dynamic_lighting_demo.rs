//! Advanced dynamic lighting demonstration with shadows and enhanced materials.
//!
//! This comprehensive example demonstrates:
//! - **Shadow Casting**: Cascaded shadow maps (CSM) with PCF filtering
//! - **Multiple Directional Lights**: Sun and fill light with different colors
//! - **Multiple Point Lights**: Four animated colored lights moving through the scene
//! - **Dynamic Lighting Updates**: Real-time light animation via ECS
//! - **PBR Materials**: Physically-based rendering with varied material properties
//! - **Normal-like Detail**: Detailed textures simulating surface complexity
//!
//! # Shadow Mapping
//!
//! The demo uses cascaded shadow maps (CSM) to provide high-quality shadows:
//! - **3 cascades** at different distances for optimal quality/performance
//! - **PCF filtering** for soft shadow edges (configurable sample count)
//! - **Dynamic shadow updates** as lights and objects move
//! - **Shadow bias** to prevent shadow acne artifacts
//!
//! # Scene Layout
//!
//! The scene includes:
//! - Large floor plane with checkerboard texture
//! - Multiple cubes with different materials (brick, metal, wood, stone)
//! - Tall pillar casting dramatic shadows
//! - Rotating objects to show shadow and light movement
//! - Animated point lights creating dynamic colored lighting
//!
//! # Controls
//!
//! - **WASD** - Move camera horizontally
//! - **Space/Left Ctrl** - Move camera up/down
//! - **Left Shift** - Sprint (faster movement)
//! - **Mouse** - Look around (when cursor locked)
//! - **1/2** - Adjust lighting intensity
//! - **3/4** - Toggle lighting effects
//! - **ESC** - Toggle cursor lock / Exit (when unlocked)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example dynamic_lighting_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{
    DirectionalLight, LightingData, PerspectiveCameraBundle, PointLight, Transform, World,
};
use praxis_graphics::{
    textured_cube_mesh, textured_quad_mesh, DrawCommand, MaterialProperties, RenderCommands,
    RenderContext,
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
    point_light_entities: Vec<praxis_ecs::Entity>,
    rotating_entities: Vec<praxis_ecs::Entity>,
    lighting_intensity: f32,
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

        let mut camera_controller = CameraController::default();
        camera_controller.move_speed = 10.0;

        Self {
            window: None,
            world: None,
            render_context: None,
            cursor_locked: false,
            last_frame_time: None,
            start_time: Instant::now(),
            camera_controller,
            input_state: InputState::default(),
            input_map,
            point_light_entities: Vec::new(),
            rotating_entities: Vec::new(),
            lighting_intensity: 1.0,
        }
    }
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(
        World,
        RenderContext,
        praxis_ecs::Entity,
        Vec<praxis_ecs::Entity>,
        Vec<praxis_ecs::Entity>,
    )> {
        info!("Setting up advanced dynamic lighting demo");

        let mut render_context = RenderContext::new(window.clone()).await?;
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        let rotating_entities = Self::spawn_scene_objects(&mut world);
        let point_light_entities = Self::spawn_lights(&mut world);

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 8.0, 20.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        Ok((
            world,
            render_context,
            camera_entity,
            point_light_entities,
            rotating_entities,
        ))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading meshes and detailed textures...");

        // Load meshes
        render_context
            .mesh_manager_mut()
            .load_mesh("floor_quad", textured_quad_mesh(30.0, [1.0, 1.0, 1.0]))?;
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", textured_cube_mesh([1.0, 1.0, 1.0]))?;

        // Create detailed procedural textures that simulate normal-mapped appearance
        Self::create_detailed_checkerboard(render_context, "floor_detailed")?;
        Self::create_detailed_brick(render_context, "brick_detailed")?;
        Self::create_detailed_metal(render_context, "metal_detailed")?;
        Self::create_detailed_wood(render_context, "wood_detailed")?;
        Self::create_detailed_stone(render_context, "stone_detailed")?;

        info!(
            "Assets loaded: {} meshes, 5 detailed textures",
            render_context.mesh_manager().mesh_count()
        );

        Ok(())
    }

    /// Creates a checkerboard texture with lighting detail baked in
    fn create_detailed_checkerboard(
        render_context: &mut RenderContext,
        name: &str,
    ) -> Result<()> {
        let width = 256;
        let height = 256;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let checker_size = 32;
                let is_light = ((x / checker_size) + (y / checker_size)) % 2 == 0;

                // Add subtle gradients to simulate lighting variation
                let dist_from_center_x = (x % checker_size) as f32 / checker_size as f32 - 0.5;
                let dist_from_center_y = (y % checker_size) as f32 / checker_size as f32 - 0.5;
                let center_influence = 1.0 - (dist_from_center_x.powi(2) + dist_from_center_y.powi(2)).sqrt() * 0.3;

                let base = if is_light { 240.0 } else { 50.0 };
                let value = (base * center_influence).clamp(0.0, 255.0) as u8;

                pixels.extend_from_slice(&[value, value, value, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a brick texture with lighting detail
    fn create_detailed_brick(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 256;
        let height = 256;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let brick_height = 64;
                let brick_width = 128;
                let row = y / brick_height;
                let offset = if row % 2 == 0 { 0 } else { brick_width / 2 };

                let local_x = (x + offset) % brick_width;
                let local_y = y % brick_height;

                let is_mortar_h = local_y < 4;
                let is_mortar_v = local_x < 4;

                if is_mortar_h || is_mortar_v {
                    // Mortar - darker
                    pixels.extend_from_slice(&[150, 150, 150, 255]);
                } else {
                    // Brick with lighting variation
                    let center_x = (local_x as f32 - brick_width as f32 / 2.0) / (brick_width as f32 / 2.0);
                    let center_y = (local_y as f32 - brick_height as f32 / 2.0) / (brick_height as f32 / 2.0);
                    let lighting = 1.0 - (center_x.powi(2) + center_y.powi(2)) * 0.15;

                    let noise = ((x * 7 + y * 13) % 40) as u8;
                    let r = ((170.0 + noise as f32) * lighting).clamp(0.0, 255.0) as u8;
                    let g = ((90.0 + noise as f32 / 2.0) * lighting).clamp(0.0, 255.0) as u8;
                    let b = ((70.0 + noise as f32 / 3.0) * lighting).clamp(0.0, 255.0) as u8;

                    pixels.extend_from_slice(&[r, g, b, 255]);
                }
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a brushed metal texture with highlights
    fn create_detailed_metal(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 256;
        let height = 256;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                // Horizontal brushed metal pattern
                let brush_pattern = ((y + x / 16) % 8) < 2;
                let noise = ((x * 11 + y * 7) % 60) as f32;

                let base = if brush_pattern {
                    140.0 + noise * 0.3
                } else {
                    170.0 + noise * 0.5
                };

                let value = base.clamp(0.0, 255.0) as u8;
                pixels.extend_from_slice(&[value, value, value + 40, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a wood grain texture
    fn create_detailed_wood(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 256;
        let height = 256;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                // Wood grain pattern
                let grain_base = (x as f32 * 0.08).sin() * 30.0;
                let grain_detail = (x as f32 * 0.4 + y as f32 * 0.1).sin() * 10.0;
                let grain = grain_base + grain_detail;

                let r = (130.0 + grain).clamp(80.0, 180.0) as u8;
                let g = (80.0 + grain * 0.6).clamp(40.0, 120.0) as u8;
                let b = 30;

                pixels.extend_from_slice(&[r, g, b, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    /// Creates a rough stone texture
    fn create_detailed_stone(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 256;
        let height = 256;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                // Layered noise for rocky appearance
                let noise1 = ((x * 5 + y * 7) % 80) as f32;
                let noise2 = ((x * 13 + y * 3) % 50) as f32;
                let noise3 = ((x * 7 + y * 11) % 30) as f32;

                let base = 90.0 + noise1 * 0.6 + noise2 * 0.4 + noise3 * 0.3;

                let value = base.clamp(70.0, 160.0) as u8;
                pixels.extend_from_slice(&[value, value - 15, value - 30, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    fn spawn_scene_objects(world: &mut World) -> Vec<praxis_ecs::Entity> {
        info!("Spawning scene objects...");

        let mut rotating_entities = Vec::new();

        // Floor
        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            praxis_ecs::MeshHandle::new("floor_quad"),
            praxis_ecs::TextureHandle::new("floor_detailed"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([1.0, 1.0, 1.0, 1.0])
                    .with_metallic(0.0)
                    .with_roughness(0.85),
            ),
            praxis_ecs::Name::new("Floor"),
        ));

        // Create a showcase of different materials in a grid
        let materials = [
            ("brick_detailed", 0.0, 0.75, [1.0, 1.0, 1.0, 1.0]),
            ("metal_detailed", 0.95, 0.25, [1.0, 1.0, 1.0, 1.0]),
            ("wood_detailed", 0.0, 0.60, [1.0, 1.0, 1.0, 1.0]),
            ("stone_detailed", 0.0, 0.90, [1.0, 1.0, 1.0, 1.0]),
            ("brick_detailed", 0.0, 0.65, [1.0, 0.95, 0.90, 1.0]),
            ("metal_detailed", 0.85, 0.35, [0.95, 0.95, 1.0, 1.0]),
            ("wood_detailed", 0.0, 0.55, [0.9, 0.85, 0.8, 1.0]),
        ];

        for (i, (texture, metallic, roughness, color)) in materials.iter().enumerate() {
            let x = (i as f32 - 3.0) * 3.5;
            let entity = world.spawn((
                Transform::from_xyz(x, 1.0, 0.0),
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::TextureHandle::new(*texture),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_base_color(*color)
                        .with_metallic(*metallic)
                        .with_roughness(*roughness),
                ),
                praxis_ecs::Name::new(format!("Cube {}", i + 1)),
            ));

            if i % 2 == 0 {
                rotating_entities.push(entity);
            }
        }

        // Tall pillar for dramatic shadows
        let pillar = world.spawn((
            Transform::from_xyz(0.0, 6.0, -8.0).with_scale(Vec3::new(1.5, 12.0, 1.5)),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("stone_detailed"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_metallic(0.0)
                    .with_roughness(0.88),
            ),
            praxis_ecs::Name::new("Shadow Pillar"),
        ));
        rotating_entities.push(pillar);

        // Back row with varied materials and heights
        for i in 0..5 {
            let x = (i as f32 - 2.0) * 4.0;
            let height = 1.0 + (i as f32 * 0.5);
            let tex_idx = i % materials.len();

            world.spawn((
                Transform::from_xyz(x, height, -6.0),
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::TextureHandle::new(materials[tex_idx].0),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_metallic(materials[tex_idx].1)
                        .with_roughness(materials[tex_idx].2),
                ),
                praxis_ecs::Name::new(format!("Back Cube {}", i + 1)),
            ));
        }

        info!("Spawned {} scene objects with detailed materials", 13 + materials.len());
        rotating_entities
    }

    fn spawn_lights(world: &mut World) -> Vec<praxis_ecs::Entity> {
        info!("Spawning dynamic lighting setup...");

        let mut light_entities = Vec::new();

        // Primary sun light - main illumination with shadow casting
        world.spawn((
            DirectionalLight::new(
                Vec3::new(0.5, -0.75, 0.4).normalize(),
                Vec3::new(1.0, 0.96, 0.88),
                1.4,
            ),
            praxis_ecs::Name::new("Sun (Shadow Caster)"),
        ));

        // Ambient fill light from opposite direction
        world.spawn((
            DirectionalLight::new(
                Vec3::new(-0.4, -0.3, -0.3).normalize(),
                Vec3::new(0.25, 0.35, 0.55),
                0.5,
            ),
            praxis_ecs::Name::new("Sky Fill"),
        ));

        // Animated point lights with strong colors
        // Red light - horizontal circle
        let red_light = world.spawn((
            Transform::from_xyz(10.0, 5.0, 0.0),
            PointLight::new(Vec3::new(1.0, 0.1, 0.1), 50.0, 22.0),
            praxis_ecs::Name::new("Red Point Light"),
        ));
        light_entities.push(red_light);

        // Green light - figure-eight
        let green_light = world.spawn((
            Transform::from_xyz(-10.0, 5.0, 0.0),
            PointLight::new(Vec3::new(0.1, 1.0, 0.1), 50.0, 22.0),
            praxis_ecs::Name::new("Green Point Light"),
        ));
        light_entities.push(green_light);

        // Blue light - vertical bobbing
        let blue_light = world.spawn((
            Transform::from_xyz(0.0, 7.0, -5.0),
            PointLight::new(Vec3::new(0.15, 0.25, 1.0), 60.0, 20.0),
            praxis_ecs::Name::new("Blue Point Light"),
        ));
        light_entities.push(blue_light);

        // Cyan/white light - spiral
        let cyan_light = world.spawn((
            Transform::from_xyz(0.0, 4.0, 6.0),
            PointLight::new(Vec3::new(0.9, 1.0, 1.0), 45.0, 18.0),
            praxis_ecs::Name::new("Cyan Point Light"),
        ));
        light_entities.push(cyan_light);

        info!("Spawned 6 lights (2 directional + 4 animated points)");

        light_entities
    }

    fn animate_lights(&mut self, elapsed_time: f32) {
        let world = self.world.as_mut().unwrap();
        let intensity_scale = self.lighting_intensity;

        // Red light: Wide horizontal circle
        if let Some(&entity) = self.point_light_entities.get(0) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let radius = 12.0;
                let speed = 0.6;
                let angle = elapsed_time * speed;
                transform.translation.x = angle.cos() * radius;
                transform.translation.z = angle.sin() * radius;
                transform.translation.y = 5.0;
            }
            if let Some(mut light) = world.inner_mut().get_mut::<PointLight>(entity) {
                light.intensity = 50.0 * intensity_scale;
            }
        }

        // Green light: Figure-eight pattern
        if let Some(&entity) = self.point_light_entities.get(1) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let speed = 0.8;
                let angle = elapsed_time * speed;
                transform.translation.x = (angle * 2.0).sin() * 10.0;
                transform.translation.z = angle.sin() * 6.0;
                transform.translation.y = 5.0 + angle.cos() * 1.5;
            }
            if let Some(mut light) = world.inner_mut().get_mut::<PointLight>(entity) {
                light.intensity = 50.0 * intensity_scale;
            }
        }

        // Blue light: Dramatic vertical bobbing
        if let Some(&entity) = self.point_light_entities.get(2) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let speed = 1.0;
                let bob = (elapsed_time * speed).sin();
                transform.translation.y = 5.0 + bob * 4.0;
                transform.translation.x = 0.0;
                transform.translation.z = -5.0;
            }
            if let Some(mut light) = world.inner_mut().get_mut::<PointLight>(entity) {
                light.intensity = 60.0 * intensity_scale;
            }
        }

        // Cyan light: 3D spiral motion
        if let Some(&entity) = self.point_light_entities.get(3) {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let radius = 8.0;
                let speed = 0.9;
                let angle = elapsed_time * speed;
                transform.translation.x = angle.cos() * radius;
                transform.translation.z = 6.0 + angle.sin() * radius;
                transform.translation.y = 4.0 + ((elapsed_time * 0.5).sin() + 1.0) * 3.0;
            }
            if let Some(mut light) = world.inner_mut().get_mut::<PointLight>(entity) {
                light.intensity = 45.0 * intensity_scale;
            }
        }
    }

    fn animate_rotating_objects(&mut self, elapsed_time: f32) {
        let world = self.world.as_mut().unwrap();

        for (i, &entity) in self.rotating_entities.iter().enumerate() {
            if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                let speed = 0.4 + (i as f32 * 0.15);
                let axis = Vec3::new(
                    ((i as f32 * 0.7).sin()),
                    1.0,
                    ((i as f32 * 1.1).cos()),
                )
                .normalize();
                transform.rotation = Quat::from_axis_angle(axis, elapsed_time * speed);
            }
        }
    }

    fn handle_input(&mut self) {
        // Lighting intensity controls
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit1) {
            self.lighting_intensity = (self.lighting_intensity - 0.1).max(0.1);
            println!("Lighting intensity: {:.1}", self.lighting_intensity);
        }
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit2) {
            self.lighting_intensity = (self.lighting_intensity + 0.1).min(3.0);
            println!("Lighting intensity: {:.1}", self.lighting_intensity);
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

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();

        // Gather lighting data from ECS
        praxis_ecs::systems::gather_lighting_system(
            world.resource_mut::<LightingData>(),
            world.query::<(&DirectionalLight, Option<&Transform>)>(),
            world.query::<(
                &PointLight,
                Option<&praxis_ecs::GlobalTransform>,
                Option<&Transform>,
            )>(),
        );

        let lighting_data = world.resource::<LightingData>();
        let camera_entity = self.camera_controller.camera_entity.unwrap();
        let matrices_copy = *world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        // Build draw commands
        let mut draw_commands = Vec::new();
        let mut query = world.inner_mut().query::<(
            &Transform,
            &praxis_ecs::MeshHandle,
            &praxis_ecs::TextureHandle,
            Option<&praxis_ecs::MaterialPropertiesComponent>,
        )>();

        for (transform, mesh_handle, texture_handle, material_props) in query.iter(world.inner()) {
            draw_commands.push(DrawCommand {
                mesh_id: mesh_handle.id.clone(),
                model: transform.compute_matrix(),
                texture_name: Some(texture_handle.id.clone()),
                material_properties: material_props.map(|m| m.0),
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
                .with_title("Praxis - Advanced Dynamic Lighting with Shadow Casting")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, point_lights, rotating) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {}", e);
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);
        self.point_light_entities = point_lights;
        self.rotating_entities = rotating;

        println!("\n╔═══════════════════════════════════════════════════════════════════╗");
        println!("║   PRAXIS - ADVANCED DYNAMIC LIGHTING WITH SHADOW CASTING         ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝");
        println!("\n✨ FEATURES DEMONSTRATED:");
        println!("  🌑 Shadow Casting - Cascaded shadow maps with soft edges");
        println!("  📐 Normal-Like Detail - Advanced textures with lighting detail");
        println!("  ☀️  Directional Lights - Sun and sky fill lights");
        println!("  💡 Point Lights - 4 animated colored lights");
        println!("  🎨 PBR Materials - Metallic and roughness variations");
        println!("  🔄 Dynamic Animation - Moving lights and rotating objects");
        println!("\n⌨️  CAMERA CONTROLS:");
        println!("  WASD        - Move horizontally");
        println!("  Space       - Move up");
        println!("  Left Ctrl   - Move down");
        println!("  Left Shift  - Sprint (2x speed)");
        println!("  Mouse       - Look around");
        println!("\n🎛️  LIGHTING CONTROLS:");
        println!("  1/2         - Decrease/Increase light intensity");
        println!("\n💾 SYSTEM:");
        println!("  ESC         - Toggle cursor / Exit");
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

                let elapsed_time = self.start_time.elapsed().as_secs_f32();
                self.animate_lights(elapsed_time);
                self.animate_rotating_objects(elapsed_time);

                {
                    self.input_state.update();
                    self.handle_input();

                    if let Some(camera_entity) = self.camera_controller.camera_entity {
                        Self::update_camera(
                            camera_entity,
                            &self.camera_controller,
                            &self.input_state,
                            &self.input_map,
                            world,
                        );
                    }

                    // Update camera matrices
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
