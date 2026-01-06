//! Procedural texture generation demonstration using GPU compute shaders.
//!
//! This example demonstrates the GPU-based procedural texture generation system:
//! - **GPU Compute Shaders**: Real-time texture generation on the GPU
//! - **Noise Functions**: Perlin, Simplex, and Worley noise
//! - **Texture Graphs**: Node-based texture composition
//! - **Multiple Materials**: Various procedural texture types
//! - **Performance Monitoring**: Cache statistics and generation times
//!
//! # Texture Types Demonstrated
//!
//! 1. **Perlin Noise**: Classic smooth noise for clouds and terrain
//! 2. **Simplex Noise**: Improved Perlin with better isotropy
//! 3. **Worley Noise**: Cellular patterns for stone and organic textures
//! 4. **Marble**: Layered Perlin with power function and color ramp
//! 5. **Wood Grain**: Stretched noise with color gradient
//! 6. **Clouds**: Multi-octave noise with brightness adjustments
//!
//! # Controls
//!
//! - **WASD** - Move camera
//! - **Mouse** - Look around
//! - **Space/Ctrl** - Move up/down
//! - **Shift** - Sprint
//! - **R** - Regenerate all textures with new seed
//! - **C** - Clear texture cache and show statistics
//! - **ESC** - Exit
//!
//! # Usage
//!
//! ```bash
//! cargo run --example procedural_texture_demo
//! ```

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{DirectionalLight, PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    textured_cube_mesh, DirectionalLightData, DrawCommand, LightingUniforms, RenderCommands,
    RenderContext,
};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::{Mat4, Quat, Vec2, Vec3};
use praxis_procedural::{
    BlendMode, ColorRamp, ColorStop, NoiseType, TextureGenerationParams, TextureGraph,
    TextureNode, TransformParams,
};
use praxis_utils::{info, Result};
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

struct ProceduralTextureDemo {
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    world: World,
    camera_entity: praxis_ecs::Entity,
    camera_controller: CameraController,
    input_state: InputState,
    last_frame_time: Instant,
    cursor_grabbed: bool,
    current_seed: u32,
    texture_names: Vec<String>,
}

impl ProceduralTextureDemo {
    fn new() -> Self {
        let mut world = World::default();

        let camera_entity = world
            .spawn()
            .insert_bundle(PerspectiveCameraBundle {
                transform: Transform::from_xyz(0.0, 2.0, 8.0),
                ..Default::default()
            })
            .id();

        world.spawn().insert(DirectionalLight {
            direction: Vec3::new(-0.5, -1.0, -0.5).normalize(),
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        });

        let mut input_map = InputMap::new();
        input_map.bind_key(KeyCode::KeyW, Action::new("forward"));
        input_map.bind_key(KeyCode::KeyS, Action::new("backward"));
        input_map.bind_key(KeyCode::KeyA, Action::new("left"));
        input_map.bind_key(KeyCode::KeyD, Action::new("right"));
        input_map.bind_key(KeyCode::Space, Action::new("up"));
        input_map.bind_key(KeyCode::ControlLeft, Action::new("down"));
        input_map.bind_key(KeyCode::ShiftLeft, Action::new("sprint"));
        input_map.bind_key(KeyCode::Escape, Action::new("toggle_cursor"));
        input_map.bind_key(KeyCode::KeyR, Action::new("regenerate"));
        input_map.bind_key(KeyCode::KeyC, Action::new("clear_cache"));

        let input_state = InputState::new(input_map);

        Self {
            window: None,
            render_context: None,
            world,
            camera_entity,
            camera_controller: CameraController::new(5.0, 0.1),
            input_state,
            last_frame_time: Instant::now(),
            cursor_grabbed: false,
            current_seed: 42,
            texture_names: Vec::new(),
        }
    }

    fn initialize_graphics(&mut self) -> Result<()> {
        info!("Initializing procedural texture demo");

        let render_context = self.render_context.as_mut().unwrap();

        let cube_mesh = textured_cube_mesh([1.0, 1.0, 1.0]);
        render_context
            .mesh_manager_mut()
            .load_mesh("textured_cube", cube_mesh)?;

        self.generate_procedural_textures()?;

        Ok(())
    }

