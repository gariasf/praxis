//! Example demonstrating OBJ mesh loading with praxis_assets.
//!
//! This example shows how to:
//! - Load OBJ files using the AssetLoader trait
//! - Upload loaded meshes to GPU via MeshAssetManager
//! - Render loaded meshes alongside procedural geometry
//!
//! Usage:
//! ```bash
//! cargo run --example obj_loader_demo
//! ```

use praxis_assets::{load_obj_mesh, AssetLoader, MeshLoader};
use praxis_graphics::{colored_cube_mesh, DrawCommand, MeshRenderCommands};
use praxis_input::InputState;
use praxis_math::{Mat4, Vec3};
use praxis_utils::Result;
use praxis_window::{State, WindowConfig};
use std::sync::Arc;

struct ObjLoaderDemo {
    rotation_angle: f32,
    rotation_speed: f32,
}

impl ObjLoaderDemo {
    fn new() -> Self {
        Self {
            rotation_angle: 0.0,
            rotation_speed: 1.0,
        }
    }
}

impl State for ObjLoaderDemo {
    fn init(&mut self, render_context: &mut praxis_graphics::RenderContext) -> Result<()> {
        println!("=== OBJ Loader Demo ===");
        println!("This example demonstrates loading OBJ mesh files using praxis_assets");
        println!();

        // Method 1: Using the high-level convenience function
        println!("Method 1: Using load_obj_mesh convenience function");
        match load_obj_mesh(
            render_context.mesh_manager_mut(),
            "obj_cube_1",
            "assets/models/cube.obj",
        ) {
            Ok(_) => println!("  ✓ Successfully loaded cube.obj as 'obj_cube_1'"),
            Err(e) => {
                println!("  ✗ Failed to load cube.obj: {}", e);
                println!("  Loading procedural cube as fallback");
                render_context
                    .mesh_manager_mut()
                    .load_mesh("obj_cube_1", colored_cube_mesh())?;
            }
        }

        // Method 2: Using AssetLoader trait directly
        println!("Method 2: Using AssetLoader trait directly");
        let loader = MeshLoader::new();
        println!("  Created MeshLoader");
        println!(
            "  Supported extensions: {:?}",
            loader.supported_extensions()
        );

        match loader.load("assets/models/cube.obj") {
            Ok(mesh_data) => {
                println!(
                    "  ✓ Loaded mesh data: {} vertices, {} indices",
                    mesh_data.positions.len(),
                    mesh_data.indices.len()
                );
                render_context
                    .mesh_manager_mut()
                    .load_mesh("obj_cube_2", mesh_data)?;
            }
            Err(e) => {
                println!("  ✗ Failed to load: {}", e);
                println!("  Loading procedural cube as fallback");
                render_context
                    .mesh_manager_mut()
                    .load_mesh("obj_cube_2", colored_cube_mesh())?;
            }
        }

        // Method 3: Loading mesh data without immediate GPU upload
        println!("Method 3: Loading mesh data for processing");
        match praxis_assets::load_obj("assets/models/cube.obj") {
            Ok(mesh_data) => {
                println!("  ✓ Loaded mesh data for processing");
                println!("    Vertices: {}", mesh_data.positions.len());
                println!("    Indices: {}", mesh_data.indices.len());
                println!("    Has normals: {}", mesh_data.normals.is_some());
                println!("    Has UVs: {}", mesh_data.uvs.is_some());
                println!("    Has colors: {}", mesh_data.colors.is_some());

                // Upload to GPU after inspection
                render_context
                    .mesh_manager_mut()
                    .load_mesh("obj_cube_3", mesh_data)?;
            }
            Err(e) => {
                println!("  ✗ Failed to load: {}", e);
                println!("  Loading procedural cube as fallback");
                render_context
                    .mesh_manager_mut()
                    .load_mesh("obj_cube_3", colored_cube_mesh())?;
            }
        }

        println!();
        println!(
            "Total meshes loaded: {}",
            render_context.mesh_manager().mesh_count()
        );
        println!();
        println!("Controls:");
        println!("  ESC - Exit");
        println!("  UP/DOWN - Adjust rotation speed");

        Ok(())
    }

    fn update(
        &mut self,
        delta_time: f32,
        _input_state: &InputState,
        _render_context: &mut praxis_graphics::RenderContext,
    ) -> Result<()> {
        self.rotation_angle += self.rotation_speed * delta_time;
        Ok(())
    }

    fn render(&mut self, render_context: &mut praxis_graphics::RenderContext) -> Result<()> {
        // Create camera matrices
        let aspect_ratio = 16.0 / 9.0;
        let fov = 45.0_f32.to_radians();
        let near = 0.1;
        let far = 100.0;

        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 2.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        let proj = Mat4::perspective_rh(fov, aspect_ratio, near, far);

        // Create draw commands for the three loaded OBJ meshes
        let draw_commands = vec![
            DrawCommand {
                mesh_id: "obj_cube_1".to_string(),
                model: Mat4::from_rotation_y(self.rotation_angle)
                    * Mat4::from_translation(Vec3::new(-2.5, 0.0, 0.0)),
            },
            DrawCommand {
                mesh_id: "obj_cube_2".to_string(),
                model: Mat4::from_rotation_x(self.rotation_angle * 1.5)
                    * Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            },
            DrawCommand {
                mesh_id: "obj_cube_3".to_string(),
                model: Mat4::from_rotation_z(self.rotation_angle * 0.7)
                    * Mat4::from_translation(Vec3::new(2.5, 0.0, 0.0)),
            },
        ];

        let cmds = MeshRenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
        };

        render_context.render_meshes(&cmds)?;

        Ok(())
    }

    fn on_input_event(&mut self, event: &praxis_input::InputEvent) -> Result<()> {
        use praxis_input::{InputEvent, KeyCode};

        match event {
            InputEvent::KeyPressed { key, .. } => match key {
                KeyCode::ArrowUp => {
                    self.rotation_speed += 0.5;
                    println!("Rotation speed: {:.1}", self.rotation_speed);
                }
                KeyCode::ArrowDown => {
                    self.rotation_speed = (self.rotation_speed - 0.5).max(0.0);
                    println!("Rotation speed: {:.1}", self.rotation_speed);
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;

    let config = WindowConfig {
        title: "Praxis - OBJ Loader Demo".to_string(),
        width: 1920,
        height: 1080,
        resizable: true,
    };

    let state = Arc::new(std::sync::Mutex::new(ObjLoaderDemo::new()));
    praxis_window::run(config, state)?;

    Ok(())
}
