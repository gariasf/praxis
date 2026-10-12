use std::io::Write;

use praxis::ReferenceScene;
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
        [command, scene, output] if command == "capture" => {
            capture(scene, output, DEFAULT_CAPTURE_SIZE)
        }
        [command, scene, output, size] if command == "capture" => capture(scene, output, size),
        _ => anyhow::bail!(
            "unexpected arguments {arguments:?}: run with no arguments, `capabilities`, \
             or `capture <scene> <png path> [<width>x<height>]`"
        ),
    }
}

/// Prints the capability report of a headless context's adapter.
fn print_capabilities() -> anyhow::Result<()> {
    let context = pollster::block_on(GpuContext::headless())?;
    std::io::stdout().write_all(context.capability_report().as_bytes())?;
    Ok(())
}

/// Renders the Reference scene named `scene` headless at `size`,
/// `<width>x<height>`, writes the last frame to `output` as a PNG, and prints
/// that frame's stats.
fn capture(scene: &str, output: &str, size: &str) -> anyhow::Result<()> {
    let scene = ReferenceScene::from_name(scene).ok_or_else(|| {
        let names: Vec<_> = ReferenceScene::ALL.map(ReferenceScene::name).into();
        anyhow::anyhow!("no scene named {scene:?}; scenes are {names:?}")
    })?;
    let (width, height) = size
        .split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| anyhow::anyhow!("size {size:?} is not <width>x<height>"))?;
    let context = pollster::block_on(GpuContext::headless())?;
    let (image, frame_stats) = praxis::capture(&context, scene, width, height)?;
    image.save_with_format(output, image::ImageFormat::Png)?;
    tracing::info!(
        scene = scene.name(),
        path = output,
        width,
        height,
        "capture written"
    );
    if let Some(last_frame) = frame_stats.last() {
        std::io::stdout().write_all(last_frame.to_string().as_bytes())?;
    }
    Ok(())
}
