//! Window management system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing windows using the `winit` library,
//! which is the de facto standard for cross-platform window management in Rust. It integrates
//! with Vulkan through surface creation and handles the complex event loop lifecycle required
//! for real-time graphics applications.
//!
//! # Winit Integration Overview
//!
//! Winit provides a platform-agnostic event loop and window creation API. The key concepts are:
//!
//! - **EventLoop**: The central dispatcher that receives OS events (keyboard, mouse, window changes)
//! - **ApplicationHandler**: A trait that defines callbacks for different lifecycle stages
//! - **Window**: The actual OS window that displays graphics content
//! - **WindowEvent**: Events specific to a window (resize, close, input, etc.)
//!
//! # Event Loop Patterns
//!
//! Modern winit (0.30+) uses the `ApplicationHandler` trait pattern instead of closures:
//!
//! 1. **resumed()**: Called when the app starts or resumes from suspension (important for mobile)
//! 2. **window_event()**: Called for each window event (input, resize, redraw, close, etc.)
//! 3. **about_to_wait()**: Called after all events are processed (not used here)
//!
//! The event loop runs continuously in one of two modes:
//! - **Poll**: Runs as fast as possible (used here for games/real-time graphics)
//! - **Wait**: Blocks until an event occurs (for non-interactive applications)
//!
//! # Window Resize Handling
//!
//! Window resizing is one of the most complex aspects of real-time graphics programming because:
//!
//! 1. **Rapid Fire Events**: OS sends many resize events during a drag operation
//! 2. **Swapchain Recreation**: Vulkan requires recreating the swapchain for each size change
//! 3. **Zero-Size Windows**: Minimized windows report 0x0, which is invalid for Vulkan
//! 4. **Race Conditions**: Resize events can arrive while rendering is in progress
//!
//! This implementation uses a **debouncing strategy**:
//! - Store the latest resize request with a timestamp
//! - Wait for a short delay (16ms, ~1 frame) before processing
//! - If another resize arrives during the delay, replace the pending one
//! - This reduces swapchain recreations from hundreds to just a few
//!
//! # Vulkan Surface Creation
//!
//! A Vulkan surface is the connection between Vulkan and the OS windowing system:
//! - Created from a raw window handle (via `raw-window-handle` trait)
//! - Must be created before the swapchain
//! - Platform-specific (VK_KHR_win32_surface, VK_KHR_xlib_surface, etc.)
//! - Owned by the VkInstance, not the device
//!
//! Surface creation happens in `RenderContext::new()` (see praxis_graphics crate).
//! The surface is then used to query capabilities and create a swapchain that matches
//! the window's size and the display's format.
//!
//! # ApplicationHandler Lifecycle
//!
//! The typical lifecycle for a window-based graphics application:
//!
//! 1. **EventLoop created**: Before any windows exist
//! 2. **resumed() called**: Create window, initialize graphics, request first redraw
//! 3. **RedrawRequested**: Render frame, request next redraw (for continuous rendering)
//! 4. **Resized**: Update window size, debounce, recreate swapchain when debounce expires
//! 5. **CloseRequested**: Clean up resources, exit event loop
//!
//! State management pattern:
//! - `App` is the ApplicationHandler that owns an `Option<State>`
//! - `State` is created in `resumed()` and contains window + graphics context
//! - `State` is accessed as mutable reference in `window_event()`
//! - Using `Option` allows proper initialization timing (window doesn't exist until resumed)

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
///
/// # Fields
///
/// - `size`: The current physical size of the window in pixels (not logical/scaled size)
/// - `render_context`: Vulkan rendering context that owns the surface, swapchain, and device
/// - `window`: Arc-wrapped window (Arc needed because Vulkan surface creation requires window reference)
/// - `pending_resize`: Debouncing mechanism - stores (size, timestamp) when resize event arrives
/// - `frame_timer`: Tracks frame delta time and FPS for performance monitoring
struct State {
    size: winit::dpi::PhysicalSize<u32>,
    render_context: RenderContext,
    window: Arc<Window>,
    pending_resize: Option<(winit::dpi::PhysicalSize<u32>, Instant)>,
    frame_timer: FrameTimer,
}

