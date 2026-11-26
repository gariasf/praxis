//! `praxis history rerecord`: runs of `main` re-record the
//! counters of this machine's Baselines where they moved. A re-recorded
//! Baseline keeps its timings and Harness version, which a person chooses,
//! so its record starts no line on its Series; where those timings no
//! longer compare with the run, the Baseline is left for a person to
//! re-record whole.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use anyhow::Context;

use super::remote_tip;
use crate::bench::provenance::git;
use crate::bench::{
    Counter, Finding, Measurements, adapter_key, baseline_directory, counters_moved,
};

/// The Series files' directory on the `bench-history` branch.
const SERIES: &str = "series";

/// The most commits a re-recorded Baseline's report lists between the
/// records its counters moved between; a longer range is counted instead.
const LISTED_COMMITS: usize = 20;

/// What a run made of the counters of its scene's Baseline.
#[derive(Debug, PartialEq)]
pub enum CounterRerecord {
    /// The run counted what the Baseline holds.
    Unchanged,
    /// The run's counters differ: the Baseline with them, its timings and
    /// provenance kept, and each metric that moved.
    Rerecorded {
        baseline: Box<Measurements>,
        moved: Vec<Finding>,
    },
    /// The counters differ, and the Baseline's timings no longer compare with
    /// the run, for `reason`: a person re-records it whole.
    Refused { moved: Vec<Finding>, reason: String },
}

/// Judges `current`, a run of `main`, against `baseline`, its scene's
/// Baseline on the same adapter and backend, as a recording does
/// (`CompareMode::Recording`, Budgets aside), and re-records the Baseline's
/// counters, `Aliasing` and GPU memory by kind from it when they moved.
/// Fails when the two ran other scenes, profiles or sizes.
pub fn rerecord_counters(
    current: &Measurements,
    baseline: &Measurements,
) -> anyhow::Result<CounterRerecord> {
    let moved = counters_moved(current, baseline)?;
    if moved.is_empty() {
        return Ok(CounterRerecord::Unchanged);
    }
    if let Some(reason) = timings_incomparable(current, baseline) {
        return Ok(CounterRerecord::Refused { moved, reason });
    }
    let rerecorded = Measurements {
        counters: current.counters,
        counter_frames: current.counter_frames.clone(),
        gpu_memory_bytes_by_kind: current.gpu_memory_bytes_by_kind.clone(),
        // A run that skipped `Aliasing` leaves the Baseline's.
        aliasing: current.aliasing.or(baseline.aliasing),
        ..baseline.clone()
    };
    Ok(CounterRerecord::Rerecorded {
        baseline: Box::new(rerecorded),
        moved,
    })
}

/// Why `baseline`'s timings would not compare with `current`'s once its
/// counters were `current`'s, or none: a Baseline holds one run's timings,
/// under one Harness version, of one length, timing the same passes.
fn timings_incomparable(current: &Measurements, baseline: &Measurements) -> Option<String> {
    let Some(provenance) = &current.provenance else {
        return Some("this run has no provenance".to_owned());
    };
    if provenance.checkout.dirty {
        return Some(format!(
            "this run's checkout differed from {}",
            provenance.checkout.commit
        ));
    }
    let Some(baseline_provenance) = &baseline.provenance else {
        return Some("the Baseline names no Harness version".to_owned());
    };
    let (version, baseline_version) = (
        provenance.harness_version,
        baseline_provenance.harness_version,
    );
    if version != baseline_version {
        return Some(format!(
            "its timings were measured under Harness version {baseline_version}, \
             this run under {version}"
        ));
    }
    if (&current.adapter, &current.backend) != (&baseline.adapter, &baseline.backend) {
        return Some(format!(
            "it ran on {} through {}, this run on {} through {}",
            baseline.adapter, baseline.backend, current.adapter, current.backend
        ));
    }
    if current.measured_frames != baseline.measured_frames {
        return Some(format!(
            "it measured {} frames, this run {}",
            baseline.measured_frames, current.measured_frames
        ));
    }
    let timed_passes = |measurements: &Measurements| -> (Vec<String>, Vec<String>) {
        (
            measurements.gpu_pass_microseconds.keys().cloned().collect(),
            measurements
                .gate_pass_microseconds
                .keys()
                .cloned()
                .collect(),
        )
    };
    if timed_passes(current) != timed_passes(baseline) {
        return Some("it times other passes than this run".to_owned());
    }
    None
}

