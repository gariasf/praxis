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
        .with_writer(std::io::stderr)
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
    Scenes,
    #[cfg(feature = "cook")]
    Cook {
        /// The one Reference scene to cook whole; every scene and source
        /// when absent.
        scene: Option<String>,
    },
    #[cfg(feature = "cook")]
    Fetch,
    #[cfg(feature = "cook")]
    Capture {
        scene: String,
        output: String,
        size: String,
        crowd_form: CrowdForm,
        frames: Vec<u32>,
        /// The scene's Golden runs, in place of `frames`.
        golden_frames: bool,
        /// The LOD every Instance draws, in place of the one selected.
        forced_lod: Option<u32>,
    },
    #[cfg(feature = "cook")]
    ShadowMaskDifference {
        scene: String,
        output: String,
        size: String,
        frames: Vec<u32>,
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
        /// The LOD every Instance draws, in place of the one selected.
        forced_lod: Option<u32>,
        publish: bool,
        /// The Experiment the results publish to, off every Series.
        experiment: Option<String>,
    },
    Publish {
        results: String,
        /// The wall time of the release build that made this binary, which
        /// then joins the results as a release build.
        release_build_seconds: Option<u64>,
        /// The Experiment the results publish to, off every Series.
        experiment: Option<String>,
    },
    ImportBaselines,
    UnrecordedScenes,
    Suspects,
    Bisect {
        night: String,
    },
    PublishNight {
        night: String,
    },
    Regressions,
    RerecordCounters {
        results: String,
    },
    Compare {
        results: String,
        mode: CompareMode,
    },
    Report {
        results: String,
    },
    Traverse,
}

