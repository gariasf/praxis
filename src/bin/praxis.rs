use std::io::Write;

use praxis::gpu::GpuContext;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "praxis starting");

    match std::env::args().nth(1).as_deref() {
        None => praxis::run(),
        Some("capabilities") => print_capabilities(),
        Some(command) => {
            anyhow::bail!("unknown command `{command}`: run with no command, or `capabilities`")
        }
    }
}

/// Prints the capability report of a headless context's adapter.
fn print_capabilities() -> anyhow::Result<()> {
    let context = pollster::block_on(GpuContext::headless())?;
    std::io::stdout().write_all(context.capability_report().as_bytes())?;
    Ok(())
}
