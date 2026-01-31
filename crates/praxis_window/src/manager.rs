//! Window manager implementation.
//!
//! This module provides the `WindowManager`, which is the main entry point for
//! creating and managing windows with the event loop.

use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use praxis_utils::{debug, error, info, trace, warn, Result};

use crate::{
    config::WindowConfig,
    event_handler::WindowEventHandler,
    window_events::{PendingResize, WindowResizeStrategy},
};

/// Main window manager that owns the window and event loop.
///
/// This is the primary interface for creating and running windowed applications.
/// It integrates window creation, event handling, and frame timing into a single
/// cohesive API.
///
/// # Architecture
///
/// The manager uses the ApplicationHandler pattern from winit 0.30+:
/// - The window is created during the `resumed()` callback (when event loop is active)
/// - Events are dispatched to the user's `WindowEventHandler` implementation
/// - The manager handles low-level details like resize debouncing and frame timing
///
/// # Examples
///
/// ## Minimal Window
///
/// ```rust,ignore
/// use praxis_window::{WindowConfig, WindowManager};
///
/// fn main() -> Result<()> {
///     praxis_utils::init()?;
///     
///     let config = WindowConfig::default()
///         .with_title("My Window")
///         .with_size(1280, 720);
///     
///     let manager = WindowManager::new(config)?;
///     manager.run()?;
///     
///     Ok(())
/// }
/// ```
///
/// ## With Custom Handler
///
/// ```rust,ignore
/// use praxis_window::{WindowConfig, WindowManager, WindowEventHandler, Window};
///
/// struct MyApp {
///     frame_count: u32,
/// }
///
/// impl WindowEventHandler for MyApp {
///     fn on_render(&mut self, _window: &Window) {
///         self.frame_count += 1;
///     }
/// }
///
/// fn main() -> Result<()> {
///     let app = MyApp { frame_count: 0 };
///     let manager = WindowManager::with_handler(WindowConfig::default(), app)?;
///     manager.run()?;
///     Ok(())
/// }
/// ```
pub struct WindowManager<H: WindowEventHandler> {
    /// Configuration for window creation
    config: WindowConfig,
    /// User event handler
    handler: H,
    /// Resize debouncing strategy
    resize_strategy: WindowResizeStrategy,
    /// Event loop (owned until run() is called)
    event_loop: Option<EventLoop<()>>,
}

impl<H: WindowEventHandler> WindowManager<H> {
    /// Creates a new window manager with a custom event handler.
    ///
    /// The window is not created immediately - it's created when `run()` is called
    /// and the event loop activates.
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration
    /// * `handler` - User event handler implementation
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop cannot be created (rare - typically only
    /// happens if the platform doesn't support windowing).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let app = MyApp::new();
    /// let manager = WindowManager::with_handler(WindowConfig::default(), app)?;
    /// ```
    pub fn with_handler(config: WindowConfig, handler: H) -> Result<Self> {
        debug!("Creating window manager");

        let event_loop = EventLoop::new()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

        Ok(Self {
            config,
            handler,
            resize_strategy: WindowResizeStrategy::default(),
            event_loop: Some(event_loop),
        })
    }

    /// Sets the resize debouncing strategy.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use praxis_window::WindowResizeStrategy;
    ///
    /// let manager = WindowManager::new(config)?
    ///     .with_resize_strategy(WindowResizeStrategy::Immediate);
    /// ```
    #[must_use]
    pub fn with_resize_strategy(mut self, strategy: WindowResizeStrategy) -> Self {
        self.resize_strategy = strategy;
        self
    }

    /// Runs the event loop, blocking until the window is closed.
    ///
    /// This consumes the manager because the event loop takes ownership and
    /// runs indefinitely until exit is requested.
    ///
    /// # Lifecycle
    ///
    /// 1. Event loop starts
    /// 2. `resumed()` callback creates window and calls handler's `on_init()`
    /// 3. Main loop repeatedly calls handler's `on_update()` and `on_render()`
    /// 4. When close is requested, calls handler's `on_close()`
    /// 5. Event loop exits and this function returns
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop encounters a fatal error.
    /// Window creation errors are logged but don't prevent the loop from running.
    pub fn run(mut self) -> Result<()> {
        info!("Starting window manager");

        let event_loop = self.event_loop.take().expect("Event loop already consumed");

        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App {
            config: self.config,
            handler: self.handler,
            resize_strategy: self.resize_strategy,
            state: None,
        };

        event_loop
            .run_app(&mut app)
            .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

        info!("Window manager shut down");
        Ok(())
    }
}

impl WindowManager<()> {
    /// Creates a new window manager without a custom event handler.
    ///
    /// This is useful for simple cases where you just want a window without
    /// custom event handling logic. The window will open and run but do nothing.
    ///
    /// # Arguments
    ///
    /// * `config` - Window configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let manager = WindowManager::new(WindowConfig::default())?;
    /// manager.run()?;
    /// ```
    pub fn new(config: WindowConfig) -> Result<Self> {
        Self::with_handler(config, ())
    }
}

/// Internal application state for the ApplicationHandler.
struct App<H: WindowEventHandler> {
    config: WindowConfig,
    handler: H,
    resize_strategy: WindowResizeStrategy,
    state: Option<WindowState>,
}

/// Window state created during resumed callback.
struct WindowState {
    window: Arc<Window>,
    current_size: winit::dpi::PhysicalSize<u32>,
    pending_resize: Option<PendingResize>,
    last_frame_time: Instant,
    initialization_complete: bool,
}

impl WindowState {
    fn new(window: Arc<Window>) -> Self {
        let current_size = window.inner_size();
        Self {
            window,
            current_size,
            pending_resize: None,
            last_frame_time: Instant::now(),
            initialization_complete: false,
        }
    }