/// The main application structure that handles the event loop and owns the state.
///
/// # ApplicationHandler Pattern
///
/// This struct implements `ApplicationHandler`, which is winit's trait for managing application lifecycle.
/// The `Option<State>` pattern is necessary because:
/// - The struct is created before the event loop starts
/// - Windows can only be created inside the `resumed()` callback (when event loop is active)
/// - Some platforms (iOS, Android) can suspend and resume, requiring re-initialization
///
/// # Fields
///
/// - `state`: None until `resumed()` is called, then contains all window/graphics resources
/// - `initialization_complete`: Tracks whether we've rendered the first frame (for logging)
#[derive(Default)]
struct App {
    state: Option<State>,
    initialization_complete: bool,
}

impl State {
    /// Creates a new `State` instance with initialized graphics context.
    ///
    /// # Vulkan Surface Creation
    ///
    /// This method calls `RenderContext::new()`, which performs these steps:
    /// 1. Create Vulkan instance with required extensions (VK_KHR_surface, platform-specific)
    /// 2. Create surface from window using raw-window-handle trait
    /// 3. Select physical device (GPU) that supports the surface
    /// 4. Create logical device and queues
    /// 5. Query surface capabilities (supported formats, present modes, size limits)
    /// 6. Create swapchain matching window size
    ///
    /// The window is wrapped in Arc because the surface needs to outlive individual frames
    /// and Vulkan requires the window to remain alive as long as the surface exists.
    ///
    /// # Arguments
    ///
    /// * `window` - An `Arc<Window>` for which to create the state
    ///
    /// # Returns
    ///
    /// Returns `Ok(State)` on success, or an error if graphics initialization fails
    async fn new(window: Arc<Window>) -> Result<Self> {
        debug!("Creating application state");
        let state_start = std::time::Instant::now();

        // RenderContext::new() creates the Vulkan surface here
        // The surface is platform-specific: Win32 on Windows, Xlib/Wayland on Linux, Metal on macOS
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

    /// Handles window resize events by reconfiguring the Vulkan swapchain.
    ///
    /// # Swapchain Recreation
    ///
    /// When a window is resized, the Vulkan swapchain becomes incompatible because:
    /// - Swapchain images have a fixed size (width × height)
    /// - The surface's size has changed, so images no longer match
    /// - Presenting to a mismatched swapchain fails with VK_ERROR_OUT_OF_DATE
    ///
    /// To fix this, we must:
    /// 1. Wait for all GPU work to complete (device idle or fence wait)
    /// 2. Destroy old swapchain and its image views
    /// 3. Query new surface capabilities
    /// 4. Create new swapchain with updated dimensions
    /// 5. Create new image views and framebuffers
    ///
    /// This is expensive (can take several milliseconds), which is why we debounce
    /// resize events to avoid recreating the swapchain hundreds of times during
    /// a window drag operation.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new physical size of the window in pixels
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        debug!(
            "Reconfiguring surface due to resize: {}x{}",
            new_size.width, new_size.height
        );
        self.size = new_size;
        // configure_surface internally destroys old swapchain and creates new one
        self.render_context
            .configure_surface(new_size.width, new_size.height);
    }

    /// Checks if a size has valid (non-zero) dimensions.
    ///
    /// # Why Zero-Size Checking Matters
    ///
    /// - Windows report 0×0 size when minimized on most platforms
    /// - Vulkan swapchains cannot be created with zero width or height
    /// - Attempting to create a 0×0 swapchain results in validation errors
    /// - We must skip rendering entirely when the window is minimized
    fn has_valid_size(size: &winit::dpi::PhysicalSize<u32>) -> bool {
        size.width > 0 && size.height > 0
    }

    /// Determines if a resize operation should actually occur.
    ///
    /// Combines two checks:
    /// 1. New size must be valid (non-zero dimensions)
    /// 2. New size must be different from current size (avoid redundant work)
    ///
    /// # Arguments
    ///
    /// * `new_size` - The potential new physical size
    ///
    /// # Returns
    ///
    /// `true` if resize should proceed, `false` if it should be skipped
    fn should_resize(&self, new_size: winit::dpi::PhysicalSize<u32>) -> bool {
        Self::has_valid_size(&new_size)
            && (new_size.width != self.size.width || new_size.height != self.size.height)
    }

