use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;

use crate::capture::render_frames;
use crate::gpu::GpuContext;
use crate::render::{RenderSwitches, TIMED_PASSES};
use crate::scenes::ReferenceScene;
use crate::stats::FrameStats;
use crate::time::SIXTY_HERTZ;

/// The Baselines `compare` reads, `<adapter key>/<scene>.json`: the reference
/// machine's (ADR 0013), and CI's lavapipe for counters only. Keyed by
/// adapter, so neither machine compares with the other's.
pub const BASELINE_DIRECTORY: &str = "bench/baselines";

/// The Budgets `compare` reads: ADR 0013's numbers.
pub const BUDGETS_PATH: &str = "bench/budgets.json";

/// A bench run's render size and frame counts. A run compares only with a
/// Baseline of the same size: every metric when both measured as many
/// frames, the counter frames' counters otherwise.
#[derive(Clone, Copy, Debug)]
pub struct Configuration {
    pub width: u32,
    pub height: u32,
    /// Frames rendered before measuring, while the first frames' allocations
    /// and the GPU's clocks settle; the counter frames among them are counted.
    pub warm_up_frames: u32,
    pub measured_frames: u32,
}

impl Configuration {
    /// ADR 0013 judges Budgets at 1440p. With the warm-up, as many frames as
    /// one run can time on the GPU.
    pub const REFERENCE: Self = Self {
        width: 2560,
        height: 1440,
        warm_up_frames: 248,
        measured_frames: 1800,
    };

    /// CI's lavapipe renders on the CPU, so CI benches at the Goldens' size
    /// and for fewer frames: `compare --ci` reads counters only.
    pub const CI: Self = Self {
        width: 640,
        height: 360,
        warm_up_frames: 248,
        measured_frames: 256,
    };

    /// The reference size, rendering the counter frames and no more: the
    /// verify command's bench, whose `compare --ci` reads counters only.
    pub const COUNTERS: Self = Self {
        width: Self::REFERENCE.width,
        height: Self::REFERENCE.height,
        warm_up_frames: COUNTER_FRAMES.start,
        measured_frames: COUNTER_FRAMES.end - COUNTER_FRAMES.start,
    };
}

/// The frames whose counters every run also records, whatever its length:
/// counters repeat exactly on every run, so runs of different lengths
/// compare on these. By the eighth frame the Scene's uploads and the first
/// frames' allocations are over (#178). Baselines record the frames, so
/// moving them refuses every Baseline until it is re-recorded.
const COUNTER_FRAMES: std::ops::Range<u32> = 8..72;

/// The Baseline directory of `adapter`: its name in lowercase, each run of
/// other characters a hyphen (`Apple M4 Pro` is `apple-m4-pro`).
pub fn baseline_directory(adapter: &str) -> PathBuf {
    let key = adapter
        .to_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    PathBuf::from(BASELINE_DIRECTORY).join(key)
}

/// A time warns above this ratio to its Baseline and fails above
/// `FAIL_RATIO`; a counter fails on any increase, as it is the same on every
/// run. Noise floor on the M4 Pro, 6 runs of each scene under the owner's
/// usual load: counters identical, GPU medians within 3.4%, CPU medians of
/// 31–42 µs within 8.1 µs (#34).
const WARN_RATIO: f64 = 1.05;
const FAIL_RATIO: f64 = 1.10;

/// CPU frames of tens of microseconds move by more than the ratios between
/// runs, so a CPU time fails only when it also grows by more than this.
const CPU_NOISE_FLOOR_MICROSECONDS: f64 = 10.0;

/// Likewise for a pass's GPU time: over three runs on the M4 Pro, helmet's
/// culling pass read 136 to 153 µs, every other pass within 3%.
const GPU_PASS_NOISE_FLOOR_MICROSECONDS: f64 = 20.0;

