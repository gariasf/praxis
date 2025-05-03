//! Tracing utilities for the Praxis engine.
//!
//! This module provides logging and tracing capabilities using the `tracing` crate.

// Re-export the tracing macros so other crates can use them directly
pub use tracing::{Level, debug, error, info, instrument, span, trace, warn};

use color_eyre::Result;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    prelude::*,
};

/// Initializes the tracing system.
///
/// This function should be called early in the application startup process,
/// typically before any other initialization.
///
/// # Examples
///
/// ```
/// // In your main.rs or lib.rs
/// fn main() -> color_eyre::Result<()> {
///     praxis_utils::tracing::init()?;
///     // ... rest of your application
///     Ok(())
/// }
/// ```
///
/// # Configuration
///
/// The tracing level can be configured via the `RUST_LOG` environment variable:
///
/// ```bash
/// # Examples:
/// RUST_LOG=debug  # Set global level to debug
/// RUST_LOG=praxis_graphics=trace,praxis_core=debug  # Different levels per module
/// ```
pub fn init() -> Result<()> {
    // Setup color-eyre for error reporting
    color_eyre::install()?;

    // Create a subscriber that formats events as strings
    let fmt_layer = fmt::layer()
        .with_target(true) // Include the target in the output
        .with_thread_ids(true) // Include thread IDs
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE) // Log span creation/closing
        .pretty(); // Use pretty printer for human readability

    // Filter based on environment variable or defaults to INFO level
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    // Register the subscriber with the tracing system
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    trace!("Tracing initialized successfully");
    Ok(())
}
