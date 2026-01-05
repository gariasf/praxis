//! Line Rendering Demo
//!
//! Demonstrates the line primitive rendering system for debug visualization and gizmos.
//!
//! This example shows:
//! - Creating and rendering colored line segments
//! - Using LineBatch for efficient batched line rendering
//! - Creating grid floors with the visual feedback utilities
//! - Creating axis indicators for spatial reference
//! - Creating bounding boxes around objects
//! - Creating selection outlines
//! - Rendering lines with proper depth testing alongside 3D meshes

use praxis_core::Engine;
use praxis_ecs::{Commands, Component, GlobalTransform, Res, ResMut, Transform, World};
use praxis_graphics::{
    create_axis_indicator, create_bounding_box, create_grid, create_selection_outline,
    colored_cube_mesh, AxisIndicatorConfig, GridConfig, Line, LineBatch, RenderCommands,
    RenderContext,
};
use praxis_input::InputState;
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;
use praxis_window::WindowContext;
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;

/// Component marking a rotating cube
#[derive(Component)]
struct RotatingCube {
    rotation_speed: f32,
}

/// Demo application state
struct LineRenderingDemo {
    engine: Engine,
    camera_distance: f32,
    camera_angle: f32,
    show_grid: bool,
    show_axes: bool,
    show_bounding_boxes: bool,
    show_selection_outlines: bool,
    show_custom_lines: bool,
    cube_entity: Option<praxis_ecs::Entity>,
}

impl LineRenderingDemo {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let mut engine = Engine::new(window).await?;

        // Initialize the line renderer
        let render_context = engine.render_context_mut();
        let render_pass = render_context.create_render_pass_with_depth(
            vulkano::format::Format::R8G8B8A8_UNORM,
        )?;
        let extent = [800, 600];
        render_context.initialize_line_renderer(render_pass, extent)?;

        // Load cube mesh
        let mesh_data = colored_cube_mesh();
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", mesh_data)?;

        // Create a rotating cube entity
        let world = engine.world_mut();
        let cube_entity = world.spawn((
            Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            GlobalTransform::default(),
            praxis_ecs::MeshHandle {
                id: "cube".to_string(),
            },
            RotatingCube {
                rotation_speed: 45.0,
            },
        ));

