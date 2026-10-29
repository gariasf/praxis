//! The markdown tables PRs and `docs/performance.md` quote from bench
//! results, so no figure is retyped by hand.

use std::fmt::Write;

use crate::render::acceleration_structures::DYNAMIC_TLAS_BUILD;

use super::{Budgets, Measurements, Spread, compared_counters, ensure_comparable};

/// `current`'s times and counters, its aliasing among them when it was
/// measured, as markdown tables, each against its
/// `baseline`, when there is one, and the Budgets. Times compare only with a
/// Baseline of as many frames on the same adapter, as `compare` judges them,
/// each pass with the Baseline's pass of its name; counters compare as
/// `compare` reads them. A Baseline of another scene,
/// profile, size or counter frames is refused.
pub fn report(
    current: &Measurements,
    baseline: Option<&Measurements>,
    budgets: &Budgets,
) -> anyhow::Result<String> {
    if let Some(baseline) = baseline {
        ensure_comparable(current, baseline)?;
    }
    let times_baseline = baseline.filter(|baseline| {
        (&baseline.adapter, baseline.measured_frames) == (&current.adapter, current.measured_frames)
    });
    let mut report = format!(
        "**{}** on {}, {}, {}x{}, {} measured frames",
        current.scene,
        current.adapter,
        current.profile,
        current.width,
        current.height,
        current.measured_frames
    );
    if let Some(baseline) = baseline {
        if times_baseline.is_none() {
            write!(
                report,
                "; its Baseline measured {} frames on {}, so times compare with none",
                baseline.measured_frames, baseline.adapter
            )?;
        }
        if baseline.measured_frames != current.measured_frames {
            write!(
                report,
                "; counters of frames {:?}",
                current.counter_frames.frames
            )?;
        }
    }

    report.push_str(
        "\n\n| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|\n",
    );
    let frame_times = [
        (
            "CPU frame",
            current.cpu_frame_microseconds,
            times_baseline.map(|baseline| baseline.cpu_frame_microseconds),
            Some(budgets.cpu_frame_microseconds),
        ),
        (
            "Dynamic TLAS, CPU",
            current.dynamic_tlas_cpu_microseconds,
            times_baseline.map(|baseline| baseline.dynamic_tlas_cpu_microseconds),
            Some(budgets.dynamic_tlas_cpu_microseconds),
        ),
        (
            "GPU frame",
            current.gpu_frame_microseconds,
            times_baseline.map(|baseline| baseline.gpu_frame_microseconds),
            Some(budgets.gpu_frame_microseconds),
        ),
        (
            "Gate, rays off",
            current.gate_microseconds,
            times_baseline.map(|baseline| baseline.gate_microseconds),
            Some(budgets.gate_microseconds),
        ),
        (
            "Shadow-ray time",
            current.shadow_ray_microseconds,
            times_baseline.map(|baseline| baseline.shadow_ray_microseconds),
            None,
        ),
    ];
    let pass_times = current.gpu_pass_microseconds.iter().map(|(pass, &time)| {
        let baseline_time = times_baseline
            .and_then(|baseline| baseline.gpu_pass_microseconds.get(pass))
            .copied();
        let budget = (pass == DYNAMIC_TLAS_BUILD).then_some(budgets.dynamic_tlas_gpu_microseconds);
        (format!("`{pass}`"), time, baseline_time, budget)
    });
    let times = frame_times
        .map(|(name, time, baseline_time, budget)| (name.to_owned(), time, baseline_time, budget))
        .into_iter()
        .chain(pass_times);
    for (name, time, baseline_time, budget) in times {
        let compared = match baseline_time {
            Some(Spread { median, p95 }) => [
                grouped(median, 1),
                grouped(p95, 1),
                signed(time.median - median, 1),
                ratio(time.median, median),
            ],
            None => std::array::from_fn(|_| "—".to_owned()),
        };
        let [baseline_median, baseline_p95, delta, ratio_text] = compared;
        writeln!(
            report,
            "| {name} | {} | {} | {baseline_median} | {baseline_p95} | {delta} | {ratio_text} | {} |",
            grouped(time.median, 1),
            grouped(time.p95, 1),
            budget.map_or("—".to_owned(), |budget| grouped(budget, 1))
        )?;
    }

    report.push_str(
        "\n| Counter | Value | Baseline | Δ | Ratio | Budget |\n\
         |---|---:|---:|---:|---:|---:|\n",
    );
    let (counters, baseline_counters) = match baseline {
        Some(baseline) => {
            let (counters, baseline_counters) = compared_counters(current, baseline);
            (counters, Some(baseline_counters))
        }
        None => (current.counters, None),
    };
    let baseline_values = baseline_counters.map_or([None; 8], |baseline_counters| {
        baseline_counters.named().map(|(_, value)| Some(value))
    });
    let baseline_aliasing = baseline.and_then(|baseline| baseline.aliasing);
    let aliasing = current.aliasing.iter().flat_map(|aliasing| {
        let baseline_values = baseline_aliasing.map_or([None; 2], |baseline_aliasing| {
            baseline_aliasing.named().map(|(_, value)| Some(value))
        });
        aliasing.named().into_iter().zip(baseline_values)
    });
    let rows = counters.named().into_iter().zip(baseline_values);
    for ((counter, value), baseline_value) in rows.chain(aliasing) {
        let value = value as f64;
        let [baseline_value, delta, ratio_text] = match baseline_value {
            Some(baseline_value) => {
                let baseline_value = baseline_value as f64;
                [
                    grouped(baseline_value, 0),
                    signed(value - baseline_value, 0),
                    ratio(value, baseline_value),
                ]
            }
            None => std::array::from_fn(|_| "—".to_owned()),
        };
        let budget = match counter {
            "gpu_memory_bytes" => grouped(budgets.gpu_memory_bytes as f64, 0),
            _ => "—".to_owned(),
        };
        writeln!(
            report,
            "| `{counter}` | {} | {baseline_value} | {delta} | {ratio_text} | {budget} |",
            grouped(value, 0)
        )?;
    }
    Ok(report)
}

