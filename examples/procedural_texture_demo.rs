//! Procedural texture generation demo.
//!
//! This example demonstrates the procedural texture generation system:
//! - Creating texture graphs with noise nodes
//! - Combining operations (blend, transform, color ramp)
//! - GPU-based generation with compute shaders
//! - Automatic caching of generated textures
//! - Using procedural textures in rendering

use praxis_graphics::{
    colored_cube_mesh, DrawCommand, ProceduralTextureManager, RenderCommands, RenderContext,
};
use praxis_math::{Mat4, Quat, Vec2, Vec3};
use praxis_procedural::{
    BlendMode, ColorRamp, ColorStop, NoiseType, TextureGenerationParams, TextureGraph,
    TextureNode, TransformParams,
};
use praxis_utils::Result;
use praxis_window::WindowManager;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

/// Creates a simple Perlin noise texture graph.
fn create_perlin_graph() -> TextureGraph {
    let mut graph = TextureGraph::new();
    let noise_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Perlin,
        scale: 8.0,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    });
    graph.set_output(noise_id);
    graph
}

/// Creates a Worley (cellular) noise texture graph.
fn create_worley_graph() -> TextureGraph {
    let mut graph = TextureGraph::new();
    let noise_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Worley,
        scale: 16.0,
        octaves: 1,
        persistence: 0.5,
        lacunarity: 2.0,
    });
    graph.set_output(noise_id);
    graph
}

/// Creates a colorized marble texture using Perlin noise with a color ramp.
fn create_marble_graph() -> TextureGraph {
    let mut graph = TextureGraph::new();

    let noise_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Perlin,
        scale: 12.0,
        octaves: 6,
        persistence: 0.6,
        lacunarity: 2.0,
    });

    let power_id = graph.add_node(TextureNode::Power {
        input: noise_id,
        exponent: 2.0,
    });

    let ramp = ColorRamp::new(vec![
        ColorStop {
            position: 0.0,
            color: [0.2, 0.1, 0.05, 1.0],
        },
        ColorStop {
            position: 0.3,
            color: [0.9, 0.8, 0.7, 1.0],
        },
        ColorStop {
            position: 0.7,
            color: [0.6, 0.5, 0.4, 1.0],
        },
        ColorStop {
            position: 1.0,
            color: [0.3, 0.2, 0.15, 1.0],
        },
    ]);

    let ramp_id = graph.add_node(TextureNode::ColorRamp {
        input: power_id,
        ramp,
    });

    graph.set_output(ramp_id);
    graph
}

/// Creates a wood grain texture using stretched Perlin noise.
fn create_wood_graph() -> TextureGraph {
    let mut graph = TextureGraph::new();

    let noise_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Perlin,
        scale: 20.0,
        octaves: 5,
        persistence: 0.5,
        lacunarity: 2.0,
    });

    let transform_id = graph.add_node(TextureNode::Transform {
        input: noise_id,
        params: TransformParams {
            offset: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::new(1.0, 8.0),
        },
    });

    let ramp = ColorRamp::new(vec![
        ColorStop {
            position: 0.0,
            color: [0.3, 0.15, 0.05, 1.0],
        },
        ColorStop {
            position: 0.5,
            color: [0.5, 0.3, 0.1, 1.0],
        },
        ColorStop {
            position: 1.0,
            color: [0.2, 0.1, 0.03, 1.0],
        },
    ]);

    let ramp_id = graph.add_node(TextureNode::ColorRamp {
        input: transform_id,
        ramp,
    });

    graph.set_output(ramp_id);
    graph
}

/// Creates a cloud texture by blending multiple noise layers.
fn create_cloud_graph() -> TextureGraph {
    let mut graph = TextureGraph::new();

    let noise1_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Perlin,
        scale: 4.0,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    });

    let noise2_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Simplex,
        scale: 8.0,
        octaves: 4,
        persistence: 0.5,
        lacunarity: 2.0,
    });

    let blend_id = graph.add_node(TextureNode::Blend {
        input_a: noise1_id,
        input_b: noise2_id,
        mode: BlendMode::Add,
        factor: 0.5,
    });

    let power_id = graph.add_node(TextureNode::Power {
        input: blend_id,
        exponent: 1.5,
    });

    let ramp = ColorRamp::new(vec![
        ColorStop {
            position: 0.0,
            color: [0.7, 0.8, 0.9, 1.0],
        },
        ColorStop {
            position: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        },
    ]);

    let ramp_id = graph.add_node(TextureNode::ColorRamp {
        input: power_id,
        ramp,
    });

    graph.set_output(ramp_id);
    graph
}

