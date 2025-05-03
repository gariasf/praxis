//! Utility functions for the Praxis engine.
//!
//! This crate provides common utilities used throughout the engine,
//! including tracing and logging capabilities.

pub mod tracing;

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
    // Initialize the tracing system
    tracing::init()?;

    // Log that initialization is complete
    tracing::info!("Praxis utilities initialized.");

    Ok(())
}

// Re-export common utility items for convenience
pub use color_eyre::{Report, Result, eyre::eyre};

// Re-export tracing macros for direct use from other crates
pub use tracing::{debug, error, info, instrument, trace, warn};

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
