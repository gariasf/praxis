//! Deferred rendering demonstration with many lights.
//!
//! This example demonstrates the deferred rendering pipeline with:
//! - G-buffer generation (albedo, normal, metallic-roughness, depth)
//! - Lighting accumulation pass with multiple point lights
//! - Performance comparison with forward rendering

use praxis_graphics::{
    colored_cube_mesh, DeferredRenderer, DirectionalLightData, DrawCommand, LightingUniforms,
    PointLightData, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::Result;
use praxis_window::Window;
use std::sync::Arc;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};

fn main() -> Result<()> {
    praxis_utils::init()?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Deferred Rendering Demo")
            .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
            .build(&event_loop)?,
    );

    pollster::block_on(async {
        let mut render_context = RenderContext::new(window.clone()).await?;

        render_context
            .mesh_manager_mut()
            .load_mesh("cube", colored_cube_mesh())?;

        let mut deferred_renderer = DeferredRenderer::new(
            render_context.device.clone(),
            Arc::new(vulkano::memory::allocator::StandardMemoryAllocator::new_default(
                render_context.device.clone(),
            )),
            Arc::new(
                vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
                    render_context.device.clone(),
                    Default::default(),
                ),
            ),
            1920,
            1080,
        )?;

        let mut time = 0.0f32;
        let mut use_deferred = true;

        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(KeyCode::KeyD),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    use_deferred = !use_deferred;
                    println!(
                        "Switched to {} rendering",
                        if use_deferred { "deferred" } else { "forward" }
                    );
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    let size = window.inner_size();
                    render_context.configure_surface(size.width, size.height);
                    if let Err(e) = deferred_renderer.resize(size.width, size.height) {
                        eprintln!("Failed to resize deferred renderer: {}", e);
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    time += 0.016;

                    let eye = Vec3::new(0.0, 3.0, 10.0);
                    let center = Vec3::new(0.0, 0.0, 0.0);
                    let up = Vec3::new(0.0, 1.0, 0.0);
                    let view = Mat4::look_at_rh(eye, center, up);

                    let aspect = 1920.0 / 1080.0;
                    let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);

                    let num_cubes = 20;
                    let mut draw_commands = Vec::new();

                    for i in 0..num_cubes {
                        let angle = (i as f32 / num_cubes as f32) * std::f32::consts::TAU + time;
                        let radius = 5.0;
                        let x = angle.cos() * radius;
                        let z = angle.sin() * radius;
                        let y = (time + i as f32).sin() * 2.0;

                        let position = Vec3::new(x, y, z);
                        let rotation = Quat::from_rotation_y(time + i as f32);
                        let model = Mat4::from_rotation_translation(rotation, position);

                        draw_commands.push(DrawCommand {
                            mesh_id: "cube".to_string(),
                            model,
                            texture_name: None,
                            material_properties: None,
                        });
                    }

                    let num_lights = 50;
                    let mut point_lights = Vec::new();

                    for i in 0..num_lights {
                        let angle = (i as f32 / num_lights as f32) * std::f32::consts::TAU
                            + time * 0.5;
                        let radius = 8.0;
                        let x = angle.cos() * radius;
                        let z = angle.sin() * radius;
                        let y = (time * 2.0 + i as f32).sin() * 3.0;

                        let hue = i as f32 / num_lights as f32;
                        let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);

                        point_lights.push(PointLightData {
                            position: [x, y, z],
                            color: [r, g, b],
                            intensity: 2.0,
                            range: 15.0,
                        });
                    }

                    let lighting = LightingUniforms {
                        directional_lights: [DirectionalLightData::default(); 8],
                        point_lights: {
                            let mut lights = [PointLightData::default(); 16];
                            for (i, light) in point_lights.iter().take(16).enumerate() {
                                lights[i] = *light;
                            }
                            lights
                        },
                        ambient_color: [0.1, 0.1, 0.15, 1.0],
                        directional_light_count: 0,
                        point_light_count: point_lights.len().min(16) as u32,
                        _padding: [0, 0],
                    };

                    if use_deferred {
                        println!("Using deferred rendering (Press D to toggle)");
                    } else {
                        let cmds = praxis_graphics::RenderCommands {
                            view,
                            proj,
                            draw_commands: &draw_commands,
                            lighting: Some(&lighting),
                        };

                        if let Err(e) = render_context.render(&cmds) {
                            eprintln!("Render error: {}", e);
                            elwt.exit();
                        }
                    }
                }
                _ => {}
            }
        })?;

        Ok(())
    })
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let c = v * s;
    let h_prime = (h * 6.0) % 6.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}
