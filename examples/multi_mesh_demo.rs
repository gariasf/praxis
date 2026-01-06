//! Multi-mesh rendering demonstration.
//!
//! This example shows how to render multiple different mesh types in a single scene.
//! It demonstrates:
//! - Loading multiple mesh types into the mesh asset manager
//! - Using DrawCommands to render different meshes
//! - Positioning meshes with transform matrices

#![allow(clippy::vec_init_then_push)]

use praxis_graphics::{
    colored_cube_mesh, pyramid_mesh, quad_mesh, solid_cube_mesh, DrawCommand, RenderCommands,
    RenderContext,
};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, error, info, timing::FrameTimer, trace, warn, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

/// Application state with render context and mesh management.
struct State {
    size: winit::dpi::PhysicalSize<u32>,
    render_context: RenderContext,
    window: Arc<Window>,
    pending_resize: Option<(winit::dpi::PhysicalSize<u32>, Instant)>,
    frame_timer: FrameTimer,
    rotation_angle: f32,
}

/// The main application structure.
#[derive(Default)]
struct App {
    state: Option<State>,
    initialization_complete: bool,
}

impl State {
    /// Creates a new `State` instance and loads meshes.
    async fn new(window: Arc<Window>) -> Result<Self> {
        debug!("Creating application state");
        let state_start = std::time::Instant::now();

        let mut render_context = RenderContext::new(window.clone()).await?;

        // Load various mesh types into the mesh manager
        info!("Loading meshes into asset manager");

        render_context
            .mesh_manager_mut()
            .load_mesh("colored_cube", colored_cube_mesh())?;

        render_context
            .mesh_manager_mut()
            .load_mesh("red_cube", solid_cube_mesh([1.0, 0.0, 0.0]))?;

        render_context
            .mesh_manager_mut()
            .load_mesh("green_cube", solid_cube_mesh([0.0, 1.0, 0.0]))?;

        render_context
            .mesh_manager_mut()
            .load_mesh("blue_cube", solid_cube_mesh([0.0, 0.0, 1.0]))?;

        render_context
            .mesh_manager_mut()
            .load_mesh("pyramid", pyramid_mesh([0.8, 0.6, 0.2], [1.0, 0.0, 0.0]))?;

        render_context
            .mesh_manager_mut()
            .load_mesh("ground", quad_mesh(10.0, [0.3, 0.3, 0.3]))?;

        info!(
            "Loaded {} meshes",
            render_context.mesh_manager().mesh_count()
        );

        let size = window.inner_size();
        trace!("Window inner size: {}x{}", size.width, size.height);

        let state = State {
            size,
            render_context,
            window,
            pending_resize: None,
            frame_timer: FrameTimer::new_with_global(),
            rotation_angle: 0.0,
        };

        debug!("Application state created in {:?}", state_start.elapsed());
        Ok(state)
    }