/// A Series row's commit and counters: the counter frames' among its other
/// fields, and the measured frames' beside them.
#[derive(serde::Deserialize)]
struct SeriesCounters {
    commit: String,
    measured_frames: u32,
    #[serde(default)]
    measured_counters: BTreeMap<String, u64>,
    #[serde(flatten)]
    fields: BTreeMap<String, serde_json::Value>,
}

impl SeriesCounters {
    /// Whether the row counted what `baseline` holds, as a recording judges
    /// it: its counter frames' counters, and its measured frames' where the
    /// row measured as many and the Baseline's did not stream, the counters
    /// no comparison judges aside. A counter the row did not count reads as
    /// the same.
    fn counted(&self, baseline: &Measurements) -> bool {
        let mut judged = Counter::ALL.into_iter().filter(|counter| counter.judged());
        let counter_frames_match = judged.clone().all(|counter| {
            let value = baseline.counter_frames.counters[counter];
            (self.fields.get(counter.name())).is_none_or(|counted| counted.as_u64() == Some(value))
        });
        let measured_compared =
            self.measured_frames == baseline.measured_frames && !baseline.counters.streamed();
        let measured_match = !measured_compared
            || judged.all(|counter| {
                (self.measured_counters.get(counter.name()))
                    .is_none_or(|&counted| counted == baseline.counters[counter])
            });
        counter_frames_match && measured_match
    }
}

/// The commits between which `baseline`'s counters moved, by the rows of
/// its Series, oldest first: the newest row that counted them, and the row
/// after it, or `run_commit` when none follows. None when no row counted
/// them.
fn moved_between<'rows>(
    rows: &'rows [SeriesCounters],
    baseline: &Measurements,
    run_commit: &'rows str,
) -> Option<(&'rows str, &'rows str)> {
    let counted = rows.iter().rposition(|row| row.counted(baseline))?;
    let after = rows.get(counted + 1).map_or(run_commit, |row| &row.commit);
    Some((&rows[counted].commit, after))
}

/// Judges each of `results`, runs of `main` in `repository`, against its
/// Baseline there, writes the Baselines whose counters moved, and returns a
/// Markdown list of what it did, a scene an item, with the commits each
/// re-recorded Baseline's counters moved between on this machine's Series
/// (`bench-history`); a scene whose counters held has none. A scene it
/// cannot judge is reported, its Baseline untouched, and the others go on.
pub fn rerecord_baselines(repository: &Path, results: &[Measurements]) -> anyhow::Result<String> {
    let tip = remote_tip(repository).unwrap_or_else(|error| {
        tracing::warn!(error = format!("{error:#}"), "bench-history unreachable");
        None
    });
    let mut report = String::new();
    for current in results {
        match rerecord_scene(repository, tip.as_deref(), current) {
            Ok(item) => report.push_str(&item),
            Err(error) => writeln!(report, "- `{}`: not judged: {error:#}", current.scene)?,
        }
    }
    Ok(report)
}

