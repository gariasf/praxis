//! Suspects (ADR 0024): timing records whose median sits far from the
//! medians of the records before them on their Series' line.

use crate::bench::percentile;

/// How many records before a record judge it.
const WINDOW_RECORDS: usize = 8;

/// The fewest records before a record that judge it; with fewer, as at the
/// start of a line, it is not judged.
const FEWEST_JUDGING_RECORDS: usize = 3;

/// How many median absolute deviations of its window a median may sit from
/// the window's median before it is a suspect.
const MEDIAN_ABSOLUTE_DEVIATIONS: f64 = 4.0;

/// One record's median of a metric, as a Series orders them.
pub(super) struct Point {
    /// None when the record did not measure the metric.
    pub median: Option<f64>,
    /// Whether no record before it compares with it: its Harness version or
    /// scene content changed, it is an imported or re-recorded Baseline,
    /// which accepted its level, or a Verdict pinned its commit.
    pub starts_line: bool,
}

/// A flagged step: the Suspect that started it.
struct Step {
    /// Its place on the line.
    position: usize,
    slower: bool,
    median: f64,
    tolerance: f64,
}

/// Each point's shift from the median of up to `WINDOW_RECORDS` medians
/// before it on its line, as a share of that median, when it lies outside
/// that median ± max(`MEDIAN_ABSOLUTE_DEVIATIONS` × their median absolute
/// deviation, `smallest_shift` of it), which keeps a window of near-equal
/// medians from flagging noise; none for every other point,
/// and none for a point that only continues a step flagged among those
/// medians: outside on the same side, and within that Suspect's tolerance of
/// its median. So a step is flagged where it starts, not again while the
/// window's median still lags it, even after a noisy record falls back inside;
/// a second step on the same side is flagged again; and a lone outlier stays
/// in later windows, outvoted.
pub(super) fn suspects(points: &[Point], smallest_shift: f64) -> Vec<Option<f64>> {
    let mut line = Vec::new();
    let mut step: Option<Step> = None;
    points
        .iter()
        .map(|point| {
            if point.starts_line {
                line.clear();
                step = None;
            }
            let median = point.median?;
            let position = line.len();
            let window = &line[position.saturating_sub(WINDOW_RECORDS)..];
            let band = band(window, smallest_shift);
            line.push(median);
            let (center, tolerance) = band?;
            if (median - center).abs() <= tolerance {
                return None;
            }
            let slower = median > center;
            let continues = step.as_ref().is_some_and(|step| {
                position - step.position <= WINDOW_RECORDS
                    && step.slower == slower
                    && (median - step.median).abs() <= step.tolerance
            });
            if continues {
                return None;
            }
            step = Some(Step {
                position,
                slower,
                median,
                tolerance,
            });
            Some((median - center) / center)
        })
        .collect()
}

/// `window`'s median and how far from it a median may lie, when it holds
/// enough records to judge.
fn band(window: &[f64], smallest_shift: f64) -> Option<(f64, f64)> {
    if window.len() < FEWEST_JUDGING_RECORDS {
        return None;
    }
    let center = percentile(window.to_vec(), 0.5);
    let deviation = percentile(
        window.iter().map(|value| (value - center).abs()).collect(),
        0.5,
    );
    Some((
        center,
        (MEDIAN_ABSOLUTE_DEVIATIONS * deviation).max(smallest_shift * center),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Points of `medians`, one line.
    fn line(medians: &[f64]) -> Vec<Point> {
        medians
            .iter()
            .map(|&median| Point {
                median: Some(median),
                starts_line: false,
            })
            .collect()
    }

    /// Each point's shift as `suspects` gives it, at a 3% smallest shift.
    fn suspects_of(points: &[Point]) -> Vec<Option<f64>> {
        suspects(points, 0.03)
    }

    /// Indices of the suspects among `points`.
    fn flagged(points: &[Point]) -> Vec<usize> {
        suspects_of(points)
            .iter()
            .enumerate()
            .filter_map(|(index, shift)| shift.map(|_| index))
            .collect()
    }

    /// A GPU frame median's noise around 10 ms: within ±1.2%, as quiet
    /// runs on the M4 Pro read.
    const NOISE: [f64; 12] = [
        10_000.0, 10_080.0, 9_950.0, 10_120.0, 9_900.0, 10_030.0, 9_970.0, 10_060.0, 9_880.0,
        10_010.0, 10_100.0, 9_940.0,
    ];

    #[test]
    fn noise_alone_is_not_flagged() {
        assert_eq!(flagged(&line(&NOISE)), [] as [usize; 0]);
    }

    #[test]
    fn a_five_percent_step_after_noise_is_flagged_at_the_step_only() {
        let mut medians = NOISE.to_vec();
        medians.extend(NOISE.iter().map(|median| median * 1.05));
        let shifts = suspects_of(&line(&medians));
        assert_eq!(flagged(&line(&medians)), [12]);
        let shift = shifts[12].unwrap();
        assert!((0.04..0.06).contains(&shift), "{shift}");
    }

    #[test]
    fn a_second_step_on_the_same_side_is_flagged_again() {
        let mut medians = NOISE.to_vec();
        medians.extend(NOISE[..2].iter().map(|median| median * 1.05));
        medians.extend(NOISE.iter().map(|median| median * 1.10));
        assert_eq!(flagged(&line(&medians)), [12, 14]);
    }

    #[test]
    fn a_step_is_flagged_once_when_a_noisy_record_falls_back_inside_the_band() {
        let mut medians = NOISE.to_vec();
        // 10,250 sits inside its band; 10,450 outside again, on the step.
        medians.extend([10_500.0, 10_250.0, 10_450.0]);
        assert_eq!(flagged(&line(&medians)), [12]);
    }

    #[test]
    fn a_step_down_is_flagged_with_its_sign() {
        let mut medians = NOISE.to_vec();
        medians.push(NOISE[0] * 0.9);
        assert!(suspects_of(&line(&medians))[12].unwrap() < -0.09);
    }

    #[test]
    fn a_step_at_a_re_recorded_baseline_is_not_flagged() {
        let mut points = line(&NOISE);
        let mut after = line(&NOISE.map(|median| median * 1.05));
        after[0].starts_line = true;
        points.extend(after);
        assert_eq!(flagged(&points), [] as [usize; 0]);
    }

    #[test]
    fn a_noisy_window_widens_past_the_smallest_shift() {
        // Medians ±8% apart: a 5% step is within four deviations.
        let noisy = [
            10_000.0, 10_800.0, 9_200.0, 10_400.0, 9_600.0, 10_800.0, 9_200.0, 10_000.0,
        ];
        let mut medians = noisy.to_vec();
        medians.push(10_500.0);
        assert_eq!(flagged(&line(&medians)), [] as [usize; 0]);
    }

    #[test]
    fn a_line_judges_only_from_its_third_record_and_skips_unmeasured_ones() {
        let mut points = line(&[10_000.0, 10_000.0, 20_000.0, 10_000.0, 20_000.0]);
        points.insert(
            3,
            Point {
                median: None,
                starts_line: false,
            },
        );
        // The third median, 20,000, has two before it; the fifth has three.
        assert_eq!(flagged(&points), [5]);
    }
}
