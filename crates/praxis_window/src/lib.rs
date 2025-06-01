//! Window management system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing windows.

use std::sync::Arc;
use std::time::{Duration, Instant};

use praxis_graphics::RenderContext;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use praxis_utils::{Result, debug, error, eyre, info};

/// Represents the application's state, including graphics context and window size.
struct State {
    size: winit::dpi::PhysicalSize<u32>,
    render_context: RenderContext,
    window: Arc<Window>,
    pending_resize: Option<(winit::dpi::PhysicalSize<u32>, Instant)>,
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
        let render_context = RenderContext::new(window.clone()).await?;

        let size = window.inner_size();

        let state = State {
            size,
            render_context,
            window,
            pending_resize: None,
        };

        // Don't configure surface immediately - let the debouncing handle it
        // The first RedrawRequested will trigger the debounced resize processing

        Ok(state)
    }

    /// Handles window resize events.
    ///
    /// Updates the stored window size and reconfigures the rendering surface.
    ///
    /// # Arguments
    /// * `new_size` - The new physical size of the window.
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        info!("Reconfiguring surface due to resize: {:?}", new_size);
        self.size = new_size;
        self.render_context
            .configure_surface(new_size.width, new_size.height);
    }

    /// Determines if a resize operation should actually occur.
    ///
    /// Checks if the new size has non-zero dimensions and is different from the current size.
    ///
    /// # Arguments
    /// * `new_size` - The potential new physical size.
    fn should_resize(&self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        new_size.width > 0
            && new_size.height > 0
            && (new_size.width != self.size.width || new_size.height != self.size.height)
    }
}

/// Implementation of the `winit` Application Handler trait for the main application loop.
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(1920, 1080))
                .with_title("In Praxis")
                .with_resizable(true),
        ) {
            Ok(window) => {
                info!("Window created successfully.");
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
                debug!("Window event received before state initialization");
                return;
            }
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting event loop...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some((pending_size, resize_time)) = state.pending_resize {
                    const DEBOUNCE_DURATION: Duration = Duration::from_millis(16); // ~1 frame at 60fps

                    if resize_time.elapsed() >= DEBOUNCE_DURATION {
                        if state.should_resize(pending_size) {
                            info!("Processing debounced resize to: {:?}", pending_size);
                            state.resize(pending_size);
                        } else {
                            debug!(
                                "Ignoring resize to zero dimensions or same size: {:?}",
                                pending_size
                            );
                        }
                        state.pending_resize = None;
                    } else {
                        // If we're still debouncing, request another redraw and skip rendering
                        state.window.request_redraw();
                        return;
                    }
                }

                // Only render if we're not in the middle of processing a resize
                match state.render_context.render() {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Render failed: {}", e);
                    }
                }

                if !self.initialization_complete {
                    self.initialization_complete = true;
                }
            }
            WindowEvent::Resized(size) => {
                info!("Received resize event: {:?}", size);
                state.pending_resize = Some((size, Instant::now()));
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
                debug!("Escape key pressed.");
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
    let mut app = App::default();

    info!("Creating event loop...");
    let event_loop =
        EventLoop::new().map_err(|e| eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    info!("Running application...");
    event_loop
        .run_app(&mut app)
        .map_err(|e| eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}
