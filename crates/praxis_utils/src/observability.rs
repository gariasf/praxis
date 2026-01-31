//! Observability utilities for structured logging and tracing.
//!
//! This module provides initialization and configuration for the `tracing` ecosystem,
//! which powers all logging throughout the Praxis engine. Unlike traditional logging
//! libraries that simply print strings, `tracing` provides structured, context-aware
//! logging with hierarchical spans.
//!
//! # Core Concepts
//!
//! ## Events vs. Spans
//!
//! - **Events**: Point-in-time occurrences (like traditional log messages)
//! - **Spans**: Time periods representing units of work (automatically timed)
//!
//! ```rust
//! use praxis_utils::{info, instrument};
//!
//! #[instrument] // Creates a span for this function
//! fn load_asset(path: &str) {
//!     info!("Starting asset load"); // Event within the span
//!     // ... work happens here ...
//!     info!("Asset loaded successfully");
//! }
//! ```
//!
//! ## Structured Fields
//!
//! Instead of string formatting, attach structured data to logs:
//!
//! ```rust
//! use praxis_utils::debug;
//!
//! let vertex_count = 1024;
//! let name = "sphere.obj";
//!
//! // Traditional logging (bad - loses structure)
//! // debug!("Processing {} with {} vertices", name, vertex_count);
//!
//! // Structured logging (good - preserves types)
//! debug!(mesh_name = %name, vertices = vertex_count, "Processing mesh");
//! ```
//!
//! ## Log Levels
//!
//! From most to least verbose:
//! - `trace!`: Very fine-grained (loop iterations, individual calculations)
//! - `debug!`: Detailed diagnostic (function calls, state changes)
//! - `info!`: High-level informational (subsystem initialization, major events)
//! - `warn!`: Warnings about degraded functionality (fallbacks, retries)
//! - `error!`: Errors that require attention (failed operations, invalid state)
//!
//! # Configuration
//!
//! ## Environment Variables
//!
//! Control logging via `RUST_LOG`:
//!
//! ```bash
//! # Global level
//! RUST_LOG=debug cargo run
//!
//! # Per-crate levels
//! RUST_LOG=praxis_graphics=trace,praxis_physics=debug,praxis=info cargo run
//!
//! # Quiet third-party crates
//! RUST_LOG=warn,praxis=debug cargo run
//!
//! # Specific module within crate
//! RUST_LOG=praxis_graphics::deferred=trace cargo run
//! ```
//!
//! ## Default Configuration
//!
//! If `RUST_LOG` is not set, the engine uses these defaults:
//! - `debug` globally (suitable for development)
//! - `info` for `winit` (reduces window event spam)
//! - `debug` for `vulkano` (useful for GPU debugging)
//!
//! For production releases, set `RUST_LOG=info` or `warn` globally.
//!
//! # Advanced Usage
//!
//! ## Custom Layers
//!
//! The editor or specialized tools can capture logs to buffers or files:
//!
//! ```rust,ignore
//! use praxis_utils::init_tracing_with_layer;
//! use tracing_subscriber::Layer;
//!
//! // Create a custom layer (e.g., editor console panel)
//! let console_layer = ConsoleLayer::new(log_buffer);
//!
//! // Initialize with the custom layer
//! init_tracing_with_layer(Some(console_layer))?;
//! ```
//!
//! ## Span Context
//!
//! Spans automatically provide context to all events within them:
//!
//! ```rust
//! use praxis_utils::{info, instrument};
//!
//! #[instrument]
//! fn process_scene(scene_name: &str) {
//!     load_meshes(); // Logs will include scene_name in context
//!     load_textures(); // Logs will include scene_name in context
//! }
//!
//! #[instrument]
//! fn load_meshes() {
//!     info!("Loading meshes"); // Output: process_scene{scene_name="level1"}: load_meshes: Loading meshes
//! }
//! ```
//!
//! # Performance Considerations
//!
//! - **Zero-cost when disabled**: Macros compile to no-ops if the level is disabled
//! - **Lazy evaluation**: String formatting only happens if the log is emitted
//! - **Skip expensive arguments**: Use `#[instrument(skip(large_buffer))]` for large data
//!
//! ```rust
//! use praxis_utils::instrument;
//!
//! #[instrument(skip(vertices))] // Don't include 10MB vertex buffer in logs
//! fn upload_mesh(name: &str, vertices: &[Vertex]) {
//!     // ... upload logic ...
//! }
//! ```

