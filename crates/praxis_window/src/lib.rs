//! Window management system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing windows.

use std::sync::Arc;
use std::time::{Duration, Instant};

use praxis_graphics::RenderContext;
use praxis_utils::timing::FrameTimer;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use praxis_utils::{debug, error, eyre, info, trace, warn, Result};

/// Represents the application's state, including graphics context and window size.
struct State {
    size: winit::dpi::PhysicalSize<u32>,
    render_context: RenderContext,
    window: Arc<Window>,
    pending_resize: Option<(winit::dpi::PhysicalSize<u32>, Instant)>,
    frame_timer: FrameTimer,
}

/// The main application structure that handles the event loop and owns the state.
#[derive(Default)]
struct App {
    state: Option<State>,
    initialization_complete: bool,
}

impl State {
    /// Creates a new `State` instance.
    ///
    /// Initializes the `RenderContext` for the given window, stores the initial size,
    /// and configures the rendering surface.
    ///
    /// # Arguments
    /// * `window` - An `Arc<Window>` for which to create the state.
    async fn new(window: Arc<Window>) -> Result<Self> {
        debug!("Creating application state");
        let state_start = std::time::Instant::now();

        let render_context = RenderContext::new(window.clone()).await?;

        let size = window.inner_size();
        trace!("Window inner size: {}x{}", size.width, size.height);

        let state = State {
            size,
            render_context,
            window,
            pending_resize: None,
            frame_timer: FrameTimer::new_with_global(),
        };

        // Don't configure surface immediately - let the debouncing handle it
        // The first RedrawRequested will trigger the debounced resize processing

        debug!("Application state created in {:?}", state_start.elapsed());
        Ok(state)
    }

    /// Handles window resize events.
    ///
    /// Updates the stored window size and reconfigures the rendering surface.
    ///
    /// # Arguments
    /// * `new_size` - The new physical size of the window.
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
    ///
    /// Checks if the new size has non-zero dimensions and is different from the current size.
    ///
    /// # Arguments
    /// * `new_size` - The potential new physical size.
    fn should_resize(&self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        Self::has_valid_size(&new_size)
            && (new_size.width != self.size.width || new_size.height != self.size.height)
    }

    /// Determines if rendering should occur.
    ///
    /// Returns false when the window is minimized or has zero size.
    fn should_render(&self) -> bool {
        Self::has_valid_size(&self.size)
    }
}

/// Implementation of the `winit` Application Handler trait for the main application loop.
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
                .with_title("In Praxis")
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
                    const DEBOUNCE_DURATION: Duration = Duration::from_millis(16); // ~1 frame at 60fps

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
                        // If we're still debouncing, request another redraw and skip rendering
                        trace!("Still debouncing resize, requesting another redraw");
                        state.window.request_redraw();
                        return;
                    }
                }

                // Rendering is not performed here. The window system provides
                // the event loop infrastructure, but actual rendering should be
                // implemented in examples or user code by extending this module
                // or using the standalone example patterns.
                //
                // See examples/ directory for rendering implementations that use
                // the State trait or direct ApplicationHandler implementation.
                if state.should_render() {
                    trace!(
                        "Frame tick (delta: {:.2}ms, FPS: {:.1})",
                        delta.as_secs_f64() * 1000.0,
                        state.frame_timer.fps()
                    );
                    state.window.request_redraw();
                } else {
                    trace!("Skipping frame tick - window minimized or zero size");
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

/// Runs the main application event loop.
///
/// Initializes the winit event loop and runs the `App` state machine.
///
/// # Returns
/// Returns `Ok(())` if the application exits cleanly, or an error if loop creation fails.
pub fn run() -> Result<()> {
    info!("Starting Praxis application");
    let app_start = std::time::Instant::now();

    let mut app = App::default();

    debug!("Creating event loop...");
    let event_loop =
        EventLoop::new().map_err(|e| eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);
    trace!("Event loop control flow set to Poll mode");

    info!(
        "Starting event loop (initialized in {:?})",
        app_start.elapsed()
    );
    event_loop
        .run_app(&mut app)
        .map_err(|e| eyre::eyre!("Event loop error: {}", e))?;

    info!("Application shutdown complete");
    Ok(())
}
