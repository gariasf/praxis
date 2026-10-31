use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anyhow::Context;
use praxis::bench::{BenchStart, Budgets, CompareMode, Configuration, Measurements, Verdict};
use praxis::gpu::GpuContext;
use praxis::{CrowdForm, ReferenceScene, RenderSwitches};

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

/// What one run of the binary does, parsed from its arguments.
#[derive(Debug)]
enum Command {
    Window(ReferenceScene),
    Capabilities,
    Preflight,
    #[cfg(feature = "cook")]
    Cook,
    #[cfg(feature = "cook")]
    Fetch,
    #[cfg(feature = "cook")]
    Capture {
        scene: String,
        output: String,
        size: String,
        crowd_form: CrowdForm,
        frame: Option<u32>,
    },
    #[cfg(feature = "cook")]
    ExportScene {
        scene: String,
        directory: String,
        frame: Option<u32>,
    },
    #[cfg(feature = "cook")]
    Difference {
        expected: String,
        actual: String,
        output: String,
    },
    Bench {
        output: String,
        configuration: Configuration,
        scenes: Vec<String>,
        crowd_form: CrowdForm,
        publish: bool,
    },
    Publish {
        results: String,
    },
    Compare {
        results: String,
        mode: CompareMode,
    },
    Report {
        results: String,
    },
}

fn run_command(arguments: &[String]) -> anyhow::Result<()> {
    match parse_command(arguments)? {
        Command::Window(scene) => praxis::run(scene),
        Command::Capabilities => print_capabilities(),
        Command::Preflight => preflight(),
        #[cfg(feature = "cook")]
        Command::Cook => cook(),
        #[cfg(feature = "cook")]
        Command::Fetch => fetch(),
        #[cfg(feature = "cook")]
        Command::Capture {
            scene,
            output,
            size,
            crowd_form,
            frame,
        } => capture(&scene, &output, &size, crowd_form, frame),
        #[cfg(feature = "cook")]
        Command::ExportScene {
            scene,
            directory,
            frame,
        } => export_scene(&scene, &directory, frame),
        #[cfg(feature = "cook")]
        Command::Difference {
            expected,
            actual,
            output,
        } => difference(&expected, &actual, &output),
        Command::Bench {
            output,
            configuration,
            scenes,
            crowd_form,
            publish,
        } => bench(&output, configuration, &scenes, crowd_form, publish),
        Command::Publish { results } => publish(&results),
        Command::Compare { results, mode } => compare(&results, mode),
        Command::Report { results } => report(&results),
    }
}