    /// Determines if rendering should occur for the current frame.
    ///
    /// Returns false when the window is minimized or has zero size to avoid:
    /// - Vulkan validation errors from attempting to render to 0×0 swapchain
    /// - Wasted GPU cycles rendering invisible content
    /// - Potential crashes from invalid framebuffer operations
    fn should_render(&self) -> bool {
        Self::has_valid_size(&self.size)
    }
}

/// Implementation of the `winit` Application Handler trait for the main application loop.
///
/// # ApplicationHandler Callbacks
///
/// This trait defines the contract between winit's event loop and your application.
/// The event loop calls these methods at specific points in the application lifecycle:
///
/// - `resumed()`: App is starting or resuming (create resources here)
/// - `suspended()`: App is suspending (not implemented - desktop apps rarely suspend)
/// - `window_event()`: Window received an event (input, resize, redraw, etc.)
/// - `device_event()`: Raw device input (not used here)
/// - `about_to_wait()`: All events processed, event loop about to wait/poll (not used here)
/// - `exiting()`: App is about to exit (not implemented - cleanup happens in Drop)
impl ApplicationHandler for App {
    /// Called when the application is resumed or started.
    ///
    /// # Lifecycle Context
    ///
    /// On desktop platforms (Windows, macOS, Linux), this is called exactly once when the event
    /// loop starts. On mobile platforms (iOS, Android), it can be called multiple times as the
    /// app is suspended and resumed by the OS.
    ///
    /// # Why Window Creation Happens Here
    ///
    /// Windows can only be created when the event loop is active because:
    /// - The event loop provides the `ActiveEventLoop` parameter
    /// - OS windowing systems require an event loop context
    /// - Some platforms don't have windows until the app is foregrounded
    ///
    /// # Initialization Steps
    ///
    /// 1. Check if already initialized (guard against multiple resume calls)
    /// 2. Create window with default attributes (size, title, resizable flag)
    /// 3. Initialize State (which creates Vulkan surface and swapchain)
    /// 4. Request initial redraw to start the render loop
    /// 5. Store state in Option for access in window_event()
    ///
    /// If any step fails, we log an error and exit the event loop cleanly.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("Application resumed");

        // Guard against double-initialization (can happen on mobile platforms)
        if self.state.is_some() {
            trace!("State already initialized, skipping");
            return;
        }

        // Create the OS window with default attributes
        // The ActiveEventLoop is required here - windows cannot exist outside event loop context
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

