//! ECS integration example for the Praxis engine.
//!
//! This example demonstrates how to integrate the ECS with the window and
//! graphics systems to create a complete game loop with entity management.

use std::sync::Arc;
use std::time::{Duration, Instant};

use praxis_ecs::{
    Active, Entity, GlobalTransform, Name, Transform, TransformBundle, Visibility, World,
};
use praxis_graphics::{RenderCommands, RenderContext};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_utils::timing::FrameTimer;
use praxis_utils::{debug, error, info, trace, warn, Result};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

/// Application state including ECS world and graphics context
struct AppState {
    /// The ECS world containing all entities
    world: World,
    /// Graphics rendering context
    render_context: RenderContext,
    /// The main window
    window: Arc<Window>,
    /// Current window size
    size: winit::dpi::PhysicalSize<u32>,
    /// Whether we need to resize the swapchain
    pending_resize: Option<(winit::dpi::PhysicalSize<u32>, Instant)>,
    /// Frame timing
    frame_timer: FrameTimer,
    /// Simple tracking for rotating entities (entity, rotation speed)
    rotating_entities: Vec<(Entity, f32)>,
}

impl AppState {
    /// Creates new application state with ECS world and entities
    async fn new(window: Arc<Window>) -> Result<Self> {
        debug!("Creating application state with ECS integration");

        // Create graphics context
        let render_context = RenderContext::new(window.clone()).await?;
        let size = window.inner_size();

        // Create ECS world
        let mut world = World::new();
        info!("Created ECS world");

        let mut rotating_entities = Vec::new();

        // Spawn a central rotating cube
        let center_cube = world.spawn((
            Name::from("Center Cube"),
            TransformBundle::from_xyz(0.0, 0.0, 0.0),
            Active,
            Visibility::Visible,
        ));
        rotating_entities.push((center_cube, 1.0));
        info!("Spawned center cube");

        // Spawn orbiting cubes in a circle
        let orbit_radius = 5.0;
        let num_orbiters = 8;
        for i in 0..num_orbiters {
            let angle = (i as f32) * std::f32::consts::TAU / num_orbiters as f32;
            let x = angle.cos() * orbit_radius;
            let z = angle.sin() * orbit_radius;

            let orbiter = world.spawn((
                Name::from(format!("Orbiter {}", i + 1)),
                TransformBundle::from_xyz(x, 0.5, z),
                Active,
                Visibility::Visible,
            ));

            // These orbiters rotate faster
            rotating_entities.push((orbiter, 2.0));
        }
        info!("Spawned {} orbiting cubes", num_orbiters);

        // Spawn a grid of cubes at different heights
        let grid_size: i32 = 3;
        let spacing = 2.5;
        for x in -grid_size..=grid_size {
            for z in -grid_size..=grid_size {
                // Skip the center area
                if x.abs() <= 1 && z.abs() <= 1 {
                    continue;
                }

                let pos_x = x as f32 * spacing;
                let pos_z = z as f32 * spacing;
                let height = ((x * x + z * z) as f32).sqrt() * 0.3 - 2.0;

                world.spawn((
                    Name::from(format!("Grid Cube ({}, {})", x, z)),
                    TransformBundle::from_transform(Transform {
                        translation: Vec3::new(pos_x, height, pos_z),
                        rotation: Quat::IDENTITY,
                        scale: Vec3::splat(0.8),
                    }),
                    Active,
                    Visibility::Visible,
                ));
            }
        }
        info!("Spawned grid of static cubes");

        info!("Total entities in world: {}", world.entity_count());

        Ok(Self {
            world,
            render_context,
            window,
            size,
            pending_resize: None,
            frame_timer: FrameTimer::new_with_global(),
            rotating_entities,
        })
    }

    /// Updates all entities based on elapsed time
    fn update_entities(&mut self, delta_time: f32) {
        // Update rotating entities
        for (entity, speed) in &self.rotating_entities {
            let inner_world = self.world.inner_mut();
            if let Some(mut transform) = inner_world.get_mut::<Transform>(*entity) {
                transform.rotation *= Quat::from_rotation_y(*speed * delta_time);
            }
        }

        // Make the orbiters orbit around the center
        // We'll do this by rotating their positions around Y axis
        let orbit_speed = 0.3;
        let rotation = Quat::from_rotation_y(orbit_speed * delta_time);

        let inner_world = self.world.inner_mut();
        let mut orbiter_query = inner_world.query::<(&Name, &mut Transform)>();

        for (name, mut transform) in orbiter_query.iter_mut(inner_world) {
            if name.as_str().starts_with("Orbiter") {
                // Rotate position around origin
                let pos = transform.translation;
                transform.translation = rotation * pos;
            }
        }

        // Update global transforms from local transforms
        let inner_world = self.world.inner_mut();
        let mut transform_query = inner_world.query::<(&Transform, &mut GlobalTransform)>();

        for (transform, mut global_transform) in transform_query.iter_mut(inner_world) {
            *global_transform = GlobalTransform::from(*transform);
        }
    }

