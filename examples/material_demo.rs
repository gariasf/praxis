//! Material system demonstration showcasing various PBR material properties.
//!
//! This example demonstrates the complete material system with extensive comments
//! explaining each material setup. It showcases:
//!
//! - **Metallic materials**: Shiny metal cubes with high metallic values
//! - **Rough materials**: Rough stone spheres with high roughness values
//! - **Emissive materials**: Glowing objects with emissive strength
//! - **Textured materials**: Different textures with varying material properties
//! - **Material batching**: Efficient rendering when multiple objects share materials
//!
//! The demo creates a gallery of objects demonstrating different material properties:
//!
//! **Row 1 (Metallic)**: Cubes with varying metallic values (0.0 to 1.0)
//!   - Non-metallic (plastic-like) to fully metallic
//!   - Shows how metallic affects specular reflections
//!
//! **Row 2 (Roughness)**: Spheres with varying roughness values (0.0 to 1.0)
//!   - Perfectly smooth (mirror-like) to completely rough (diffuse)
//!   - Shows how roughness affects surface smoothness
//!
//! **Row 3 (Emissive)**: Objects with emissive properties
//!   - Self-illuminating objects that glow regardless of lighting
//!   - Useful for light sources, signs, UI elements
//!
//! **Row 4 (Combined)**: Objects combining multiple properties
//!   - Demonstrates realistic material combinations
//!   - Shows how properties interact
//!
//! # Material Property Explanation
//!
//! ## Metallic (0.0 to 1.0)
//! - **0.0**: Non-metallic (dielectric) - plastic, wood, stone
//! - **1.0**: Fully metallic - gold, silver, copper
//! - Controls whether the material reflects light like a metal or insulator
//!
//! ## Roughness (0.0 to 1.0)
//! - **0.0**: Perfectly smooth - mirror, polished metal
//! - **1.0**: Completely rough - matte paint, rough stone
//! - Controls the smoothness/glossiness of the surface
//!
//! ## Emissive Strength (0.0+)
//! - **0.0**: No emission - normal object
//! - **1.0+**: Glowing object - light source, neon sign
//! - Adds color regardless of lighting (self-illumination)
//!
//! ## Base Color
//! - RGBA tint multiplied with texture color
//! - Use white [1,1,1,1] to show texture colors unchanged
//! - Use colored tints to modify texture appearance
//!
//! # Controls
//!
//! - WASD - Move camera horizontally
//! - Space/Left Ctrl - Move camera up/down
//! - Left Shift - Sprint (faster movement)
//! - Mouse - Look around (when cursor locked)
//! - ESC - Toggle cursor lock / Exit (when unlocked)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example material_demo
//! ```