    fn has_valid_size(size: &winit::dpi::PhysicalSize<u32>) -> bool {
        size.width > 0 && size.height > 0
    }

    fn should_render(&self) -> bool {
        Self::has_valid_size(&self.current_size)
    }

    fn should_process_resize(&self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        Self::has_valid_size(&new_size)
            && (new_size.width != self.current_size.width
                || new_size.height != self.current_size.height)
    }
}

impl<H: WindowEventHandler> ApplicationHandler for App<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("Application resumed");

        if self.state.is_some() {
            trace!("State already initialized, skipping");
            return;
        }

        let window = match event_loop.create_window(self.config.to_window_attributes()) {
            Ok(window) => {
                info!(
                    "Created window: {}x{}",
                    self.config.width, self.config.height
                );
                Arc::new(window)
            }
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let state = WindowState::new(window.clone());

        self.handler.on_init(&window);

        trace!("Requesting initial redraw");
        state.window.request_redraw();

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
                info!("Close requested");
                if self.handler.on_close() {
                    info!("Exiting event loop");
                    event_loop.exit();
                } else {
                    debug!("Close prevented by handler");
                }
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta_time = now.duration_since(state.last_frame_time).as_secs_f32();
                state.last_frame_time = now;

                if let Some(pending) = state.pending_resize {
                    if pending.is_ready(self.resize_strategy) {
                        if state.should_process_resize(pending.size) {
                            debug!(
                                "Processing debounced resize to {}x{}",
                                pending.size.width, pending.size.height
                            );

                            state.current_size = pending.size;
                            self.handler
                                .on_resize(pending.size.width, pending.size.height);
                        }
                        state.pending_resize = None;
                    } else {
                        trace!("Still debouncing resize");
                        state.window.request_redraw();
                        return;
                    }
                }

                if state.should_render() {
                    trace!("Frame update (delta: {:.2}ms)", delta_time * 1000.0);

                    self.handler.on_update(delta_time);
                    self.handler.on_render(&state.window);

                    state.window.request_redraw();
                } else {
                    trace!("Skipping frame - window minimized or zero size");
                }

                if !state.initialization_complete {
                    state.initialization_complete = true;
                    info!("Window initialization complete");
                }
            }

            WindowEvent::Resized(size) => {
                state.current_size = size;

                if state.should_process_resize(size) {
                    debug!("Received resize event: {}x{}", size.width, size.height);
                    state.pending_resize = Some(PendingResize::new(size));
                }

                state.window.request_redraw();
            }

            WindowEvent::Focused(focused) => {
                if focused {
                    debug!("Window gained focus");
                    self.handler.on_focused();
                } else {
                    debug!("Window lost focus");
                    self.handler.on_unfocused();
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: key_state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if logical_key == Key::Named(NamedKey::Escape) && key_state == ElementState::Pressed
                {
                    info!("Escape key pressed, requesting exit");
                    if self.handler.on_close() {
                        event_loop.exit();
                    }
                } else {
                    match key_state {
                        ElementState::Pressed => {
                            self.handler.on_key_pressed(logical_key, repeat);
                        }
                        ElementState::Released => {
                            self.handler.on_key_released(logical_key);
                        }
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.handler.on_mouse_moved(position.x, position.y);
            }

            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.handler.on_mouse_button_pressed(button);
                }
                ElementState::Released => {
                    self.handler.on_mouse_button_released(button);
                }
            },

            WindowEvent::MouseWheel { delta, .. } => {
                let (delta_x, delta_y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
                };
                self.handler.on_mouse_wheel(delta_x, delta_y);
            }

            _ => {}
        }
    }
}