    /// Collects visible entities and their transforms for rendering
    fn collect_render_data(&mut self) -> Vec<Mat4> {
        let inner_world = self.world.inner_mut();
        let mut visible_query = inner_world.query::<(&GlobalTransform, &Visibility, &Active)>();

        visible_query
            .iter(inner_world)
            .filter(|(_, visibility, _)| visibility.is_visible())
            .map(|(global_transform, _, _)| global_transform.matrix)
            .collect()
    }

    /// Handles window resize
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        debug!(
            "Reconfiguring surface due to resize: {}x{}",
            new_size.width, new_size.height
        );
        self.size = new_size;
        self.render_context
            .configure_surface(new_size.width, new_size.height);
    }

    fn should_resize(&self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        new_size.width > 0
            && new_size.height > 0
            && (new_size.width != self.size.width || new_size.height != self.size.height)
    }

    fn should_render(&self) -> bool {
        self.size.width > 0 && self.size.height > 0
    }
}

/// Main application handler
#[derive(Default)]
struct App {
    state: Option<AppState>,
    initialization_complete: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("Application resumed");
        if self.state.is_some() {
            return;
        }

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(1280, 720))
                .with_title("Praxis ECS Integration Demo")
                .with_resizable(true),
        ) {
            Ok(window) => {
                info!("Created window: 1280x720");
                Arc::new(window)
            }
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let state = match pollster::block_on(AppState::new(window.clone())) {
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
                warn!("Window event received before state initialization");
                return;
            }
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let delta = state.frame_timer.tick();
                let delta_secs = delta.as_secs_f32();

                // Handle pending resize
                if let Some((pending_size, resize_time)) = state.pending_resize {
                    const DEBOUNCE_DURATION: Duration = Duration::from_millis(16);

                    if resize_time.elapsed() >= DEBOUNCE_DURATION {
                        if state.should_resize(pending_size) {
                            state.resize(pending_size);
                        }
                        state.pending_resize = None;
                    } else {
                        state.window.request_redraw();
                        return;
                    }
                }

                if state.should_render() {
                    // Update all entities
                    state.update_entities(delta_secs);

                    // Collect render data from visible entities
                    let model_matrices = state.collect_render_data();

                    // Set up camera
                    let aspect = state.size.width as f32 / state.size.height as f32;
                    let time = praxis_utils::timing::total_time().as_secs_f32() * 0.2;

                    // Rotating camera that looks at the center
                    let camera_distance = 15.0;
                    let camera_height = 8.0;
                    let eye_x = time.cos() * camera_distance;
                    let eye_z = time.sin() * camera_distance;
                    let eye = Vec3::new(eye_x, camera_height, eye_z);
                    let target = Vec3::new(0.0, 0.0, 0.0);
                    let up = Vec3::Y;

                    let view = Mat4::look_at_rh(eye, target, up);
                    let proj = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);

                    // Render frame
                    let cmds = RenderCommands {
                        view,
                        proj,
                        models: &model_matrices,
                    };

                    match state.render_context.render(&cmds) {
                        Ok(()) => {
                            trace!(
                                "Frame rendered - {} entities, FPS: {:.1}",
                                model_matrices.len(),
                                state.frame_timer.fps()
                            );
                        }
                        Err(e) => {
                            error!("Render failed: {}", e);
                        }
                    }

                    state.window.request_redraw();
                }

                if !self.initialization_complete {
                    self.initialization_complete = true;
                    info!("Initialization complete, rendering started");
                    info!("Press ESC to exit");
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
                info!("Escape pressed, exiting...");
                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn main() -> Result<()> {
    // Initialize engine subsystems
    praxis_utils::init()?;
    praxis_ecs::init()?;

    info!("Starting Praxis ECS Integration Demo");

    let mut app = App::default();

    debug!("Creating event loop...");
    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    info!("Starting event loop...");
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    info!("Application shutdown complete");
    Ok(())
}