/// `rerecord_baselines` of one run, `current`: its report item, its Baseline
/// written only once it is judged.
fn rerecord_scene(
    repository: &Path,
    tip: Option<&str>,
    current: &Measurements,
) -> anyhow::Result<String> {
    let scene = &current.scene;
    let relative_path =
        baseline_directory(&current.adapter, &current.backend).join(format!("{scene}.json"));
    let path = repository.join(&relative_path);
    if !path.exists() {
        return Ok(format!(
            "- `{scene}`: no Baseline at `{}`\n",
            relative_path.display()
        ));
    }
    let json = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let baseline = Measurements::from_baseline_json(&json)
        .with_context(|| format!("reading {}", path.display()))?;
    let moved_list = |moved: &[Finding]| {
        (moved.iter())
            .map(|finding| format!("`{}` {}", finding.metric, finding.detail))
            .collect::<Vec<_>>()
            .join(", ")
    };
    Ok(match rerecord_counters(current, &baseline)? {
        CounterRerecord::Unchanged => String::new(),
        CounterRerecord::Refused { moved, reason } => format!(
            "- `{scene}`: not re-recorded, as {reason}; a person re-records it whole. \
             Moved: {}\n",
            moved_list(&moved)
        ),
        CounterRerecord::Rerecorded {
            baseline: rerecorded,
            moved,
        } => {
            std::fs::write(&path, serde_json::to_string_pretty(&rerecorded)? + "\n")
                .with_context(|| format!("writing {}", path.display()))?;
            let mut item = format!("- `{scene}`: re-recorded {}.\n", moved_list(&moved));
            match moved_commits(repository, tip, current, &baseline, &relative_path) {
                Ok(commits) => item.push_str(&commits.unwrap_or_default()),
                Err(error) => writeln!(item, "  The commits between are unknown: {error:#}")?,
            }
            item
        }
    })
}

/// A commit's first 8 characters, as reports name it.
fn short_commit(commit: &str) -> &str {
    commit.get(..8).unwrap_or(commit)
}

