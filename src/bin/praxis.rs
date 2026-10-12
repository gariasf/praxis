use std::io::Write;

use praxis::gpu::GpuContext;

/// The capture size when the command names none: small, and 16:9 like the
/// window.
const DEFAULT_CAPTURE_SIZE: &str = "640x360";

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "praxis starting");

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [] => praxis::run(),
        [command] if command == "capabilities" => print_capabilities(),
        [command, output] if command == "capture" => capture(output, DEFAULT_CAPTURE_SIZE),
        [command, output, size] if command == "capture" => capture(output, size),
        _ => anyhow::bail!(
            "unexpected arguments {arguments:?}: run with no arguments, `capabilities`, \
             or `capture <png path> [<width>x<height>]`"
        ),
    }
}

/// Prints the capability report of a headless context's adapter.
fn print_capabilities() -> anyhow::Result<()> {
    let context = pollster::block_on(GpuContext::headless())?;
    std::io::stdout().write_all(context.capability_report().as_bytes())?;
    Ok(())
}

/// Renders the Scene headless at `size`, `<width>x<height>`, and writes the
/// last frame to `output` as a PNG.
fn capture(output: &str, size: &str) -> anyhow::Result<()> {
    let (width, height) = size
        .split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| anyhow::anyhow!("size {size:?} is not <width>x<height>"))?;
    let context = pollster::block_on(GpuContext::headless())?;
    praxis::capture(&context, width, height)?.save_with_format(output, image::ImageFormat::Png)?;
    tracing::info!(path = output, width, height, "capture written");
    Ok(())
}
