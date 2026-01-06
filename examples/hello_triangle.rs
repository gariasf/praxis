//! Hello Triangle - Minimal rendering example.
//!
//! This is the simplest possible example that demonstrates basic rendering in Praxis.
//! It shows:
//! - Creating a window and render context
//! - Loading a simple triangle mesh
//! - Setting up a basic camera
//! - Rendering a frame
//!
//! This example is designed as a starting point for learning the engine.
//! No advanced features like lighting, textures, or input are used.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example hello_triangle
//! ```

use praxis_ecs::{PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{DrawCommand, MeshData, RenderCommands, RenderContext};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

/// Creates a simple triangle mesh with vertex colors.
///
/// The triangle is centered at the origin and has three different colored vertices:
/// - Top: Red
/// - Bottom Left: Green  
/// - Bottom Right: Blue
fn create_triangle_mesh() -> MeshData {
    let positions = vec![
        [0.0, 0.5, 0.0],   // Top vertex
        [-0.5, -0.5, 0.0], // Bottom left vertex
        [0.5, -0.5, 0.0],  // Bottom right vertex
    ];

    let colors = vec![
        [1.0, 0.0, 0.0], // Red
        [0.0, 1.0, 0.0], // Green
        [0.0, 0.0, 1.0], // Blue
    ];

    let normals = vec![
        [0.0, 0.0, 1.0], // All normals point forward
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ];

    let indices = vec![0, 1, 2];

    MeshData {
        positions,
        colors: Some(colors),
        normals: Some(normals),
        uvs: None,
        tangents: None,
        indices,
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    camera_entity: Option<praxis_ecs::Entity>,
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(World, RenderContext, praxis_ecs::Entity)> {
        info!("Initializing Hello Triangle example");

        // Create the render context (handles Vulkan setup)
        let mut render_context = RenderContext::new(window.clone()).await?;

        // Load the triangle mesh
        let triangle_mesh = create_triangle_mesh();
        render_context
            .mesh_manager_mut()
            .load_mesh("triangle", triangle_mesh)?;

        // Create the world and spawn a camera
        let mut world = World::new();
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 0.0, 2.0), // Position camera back a bit
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        info!("Scene setup complete");

        Ok((world, render_context, camera_entity))
    }

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_ref().unwrap();
        let render_context = self.render_context.as_mut().unwrap();
        let camera_entity = self.camera_entity.unwrap();

        // Get camera matrices
        let camera_matrices = world
            .inner()
            .get::<praxis_ecs::CameraMatrices>(camera_entity)
            .unwrap();

        // Create a draw command for the triangle
        let draw_commands = vec![DrawCommand {
            mesh_id: "triangle".to_string(),
            model: Mat4::IDENTITY,     // Identity matrix (no transformation)
            texture_name: None,        // No texture (will use vertex colors)
            material_properties: None, // No material properties
        }];

        // Submit render commands
        let cmds = RenderCommands {
            view: camera_matrices.view,
            proj: camera_matrices.projection,
            draw_commands: &draw_commands,
            lighting: None, // No lighting
        };

        render_context.render(&cmds)?;

        Ok(())
    }

    fn update_camera_matrices(&mut self) {
        if let Some(world) = &mut self.world {
            if let Some(camera_entity) = self.camera_entity {
                let inner = world.inner_mut();

                // Get transform and projection
                if let (Some(transform), Some(projection)) = (
                    inner.get::<Transform>(camera_entity),
                    inner.get::<praxis_ecs::PerspectiveProjection>(camera_entity),
                ) {
                    // Compute view matrix (camera looks down -Z axis)
                    let view = Mat4::look_at_rh(
                        transform.translation,
                        transform.translation + (transform.rotation * Vec3::NEG_Z),
                        Vec3::Y,
                    );

                    // Compute projection matrix
                    let proj = projection.compute_matrix();

                    // Update camera matrices
                    if let Some(mut matrices) =
                        inner.get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
                    {
                        matrices.update(view, proj);
                    }
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Creating window");

        // Create window
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Hello Triangle")
                .with_resizable(false),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        // Setup scene
        let (world, render_context, camera_entity) =
            match pollster::block_on(Self::setup_scene(window.clone())) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to setup scene: {e}");
                    event_loop.exit();
                    return;
                }
            };

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.camera_entity = Some(camera_entity);

        // Update camera matrices before first render
        self.update_camera_matrices();

        println!("\n╔═══════════════════════════════════════════╗");
        println!("║       PRAXIS - HELLO TRIANGLE            ║");
        println!("╚═══════════════════════════════════════════╝");
        println!("\nA simple colored triangle is being rendered.");
        println!("This is the most basic rendering example.");
        println!("\nPress ESC or close the window to exit.\n");

        // Request first frame
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key.to_text() == Some("Escape") {
                    info!("ESC pressed, exiting");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {e}");
                    event_loop.exit();
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    // Initialize Praxis subsystems
    praxis_utils::init()?;
    praxis_ecs::init()?;

    info!("Starting Hello Triangle example");

    // Create event loop
    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    // Run application
    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("hello_triangle example requires graphics support and cannot run in headless mode");
    Ok(())
}
