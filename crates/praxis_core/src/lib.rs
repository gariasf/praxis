//! Praxis is the main crate for the Praxis game engine.
//!
//! This crate provides the core functionality and coordinates all the subsystems.
pub fn run() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_window::run()?;

    Ok(())
}