    fn generate_procedural_textures(&mut self) -> Result<()> {
        let render_context = self.render_context.as_mut().unwrap();
        let procedural_manager = render_context.procedural_texture_manager_mut();

        self.texture_names.clear();

        let params = TextureGenerationParams {
            width: 512,
            height: 512,
            seed: self.current_seed,
        };

        info!("Generating procedural textures with seed {}", self.current_seed);

        // 1. Simple Perlin Noise
        {
            let mut graph = TextureGraph::new();
            let noise_id = graph.add_node(TextureNode::Noise {
                noise_type: NoiseType::Perlin,
                scale: 8.0,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            });
            graph.set_output(noise_id);

            let texture = procedural_manager.generate_texture(&graph, params)?;
            render_context.texture_manager_mut().add_texture("perlin_noise", texture);
            self.texture_names.push("perlin_noise".to_string());
            info!("Generated Perlin noise texture");
        }

        // 2. Simplex Noise
        {
            let mut graph = TextureGraph::new();
            let noise_id = graph.add_node(TextureNode::Noise {
                noise_type: NoiseType::Simplex,
                scale: 8.0,
                octaves: 4,
                persistence: 0.5,
                lacunarity: 2.0,
            });
            graph.set_output(noise_id);

            let texture = procedural_manager.generate_texture(&graph, params)?;
            render_context.texture_manager_mut().add_texture("simplex_noise", texture);
            self.texture_names.push("simplex_noise".to_string());
            info!("Generated Simplex noise texture");
        }

        // 3. Worley Noise
        {
            let mut graph = TextureGraph::new();
            let noise_id = graph.add_node(TextureNode::Noise {
                noise_type: NoiseType::Worley,
                scale: 8.0,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            });
            graph.set_output(noise_id);

            let texture = procedural_manager.generate_texture(&graph, params)?;
            render_context.texture_manager_mut().add_texture("worley_noise", texture);
            self.texture_names.push("worley_noise".to_string());
            info!("Generated Worley noise texture");
        }

        // 4. Marble Texture
        {
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

            let texture = procedural_manager.generate_texture(&graph, params)?;
            render_context.texture_manager_mut().add_texture("marble", texture);
            self.texture_names.push("marble".to_string());
            info!("Generated marble texture");
        }

        // 5. Wood Grain
        {
            let mut graph = TextureGraph::new();

            let noise_id = graph.add_node(TextureNode::Noise {
                noise_type: NoiseType::Perlin,
                scale: 10.0,
                octaves: 4,
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
                    color: [0.6, 0.4, 0.2, 1.0],
                },
                ColorStop {
                    position: 1.0,
                    color: [0.4, 0.25, 0.1, 1.0],
                },
            ]);

            let ramp_id = graph.add_node(TextureNode::ColorRamp {
                input: transform_id,
                ramp,
            });

            graph.set_output(ramp_id);

            let texture = procedural_manager.generate_texture(&graph, params)?;
            render_context.texture_manager_mut().add_texture("wood_grain", texture);
            self.texture_names.push("wood_grain".to_string());
            info!("Generated wood grain texture");
        }

