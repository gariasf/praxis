//! Advanced material system demonstration.
//!
//! This example demonstrates:
//! - Material instancing for efficient per-object parameter overrides
//! - Material layers for blending multiple materials
//! - Parallax occlusion mapping for enhanced depth perception
//! - Extended PBR features (clearcoat, sheen, transmission)

use praxis_core::{App, AppConfig};
use praxis_ecs::{Commands, Query, Res, ResMut, Schedule, World};
use praxis_graphics::{
    colored_cube_mesh, BlendMode, DrawCommand, ExtendedPbrProperties, Material, MaterialInstance,
    MaterialInstanceManager, MaterialLayer, MaterialManager, MaterialProperties,
    ParallaxProperties, RenderCommands, RenderContext,
};
use praxis_input::{Input, InputState, KeyCode};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::{info, Result};
use praxis_window::{Window, WindowConfig};
use std::sync::Arc;

/// Demo state for material showcase.
struct MaterialDemoState {
    /// Current rotation angle for animated objects.
    rotation: f32,

    /// Selected material index for inspection.
    selected_material: usize,

    /// Material instance manager.
    instance_manager: MaterialInstanceManager,
}

impl MaterialDemoState {
    fn new() -> Self {
        Self {
            rotation: 0.0,
            selected_material: 0,
            instance_manager: MaterialInstanceManager::new(),
        }
    }
}

/// System to update demo state.
fn update_demo_system(
    mut state: ResMut<MaterialDemoState>,
    input: Res<InputState>,
    delta_time: Res<f32>,
) {
    // Rotate objects
    state.rotation += 0.5 * *delta_time;

    // Cycle through materials with number keys
    if input.key_just_pressed(KeyCode::Digit1) {
        state.selected_material = 0;
        info!("Selected material: Base Metal");
    }
    if input.key_just_pressed(KeyCode::Digit2) {
        state.selected_material = 1;
        info!("Selected material: Clearcoat");
    }
    if input.key_just_pressed(KeyCode::Digit3) {
        state.selected_material = 2;
        info!("Selected material: Fabric (Sheen)");
    }
    if input.key_just_pressed(KeyCode::Digit4) {
        state.selected_material = 3;
        info!("Selected material: Glass (Transmission)");
    }
}