/// The steady-state cost of one Reference scene: medians over the measured
/// frames, plus p95 for times, and the counter frames' counters. Bench
/// output and Baselines share this form.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Measurements {
    pub scene: String,
    pub adapter: String,
    /// `debug` or `release`: allocations differ between the two.
    pub profile: String,
    pub width: u32,
    pub height: u32,
    pub measured_frames: u32,
    pub cpu_frame_microseconds: Spread,
    /// From the start of each frame's first pass to the end of its last.
    pub gpu_frame_microseconds: Spread,
    /// Each pass's own GPU time, by pass.
    pub gpu_pass_microseconds: BTreeMap<String, Spread>,
    #[serde(flatten)]
    pub counters: Counters,
    pub counter_frames: CounterFrames,
}

impl Measurements {
    /// A Baseline from its JSON; one that another version of the bench
    /// recorded is refused with a message to re-record it.
    pub fn from_baseline_json(json: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(json).context("not in this bench's form: re-record it")
    }
}

/// Medians of counters, which repeat exactly on every run, but for the
/// Instances overflowed: the most of any frame, so one overflow counts.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Counters {
    pub draw_calls: u64,
    pub instances_drawn: u64,
    pub triangles_drawn: u64,
    pub instances_overflowed: u64,
    pub uploaded_bytes: u64,
    pub gpu_memory_bytes: u64,
    pub arena_bytes_in_use: u64,
    pub heap_allocations: u64,
}

impl Counters {
    /// The counters of `frames`, each with its GPU stats.
    fn of(frames: &[FrameStats]) -> Self {
        let median = |value: fn(&FrameStats) -> u64| {
            percentile(
                frames.iter().map(|stats| value(stats) as f64).collect(),
                0.5,
            ) as u64
        };
        let most = |value: fn(&FrameStats) -> u64| frames.iter().map(value).max().unwrap_or(0);
        Self {
            draw_calls: median(|stats| u64::from(stats.draw_calls)),
            instances_drawn: median(|stats| stats.gpu.map_or(0, |gpu| gpu.instances_drawn.into())),
            triangles_drawn: median(|stats| stats.gpu.map_or(0, |gpu| gpu.triangles_drawn.into())),
            instances_overflowed: most(|stats| {
                stats.gpu.map_or(0, |gpu| gpu.instances_overflowed.into())
            }),
            uploaded_bytes: median(|stats| stats.uploaded_bytes),
            gpu_memory_bytes: median(|stats| stats.gpu_memory_bytes),
            arena_bytes_in_use: median(|stats| stats.arena_bytes_in_use),
            heap_allocations: median(|stats| stats.heap_allocations),
        }
    }

