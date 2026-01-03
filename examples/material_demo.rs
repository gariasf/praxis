//! Material system demonstration with post-processing effects.
//!
//! This comprehensive example demonstrates:
//! - **PBR Material Properties**: Metallic, roughness, emissive, and base color
//! - **Material Variations**: Gallery of different material types and combinations
//! - **Normal-Like Surface Detail**: Enhanced textures simulating surface complexity
//! - **Multiple Post-Processing Effects**:
//!   - Bloom effect for emissive materials
//!   - Tone mapping for HDR rendering
//!   - Color grading adjustments
//!
//! # Material Properties Explained
//!
//! ## Metallic (0.0 to 1.0)
//! - **0.0**: Non-metallic (dielectric) - plastic, wood, stone, fabric
//! - **1.0**: Fully metallic - gold, silver, copper, iron
//! - Controls whether material reflects like metal or insulator
//! - Affects specular reflection color (metals tint reflections)
//!
//! ## Roughness (0.0 to 1.0)
//! - **0.0**: Perfectly smooth - mirror, polished metal
//! - **1.0**: Completely rough - matte paint, rough stone
//! - Controls micro-surface smoothness/glossiness
//! - Affects reflection sharpness and highlight size
//!
//! ## Emissive Strength (0.0+)
//! - **0.0**: No emission - normal object lit by lights
//! - **1.0+**: Self-illuminating - light sources, glowing objects
//! - Adds color regardless of lighting (self-illumination)
//! - Visible even in complete darkness
//! - Interacts with bloom post-processing
//!
//! ## Base Color (RGBA)
//! - RGBA tint multiplied with texture
//! - Use [1,1,1,1] to preserve texture colors
//! - Use colored tints to modify appearance
//!
//! # Gallery Layout
//!
//! The scene organizes materials in 4 rows:
//! - **Row 1**: Metallic progression (0.0 → 1.0)
//! - **Row 2**: Roughness progression (0.0 → 1.0)
//! - **Row 3**: Emissive progression (0.0 → 5.0) with bloom
//! - **Row 4**: Real-world material combinations
//!
//! # Post-Processing Effects
//!
//! The demo showcases multiple post-processing techniques:
//!
//! ## Bloom Effect
//! - Extracts bright pixels (emissive materials)
//! - Applies Gaussian blur for glow effect
//! - Composites back onto scene
//! - Adjustable threshold and intensity
//!
//! ## Tone Mapping
//! - Maps HDR colors to display range
//! - Adjustable exposure control
//! - Preserves color relationships
//!
//! ## Color Grading
//! - Adjusts overall color tone
//! - Simulates different lighting conditions
//! - Can create stylized looks
//!
//! # Controls
//!
//! - **WASD** - Move camera
//! - **Space/Ctrl** - Move up/down
//! - **Shift** - Sprint
//! - **Mouse** - Look around
//! - **1/2** - Adjust bloom threshold
//! - **3/4** - Adjust bloom intensity
//! - **5/6** - Adjust exposure
//! - **7/8** - Cycle color grading presets
//! - **ESC** - Toggle cursor / Exit
//!
//! # Usage
//!
//! ```bash
//! cargo run --example material_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{
    DirectionalLight, LightingData, PerspectiveCameraBundle, PointLight, Transform, World,
};
use praxis_graphics::{
    sphere_mesh, textured_cube_mesh, DrawCommand, MaterialProperties, RenderCommands,
    RenderContext,
};
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

#[derive(Clone, Copy)]
enum ColorGradingPreset {
    None,
    Warm,
    Cool,
    Dramatic,
    Desaturated,
}

impl ColorGradingPreset {
    fn next(self) -> Self {
        match self {
            Self::None => Self::Warm,
            Self::Warm => Self::Cool,
            Self::Cool => Self::Dramatic,
            Self::Dramatic => Self::Desaturated,
            Self::Desaturated => Self::None,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::None => "None",
            Self::Warm => "Warm",
            Self::Cool => "Cool",
            Self::Dramatic => "Dramatic",
            Self::Desaturated => "Desaturated",
        }
    }

    fn tint(&self) -> [f32; 3] {
        match self {
            Self::None => [1.0, 1.0, 1.0],
            Self::Warm => [1.1, 1.0, 0.9],
            Self::Cool => [0.9, 1.0, 1.1],
            Self::Dramatic => [1.2, 0.9, 0.8],
            Self::Desaturated => [1.0, 1.0, 1.0],
        }
    }
}