    /// Handles window resize events.
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        debug!(
            "Reconfiguring surface due to resize: {}x{}",
            new_size.width, new_size.height
        );
        self.size = new_size;
        self.render_context
            .configure_surface(new_size.width, new_size.height);
    }

    /// Checks if a size has valid (non-zero) dimensions.
    fn has_valid_size(size: &winit::dpi::PhysicalSize<u32>) -> bool {
        size.width > 0 && size.height > 0
    }

    /// Determines if a resize operation should actually occur.
    fn should_resize(&self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        Self::has_valid_size(&new_size)
            && (new_size.width != self.size.width || new_size.height != self.size.height)
    }

    /// Determines if rendering should occur.
    fn should_render(&self) -> bool {
        Self::has_valid_size(&self.size)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("Application resumed");
        if self.state.is_some() {
            trace!("State already initialized, skipping");
            return;
        }

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(1920, 1080))
                .with_title("Multi-Mesh Demo - Praxis Engine")
                .with_resizable(true),
        ) {
            Ok(window) => {
                info!("Created window: 1920x1080");
                Arc::new(window)
            }
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let state = match pollster::block_on(State::new(window.clone())) {
            Ok(state) => {
                trace!("Requesting initial redraw");
                state.window.request_redraw();
                state
            }
            Err(e) => {
                error!("Failed to initialize state: {}", e);
                event_loop.exit();
                return;
            }
        };

        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() {
            Some(state) => state,
            None => {
                warn!(
                    "Window event {:?} received before state initialization",
                    event
                );
                return;
            }
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting event loop...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let delta = state.frame_timer.tick();

                if let Some((pending_size, resize_time)) = state.pending_resize {
                    const DEBOUNCE_DURATION: Duration = Duration::from_millis(16);

                    if resize_time.elapsed() >= DEBOUNCE_DURATION {
                        if state.should_resize(pending_size) {
                            debug!(
                                "Processing debounced resize to: {}x{}",
                                pending_size.width, pending_size.height
                            );
                            state.resize(pending_size);
                        } else {
                            trace!(
                                "Ignoring resize to zero dimensions or same size: {}x{}",
                                pending_size.width,
                                pending_size.height
                            );
                        }
                        state.pending_resize = None;
                    } else {
                        trace!("Still debouncing resize, requesting another redraw");
                        state.window.request_redraw();
                        return;
                    }
                }

                if state.should_render() {
                    trace!(
                        "Starting frame render (delta: {:.2}ms)",
                        delta.as_secs_f64() * 1000.0
                    );

                    // Update rotation
                    state.rotation_angle += delta.as_secs_f32() * 0.5;

                    // Set up camera
                    let aspect = state.size.width as f32 / state.size.height as f32;
                    let proj = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
                    let view = Mat4::look_at_rh(
                        Vec3::new(5.0, 4.0, 8.0),
                        Vec3::new(0.0, 0.5, 0.0),
                        Vec3::Y,
                    );

                    // Create draw commands for multiple meshes
                    let mut draw_commands = Vec::new();

                    // Ground plane
                    draw_commands.push(DrawCommand {
                        mesh_id: "ground".to_string(),
                        model: Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                        texture_name: None,
                        material_properties: None,
                    });

                    // Rotating colored cube at center
                    draw_commands.push(DrawCommand {
                        mesh_id: "colored_cube".to_string(),
                        model: Mat4::from_rotation_y(state.rotation_angle)
                            * Mat4::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                        texture_name: None,
                        material_properties: None,
                    });

                    // Red cube to the left
                    draw_commands.push(DrawCommand {
                        mesh_id: "red_cube".to_string(),
                        model: Mat4::from_translation(Vec3::new(-2.5, 0.0, 0.0))
                            * Mat4::from_rotation_y(state.rotation_angle * 0.7)
                            * Mat4::from_scale(Vec3::splat(0.8)),
                        texture_name: None,
                        material_properties: None,
                    });

                    // Green cube to the right
                    draw_commands.push(DrawCommand {
                        mesh_id: "green_cube".to_string(),
                        model: Mat4::from_translation(Vec3::new(2.5, 0.0, 0.0))
                            * Mat4::from_rotation_y(-state.rotation_angle * 0.7)
                            * Mat4::from_scale(Vec3::splat(0.8)),
                        texture_name: None,
                        material_properties: None,
                    });

                    // Blue cube in front
                    draw_commands.push(DrawCommand {
                        mesh_id: "blue_cube".to_string(),
                        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 2.5))
                            * Mat4::from_rotation_x(state.rotation_angle * 0.5)
                            * Mat4::from_scale(Vec3::splat(0.6)),
                        texture_name: None,
                        material_properties: None,
                    });

                    // Pyramid behind
                    draw_commands.push(DrawCommand {
                        mesh_id: "pyramid".to_string(),
                        model: Mat4::from_translation(Vec3::new(0.0, 0.0, -2.5))
                            * Mat4::from_rotation_y(state.rotation_angle * 1.2),
                        texture_name: None,
                        material_properties: None,
                    });

                    let cmds = RenderCommands {
                        view,
                        proj,
                        draw_commands: &draw_commands,
                        lighting: None,
                    };

                    match state.render_context.render(&cmds) {
                        Ok(()) => {
                            trace!(
                                "Frame rendered (current FPS: {:.1})",
                                state.frame_timer.fps()
                            );
                        }
                        Err(e) => {
                            error!("Render failed: {}", e);
                        }
                    }

                    state.window.request_redraw();
                } else {
                    trace!("Skipping render - window minimized or zero size");
                }

                if !self.initialization_complete {
                    self.initialization_complete = true;
                    info!("Window initialization complete, rendering started");
                }
            }
            WindowEvent::Resized(size) => {
                state.size = size;

                if state.should_resize(size) {
                    debug!("Received resize event: {}x{}", size.width, size.height);
                    state.pending_resize = Some((size, Instant::now()));
                }

                state.window.request_redraw();
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
                info!("Escape key pressed, exiting application");
                event_loop.exit();
            }
            _ => (),
        }
    }
}

/// Runs the multi-mesh demo application.
pub fn run() -> Result<()> {
    praxis_utils::init()?;

    info!("Starting Multi-Mesh Demo");
    let app_start = std::time::Instant::now();

    let mut app = App::default();

    debug!("Creating event loop...");
    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);
    trace!("Event loop control flow set to Poll mode");

    info!(
        "Starting event loop (initialized in {:?})",
        app_start.elapsed()
    );
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    info!("Application shutdown complete");
    Ok(())
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    run()
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("multi_mesh_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}