    fn named(&self) -> [(&'static str, u64); 8] {
        [
            ("draw_calls", self.draw_calls),
            ("instances_drawn", self.instances_drawn),
            ("triangles_drawn", self.triangles_drawn),
            ("instances_overflowed", self.instances_overflowed),
            ("uploaded_bytes", self.uploaded_bytes),
            ("gpu_memory_bytes", self.gpu_memory_bytes),
            ("arena_bytes_in_use", self.arena_bytes_in_use),
            ("heap_allocations", self.heap_allocations),
        ]
    }
}

/// The counters of `COUNTER_FRAMES` alone, which every run renders.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CounterFrames {
    pub frames: std::ops::Range<u32>,
    #[serde(flatten)]
    pub counters: Counters,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Spread {
    pub median: f64,
    pub p95: f64,
}

/// ADR 0013's per-frame limits on the reference machine.
#[derive(Debug, serde::Deserialize)]
pub struct Budgets {
    pub cpu_frame_microseconds: f64,
    /// From the start of the frame's first pass to the end of its last.
    pub gpu_frame_microseconds: f64,
    pub gpu_memory_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompareMode {
    /// Every metric, against a Baseline from this adapter.
    Local,
    /// Deterministic counters only: times depend on the machine and its load.
    Ci,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verdict {
    Warn,
    Fail,
}

/// A metric that regressed against its Baseline or exceeds its Budget.
#[derive(Debug, PartialEq)]
pub struct Finding {
    pub metric: &'static str,
    pub verdict: Verdict,
    pub detail: String,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let verdict = match self.verdict {
            Verdict::Warn => "warn",
            Verdict::Fail => "FAIL",
        };
        write!(formatter, "{verdict} {} {}", self.metric, self.detail)
    }
}

/// Renders `scene` for `configuration`'s warm-up and measured frames, as
/// `switches` set, and returns the measured frames' medians and p95s, and
/// the counter frames' counters.
pub fn run(
    context: &GpuContext,
    scene: ReferenceScene,
    configuration: Configuration,
    switches: RenderSwitches,
) -> anyhow::Result<Measurements> {
    let frame_count = configuration.warm_up_frames + configuration.measured_frames;
    anyhow::ensure!(
        frame_count >= COUNTER_FRAMES.end,
        "a run of {frame_count} frames stops before the counter frames {COUNTER_FRAMES:?} end"
    );
    let (_, frames) = render_frames(
        context,
        scene,
        glam::DVec3::ZERO,
        configuration.width,
        configuration.height,
        frame_count,
        // ADR 0022 judges Budgets at 60 Hz.
        SIXTY_HERTZ,
        switches,
    )?;
    measure(
        scene,
        context.adapter.get_info().name,
        configuration,
        &frames,
    )
}

/// The Measurements of `frames`, a `configuration` run of `scene` on
/// `adapter`, which fail unless every frame read back its GPU stats.
fn measure(
    scene: ReferenceScene,
    adapter: String,
    configuration: Configuration,
    frames: &[FrameStats],
) -> anyhow::Result<Measurements> {
    let gpu_frames: Vec<_> = frames.iter().filter_map(|stats| stats.gpu).collect();
    anyhow::ensure!(
        gpu_frames.len() == frames.len(),
        "{} of {} frames read back no GPU stats",
        frames.len() - gpu_frames.len(),
        frames.len()
    );
    let warm_up_frames = configuration.warm_up_frames as usize;
    let (measured, measured_gpu) = (&frames[warm_up_frames..], &gpu_frames[warm_up_frames..]);
    let microseconds = |time: Duration| time.as_secs_f64() * 1e6;
    let gpu_pass_microseconds = TIMED_PASSES
        .iter()
        .enumerate()
        .map(|(pass, &name)| {
            let times = measured_gpu
                .iter()
                .map(|gpu| microseconds(gpu.pass_times[pass].1));
            (name.to_owned(), spread(times.collect()))
        })
        .collect();
    Ok(Measurements {
        scene: scene.name().to_owned(),
        adapter,
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_owned(),
        width: configuration.width,
        height: configuration.height,
        measured_frames: configuration.measured_frames,
        cpu_frame_microseconds: spread(
            measured
                .iter()
                .map(|stats| microseconds(stats.cpu_frame_time))
                .collect(),
        ),
        gpu_frame_microseconds: spread(
            measured_gpu
                .iter()
                .map(|gpu| microseconds(gpu.frame_time))
                .collect(),
        ),
        gpu_pass_microseconds,
        counters: Counters::of(measured),
        counter_frames: CounterFrames {
            frames: COUNTER_FRAMES,
            counters: Counters::of(
                &frames[COUNTER_FRAMES.start as usize..COUNTER_FRAMES.end as usize],
            ),
        },
    })
}

/// One line, `key=value` pairs after the scene name: medians and p95s of
/// the CPU and GPU frame, each pass's median, then the counters.
impl std::fmt::Display for Measurements {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cpu, gpu) = (self.cpu_frame_microseconds, self.gpu_frame_microseconds);
        write!(
            formatter,
            "{} cpu_us median={:.1} p95={:.1} gpu_us median={:.1} p95={:.1}",
            self.scene, cpu.median, cpu.p95, gpu.median, gpu.p95
        )?;
        for (pass, time) in &self.gpu_pass_microseconds {
            write!(formatter, " {pass}={:.1}", time.median)?;
        }
        for (counter, value) in self.counters.named() {
            write!(formatter, " {counter}={value}")?;
        }
        Ok(())
    }
}