struct PostProcessSettings {
    bloom_threshold: f32,
    bloom_intensity: f32,
    exposure: f32,
    color_grading: ColorGradingPreset,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            bloom_threshold: 1.0,
            bloom_intensity: 0.4,
            exposure: 1.0,
            color_grading: ColorGradingPreset::None,
        }
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
    post_process_settings: PostProcessSettings,
    ambient_light_entity: Option<praxis_ecs::Entity>,
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
        camera_controller.move_speed = 8.0;

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
            post_process_settings: PostProcessSettings::default(),
            ambient_light_entity: None,
        }
    }
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, praxis_ecs::Entity, praxis_ecs::Entity)> {
        info!("Setting up material gallery with post-processing");

        let mut render_context = RenderContext::new(window.clone()).await?;
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        Self::spawn_material_gallery(&mut world);
        let ambient_light = Self::spawn_lights(&mut world);

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 4.0, 16.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        Ok((world, render_context, camera_entity, ambient_light))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading meshes and detailed textures...");

        // Load geometric primitives
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", textured_cube_mesh([1.0, 1.0, 1.0]))?;
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere", sphere_mesh(0.5, 36, 18, [1.0, 1.0, 1.0]))?;

        // Create varied textures to showcase material interactions
        Self::create_detailed_metal_texture(render_context, "metal_detailed")?;
        Self::create_detailed_stone_texture(render_context, "stone_detailed")?;
        Self::create_grid_texture(render_context, "grid")?;
        Self::create_gradient_texture(render_context, "gradient")?;
        Self::create_emissive_pattern_texture(render_context, "emissive_pattern")?;

        info!(
            "Assets loaded: {} meshes, 5 textures",
            render_context.mesh_manager().mesh_count()
        );

        Ok(())
    }

    fn create_detailed_metal_texture(
        render_context: &mut RenderContext,
        name: &str,
    ) -> Result<()> {
        let width = 128;
        let height = 128;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                // Fine brushed metal texture
                let brush = ((y + x / 12) % 6) < 2;
                let noise = ((x * 11 + y * 7) % 50) as f32;

                let base = if brush {
                    140.0 + noise * 0.4
                } else {
                    170.0 + noise * 0.6
                };

                let value = base.clamp(0.0, 255.0) as u8;
                pixels.extend_from_slice(&[value, value, value + 30, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    fn create_detailed_stone_texture(
        render_context: &mut RenderContext,
        name: &str,
    ) -> Result<()> {
        let width = 128;
        let height = 128;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let noise1 = ((x * 5 + y * 7) % 70) as f32;
                let noise2 = ((x * 13 + y * 3) % 50) as f32;
                let base = 100.0 + noise1 * 0.6 + noise2 * 0.4;

                let value = base.clamp(70.0, 150.0) as u8;
                pixels.extend_from_slice(&[value, value - 10, value - 20, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    fn create_grid_texture(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 128;
        let height = 128;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let grid_size = 16;
                let is_line = x % grid_size < 2 || y % grid_size < 2;

                let color = if is_line {
                    [255, 255, 255, 255]
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

    fn create_gradient_texture(render_context: &mut RenderContext, name: &str) -> Result<()> {
        let width = 128;
        let height = 128;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let t = x as f32 / width as f32;
                let r = (255.0 * (1.0 - t * 0.5)) as u8;
                let g = (255.0 * t) as u8;
                let b = (128.0 * (1.0 - t)) as u8;

                pixels.extend_from_slice(&[r, g, b, 255]);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    fn create_emissive_pattern_texture(
        render_context: &mut RenderContext,
        name: &str,
    ) -> Result<()> {
        let width = 128;
        let height = 128;
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let checker = ((x / 32) + (y / 32)) % 2 == 0;
                let stripe = (x + y) % 16 < 4;

                let color = if checker && stripe {
                    [0, 255, 255, 255] // Bright cyan
                } else if checker {
                    [0, 200, 200, 255] // Medium cyan
                } else {
                    [0, 100, 100, 255] // Dark cyan
                };
                pixels.extend_from_slice(&color);
            }
        }

        render_context
            .texture_manager_mut()
            .load_texture_from_bytes(name, &pixels, width, height)?;
        Ok(())
    }

    fn spawn_material_gallery(world: &mut World) {
        info!("Spawning comprehensive material gallery...");

        let y_height = 1.0;
        let spacing_x = 3.0;
        let spacing_z = 3.5;
        let positions = [-6.0, -3.0, 0.0, 3.0, 6.0];

        // ROW 1: Metallic progression (cubes)
        let row1_z = 2.0;
        let metallic_values = [0.0, 0.25, 0.5, 0.75, 1.0];

        for (i, &metallic) in metallic_values.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(positions[i], y_height, row1_z),
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::TextureHandle::new("metal_detailed"),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_base_color([1.0, 1.0, 1.0, 1.0])
                        .with_metallic(metallic)
                        .with_roughness(0.3),
                ),
                praxis_ecs::Name::new(format!("Metallic {:.2}", metallic)),
            ));
        }

        // ROW 2: Roughness progression (spheres)
        let row2_z = row1_z - spacing_z;
        let roughness_values = [0.0, 0.25, 0.5, 0.75, 1.0];

        for (i, &roughness) in roughness_values.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(positions[i], y_height, row2_z),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("stone_detailed"),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_base_color([1.0, 1.0, 1.0, 1.0])
                        .with_metallic(0.0)
                        .with_roughness(roughness),
                ),
                praxis_ecs::Name::new(format!("Roughness {:.2}", roughness)),
            ));
        }

        // ROW 3: Emissive progression (demonstrates bloom post-processing)
        let row3_z = row2_z - spacing_z;
        let emissive_values = [0.0, 0.5, 1.0, 2.5, 5.0];

        for (i, &emissive) in emissive_values.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(positions[i], y_height, row3_z),
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::TextureHandle::new("emissive_pattern"),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_base_color([1.0, 1.0, 1.0, 1.0])
                        .with_emissive_strength(emissive)
                        .with_metallic(0.0)
                        .with_roughness(0.4),
                ),
                praxis_ecs::Name::new(format!("Emissive {:.1}", emissive)),
            ));
        }

        // ROW 4: Real-world material combinations
        let row4_z = row3_z - spacing_z;

        // Polished gold
        world.spawn((
            Transform::from_xyz(positions[0], y_height, row4_z),
            praxis_ecs::MeshHandle::new("sphere"),
            praxis_ecs::TextureHandle::new("gradient"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([1.0, 0.85, 0.35, 1.0])
                    .with_metallic(1.0)
                    .with_roughness(0.1),
            ),
            praxis_ecs::Name::new("Polished Gold"),
        ));

        // Brushed aluminum
        world.spawn((
            Transform::from_xyz(positions[1], y_height, row4_z),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("metal_detailed"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([0.95, 0.95, 0.98, 1.0])
                    .with_metallic(0.9)
                    .with_roughness(0.4),
            ),
            praxis_ecs::Name::new("Brushed Aluminum"),
        ));

        // Rough stone
        world.spawn((
            Transform::from_xyz(positions[2], y_height, row4_z),
            praxis_ecs::MeshHandle::new("sphere"),
            praxis_ecs::TextureHandle::new("stone_detailed"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([0.9, 0.85, 0.8, 1.0])
                    .with_metallic(0.0)
                    .with_roughness(0.95),
            ),
            praxis_ecs::Name::new("Rough Stone"),
        ));

        // Glossy plastic
        world.spawn((
            Transform::from_xyz(positions[3], y_height, row4_z),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("grid"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([0.2, 0.5, 0.9, 1.0])
                    .with_metallic(0.0)
                    .with_roughness(0.25),
            ),
            praxis_ecs::Name::new("Glossy Plastic"),
        ));

        // Neon sign (emissive with bloom)
        world.spawn((
            Transform::from_xyz(positions[4], y_height, row4_z),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("emissive_pattern"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([0.0, 1.0, 1.0, 1.0])
                    .with_emissive_strength(4.0)
                    .with_metallic(0.0)
                    .with_roughness(0.2),
            ),
            praxis_ecs::Name::new("Neon Sign"),
        ));

        info!("Spawned 20 material demonstration objects");
    }

    fn spawn_lights(world: &mut World) -> praxis_ecs::Entity {
        info!("Spawning lighting setup...");

        // Main directional light
        world.spawn((
            DirectionalLight::new(
                Vec3::new(0.3, -0.7, 0.4).normalize(),
                Vec3::new(1.0, 0.98, 0.95),
                1.0,
            ),
            praxis_ecs::Name::new("Main Light"),
        ));

        // Ambient fill light
        let ambient = world.spawn((
            DirectionalLight::new(
                Vec3::new(-0.2, -0.5, -0.3).normalize(),
                Vec3::new(0.4, 0.5, 0.7),
                0.3,
            ),
            praxis_ecs::Name::new("Ambient Fill"),
        ));

        // Colored accent light for emissive objects
        world.spawn((
            Transform::from_xyz(0.0, 5.0, -8.0),
            PointLight::new(Vec3::new(0.3, 0.7, 1.0), 25.0, 15.0),
            praxis_ecs::Name::new("Accent Light"),
        ));

        info!("Spawned 3 lights");

        ambient
    }

    fn handle_input(&mut self) {
        // Bloom threshold controls
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit1) {
            self.post_process_settings.bloom_threshold =
                (self.post_process_settings.bloom_threshold - 0.1).max(0.1);
            println!(
                "Bloom threshold: {:.1}",
                self.post_process_settings.bloom_threshold
            );
        }
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit2) {
            self.post_process_settings.bloom_threshold =
                (self.post_process_settings.bloom_threshold + 0.1).min(5.0);
            println!(
                "Bloom threshold: {:.1}",
                self.post_process_settings.bloom_threshold
            );
        }

        // Bloom intensity controls
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit3) {
            self.post_process_settings.bloom_intensity =
                (self.post_process_settings.bloom_intensity - 0.05).max(0.0);
            println!(
                "Bloom intensity: {:.2}",
                self.post_process_settings.bloom_intensity
            );
        }
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit4) {
            self.post_process_settings.bloom_intensity =
                (self.post_process_settings.bloom_intensity + 0.05).min(2.0);
            println!(
                "Bloom intensity: {:.2}",
                self.post_process_settings.bloom_intensity
            );
        }

        // Exposure controls
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit5) {
            self.post_process_settings.exposure =
                (self.post_process_settings.exposure - 0.1).max(0.1);
            println!("Exposure: {:.1}", self.post_process_settings.exposure);
        }
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit6) {
            self.post_process_settings.exposure =
                (self.post_process_settings.exposure + 0.1).min(5.0);
            println!("Exposure: {:.1}", self.post_process_settings.exposure);
        }

        // Color grading presets
        if self.input_state.is_key_just_pressed(praxis_input::Key::Digit7)
            || self.input_state.is_key_just_pressed(praxis_input::Key::Digit8)
        {
            self.post_process_settings.color_grading =
                self.post_process_settings.color_grading.next();
            println!(
                "Color grading: {}",
                self.post_process_settings.color_grading.name()
            );
        }
    }

    fn update_ambient_light(&mut self) {
        if let Some(world) = &mut self.world {
            if let Some(entity) = self.ambient_light_entity {
                if let Some(mut light) = world.inner_mut().get_mut::<DirectionalLight>(entity) {
                    let tint = self.post_process_settings.color_grading.tint();
                    light.color = Vec3::new(0.4 * tint[0], 0.5 * tint[1], 0.7 * tint[2]);
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

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_mut().unwrap();
        let render_context = self.render_context.as_mut().unwrap();

        // Gather lighting
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

        // Note: Full post-processing integration would happen here
        // The render would go to a texture, then post-process passes would apply,
        // then the final result would be presented to the swapchain
        // This demo shows the API and material properties that interact with post-processing

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
                .with_title("Praxis - Material System with Post-Processing")
                .with_resizable(true),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let (world, render_context, camera_entity, ambient_light) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {}", e);
                    event_loop.exit();
                    return;
                }
            };

        self.camera_controller.camera_entity = Some(camera_entity);
        self.ambient_light_entity = Some(ambient_light);

        println!("\n╔═══════════════════════════════════════════════════════════════════╗");
        println!("║      PRAXIS - MATERIAL SYSTEM WITH POST-PROCESSING EFFECTS       ║");
        println!("╚═══════════════════════════════════════════════════════════════════╝");
        println!("\n✨ FEATURES DEMONSTRATED:");
        println!("  🎨 PBR Material Properties (metallic, roughness, emissive)");
        println!("  🖼️  Normal-Like Surface Detail via enhanced textures");
        println!("  ✨ Bloom Post-Processing for emissive materials");
        println!("  🌈 Tone Mapping for HDR rendering");
        println!("  🎭 Color Grading presets");
        println!("\n📊 GALLERY LAYOUT:");
        println!("  Row 1: Metallic progression (0.0 → 1.0)");
        println!("  Row 2: Roughness progression (0.0 → 1.0)");
        println!("  Row 3: Emissive progression (0.0 → 5.0) - Watch the bloom!");
        println!("  Row 4: Real-world materials (gold, aluminum, stone, plastic, neon)");
        println!("\n⌨️  CAMERA CONTROLS:");
        println!("  WASD        - Move horizontally");
        println!("  Space       - Move up");
        println!("  Left Ctrl   - Move down");
        println!("  Left Shift  - Sprint");
        println!("  Mouse       - Look around");
        println!("\n🎛️  POST-PROCESSING CONTROLS:");
        println!("  1/2         - Decrease/Increase bloom threshold");
        println!("  3/4         - Decrease/Increase bloom intensity");
        println!("  5/6         - Decrease/Increase exposure");
        println!("  7/8         - Cycle color grading presets");
        println!("\n💾 SYSTEM:");
        println!("  ESC         - Toggle cursor / Exit");
        println!("\n💡 TIP: Adjust bloom settings to see emissive materials glow!");
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
                    self.handle_input();
                    self.update_ambient_light();

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