        // 6. Cloud Texture
        {
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
                scale: 16.0,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            });

            let blend_id = graph.add_node(TextureNode::Blend {
                input_a: noise1_id,
                input_b: noise2_id,
                mode: BlendMode::Multiply,
                factor: 0.5,
            });

            let brightness_id = graph.add_node(TextureNode::Brightness {
                input: blend_id,
                amount: 0.2,
            });

            graph.set_output(brightness_id);

            let texture = procedural_manager.generate_texture(&graph, params)?;
            render_context.texture_manager_mut().add_texture("clouds", texture);
            self.texture_names.push("clouds".to_string());
            info!("Generated cloud texture");
        }

        let stats = procedural_manager.cache_statistics();
        info!("Cache statistics: hits={}, misses={}, evictions={}", 
              stats.hits, stats.misses, stats.evictions);

        Ok(())
    }

    fn update(&mut self, delta_time: f32) {
        self.input_state.update();

        let mut transform = self.world.get_mut::<Transform>(self.camera_entity).unwrap();

        self.camera_controller.update(
            &mut transform,
            &self.input_state,
            delta_time,
            self.cursor_grabbed,
        );

        if self.input_state.action_just_pressed("toggle_cursor") {
            self.toggle_cursor();
        }

        if self.input_state.action_just_pressed("regenerate") {
            self.current_seed = self.current_seed.wrapping_add(1);
            info!("Regenerating textures with new seed: {}", self.current_seed);
            if let Err(e) = self.generate_procedural_textures() {
                praxis_utils::error!("Failed to regenerate textures: {}", e);
            }
        }

        if self.input_state.action_just_pressed("clear_cache") {
            if let Some(render_context) = &mut self.render_context {
                let manager = render_context.procedural_texture_manager_mut();
                let stats = manager.cache_statistics();
                info!("Cache statistics before clear:");
                info!("  Hits: {}", stats.hits);
                info!("  Misses: {}", stats.misses);
                info!("  Evictions: {}", stats.evictions);
                info!("  Cached textures: {}", manager.cached_texture_count());
                info!("  Memory usage: {} bytes", manager.cache_memory_usage());
                
                manager.clear_cache();
                manager.reset_cache_statistics();
                info!("Cache cleared!");
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        let render_context = self.render_context.as_mut().unwrap();

        let camera_transform = self.world.get::<Transform>(self.camera_entity).unwrap();
        let camera = self
            .world
            .get::<praxis_ecs::PerspectiveCamera>(self.camera_entity)
            .unwrap();

        let view = camera_transform.compute_matrix().inverse();
        let projection = camera.projection_matrix();

        let mut commands = RenderCommands::new();

        let spacing = 2.5;
        for (i, texture_name) in self.texture_names.iter().enumerate() {
            let x = (i as f32 - 2.5) * spacing;
            let y_offset = ((self.last_frame_time.elapsed().as_secs_f32() + i as f32 * 0.5).sin()
                * 0.3)
                .abs();

            let model = Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::from_rotation_y(self.last_frame_time.elapsed().as_secs_f32() * 0.5),
                Vec3::new(x, y_offset, 0.0),
            );

            commands.add_draw_command(DrawCommand {
                mesh_id: "textured_cube".to_string(),
                model,
                texture_name: Some(texture_name.clone()),
                material_properties: None,
            });
        }

        let directional_lights: Vec<DirectionalLightData> = self
            .world
            .query::<&DirectionalLight>()
            .iter(&self.world)
            .map(|(_, light)| DirectionalLightData {
                direction: [light.direction.x, light.direction.y, light.direction.z, 0.0],
                color: [light.color.x, light.color.y, light.color.z, light.intensity],
            })
            .collect();

        let lighting = LightingUniforms {
            view_pos: [
                camera_transform.translation.x,
                camera_transform.translation.y,
                camera_transform.translation.z,
                1.0,
            ],
            num_point_lights: 0,
            num_directional_lights: directional_lights.len() as u32,
            point_lights: Vec::new(),
            directional_lights,
        };

        render_context.render(&commands, &view, &projection, &lighting)?;

        Ok(())
    }

    fn toggle_cursor(&mut self) {
        self.cursor_grabbed = !self.cursor_grabbed;
        if let Some(window) = &self.window {
            if self.cursor_grabbed {
                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                window.set_cursor_visible(false);
            } else {
                let _ = window.set_cursor_grab(CursorGrabMode::None);
                window.set_cursor_visible(true);
            }
        }
    }
}

impl ApplicationHandler for ProceduralTextureDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Praxis - Procedural Texture Demo")
                        .with_inner_size(PhysicalSize::new(1280, 720)),
                )
                .expect("Failed to create window"),
        );

        let render_context = RenderContext::new(window.clone()).expect("Failed to create render context");

        self.window = Some(window.clone());
        self.render_context = Some(render_context);

        self.initialize_graphics()
            .expect("Failed to initialize graphics");

        self.toggle_cursor();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context
                        .resize(new_size.width, new_size.height)
                        .expect("Failed to resize");
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;

                self.update(delta_time);

                if let Err(e) = self.render() {
                    praxis_utils::error!("Render error: {}", e);
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                if key_code == KeyCode::Escape && state == ElementState::Pressed {
                    info!("Escape pressed, exiting");
                    event_loop.exit();
                } else {
                    self.input_state.handle_key(key_code, state);
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_grabbed {
                self.camera_controller.process_mouse(delta.0, delta.1);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init_logging();

    info!("Starting Procedural Texture Demo");

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ProceduralTextureDemo::new();

    event_loop.run_app(&mut app).expect("Failed to run event loop");

    Ok(())
}