/// Compares `current` with its `baseline` and the Budgets. Medians count;
/// p95s are reported only, as they move with one slow frame. Runs that
/// measured as many frames compare their measured frames' counters; runs of
/// other lengths compare their counter frames', and only with
/// `CompareMode::Ci`, as their times cover other frames.
pub fn compare(
    current: &Measurements,
    baseline: &Measurements,
    budgets: &Budgets,
    mode: CompareMode,
) -> anyhow::Result<Vec<Finding>> {
    anyhow::ensure!(
        (
            &current.scene,
            &current.profile,
            current.width,
            current.height
        ) == (
            &baseline.scene,
            &baseline.profile,
            baseline.width,
            baseline.height
        ),
        "{} ran {} at {}x{}, its Baseline is {} {} at {}x{}",
        current.scene,
        current.profile,
        current.width,
        current.height,
        baseline.scene,
        baseline.profile,
        baseline.width,
        baseline.height
    );
    anyhow::ensure!(
        current.counter_frames.frames == baseline.counter_frames.frames,
        "{} counted frames {:?}, its Baseline {:?}: re-record the Baseline",
        current.scene,
        current.counter_frames.frames,
        baseline.counter_frames.frames
    );
    let (counters, baseline_counters) = if current.measured_frames == baseline.measured_frames {
        (current.counters, baseline.counters)
    } else {
        (
            current.counter_frames.counters,
            baseline.counter_frames.counters,
        )
    };
    let mut findings = Vec::new();
    for ((metric, current), (_, baseline)) in
        counters.named().into_iter().zip(baseline_counters.named())
    {
        if current > baseline {
            findings.push(Finding {
                metric,
                verdict: Verdict::Fail,
                detail: format!("{current} against Baseline {baseline}"),
            });
        }
    }
    check_budget(
        &mut findings,
        "gpu_memory_bytes",
        current.counters.gpu_memory_bytes as f64,
        budgets.gpu_memory_bytes as f64,
    );
    if mode == CompareMode::Ci {
        return Ok(findings);
    }

    anyhow::ensure!(
        current.measured_frames == baseline.measured_frames,
        "{} measured {} frames, its Baseline {}: compare times between runs of one length, \
         or counters only with --ci",
        current.scene,
        current.measured_frames,
        baseline.measured_frames
    );
    anyhow::ensure!(
        current.adapter == baseline.adapter,
        "{} ran on {:?}, its Baseline on {:?}: compare times on the Baseline's machine, \
         or counters only with --ci",
        current.scene,
        current.adapter,
        baseline.adapter
    );
    let cpu = current.cpu_frame_microseconds.median;
    check_regression(
        &mut findings,
        "cpu_frame_microseconds",
        cpu,
        baseline.cpu_frame_microseconds.median,
        CPU_NOISE_FLOOR_MICROSECONDS,
    );
    check_budget(
        &mut findings,
        "cpu_frame_microseconds",
        cpu,
        budgets.cpu_frame_microseconds,
    );
    let passes = &current.gpu_pass_microseconds;
    let baseline_passes = &baseline.gpu_pass_microseconds;
    anyhow::ensure!(
        passes.keys().eq(baseline_passes.keys()),
        "{} timed passes {:?}, its Baseline {:?}: re-record the Baseline",
        current.scene,
        passes.keys().collect::<Vec<_>>(),
        baseline_passes.keys().collect::<Vec<_>>()
    );
    for ((pass, time), baseline_time) in passes.iter().zip(baseline_passes.values()) {
        let earlier = findings.len();
        let metric = "gpu_pass_microseconds";
        let (median, baseline_median) = (time.median, baseline_time.median);
        let noise_floor = GPU_PASS_NOISE_FLOOR_MICROSECONDS;
        check_regression(&mut findings, metric, median, baseline_median, noise_floor);
        for finding in &mut findings[earlier..] {
            finding.detail = format!("{pass} {}", finding.detail);
        }
    }
    check_budget(
        &mut findings,
        "gpu_frame_microseconds",
        current.gpu_frame_microseconds.median,
        budgets.gpu_frame_microseconds,
    );
    Ok(findings)
}

