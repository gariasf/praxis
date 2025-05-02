//! Praxis is the main crate for the Praxis game engine.
//!
//! This crate provides the core functionality and coordinates all the subsystems.
use praxis_utils::{Result, info};

pub fn initialize() -> Result<()> {
    praxis_utils::initialize()?;
    info!("Praxis engine initialized");
    Ok(())
}
