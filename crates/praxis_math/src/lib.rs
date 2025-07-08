//! Math library for the Praxis engine.
//!
//! This crate provides mathematical utilities used throughout the engine.

/// Initializes the math library.
pub fn init() {
    println!("Math library initialized");
}

// Re-export glam so other crates can use it via `praxis_math`.
pub use glam::*;