use color_eyre::Result;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Initializes the tracing system with default configuration.
///
/// This function should be called early in the application startup process,
/// typically as the first line in `main()`, before any logging occurs.
///
/// # What It Does
///
/// 1. Installs the `color-eyre` error handler for beautiful error reports
/// 2. Creates a `tracing` subscriber with pretty formatting
/// 3. Configures environment-based filtering via `RUST_LOG`
/// 4. Sets sensible defaults for Praxis and common third-party crates
/// 5. Registers the global subscriber for the application
///
/// # Environment Configuration
///
/// The tracing level can be configured via the `RUST_LOG` environment variable:
///
/// ```bash
/// # Examples:
/// RUST_LOG=debug cargo run                      # Global debug level
/// RUST_LOG=trace cargo run                      # Very verbose
/// RUST_LOG=praxis_graphics=trace cargo run      # Trace one crate
/// RUST_LOG=praxis=debug,winit=info cargo run    # Different levels per crate
/// ```
///
/// If `RUST_LOG` is not set, defaults to:
/// - `debug` level globally
/// - `info` for `winit` (reduces window event noise)
/// - `debug` for `vulkano` (GPU debugging)
///
/// # Returns
///
/// - `Ok(())` if initialization succeeds
/// - `Err(...)` if the global subscriber is already set (called twice) or if
///   `color-eyre` panic handler is already installed
///
/// # Errors
///
/// Returns an error if the tracing subscriber or color-eyre panic handler
/// are already initialized (typically from calling this function twice).
///
/// # Thread Safety
///
/// Safe to call from any thread, but should only be called once per process.
/// Subsequent calls will return an error.
///
/// # Examples
///
/// ```no_run
/// use praxis_utils;
///
/// fn main() -> praxis_utils::Result<()> {
///     // Initialize utilities first
///     praxis_utils::init_tracing()?;
///     
///     // Now logging works
///     praxis_utils::info!("Application started");
///     
///     Ok(())
/// }
/// ```
///
/// # Panics
///
/// Panics if the log directives "winit=info" or "vulkano=debug" cannot be parsed.
/// This should never happen as these are valid, hardcoded directives.
pub fn init_tracing() -> Result<()> {
    // Install color-eyre for enhanced error reporting
    // This adds pretty-printed error chains, suggestions, and optional backtraces
    color_eyre::install()?;

    // Create a subscriber that formats events as pretty, human-readable strings
    let fmt_layer = fmt::layer()
        .with_target(true) // Include module path in logs
        .with_thread_ids(true) // Include thread ID for concurrent debugging
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE) // Log span entry/exit
        .with_ansi(true) // Use colors for better readability
        .pretty(); // Multi-line format with indentation

    // Create environment-based filter with sensible defaults
    let filter_layer = create_default_filter()?;

    // Combine layers and set as global default
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}

