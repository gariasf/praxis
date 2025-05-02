//! A simple example that initializes the praxis_core engine.
use praxis_utils::{Result, info};
fn main() -> Result<()> {
    info!("Starting praxis_core example");

    praxis_core::initialize()?;

    Ok(())
}