        // Initialize graphics state (Vulkan surface creation happens inside State::new)
        // pollster::block_on is used because State::new is async (some graphics APIs require async init)
        let state = match pollster::block_on(State::new(window.clone())) {
            Ok(state) => {
                trace!("Requesting initial redraw");
                // Request the first redraw to start the rendering loop
                // Without this, the application would wait indefinitely for an event
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

    /// Called for each window event dispatched by the OS.
    ///
    /// # Event Loop Flow
    ///
    /// The event loop continuously:
    /// 1. Polls for OS events (input, window changes, system messages)
    /// 2. Dispatches each event to this callback
    /// 3. Repeats until `event_loop.exit()` is called
    ///
    /// In Poll mode (set in run()), the loop never blocks - it calls this method as fast as
    /// possible, allowing for smooth real-time rendering. In Wait mode, it would block until
    /// an event arrives, saving CPU but unsuitable for games.
    ///
    /// # Key Events Handled
    ///
    /// - **CloseRequested**: User clicked X button or pressed Alt+F4
    /// - **RedrawRequested**: Time to render a frame
    /// - **Resized**: Window size changed
    /// - **KeyboardInput**: Keyboard event (we handle Escape to exit)
    ///
    /// # Parameters
    ///
    /// * `event_loop` - Active event loop (can call exit() to quit)
    /// * `_id` - Window ID (unused because we only have one window)
    /// * `event` - The specific window event that occurred
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Access state or return early if not initialized yet
        // This can happen if events arrive before resumed() completes
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
            // User requested to close the window (X button, Alt+F4, etc.)
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting event loop...");
                event_loop.exit();
            }

            // Time to render a frame
            // This is the heart of the rendering loop for real-time graphics
            WindowEvent::RedrawRequested => {
                // Update frame timer to track delta time and FPS
                let delta = state.frame_timer.tick();

                // Process any pending resize with debouncing
                // This is where swapchain recreation actually happens
                if let Some((pending_size, resize_time)) = state.pending_resize {
                    const DEBOUNCE_DURATION: Duration = Duration::from_millis(16); // ~1 frame at 60fps

                    // Has enough time passed since the last resize event?
                    if resize_time.elapsed() >= DEBOUNCE_DURATION {
                        if state.should_resize(pending_size) {
                            debug!(
                                "Processing debounced resize to: {}x{}",
                                pending_size.width, pending_size.height
                            );
                            // This calls configure_surface, which recreates the swapchain
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
                        // Still within debounce window - skip rendering and wait for next frame
                        // This prevents rendering to the old swapchain while resize is pending
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
                    // Request next frame to maintain continuous rendering loop
                    // Without this, we'd only get one frame
                    state.window.request_redraw();
                } else {
                    trace!("Skipping frame tick - window minimized or zero size");
                }

                if !self.initialization_complete {
                    self.initialization_complete = true;
                    info!("Window initialization complete, rendering started");
                }
            }

            // Window size changed (user dragged border, maximized, restored, etc.)
            WindowEvent::Resized(size) => {
                // Update stored size immediately (even if we defer swapchain recreation)
                state.size = size;

                if state.should_resize(size) {
                    debug!("Received resize event: {}x{}", size.width, size.height);
                    // Store the resize request with current timestamp for debouncing
                    // The actual swapchain recreation happens in RedrawRequested
                    state.pending_resize = Some((size, Instant::now()));
                }

                // Request redraw to process the resize or continue rendering
                state.window.request_redraw();
            }

            // Escape key pressed - exit application
            // This is a convenience feature for quick testing/debugging
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

            // Ignore all other events (mouse movement, focus changes, etc.)
            _ => (),
        }
    }
}

/// Runs the main application event loop.
///
/// # Event Loop Setup
///
/// This is the entry point for the window management system. It:
/// 1. Creates the default App struct (empty state)
/// 2. Creates the platform event loop
/// 3. Sets control flow to Poll mode (run as fast as possible)
/// 4. Runs the event loop, which calls App's ApplicationHandler methods
///
/// # Control Flow Modes
///
/// - **Poll**: Event loop runs continuously without blocking (used here for games)
///   - Pros: Smooth real-time rendering, predictable frame pacing
///   - Cons: Uses CPU constantly (even if no events)
///
/// - **Wait**: Event loop blocks until an event arrives
///   - Pros: Energy efficient, no busy waiting
///   - Cons: Unsuitable for real-time graphics (would freeze between events)
///
/// - **WaitUntil**: Blocks until event or timeout
///   - Pros: Balance between responsiveness and efficiency
///   - Cons: More complex timing logic
///
/// # Error Handling
///
/// Event loop creation can fail if:
/// - Platform doesn't support windowing (headless server, broken display)
/// - Required OS resources unavailable
/// - Platform-specific initialization errors
///
/// The event loop itself runs until exit() is called, then returns Ok(()).
/// Any panics during event handling will crash the application (by design).
///
/// # Returns
///
/// Returns `Ok(())` if the application exits cleanly, or an error if loop creation fails.
pub fn run() -> Result<()> {
    info!("Starting Praxis application");
    let app_start = std::time::Instant::now();

    let mut app = App::default();

    debug!("Creating event loop...");
    // EventLoop::new() creates platform-specific event loop (Win32, Cocoa, X11, Wayland)
    let event_loop =
        EventLoop::new().map_err(|e| eyre::eyre!("Failed to create event loop: {}", e))?;

    // Set to Poll mode: run continuously without waiting for events
    // This is essential for real-time graphics - we need to render every frame
    event_loop.set_control_flow(ControlFlow::Poll);
    trace!("Event loop control flow set to Poll mode");

    info!(
        "Starting event loop (initialized in {:?})",
        app_start.elapsed()
    );

    // Run the event loop - this blocks until event_loop.exit() is called
    // The event loop will call app.resumed() once, then app.window_event() for each event
    event_loop
        .run_app(&mut app)
        .map_err(|e| eyre::eyre!("Event loop error: {}", e))?;

    info!("Application shutdown complete");
    Ok(())
}
