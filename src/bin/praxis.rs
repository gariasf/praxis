use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anyhow::Context;
use praxis::bench::{Budgets, CompareMode, Configuration, Measurements, Verdict};
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
    },
    Bench {
        output: String,
        configuration: Configuration,
        scenes: Vec<String>,
        crowd_form: CrowdForm,
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
        } => capture(&scene, &output, &size, crowd_form),
        Command::Bench {
            output,
            configuration,
            scenes,
            crowd_form,
        } => bench(&output, configuration, &scenes, crowd_form),
        Command::Compare { results, mode } => compare(&results, mode),
        Command::Report { results } => report(&results),
    }
}

fn parse_command(arguments: &[String]) -> anyhow::Result<Command> {
    let (arguments, crowd_form) = take_crowd_form(arguments)?;
    let takes_crowd_form = matches!(
        arguments.first().map(String::as_str),
        Some("bench" | "capture")
    );
    anyhow::ensure!(
        crowd_form.is_none() || takes_crowd_form,
        "only `bench` and `capture` take --crowd-form"
    );
    let crowd_form = crowd_form.unwrap_or_default();
    let bench = |output: &String, configuration, scenes: &[String]| Command::Bench {
        output: output.clone(),
        configuration,
        scenes: scenes.to_vec(),
        crowd_form,
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
        },
        #[cfg(feature = "cook")]
        [command, scene, output, size] if command == "capture" => Command::Capture {
            scene: scene.clone(),
            output: output.clone(),
            size: size.clone(),
            crowd_form,
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
             `cook`, `fetch`, `capture <scene> <png path> [<width>x<height>]`, \
             `bench <output directory> [--ci | --counters | --ci-counters] [<scene>...]`, \
             `compare <bench directory> [--ci]`, or `report <bench directory>`; `capture` and `bench` also take \
             `--crowd-form <proxies | refit | rebuild>`; scenes are {:?}",
            ReferenceScene::ALL.map(ReferenceScene::name)
        ),
    })
}

/// `arguments` without `--crowd-form <form>`, and the form it names.
fn take_crowd_form(arguments: &[String]) -> anyhow::Result<(Vec<String>, Option<CrowdForm>)> {
    let mut arguments = arguments.to_vec();
    let Some(flag) = arguments
        .iter()
        .position(|argument| argument == "--crowd-form")
    else {
        return Ok((arguments, None));
    };
    let names = CrowdForm::ALL.map(CrowdForm::name);
    let form = arguments
        .get(flag + 1)
        .and_then(|name| CrowdForm::from_name(name))
        .ok_or_else(|| anyhow::anyhow!("--crowd-form takes one of {names:?}"))?;
    arguments.drain(flag..flag + 2);
    Ok((arguments, Some(form)))
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
/// written as `crowd-<form>.json`.
fn bench(
    output: &str,
    configuration: Configuration,
    scene_names: &[String],
    crowd_form: CrowdForm,
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
    let context = pollster::block_on(GpuContext::headless())?;
    for scene in scenes {
        let switches = RenderSwitches {
            crowd_form,
            ..RenderSwitches::default()
        };
        let mut result = praxis::bench::run(&context, scene, configuration, switches)?;
        if other_form {
            result.scene = format!("{crowd}-{}", crowd_form.name());
        }
        let path = Path::new(output).join(format!("{}.json", result.scene));
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
/// `<width>x<height>`, its crowd in `crowd_form`, writes the last frame to
/// `output` as a PNG, and prints that frame's stats, its point lights per
/// lit pixel counted. The PNG encoder comes with `cook`.
#[cfg(feature = "cook")]
fn capture(scene: &str, output: &str, size: &str, crowd_form: CrowdForm) -> anyhow::Result<()> {
    let scene = parse_scene(scene)?;
    let (width, height) = size
        .split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| anyhow::anyhow!("size {size:?} is not <width>x<height>"))?;
    let context = pollster::block_on(GpuContext::headless())?;
    let (pixels, frame_stats) = praxis::capture(
        &context,
        scene,
        glam::DVec3::ZERO,
        width,
        height,
        RenderSwitches {
            count_point_lights: true,
            crowd_form,
            ..RenderSwitches::default()
        },
    )?;
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
        } = parse(&["bench", "out", "--ci", "town", "crowd"]).unwrap()
        else {
            panic!("bench parsed as another command");
        };
        assert_eq!(output, "out");
        assert_eq!(configuration.width, Configuration::CI.width);
        assert_eq!(scenes, ["town", "crowd"]);
        assert_eq!(crowd_form, CrowdForm::default());
    }

    #[test]
    fn report_parses_its_results_directory() {
        let Command::Report { results } = parse(&["report", "out"]).unwrap() else {
            panic!("report parsed as another command");
        };
        assert_eq!(results, "out");
    }
}
