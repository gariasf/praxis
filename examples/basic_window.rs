use praxis_utils::Result;
use praxis_window::open_window;
fn main() -> Result<()> {
    praxis_core::initialize()?;
    let _ = open_window();
    Ok(())
}