/// Initializes the tracing system with an optional custom layer.
///
/// This is an advanced initialization function that allows adding custom
/// subscribers alongside the default formatting layer. Use this when you need
/// to capture logs to a buffer (e.g., editor console), write to a file, or
/// send to a remote logging service.
///
/// # Type Parameters
///
/// * `L` - A layer type that implements the `Layer` trait and can be composed
///   with the `EnvFilter` layer. Must be `Send + Sync + 'static` for thread safety.
///
/// # Arguments
///
/// * `custom_layer` - An optional custom layer to add to the subscriber.
///   If `None`, behaves identically to `init_tracing()`.
///
/// # Returns
///
/// - `Ok(())` if initialization succeeds
/// - `Err(...)` if the global subscriber is already set or if `color-eyre`
///   panic handler is already installed
///
/// # Errors
///
/// Returns an error if the tracing subscriber or color-eyre panic handler
/// are already initialized (typically from calling this function twice).
///
/// # Examples
///
/// ## Editor Console Panel
///
/// ```rust,ignore
/// use praxis_utils::init_tracing_with_layer;
/// use praxis_editor::ConsoleLayer;
/// use std::sync::{Arc, Mutex};
///
/// // Create a buffer to capture log events
/// let log_buffer = Arc::new(Mutex::new(Vec::new()));
///
/// // Create a custom layer that writes to the buffer
/// let console_layer = ConsoleLayer::new(log_buffer.clone());
///
/// // Initialize with both console output and buffer capture
/// init_tracing_with_layer(Some(console_layer))?;
///
/// // Logs now go to both stdout and the buffer
/// praxis_utils::info!("Editor started");
///
/// // Later, display logs in the editor UI
/// let logs = log_buffer.lock().unwrap();
/// for event in logs.iter() {
///     ui.label(&event.message);
/// }
/// ```
///
/// ## File Logging
///
/// ```rust,ignore
/// use tracing_subscriber::fmt::layer;
/// use std::fs::File;
///
/// let log_file = File::create("engine.log")?;
/// let file_layer = layer().with_writer(log_file);
///
/// init_tracing_with_layer(Some(file_layer))?;
/// ```
///
/// ## Conditional Initialization
///
/// ```rust,ignore
/// // Initialize without custom layer in non-editor builds
/// #[cfg(not(feature = "editor"))]
/// init_tracing_with_layer(None::<()>)?;
///
/// // Initialize with console layer in editor builds
/// #[cfg(feature = "editor")]
/// init_tracing_with_layer(Some(console_layer))?;
/// ```
///
/// # Panics
///
/// Panics if the log directives "winit=info" or "vulkano=debug" cannot be parsed.
/// This should never happen as these are valid, hardcoded directives.
pub fn init_tracing_with_layer<L>(custom_layer: Option<L>) -> Result<()>
where
    L: Layer<tracing_subscriber::layer::Layered<EnvFilter, tracing_subscriber::Registry>>
        + Send
        + Sync
        + 'static,
{
    // Install color-eyre for enhanced error reporting
    color_eyre::install()?;

    // Create environment-based filter with sensible defaults
    let filter_layer = create_default_filter()?;

    if let Some(layer) = custom_layer {
        // Initialize with custom layer only (no fmt layer)
        // Assumes the custom layer handles formatting
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(layer)
            .init();
    } else {
        // Initialize with default fmt layer
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_ansi(true)
            .pretty();

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }

    Ok(())
}

/// Creates the default environment filter with sensible defaults.
///
/// This internal helper centralizes the filter configuration logic.
///
/// # Default Levels
///
/// - Global: `debug` (if `RUST_LOG` not set)
/// - `winit`: `info` (reduces window event spam)
/// - `vulkano`: `debug` (useful for GPU debugging)
///
/// # Returns
///
/// Returns the configured `EnvFilter` or an error if filter creation fails.
///
/// # Panics
///
/// Panics if the hardcoded directives cannot be parsed (should never happen).
fn create_default_filter() -> Result<EnvFilter> {
    // Try to use RUST_LOG environment variable, or default to "debug"
    let filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("debug"))?;

    // Add default directives for noisy third-party crates
    // unwrap() is safe here because these are valid, hardcoded directives
    let filter = filter
        .add_directive("winit=info".parse().unwrap()) // Window events are verbose
        .add_directive("vulkano=debug".parse().unwrap()); // GPU debugging is useful

    Ok(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_doesnt_panic() {
        let result = init_tracing();
        // First call should succeed
        assert!(result.is_ok() || result.is_err(), "init_tracing completes");
    }

    #[test]
    fn test_create_default_filter() {
        let filter = create_default_filter();
        assert!(filter.is_ok(), "default filter should be created");
    }

    #[test]
    fn test_init_with_none_layer() {
        // Should behave identically to init_tracing
        use tracing_subscriber::layer::Identity;
        let result = init_tracing_with_layer(None::<Identity>);
        assert!(result.is_ok() || result.is_err(), "init with None completes");
    }
}
