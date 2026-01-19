//! Demonstrates the rendering optimization panel with real-time performance comparison.
//!
//! This example shows how to:
//! - Use the OptimizationPanel for configuring rendering optimizations
//! - Compare performance metrics before and after toggling optimizations
//! - Use preset configurations (Low/Medium/High/Ultra)
//! - View live statistics and performance graphs

use praxis_core::{Engine, EngineBuilder};
use praxis_ecs::{GlobalTransform, MeshHandle, Name, Transform, World};
use praxis_editor::{EditorState, OptimizationPanel};
use praxis_graphics::{MaterialHandle, RenderContext, RenderStats};
use praxis_input::InputState;
use praxis_math::{Vec3, Quat};
use praxis_utils::Result;
use praxis_window::{Window, WindowBuilder};
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<()> {
    praxis_utils::init_tracing()?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Optimization Panel Demo")
            .with_dimensions(1280, 720)
            .build(&event_loop)?,
    );

    let mut render_context = RenderContext::new(
        Arc::clone(&window),
        window.inner_size().width,
        window.inner_size().height,
        true,
    )?;

    let mut world = World::new();
    let mut input_state = InputState::new();
    let mut egui_integration = praxis_gui::EguiIntegration::new(
        &event_loop,
        render_context.device(),
        render_context.queue(),
        render_context.swapchain_format(),
        Some(render_context.depth_format()),
        1,
        window.inner_size().width,
        window.inner_size().height,
    );

    // Create test scene with multiple objects for meaningful performance comparison
    create_test_scene(&mut world, &mut render_context)?;

    // Create optimization panel
    let mut optimization_panel = OptimizationPanel::new();

    // Frame counter for stats
    let mut frame_number = 0u64;

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                render_context.resize(physical_size.width, physical_size.height);
                egui_integration.handle_event(&window, &event);
            }
            Event::WindowEvent { ref event, .. } => {
                input_state.handle_event(event);
                egui_integration.handle_event(&window, &event);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Update frame counter
                frame_number += 1;

                // Simulate render stats (in real usage, these would come from RenderContext)
                let stats = RenderStats {
                    frame_number,
                    total_objects: 1000,
                    visible_objects: 250 + (frame_number % 50) as usize,
                    frustum_culled: 600,
                    occlusion_culled: 150,
                    draw_calls: 120 + (frame_number % 20) as usize,
                    descriptor_allocations: 15,
                    active_lod_levels: vec![(0, 50), (1, 150), (2, 50)],
                    streaming_queue_depth: 5,
                };

                // Update panel with latest stats
                optimization_panel.update_stats(stats);

                // Begin frame
                let raw_input = egui_integration.take_raw_input(&window);
                egui_integration.context().begin_frame(raw_input);

                // Render optimization panel
                egui::Window::new("Optimization Panel Demo")
                    .default_size([800.0, 600.0])
                    .resizable(true)
                    .show(egui_integration.context(), |ui| {
                        ui.heading("Rendering Optimization Configuration");
                        ui.separator();

                        ui.label("This panel allows you to:");
                        ui.label("• Toggle individual rendering optimizations");
                        ui.label("• Use preset configurations (Low/Medium/High/Ultra)");
                        ui.label("• Compare performance before/after changes");
                        ui.label("• View live performance statistics");

                        ui.add_space(10.0);
                        ui.separator();

                        // Render the optimization panel
                        optimization_panel.ui(ui, None, None);
                    });

                // End frame and render
                let full_output = egui_integration.context().end_frame();
                let paint_jobs = egui_integration.context().tessellate(
                    full_output.shapes,
                    full_output.pixels_per_point,
                );

                if let Err(e) = egui_integration.render(
                    &window,
                    &mut render_context,
                    paint_jobs,
                    &full_output.textures_delta,
                ) {
                    eprintln!("Failed to render egui: {}", e);
                }

                // Apply optimization config to render context (if available)
                if let Some(config) = optimization_panel.config() {
                    // In a real application, you would apply these settings to RenderContext
                    // For example:
                    // render_context.set_optimization_config(config.clone());
                    if config.has_changed() {
                        println!("Optimization config changed: {} optimizations enabled", 
                                 config.enabled_count());
                    }
                }

                // Present frame
                if let Err(e) = render_context.present() {
                    eprintln!("Failed to present frame: {}", e);
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}

fn create_test_scene(world: &mut World, render_context: &mut RenderContext) -> Result<()> {
    // Create a grid of test objects
    let cube_mesh = render_context.create_cube_mesh()?;
    let default_material = render_context.create_default_material()?;

    for x in -5..5 {
        for z in -5..5 {
            let entity = world.spawn((
                Name::new(format!("Cube_{}_{}", x, z)),
                Transform {
                    translation: Vec3::new(x as f32 * 3.0, 0.0, z as f32 * 3.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
                GlobalTransform::default(),
                MeshHandle::new(cube_mesh.clone()),
                MaterialHandle::new(default_material.clone()),
            ));
        }
    }

    Ok(())
}
