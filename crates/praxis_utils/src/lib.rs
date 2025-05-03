//! Utility functions for the Praxis engine.
//!
//! This crate provides common utilities used throughout the engine,
//! including tracing and logging capabilities.
//! 
//! pub use tracing::{Level, debug, error, info, instrument, span, trace, warn};

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*, EnvFilter,
};


// Re-export common utility items for convenience
pub use color_eyre::{Report, Result, eyre::eyre};

// Re-export tracing macros for direct use from other crates
pub use tracing::{debug, error, info, instrument, trace, warn};

/// Initializes the utility library.
///
/// This function sets up the tracing system and other utility components.
///
/// # Returns
///
/// Returns `Ok(())` if initialization succeeds, or an error if it fails.
///
/// # Examples
///
/// ```
/// fn main() -> color_eyre::Result<()> {
///     praxis_utils::init()?;
///     // Rest of your application
///     Ok(())
/// }
/// ```
pub fn init() -> Result<()> {
    info!("Initializing Praxis utilities...");
    // Initialize the tracing system
    init_tracing()?;

    Ok(())
}


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
pub fn init_tracing() -> Result<()> {
    // Setup color-eyre for error reporting
    color_eyre::install()?;

    // Create a subscriber that formats events as strings
    let fmt_layer = fmt::layer()
        .with_target(true) // Include the target in the output
        .with_thread_ids(true) // Include thread IDs
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE) // Log span creation/closing
        .pretty(); // Use pretty printer for human readability

    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("debug"))?;

    // Register the subscriber with the tracing system
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // This is a simple test that ensures init() doesn't panic
        let result = init();
        assert!(result.is_ok());
    }
}