/// Warns when the time `current` exceeds `baseline` by more than
/// `WARN_RATIO`, and fails above `FAIL_RATIO` when it also grew by more than
/// `noise_floor`.
fn check_regression(
    findings: &mut Vec<Finding>,
    metric: &'static str,
    current: f64,
    baseline: f64,
    noise_floor: f64,
) {
    let ratio = current / baseline;
    let verdict = if ratio > FAIL_RATIO && current - baseline > noise_floor {
        Verdict::Fail
    } else if ratio > WARN_RATIO {
        Verdict::Warn
    } else {
        return;
    };
    findings.push(Finding {
        metric,
        verdict,
        detail: format!(
            "{current:.1} against Baseline {baseline:.1} ({:+.1}%)",
            (ratio - 1.0) * 100.0
        ),
    });
}

/// Fails when `current` exceeds `budget`.
fn check_budget(findings: &mut Vec<Finding>, metric: &'static str, current: f64, budget: f64) {
    if current > budget {
        findings.push(Finding {
            metric,
            verdict: Verdict::Fail,
            detail: format!("{current:.1} over Budget {budget:.1}"),
        });
    }
}

fn spread(values: Vec<f64>) -> Spread {
    let median = percentile(values.clone(), 0.5);
    Spread {
        median,
        p95: percentile(values, 0.95),
    }
}

