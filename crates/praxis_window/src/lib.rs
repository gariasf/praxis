//! Window management system for the Praxis engine.
//!
//! This crate provides cross-platform window creation, event handling, and platform abstraction
//! using the `winit` library. It is deliberately decoupled from graphics initialization to avoid
//! circular dependencies and provide flexibility in graphics backend choice.
//!
//! # Design Philosophy
//!
//! This crate follows the principle of **separation of concerns**:
//! - **Window management** (this crate): Window creation, event loop, input events, resize handling
//! - **Graphics initialization** (praxis_graphics): Vulkan surface creation, swapchain, rendering
//!
//! By keeping these separate, we:
//! - Avoid circular dependencies between praxis_window and praxis_graphics
//! - Allow graphics backends to be swapped without changing window code
//! - Enable headless testing and non-graphics applications
//! - Follow the architecture of professional engines (Unity, Unreal, Godot)
//!
//! # Winit Integration Overview
//!
//! Winit provides a platform-agnostic event loop and window creation API. Key concepts:
//!
//! - **EventLoop**: The central dispatcher that receives OS events (keyboard, mouse, window changes)
//! - **ApplicationHandler**: A trait that defines callbacks for different lifecycle stages
//! - **Window**: The actual OS window that can be used for graphics rendering
//! - **WindowEvent**: Events specific to a window (resize, close, input, etc.)
//! - **Raw Window Handle**: Platform-specific handle for graphics API integration
//!
//! # Architecture Patterns
//!
//! ## Window Builder Pattern
//!
//! Configure window attributes before creation:
//!
//! ```rust,ignore
//! use praxis_window::WindowConfig;
//!
//! let config = WindowConfig::default()
//!     .with_title("My Game")
//!     .with_size(1920, 1080)
//!     .with_resizable(true)
//!     .with_maximized(false);
//! ```
//!
//! ## Event Handler Trait
//!
//! Implement custom event handling logic:
//!
//! ```rust,ignore
//! use praxis_window::{WindowEventHandler, WindowEvent};
//!
//! struct MyApp {
//!     // Your application state
//! }
//!
//! impl WindowEventHandler for MyApp {
//!     fn on_init(&mut self, window: &Window) {
//!         // Initialize graphics, load assets, etc.
//!     }
//!
//!     fn on_update(&mut self, delta_time: f32) {
//!         // Update game logic
//!     }
//!
//!     fn on_render(&mut self, window: &Window) {
//!         // Render frame
//!     }
//!
//!     fn on_resize(&mut self, width: u32, height: u32) {
//!         // Recreate swapchain, update camera aspect ratio, etc.
//!     }
//!
//!     fn on_close(&mut self) -> bool {
//!         // Return true to allow close, false to prevent
//!         true
//!     }
//! }
//! ```
//!
//! # Event Loop Lifecycle
//!
//! The event loop follows this lifecycle:
//!
//! 1. **EventLoop Creation**: Platform-specific event loop is created
//! 2. **Window Creation**: OS window is created during `resumed()` callback
//! 3. **Initialization**: User handler's `on_init()` is called with window reference
//! 4. **Main Loop**: Continuously processes events and calls user callbacks
//!    - `on_update()`: Called each frame before rendering
//!    - `on_render()`: Called when window needs redraw
//!    - `on_resize()`: Called when window size changes
//!    - `on_input()`: Called for keyboard/mouse events
//! 5. **Shutdown**: User handler's `on_close()` is called, resources cleaned up
//!
//! # Resize Handling Strategy
//!
//! Window resizing is complex in real-time graphics because:
//!
//! 1. **Rapid Events**: OS sends many resize events during drag operations
//! 2. **Graphics Sync**: Graphics backends may need to recreate resources (swapchains, framebuffers)
//! 3. **Zero-Size Windows**: Minimized windows report 0×0, invalid for rendering
//! 4. **Race Conditions**: Resize events can arrive during rendering
//!
//! This crate implements **resize debouncing**:
//! - Store latest resize with timestamp
//! - Wait for short delay (16ms ≈ 1 frame) before notifying handler
//! - If another resize arrives during delay, replace pending resize
//! - Skip zero-size resizes entirely
//!
//! This reduces handler notifications from hundreds to just a few per resize operation.
//!
//! # Raw Window Handles
//!
//! For graphics API integration (Vulkan, DirectX, Metal), windows provide raw handles:
//!
//! ```rust,ignore
//! use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
//!
//! // Window implements these traits automatically via winit
//! let window_handle = window.window_handle()?;
//! let display_handle = window.display_handle()?;
//!
//! // Pass to Vulkan/DirectX/Metal for surface creation
//! // (This happens in praxis_graphics, not here)
//! ```
//!
//! Platform-specific handles:
//! - **Windows**: HWND (Win32 window handle)
//! - **macOS**: NSView/NSWindow (Cocoa)
//! - **Linux**: XWindow (X11) or WlSurface (Wayland)
//! - **Android**: ANativeWindow
//! - **iOS**: UIView
//!
//! # Control Flow Modes
//!
//! The event loop supports different control flow strategies:
//!
//! - **Poll** (default): Runs continuously without blocking
//!   - Use for: Real-time graphics, games, simulations
//!   - Pros: Smooth rendering, predictable frame pacing
//!   - Cons: High CPU usage even when idle
//!
//! - **Wait**: Blocks until an event arrives
//!   - Use for: Tools, editors, non-real-time applications
//!   - Pros: Energy efficient, minimal CPU usage
//!   - Cons: No continuous rendering without events
//!
//! - **WaitUntil**: Blocks until event or timeout
//!   - Use for: Applications with target frame rate
//!   - Pros: Balance between efficiency and responsiveness
//!   - Cons: More complex timing management
//!
//! # Examples
//!
//! ## Basic Window Creation
//!
//! ```rust,ignore
//! use praxis_window::{WindowConfig, WindowManager};
//!
//! fn main() -> Result<()> {
//!     praxis_utils::init()?;
//!
//!     let config = WindowConfig::default()
//!         .with_title("Hello Window")
//!         .with_size(800, 600);
//!
//!     let mut manager = WindowManager::new(config)?;
//!     manager.run()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## With Custom Event Handler
//!
//! ```rust,ignore
//! use praxis_window::{WindowConfig, WindowManager, WindowEventHandler};
//!
//! struct MyApp {
//!     frame_count: u32,
//! }
//!
//! impl WindowEventHandler for MyApp {
//!     fn on_render(&mut self, _window: &Window) {
//!         self.frame_count += 1;
//!         if self.frame_count % 60 == 0 {
//!             println!("60 frames rendered");
//!         }
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     let mut app = MyApp { frame_count: 0 };
//!     let mut manager = WindowManager::with_handler(WindowConfig::default(), app)?;
//!     manager.run()?;
//!     Ok(())
//! }
//! ```
//!
//! # Thread Safety
//!
//! Windows and event loops have platform-specific thread requirements:
//! - **Windows/Linux**: Can be created on any thread
//! - **macOS**: Must be created on main thread
//! - **Mobile**: OS controls thread lifecycle
//!
//! For maximum compatibility, create windows and event loops on the main thread.
//!
//! # Platform Differences
//!
//! ## Desktop (Windows, macOS, Linux)
//! - Event loop runs until explicitly exited
//! - Windows persist across suspend/resume
//! - `resumed()` called once at startup
//!
//! ## Mobile (iOS, Android)
//! - OS controls app lifecycle
//! - Apps can be suspended/resumed frequently
//! - `resumed()` may be called multiple times
//! - Windows destroyed on suspend, recreated on resume
//!
//! # See Also
//!
//! - [`WindowConfig`]: Configuration for window creation
//! - [`WindowManager`]: Main entry point for window management
//! - [`WindowEventHandler`]: Trait for custom event handling
//! - [`WindowEvent`]: Enumeration of window events
//! - [`winit` documentation](https://docs.rs/winit) for low-level details

mod config;
mod event_handler;
mod manager;
pub mod utils;
mod window_events;

// Re-export main public API
pub use config::WindowConfig;
pub use event_handler::WindowEventHandler;
pub use manager::WindowManager;
pub use window_events::WindowResizeStrategy;

// Re-export key winit types for convenience
pub use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton},
    event_loop::ControlFlow,
    keyboard::{Key, KeyCode, NamedKey},
    window::Window,
};

// Re-export raw window handle traits for graphics integration
pub use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

// Re-export common result type
pub use praxis_utils::Result;

/// Runs a basic window with no custom event handling.
///
/// This is a convenience function that creates a window with default configuration
/// and runs it until closed. Useful for simple applications or backward compatibility.
///
/// For more control, use `WindowManager` directly with a custom `WindowEventHandler`.
///
/// # Errors
///
/// Returns an error if window creation or event loop initialization fails.
///
/// # Examples
///
/// ```rust,ignore
/// fn main() -> praxis_utils::Result<()> {
///     praxis_utils::init()?;
///     praxis_window::run()?;
///     Ok(())
/// }
/// ```
pub fn run() -> Result<()> {
    let config = WindowConfig::default();
    let manager = WindowManager::new(config)?;
    manager.run()
}
