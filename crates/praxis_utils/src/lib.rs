//! Utility functions for the Praxis engine.
//!
//! This crate provides common utilities used throughout the engine,
//! including tracing and logging capabilities.
//!
//! # Overview
//!
//! `praxis_utils` serves as the foundational utility layer for the Praxis engine,
//! providing three critical capabilities:
//!
//! 1. **Structured Logging with `tracing`** - Context-rich, performant logging
//! 2. **Error Reporting with `color-eyre`** - Beautiful, actionable error messages
//! 3. **Timing Utilities** - Frame delta time and FPS tracking
//!
//! # Structured Logging with `tracing`
//!
//! Unlike traditional logging that simply prints strings, `tracing` provides
//! **structured, context-aware logging** with hierarchical spans and events.
//!
//! ## Why `tracing` instead of `log`?
//!
//! - **Spans**: Track execution flow and timing of code blocks
//! - **Context**: Attach structured data to logs (not just strings)
//! - **Performance**: Zero-cost abstractions when logging is disabled
//! - **Composability**: Filter and route logs to multiple backends
//!
//! ## Basic Usage
//!
//! ```rust
//! use praxis_utils::{info, debug, error, warn};
//!
//! fn load_asset(path: &str) -> Result<(), String> {
//!     info!("Loading asset from {}", path);
//!     
//!     // Structured fields (not string concatenation)
//!     debug!(asset_path = %path, "Starting asset load");
//!     
//!     // ... actual loading logic ...
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Instrumentation with Spans
//!
//! Spans provide hierarchical context and automatic timing:
//!
//! ```rust
//! use praxis_utils::instrument;
//!
//! #[instrument] // Automatically logs function entry/exit with args
//! fn process_mesh(name: &str, vertex_count: usize) {
//!     // All logs within this function automatically include
//!     // the function name and arguments in their context
//!     praxis_utils::info!("Processing mesh vertices");
//! }
//! ```
//!
//! Output includes automatic context:
//! ```text
//! DEBUG process_mesh{name="sphere.obj" vertex_count=1024}: Processing mesh vertices
//! ```
//!
//! ## Environment Variable Configuration
//!
//! Control logging levels via `RUST_LOG` environment variable:
//!
//! ```bash
//! # All modules at debug level
//! RUST_LOG=debug cargo run
//!
//! # Different levels per module
//! RUST_LOG=praxis_graphics=trace,praxis_physics=debug,praxis_core=info cargo run
//!
//! # Quiet third-party crates
//! RUST_LOG=warn,praxis=debug cargo run
//! ```
//!
//! The engine sets sensible defaults:
//! - `debug` level globally (if `RUST_LOG` not set)
//! - `info` for `winit` (reduces window event spam)
//! - `debug` for `vulkano` (useful GPU debugging)
//!
//! # Error Reporting with `color-eyre`
//!
//! `color-eyre` provides enhanced error reporting with:
//! - **Colorful output**: Syntax-highlighted error chains
//! - **Context capture**: Automatic capture of spans when errors occur
//! - **Suggestions**: Actionable hints for fixing errors
//! - **Backtraces**: Optional stack traces for debugging
//!
//! ## The `Result` Type
//!
//! This crate re-exports a convenient `Result` type:
//!
//! ```rust
//! use praxis_utils::Result; // Alias for Result<T, color_eyre::Report>
//!
//! fn init_engine() -> Result<()> {
//!     // Automatically converts any error type that implements std::error::Error
//!     let data = std::fs::read_to_string("config.toml")?;
//!     
//!     // Add context to errors with .wrap_err()
//!     // let config: Config = toml::from_str(&data)
//!     //     .wrap_err("Failed to parse engine configuration")?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Error Context Patterns
//!
//! The `errors` module provides extension traits for adding context to errors.
//! Common patterns used throughout the engine:
//!
//! ```rust,ignore
//! use praxis_utils::{Result, WrapErr, Context, bail, ensure};
//!
//! // Pattern 1: Add context to fallible operations
//! fn load_shader(path: &str) -> Result<ShaderModule> {
//!     std::fs::read(path)
//!         .wrap_err_with(|| format!("Failed to read shader file: {}", path))?;
//!     // ...
//! }
//!
//! // Pattern 2: Convert Option to Result with context
//! fn find_entity(id: u32) -> Result<Entity> {
//!     world.get(id)
//!         .context("Entity not found")
//! }
//!
//! // Pattern 3: Validation with ensure!
//! fn validate_mesh(mesh: &Mesh) -> Result<()> {
//!     ensure!(!mesh.vertices.is_empty(), "Mesh has no vertices");
//!     Ok(())
//! }
//!
//! // Pattern 4: Early return with bail!
//! fn check_requirements() -> Result<()> {
//!     if !requirements_met() {
//!         bail!("Requirements not met");
//!     }
//!     Ok(())
//! }
//! ```
//!
//! See the [`errors`] module documentation for more patterns and best practices.
//!
//! ## Error Chain Example
//!
//! When an error occurs deep in the call stack, `color-eyre` displays
//! a helpful chain showing how the error propagated:
//!
//! ```text
//! Error: Failed to initialize graphics subsystem
//!
//! Caused by:
//!    0: Failed to create Vulkan instance
//!    1: VK_ERROR_INCOMPATIBLE_DRIVER
//!
//! Suggestion: Ensure Vulkan drivers are installed
//!
//! Backtrace: (enable with RUST_BACKTRACE=1)
//! ```
//!
//! # Initialization Patterns
//!
//! ## Simple Initialization
//!
//! Most applications use the simple `init()` function:
//!
//! ```rust,ignore
//! fn main() -> praxis_utils::Result<()> {
//!     // Initialize utilities first, before any logging
//!     praxis_utils::init()?;
//!     
//!     // Now you can use logging
//!     info!("Application started");
//!     
//!     // Rest of your application
//!     run_game()?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Editor/Custom Initialization
//!
//! For advanced use cases (like the editor capturing logs), use custom layers:
//!
//! ```rust,ignore
//! use praxis_utils::init_tracing_with_layer;
//! use praxis_editor::ConsoleLayer;
//!
//! fn main() -> praxis_utils::Result<()> {
//!     // Create a custom layer that captures logs to a buffer
//!     let log_buffer = Arc::new(Mutex::new(Vec::new()));
//!     let console_layer = ConsoleLayer::new(log_buffer.clone());
//!     
//!     // Initialize with custom layer
//!     init_tracing_with_layer(Some(console_layer))?;
//!     
//!     // Logs now go to both console and the buffer
//!     info!("Editor started");
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Timing Utilities
//!
//! The `timing` module provides frame timing and delta time tracking:
//!
//! ## Global Timing Access
//!
//! ```rust,ignore
//! use praxis_utils::timing::{delta_time, current_fps, frame_count};
//!
//! fn update_system(mut query: Query<(&mut Transform, &Velocity)>) {
//!     let dt = delta_time(); // Seconds since last frame
//!     
//!     for (mut transform, velocity) in query.iter_mut() {
//!         transform.translation += velocity.0 * dt;
//!     }
//! }
//! ```
//!
//! ## Frame Timer for Main Loop
//!
//! ```rust,ignore
//! use praxis_utils::timing::FrameTimer;
//!
//! fn main() -> Result<()> {
//!     let mut timer = FrameTimer::new_with_global(); // Updates global timing
//!     timer.set_target_fps(Some(60.0)); // Optional FPS cap
//!     
//!     loop {
//!         // Update timing (must be called once per frame)
//!         timer.tick();
//!         
//!         // Update game logic
//!         update_systems();
//!         
//!         // Render frame
//!         render();
//!         
//!         // Sleep to maintain target FPS (optional)
//!         timer.sleep_if_needed();
//!         
//!         // Display stats periodically
//!         if frame_count() % 60 == 0 {
//!             info!("{}", timer.stats());
//!         }
//!     }
//! }
//! ```
//!
//! ## Delta Time Clamping
//!
//! The timer automatically clamps delta time to prevent huge jumps:
//! - **Problem**: If the game pauses for 5 seconds, `delta_time` would be 5.0s
//! - **Solution**: Clamped to 100ms (0.1s) maximum
//! - **Benefit**: Physics simulations remain stable after debugger breakpoints
//!
//! # Common Patterns Throughout the Engine
//!
//! ## Pattern 1: Function Entry Logging
//!
//! ```rust,ignore
//! #[instrument(skip(context))] // Skip large arguments
//! pub fn render_frame(context: &RenderContext, scene: &Scene) -> Result<()> {
//!     debug!("Rendering frame");
//!     // Automatically logs "render_frame{scene=...}" at entry/exit
//! }
//! ```
//!
//! ## Pattern 2: Error Context Chain
//!
//! ```rust,ignore
//! // Low-level function
//! fn read_file(path: &Path) -> Result<Vec<u8>> {
//!     std::fs::read(path)
//!         .wrap_err_with(|| format!("Failed to read file: {}", path.display()))?
//! }
//!
//! // Mid-level function adds more context
//! fn parse_gltf(path: &Path) -> Result<GltfScene> {
//!     let data = read_file(path)
//!         .wrap_err("Failed to load GLTF file")?;
//!     // Parse...
//! }
//!
//! // High-level function adds user-facing context
//! fn load_scene(name: &str) -> Result<Scene> {
//!     parse_gltf(Path::new(name))
//!         .wrap_err_with(|| format!("Failed to load scene '{}'", name))?
//! }
//! ```
//!
//! ## Pattern 3: Conditional Logging
//!
//! ```rust,ignore
//! // Only evaluated if debug logging is enabled (zero cost otherwise)
//! debug!(?vertex_buffer, "Created vertex buffer"); // Debug format
//! trace!(count = %meshes.len(), "Processing meshes"); // Display format
//! ```
//!
//! ## Pattern 4: Error + Log
//!
//! ```rust,ignore
//! fn critical_operation() -> Result<()> {
//!     perform_operation().map_err(|e| {
//!         error!("Critical operation failed: {}", e);
//!         e // Return error for caller to handle
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! # Performance Considerations
//!
//! - **Logging**: Macros compile to no-ops when disabled (zero cost)
//! - **Instrumentation**: Minimal overhead, but skip large arguments with `#[instrument(skip(...))]`
//! - **Error handling**: `?` operator has zero cost compared to manual error handling
//! - **Global timing**: Uses `OnceLock` + `Mutex`, but only one write per frame
//!
//! # See Also
//!
//! - [`tracing` documentation](https://docs.rs/tracing)
//! - [`color-eyre` documentation](https://docs.rs/color-eyre)
//! - [`observability`] module for detailed tracing initialization
//! - [`timing`] module for frame timing utilities
//! - [`errors`] module for error handling utilities

