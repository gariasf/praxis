use praxis_window::open_window;
use praxis_utils::Result;
fn main() -> Result<()> {
    praxis_core::initialize()?;
    let _ = open_window();
    Ok(())
} 