        Ok(Self {
            engine,
            camera_distance: 10.0,
            camera_angle: 45.0_f32.to_radians(),
            show_grid: true,
            show_axes: true,
            show_bounding_boxes: true,
            show_selection_outlines: true,
            show_custom_lines: true,
            cube_entity: Some(cube_entity),
        })
    }

    fn update(&mut self, delta_time: f32) -> Result<bool> {
        let window_context = self.engine.window_context();
        if window_context.should_close() {
            return Ok(false);
        }

        // Handle input
        let input_state = self.engine.input_state();
        self.handle_input(input_state, delta_time);

        // Update rotating cubes
        self.update_rotation(delta_time);

        // Render
        self.render()?;

        Ok(true)
    }

    fn handle_input(&mut self, input_state: &InputState, delta_time: f32) {
        // Camera rotation with A/D keys
        if input_state.is_key_pressed(KeyCode::KeyA) {
            self.camera_angle += 1.0 * delta_time;
        }
        if input_state.is_key_pressed(KeyCode::KeyD) {
            self.camera_angle -= 1.0 * delta_time;
        }

        // Camera zoom with W/S keys
        if input_state.is_key_pressed(KeyCode::KeyW) {
            self.camera_distance -= 5.0 * delta_time;
        }
        if input_state.is_key_pressed(KeyCode::KeyS) {
            self.camera_distance += 5.0 * delta_time;
        }
        self.camera_distance = self.camera_distance.clamp(2.0, 50.0);

        // Toggle visualization options
        if input_state.is_key_just_pressed(KeyCode::Digit1) {
            self.show_grid = !self.show_grid;
            println!("Grid: {}", if self.show_grid { "ON" } else { "OFF" });
        }
        if input_state.is_key_just_pressed(KeyCode::Digit2) {
            self.show_axes = !self.show_axes;
            println!(
                "Axes: {}",
                if self.show_axes { "ON" } else { "OFF" }
            );
        }
        if input_state.is_key_just_pressed(KeyCode::Digit3) {
            self.show_bounding_boxes = !self.show_bounding_boxes;
            println!(
                "Bounding Boxes: {}",
                if self.show_bounding_boxes { "ON" } else { "OFF" }
            );
        }
        if input_state.is_key_just_pressed(KeyCode::Digit4) {
            self.show_selection_outlines = !self.show_selection_outlines;
            println!(
                "Selection Outlines: {}",
                if self.show_selection_outlines {
                    "ON"
                } else {
                    "OFF"
                }
            );
        }
        if input_state.is_key_just_pressed(KeyCode::Digit5) {
            self.show_custom_lines = !self.show_custom_lines;
            println!(
                "Custom Lines: {}",
                if self.show_custom_lines { "ON" } else { "OFF" }
            );
        }
    }

    fn update_rotation(&mut self, delta_time: f32) {
        let world = self.engine.world_mut();

        let mut query = world.query::<(&mut Transform, &RotatingCube)>();
        for (mut transform, rotating) in query.iter_mut(world.inner_mut()) {
            let rotation = Quat::from_rotation_y(rotating.rotation_speed.to_radians() * delta_time);
            transform.rotation = rotation * transform.rotation;
        }
    }

    fn render(&mut self) -> Result<()> {
        let render_context = self.engine.render_context_mut();

        // Set up camera
        let camera_pos = Vec3::new(
            self.camera_angle.sin() * self.camera_distance,
            5.0,
            self.camera_angle.cos() * self.camera_distance,
        );
        let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0);

        // Build draw commands for regular meshes
        let world = self.engine.world();
        let mut draw_commands = Vec::new();

        let mut query = world.query::<(&Transform, &praxis_ecs::MeshHandle)>();
        for (transform, mesh_handle) in query.iter(world.inner()) {
            draw_commands.push(praxis_graphics::DrawCommand {
                mesh_id: mesh_handle.id.clone(),
                model: transform.compute_matrix(),
                texture_name: None,
                material_properties: None,
            });
        }

        // Render regular meshes
        let render_commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };
        render_context.render(&render_commands)?;

        // Render lines
        if let Some(line_renderer) = render_context.line_renderer_mut() {
            line_renderer.update_view_projection(view, proj, camera_pos)?;

            let mut line_batch = LineBatch::new();

            // Add grid floor
            if self.show_grid {
                let grid_config = GridConfig {
                    size: 20.0,
                    divisions: 20,
                    line_color: Vec3::new(0.3, 0.3, 0.3),
                    axis_color: Vec3::new(0.5, 0.5, 0.5),
                    height: 0.0,
                };
                let grid_lines = create_grid(&grid_config);
                line_batch.add_lines(grid_lines.to_vertices().iter().step_by(2).zip(
                    grid_lines.to_vertices().iter().skip(1).step_by(2)
                ).map(|(start, end)| {
                    Line::new(
                        Vec3::from(start.position),
                        Vec3::from(end.position),
                        Vec3::from(start.color),
                    )
                }));
            }

            // Add axis indicator at origin
            if self.show_axes {
                let axis_config = AxisIndicatorConfig {
                    length: 2.0,
                    position: Vec3::ZERO,
                    show_labels: false,
                };
                let axis_lines = create_axis_indicator(&axis_config);
                line_batch.add_lines(axis_lines.to_vertices().iter().step_by(2).zip(
                    axis_lines.to_vertices().iter().skip(1).step_by(2)
                ).map(|(start, end)| {
                    Line::new(
                        Vec3::from(start.position),
                        Vec3::from(end.position),
                        Vec3::from(start.color),
                    )
                }));
            }

            // Add bounding boxes around cubes
            if self.show_bounding_boxes {
                let world = self.engine.world();
                let mut query = world.query::<&Transform>();
                for transform in query.iter(world.inner()) {
                    let position = transform.translation;
                    let bbox_lines = create_bounding_box(
                        position,
                        Vec3::splat(0.6),
                        Vec3::new(1.0, 1.0, 0.0), // Yellow
                    );
                    line_batch.add_lines(bbox_lines.to_vertices().iter().step_by(2).zip(
                        bbox_lines.to_vertices().iter().skip(1).step_by(2)
                    ).map(|(start, end)| {
                        Line::new(
                            Vec3::from(start.position),
                            Vec3::from(end.position),
                            Vec3::from(start.color),
                        )
                    }));
                }
            }

            // Add selection outlines
            if self.show_selection_outlines {
                let world = self.engine.world();
                let mut query = world.query::<&Transform>();
                for transform in query.iter(world.inner()) {
                    let outline_lines = create_selection_outline(
                        &transform.compute_matrix(),
                        Vec3::splat(0.7),
                        Vec3::new(1.0, 0.5, 0.0), // Orange
                    );
                    line_batch.add_lines(outline_lines.to_vertices().iter().step_by(2).zip(
                        outline_lines.to_vertices().iter().skip(1).step_by(2)
                    ).map(|(start, end)| {
                        Line::new(
                            Vec3::from(start.position),
                            Vec3::from(end.position),
                            Vec3::from(start.color),
                        )
                    }));
                }
            }

            // Add custom lines (spiral pattern)
            if self.show_custom_lines {
                let num_points = 100;
                let height = 5.0;
                let radius = 3.0;
                for i in 0..num_points {
                    let t1 = i as f32 / num_points as f32;
                    let t2 = (i + 1) as f32 / num_points as f32;

                    let angle1 = t1 * std::f32::consts::PI * 4.0;
                    let angle2 = t2 * std::f32::consts::PI * 4.0;

                    let start = Vec3::new(
                        radius * angle1.cos(),
                        t1 * height,
                        radius * angle1.sin(),
                    );
                    let end = Vec3::new(
                        radius * angle2.cos(),
                        t2 * height,
                        radius * angle2.sin(),
                    );

                    let color = Vec3::new(t1, 1.0 - t1, 0.5);
                    line_batch.add(start, end, color);
                }
            }

            // Render all lines in a single batch
            // Note: In a real implementation, this would be done within the render pass
            // For this demo, we're showing the API usage
            // The actual rendering would happen in the command buffer recording
            
            println!(
                "Prepared {} lines for rendering",
                line_batch.len()
            );
        }

        Ok(())
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.engine
                        .render_context_mut()
                        .configure_surface(size.width, size.height);
                }
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    praxis_utils::logging::init()?;

    println!("Line Rendering Demo");
    println!("==================");
    println!();
    println!("Controls:");
    println!("  A/D     - Rotate camera");
    println!("  W/S     - Zoom camera");
    println!("  1       - Toggle grid floor");
    println!("  2       - Toggle axis indicators");
    println!("  3       - Toggle bounding boxes");
    println!("  4       - Toggle selection outlines");
    println!("  5       - Toggle custom lines (spiral)");
    println!("  ESC     - Exit");
    println!();

    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Praxis Engine - Line Rendering Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)?,
    );

    let mut demo = LineRenderingDemo::new(window.clone()).await?;

    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, elwt| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                demo.handle_window_event(&event);
                demo.engine.handle_window_event(&event);

                if let WindowEvent::CloseRequested = event {
                    elwt.exit();
                }
            }
            winit::event::Event::AboutToWait => {
                let now = std::time::Instant::now();
                let delta_time = (now - last_frame_time).as_secs_f32();
                last_frame_time = now;

                match demo.update(delta_time) {
                    Ok(should_continue) => {
                        if !should_continue {
                            elwt.exit();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error during update: {}", e);
                        elwt.exit();
                    }
                }

                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