#[pollster::main]
async fn main() -> Result<()> {
    praxis_utils::init_logging()?;

    let event_loop = EventLoop::new()?;
    let window_manager = WindowManager::new(&event_loop, "Procedural Texture Demo", 1280, 720)?;
    let window = Arc::new(window_manager.window);

    let mut render_context = RenderContext::new(window.clone()).await?;

    let mut procedural_manager = ProceduralTextureManager::new(
        render_context.device.clone(),
        render_context.graphics_queue.clone(),
        render_context.memory_allocator().clone(),
        render_context.command_buffer_allocator().clone(),
        Arc::new(vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator::new(
            render_context.device.clone(),
            Default::default(),
        )),
    );

    render_context
        .mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())?;

    let params = TextureGenerationParams {
        width: 512,
        height: 512,
        seed: 42,
    };

    println!("Generating procedural textures...");

    let perlin_graph = create_perlin_graph();
    let perlin_texture = procedural_manager.generate_texture(&perlin_graph, params)?;
    render_context
        .texture_manager_mut()
        .add_texture("perlin", perlin_texture);

    let worley_graph = create_worley_graph();
    let worley_texture = procedural_manager.generate_texture(&worley_graph, params)?;
    render_context
        .texture_manager_mut()
        .add_texture("worley", worley_texture);

    let marble_graph = create_marble_graph();
    let marble_texture = procedural_manager.generate_texture(&marble_graph, params)?;
    render_context
        .texture_manager_mut()
        .add_texture("marble", marble_texture);

    let wood_graph = create_wood_graph();
    let wood_texture = procedural_manager.generate_texture(&wood_graph, params)?;
    render_context
        .texture_manager_mut()
        .add_texture("wood", wood_texture);

    let cloud_graph = create_cloud_graph();
    let cloud_texture = procedural_manager.generate_texture(&cloud_graph, params)?;
    render_context
        .texture_manager_mut()
        .add_texture("cloud", cloud_texture);

    println!("Generated 5 procedural textures");
    println!(
        "Cache statistics: {} textures cached, using {} KB",
        procedural_manager.cached_texture_count(),
        procedural_manager.cache_memory_usage() / 1024
    );

    let camera_distance = 8.0;
    let mut angle = 0.0f32;

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render_context.configure_surface(size.width, size.height);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                angle += 0.01;

                let eye = Vec3::new(
                    angle.cos() * camera_distance,
                    4.0,
                    angle.sin() * camera_distance,
                );
                let target = Vec3::ZERO;
                let up = Vec3::Y;

                let view = Mat4::look_at_rh(eye, target, up);
                let proj = Mat4::perspective_rh(
                    45.0_f32.to_radians(),
                    1280.0 / 720.0,
                    0.1,
                    100.0,
                );

                let textures = ["perlin", "worley", "marble", "wood", "cloud"];
                let mut draw_commands = Vec::new();

                for (i, texture_name) in textures.iter().enumerate() {
                    let x = (i as f32 - 2.0) * 2.5;
                    let model = Mat4::from_rotation_translation(
                        Quat::from_rotation_y(angle),
                        Vec3::new(x, 0.0, 0.0),
                    );

                    draw_commands.push(DrawCommand {
                        mesh_id: "cube".to_string(),
                        model,
                        texture_name: Some(texture_name.to_string()),
                        material_properties: None,
                    });
                }

                let cmds = RenderCommands {
                    view,
                    proj,
                    draw_commands: &draw_commands,
                    lighting: None,
                };

                if let Err(e) = render_context.render(&cmds) {
                    eprintln!("Render error: {}", e);
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
