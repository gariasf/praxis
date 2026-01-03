//! Math library for the Praxis engine.
//!
//! This crate provides mathematical utilities used throughout the engine.

use praxis_utils::{info, Result};

/// Initializes the math library.
///
/// This function sets up any necessary global state for the math library.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for math
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., SIMD feature detection)
///
/// # Example
///
/// ```rust,no_run
/// praxis_math::init().expect("Failed to initialize math library");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    info!("Initializing math library");
    Ok(())
}

// Re-export glam so other crates can use it via `praxis_math`.
pub use glam::*;