/// The commits `baseline`, at `relative_path`, moved between by `current`'s
/// Series at the `bench-history` commit `tip`, as indented Markdown lines:
/// from the newest record that counted its counters, or else the commit that
/// last wrote it, to the record after that, or else the run's commit. None
/// when the run names no commit.
fn moved_commits(
    repository: &Path,
    tip: Option<&str>,
    current: &Measurements,
    baseline: &Measurements,
    relative_path: &Path,
) -> anyhow::Result<Option<String>> {
    let Some(provenance) = &current.provenance else {
        return Ok(None);
    };
    let run_commit = provenance.checkout.commit.as_str();
    // Git names paths with `/` on every platform.
    let series = tip.map(|tip| {
        format!(
            "{tip}:{SERIES}/{}/{}.json",
            adapter_key(&current.adapter, Some(&current.backend)),
            current.scene
        )
    });
    let rows: Vec<SeriesCounters> = match series
        .as_ref()
        .and_then(|series| git(repository, &["show", series], None).ok())
    {
        Some(json) => serde_json::from_str(&json)
            .with_context(|| format!("parsing {}", series.as_deref().unwrap_or_default()))?,
        None => Vec::new(),
    };
    let written;
    let (start, end, sentence) = match moved_between(&rows, baseline, run_commit) {
        Some((counted, moved)) => (
            counted,
            moved,
            format!(
                "They moved after `{}`, the newest record on this machine's Series that \
                 counted them, by `{}`",
                short_commit(counted),
                short_commit(moved)
            ),
        ),
        // A Baseline whose counters alone were re-recorded starts no record
        // on its Series (`import::remeasured`), and one edited by hand was
        // never run, so the commit that last wrote it bounds the range.
        None => {
            let path = relative_path.to_string_lossy().replace('\\', "/");
            written = git(
                repository,
                &["log", "-1", "--format=%H", run_commit, "--", &path],
                None,
            )?;
            let later = |row: &&SeriesCounters| {
                row.commit != written
                    && git(
                        repository,
                        &["merge-base", "--is-ancestor", &written, &row.commit],
                        None,
                    )
                    .is_ok()
            };
            let moved = (rows.iter().find(later)).map_or(run_commit, |row| &row.commit);
            (
                written.as_str(),
                moved,
                format!(
                    "No record on this machine's Series counted them; `{}` last wrote the \
                     Baseline, and `{}`, the first record since, counts otherwise",
                    short_commit(&written),
                    short_commit(moved)
                ),
            )
        }
    };
    let range = format!("{start}..{end}");
    let log = git(
        repository,
        &["log", "--first-parent", "--format=%h %s", &range],
        None,
    )
    .with_context(|| format!("listing {range}"))?;
    let commits: Vec<&str> = log.lines().collect();
    let mut lines = String::new();
    writeln!(
        lines,
        "  {sentence}. The commits between ({}):",
        commits.len()
    )?;
    for commit in commits.iter().take(LISTED_COMMITS) {
        writeln!(lines, "  - {commit}")?;
    }
    if commits.len() > LISTED_COMMITS {
        writeln!(lines, "  - and {} more", commits.len() - LISTED_COMMITS)?;
    }
    Ok(Some(lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::Checkout;
    use crate::bench::Verdict;
    use crate::bench::tests::{baseline, provenance};

    /// A clean run of commit `commit` under Harness version `version`.
    fn run(commit: &str, version: u32) -> Measurements {
        let checkout = Checkout {
            source_hash: format!("source-{commit}"),
            commit: commit.to_owned(),
            dirty: false,
            branch: "HEAD".to_owned(),
        };
        Measurements {
            provenance: Some(crate::bench::Provenance {
                harness_version: version,
                ..provenance(checkout, 1.01)
            }),
            ..baseline()
        }
    }

    fn with_draw_calls(mut measurements: Measurements, draw_calls: u64) -> Measurements {
        measurements.counters[Counter::DrawCalls] = draw_calls;
        measurements.counter_frames.counters[Counter::DrawCalls] = draw_calls;
        measurements
    }

    #[test]
    fn a_run_that_counts_what_its_baseline_holds_changes_nothing() {
        let baseline = run("a", 12);
        let mut slower = run("b", 12);
        slower.cpu_frame_microseconds.median *= 2.0;
        assert_eq!(
            rerecord_counters(&slower, &baseline).unwrap(),
            CounterRerecord::Unchanged
        );
    }

    #[test]
    fn a_moved_counter_is_re_recorded_and_the_baselines_timings_and_provenance_stay() {
        let baseline = run("a", 12);
        let mut current = with_draw_calls(run("b", 12), baseline.counters[Counter::DrawCalls] + 4);
        current.cpu_frame_microseconds.median *= 2.0;
        current.gpu_memory_bytes_by_kind = [("staging".to_owned(), 7)].into();
        let CounterRerecord::Rerecorded {
            baseline: rerecorded,
            moved,
        } = rerecord_counters(&current, &baseline).unwrap()
        else {
            panic!("the Baseline is re-recorded");
        };
        assert_eq!(
            (moved.iter())
                .map(|finding| (finding.metric, finding.verdict))
                .collect::<Vec<_>>(),
            [("draw_calls", Verdict::Fail)]
        );
        assert_eq!(rerecorded.counters, current.counters);
        assert_eq!(rerecorded.counter_frames, current.counter_frames);
        assert_eq!(
            rerecorded.gpu_memory_bytes_by_kind,
            current.gpu_memory_bytes_by_kind
        );
        assert_eq!(
            rerecorded.cpu_frame_microseconds,
            baseline.cpu_frame_microseconds
        );
        assert_eq!(rerecorded.provenance, baseline.provenance);
        // Its import would measure nothing again, so no Series line starts.
        let json = |measurements: &Measurements| serde_json::to_string(measurements).unwrap();
        assert!(!super::super::import::remeasured(&json(&baseline), &json(&rerecorded)).unwrap());
    }

    /// A re-record rewrites the whole file, so a timing read and written
    /// again keeps its text: serde_json parses floats exactly
    /// (`float_roundtrip`).
    #[test]
    fn a_baselines_timings_keep_their_text_through_a_re_record() {
        let mut baseline = run("a", 12);
        baseline.cpu_frame_microseconds.maximum = Some(1086.9759999999999);
        let json = serde_json::to_string_pretty(&baseline).unwrap();
        assert!(json.contains("1086.9759999999999"));
        let read = Measurements::from_baseline_json(json.as_bytes()).unwrap();
        assert_eq!(serde_json::to_string_pretty(&read).unwrap(), json);
    }

    #[test]
    fn a_baseline_whose_timings_no_longer_compare_is_refused() {
        let current = with_draw_calls(run("b", 12), 9);
        let refused = |baseline: &Measurements| match rerecord_counters(&current, baseline) {
            Ok(CounterRerecord::Refused { reason, .. }) => reason,
            other => panic!("refused, not {other:?}"),
        };
        assert!(refused(&run("a", 11)).contains("Harness version 11"));
        let mut retimed = run("a", 12);
        retimed.gpu_pass_microseconds.remove("main_pass");
        assert!(refused(&retimed).contains("other passes"));
        let mut shorter = run("a", 12);
        shorter.measured_frames = 64;
        assert!(refused(&shorter).contains("measured 64 frames"));
        let mut dirty = with_draw_calls(run("b", 12), 9);
        if let Some(provenance) = &mut dirty.provenance {
            provenance.checkout.dirty = true;
        }
        assert!(matches!(
            rerecord_counters(&dirty, &run("a", 12)),
            Ok(CounterRerecord::Refused { .. })
        ));
    }

    /// A Series row of `commit` counting `draw_calls` in its counter frames
    /// and its measured frames.
    fn row(commit: &str, draw_calls: u64) -> SeriesCounters {
        let measured = with_draw_calls(baseline(), draw_calls);
        serde_json::from_value(serde_json::json!({
            "commit": commit,
            "harness_version": 12,
            "draw_calls": draw_calls,
            "measured_frames": measured.measured_frames,
            "measured_counters": {"draw_calls": draw_calls},
        }))
        .unwrap()
    }

    #[test]
    fn counters_moved_between_the_last_row_that_counted_them_and_the_next() {
        let baseline = with_draw_calls(baseline(), 4);
        let rows = [
            row("a", 4),
            row("b", 8),
            row("c", 4),
            row("d", 8),
            row("e", 8),
        ];
        assert_eq!(moved_between(&rows, &baseline, "f"), Some(("c", "d")));
    }

    #[test]
    fn counters_the_series_still_counts_moved_after_its_last_row() {
        let baseline = with_draw_calls(baseline(), 4);
        let rows = [row("a", 8), row("b", 4)];
        assert_eq!(moved_between(&rows, &baseline, "f"), Some(("b", "f")));
        assert_eq!(moved_between(&[row("a", 8)], &baseline, "f"), None);
    }

    #[test]
    fn a_row_of_another_length_judges_by_its_counter_frames_alone() {
        let baseline = with_draw_calls(baseline(), 4);
        let mut other_length = row("a", 4);
        other_length.measured_frames = 64;
        other_length
            .measured_counters
            .insert("draw_calls".to_owned(), 9);
        assert!(other_length.counted(&baseline));
        let mut measured_moved = row("b", 4);
        measured_moved
            .measured_counters
            .insert("draw_calls".to_owned(), 9);
        assert!(!measured_moved.counted(&baseline));
    }

    #[test]
    fn a_row_differing_only_where_comparisons_do_not_judge_counted_the_baseline() {
        let baseline = with_draw_calls(baseline(), 4);
        let mut unjudged = row("a", 4);
        for counter in Counter::ALL.into_iter().filter(|counter| !counter.judged()) {
            unjudged.fields.insert(counter.name().to_owned(), 9.into());
            unjudged
                .measured_counters
                .insert(counter.name().to_owned(), 9);
        }
        assert!(unjudged.counted(&baseline));
    }

    #[test]
    fn a_streamed_baselines_measured_counters_judge_no_row() {
        let mut streamed = with_draw_calls(baseline(), 4);
        streamed.counters[Counter::ChunksLoaded] = 3;
        let mut measured_moved = row("a", 4);
        measured_moved
            .measured_counters
            .insert("draw_calls".to_owned(), 9);
        assert!(measured_moved.counted(&streamed));
    }

    #[test]
    fn a_run_that_skipped_aliasing_keeps_the_baselines() {
        let mut baseline = run("a", 12);
        baseline.aliasing = Some(crate::bench::Aliasing {
            aliasing_rmse: 5,
            aliasing_p99: 7,
        });
        let current = Measurements {
            aliasing: None,
            ..with_draw_calls(run("b", 12), 9)
        };
        let Ok(CounterRerecord::Rerecorded {
            baseline: rerecorded,
            ..
        }) = rerecord_counters(&current, &baseline)
        else {
            panic!("the Baseline is re-recorded");
        };
        assert_eq!(rerecorded.aliasing, baseline.aliasing);
    }
}
