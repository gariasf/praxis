mod report;

pub use report::report;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;

use crate::capture::{FrameWait, capture, render_frames};
use crate::gpu::GpuContext;
use crate::render::acceleration_structures::DYNAMIC_TLAS_BUILD;
use crate::render::{
    BLAS_REFIT, DYNAMIC_TLAS_RECORDING, DYNAMIC_TLAS_SUBMIT, RenderSwitches, TIMED_PASSES,
};
use crate::scenes::ReferenceScene;
use crate::stats::FrameStats;
use crate::time::SIXTY_HERTZ;

/// The Baselines `compare` reads, `<adapter key>/<scene>.json`: the reference
/// machine's (ADR 0013), CI's lavapipe for counters only, and the comparison
/// adapter's. Keyed by adapter and backend, so no run compares with another
/// machine's or another API's.
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
    /// Whether the run measures `Aliasing`, which costs two more captures.
    pub aliasing: bool,
}

impl Configuration {
    /// ADR 0013 judges Budgets at 1440p. With the warm-up, as many frames as
    /// one run can time on the GPU.
    pub const REFERENCE: Self = Self {
        width: 2560,
        height: 1440,
        warm_up_frames: 248,
        measured_frames: 1800,
        aliasing: true,
    };

    /// CI's lavapipe renders on the CPU, so CI benches at the Goldens' size
    /// and for fewer frames, and without `Aliasing`, whose render at three
    /// times the Goldens' size it is too slow for: `compare --ci` reads
    /// counters only.
    pub const CI: Self = Self {
        width: 640,
        height: 360,
        warm_up_frames: 248,
        measured_frames: 256,
        aliasing: false,
    };

    /// The reference size, rendering the counter frames and no more: the
    /// verify command's bench, whose `compare --ci` reads counters only.
    pub const COUNTERS: Self = Self {
        width: Self::REFERENCE.width,
        height: Self::REFERENCE.height,
        warm_up_frames: COUNTER_FRAMES.start,
        measured_frames: COUNTER_FRAMES.end - COUNTER_FRAMES.start,
        aliasing: true,
    };

    /// CI's size, rendering the counter frames and no more: CI's bench of a
    /// scene too dense for lavapipe to render `CI`'s frames in its time.
    pub const CI_COUNTERS: Self = Self {
        width: Self::CI.width,
        height: Self::CI.height,
        aliasing: Self::CI.aliasing,
        ..Self::COUNTERS
    };
}

/// The frames whose counters every run also records, whatever its length:
/// counters repeat exactly on every run, so runs of different lengths
/// compare on these. By the eighth frame the Scene's uploads and the first
/// frames' allocations are over. Baselines record the frames, so
/// moving them refuses every Baseline until it is re-recorded.
const COUNTER_FRAMES: std::ops::Range<u32> = 8..72;

/// The Baseline directory of `adapter` on `backend`: the adapter's name in
/// lowercase, each run of other characters a hyphen (`Apple M4 Pro` is
/// `apple-m4-pro`), then the backend's name unless it is Vulkan or Metal,
/// which lead the engine's backend order, so an adapter runs on one of them
/// unless `WGPU_BACKEND` asks for another
/// (`NVIDIA GeForce RTX 2070 SUPER` on DX12 is
/// `nvidia-geforce-rtx-2070-super-dx12`).
pub fn baseline_directory(adapter: &str, backend: &str) -> PathBuf {
    let default_backend = [wgpu::Backend::Vulkan, wgpu::Backend::Metal]
        .iter()
        .any(|default| default.to_str() == backend);
    let key = adapter
        .to_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .chain((!default_backend).then_some(backend))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    PathBuf::from(BASELINE_DIRECTORY).join(key)
}

