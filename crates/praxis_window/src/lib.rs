//! Window management system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing windows.

use std::sync::Arc;

use praxis_graphics::RenderContext;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Fullscreen, Window, WindowId},
};

use praxis_utils::{Result, debug, eyre, info};

/// Represents the application's state, including graphics context and window size.
struct State {
    size: winit::dpi::PhysicalSize<u32>,
    render_context: RenderContext,
    window: Arc<Window>,
}

/// The main application structure that handles the event loop and owns the state.
#[derive(Default)]
struct App {
    state: Option<State>,
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
        info!("Initializing graphics render context...");
        let render_context = RenderContext::new(window.clone()).await?;

        info!("Getting initial window size...");
        let size = window.inner_size();

        let state = State {
            size,
            render_context,
            window,
        };

        info!("Configuring surface for the first time...");
        state
            .render_context
            .configure_surface(size.width, size.height);

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
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .with_title("In Praxis")
                .with_resizable(false),
        ) {
            Ok(window) => {
                info!("Window created successfully.");
                Arc::new(window)
            }
            Err(e) => {
                info!("Failed to create window: {}", e);
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
                let _ = state.render_context.render();
            }
            WindowEvent::Resized(size) => {
                if state.should_resize(size) {
                    info!("Window resized to: {:?}", size);
                    state.resize(size);
                } else {
                    debug!(
                        "Ignoring resize to zero dimensions or same size: {:?}",
                        size
                    );
                }
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
                info!("Escape key pressed.");
                event_loop.exit();
            }
            _ => (),
        }
    }

    // Add other ApplicationHandler methods if needed, default is fine for now
    // fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {}
    // fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {}
    // fn user_event(&mut self, event_loop: &ActiveEventLoop, event: T) {}
    // fn suspended(&mut self, event_loop: &ActiveEventLoop) {}
    // fn exiting(&mut self, event_loop: &ActiveEventLoop) {}
    // fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {}
}

/// Runs the main application event loop.
///
/// Initializes the winit event loop and runs the `App` state machine.
///
/// # Returns
/// Returns `Ok(())` if the application exits cleanly, or an error if loop creation fails.
pub fn run() -> Result<()> {
    info!("Creating event loop...");
    let event_loop =
        EventLoop::new().map_err(|e| eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    info!("Running application...");
    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}
