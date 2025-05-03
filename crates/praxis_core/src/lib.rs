//! Praxis is the main crate for the Praxis game engine.
//!
//! This crate provides the core functionality and coordinates all the subsystems.
use praxis_utils::Result;

pub fn init() -> Result<()> {
    praxis_utils::init()?;
    praxis_graphics::init();
    Ok(())
}