/// A time warns above this ratio to its Baseline and fails above
/// `FAIL_RATIO`; a counter fails on any increase, as it is the same on every
/// run. Noise floor on the M4 Pro, 6 runs of each scene under the owner's
/// usual load: counters identical, GPU medians within 3.4%, CPU medians of
/// 31–42 µs within 8.1 µs.
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
    /// wgpu's name for the backend the adapter ran on (`vulkan`, `dx12`).
    pub backend: String,
    /// `debug` or `release`: allocations differ between the two.
    pub profile: String,
    pub width: u32,
    pub height: u32,
    pub measured_frames: u32,
    pub cpu_frame_microseconds: Spread,
    /// The dynamic TLAS's build on the CPU: its recording and its submit
    /// (ADR 0020), and with the crowd's refit or rebuild form the BLAS
    /// updates' recording and their submission's.
    pub dynamic_tlas_cpu_microseconds: Spread,
    /// Each frame's dynamic TLAS build, plus the span from the start of its
    /// first pass to the end of its last.
    pub gpu_frame_microseconds: Spread,
    /// ADR 0019's gate, with shadow rays off: from the start of culling
    /// phase one to the latest end of the passes up to the lit pass.
    pub gate_microseconds: Spread,
    /// Each frame's GPU frame with shadow rays on, less the same frame's
    /// with them off.
    pub shadow_ray_microseconds: Spread,
    /// Each pass's own GPU time, by pass.
    pub gpu_pass_microseconds: BTreeMap<String, Spread>,
    #[serde(flatten)]
    pub counters: Counters,
    pub counter_frames: CounterFrames,
    /// Measured unless the run skipped it.
    #[serde(flatten)]
    pub aliasing: Option<Aliasing>,
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

/// The aliasing of the Goldens' frame, which is among `COUNTER_FRAMES`:
/// the scene captured at the Goldens' size against the same frame captured
/// at three times it and box-filtered 3x3 in linear light. The reference is
/// the engine's own render, so this measures aliasing (specular sparkle,
/// shimmering edges, terminator steps), not bias. Each pixel's error is the
/// root mean square of its three channels' differences in linear light; the
/// counters are the root mean square of every pixel's error and the 99th
/// percentile pixel error, in millionths, so they compare as integers.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Aliasing {
    pub aliasing_rmse: u64,
    pub aliasing_p99: u64,
}

impl Aliasing {
    /// The aliasing of `image`, RGBA8 sRGB-encoded rows of `width` pixels,
    /// against `supersampled`, the same frame at three times its width and
    /// height.
    pub fn between(image: &[u8], supersampled: &[u8], width: usize) -> Self {
        assert_eq!(
            image.len() * 9,
            supersampled.len(),
            "the supersampled image is not three times the size"
        );
        // The sRGB EOTF, ≤ at the knee [ref:khronos-dfs-srgb-eotf].
        let linear: [f64; 256] = std::array::from_fn(|value| {
            let encoded = value as f64 / 255.0;
            if encoded <= 0.04045 {
                encoded / 12.92
            } else {
                ((encoded + 0.055) / 1.055).powf(2.4)
            }
        });
        let supersampled_width = width * 3;
        let squared_errors: Vec<f64> = image
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .map(|(pixel, color)| {
                let (column, row) = (pixel % width, pixel / width);
                let mut filtered = [0.0; 3];
                for block_row in row * 3..row * 3 + 3 {
                    let start = (block_row * supersampled_width + column * 3) * 4;
                    for sample in supersampled[start..start + 12].as_chunks::<4>().0 {
                        for channel in 0..3 {
                            filtered[channel] += linear[sample[channel] as usize] / 9.0;
                        }
                    }
                }
                (0..3)
                    .map(|channel| (linear[color[channel] as usize] - filtered[channel]).powi(2))
                    .sum::<f64>()
                    / 3.0
            })
            .collect();
        let millionths = |value: f64| (value * 1e6).round() as u64;
        let mean = squared_errors.iter().sum::<f64>() / squared_errors.len() as f64;
        Self {
            aliasing_rmse: millionths(mean.sqrt()),
            aliasing_p99: millionths(percentile(squared_errors, 0.99).sqrt()),
        }
    }

