use std::io::Write;

use praxis::gpu::GpuContext;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "praxis starting");

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => praxis::run(),
        [command] if command == "capabilities" => print_capabilities(),
        _ => anyhow::bail!(
            "unexpected arguments {arguments:?}: run with no arguments, or `capabilities`"
        ),
    }
}

/// Prints the capability report of a headless context's adapter.
fn print_capabilities() -> anyhow::Result<()> {
    let context = pollster::block_on(GpuContext::headless())?;
    std::io::stdout().write_all(context.capability_report().as_bytes())?;
    Ok(())
}