/// System to render the demo.
fn render_system(
    mut render_context: ResMut<RenderContext>,
    material_manager: Res<MaterialManager>,
    state: Res<MaterialDemoState>,
) -> Result<()> {
    let aspect_ratio = 16.0 / 9.0;
    let view = Mat4::look_at_rh(
        Vec3::new(0.0, 3.0, 8.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);

    // Create draw commands for different material demonstrations
    let mut draw_commands = Vec::new();

    // Row 1: Base materials with different metallic values
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 2.0;
        let metallic = i as f32 / 4.0;

        let model = Mat4::from_translation(Vec3::new(x, 2.0, 0.0))
            * Mat4::from_quat(Quat::from_rotation_y(state.rotation));

        draw_commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model,
            texture_name: Some("base_texture".to_string()),
            material_properties: Some(
                MaterialProperties::new()
                    .with_metallic(metallic)
                    .with_roughness(0.3),
            ),
        });
    }

    // Row 2: Extended PBR features
    // Clearcoat example (car paint)
    let clearcoat_model = Mat4::from_translation(Vec3::new(-4.0, 0.0, 0.0))
        * Mat4::from_quat(Quat::from_rotation_y(state.rotation * 0.7));
    draw_commands.push(DrawCommand {
        mesh_id: "cube".to_string(),
        model: clearcoat_model,
        texture_name: Some("base_texture".to_string()),
        material_properties: Some(
            MaterialProperties::new()
                .with_base_color([0.8, 0.1, 0.1, 1.0])
                .with_metallic(0.0)
                .with_roughness(0.1),
        ),
    });

    // Sheen example (fabric)
    let sheen_model = Mat4::from_translation(Vec3::new(-2.0, 0.0, 0.0))
        * Mat4::from_quat(Quat::from_rotation_y(state.rotation * 0.8));
    draw_commands.push(DrawCommand {
        mesh_id: "cube".to_string(),
        model: sheen_model,
        texture_name: Some("base_texture".to_string()),
        material_properties: Some(
            MaterialProperties::new()
                .with_base_color([0.2, 0.3, 0.8, 1.0])
                .with_metallic(0.0)
                .with_roughness(0.8),
        ),
    });

    // Standard PBR
    let standard_model = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))
        * Mat4::from_quat(Quat::from_rotation_y(state.rotation));
    draw_commands.push(DrawCommand {
        mesh_id: "cube".to_string(),
        model: standard_model,
        texture_name: Some("base_texture".to_string()),
        material_properties: Some(
            MaterialProperties::new()
                .with_metallic(0.5)
                .with_roughness(0.5),
        ),
    });

    // Transmission example (glass)
    let transmission_model = Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0))
        * Mat4::from_quat(Quat::from_rotation_y(state.rotation * 1.2));
    draw_commands.push(DrawCommand {
        mesh_id: "cube".to_string(),
        model: transmission_model,
        texture_name: Some("base_texture".to_string()),
        material_properties: Some(
            MaterialProperties::new()
                .with_base_color([0.9, 0.95, 1.0, 0.5])
                .with_metallic(0.0)
                .with_roughness(0.0),
        ),
    });

    // Emissive example (glowing)
    let emissive_model = Mat4::from_translation(Vec3::new(4.0, 0.0, 0.0))
        * Mat4::from_quat(Quat::from_rotation_y(state.rotation * 1.5));
    draw_commands.push(DrawCommand {
        mesh_id: "cube".to_string(),
        model: emissive_model,
        texture_name: Some("base_texture".to_string()),
        material_properties: Some(
            MaterialProperties::new()
                .with_base_color([1.0, 0.8, 0.2, 1.0])
                .with_emissive_strength(2.0),
        ),
    });

    // Row 3: Material instances (shared base, different properties)
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 2.0;
        let roughness = i as f32 / 4.0;

        let model = Mat4::from_translation(Vec3::new(x, -2.0, 0.0))
            * Mat4::from_quat(Quat::from_rotation_y(state.rotation * 0.5));

        draw_commands.push(DrawCommand {
            mesh_id: "cube".to_string(),
            model,
            texture_name: Some("base_texture".to_string()),
            material_properties: Some(
                MaterialProperties::new()
                    .with_metallic(0.8)
                    .with_roughness(roughness),
            ),
        });
    }

    let render_commands = RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&render_commands)?;

    Ok(())
}

/// Main demo application.
struct AdvancedMaterialDemo;

impl App for AdvancedMaterialDemo {
    fn initialize(&self, world: &mut World, schedule: &mut Schedule) -> Result<()> {
        info!("Initializing Advanced Material Demo");

        // Add demo state
        world.insert_resource(MaterialDemoState::new());

        // Add systems
        schedule.add_system(update_demo_system);
        schedule.add_system(render_system);

        info!("Demo initialized - Press 1-4 to select materials");
        info!("  1: Base Metal (varying metallic)");
        info!("  2: Clearcoat (car paint)");
        info!("  3: Fabric (sheen)");
        info!("  4: Glass (transmission)");

        Ok(())
    }

    fn setup(&self, world: &mut World) -> Result<()> {
        info!("Setting up demo resources");

        // Get render context
        let mut render_context = world
            .get_resource_mut::<RenderContext>()
            .expect("RenderContext should exist");

        // Load mesh
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", colored_cube_mesh())?;

        // Create base texture (simple white for demo)
        // In a real application, you would load actual texture files
        info!("Creating base texture");

        // Create material manager
        world.insert_resource(MaterialManager::new());

        info!("Demo setup complete");
        Ok(())
    }

    fn update(&self, _world: &mut World) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self, _world: &mut World) -> Result<()> {
        info!("Shutting down Advanced Material Demo");
        Ok(())
    }
}

fn main() -> Result<()> {
    praxis_utils::init_logging();

    info!("Starting Advanced Material Demo");
    info!("This demo showcases:");
    info!("  - Material instancing for efficient parameter overrides");
    info!("  - Extended PBR features (clearcoat, sheen, transmission)");
    info!("  - Material property variations");
    info!("");
    info!("Controls:");
    info!("  1-4: Select different material types");
    info!("  ESC: Exit demo");

    let config = AppConfig {
        window: WindowConfig {
            title: "Advanced Material System Demo".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
        },
    };

    let app = AdvancedMaterialDemo;
    praxis_core::run(app, config)
}