    fn named(&self) -> [(&'static str, u64); 2] {
        [
            ("aliasing_rmse", self.aliasing_rmse),
            ("aliasing_p99", self.aliasing_p99),
        ]
    }
}

/// `scene`'s `Aliasing`, captured as its Golden is, as `switches` set.
fn measure_aliasing(
    context: &GpuContext,
    scene: ReferenceScene,
    switches: RenderSwitches,
) -> anyhow::Result<Aliasing> {
    // CI benches at the Goldens' size.
    let (width, height) = (Configuration::CI.width, Configuration::CI.height);
    let capture = |scale| {
        let (pixels, _) = capture(
            context,
            scene,
            glam::DVec3::ZERO,
            width * scale,
            height * scale,
            switches,
        )?;
        anyhow::Ok(pixels)
    };
    Ok(Aliasing::between(
        &capture(1)?,
        &capture(3)?,
        width as usize,
    ))
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

/// ADR 0013's per-frame limits on the reference machine, ADR 0019's gate
/// and the dynamic TLAS's ceilings.
#[derive(Debug, serde::Deserialize)]
pub struct Budgets {
    pub cpu_frame_microseconds: f64,
    /// The frame's dynamic TLAS build, plus the span from the start of its
    /// first pass to the end of its last.
    pub gpu_frame_microseconds: f64,
    /// The median gate, past which ADR 0019's visibility pre-pass and
    /// visibility buffer are prototyped.
    pub gate_microseconds: f64,
    /// Ceilings on the dynamic TLAS at its 10,000 Moving Instances (ADR
    /// 0020): its build's recording and submit on the CPU, and its build on
    /// the GPU, from the end of the frame before.
    pub dynamic_tlas_cpu_microseconds: f64,
    pub dynamic_tlas_gpu_microseconds: f64,
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
/// `switches` set, and again with shadow rays off, and returns the measured
/// frames' medians and p95s, and the counter frames' counters.
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
    // A software rasterizer's threads would share the CPU with the spin.
    let wait = match context.adapter.get_info().device_type {
        wgpu::DeviceType::Cpu => FrameWait::Block,
        _ => FrameWait::Spin,
    };
    let render = |switches| {
        let (_, frames) = render_frames(
            context,
            scene,
            glam::DVec3::ZERO,
            configuration.width,
            configuration.height,
            0..frame_count,
            &[],
            // ADR 0022 judges Budgets at 60 Hz.
            SIXTY_HERTZ,
            switches,
            wait,
        )?;
        anyhow::Ok(frames)
    };
    let frames = render(switches)?;
    let frames_without_rays = render(RenderSwitches {
        shadow_rays: false,
        ..switches
    })?;
    let adapter = context.adapter.get_info();
    let measurements = measure(
        scene,
        adapter.name,
        adapter.backend,
        configuration,
        &frames,
        &frames_without_rays,
    )?;
    let aliasing = configuration
        .aliasing
        .then(|| measure_aliasing(context, scene, switches))
        .transpose()?;
    Ok(Measurements {
        aliasing,
        ..measurements
    })
}

/// The Measurements of `frames`, a `configuration` run of `scene` on
/// `adapter` through `backend`, and of `frames_without_rays`, the same run
/// with shadow rays off, which fail unless every frame of both read back
/// its GPU stats.
fn measure(
    scene: ReferenceScene,
    adapter: String,
    backend: wgpu::Backend,
    configuration: Configuration,
    frames: &[FrameStats],
    frames_without_rays: &[FrameStats],
) -> anyhow::Result<Measurements> {
    let gpu_stats = |frames: &[FrameStats]| {
        let gpu_frames: Vec<_> = frames.iter().filter_map(|stats| stats.gpu).collect();
        anyhow::ensure!(
            gpu_frames.len() == frames.len(),
            "{} of {} frames read back no GPU stats",
            frames.len() - gpu_frames.len(),
            frames.len()
        );
        Ok(gpu_frames)
    };
    let (gpu_frames, gpu_frames_without_rays) =
        (gpu_stats(frames)?, gpu_stats(frames_without_rays)?);
    let warm_up_frames = configuration.warm_up_frames as usize;
    let (measured, measured_gpu) = (&frames[warm_up_frames..], &gpu_frames[warm_up_frames..]);
    let measured_without_rays = &gpu_frames_without_rays[warm_up_frames..];
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
        backend: backend.to_str().to_owned(),
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
        dynamic_tlas_cpu_microseconds: spread(
            measured
                .iter()
                .map(|stats| {
                    let stages = stats.cpu_times.iter().filter(|(stage, _)| {
                        [BLAS_REFIT, DYNAMIC_TLAS_RECORDING, DYNAMIC_TLAS_SUBMIT]
                            .contains(&stage.as_str())
                    });
                    stages.map(|&(_, time)| microseconds(time)).sum()
                })
                .collect(),
        ),
        gpu_frame_microseconds: spread(
            measured_gpu
                .iter()
                .map(|gpu| microseconds(gpu.frame_time))
                .collect(),
        ),
        gate_microseconds: spread(
            measured_without_rays
                .iter()
                .map(|gpu| microseconds(gpu.gate_time))
                .collect(),
        ),
        shadow_ray_microseconds: spread(
            measured_gpu
                .iter()
                .zip(measured_without_rays)
                .map(|(with, without)| {
                    microseconds(with.frame_time) - microseconds(without.frame_time)
                })
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
        aliasing: None,
    })
}

/// One line, `key=value` pairs after the scene name: medians and p95s of
/// the CPU and GPU frame, the gate and shadow-ray time, each pass's median,
/// then the counters.
impl std::fmt::Display for Measurements {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (cpu, gpu) = (self.cpu_frame_microseconds, self.gpu_frame_microseconds);
        write!(
            formatter,
            "{} cpu_us median={:.1} p95={:.1} dynamic_tlas_cpu_us={:.1} gpu_us median={:.1} p95={:.1}",
            self.scene,
            cpu.median,
            cpu.p95,
            self.dynamic_tlas_cpu_microseconds.median,
            gpu.median,
            gpu.p95
        )?;
        write!(
            formatter,
            " gate_us median={:.1} p95={:.1} shadow_ray_us median={:.1} p95={:.1}",
            self.gate_microseconds.median,
            self.gate_microseconds.p95,
            self.shadow_ray_microseconds.median,
            self.shadow_ray_microseconds.p95
        )?;
        for (pass, time) in &self.gpu_pass_microseconds {
            write!(formatter, " {pass}={:.1}", time.median)?;
        }
        let aliasing = self.aliasing.iter().flat_map(Aliasing::named);
        for (counter, value) in self.counters.named().into_iter().chain(aliasing) {
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
    ensure_comparable(current, baseline)?;
    let (counters, baseline_counters) = compared_counters(current, baseline);
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
    check_aliasing(&mut findings, current.aliasing, baseline.aliasing);
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
        (&current.adapter, &current.backend) == (&baseline.adapter, &baseline.backend),
        "{} ran on {:?} through {}, its Baseline on {:?} through {}: compare times on the \
         Baseline's machine and backend, or counters only with --ci",
        current.scene,
        current.adapter,
        current.backend,
        baseline.adapter,
        baseline.backend
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
    let gate = current.gate_microseconds.median;
    let noise_floor = GPU_PASS_NOISE_FLOOR_MICROSECONDS;
    for (metric, current, baseline) in [
        ("gate_microseconds", gate, baseline.gate_microseconds.median),
        (
            "shadow_ray_microseconds",
            current.shadow_ray_microseconds.median,
            baseline.shadow_ray_microseconds.median,
        ),
    ] {
        check_regression(&mut findings, metric, current, baseline, noise_floor);
    }
    check_budget(
        &mut findings,
        "gate_microseconds",
        gate,
        budgets.gate_microseconds,
    );
    check_budget(
        &mut findings,
        "dynamic_tlas_cpu_microseconds",
        current.dynamic_tlas_cpu_microseconds.median,
        budgets.dynamic_tlas_cpu_microseconds,
    );
    if let Some(build) = passes.get(DYNAMIC_TLAS_BUILD) {
        check_budget(
            &mut findings,
            "dynamic_tlas_gpu_microseconds",
            build.median,
            budgets.dynamic_tlas_gpu_microseconds,
        );
    }
    Ok(findings)
}

/// Fails unless `baseline` ran `current`'s scene in its profile at its size,
/// counting the same counter frames.
fn ensure_comparable(current: &Measurements, baseline: &Measurements) -> anyhow::Result<()> {
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
    Ok(())
}

/// The counters `current` and `baseline` compare: their measured frames'
/// when both measured as many frames, their counter frames' otherwise.
fn compared_counters(current: &Measurements, baseline: &Measurements) -> (Counters, Counters) {
    if current.measured_frames == baseline.measured_frames {
        (current.counters, baseline.counters)
    } else {
        (
            current.counter_frames.counters,
            baseline.counter_frames.counters,
        )
    }
}

/// Judges `current`'s aliasing against `baseline`'s by the ratios times
/// answer to, with no noise floor: on the M4 Pro two runs of every
/// Reference scene measured equal counters, so a rise past the ratios is a
/// change of the image. A run that skipped it passes, and one that measured it
/// against a Baseline without it fails.
fn check_aliasing(
    findings: &mut Vec<Finding>,
    current: Option<Aliasing>,
    baseline: Option<Aliasing>,
) {
    let Some(current) = current else {
        return;
    };
    let Some(baseline) = baseline else {
        findings.extend(current.named().map(|(metric, _)| Finding {
            metric,
            verdict: Verdict::Fail,
            detail: "measured, and its Baseline has no value: re-record the Baseline".to_owned(),
        }));
        return;
    };
    for ((metric, current), (_, baseline)) in current.named().into_iter().zip(baseline.named()) {
        let (current, baseline) = (current as f64, baseline as f64);
        check_regression(findings, metric, current, baseline, 0.0);
    }
}

/// Warns when the time `current` exceeds `baseline` by more than
/// `WARN_RATIO`, and fails above `FAIL_RATIO` when it also grew by more than
/// `noise_floor`. A time that did not grow passes, whatever its sign.
fn check_regression(
    findings: &mut Vec<Finding>,
    metric: &'static str,
    current: f64,
    baseline: f64,
    noise_floor: f64,
) {
    if current <= baseline {
        return;
    }
    // From a Baseline at or below zero, any growth is past every ratio.
    let ratio = if baseline > 0.0 {
        current / baseline
    } else {
        f64::INFINITY
    };
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
            backend: "metal".to_owned(),
            profile: "release".to_owned(),
            width: Configuration::REFERENCE.width,
            height: Configuration::REFERENCE.height,
            measured_frames: Configuration::REFERENCE.measured_frames,
            cpu_frame_microseconds: Spread {
                median: 1000.0,
                p95: 1200.0,
            },
            dynamic_tlas_cpu_microseconds: Spread {
                median: 100.0,
                p95: 120.0,
            },
            gpu_frame_microseconds: Spread {
                median: 2500.0,
                p95: 2600.0,
            },
            gate_microseconds: Spread {
                median: 1500.0,
                p95: 1600.0,
            },
            shadow_ray_microseconds: Spread {
                median: 500.0,
                p95: 600.0,
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
            aliasing: None,
        }
    }