/// The nearest-rank percentile: the smallest value with at least `fraction`
/// of `values` at or below it.
fn percentile(mut values: Vec<f64>, fraction: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    let rank = (fraction * values.len() as f64).ceil() as usize;
    values[rank.clamp(1, values.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::GpuFrameStats;

    const COUNTERS: Counters = Counters {
        draw_calls: 10,
        instances_drawn: 10,
        triangles_drawn: 22_080,
        instances_overflowed: 0,
        uploaded_bytes: 1712,
        gpu_memory_bytes: 1 << 30,
        arena_bytes_in_use: 85_296,
        heap_allocations: 100,
    };

    fn baseline() -> Measurements {
        Measurements {
            scene: "pbr-sweep".to_owned(),
            adapter: "Apple M4 Pro".to_owned(),
            profile: "release".to_owned(),
            width: Configuration::REFERENCE.width,
            height: Configuration::REFERENCE.height,
            measured_frames: Configuration::REFERENCE.measured_frames,
            cpu_frame_microseconds: Spread {
                median: 1000.0,
                p95: 1200.0,
            },
            gpu_frame_microseconds: Spread {
                median: 2500.0,
                p95: 2600.0,
            },
            gpu_pass_microseconds: [("culling_pass", 100.0), ("main_pass", 2000.0)]
                .into_iter()
                .map(|(pass, median)| {
                    let p95 = median + 100.0;
                    (pass.to_owned(), Spread { median, p95 })
                })
                .collect(),
            counters: COUNTERS,
            counter_frames: CounterFrames {
                frames: COUNTER_FRAMES,
                counters: COUNTERS,
            },
        }
    }

    const BUDGETS: Budgets = Budgets {
        cpu_frame_microseconds: 6000.0,
        gpu_frame_microseconds: 12000.0,
        gpu_memory_bytes: 8_000_000_000,
    };

    fn verdicts(
        current: &Measurements,
        baseline: &Measurements,
        mode: CompareMode,
    ) -> Vec<(&'static str, Verdict)> {
        compare(current, baseline, &BUDGETS, mode)
            .expect("comparable")
            .into_iter()
            .map(|finding| (finding.metric, finding.verdict))
            .collect()
    }

    #[test]
    fn the_baseline_compares_clean() {
        assert_eq!(verdicts(&baseline(), &baseline(), CompareMode::Local), []);
    }

    #[test]
    fn a_time_warns_above_5_percent_and_fails_above_10() {
        let mut current = baseline();
        current.cpu_frame_microseconds.median = 1060.0;
        current.gpu_pass_microseconds.insert(
            "main_pass".to_owned(),
            Spread {
                median: 2300.0,
                p95: 2400.0,
            },
        );
        assert_eq!(
            verdicts(&current, &baseline(), CompareMode::Local),
            [
                ("cpu_frame_microseconds", Verdict::Warn),
                ("gpu_pass_microseconds", Verdict::Fail)
            ]
        );
    }

    #[test]
    fn exactly_5_percent_and_improvements_pass() {
        let mut current = baseline();
        current.cpu_frame_microseconds.median = 1050.0;
        current.gpu_pass_microseconds.insert(
            "main_pass".to_owned(),
            Spread {
                median: 1000.0,
                p95: 9000.0,
            },
        );
        current.counters.heap_allocations = 90;
        assert_eq!(verdicts(&current, &baseline(), CompareMode::Local), []);
    }

    #[test]
    fn a_counter_fails_on_any_increase() {
        let mut current = baseline();
        current.counters.heap_allocations = 101;
        assert_eq!(
            verdicts(&current, &baseline(), CompareMode::Local),
            [("heap_allocations", Verdict::Fail)]
        );
    }

    #[test]
    fn ci_mode_ignores_times_and_fails_on_counters() {
        let mut current = baseline();
        current.adapter = "llvmpipe".to_owned();
        current.cpu_frame_microseconds.median = 5000.0;
        current.gpu_frame_microseconds.median = 50_000.0;
        current.counters.draw_calls = 11;
        current.counters.instances_drawn += 1;
        current.counters.triangles_drawn += 1;
        current.counters.instances_overflowed += 1;
        current.counters.uploaded_bytes = 1712 * 2;
        current.counters.gpu_memory_bytes = (1 << 30) + 1;
        current.counters.arena_bytes_in_use += 1;
        assert_eq!(
            verdicts(&current, &baseline(), CompareMode::Ci),
            [
                ("draw_calls", Verdict::Fail),
                ("instances_drawn", Verdict::Fail),
                ("triangles_drawn", Verdict::Fail),
                ("instances_overflowed", Verdict::Fail),
                ("uploaded_bytes", Verdict::Fail),
                ("gpu_memory_bytes", Verdict::Fail),
                ("arena_bytes_in_use", Verdict::Fail)
            ]
        );
    }

    #[test]
    fn over_budget_fails_even_without_regression() {
        let mut current = baseline();
        current.cpu_frame_microseconds.median = 7000.0;
        let mut over_budget = baseline();
        over_budget.cpu_frame_microseconds.median = 6900.0;
        assert_eq!(
            verdicts(&current, &over_budget, CompareMode::Local),
            [("cpu_frame_microseconds", Verdict::Fail)]
        );
    }

    /// The GPU frame total answers to the 12 ms Budget only, and each pass
    /// to its Baseline.
    #[test]
    fn the_gpu_frame_total_fails_over_its_budget_alone() {
        let mut slower = baseline();
        slower.gpu_frame_microseconds.median = 3000.0;
        assert_eq!(verdicts(&slower, &baseline(), CompareMode::Local), []);
        let mut over_budget = baseline();
        over_budget.gpu_frame_microseconds.median = 12_500.0;
        assert_eq!(
            verdicts(&over_budget, &over_budget, CompareMode::Local),
            [("gpu_frame_microseconds", Verdict::Fail)]
        );
    }

    #[test]
    fn a_baseline_of_other_passes_is_refused_with_a_message_to_re_record_it() {
        let mut more_passes = baseline();
        let depth_prepass = Spread {
            median: 300.0,
            p95: 350.0,
        };
        more_passes
            .gpu_pass_microseconds
            .insert("depth_prepass".to_owned(), depth_prepass);
        let error = compare(&more_passes, &baseline(), &BUDGETS, CompareMode::Local)
            .expect_err("other passes");
        assert!(error.to_string().contains("re-record"), "{error}");
    }

    /// The helmet's Baseline as recorded before GPU time was split by pass.
    const OLD_BASELINE: &str = r#"{
  "scene": "helmet",
  "adapter": "Apple M4 Pro",
  "profile": "release",
  "width": 2560,
  "height": 1440,
  "measured_frames": 1800,
  "cpu_frame_microseconds": { "median": 48.583, "p95": 106.042 },
  "gpu_main_pass_microseconds": { "median": 509.167, "p95": 641.458 },
  "draw_calls": 1,
  "uploaded_bytes": 288,
  "gpu_memory_bytes": 271864916,
  "arena_bytes_in_use": 844804,
  "heap_allocations": 98,
  "counter_frames": {
    "frames": { "start": 8, "end": 72 },
    "draw_calls": 1,
    "uploaded_bytes": 288,
    "gpu_memory_bytes": 271864916,
    "arena_bytes_in_use": 844804,
    "heap_allocations": 98
  }
}"#;

    #[test]
    fn a_baseline_in_the_old_form_is_refused_with_a_message_to_re_record_it() {
        let error =
            Measurements::from_baseline_json(OLD_BASELINE.as_bytes()).expect_err("the old form");
        assert!(format!("{error:#}").contains("re-record"), "{error:#}");
        let current = serde_json::to_vec(&baseline()).expect("serializable");
        let parsed = Measurements::from_baseline_json(&current).expect("the current form");
        assert_eq!(parsed, baseline());
    }

    #[test]
    fn cpu_time_within_the_noise_floor_only_warns() {
        let mut small = baseline();
        small.cpu_frame_microseconds.median = 32.0;
        let mut current = small.clone();
        current.cpu_frame_microseconds.median = 42.0;
        assert_eq!(
            verdicts(&current, &small, CompareMode::Local),
            [("cpu_frame_microseconds", Verdict::Warn)]
        );
        current.cpu_frame_microseconds.median = 42.5;
        assert_eq!(
            verdicts(&current, &small, CompareMode::Local),
            [("cpu_frame_microseconds", Verdict::Fail)]
        );
    }

    #[test]
    fn a_pass_time_within_the_noise_floor_only_warns() {
        let culling = |median| Spread {
            median,
            p95: median,
        };
        let mut current = baseline();
        current
            .gpu_pass_microseconds
            .insert("culling_pass".to_owned(), culling(115.0));
        assert_eq!(
            verdicts(&current, &baseline(), CompareMode::Local),
            [("gpu_pass_microseconds", Verdict::Warn)]
        );
        current
            .gpu_pass_microseconds
            .insert("culling_pass".to_owned(), culling(120.5));
        assert_eq!(
            verdicts(&current, &baseline(), CompareMode::Local),
            [("gpu_pass_microseconds", Verdict::Fail)]
        );
    }

    #[test]
    fn another_adapter_or_profile_is_refused_locally() {
        let mut other_adapter = baseline();
        other_adapter.adapter = "llvmpipe".to_owned();
        assert!(compare(&other_adapter, &baseline(), &BUDGETS, CompareMode::Local).is_err());
        let mut debug = baseline();
        debug.profile = "debug".to_owned();
        assert!(compare(&debug, &baseline(), &BUDGETS, CompareMode::Ci).is_err());
    }

    /// Frames of a `configuration` run: draw calls 10, Instances and
    /// triangles drawn 5 and 15 and heap allocations 50 in the counter frames
    /// and more elsewhere, 3 Instances overflowed in frame 300 alone, a
    /// CPU time of 900 µs in the warm-up and 100 µs after it, and on the GPU
    /// a frame of 300 µs whose passes take 200 µs each.
    fn frames(configuration: Configuration) -> Vec<FrameStats> {
        (0..configuration.warm_up_frames + configuration.measured_frames)
            .map(|frame| {
                let counted = COUNTER_FRAMES.contains(&frame);
                let warming_up = frame < configuration.warm_up_frames;
                FrameStats {
                    cpu_frame_time: Duration::from_micros(if warming_up { 900 } else { 100 }),
                    gpu: Some(GpuFrameStats {
                        frame: frame.into(),
                        pass_times: TIMED_PASSES.map(|pass| (pass, Duration::from_micros(200))),
                        frame_time: Duration::from_micros(300),
                        instances_drawn: if counted { 5 } else { 77 },
                        triangles_drawn: if counted { 15 } else { 231 },
                        instances_overflowed: if frame == 300 { 3 } else { 0 },
                    }),
                    draw_calls: if counted { 10 } else { 99 },
                    heap_allocations: if counted { 50 } else { 999 },
                    ..FrameStats::default()
                }
            })
            .collect()
    }

    #[test]
    fn every_run_counts_its_measured_frames_and_the_counter_frames() {
        let measurements = |configuration| {
            measure(
                ReferenceScene::PbrSweep,
                "Apple M4 Pro".to_owned(),
                configuration,
                &frames(configuration),
            )
            .expect("every frame read back its GPU stats")
        };
        let reference = measurements(Configuration::REFERENCE);
        let counted = |counters: Counters| {
            (
                counters.draw_calls,
                counters.instances_drawn,
                counters.triangles_drawn,
                counters.instances_overflowed,
                counters.heap_allocations,
            )
        };
        assert_eq!(counted(reference.counters), (99, 77, 231, 3, 999));
        assert_eq!(
            counted(reference.counter_frames.counters),
            (10, 5, 15, 0, 50)
        );
        assert_eq!(reference.cpu_frame_microseconds.median, 100.0);
        assert_eq!(reference.gpu_frame_microseconds.median, 300.0);
        assert_eq!(reference.gpu_pass_microseconds.len(), TIMED_PASSES.len());
        assert!(
            reference
                .gpu_pass_microseconds
                .values()
                .all(|pass| pass.median == 200.0)
        );
        let counters_only = measurements(Configuration::COUNTERS);
        assert_eq!(counters_only.counter_frames, reference.counter_frames);
    }

    #[test]
    fn a_run_with_a_frame_that_read_back_no_gpu_stats_fails() {
        let configuration = Configuration::COUNTERS;
        let mut frames = frames(configuration);
        frames[40].gpu = None;
        let scene = ReferenceScene::PbrSweep;
        let adapter = "Apple M4 Pro".to_owned();
        assert!(measure(scene, adapter, configuration, &frames).is_err());
    }

    #[test]
    fn a_run_of_another_length_compares_its_counter_frames() {
        let mut shorter = baseline();
        shorter.measured_frames = Configuration::COUNTERS.measured_frames;
        // Its measured frames are other frames than the Baseline's.
        shorter.counters.draw_calls += 1;
        assert_eq!(verdicts(&shorter, &baseline(), CompareMode::Ci), []);
        shorter.counter_frames.counters.uploaded_bytes += 1;
        assert_eq!(
            verdicts(&shorter, &baseline(), CompareMode::Ci),
            [("uploaded_bytes", Verdict::Fail)]
        );
        assert!(compare(&shorter, &baseline(), &BUDGETS, CompareMode::Local).is_err());
        let mut moved = baseline();
        moved.counter_frames.frames = 0..64;
        assert!(compare(&moved, &baseline(), &BUDGETS, CompareMode::Ci).is_err());
        moved.measured_frames = Configuration::COUNTERS.measured_frames;
        assert!(compare(&moved, &baseline(), &BUDGETS, CompareMode::Ci).is_err());
        let mut smaller = baseline();
        smaller.width /= 2;
        assert!(compare(&smaller, &baseline(), &BUDGETS, CompareMode::Ci).is_err());
    }

    #[test]
    fn another_scenes_baseline_is_refused() {
        let mut other_scene = baseline();
        other_scene.scene = "uv-grid".to_owned();
        assert!(compare(&other_scene, &baseline(), &BUDGETS, CompareMode::Ci).is_err());
    }

    #[test]
    fn baseline_directories_are_keyed_by_adapter() {
        assert_eq!(
            baseline_directory("Apple M4 Pro"),
            PathBuf::from("bench/baselines/apple-m4-pro")
        );
        assert_eq!(
            baseline_directory("llvmpipe (LLVM 20.1.2, 256 bits)"),
            PathBuf::from("bench/baselines/llvmpipe-llvm-20-1-2-256-bits")
        );
    }

    #[test]
    fn nearest_rank_percentiles() {
        let values: Vec<f64> = (1..=20).map(f64::from).collect();
        assert_eq!(percentile(values.clone(), 0.5), 10.0);
        assert_eq!(percentile(values.clone(), 0.95), 19.0);
        assert_eq!(percentile(vec![3.0, 1.0, 2.0], 0.5), 2.0);
        assert_eq!(percentile(vec![4.0], 0.95), 4.0);
    }
}