fn parse_command(arguments: &[String]) -> anyhow::Result<Command> {
    let mut arguments = arguments.to_vec();
    let crowd_form = take_option(&mut arguments, "--crowd-form")?
        .map(|name| {
            CrowdForm::from_name(&name).ok_or_else(|| {
                let names = CrowdForm::ALL.map(CrowdForm::name);
                anyhow::anyhow!("--crowd-form takes one of {names:?}")
            })
        })
        .transpose()?;
    let frame = take_option(&mut arguments, "--frame")?
        .map(|frame| {
            frame
                .parse::<u32>()
                .map_err(|_| anyhow::anyhow!("--frame takes a frame number, not {frame:?}"))
        })
        .transpose()?;
    let skip_aliasing = take_flag(&mut arguments, "--skip-aliasing");
    let no_publish = take_flag(&mut arguments, "--no-publish");
    let command = arguments.first().map(String::as_str);
    anyhow::ensure!(
        !skip_aliasing || command == Some("bench"),
        "only `bench` takes --skip-aliasing"
    );
    anyhow::ensure!(
        !no_publish || command == Some("bench"),
        "only `bench` takes --no-publish"
    );
    anyhow::ensure!(
        crowd_form.is_none() || matches!(command, Some("bench" | "capture")),
        "only `bench` and `capture` take --crowd-form"
    );
    anyhow::ensure!(
        frame.is_none() || matches!(command, Some("capture" | "export-scene")),
        "only `capture` and `export-scene` take --frame"
    );
    let crowd_form = crowd_form.unwrap_or_default();
    let bench = |output: &String, configuration: Configuration, scenes: &[String]| Command::Bench {
        output: output.clone(),
        configuration: Configuration {
            aliasing: configuration.aliasing && !skip_aliasing,
            ..configuration
        },
        scenes: scenes.to_vec(),
        crowd_form,
        publish: !no_publish,
    };
    Ok(match arguments.as_slice() {
        [] => Command::Window(ReferenceScene::Helmet),
        [command] if command == "capabilities" => Command::Capabilities,
        [command] if command == "preflight" => Command::Preflight,
        #[cfg(feature = "cook")]
        [command] if command == "cook" => Command::Cook,
        #[cfg(feature = "cook")]
        [command] if command == "fetch" => Command::Fetch,
        #[cfg(feature = "cook")]
        [command, scene, output] if command == "capture" => Command::Capture {
            scene: scene.clone(),
            output: output.clone(),
            size: DEFAULT_CAPTURE_SIZE.into(),
            crowd_form,
            frame,
        },
        #[cfg(feature = "cook")]
        [command, scene, output, size] if command == "capture" => Command::Capture {
            scene: scene.clone(),
            output: output.clone(),
            size: size.clone(),
            crowd_form,
            frame,
        },
        #[cfg(feature = "cook")]
        [command, scene, directory] if command == "export-scene" => Command::ExportScene {
            scene: scene.clone(),
            directory: directory.clone(),
            frame,
        },
        #[cfg(feature = "cook")]
        [command, expected, actual, output] if command == "difference" => Command::Difference {
            expected: expected.clone(),
            actual: actual.clone(),
            output: output.clone(),
        },
        [command, output, flag, scenes @ ..] if command == "bench" && flag == "--ci" => {
            bench(output, Configuration::CI, scenes)
        }
        [command, output, flag, scenes @ ..] if command == "bench" && flag == "--counters" => {
            bench(output, Configuration::COUNTERS, scenes)
        }
        [command, output, flag, scenes @ ..] if command == "bench" && flag == "--ci-counters" => {
            bench(output, Configuration::CI_COUNTERS, scenes)
        }
        [command, output, scenes @ ..] if command == "bench" => {
            bench(output, Configuration::REFERENCE, scenes)
        }
        [command, results] if command == "publish" => Command::Publish {
            results: results.clone(),
        },
        [command, results] if command == "compare" => Command::Compare {
            results: results.clone(),
            mode: CompareMode::Local,
        },
        [command, results, flag] if command == "compare" && flag == "--ci" => Command::Compare {
            results: results.clone(),
            mode: CompareMode::Ci,
        },
        [command, results] if command == "report" => Command::Report {
            results: results.clone(),
        },
        [scene] if ReferenceScene::from_name(scene).is_some() => {
            Command::Window(parse_scene(scene)?)
        }
        _ => anyhow::bail!(
            "unexpected arguments {arguments:?}: run `praxis [<scene>]`, `capabilities`, \
             `cook`, `fetch`, `capture <scene> <png path> [<width>x<height>] [--frame <n>]`, \
             `export-scene <scene> <directory> [--frame <n>]`, \
             `difference <expected png> <actual png> <difference png>`, \
             `bench <output directory> [--ci | --counters | --ci-counters] [--skip-aliasing] [--no-publish] [<scene>...]`, \
             `publish <bench directory>`, `compare <bench directory> [--ci]`, or `report <bench directory>`; `capture` and `bench` also take \
             `--crowd-form <proxies | refit | rebuild>`; scenes are {:?}",
            ReferenceScene::ALL.map(ReferenceScene::name)
        ),
    })
}

/// Removes `flag` and the value after it from `arguments`, and returns the
/// value.
fn take_option(arguments: &mut Vec<String>, flag: &str) -> anyhow::Result<Option<String>> {
    let Some(position) = arguments.iter().position(|argument| argument == flag) else {
        return Ok(None);
    };
    anyhow::ensure!(position + 1 < arguments.len(), "{flag} takes a value");
    let value = arguments.remove(position + 1);
    arguments.remove(position);
    Ok(Some(value))
}

