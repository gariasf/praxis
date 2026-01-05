use color_eyre::Result;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
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
///
/// # Panics
///
/// Panics if the log directives "winit=info" or "vulkano=debug" cannot be parsed.
/// This should never happen under normal circumstances as these are valid directives.
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

/// Initializes the tracing system with an optional custom layer.
///
/// This allows adding custom subscribers, such as a console panel layer for capturing
/// logs in the editor.
///
/// # Type Parameters
///
/// * `L` - A layer type that implements the `Layer` trait
///
/// # Examples
///
/// ```ignore
/// use praxis_editor::ConsoleLayer;
///
/// let console_layer = ConsoleLayer::new(log_buffer);
/// init_tracing_with_layer(Some(console_layer))?;
/// ```
///
/// # Panics
///
/// Panics if the log directives "winit=info" or "vulkano=debug" cannot be parsed.
/// This should never happen under normal circumstances as these are valid directives.
pub fn init_tracing_with_layer<L>(custom_layer: Option<L>) -> Result<()>
where
    L: Layer<tracing_subscriber::layer::Layered<EnvFilter, tracing_subscriber::Registry>>
        + Send
        + Sync
        + 'static,
{
    color_eyre::install()?;

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))?
        .add_directive("winit=info".parse().unwrap())
        .add_directive("vulkano=debug".parse().unwrap());

    if let Some(layer) = custom_layer {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(layer)
            .init();
    } else {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .pretty();

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }

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
