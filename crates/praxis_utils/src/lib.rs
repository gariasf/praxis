//! Utility functions for the Praxis engine.
//!
//! This crate provides common utilities used throughout the engine,
//! including tracing and logging capabilities.
//!

mod observability;
pub mod timing;

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
    info!("Initializing Praxis utilities...");
    observability::init_tracing()?;

    Ok(())
}
