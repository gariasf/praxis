use color_eyre::Result;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

/// Initializes the tracing system.
///
/// This function should be called early in the application startup process,
/// typically before any other initialization.
///
/// # Examples
///
/// ```
/// // In your main.rs or lib.rs
/// fn main() -> color_eyre::Result<()> {
///     Ok(())
/// }
/// ```
///
/// # Configuration
///
/// The tracing level can be configured via the `RUST_LOG` environment variable:
///
/// ```bash
/// # Examples:
/// RUST_LOG=debug  # Set global level to debug
/// RUST_LOG=praxis_graphics=trace,praxis_core=debug  # Different levels per module
/// ```
pub fn init_tracing() -> Result<()> {
    color_eyre::install()?;

    // Create a subscriber that formats events as strings
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))?
        .add_directive("winit=info".parse().unwrap())
        .add_directive("vulkano=debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_doesnt_panic() {
        let result = init_tracing();
        assert!(result.is_ok(), "init_tracing should not error");
    }
}
