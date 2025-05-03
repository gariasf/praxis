//! A simple example that initializes the praxis_core engine.
use praxis_utils::Result;
fn main() -> Result<()> {
    praxis_core::init()?;

    Ok(())
}
