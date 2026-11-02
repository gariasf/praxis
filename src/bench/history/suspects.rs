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
const DEVIATIONS: f64 = 4.0;

/// The smallest shift that makes a suspect, as a share of the window's
/// median, so that a window of near-equal medians does not flag noise.
const SMALLEST_SHIFT: f64 = 0.03;

/// One record's median of a metric, as a Series orders them.
pub(super) struct Point {
    /// None when the record did not measure the metric.
    pub median: Option<f64>,
    /// Whether no record before it compares with it: its Harness version or
    /// scene content changed, or a re-recorded Baseline accepted its level.
    pub starts_line: bool,
}

/// Each point's shift from the median of up to `WINDOW_RECORDS` medians
/// before it on its line, as a share of that median, when it lies outside
/// that median ± max(`DEVIATIONS` × their median absolute deviation,
/// `SMALLEST_SHIFT` of it) and the measured point before it on its line did
/// not lie outside on the same side; none for every other point. So a step
/// is flagged where it starts, not again while the window's median still
/// lags it, and a lone outlier stays in later windows, outvoted.
pub(super) fn suspects(points: &[Point]) -> Vec<Option<f64>> {
    let mut line = Vec::new();
    let mut side_before = None;
    points
        .iter()
        .map(|point| {
            if point.starts_line {
                line.clear();
                side_before = None;
            }
            let median = point.median?;
            let window = &line[line.len().saturating_sub(WINDOW_RECORDS)..];
            let shift = shift_outside(window, median);
            line.push(median);
            let side = shift.map(|shift: f64| shift > 0.0);
            let continues_step = side.is_some() && side == side_before;
            side_before = side;
            shift.filter(|_| !continues_step)
        })
        .collect()
}

/// `median`'s shift from `window`'s median when it lies outside the window.
fn shift_outside(window: &[f64], median: f64) -> Option<f64> {
    if window.len() < FEWEST_JUDGING_RECORDS {
        return None;
    }
    let center = percentile(window.to_vec(), 0.5);
    let deviation = percentile(
        window.iter().map(|value| (value - center).abs()).collect(),
        0.5,
    );
    let tolerance = (DEVIATIONS * deviation).max(SMALLEST_SHIFT * center);
    ((median - center).abs() > tolerance).then(|| (median - center) / center)
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

    /// Indices of the suspects among `points`.
    fn flagged(points: &[Point]) -> Vec<usize> {
        suspects(points)
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
        let shifts = suspects(&line(&medians));
        assert_eq!(flagged(&line(&medians)), [12]);
        let shift = shifts[12].unwrap();
        assert!((0.04..0.06).contains(&shift), "{shift}");
    }

    #[test]
    fn a_step_down_is_flagged_with_its_sign() {
        let mut medians = NOISE.to_vec();
        medians.push(NOISE[0] * 0.9);
        assert!(suspects(&line(&medians))[12].unwrap() < -0.09);
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