    const BUDGETS: Budgets = Budgets {
        cpu_frame_microseconds: 6000.0,
        gpu_frame_microseconds: 12000.0,
        gate_microseconds: 6000.0,
        dynamic_tlas_cpu_microseconds: 1100.0,
        dynamic_tlas_gpu_microseconds: 1000.0,
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

    /// The gate answers to ADR 0019's 6 ms and to its Baseline, and
    /// shadow-ray time to its Baseline.
    #[test]
    fn the_gate_fails_over_6_ms_and_both_lines_against_their_baselines() {
        let mut slower = baseline();
        slower.gate_microseconds.median = 1700.0;
        slower.shadow_ray_microseconds.median = 600.0;
        assert_eq!(
            verdicts(&slower, &baseline(), CompareMode::Local),
            [
                ("gate_microseconds", Verdict::Fail),
                ("shadow_ray_microseconds", Verdict::Fail)
            ]
        );
        let mut over_budget = baseline();
        over_budget.gate_microseconds.median = 6100.0;
        assert_eq!(
            verdicts(&over_budget, &over_budget, CompareMode::Local),
            [("gate_microseconds", Verdict::Fail)]
        );
        assert_eq!(verdicts(&over_budget, &over_budget, CompareMode::Ci), []);
    }

    /// The dynamic TLAS's CPU cost and its build answer to their ceilings,
    /// whatever their Baselines.
    #[test]
    fn the_dynamic_tlas_fails_over_its_ceilings() {
        let mut over = baseline();
        over.dynamic_tlas_cpu_microseconds.median = 1200.0;
        over.gpu_pass_microseconds.insert(
            DYNAMIC_TLAS_BUILD.to_owned(),
            Spread {
                median: 1100.0,
                p95: 1200.0,
            },
        );
        assert_eq!(
            verdicts(&over, &over, CompareMode::Local),
            [
                ("dynamic_tlas_cpu_microseconds", Verdict::Fail),
                ("dynamic_tlas_gpu_microseconds", Verdict::Fail)
            ]
        );
        assert_eq!(verdicts(&over, &over, CompareMode::Ci), []);
    }

    /// Shadow-ray time is a difference of two runs, so near zero it can be
    /// negative: less of it is no regression, and more of it from below
    /// zero is judged by its growth.
    #[test]
    fn less_shadow_ray_time_passes_below_zero() {
        let mut none = baseline();
        none.shadow_ray_microseconds.median = -10.0;
        let mut current = none.clone();
        current.shadow_ray_microseconds.median = -40.0;
        assert_eq!(verdicts(&current, &none, CompareMode::Local), []);
        current.shadow_ray_microseconds.median = 600.0;
        assert_eq!(
            verdicts(&current, &none, CompareMode::Local),
            [("shadow_ray_microseconds", Verdict::Fail)]
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
        let mut other_backend = baseline();
        other_backend.backend = "vulkan".to_owned();
        assert!(compare(&other_backend, &baseline(), &BUDGETS, CompareMode::Local).is_err());
        let mut debug = baseline();
        debug.profile = "debug".to_owned();
        assert!(compare(&debug, &baseline(), &BUDGETS, CompareMode::Ci).is_err());
    }

    /// Frames of a `configuration` run: draw calls 10, Instances and
    /// triangles drawn 5 and 15 and heap allocations 50 in the counter frames
    /// and more elsewhere, 3 Instances overflowed in frame 300 alone, a
    /// CPU time of 900 µs in the warm-up and 100 µs after it, 20 µs of it
    /// refitting BLASes, 30 µs recording the dynamic TLAS's build and 10 µs
    /// submitting it, and on the GPU
    /// a frame of 300 µs whose passes take 200 µs each, 150 µs of it the
    /// gate.
    fn frames(configuration: Configuration) -> Vec<FrameStats> {
        (0..configuration.warm_up_frames + configuration.measured_frames)
            .map(|frame| {
                let counted = COUNTER_FRAMES.contains(&frame);
                let warming_up = frame < configuration.warm_up_frames;
                FrameStats {
                    cpu_frame_time: Duration::from_micros(if warming_up { 900 } else { 100 }),
                    cpu_times: [
                        ("prepare_frame", 50),
                        (BLAS_REFIT, 20),
                        (DYNAMIC_TLAS_RECORDING, 30),
                        (DYNAMIC_TLAS_SUBMIT, 10),
                    ]
                    .map(|(stage, time)| (stage.to_owned(), Duration::from_micros(time)))
                    .into(),
                    gpu: Some(GpuFrameStats {
                        frame: frame.into(),
                        pass_times: TIMED_PASSES.map(|pass| (pass, Duration::from_micros(200))),
                        frame_time: Duration::from_micros(300),
                        gate_time: Duration::from_micros(150),
                        instances_drawn: if counted { 5 } else { 77 },
                        triangles_drawn: if counted { 15 } else { 231 },
                        phase_two_triangles_drawn: 0,
                        instances_overflowed: if frame == 300 { 3 } else { 0 },
                        froxel_lights_overflowed: 0,
                        shadow_mask_lights_overflowed: 0,
                        lit_pixels: 0,
                        listed_point_lights: 0,
                        reaching_point_lights: 0,
                    }),
                    draw_calls: if counted { 10 } else { 99 },
                    heap_allocations: if counted { 50 } else { 999 },
                    ..FrameStats::default()
                }
            })
            .collect()
    }

    /// `frames`, with shadow rays off: a GPU frame of 250 µs in the measured
    /// frames, of 900 µs in the warm-up, and a gate of 140 µs.
    fn frames_without_rays(configuration: Configuration) -> Vec<FrameStats> {
        let mut frames = frames(configuration);
        for (frame, stats) in frames.iter_mut().enumerate() {
            let gpu = stats.gpu.as_mut().expect("every frame has GPU stats");
            let warming_up = frame < configuration.warm_up_frames as usize;
            gpu.frame_time = Duration::from_micros(if warming_up { 900 } else { 250 });
            gpu.gate_time = Duration::from_micros(140);
        }
        frames
    }

    #[test]
    fn every_run_counts_its_measured_frames_and_the_counter_frames() {
        let measurements = |configuration| {
            measure(
                ReferenceScene::PbrSweep,
                "Apple M4 Pro".to_owned(),
                wgpu::Backend::Metal,
                configuration,
                &frames(configuration),
                &frames_without_rays(configuration),
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
        assert_eq!(reference.dynamic_tlas_cpu_microseconds.median, 60.0);
        assert_eq!(reference.gpu_frame_microseconds.median, 300.0);
        // Both lines read the measured frames alone, the gate with rays off.
        assert_eq!(reference.gate_microseconds.median, 140.0);
        assert_eq!(reference.shadow_ray_microseconds.median, 50.0);
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
        let without_rays = frames_without_rays(configuration);
        let mut frames = frames(configuration);
        frames[40].gpu = None;
        let scene = ReferenceScene::PbrSweep;
        let adapter = "Apple M4 Pro".to_owned();
        assert!(
            measure(
                scene,
                adapter.clone(),
                wgpu::Backend::Metal,
                configuration,
                &frames,
                &without_rays
            )
            .is_err()
        );
        assert!(
            measure(
                scene,
                adapter,
                wgpu::Backend::Metal,
                configuration,
                &without_rays,
                &frames
            )
            .is_err()
        );
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
    fn baseline_directories_are_keyed_by_adapter_and_any_backend_but_vulkan_or_metal() {
        assert_eq!(
            baseline_directory("Apple M4 Pro", "metal"),
            PathBuf::from("bench/baselines/apple-m4-pro")
        );
        assert_eq!(
            baseline_directory("llvmpipe (LLVM 20.1.2, 256 bits)", "vulkan"),
            PathBuf::from("bench/baselines/llvmpipe-llvm-20-1-2-256-bits")
        );
        assert_eq!(
            baseline_directory("NVIDIA GeForce RTX 2070 SUPER", "vulkan"),
            PathBuf::from("bench/baselines/nvidia-geforce-rtx-2070-super")
        );
        assert_eq!(
            baseline_directory("NVIDIA GeForce RTX 2070 SUPER", "dx12"),
            PathBuf::from("bench/baselines/nvidia-geforce-rtx-2070-super-dx12")
        );
    }

    /// An RGBA8 image of `width` by `height` pixels of grey `value`.
    fn grey(value: u8, width: usize, height: usize) -> Vec<u8> {
        [value, value, value, 255].repeat(width * height)
    }

    #[test]
    fn an_image_matching_its_supersampled_render_has_no_aliasing() {
        let aliasing = Aliasing::between(&grey(188, 2, 1), &grey(188, 6, 3), 2);
        assert_eq!(
            aliasing,
            Aliasing {
                aliasing_rmse: 0,
                aliasing_p99: 0
            }
        );
    }

    /// The 3x render is filtered in linear light: a block of three white and
    /// six black pixels averages to a third, where averaging the encoded
    /// bytes, to 85, would decode to 0.0908.
    #[test]
    fn the_supersampled_render_is_box_filtered_in_linear_light() {
        let mut supersampled = grey(0, 3, 3);
        supersampled[..3 * 4].copy_from_slice(&grey(255, 3, 1));
        let aliasing = Aliasing::between(&grey(0, 1, 1), &supersampled, 1);
        assert_eq!(aliasing.aliasing_rmse, 333_333);
        assert_eq!(aliasing.aliasing_p99, 333_333);
    }

    /// Of two pixels, one black against a white block (an error of 1 in each
    /// channel) and one exact: the RMSE is the root of half of 1, the 99th
    /// percentile the worse pixel. Alpha is no channel of the error.
    #[test]
    fn the_rmse_spans_every_pixel_and_the_p99_the_worst_ones() {
        let mut image = grey(255, 2, 1);
        image[..4].copy_from_slice(&[0, 0, 0, 0]);
        let aliasing = Aliasing::between(&image, &grey(255, 6, 3), 2);
        assert_eq!(
            aliasing,
            Aliasing {
                aliasing_rmse: 707_107,
                aliasing_p99: 1_000_000
            }
        );
    }

    const ALIASING: Aliasing = Aliasing {
        aliasing_rmse: 20_000,
        aliasing_p99: 150_000,
    };

    fn with_aliasing(aliasing: Option<Aliasing>) -> Measurements {
        Measurements {
            aliasing,
            ..baseline()
        }
    }

    #[test]
    fn aliasing_fails_past_its_tolerance_against_its_baseline() {
        let baseline = with_aliasing(Some(ALIASING));
        let mut current = baseline.clone();
        assert_eq!(verdicts(&current, &baseline, CompareMode::Ci), []);
        current.aliasing = Some(Aliasing {
            aliasing_rmse: 23_000,
            aliasing_p99: 160_000,
        });
        assert_eq!(
            verdicts(&current, &baseline, CompareMode::Ci),
            [
                ("aliasing_rmse", Verdict::Fail),
                ("aliasing_p99", Verdict::Warn)
            ]
        );
    }

    /// A run that skipped the aliasing counters compares the rest; a run that
    /// measured them against a Baseline without them fails until the
    /// Baseline is re-recorded.
    #[test]
    fn skipped_aliasing_passes_and_aliasing_without_a_baseline_fails() {
        let measured = with_aliasing(Some(ALIASING));
        let skipped = with_aliasing(None);
        assert_eq!(verdicts(&skipped, &measured, CompareMode::Ci), []);
        let findings = compare(&measured, &skipped, &BUDGETS, CompareMode::Ci).expect("comparable");
        assert_eq!(
            findings
                .iter()
                .map(|finding| (finding.metric, finding.verdict))
                .collect::<Vec<_>>(),
            [
                ("aliasing_rmse", Verdict::Fail),
                ("aliasing_p99", Verdict::Fail)
            ]
        );
        assert!(findings[0].detail.contains("re-record"), "{}", findings[0]);
    }

    #[test]
    fn aliasing_is_written_only_when_measured() {
        let skipped = serde_json::to_string(&with_aliasing(None)).expect("serializable");
        assert!(!skipped.contains("aliasing"), "{skipped}");
        let measured = with_aliasing(Some(ALIASING));
        let json = serde_json::to_vec(&measured).expect("serializable");
        let parsed = Measurements::from_baseline_json(&json).expect("the current form");
        assert_eq!(parsed, measured);
        let parsed = Measurements::from_baseline_json(skipped.as_bytes()).expect("skipped");
        assert_eq!(parsed.aliasing, None);
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
