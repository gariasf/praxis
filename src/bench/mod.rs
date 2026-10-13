use std::time::Duration;

use crate::capture::render_frames;
use crate::gpu::GpuContext;
use crate::scenes::ReferenceScene;
use crate::state::MAIN_PASS;
use crate::stats::FrameStats;

/// ADR 0013 judges Budgets at 1440p.
const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1440;

/// Frames rendered and discarded before measuring, while the first frames'
/// allocations and the GPU's clocks settle.
const WARM_UP_FRAMES: u32 = 248;

/// Frames measured after the warm-up; with the warm-up, as many as one run
/// can time on the GPU.
const MEASURED_FRAMES: u32 = 1800;

/// The steady-state cost of one Reference scene: medians over the measured
/// frames, plus p95 for times. Bench writes it as JSON.
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
    /// `None` when the device has no timestamp queries.
    pub gpu_main_pass_microseconds: Option<Spread>,
    pub draw_calls: u64,
    pub uploaded_bytes: u64,
    pub gpu_memory_bytes: u64,
    pub heap_allocations: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Spread {
    pub median: f64,
    pub p95: f64,
}

/// Renders `scene` at 1440p for the warm-up and measured frames and returns
/// the measured frames' medians and p95s.
pub fn run(context: &GpuContext, scene: ReferenceScene) -> anyhow::Result<Measurements> {
    let (_, frames) = render_frames(
        context,
        scene,
        WIDTH,
        HEIGHT,
        WARM_UP_FRAMES + MEASURED_FRAMES,
    )?;
    let measured = &frames[WARM_UP_FRAMES as usize..];
    let microseconds = |time: Duration| time.as_secs_f64() * 1e6;
    let main_pass_times: Vec<f64> = measured
        .iter()
        .filter_map(|stats| {
            let (_, time) = stats
                .gpu_times
                .iter()
                .find(|(pass, _)| *pass == MAIN_PASS)?;
            Some(microseconds(*time))
        })
        .collect();
    let counter = |value: fn(&FrameStats) -> u64| {
        percentile(
            measured.iter().map(|stats| value(stats) as f64).collect(),
            0.5,
        ) as u64
    };
    Ok(Measurements {
        scene: scene.name().to_owned(),
        adapter: context.adapter.get_info().name,
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_owned(),
        width: WIDTH,
        height: HEIGHT,
        measured_frames: MEASURED_FRAMES,
        cpu_frame_microseconds: spread(
            measured
                .iter()
                .map(|stats| microseconds(stats.cpu_frame_time))
                .collect(),
        ),
        gpu_main_pass_microseconds: (main_pass_times.len() == measured.len())
            .then(|| spread(main_pass_times)),
        draw_calls: counter(|stats| u64::from(stats.draw_calls)),
        uploaded_bytes: counter(|stats| stats.uploaded_bytes),
        gpu_memory_bytes: counter(|stats| stats.gpu_memory_bytes),
        heap_allocations: counter(|stats| stats.heap_allocations),
    })
}

/// One line, `key=value` pairs after the scene name.
impl std::fmt::Display for Measurements {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} cpu_us median={:.1} p95={:.1}",
            self.scene, self.cpu_frame_microseconds.median, self.cpu_frame_microseconds.p95
        )?;
        if let Some(gpu) = self.gpu_main_pass_microseconds {
            write!(
                formatter,
                " gpu_us median={:.1} p95={:.1}",
                gpu.median, gpu.p95
            )?;
        }
        write!(
            formatter,
            " draw_calls={} uploaded_bytes={} gpu_memory_bytes={} heap_allocations={}",
            self.draw_calls, self.uploaded_bytes, self.gpu_memory_bytes, self.heap_allocations
        )
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

    #[test]
    fn nearest_rank_percentiles() {
        let values: Vec<f64> = (1..=20).map(f64::from).collect();
        assert_eq!(percentile(values.clone(), 0.5), 10.0);
        assert_eq!(percentile(values.clone(), 0.95), 19.0);
        assert_eq!(percentile(vec![3.0, 1.0, 2.0], 0.5), 2.0);
        assert_eq!(percentile(vec![4.0], 0.95), 4.0);
    }
}