/// `value` to `decimals` places, its whole part in groups of three digits
/// (`4,208.6`), and a minus sign only when a digit it shows is nonzero.
fn grouped(value: f64, decimals: usize) -> String {
    let digits = format!("{:.decimals$}", value.abs());
    let (whole, fraction) = digits.split_at(digits.find('.').unwrap_or(digits.len()));
    let mut text = String::new();
    if value < 0.0 && digits.bytes().any(|digit| matches!(digit, b'1'..=b'9')) {
        text.push('-');
    }
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            text.push(',');
        }
        text.push(digit);
    }
    text + fraction
}

/// `grouped`, with a plus sign unless it shows a minus.
fn signed(value: f64, decimals: usize) -> String {
    let text = grouped(value, decimals);
    if text.starts_with('-') {
        text
    } else {
        format!("+{text}")
    }
}

/// `current` over `baseline` to three places, or a dash over a Baseline at
/// or below zero, where a ratio means nothing.
fn ratio(current: f64, baseline: f64) -> String {
    if baseline <= 0.0 {
        "—".to_owned()
    } else {
        format!("{:.3}", current / baseline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::{Aliasing, CounterFrames, Counters, Spread};

    const COUNTERS: Counters = Counters {
        draw_calls: 4,
        instances_drawn: 11,
        triangles_drawn: 256_938,
        instances_overflowed: 0,
        uploaded_bytes: 384,
        gpu_memory_bytes: 536_471_449,
        arena_bytes_in_use: 844_804,
        heap_allocations: 326,
    };

    const BUDGETS: Budgets = Budgets {
        cpu_frame_microseconds: 6000.0,
        gpu_frame_microseconds: 12000.0,
        gate_microseconds: 6000.0,
        dynamic_tlas_cpu_microseconds: 1100.0,
        dynamic_tlas_gpu_microseconds: 1000.0,
        gpu_memory_bytes: 8_000_000_000,
    };

    fn spread(median: f64, p95: f64) -> Spread {
        Spread { median, p95 }
    }

    fn measurements(
        cpu: Spread,
        gpu: Spread,
        gate: Spread,
        passes: [(&str, Spread); 3],
        counters: Counters,
    ) -> Measurements {
        Measurements {
            scene: "helmet".to_owned(),
            adapter: "Apple M4 Pro".to_owned(),
            backend: "metal".to_owned(),
            profile: "release".to_owned(),
            width: 2560,
            height: 1440,
            measured_frames: 1800,
            cpu_frame_microseconds: cpu,
            dynamic_tlas_cpu_microseconds: spread(0.0, 0.1),
            gpu_frame_microseconds: gpu,
            gate_microseconds: gate,
            shadow_ray_microseconds: spread(-3.2, 40.5),
            gpu_pass_microseconds: passes
                .into_iter()
                .map(|(pass, time)| (pass.to_owned(), time))
                .collect(),
            counters,
            counter_frames: CounterFrames {
                frames: 8..72,
                counters,
            },
            aliasing: None,
        }
    }

    fn current() -> Measurements {
        measurements(
            spread(190.0, 262.3),
            spread(4208.6, 5204.5),
            spread(2950.0, 3010.4),
            [
                ("culling_phase_one", spread(166.3, 169.8)),
                ("dynamic_tlas_build", spread(48.7, 462.9)),
                ("lit_pass", spread(755.8, 768.1)),
            ],
            COUNTERS,
        )
    }

    fn baseline() -> Measurements {
        measurements(
            spread(200.0, 250.0),
            spread(4000.0, 5000.0),
            spread(3000.0, 3100.0),
            [
                ("culling_phase_one", spread(160.0, 170.0)),
                ("dynamic_tlas_build", spread(50.0, 400.0)),
                ("lit_pass", spread(800.0, 810.0)),
            ],
            Counters {
                gpu_memory_bytes: 536_000_000,
                heap_allocations: 300,
                ..COUNTERS
            },
        )
    }

    #[test]
    fn a_run_renders_against_its_baseline_and_the_budgets() {
        let report = report(&current(), Some(&baseline()), &BUDGETS).expect("comparable");
        assert_eq!(
            report,
            "\
**helmet** on Apple M4 Pro, release, 2560x1440, 1800 measured frames

| Time, µs | Median | p95 | Baseline median | Baseline p95 | Δ median | Ratio | Budget |
|---|---:|---:|---:|---:|---:|---:|---:|
| CPU frame | 190.0 | 262.3 | 200.0 | 250.0 | -10.0 | 0.950 | 6,000.0 |
| Dynamic TLAS, CPU | 0.0 | 0.1 | 0.0 | 0.1 | +0.0 | — | 1,100.0 |
| GPU frame | 4,208.6 | 5,204.5 | 4,000.0 | 5,000.0 | +208.6 | 1.052 | 12,000.0 |
| Gate, rays off | 2,950.0 | 3,010.4 | 3,000.0 | 3,100.0 | -50.0 | 0.983 | 6,000.0 |
| Shadow-ray time | -3.2 | 40.5 | -3.2 | 40.5 | +0.0 | — | — |
| `culling_phase_one` | 166.3 | 169.8 | 160.0 | 170.0 | +6.3 | 1.039 | — |
| `dynamic_tlas_build` | 48.7 | 462.9 | 50.0 | 400.0 | -1.3 | 0.974 | 1,000.0 |
| `lit_pass` | 755.8 | 768.1 | 800.0 | 810.0 | -44.2 | 0.945 | — |

| Counter | Value | Baseline | Δ | Ratio | Budget |
|---|---:|---:|---:|---:|---:|
| `draw_calls` | 4 | 4 | +0 | 1.000 | — |
| `instances_drawn` | 11 | 11 | +0 | 1.000 | — |
| `triangles_drawn` | 256,938 | 256,938 | +0 | 1.000 | — |
| `instances_overflowed` | 0 | 0 | +0 | — | — |
| `uploaded_bytes` | 384 | 384 | +0 | 1.000 | — |
| `gpu_memory_bytes` | 536,471,449 | 536,000,000 | +471,449 | 1.001 | 8,000,000,000 |
| `arena_bytes_in_use` | 844,804 | 844,804 | +0 | 1.000 | — |
| `heap_allocations` | 326 | 300 | +26 | 1.087 | — |
"
        );
    }

    /// The rows of `report` that start with `name`'s cell, and its first line.
    fn lines<'a>(report: &'a str, names: &[&str]) -> Vec<&'a str> {
        let rows = report.lines().filter(|line| {
            names
                .iter()
                .any(|name| line.starts_with(&format!("| {name} |")))
        });
        report.lines().take(1).chain(rows).collect()
    }

    #[test]
    fn a_shorter_run_compares_its_counter_frames_and_no_times() {
        let mut shorter = current();
        shorter.measured_frames = 64;
        shorter.counters.heap_allocations = 999;
        shorter.counter_frames.counters.heap_allocations = 310;
        let report = report(&shorter, Some(&baseline()), &BUDGETS).expect("comparable");
        assert_eq!(
            lines(&report, &["CPU frame", "`lit_pass`", "`heap_allocations`"]),
            [
                "**helmet** on Apple M4 Pro, release, 2560x1440, 64 measured frames; its Baseline \
                 measured 1800 frames on Apple M4 Pro, so times compare with none; counters of \
                 frames 8..72",
                "| CPU frame | 190.0 | 262.3 | — | — | — | — | 6,000.0 |",
                "| `lit_pass` | 755.8 | 768.1 | — | — | — | — | — |",
                "| `heap_allocations` | 310 | 300 | +10 | 1.033 | — |",
            ]
        );
    }

    #[test]
    fn a_run_without_a_baseline_shows_only_its_own_figures_and_the_budgets() {
        let report = report(&current(), None, &BUDGETS).expect("no Baseline to refuse");
        assert_eq!(
            lines(&report, &["GPU frame", "`gpu_memory_bytes`"]),
            [
                "**helmet** on Apple M4 Pro, release, 2560x1440, 1800 measured frames",
                "| GPU frame | 4,208.6 | 5,204.5 | — | — | — | — | 12,000.0 |",
                "| `gpu_memory_bytes` | 536,471,449 | — | — | — | 8,000,000,000 |",
            ]
        );
    }

    /// The aliasing counters follow the others, against the Baseline's when
    /// it has them.
    #[test]
    fn aliasing_shows_beside_the_counters() {
        let mut current = current();
        current.aliasing = Some(Aliasing {
            aliasing_rmse: 21_500,
            aliasing_p99: 150_000,
        });
        let mut baseline = baseline();
        baseline.aliasing = Some(Aliasing {
            aliasing_rmse: 20_000,
            aliasing_p99: 150_000,
        });
        let names = ["`heap_allocations`", "`aliasing_rmse`", "`aliasing_p99`"];
        let report = report(&current, Some(&baseline), &BUDGETS).expect("comparable");
        assert_eq!(
            lines(&report, &names)[1..],
            [
                "| `heap_allocations` | 326 | 300 | +26 | 1.087 | — |",
                "| `aliasing_rmse` | 21,500 | 20,000 | +1,500 | 1.075 | — |",
                "| `aliasing_p99` | 150,000 | 150,000 | +0 | 1.000 | — |",
            ]
        );
        baseline.aliasing = None;
        let report = super::report(&current, Some(&baseline), &BUDGETS).expect("comparable");
        assert_eq!(
            lines(&report, &names[1..])[1..],
            [
                "| `aliasing_rmse` | 21,500 | — | — | — | — |",
                "| `aliasing_p99` | 150,000 | — | — | — | — |",
            ]
        );
    }

    #[test]
    fn a_baseline_of_another_size_is_refused() {
        let mut smaller = baseline();
        smaller.width /= 2;
        assert!(report(&current(), Some(&smaller), &BUDGETS).is_err());
    }
}
