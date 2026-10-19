use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anyhow::Context;
use praxis::ReferenceScene;
use praxis::bench::{Budgets, CompareMode, Configuration, Measurements, Verdict};
use praxis::gpu::GpuContext;

/// The capture size when the command names none: small, and 16:9 like the
/// window.
#[cfg(feature = "cook")]
const DEFAULT_CAPTURE_SIZE: &str = "640x360";

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "praxis starting");

    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run_command(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = %format_args!("{error:#}"), "praxis failed");
            ExitCode::FAILURE
        }
    }
}

fn run_command(arguments: &[String]) -> anyhow::Result<()> {
    match arguments {
        [] => praxis::run(),
        [command] if command == "capabilities" => print_capabilities(),
        #[cfg(feature = "cook")]
        [command] if command == "cook" => cook(),
        #[cfg(feature = "cook")]
        [command, scene, output] if command == "capture" => {
            capture(scene, output, DEFAULT_CAPTURE_SIZE)
        }
        #[cfg(feature = "cook")]
        [command, scene, output, size] if command == "capture" => capture(scene, output, size),
        [command, output, flag, scenes @ ..] if command == "bench" && flag == "--ci" => {
            bench(output, Configuration::CI, scenes)
        }
        [command, output, scenes @ ..] if command == "bench" => {
            bench(output, Configuration::REFERENCE, scenes)
        }
        [command, results] if command == "compare" => compare(results, CompareMode::Local),
        [command, results, flag] if command == "compare" && flag == "--ci" => {
            compare(results, CompareMode::Ci)
        }
        _ => anyhow::bail!(
            "unexpected arguments {arguments:?}: run with no arguments, `capabilities`, \
             `cook`, `capture <scene> <png path> [<width>x<height>]`, \
             `bench <output directory> [--ci] [<scene>...]`, or `compare <bench directory> [--ci]`"
        ),
    }
}

/// Prints the capability report of the adapter a headless context would use.
fn print_capabilities() -> anyhow::Result<()> {
    let report = pollster::block_on(praxis::gpu::capability_report())?;
    std::io::stdout().write_all(report.as_bytes())?;
    Ok(())
}

/// Cooks `assets/` and every procedural Reference scene into `cooked/`.
#[cfg(feature = "cook")]
fn cook() -> anyhow::Result<()> {
    let report = praxis::cook::cook(
        Path::new(praxis::cook::ASSETS_DIRECTORY),
        &ReferenceScene::ALL,
        &praxis::cooked::CookedCache::new(praxis::cooked::COOKED_CACHE_DIRECTORY),
        &praxis::cook::CookSettings::default(),
    )?;
    writeln!(
        std::io::stdout(),
        "cooked {}, unchanged {}",
        report.cooked.len(),
        report.unchanged.len()
    )?;
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
fn bench(output: &str, configuration: Configuration, scene_names: &[String]) -> anyhow::Result<()> {
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
        let result = praxis::bench::run(&context, scene, configuration)?;
        let path = Path::new(output).join(format!("{}.json", scene.name()));
        std::fs::write(&path, serde_json::to_string_pretty(&result)? + "\n")?;
        writeln!(std::io::stdout(), "{result}")?;
    }
    Ok(())
}

/// Compares every `<scene>.json` in `results` with its adapter's Baseline and the
/// Budgets, prints one line per scene and finding, and fails on any failed
/// metric.
fn compare(results: &str, mode: CompareMode) -> anyhow::Result<()> {
    let budgets: Budgets = read_json(Path::new(praxis::bench::BUDGETS_PATH))?;
    let mut paths: Vec<_> = std::fs::read_dir(results)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    anyhow::ensure!(!paths.is_empty(), "no bench results in {results}");
    let mut failed = 0;
    let mut stdout = std::io::stdout();
    for path in paths {
        let current: Measurements = read_json(&path)?;
        let baseline_directory = praxis::bench::baseline_directory(&current.adapter);
        let baseline_path =
            baseline_directory.join(path.file_name().expect("a read_dir entry has a file name"));
        if !baseline_path.exists() {
            failed += 1;
            let ci_run = (current.width, current.height, current.measured_frames)
                == (
                    Configuration::CI.width,
                    Configuration::CI.height,
                    Configuration::CI.measured_frames,
                );
            let record = if ci_run {
                format!(
                    "copy {}.json from CI's bench-results artifact",
                    current.scene
                )
            } else {
                format!(
                    "record one with `praxis bench {} {}`",
                    baseline_directory.display(),
                    current.scene
                )
            };
            writeln!(
                stdout,
                "{} FAIL no Baseline at {}; {record}",
                current.scene,
                baseline_path.display(),
            )?;
            continue;
        }
        let baseline: Measurements = read_json(&baseline_path)?;
        let findings = praxis::bench::compare(&current, &baseline, &budgets, mode)?;
        if findings.is_empty() {
            writeln!(stdout, "{} ok", current.scene)?;
        }
        for finding in findings {
            failed += usize::from(finding.verdict == Verdict::Fail);
            writeln!(stdout, "{} {finding}", current.scene)?;
        }
    }
    anyhow::ensure!(
        failed == 0,
        "{failed} metrics failed against their Baseline or Budget"
    );
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

/// Renders the Reference scene named `scene` headless at `size`,
/// `<width>x<height>`, writes the last frame to `output` as a PNG, and prints
/// that frame's stats. The PNG encoder comes with `cook`.
#[cfg(feature = "cook")]
fn capture(scene: &str, output: &str, size: &str) -> anyhow::Result<()> {
    let scene = parse_scene(scene)?;
    let (width, height) = size
        .split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| anyhow::anyhow!("size {size:?} is not <width>x<height>"))?;
    let context = pollster::block_on(GpuContext::headless())?;
    let (pixels, frame_stats) = praxis::capture(&context, scene, glam::DVec3::ZERO, width, height)?;
    image::save_buffer_with_format(
        output,
        &pixels,
        width,
        height,
        image::ExtendedColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
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
