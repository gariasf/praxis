//! Utility functions for the Praxis engine.
//!
//! This crate provides common utilities used throughout the engine,
//! including tracing and logging capabilities.
//!

mod observability;
pub mod timing;

pub use observability::{init_tracing, init_tracing_with_layer};

// Re-export common utility items for convenience
pub use color_eyre::{
    eyre::{self, Error},
    Report, Result,
};

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
    observability::init_tracing()?;
    info!("Praxis utilities initialized");

    Ok(())
}