mod observability;
pub mod errors;
pub mod timing;

pub use observability::{init_tracing, init_tracing_with_layer};

// Re-export common utility items for convenience
pub use color_eyre::{
    eyre::{self, Error},
    Report, Result,
};

// Re-export error utilities
pub use errors::{bail, ensure, Context, WrapErr};

// Re-export tracing macros for direct use from other crates
pub use tracing::{debug, error, info, instrument, trace, warn};

/// Initializes the utility library.
///
/// This function sets up the tracing system and error reporting system.
/// **Must be called before any logging occurs**, typically as the first line
/// in `main()`.
///
/// # What it does
///
/// 1. Installs `color-eyre` error handler (enables pretty error reports)
/// 2. Initializes `tracing` subscriber (enables logging)
/// 3. Configures default log levels (debug globally, info for winit, debug for vulkano)
/// 4. Sets up log formatting (pretty, with thread IDs and span events)
///
/// # Returns
///
/// Returns `Ok(())` if initialization succeeds, or an error if:
/// - The tracing subscriber is already initialized (called twice)
/// - The color-eyre panic handler is already installed
///
/// # Errors
///
/// Returns an error if the tracing subscriber or color-eyre panic handler
/// are already initialized (typically from calling this function twice).
///
/// # Examples
///
/// ```no_run
/// fn main() -> praxis_utils::Result<()> {
///     // First thing: initialize utilities
///     praxis_utils::init()?;
///     
///     // Now logging is available
///     praxis_utils::info!("Application started");
///     
///     // Rest of your application
///     Ok(())
/// }
/// ```
///
/// # Thread Safety
///
/// This function is safe to call from any thread, but should only be called
/// once in the lifetime of the application (typically in `main()`).
pub fn init() -> Result<()> {
    observability::init_tracing()?;
    info!("Praxis utilities initialized");

    Ok(())
}