/// Whether `arguments` hold `flag`, which it removes.
fn take_flag(arguments: &mut Vec<String>, flag: &str) -> bool {
    let length = arguments.len();
    arguments.retain(|argument| argument != flag);
    arguments.len() != length
}

/// Prints the capability report of the adapter a headless context would use.
fn print_capabilities() -> anyhow::Result<()> {
    let report = pollster::block_on(praxis::gpu::capability_report())?;
    std::io::stdout().write_all(report.as_bytes())?;
    Ok(())
}

/// Prints one line per preflight check, and fails when any blocks
/// `scripts/verify.sh`.
fn preflight() -> anyhow::Result<()> {
    let checks = praxis::preflight::run();
    let mut stdout = std::io::stdout();
    for check in &checks {
        writeln!(stdout, "{check}")?;
    }
    let blocking = checks
        .iter()
        .filter(|check| check.verdict == praxis::preflight::Verdict::Fail)
        .count();
    anyhow::ensure!(
        blocking == 0,
        "{blocking} checks block scripts/verify.sh; run the fixes they name"
    );
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

/// Fetches the manifest's fetched files into `assets/`, and prints one line
/// per file: `ok` for one kept, `fetched` for one downloaded.
#[cfg(feature = "cook")]
fn fetch() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    for (path, fetch) in praxis::cook::fetch(Path::new(praxis::cook::ASSETS_DIRECTORY))? {
        let verdict = match fetch {
            praxis::cook::Fetch::Kept => "ok",
            praxis::cook::Fetch::Fetched => "fetched",
        };
        writeln!(stdout, "{verdict:<8} {path}")?;
    }
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
/// `output`. A crowd form other than the default benches `crowd` alone,
/// written as `crowd-<form>.json`. With `publish`, the results then join
/// the Bench history; a publish that fails warns and fails no bench.
fn bench(
    output: &str,
    configuration: Configuration,
    scene_names: &[String],
    crowd_form: CrowdForm,
    publish: bool,
) -> anyhow::Result<()> {
    let crowd = ReferenceScene::Crowd.name();
    let other_form = crowd_form != CrowdForm::default();
    anyhow::ensure!(
        !other_form || scene_names.iter().all(|name| name == crowd),
        "--crowd-form {} benches {crowd} alone",
        crowd_form.name()
    );
    let scenes = if other_form {
        vec![ReferenceScene::Crowd]
    } else if scene_names.is_empty() {
        let helmet_fetched = Path::new(praxis::DEFAULT_MODEL).exists();
        if !helmet_fetched {
            tracing::warn!("helmet skipped: run `praxis fetch` to bench it");
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
    let start = BenchStart::read();
    let context = pollster::block_on(GpuContext::headless())?;
    let mut results = Vec::new();
    for scene in scenes {
        let switches = RenderSwitches {
            crowd_form,
            ..RenderSwitches::default()
        };
        let mut result = praxis::bench::run(&context, scene, configuration, switches, &start)?;
        if other_form {
            result.scene = format!("{crowd}-{}", crowd_form.name());
        }
        let path = Path::new(output).join(format!("{}.json", result.scene));
        std::fs::write(&path, serde_json::to_string_pretty(&result)? + "\n")?;
        writeln!(std::io::stdout(), "{result}")?;
        results.push(result);
    }
    if publish {
        match praxis::bench::publish(Path::new("."), &results) {
            Ok(publication) => writeln!(std::io::stdout(), "{publication}")?,
            Err(error) => tracing::warn!(
                error = format!("{error:#}"),
                "bench results not published; `praxis publish {output}` retries"
            ),
        }
    }
    Ok(())
}

/// Publishes every `<scene>.json` in `results` as Bench records on the
/// `bench-history` branch, and prints what it did.
fn publish(results: &str) -> anyhow::Result<()> {
    let measurements = result_paths(results)?
        .iter()
        .map(|path| read_json(path))
        .collect::<anyhow::Result<Vec<Measurements>>>()?;
    let publication = praxis::bench::publish(Path::new("."), &measurements)?;
    writeln!(std::io::stdout(), "{publication}")?;
    Ok(())
}

/// Compares every `<scene>.json` in `results` with its adapter's Baseline and the
/// Budgets, prints one line per scene and finding, and fails on any failed
/// metric.
fn compare(results: &str, mode: CompareMode) -> anyhow::Result<()> {
    let budgets: Budgets = read_json(Path::new(praxis::bench::BUDGETS_PATH))?;
    let mut failed = 0;
    let mut stdout = std::io::stdout();
    for path in result_paths(results)? {
        let current: Measurements = read_json(&path)?;
        let baseline_path = baseline_path(&current, &path);
        let baseline_directory = baseline_path
            .parent()
            .expect("a Baseline path has a directory");
        // Both of CI's configurations render at its size.
        let ci_run =
            (current.width, current.height) == (Configuration::CI.width, Configuration::CI.height);
        let record = if ci_run {
            format!(
                "copy {}.json from CI's bench-results artifact",
                current.scene
            )
        } else {
            let scene = match current.scene.strip_prefix("crowd-") {
                Some(form) => format!("--crowd-form {form}"),
                None => current.scene.clone(),
            };
            format!(
                "record one with `praxis bench {} {scene}`",
                baseline_directory.display(),
            )
        };
        if !baseline_path.exists() {
            failed += 1;
            writeln!(
                stdout,
                "{} FAIL no Baseline at {}; {record}",
                current.scene,
                baseline_path.display(),
            )?;
            continue;
        }
        let baseline = std::fs::read(&baseline_path)
            .map_err(anyhow::Error::from)
            .and_then(|json| Measurements::from_baseline_json(&json));
        let baseline = match baseline {
            Ok(baseline) => baseline,
            Err(error) => {
                failed += 1;
                writeln!(
                    stdout,
                    "{} FAIL Baseline {} {error:#}; {record}",
                    current.scene,
                    baseline_path.display(),
                )?;
                continue;
            }
        };
        let findings = match praxis::bench::compare(&current, &baseline, &budgets, mode) {
            Ok(findings) => findings,
            Err(error) => {
                failed += 1;
                writeln!(stdout, "{} FAIL {error:#}", current.scene)?;
                continue;
            }
        };
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

/// Prints the markdown tables of every `<scene>.json` in `results` against
/// its adapter's Baseline and the Budgets, for PRs and `docs/performance.md`
/// to quote. A scene without a Baseline its tables can compare with gets a
/// line saying why, then tables of its own figures.
fn report(results: &str) -> anyhow::Result<()> {
    let budgets: Budgets = read_json(Path::new(praxis::bench::BUDGETS_PATH))?;
    let mut stdout = std::io::stdout();
    for path in result_paths(results)? {
        let current: Measurements = read_json(&path)?;
        let baseline_path = baseline_path(&current, &path);
        let tables = std::fs::read(&baseline_path)
            .map_err(anyhow::Error::from)
            .and_then(|json| Measurements::from_baseline_json(&json))
            .and_then(|baseline| praxis::bench::report(&current, Some(&baseline), &budgets));
        let tables = match tables {
            Ok(tables) => tables,
            Err(error) => {
                writeln!(
                    stdout,
                    "{} compares with no Baseline: {} {error:#}\n",
                    current.scene,
                    baseline_path.display()
                )?;
                praxis::bench::report(&current, None, &budgets)?
            }
        };
        writeln!(stdout, "{tables}")?;
    }
    Ok(())
}

/// The Baseline of `current`, read from `path`: its adapter and backend's
/// file of the same name.
fn baseline_path(current: &Measurements, path: &Path) -> std::path::PathBuf {
    praxis::bench::baseline_directory(&current.adapter, &current.backend)
        .join(path.file_name().expect("a read_dir entry has a file name"))
}

/// Every `<scene>.json` in `results`, by name; none is an error.
fn result_paths(results: &str) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut paths: Vec<_> = std::fs::read_dir(results)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    anyhow::ensure!(!paths.is_empty(), "no bench results in {results}");
    Ok(paths)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

/// Renders the Reference scene named `scene` headless at `size`,
/// `<width>x<height>`, its crowd in `crowd_form`, and writes `frame` of its
/// camera path to `output` as a PNG. With no `frame`, it writes the last
/// frame `praxis::capture` renders and prints that frame's stats, its point
/// lights per lit pixel counted. The PNG encoder comes with `cook`.
#[cfg(feature = "cook")]
fn capture(
    scene: &str,
    output: &str,
    size: &str,
    crowd_form: CrowdForm,
    frame: Option<u32>,
) -> anyhow::Result<()> {
    let scene = parse_scene(scene)?;
    let (width, height) = size
        .split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| anyhow::anyhow!("size {size:?} is not <width>x<height>"))?;
    let context = pollster::block_on(GpuContext::headless())?;
    let switches = RenderSwitches {
        count_point_lights: frame.is_none(),
        crowd_form,
        ..RenderSwitches::default()
    };
    let (pixels, last_frame) = match frame {
        Some(frame) => {
            let mut pixels = praxis::capture_frames(
                &context,
                scene,
                glam::DVec3::ZERO,
                width,
                height,
                &[frame],
                switches,
            )?;
            (pixels.remove(0), None)
        }
        None => {
            let (pixels, mut frame_stats) =
                praxis::capture(&context, scene, glam::DVec3::ZERO, width, height, switches)?;
            (pixels, frame_stats.pop())
        }
    };
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
        frame,
        "capture written"
    );
    if let Some(last_frame) = last_frame {
        std::io::stdout().write_all(last_frame.to_string().as_bytes())?;
    }
    Ok(())
}

/// Writes what prepare reads of the Reference scene named `scene` at
/// `frame` of its camera path, or at the frame `capture` writes, into
/// `directory`, for a reference renderer.
#[cfg(feature = "cook")]
fn export_scene(scene: &str, directory: &str, frame: Option<u32>) -> anyhow::Result<()> {
    praxis::cook::export_scene(
        parse_scene(scene)?,
        frame,
        Path::new(praxis::cook::ASSETS_DIRECTORY),
        Path::new(directory),
    )
}

/// Writes the difference image of two PNGs of equal size to `output`, as the
/// Goldens test does on a mismatch, and prints how many pixels differ.
#[cfg(feature = "cook")]
fn difference(expected: &str, actual: &str, output: &str) -> anyhow::Result<()> {
    let open = |path: &str| {
        image::open(path)
            .with_context(|| format!("reading {path}"))
            .map(image::DynamicImage::into_rgba8)
    };
    let (expected_image, actual_image) = (open(expected)?, open(actual)?);
    anyhow::ensure!(
        expected_image.dimensions() == actual_image.dimensions(),
        "{expected} is {:?} and {actual} is {:?}",
        expected_image.dimensions(),
        actual_image.dimensions()
    );
    let (width, height) = actual_image.dimensions();
    image::save_buffer_with_format(
        output,
        &praxis::difference_image(expected_image.as_raw(), actual_image.as_raw()),
        width,
        height,
        image::ExtendedColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
    let difference = praxis::compare_images(expected_image.as_raw(), actual_image.as_raw());
    writeln!(
        std::io::stdout(),
        "{} pixels differ by more than {} (max channel delta {})",
        difference.differing_pixels,
        praxis::GOLDEN_TOLERANCE,
        difference.max_channel_delta
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> anyhow::Result<Command> {
        let arguments: Vec<String> = arguments.iter().map(|&argument| argument.into()).collect();
        parse_command(&arguments)
    }

    #[test]
    fn no_arguments_open_the_window_on_the_helmet() {
        assert!(matches!(
            parse(&[]).unwrap(),
            Command::Window(ReferenceScene::Helmet)
        ));
    }

    #[test]
    fn a_scene_name_opens_the_window_on_that_scene() {
        assert!(matches!(
            parse(&["town"]).unwrap(),
            Command::Window(ReferenceScene::Town)
        ));
    }

    #[test]
    fn an_unknown_scene_name_is_refused_with_the_scene_names() {
        let error = parse(&["no-such-scene"]).unwrap_err().to_string();
        assert!(error.contains("no-such-scene"), "{error}");
        assert!(error.contains("\"town\""), "{error}");
        assert!(error.contains("\"helmet\""), "{error}");
    }

    #[test]
    fn a_command_missing_its_arguments_is_refused_with_the_usage() {
        let error = parse(&["bench"]).unwrap_err().to_string();
        assert!(error.contains("praxis [<scene>]"), "{error}");
    }

    #[test]
    fn bench_still_parses_its_configuration_and_scenes() {
        let Command::Bench {
            output,
            configuration,
            scenes,
            crowd_form,
            publish,
        } = parse(&["bench", "out", "--ci", "town", "crowd"]).unwrap()
        else {
            panic!("bench parsed as another command");
        };
        assert_eq!(output, "out");
        assert_eq!(configuration.width, Configuration::CI.width);
        assert_eq!(scenes, ["town", "crowd"]);
        assert_eq!(crowd_form, CrowdForm::default());
        assert!(publish);
    }

    #[test]
    fn bench_publishes_unless_told_not_to_and_publish_parses_its_results() {
        let publish = |arguments: &[&str]| match parse(arguments).unwrap() {
            Command::Bench { publish, .. } => publish,
            _ => panic!("bench parsed as another command"),
        };
        assert!(!publish(&[
            "bench",
            "out",
            "--counters",
            "--no-publish",
            "town"
        ]));
        let error = parse(&["compare", "out", "--no-publish"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("only `bench` takes --no-publish"), "{error}");
        let Command::Publish { results } = parse(&["publish", "out"]).unwrap() else {
            panic!("publish parsed as another command");
        };
        assert_eq!(results, "out");
    }

    #[test]
    fn bench_skips_aliasing_on_request_only() {
        let aliasing = |arguments: &[&str]| match parse(arguments).unwrap() {
            Command::Bench { configuration, .. } => configuration.aliasing,
            _ => panic!("bench parsed as another command"),
        };
        assert!(aliasing(&["bench", "out", "--counters", "town"]));
        assert!(!aliasing(&[
            "bench",
            "out",
            "--counters",
            "--skip-aliasing",
            "town"
        ]));
        assert!(!aliasing(&["bench", "--skip-aliasing", "out"]));
        let error = parse(&["capture", "town", "out.png", "--skip-aliasing"])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("only `bench` takes --skip-aliasing"),
            "{error}"
        );
    }

    #[cfg(feature = "cook")]
    #[test]
    fn capture_parses_a_frame_wherever_it_stands() {
        for arguments in [
            ["capture", "town", "out.png", "64x36", "--frame", "300"],
            ["capture", "--frame", "300", "town", "out.png", "64x36"],
        ] {
            let Command::Capture { scene, frame, .. } = parse(&arguments).unwrap() else {
                panic!("capture parsed as another command");
            };
            assert_eq!(scene, "town");
            assert_eq!(frame, Some(300));
        }
    }

    #[cfg(feature = "cook")]
    #[test]
    fn export_scene_parses_its_scene_directory_and_frame() {
        for (arguments, expected_frame) in [
            (&["export-scene", "town", "out"][..], None),
            (
                &["export-scene", "--frame", "300", "town", "out"][..],
                Some(300),
            ),
        ] {
            let Command::ExportScene {
                scene,
                directory,
                frame,
            } = parse(arguments).unwrap()
            else {
                panic!("export-scene parsed as another command");
            };
            assert_eq!((scene.as_str(), directory.as_str()), ("town", "out"));
            assert_eq!(frame, expected_frame);
        }
    }

    #[test]
    fn a_frame_that_is_no_number_or_not_on_capture_is_refused() {
        let error = parse(&["capture", "town", "out.png", "--frame", "last"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("\"last\""), "{error}");
        let error = parse(&["capture", "town", "out.png", "--frame"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("--frame takes a value"), "{error}");
        let error = parse(&["bench", "out", "--frame", "3"])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("only `capture` and `export-scene` take --frame"),
            "{error}"
        );
    }

    #[test]
    fn report_parses_its_results_directory() {
        let Command::Report { results } = parse(&["report", "out"]).unwrap() else {
            panic!("report parsed as another command");
        };
        assert_eq!(results, "out");
    }
}