fn run_command(arguments: &[String]) -> anyhow::Result<()> {
    match parse_command(arguments)? {
        Command::Window(scene) => praxis::run(scene),
        Command::Capabilities => print_capabilities(),
        Command::Preflight => preflight(),
        Command::Scenes => print_scenes(),
        #[cfg(feature = "cook")]
        Command::Cook { scene } => cook(scene.as_deref()),
        #[cfg(feature = "cook")]
        Command::Fetch => fetch(),
        #[cfg(feature = "cook")]
        Command::Capture {
            scene,
            output,
            size,
            crowd_form,
            frames,
            golden_frames,
            forced_lod,
        } => capture(
            &scene,
            &output,
            &size,
            crowd_form,
            &frames,
            golden_frames,
            forced_lod,
        ),
        #[cfg(feature = "cook")]
        Command::ShadowMaskDifference {
            scene,
            output,
            size,
            frames,
        } => shadow_mask_difference(&scene, &output, &size, &frames),
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
            forced_lod,
            publish,
            experiment,
        } => bench(
            &output,
            configuration,
            &scenes,
            crowd_form,
            forced_lod,
            publish,
            experiment.as_deref(),
        ),
        Command::Publish {
            results,
            release_build_seconds,
            experiment,
        } => publish(&results, release_build_seconds, experiment.as_deref()),
        Command::ImportBaselines => import_baselines(),
        Command::UnrecordedScenes => unrecorded_scenes(),
        Command::Suspects => suspects(),
        Command::Bisect { night } => bisect(&night),
        Command::PublishNight { night } => publish_night(&night),
        Command::Regressions => regressions(),
        Command::RerecordCounters { results } => rerecord_counters(&results),
        Command::Compare { results, mode } => compare(&results, mode),
        Command::Report { results } => report(&results),
        Command::Traverse => traverse(),
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
    let frames = take_option(&mut arguments, "--frame")?
        .map(|frames| {
            frames
                .split(',')
                .map(|frame| {
                    frame
                        .parse::<u32>()
                        .map_err(|_| anyhow::anyhow!("--frame takes frame numbers, not {frames:?}"))
                })
                .collect::<anyhow::Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    let forced_lod = take_option(&mut arguments, "--lod")?
        .map(|lod| {
            lod.parse::<u32>()
                .map_err(|_| anyhow::anyhow!("--lod takes a LOD's number, not {lod:?}"))
        })
        .transpose()?;
    let release_build_seconds = take_option(&mut arguments, "--release-build-seconds")?
        .map(|seconds| {
            seconds.parse::<u64>().map_err(|_| {
                anyhow::anyhow!("--release-build-seconds takes whole seconds, not {seconds:?}")
            })
        })
        .transpose()?;
    let experiment = take_option(&mut arguments, "--experiment")?;
    if let Some(name) = &experiment {
        praxis::bench::check_experiment_name(name)?;
    }
    let skip_aliasing = take_flag(&mut arguments, "--skip-aliasing");
    let no_publish = take_flag(&mut arguments, "--no-publish");
    let golden_frames = take_flag(&mut arguments, "--golden-frames");
    let command = arguments.first().map(String::as_str);
    anyhow::ensure!(
        !golden_frames || command == Some("capture"),
        "only `capture` takes --golden-frames"
    );
    anyhow::ensure!(
        !golden_frames || frames.is_empty(),
        "`capture` takes --golden-frames or --frame, not both"
    );
    anyhow::ensure!(
        !skip_aliasing || command == Some("bench"),
        "only `bench` takes --skip-aliasing"
    );
    anyhow::ensure!(
        !no_publish || command == Some("bench"),
        "only `bench` takes --no-publish"
    );
    anyhow::ensure!(
        release_build_seconds.is_none() || command == Some("publish"),
        "only `publish` takes --release-build-seconds"
    );
    anyhow::ensure!(
        experiment.is_none() || matches!(command, Some("bench" | "publish")),
        "only `bench` and `publish` take --experiment"
    );
    anyhow::ensure!(
        experiment.is_none() || !no_publish,
        "--experiment publishes, so it takes no --no-publish"
    );
    anyhow::ensure!(
        experiment.is_none() || release_build_seconds.is_none(),
        "a release build publishes beside main's results, never an Experiment's"
    );
    anyhow::ensure!(
        crowd_form.is_none() || matches!(command, Some("bench" | "capture")),
        "only `bench` and `capture` take --crowd-form"
    );
    anyhow::ensure!(
        frames.is_empty()
            || matches!(
                command,
                Some("capture" | "shadow-mask-difference" | "export-scene")
            ),
        "only `capture`, `shadow-mask-difference` and `export-scene` take --frame"
    );
    anyhow::ensure!(
        forced_lod.is_none() || matches!(command, Some("bench" | "capture")),
        "only `bench` and `capture` take --lod"
    );
    anyhow::ensure!(
        forced_lod.is_none() || command != Some("bench") || no_publish,
        "a bench with --lod joins no Series, so it takes --no-publish"
    );
    anyhow::ensure!(
        frames.len() <= 1 || command != Some("export-scene"),
        "`export-scene` takes one --frame"
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
        forced_lod,
        publish: !no_publish,
        experiment: experiment.clone(),
    };
    Ok(match arguments.as_slice() {
        [] => Command::Window(ReferenceScene::Helmet),
        [command] if command == "capabilities" => Command::Capabilities,
        [command] if command == "preflight" => Command::Preflight,
        [command] if command == "scenes" => Command::Scenes,
        #[cfg(feature = "cook")]
        [command] if command == "cook" => Command::Cook { scene: None },
        #[cfg(feature = "cook")]
        [command, scene] if command == "cook" => Command::Cook {
            scene: Some(scene.clone()),
        },
        #[cfg(feature = "cook")]
        [command] if command == "fetch" => Command::Fetch,
        #[cfg(feature = "cook")]
        [command, scene, output] if command == "capture" => Command::Capture {
            scene: scene.clone(),
            output: output.clone(),
            size: DEFAULT_CAPTURE_SIZE.into(),
            crowd_form,
            frames,
            golden_frames,
            forced_lod,
        },
        #[cfg(feature = "cook")]
        [command, scene, output, size] if command == "capture" => Command::Capture {
            scene: scene.clone(),
            output: output.clone(),
            size: size.clone(),
            crowd_form,
            frames,
            golden_frames,
            forced_lod,
        },
        #[cfg(feature = "cook")]
        [command, scene, output, size @ ..]
            if command == "shadow-mask-difference" && size.len() <= 1 =>
        {
            Command::ShadowMaskDifference {
                scene: scene.clone(),
                output: output.clone(),
                size: size
                    .first()
                    .map_or(DEFAULT_CAPTURE_SIZE, String::as_str)
                    .into(),
                frames,
            }
        }
        #[cfg(feature = "cook")]
        [command, scene, directory] if command == "export-scene" => Command::ExportScene {
            scene: scene.clone(),
            directory: directory.clone(),
            frame: frames.first().copied(),
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
            release_build_seconds,
            experiment,
        },
        [command, subcommand] if command == "history" && subcommand == "import" => {
            Command::ImportBaselines
        }
        [command, subcommand] if command == "history" && subcommand == "unrecorded" => {
            Command::UnrecordedScenes
        }
        [command, subcommand] if command == "history" && subcommand == "suspects" => {
            Command::Suspects
        }
        [command, subcommand] if command == "history" && subcommand == "regressions" => {
            Command::Regressions
        }
        [command, subcommand, night] if command == "history" && subcommand == "bisect" => {
            Command::Bisect {
                night: night.clone(),
            }
        }
        [command, subcommand, night] if command == "history" && subcommand == "publish-night" => {
            Command::PublishNight {
                night: night.clone(),
            }
        }
        [command, subcommand, results] if command == "history" && subcommand == "rerecord" => {
            Command::RerecordCounters {
                results: results.clone(),
            }
        }
        [command, results] if command == "compare" => Command::Compare {
            results: results.clone(),
            mode: CompareMode::Local,
        },
        [command, results, flag] if command == "compare" && flag == "--ci" => Command::Compare {
            results: results.clone(),
            mode: CompareMode::Ci,
        },
        [command, results, flag] if command == "compare" && flag == "--recording" => {
            Command::Compare {
                results: results.clone(),
                mode: CompareMode::Recording,
            }
        }
        [command, results] if command == "report" => Command::Report {
            results: results.clone(),
        },
        [command] if command == "traverse" => Command::Traverse,
        [scene] if ReferenceScene::from_name(scene).is_some() => {
            Command::Window(parse_scene(scene)?)
        }
        _ => {
            // Like the commands, their usage exists only in a `cook` build.
            let (cook_commands, crowd_form_clause) = if cfg!(feature = "cook") {
                (
                    "`cook [<scene>]`, `fetch`, `capture <scene> <png path> [<width>x<height>] [--frame <n>[,<n>...] | --golden-frames] [--lod <n>]`, \
                     `shadow-mask-difference <scene> <png path> [<width>x<height>] --frame <n>[,<n>...]`, \
                     `export-scene <scene> <directory> [--frame <n>]`, \
                     `difference <expected png> <actual png> <difference png>`, ",
                    "`capture` and `bench` also take",
                )
            } else {
                ("", "`bench` also takes")
            };
            anyhow::bail!(
                "unexpected arguments {arguments:?}: run `praxis [<scene>]`, `capabilities`, \
             `preflight`, `scenes`, {cook_commands}`bench <output directory> [--ci | --counters | --ci-counters] [--skip-aliasing] [--no-publish [--lod <n>] | --experiment <name>] [<scene>...]`, \
             `publish <bench directory> [--release-build-seconds <seconds> | --experiment <name>]`, `history import`, `history unrecorded`, `history suspects`, `history bisect <night directory>`, `history publish-night <night directory>`, `history regressions`, `history rerecord <bench directory>`, `compare <bench directory> [--ci | --recording]`, `report <bench directory>`, or `traverse`; {crowd_form_clause} \
             `--crowd-form <proxies | refit | rebuild>`; scenes are {:?}",
                ReferenceScene::ALL.map(ReferenceScene::name)
            )
        }
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

/// Prints a line per Reference scene: its name, then its checked frames.
fn print_scenes() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    for scene in ReferenceScene::ALL {
        let frames: Vec<String> = scene.checked_frames().iter().map(u32::to_string).collect();
        writeln!(stdout, "{} {}", scene.name(), frames.join(" "))?;
    }
    Ok(())
}

/// Cooks `assets/` and every procedural Reference scene but `world`, whose
/// library of more than 8 GB cooks by name or on a miss, into `cooked/`, or
/// with `scene`, that scene whole.
#[cfg(feature = "cook")]
fn cook(scene: Option<&str>) -> anyhow::Result<()> {
    let assets = Path::new(praxis::cook::ASSETS_DIRECTORY);
    let cache = praxis::cooked::CookedCache::new(praxis::cooked::COOKED_CACHE_DIRECTORY);
    let settings = praxis::cook::CookSettings::default();
    let report = match scene {
        None => {
            let scenes: Vec<_> = (ReferenceScene::ALL.into_iter())
                .filter(|&scene| scene != ReferenceScene::World)
                .collect();
            praxis::cook::cook(assets, &scenes, &cache, &settings)?
        }
        Some(scene) => praxis::cook::cook_scene(parse_scene(scene)?, assets, &cache, &settings)?,
    };
    writeln!(
        std::io::stdout(),
        "cooked {}, unchanged {}, dangling references {}",
        report.cooked.len(),
        report.unchanged.len(),
        report.dangling_references.len()
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

/// Benches the named Reference scenes, or with none every scene whose
/// assets are present, prints one line per scene, and writes
/// `<scene>.json` into `output`. A crowd form other than the default
/// benches `crowd` alone, written as `crowd-<form>.json`. With `forced_lod`,
/// every Instance draws that LOD, and a scene is written as
/// `<scene>-lod-<n>.json`. With `publish`,
/// the results then join the Bench history, as the `experiment` named; a
/// publish that fails warns and fails no bench.
fn bench(
    output: &str,
    configuration: Configuration,
    scene_names: &[String],
    crowd_form: CrowdForm,
    forced_lod: Option<u32>,
    publish: bool,
    experiment: Option<&str>,
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
            forced_lod,
            ..RenderSwitches::default()
        };
        let mut result = praxis::bench::run(&context, scene, configuration, switches, &start)?;
        if other_form {
            result.scene = format!("{crowd}-{}", crowd_form.name());
        }
        if let Some(lod) = forced_lod {
            result.scene = format!("{}-lod-{lod}", result.scene);
        }
        let path = Path::new(output).join(format!("{}.json", result.scene));
        std::fs::write(&path, serde_json::to_string_pretty(&result)? + "\n")?;
        writeln!(std::io::stdout(), "{result}")?;
        results.push(result);
    }
    if publish {
        let published = match experiment {
            Some(name) => praxis::bench::publish_experiment(Path::new("."), &results, name),
            None => praxis::bench::publish(Path::new("."), &results, None),
        };
        let retry_arguments =
            experiment.map_or(String::new(), |name| format!(" --experiment {name}"));
        match published {
            Ok(publication) => writeln!(std::io::stdout(), "{publication}")?,
            Err(error) => tracing::warn!(
                error = format!("{error:#}"),
                "bench results not published; `praxis publish {output}{retry_arguments}` retries"
            ),
        }
    }
    Ok(())
}

/// Publishes every `<scene>.json` in `results` as Bench records on the
/// `bench-history` branch, and prints what it did. With
/// `release_build_seconds`, this binary's release build joins them: its
/// size and the build's wall time. With `experiment`, they publish as that
/// Experiment's records instead, on no Series.
fn publish(
    results: &str,
    release_build_seconds: Option<u64>,
    experiment: Option<&str>,
) -> anyhow::Result<()> {
    let measurements = result_paths(results)?
        .iter()
        .map(|path| read_json(path))
        .collect::<anyhow::Result<Vec<Measurements>>>()?;
    let release_build = release_build_seconds
        .map(|build_seconds| {
            anyhow::ensure!(
                !cfg!(debug_assertions),
                "--release-build-seconds needs a release binary, whose size it records"
            );
            Ok(praxis::bench::ReleaseBuild {
                binary_bytes: std::env::current_exe()?.metadata()?.len(),
                build_seconds,
            })
        })
        .transpose()?;
    let publication = match experiment {
        Some(name) => praxis::bench::publish_experiment(Path::new("."), &measurements, name)?,
        None => praxis::bench::publish(Path::new("."), &measurements, release_build)?,
    };
    writeln!(std::io::stdout(), "{publication}")?;
    Ok(())
}

/// Imports the Baselines committed to `main` into the Bench history, those
/// since it started marked as re-recorded, and prints what it did.
fn import_baselines() -> anyhow::Result<()> {
    let publication = praxis::bench::import_baselines(Path::new("."))?;
    writeln!(std::io::stdout(), "{publication}")?;
    Ok(())
}

/// Prints, one per line, the Reference scenes but `world` with no Bench
/// record on a Series for the source of the remote's `main` on this
/// machine's adapter: the scenes a sweep benches.
fn unrecorded_scenes() -> anyhow::Result<()> {
    let adapter = pollster::block_on(praxis::gpu::adapter_info())?;
    let scenes: Vec<_> = (ReferenceScene::ALL.into_iter())
        .filter(|&scene| scene != ReferenceScene::World)
        .map(ReferenceScene::name)
        .collect();
    let mut stdout = std::io::stdout();
    for scene in praxis::bench::unrecorded_scenes(
        Path::new("."),
        &adapter.name,
        adapter.backend.to_str(),
        &scenes,
    )? {
        writeln!(stdout, "{scene}")?;
    }
    Ok(())
}

/// Prints, one per line and oldest first, this machine's Suspects that got
/// slower and have no Verdict yet, each with the record before it.
fn suspects() -> anyhow::Result<()> {
    let adapter = pollster::block_on(praxis::gpu::adapter_info())?;
    let mut stdout = std::io::stdout();
    for suspect in praxis::bench::suspects(Path::new("."), &adapter.name, adapter.backend.to_str())?
    {
        writeln!(stdout, "{suspect}")?;
    }
    Ok(())
}

/// Takes the sweep's next step of re-benching and bisecting a Suspect on
/// this machine, whose runs so far are in the Night `night`, creates the
/// next run's directory and prints three lines: the commit to bench, the
/// run's directory, then the rest of `bench`'s arguments. Nothing once the
/// Night is done.
fn bisect(night: &str) -> anyhow::Result<()> {
    let adapter = pollster::block_on(praxis::gpu::adapter_info())?;
    let (repository, night) = (Path::new("."), Path::new(night));
    if let Some((scene, commit)) =
        praxis::bench::bisect(repository, &adapter.name, adapter.backend.to_str(), night)?
    {
        let run = praxis::bench::next_run(repository, night, &scene, &commit)?;
        writeln!(
            std::io::stdout(),
            "{commit}\n{}\n{}",
            run.directory.display(),
            run.arguments.join(" ")
        )?;
    }
    Ok(())
}

/// Publishes the results of the Night `night`'s runs in one commit, and
/// prints what it did.
fn publish_night(night: &str) -> anyhow::Result<()> {
    if let Some(publication) = praxis::bench::publish_night(Path::new("."), Path::new(night))? {
        writeln!(std::io::stdout(), "{publication}")?;
    }
    Ok(())
}

/// Opens one `needs-triage` issue per Regression this machine's Verdicts
/// pinned that no re-recorded Baseline covers, and prints each Verdict's
/// scene, commit and issue.
fn regressions() -> anyhow::Result<()> {
    let adapter = pollster::block_on(praxis::gpu::adapter_info())?;
    let mut stdout = std::io::stdout();
    let repository = Path::new(".");
    for (scene, commit, issue) in praxis::bench::file_regressions(
        repository,
        &adapter.name,
        adapter.backend.to_str(),
        &mut praxis::bench::GitHubIssues { repository },
    )? {
        writeln!(stdout, "{scene} {commit} #{issue}")?;
    }
    Ok(())
}

/// Re-records the counters of each Baseline that a `<scene>.json` in
/// `results`, runs of `main`, counted otherwise, its timings kept, and prints
/// what it did as a Markdown list.
fn rerecord_counters(results: &str) -> anyhow::Result<()> {
    let results = result_paths(results)?
        .iter()
        .map(|path| read_json(path))
        .collect::<anyhow::Result<Vec<Measurements>>>()?;
    let report = praxis::bench::rerecord_baselines(Path::new("."), &results)?;
    write!(std::io::stdout(), "{report}")?;
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
            "`scripts/lavapipe-files.sh` copies it from CI".to_owned()
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

/// Renders the world's traversal and prints what it measured as a markdown
/// table.
fn traverse() -> anyhow::Result<()> {
    let context = pollster::block_on(GpuContext::headless())?;
    let traversal = praxis::bench::traverse(&context)?;
    writeln!(std::io::stdout(), "{traversal}")?;
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
/// `<width>x<height>`, its crowd in `crowd_form`, and writes `frames` of its
/// camera path, rendered in one run, as PNGs: one frame to `output`, each of
/// several beside it as `<output stem>-frame-<n>.png`. With `golden_frames`,
/// it writes the scene's Golden frames, named the same way, rendered as the
/// Goldens test renders them (`praxis::golden_captures`). With neither, it
/// writes the last frame `praxis::capture` renders and prints that frame's
/// stats, its point lights per lit pixel counted. With `forced_lod`, every Instance draws that LOD.
/// The PNG encoder comes with `cook`.
#[cfg(feature = "cook")]
fn capture(
    scene: &str,
    output: &str,
    size: &str,
    crowd_form: CrowdForm,
    frames: &[u32],
    golden_frames: bool,
    forced_lod: Option<u32>,
) -> anyhow::Result<()> {
    let scene = parse_scene(scene)?;
    let (width, height) = parse_size(size)?;
    let switches = RenderSwitches {
        count_point_lights: frames.is_empty() && !golden_frames,
        crowd_form,
        forced_lod,
        ..RenderSwitches::default()
    };
    let context = || pollster::block_on(GpuContext::headless());
    let (captures, last_frame) = if golden_frames {
        let pixels = praxis::golden_captures(scene, glam::DVec3::ZERO, width, height, switches)?;
        let paths = frame_paths(output, scene.golden_frames());
        (paths.zip(pixels).collect(), None)
    } else if frames.is_empty() {
        let (pixels, mut frame_stats) = praxis::capture(
            &context()?,
            scene,
            glam::DVec3::ZERO,
            width,
            height,
            switches,
        )?;
        (vec![(output.to_owned(), pixels)], frame_stats.pop())
    } else {
        let pixels = praxis::capture_frames(
            &context()?,
            scene,
            glam::DVec3::ZERO,
            width,
            height,
            frames,
            switches,
        )?;
        (frame_paths(output, frames).zip(pixels).collect(), None)
    };
    for (path, pixels) in &captures {
        image::save_buffer_with_format(
            path,
            pixels,
            width,
            height,
            image::ExtendedColorType::Rgba8,
            image::ImageFormat::Png,
        )?;
        tracing::info!(scene = scene.name(), path, width, height, "capture written");
    }
    if let Some(last_frame) = last_frame {
        std::io::stdout().write_all(last_frame.to_string().as_bytes())?;
    }
    Ok(())
}

/// The width and height `size` names as `<width>x<height>`.
#[cfg(feature = "cook")]
fn parse_size(size: &str) -> anyhow::Result<(u32, u32)> {
    size.split_once('x')
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .ok_or_else(|| anyhow::anyhow!("size {size:?} is not <width>x<height>"))
}

/// Where each of `frames` is written: one frame to `output`, each of
/// several beside it as `<output stem>-frame-<n>.png`.
#[cfg(feature = "cook")]
fn frame_paths<'a>(output: &'a str, frames: &'a [u32]) -> impl Iterator<Item = String> + 'a {
    let stem = output.strip_suffix(".png").unwrap_or(output);
    frames.iter().map(move |frame| match frames {
        [_] => output.to_owned(),
        _ => format!("{stem}-frame-{frame}.png"),
    })
}

/// Renders `frames` of the Reference scene named `scene` headless at `size`,
/// and again with LOD 0 forced, prints for each frame how many pixels of its
/// shadow mask differ between the two, and writes the image of them as
/// `capture` names its frames' (`praxis::shadow_mask_differences`).
#[cfg(feature = "cook")]
fn shadow_mask_difference(
    scene: &str,
    output: &str,
    size: &str,
    frames: &[u32],
) -> anyhow::Result<()> {
    anyhow::ensure!(!frames.is_empty(), "`shadow-mask-difference` takes --frame");
    let scene = parse_scene(scene)?;
    let (width, height) = parse_size(size)?;
    let context = pollster::block_on(GpuContext::headless())?;
    let differences = praxis::shadow_mask_differences(&context, scene, width, height, frames)?;
    for ((frame, path), difference) in frames
        .iter()
        .zip(frame_paths(output, frames))
        .zip(differences)
    {
        image::save_buffer_with_format(
            &path,
            &difference.image,
            width,
            height,
            image::ExtendedColorType::Rgba8,
            image::ImageFormat::Png,
        )?;
        writeln!(
            std::io::stdout(),
            "{} frame {frame}: {} of {} shadow mask pixels differ from LOD 0's ({path})",
            scene.name(),
            difference.differing_pixels,
            u64::from(width) * u64::from(height)
        )?;
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

    #[cfg(feature = "cook")]
    #[test]
    fn cook_takes_one_scene_to_cook_whole() {
        assert!(matches!(
            parse(&["cook"]).unwrap(),
            Command::Cook { scene: None }
        ));
        let Command::Cook { scene: Some(scene) } = parse(&["cook", "town"]).unwrap() else {
            panic!("`cook town` parsed as another command");
        };
        assert_eq!(scene, "town");
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
        assert!(error.contains("`preflight`"), "{error}");
        for command in [
            "`cook ",
            "`fetch`",
            "`capture ",
            "`export-scene ",
            "`difference ",
        ] {
            assert_eq!(
                error.contains(command),
                cfg!(feature = "cook"),
                "{command} in a build {} `cook`: {error}",
                if cfg!(feature = "cook") {
                    "with"
                } else {
                    "without"
                }
            );
        }
    }

    #[test]
    fn bench_still_parses_its_configuration_and_scenes() {
        let Command::Bench {
            output,
            configuration,
            scenes,
            crowd_form,
            forced_lod,
            publish,
            experiment,
        } = parse(&["bench", "out", "--ci", "town", "crowd"]).unwrap()
        else {
            panic!("bench parsed as another command");
        };
        assert_eq!(output, "out");
        assert_eq!(configuration.width, Configuration::CI.width);
        assert_eq!(scenes, ["town", "crowd"]);
        assert_eq!(crowd_form, CrowdForm::default());
        assert_eq!(forced_lod, None);
        assert!(publish);
        assert_eq!(experiment, None);
    }

    #[test]
    fn bench_and_publish_take_an_experiment_and_nothing_else_does() {
        let experiment = |arguments: &[&str]| match parse(arguments).unwrap() {
            Command::Bench {
                publish,
                experiment,
                ..
            } => publish.then_some(experiment).flatten(),
            Command::Publish { experiment, .. } => experiment,
            _ => panic!("parsed as another command"),
        };
        for arguments in [
            &["bench", "out", "--experiment", "faster-cull", "town"][..],
            &["publish", "out", "--experiment", "faster-cull"][..],
        ] {
            assert_eq!(experiment(arguments).as_deref(), Some("faster-cull"));
        }
        for (arguments, expected) in [
            (
                &["compare", "out", "--experiment", "faster-cull"][..],
                "only `bench` and `publish` take --experiment",
            ),
            (
                &[
                    "bench",
                    "out",
                    "--experiment",
                    "faster-cull",
                    "--no-publish",
                ][..],
                "takes no --no-publish",
            ),
            (
                &[
                    "publish",
                    "out",
                    "--experiment",
                    "faster-cull",
                    "--release-build-seconds",
                    "412",
                ][..],
                "never an Experiment's",
            ),
            (&["bench", "out", "--experiment", "../main"][..], "../main"),
        ] {
            let error = parse(arguments).unwrap_err().to_string();
            assert!(error.contains(expected), "{arguments:?}: {error}");
        }
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
        let Command::Publish {
            results,
            release_build_seconds,
            ..
        } = parse(&["publish", "out"]).unwrap()
        else {
            panic!("publish parsed as another command");
        };
        assert_eq!((results.as_str(), release_build_seconds), ("out", None));
        assert!(matches!(
            parse(&["history", "import"]).unwrap(),
            Command::ImportBaselines
        ));
        assert!(matches!(
            parse(&["history", "unrecorded"]).unwrap(),
            Command::UnrecordedScenes
        ));
        assert!(matches!(
            parse(&["history", "suspects"]).unwrap(),
            Command::Suspects
        ));
        let Command::Bisect { night } = parse(&["history", "bisect", "night"]).unwrap() else {
            panic!("history bisect parsed as another command");
        };
        assert_eq!(night, "night");
        let Command::PublishNight { night } =
            parse(&["history", "publish-night", "night"]).unwrap()
        else {
            panic!("history publish-night parsed as another command");
        };
        assert_eq!(night, "night");
        let Command::RerecordCounters { results } =
            parse(&["history", "rerecord", "results"]).unwrap()
        else {
            panic!("history rerecord parsed as another command");
        };
        assert_eq!(results, "results");
        assert!(matches!(
            parse(&["history", "regressions"]).unwrap(),
            Command::Regressions
        ));
        assert!(parse(&["history"]).is_err());
    }

    #[test]
    fn publish_takes_a_release_builds_seconds_and_only_publish_does() {
        let Command::Publish {
            release_build_seconds,
            ..
        } = parse(&["publish", "out", "--release-build-seconds", "412"]).unwrap()
        else {
            panic!("publish parsed as another command");
        };
        assert_eq!(release_build_seconds, Some(412));
        for (arguments, expected) in [
            (
                &["publish", "out", "--release-build-seconds", "4m"][..],
                "takes whole seconds",
            ),
            (
                &["bench", "out", "--release-build-seconds", "412"][..],
                "only `publish` takes --release-build-seconds",
            ),
        ] {
            let error = parse(arguments).unwrap_err().to_string();
            assert!(error.contains(expected), "{error}");
        }
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
            let Command::Capture { scene, frames, .. } = parse(&arguments).unwrap() else {
                panic!("capture parsed as another command");
            };
            assert_eq!(scene, "town");
            assert_eq!(frames, [300]);
        }
    }

    #[test]
    fn scenes_parses_alone() {
        assert!(matches!(parse(&["scenes"]).unwrap(), Command::Scenes));
        assert!(parse(&["scenes", "town"]).is_err());
    }

    #[cfg(feature = "cook")]
    #[test]
    fn capture_takes_golden_frames_in_place_of_frames() {
        let golden_frames = |arguments: &[&str]| match parse(arguments).unwrap() {
            Command::Capture { golden_frames, .. } => golden_frames,
            _ => panic!("capture parsed as another command"),
        };
        assert!(golden_frames(&[
            "capture",
            "world",
            "out.png",
            "--golden-frames"
        ]));
        assert!(golden_frames(&[
            "capture",
            "--golden-frames",
            "world",
            "out.png",
            "64x36"
        ]));
        assert!(!golden_frames(&["capture", "world", "out.png"]));
        for (arguments, expected) in [
            (
                &[
                    "capture",
                    "world",
                    "out.png",
                    "--golden-frames",
                    "--frame",
                    "300",
                ][..],
                "takes --golden-frames or --frame, not both",
            ),
            (
                &["bench", "out", "--golden-frames"][..],
                "only `capture` takes --golden-frames",
            ),
        ] {
            let error = parse(arguments).unwrap_err().to_string();
            assert!(error.contains(expected), "{arguments:?}: {error}");
        }
    }

    #[cfg(feature = "cook")]
    #[test]
    fn capture_and_an_unpublished_bench_take_a_forced_lod() {
        let Command::Capture { forced_lod, .. } =
            parse(&["capture", "town", "out.png", "--lod", "0"]).unwrap()
        else {
            panic!("capture parsed as another command");
        };
        assert_eq!(forced_lod, Some(0));
        let Command::Capture { forced_lod, .. } = parse(&["capture", "town", "out.png"]).unwrap()
        else {
            panic!("capture parsed as another command");
        };
        assert_eq!(forced_lod, None);
        let error = parse(&["capture", "town", "out.png", "--lod", "finest"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("--lod takes a LOD's number"), "{error}");
        let Command::Bench { forced_lod, .. } =
            parse(&["bench", "out", "--no-publish", "--lod", "0", "town"]).unwrap()
        else {
            panic!("bench parsed as another command");
        };
        assert_eq!(forced_lod, Some(0));
        let error = parse(&["bench", "out", "--lod", "0"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("it takes --no-publish"), "{error}");
        let error = parse(&["report", "out", "--lod", "0"])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("only `bench` and `capture` take --lod"),
            "{error}"
        );
    }

    #[cfg(feature = "cook")]
    #[test]
    fn shadow_mask_difference_takes_frames_and_a_size() {
        let Command::ShadowMaskDifference { size, frames, .. } = parse(&[
            "shadow-mask-difference",
            "town",
            "out.png",
            "2560x1440",
            "--frame",
            "59,767,900",
        ])
        .unwrap() else {
            panic!("shadow-mask-difference parsed as another command");
        };
        assert_eq!(size, "2560x1440");
        assert_eq!(frames, [59, 767, 900]);
        let Command::ShadowMaskDifference { size, .. } =
            parse(&["shadow-mask-difference", "town", "out.png"]).unwrap()
        else {
            panic!("shadow-mask-difference parsed as another command");
        };
        assert_eq!(size, DEFAULT_CAPTURE_SIZE);
    }

    #[cfg(feature = "cook")]
    #[test]
    fn capture_parses_frames_in_one_run_and_export_scene_refuses_them() {
        let Command::Capture { frames, .. } =
            parse(&["capture", "world", "out.png", "--frame", "59,300,599"]).unwrap()
        else {
            panic!("capture parsed as another command");
        };
        assert_eq!(frames, [59, 300, 599]);
        let error = parse(&["export-scene", "world", "out", "--frame", "59,300"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("one --frame"), "{error}");
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
            error.contains(
                "only `capture`, `shadow-mask-difference` and `export-scene` take --frame"
            ),
            "{error}"
        );
    }

    #[test]
    fn report_parses_its_results_directory() {
        let Command::Report { results } = parse(&["report", "out"]).unwrap() else {
            panic!("report parsed as another command");
        };
        assert_eq!(results, "out");
        assert!(matches!(parse(&["traverse"]).unwrap(), Command::Traverse));
    }

    #[test]
    fn compare_parses_its_mode() {
        for (flag, expected) in [
            (None, CompareMode::Local),
            (Some("--ci"), CompareMode::Ci),
            (Some("--recording"), CompareMode::Recording),
        ] {
            let arguments: Vec<&str> = ["compare", "out"].into_iter().chain(flag).collect();
            let Command::Compare { results, mode } = parse(&arguments).unwrap() else {
                panic!("{arguments:?} is a compare");
            };
            assert_eq!((results.as_str(), mode), ("out", expected));
        }
    }
}