use praxis_ecs::{PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    sphere_mesh, textured_cube_mesh, DrawCommand, MaterialProperties, RenderCommands, RenderContext,
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
            move_speed: 5.0,
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
        info!("Setting up material demo scene");

        let mut render_context = RenderContext::new(window.clone()).await?;

        info!("Loading assets...");
        Self::load_assets(&mut render_context)?;

        let mut world = World::new();
        Self::spawn_material_gallery(&mut world);

        // Create camera positioned to view the gallery
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 3.0, 12.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));
        info!("Created camera entity: {:?}", camera_entity);

        Ok((world, render_context, camera_entity))
    }

    fn load_assets(render_context: &mut RenderContext) -> Result<()> {
        info!("Loading meshes...");

        // ===================================================================
        // MESH LOADING
        // ===================================================================
        // Load the meshes we'll use for the demo. We use:
        // - Cubes for metallic demonstrations (flat faces show reflections well)
        // - Spheres for roughness demonstrations (curved surface shows gradients)

        render_context
            .mesh_manager_mut()
            .load_mesh("cube", textured_cube_mesh([1.0, 1.0, 1.0]))?;

        // Generate a UV sphere with good tessellation for smooth lighting
        // sectors=36, stacks=18 gives a good balance of detail and performance
        render_context
            .mesh_manager_mut()
            .load_mesh("sphere", sphere_mesh(0.5, 36, 18, [1.0, 1.0, 1.0]))?;

        info!("Creating procedural textures...");

        // ===================================================================
        // TEXTURE CREATION
        // ===================================================================
        // Create various procedural textures to demonstrate how materials
        // interact with texture data. Each texture shows different patterns
        // that help visualize how material properties affect appearance.

        // METAL TEXTURE: Fine noise pattern simulating brushed metal
        // - High-frequency noise creates micro-surface detail
        // - Gray values work well with metallic materials
        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "metal",
            64,
            64,
            |x, y| {
                let noise = ((x * 7 + y * 13) % 40) as u8;
                let base = 160 + noise;
                [base, base, base + 20, 255] // Slightly blue-tinted metal
            },
        )?;

        // STONE TEXTURE: Natural stone pattern with variation
        // - Lower-frequency noise creates rocky appearance
        // - Works well with high roughness values
        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "stone",
            64,
            64,
            |x, y| {
                let noise = ((x * 3 + y * 7) % 60) as u8;
                let base = 100 + noise;
                [base, base - 10, base - 20, 255] // Gray-brown stone
            },
        )?;

        // GRID TEXTURE: High-contrast grid for visualizing distortion
        // - Sharp lines make reflections and roughness changes obvious
        // - Useful for debugging material properties
        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "grid",
            64,
            64,
            |x, y| {
                let grid_size = 8;
                let is_line = x % grid_size == 0 || y % grid_size == 0;
                if is_line {
                    [255, 255, 255, 255] // White lines
                } else {
                    [50, 50, 50, 255] // Dark gray background
                }
            },
        )?;

        // GRADIENT TEXTURE: Smooth color gradient
        // - Shows how materials affect color perception
        // - Red to yellow gradient demonstrates color preservation
        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "gradient",
            64,
            64,
            |x, _y| {
                let t = x as f32 / 64.0;
                let r = (255.0 * (1.0 - t * 0.5)) as u8;
                let g = (255.0 * t) as u8;
                [r, g, 0, 255] // Red to yellow gradient
            },
        )?;

        // EMISSIVE TEXTURE: Bright colors for glowing objects
        // - High-intensity colors work well with emissive materials
        // - Cyan color creates a neon-like glow effect
        Self::create_procedural_texture(
            render_context.texture_manager_mut(),
            "emissive",
            64,
            64,
            |x, y| {
                let checker_size = 16;
                let is_light = ((x / checker_size) + (y / checker_size)) % 2 == 0;
                if is_light {
                    [0, 255, 255, 255] // Bright cyan
                } else {
                    [0, 128, 128, 255] // Darker cyan
                }
            },
        )?;

        info!(
            "Assets loaded: {} meshes, {} textures",
            render_context.mesh_manager().mesh_count(),
            5
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

    fn spawn_material_gallery(world: &mut World) {
        info!("Spawning material gallery...");

        // ===================================================================
        // MATERIAL GALLERY LAYOUT
        // ===================================================================
        // The gallery is organized in rows, each demonstrating a different
        // material property. Objects are spaced evenly for easy comparison.
        //
        // Layout (top view):
        //
        //    Row 1: [-6, -3, 0, 3, 6] - Metallic variation (cubes)
        //    Row 2: [-6, -3, 0, 3, 6] - Roughness variation (spheres)
        //    Row 3: [-6, -3, 0, 3, 6] - Emissive variation (cubes)
        //    Row 4: [-6, -3, 0, 3, 6] - Combined properties (mixed)
        //
        // Z-spacing: 3 units between rows
        // X-spacing: 3 units between objects in each row

        let y_height = 1.0; // All objects at same height
        let spacing_x = 3.0; // Horizontal spacing between objects
        let spacing_z = 3.0; // Depth spacing between rows

        // ===================================================================
        // ROW 1: METALLIC PROPERTY DEMONSTRATION
        // ===================================================================
        // The metallic property controls whether a surface behaves like a
        // metal or a dielectric (non-metal). This affects how light reflects:
        //
        // - Metallic = 0.0: Dielectric behavior (plastic, wood, stone)
        //   * Diffuse reflections dominate
        //   * Specular highlights are white/colored by light source
        //   * Base color affects diffuse component
        //
        // - Metallic = 1.0: Metallic behavior (gold, silver, copper)
        //   * Specular reflections dominate
        //   * Reflections are colored by the base color
        //   * Little to no diffuse component
        //
        // We use cubes with the same texture but varying metallic values
        // to show how metallicness affects appearance.

        let row1_z = 0.0;
        let metallics = [0.0, 0.25, 0.5, 0.75, 1.0];
        let positions = [-6.0, -3.0, 0.0, 3.0, 6.0];

        for (i, &metallic) in metallics.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(positions[i], y_height, row1_z),
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::TextureHandle::new("metal"),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_metallic(metallic)
                        .with_roughness(0.3), // Moderate roughness to show reflections
                ),
            ));
        }

        // ===================================================================
        // ROW 2: ROUGHNESS PROPERTY DEMONSTRATION
        // ===================================================================
        // The roughness property controls the micro-surface detail that
        // affects how light scatters:
        //
        // - Roughness = 0.0: Perfectly smooth (mirror-like)
        //   * Sharp, focused reflections
        //   * Clear mirror reflections of environment
        //   * High specular highlights
        //
        // - Roughness = 1.0: Completely rough (matte/diffuse)
        //   * Scattered, blurred reflections
        //   * No clear reflections
        //   * Broad, soft highlights
        //
        // We use spheres because their curved surface shows the gradual
        // transition of reflections across the surface better than flat faces.

        let row2_z = row1_z - spacing_z;
        let roughnesses = [0.0, 0.25, 0.5, 0.75, 1.0];

        for (i, &roughness) in roughnesses.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(positions[i], y_height, row2_z),
                praxis_ecs::MeshHandle::new("sphere"),
                praxis_ecs::TextureHandle::new("stone"),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_metallic(0.0) // Non-metallic to focus on roughness
                        .with_roughness(roughness),
                ),
            ));
        }

        // ===================================================================
        // ROW 3: EMISSIVE PROPERTY DEMONSTRATION
        // ===================================================================
        // The emissive property makes objects self-illuminating, adding
        // color regardless of scene lighting:
        //
        // - Emissive = 0.0: Normal object (affected only by lights)
        //   * Appears dark in shadow
        //   * Brightness depends on lighting
        //
        // - Emissive > 0.0: Self-illuminating object
        //   * Always visible even in darkness
        //   * Adds constant color to final output
        //   * Useful for: light sources, neon signs, UI elements, glowing effects
        //
        // Higher emissive values create stronger glow effects. The emissive
        // color is multiplied by the base color and added to the final pixel color.

        let row3_z = row2_z - spacing_z;
        let emissive_strengths = [0.0, 0.5, 1.0, 2.0, 5.0];

        for (i, &emissive) in emissive_strengths.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(positions[i], y_height, row3_z),
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::TextureHandle::new("emissive"),
                praxis_ecs::MaterialPropertiesComponent(
                    MaterialProperties::new()
                        .with_emissive_strength(emissive)
                        .with_metallic(0.0)
                        .with_roughness(0.5),
                ),
            ));
        }

        // ===================================================================
        // ROW 4: COMBINED PROPERTIES - REALISTIC MATERIALS
        // ===================================================================
        // Real-world materials combine multiple properties. This row shows
        // common material combinations you'd use in actual games:

        let row4_z = row3_z - spacing_z;

        // 1. POLISHED GOLD: High metallic + low roughness
        //    Creates shiny, reflective metal appearance
        //    Typical of: polished metal, jewelry, chrome
        world.spawn((
            Transform::from_xyz(positions[0], y_height, row4_z),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("gradient"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([1.0, 0.8, 0.3, 1.0]) // Gold color
                    .with_metallic(1.0) // Fully metallic
                    .with_roughness(0.1), // Very smooth
            ),
        ));

        // 2. BRUSHED METAL: High metallic + moderate roughness
        //    Creates realistic brushed/satin metal finish
        //    Typical of: aluminum panels, steel appliances, tools
        world.spawn((
            Transform::from_xyz(positions[1], y_height, row4_z),
            praxis_ecs::MeshHandle::new("sphere"),
            praxis_ecs::TextureHandle::new("metal"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_metallic(0.9)
                    .with_roughness(0.4), // Brushed finish
            ),
        ));

        // 3. ROUGH STONE: Low metallic + high roughness
        //    Creates matte, diffuse surface
        //    Typical of: concrete, unpolished stone, clay, fabric
        world.spawn((
            Transform::from_xyz(positions[2], y_height, row4_z),
            praxis_ecs::MeshHandle::new("sphere"),
            praxis_ecs::TextureHandle::new("stone"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_metallic(0.0) // Non-metallic
                    .with_roughness(0.9), // Very rough
            ),
        ));

        // 4. PLASTIC: Low metallic + low-to-moderate roughness
        //    Creates typical plastic appearance with some shine
        //    Typical of: toys, containers, keyboards
        world.spawn((
            Transform::from_xyz(positions[3], y_height, row4_z),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("grid"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([0.2, 0.4, 0.8, 1.0]) // Blue plastic
                    .with_metallic(0.0)
                    .with_roughness(0.3), // Slightly glossy
            ),
        ));

        // 5. GLOWING SIGN: Emissive + low roughness
        //    Creates neon sign or holographic effect
        //    Typical of: neon signs, screens, holograms, sci-fi elements
        world.spawn((
            Transform::from_xyz(positions[4], y_height, row4_z),
            praxis_ecs::MeshHandle::new("cube"),
            praxis_ecs::TextureHandle::new("emissive"),
            praxis_ecs::MaterialPropertiesComponent(
                MaterialProperties::new()
                    .with_base_color([0.0, 1.0, 1.0, 1.0]) // Cyan
                    .with_emissive_strength(3.0) // Strong glow
                    .with_metallic(0.0)
                    .with_roughness(0.2), // Slightly glossy surface
            ),
        ));

        info!("Spawned 20 material demonstration objects in 4 rows");
        println!("\n=== Material Gallery Layout ===");
        println!("Row 1 (Z=0.0):  Metallic variation [0.0 → 1.0]");
        println!("Row 2 (Z=-3.0): Roughness variation [0.0 → 1.0]");
        println!("Row 3 (Z=-6.0): Emissive variation [0.0 → 5.0]");
        println!("Row 4 (Z=-9.0): Realistic material combinations");
        println!("                1. Polished gold");
        println!("                2. Brushed metal");
        println!("                3. Rough stone");
        println!("                4. Plastic");
        println!("                5. Glowing sign");
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

        // ===================================================================
        // RENDER COMMAND COLLECTION
        // ===================================================================
        // Query all renderable entities and build draw commands. The
        // renderer will automatically sort these by material properties
        // to minimize GPU state changes and descriptor set binds.
        //
        // Material batching benefits:
        // - Objects with identical materials use the same descriptor set
        // - Reduces descriptor set allocations from 20 to ~10
        // - Reduces GPU bind operations significantly
        // - Improves texture cache coherency

        let mut query = world.inner_mut().query::<(
            &Transform,
            &praxis_ecs::MeshHandle,
            &praxis_ecs::TextureHandle,
            &praxis_ecs::MaterialPropertiesComponent,
        )>();

        for (transform, mesh_handle, texture_handle, material_props) in query.iter(world.inner()) {
            draw_commands.push(DrawCommand {
                mesh_id: mesh_handle.id.clone(),
                model: transform.compute_matrix(),
                texture_name: Some(texture_handle.id.clone()),
                material_properties: Some(material_props.0),
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
                .with_title("Praxis - Material System Demo")
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

        println!("\n=== Praxis Material System Demo ===");
        println!("\nThis demo showcases various PBR material properties:");
        println!("  • Metallic: How metal-like the surface is (0=plastic, 1=metal)");
        println!("  • Roughness: Surface smoothness (0=mirror, 1=matte)");
        println!("  • Emissive: Self-illumination strength");
        println!("  • Combined: Realistic material combinations");
        println!("\nControls:");
        println!("  WASD - Move camera");
        println!("  Space - Move up");
        println!("  Left Ctrl - Move down");
        println!("  Left Shift - Sprint");
        println!("  Mouse - Look around");
        println!("  ESC - Toggle cursor / Exit");
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
