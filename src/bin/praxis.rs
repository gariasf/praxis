use std::io::Write;
use std::path::Path;

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
        [command, output, scenes @ ..] if command == "bench" => bench(output, scenes),
        _ => anyhow::bail!(
            "unexpected arguments {arguments:?}: run with no arguments, `capabilities`, \
             `capture <scene> <png path> [<width>x<height>]`, \
             or `bench <output directory> [<scene>...]`"
        ),
    }
}

/// Prints the capability report of a headless context's adapter.
fn print_capabilities() -> anyhow::Result<()> {
    let context = pollster::block_on(GpuContext::headless())?;
    std::io::stdout().write_all(context.capability_report().as_bytes())?;
    Ok(())
}

fn parse_scene(name: &str) -> anyhow::Result<ReferenceScene> {
    ReferenceScene::from_name(name).ok_or_else(|| {
        let names: Vec<_> = ReferenceScene::ALL.map(ReferenceScene::name).into();
        anyhow::anyhow!("no scene named {name:?}; scenes are {names:?}")
    })
}

/// Benches the named Reference scenes, or with none every scene whose assets
/// are present, prints one line per scene, and writes `<scene>.json` into
/// `output`.
fn bench(output: &str, scene_names: &[String]) -> anyhow::Result<()> {
    let scenes = if scene_names.is_empty() {
        let helmet_fetched = Path::new(praxis::DEFAULT_MODEL).exists();
        if !helmet_fetched {
            tracing::warn!("helmet skipped: run scripts/fetch-assets.sh to bench it");
        }
        ReferenceScene::ALL
            .into_iter()
            .filter(|&scene| scene != ReferenceScene::Helmet || helmet_fetched)
            .collect()
    } else {
        scene_names
            .iter()
            .map(|name| parse_scene(name))
            .collect::<anyhow::Result<Vec<_>>>()?
    };
    std::fs::create_dir_all(output)?;
    let context = pollster::block_on(GpuContext::headless())?;
    for scene in scenes {
        let result = praxis::bench::run(&context, scene)?;
        let path = Path::new(output).join(format!("{}.json", scene.name()));
        std::fs::write(&path, serde_json::to_string_pretty(&result)? + "\n")?;
        writeln!(std::io::stdout(), "{result}")?;
    }
    Ok(())
}

/// Renders the Reference scene named `scene` headless at `size`,
/// `<width>x<height>`, writes the last frame to `output` as a PNG, and prints
/// that frame's stats.
fn capture(scene: &str, output: &str, size: &str) -> anyhow::Result<()> {
    let scene = parse_scene(scene)?;
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
