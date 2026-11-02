//! Verdicts (ADR 0024): what the sweep found when it re-benched a slower
//! Suspect beside the record before it and bisected the commits between.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context;

use super::{
    Publication, README, RELEASE_BUILD, adapter_key, fetch_main, git, push, remote_tip, source_hash,
};
use crate::bench::baseline_directory;

/// The branch's directory of Verdicts: `verdicts/<adapter>-<backend>/<scene>/<Suspect's source hash>.json`.
pub(super) const VERDICTS: &str = "verdicts";

/// The file in a night's directory holding the Verdict as the night started.
const NIGHT_VERDICT: &str = "verdict.json";

/// The smallest shift a night's re-bench must still show, as a share of
/// the good end's median, whatever the Suspect's marked shift.
const SMALLEST_HELD_SHIFT: f64 = 0.03;

/// How many runs of each end a night benches, interleaved, before it
/// judges them.
const END_RUNS: usize = 2;

/// A commit and its source hash.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct Commit {
    pub commit: String,
    pub source_hash: String,
}

/// Whether a night's re-bench still showed the Suspect's shift.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum State {
    Held,
    Cleared,
}

/// The commit a bisect pinned the shift to.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct Pinned {
    #[serde(flatten)]
    pub commit: Commit,
    /// Whether the commit changed the scene's Baseline for the adapter, so
    /// re-recorded it at the new level.
    pub covered_by_baseline: bool,
}

/// A Verdict file: one Suspect of one Series, and what the sweep found.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct Verdict {
    pub scene: String,
    /// The judged timing whose median made the record a Suspect.
    pub metric: String,
    /// Its marked shift, as a share of the median before it.
    pub shift: f64,
    pub suspect: Commit,
    /// The record before it on its line.
    pub predecessor: Commit,
    /// None until a night has judged the re-bench.
    pub state: Option<State>,
    /// The last commit known to run at the predecessor's level.
    pub good: String,
    /// The first commit known to run at the Suspect's level.
    pub bad: String,
    pub pinned: Option<Pinned>,
}

impl Verdict {
    /// The Verdict a night starts for `suspect`, judged on none yet.
    pub fn of(suspect: Suspect) -> Self {
        Self {
            good: suspect.predecessor.commit.clone(),
            bad: suspect.suspect.commit.clone(),
            scene: suspect.scene,
            metric: suspect.metric,
            shift: suspect.shift,
            suspect: suspect.suspect,
            predecessor: suspect.predecessor,
            state: None,
            pinned: None,
        }
    }
}

/// A slower Suspect with no Verdict.
#[derive(Clone, Debug, PartialEq)]
pub struct Suspect {
    pub(super) scene: String,
    pub(super) metric: String,
    pub(super) shift: f64,
    pub(super) suspect: Commit,
    pub(super) predecessor: Commit,
    pub(super) committed_unix_seconds: u64,
}

impl std::fmt::Display for Suspect {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "{} {} {} +{:.1}% after {}",
            self.scene,
            self.suspect.commit,
            self.metric,
            self.shift * 100.0,
            self.predecessor.commit
        )
    }
}

/// A Series row as the listing of Suspects reads it.
#[derive(Debug, serde::Deserialize)]
pub(super) struct Row {
    commit: String,
    source_hash: String,
    committed_unix_seconds: u64,
    #[serde(default)]
    suspects: BTreeMap<String, f64>,
}

/// One run of a night: the commit it benched and its median of the
/// Verdict's metric.
#[derive(Debug, PartialEq)]
pub(super) struct Run {
    pub commit: String,
    pub median: f64,
}

/// The slower Suspects among the rows of `scene`'s Series that no Verdict
/// of `verdicts`, that Series', judged: none whose span holds the row,
/// from the record after its predecessor to its Suspect, as a bisect's
/// records land there and may be marked in the Suspect's place. Each
/// takes its largest slower shift.
pub(super) fn unjudged(scene: &str, rows: &[Row], verdicts: &[Verdict]) -> Vec<Suspect> {
    let position = |source_hash: &str| rows.iter().position(|row| row.source_hash == source_hash);
    let judged = |index: usize| {
        verdicts.iter().any(|verdict| {
            match (
                position(&verdict.predecessor.source_hash),
                position(&verdict.suspect.source_hash),
            ) {
                (Some(predecessor), Some(suspect)) => (predecessor + 1..=suspect).contains(&index),
                _ => rows[index].source_hash == verdict.suspect.source_hash,
            }
        })
    };
    rows.iter()
        .enumerate()
        .skip(1)
        .filter(|&(index, _)| !judged(index))
        .filter_map(|(index, row)| {
            let (metric, shift) = row
                .suspects
                .iter()
                .filter(|(_, shift)| **shift > 0.0)
                .max_by(|left, right| left.1.total_cmp(right.1))?;
            let predecessor = &rows[index - 1];
            Some(Suspect {
                scene: scene.to_owned(),
                metric: metric.clone(),
                shift: *shift,
                suspect: Commit {
                    commit: row.commit.clone(),
                    source_hash: row.source_hash.clone(),
                },
                predecessor: Commit {
                    commit: predecessor.commit.clone(),
                    source_hash: predecessor.source_hash.clone(),
                },
                committed_unix_seconds: row.committed_unix_seconds,
            })
        })
        .collect()
}

/// What a night does next for `night`, its Verdict as the night started,
/// after `runs`, the night's runs so far in the order benched, along
/// `first_parent`: `main`'s first-parent commits from the night's good
/// end to its bad end. Until each end has `END_RUNS` runs, the next run
/// is the end with fewer, the good end first, and nothing is judged.
/// Then the Verdict is cleared unless the ends' mean medians hold the
/// shift; held, each later run moves the good or bad end to its commit,
/// by which end's median it lies nearer, and the bisect benches the next
/// commit or pins one. Returns the Verdict once judged, and the commit to
/// bench next, none once the night is done with this Suspect.
pub(super) fn step(
    night: &Verdict,
    runs: &[Run],
    first_parent: &[Commit],
) -> (Option<Verdict>, Option<String>) {
    let medians = |commit: &str| -> Vec<f64> {
        runs.iter()
            .filter(|run| run.commit == commit)
            .map(|run| run.median)
            .collect()
    };
    let (good_runs, bad_runs) = (medians(&night.good), medians(&night.bad));
    if good_runs.len() < END_RUNS || bad_runs.len() < END_RUNS {
        let next = if good_runs.len() <= bad_runs.len() {
            &night.good
        } else {
            &night.bad
        };
        return (None, Some(next.clone()));
    }
    let mean = |values: &[f64]| values.iter().sum::<f64>() / values.len() as f64;
    let (good_median, bad_median) = (mean(&good_runs), mean(&bad_runs));
    let mut verdict = night.clone();
    if !holds(night.shift, good_median, bad_median) {
        verdict.state = Some(State::Cleared);
        return (Some(verdict), None);
    }
    verdict.state = Some(State::Held);
    for run in runs
        .iter()
        .filter(|run| run.commit != night.good && run.commit != night.bad)
    {
        if (run.median - bad_median).abs() < (run.median - good_median).abs() {
            verdict.bad.clone_from(&run.commit);
        } else {
            verdict.good.clone_from(&run.commit);
        }
    }
    let position = |commit: &str| first_parent.iter().position(|each| each.commit == commit);
    let (Some(good), Some(bad)) = (position(&verdict.good), position(&verdict.bad)) else {
        return (Some(verdict), None);
    };
    match next_commit(&first_parent[good..=bad]) {
        Ok(next) => {
            let next = next.commit.clone();
            (Some(verdict), Some(next))
        }
        Err(pinned) => {
            verdict.pinned = Some(Pinned {
                commit: pinned.clone(),
                covered_by_baseline: false,
            });
            (Some(verdict), None)
        }
    }
}

/// The next commit a bisect benches among `commits`, first-parent commits
/// from its good end to its bad end: the first commit of the middle source
/// hash between the ends', a run of commits sharing one source hash being
/// one source. When no other source lies between, the commit that changed
/// the source to the bad end's is pinned instead.
fn next_commit(commits: &[Commit]) -> Result<&Commit, &Commit> {
    let changes: Vec<&Commit> = commits
        .iter()
        .enumerate()
        .filter(|&(index, commit)| {
            index == 0 || commits[index - 1].source_hash != commit.source_hash
        })
        .map(|(_, commit)| commit)
        .collect();
    match changes.len() {
        0..=2 => Err(changes
            .last()
            .copied()
            .unwrap_or(&commits[commits.len() - 1])),
        count => Ok(changes[count / 2]),
    }
}

/// This machine's slower Suspects with no Verdict yet, `adapter`'s on
/// `backend`, on the remote's `bench-history`, oldest first.
pub fn suspects(repository: &Path, adapter: &str, backend: &str) -> anyhow::Result<Vec<Suspect>> {
    let Some(tip) = remote_tip(repository)? else {
        return Ok(Vec::new());
    };
    let key = adapter_key(adapter, Some(backend));
    let verdicts = stored(repository, &tip, &format!("{VERDICTS}/{key}"))?;
    let series = git(
        repository,
        &[
            "ls-tree",
            "--name-only",
            &tip,
            "--",
            &format!("series/{key}/"),
        ],
        None,
    )?;
    let mut suspects = Vec::new();
    for path in series.lines() {
        let Some(scene) = path
            .strip_suffix(".json")
            .and_then(|path| path.rsplit('/').next())
        else {
            continue;
        };
        if scene == RELEASE_BUILD {
            continue;
        }
        let rows: Vec<Row> =
            serde_json::from_str(&git(repository, &["show", &format!("{tip}:{path}")], None)?)
                .with_context(|| format!("parsing {path}"))?;
        let judging: Vec<Verdict> = verdicts
            .iter()
            .filter(|verdict| verdict.scene == scene)
            .cloned()
            .collect();
        suspects.extend(unjudged(scene, &rows, &judging));
    }
    suspects.sort_by_key(|suspect| suspect.committed_unix_seconds);
    Ok(suspects)
}

/// Takes a night's next step of re-benching and bisecting, `adapter`'s on
/// `backend`, its runs `<scene>.json` in the directories of `night` in
/// name order: the night resumes the bisect of a held Verdict, or else
/// starts on the oldest Suspect with none, and keeps to it. Records the
/// Verdict on the remote's `bench-history` once the night's ends are
/// judged, and returns the scene and commit to bench next, none once the
/// night is done.
pub fn bisect(
    repository: &Path,
    adapter: &str,
    backend: &str,
    night: &Path,
) -> anyhow::Result<Option<(String, String)>> {
    let key = adapter_key(adapter, Some(backend));
    let night_file = night.join(NIGHT_VERDICT);
    fetch_main(repository)?;
    let start: Verdict = if night_file.exists() {
        serde_json::from_slice(&std::fs::read(&night_file)?)
            .with_context(|| format!("parsing {}", night_file.display()))?
    } else {
        let resumed = match remote_tip(repository)? {
            Some(tip) => stored(repository, &tip, &format!("{VERDICTS}/{key}"))?
                .into_iter()
                .find(|verdict| verdict.state == Some(State::Held) && verdict.pinned.is_none()),
            None => None,
        };
        let start = match resumed {
            Some(verdict) => verdict,
            None => match suspects(repository, adapter, backend)?.into_iter().next() {
                Some(suspect) => Verdict::of(suspect),
                None => return Ok(None),
            },
        };
        std::fs::create_dir_all(night)?;
        std::fs::write(&night_file, serde_json::to_string_pretty(&start)? + "\n")?;
        start
    };
    let first_parent = first_parent(repository, &start.good, &start.bad)?;
    let (verdict, next) = step(&start, &night_runs(night, &start)?, &first_parent);
    if let Some(mut verdict) = verdict {
        if let Some(pinned) = &mut verdict.pinned {
            // Git names paths with `/` on every platform.
            let baseline: Vec<_> = baseline_directory(adapter, backend)
                .iter()
                .map(|component| component.to_string_lossy().into_owned())
                .chain([format!("{}.json", verdict.scene)])
                .collect();
            let commit = &pinned.commit.commit;
            pinned.covered_by_baseline = !git(
                repository,
                &[
                    "diff",
                    "--name-only",
                    &format!("{commit}^"),
                    commit,
                    "--",
                    &baseline.join("/"),
                ],
                None,
            )?
            .is_empty();
        }
        record(repository, &key, &verdict)?;
    }
    Ok(next.map(|commit| (start.scene, commit)))
}

/// The source hashes of the commits the Verdicts in `directory` pinned:
/// where their Series' lines start.
pub(super) fn pinned_sources(directory: &Path) -> anyhow::Result<BTreeSet<String>> {
    if !directory.exists() {
        return Ok(BTreeSet::new());
    }
    let mut sources = BTreeSet::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        let verdict: Verdict = serde_json::from_slice(&std::fs::read(&path)?)
            .with_context(|| format!("parsing {}", path.display()))?;
        sources.extend(verdict.pinned.map(|pinned| pinned.commit.source_hash));
    }
    Ok(sources)
}

/// Every Verdict under `directory` on the branch's `tip`.
fn stored(repository: &Path, tip: &str, directory: &str) -> anyhow::Result<Vec<Verdict>> {
    git(
        repository,
        &["ls-tree", "-r", "--name-only", tip, "--", directory],
        None,
    )?
    .lines()
    .map(|path| {
        serde_json::from_str(&git(repository, &["show", &format!("{tip}:{path}")], None)?)
            .with_context(|| format!("parsing {path}"))
    })
    .collect()
}

/// `main`'s first-parent commits from `good` to `bad`, each with its
/// source hash.
fn first_parent(repository: &Path, good: &str, bad: &str) -> anyhow::Result<Vec<Commit>> {
    let later = git(
        repository,
        &[
            "rev-list",
            "--first-parent",
            "--reverse",
            &format!("{good}..{bad}"),
        ],
        None,
    )?;
    std::iter::once(good)
        .chain(later.lines())
        .map(|commit| {
            Ok(Commit {
                commit: commit.to_owned(),
                source_hash: source_hash(repository, commit)?,
            })
        })
        .collect()
}

/// The runs in `night`'s directories, in name order, of `verdict`'s scene:
/// the commit each benched and its median of the Verdict's metric.
fn night_runs(night: &Path, verdict: &Verdict) -> anyhow::Result<Vec<Run>> {
    let mut directories: Vec<PathBuf> = std::fs::read_dir(night)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    directories.sort();
    let mut runs = Vec::new();
    for directory in directories {
        let path = directory.join(format!("{}.json", verdict.scene));
        if !path.exists() {
            continue;
        }
        // Read loosely: an earlier commit's binary wrote it.
        let result: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)
            .with_context(|| format!("parsing {}", path.display()))?;
        let commit = result["provenance"]["checkout"]["commit"].as_str();
        let median = result[&verdict.metric]["median"].as_f64();
        if let (Some(commit), Some(median)) = (commit, median) {
            runs.push(Run {
                commit: commit.to_owned(),
                median,
            });
        }
    }
    Ok(runs)
}

/// Pushes `verdict` to the branch under `adapter_key`, and rebuilds every
/// Series, as a pinned commit starts a line.
fn record(repository: &Path, adapter_key: &str, verdict: &Verdict) -> anyhow::Result<()> {
    let path = Path::new(VERDICTS)
        .join(adapter_key)
        .join(&verdict.scene)
        .join(format!("{}.json", verdict.suspect.source_hash));
    let contents = serde_json::to_string_pretty(verdict)? + "\n";
    let message = format!(
        "Record the Verdict on {}'s Suspect {} from {adapter_key}",
        verdict.scene, verdict.suspect.commit
    );
    push(repository, &message, |worktree| {
        let file = worktree.directory.join(&path);
        std::fs::create_dir_all(file.parent().context("a Verdict has a directory")?)?;
        std::fs::write(file, &contents)?;
        worktree.rebuild_every_series()?;
        std::fs::write(worktree.directory.join("README.md"), README)?;
        Ok(Publication::default())
    })?;
    Ok(())
}

/// Whether a re-bench holds a Suspect marked `shift` slower: the bad end's
/// median `bad_median` is still above the good end's `good_median` by at
/// least half that shift and by `SMALLEST_HELD_SHIFT`, both as shares of
/// `good_median`.
fn holds(shift: f64, good_median: f64, bad_median: f64) -> bool {
    (bad_median - good_median) / good_median >= (shift / 2.0).max(SMALLEST_HELD_SHIFT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shift_holds_at_half_its_marked_size_and_three_percent() {
        // Marked 10% slower: 5% still holds, 4.9% does not.
        assert!(holds(0.10, 10_000.0, 10_500.0));
        assert!(!holds(0.10, 10_000.0, 10_490.0));
        // Marked 4% slower: half is 2%, but under 3% never holds.
        assert!(!holds(0.04, 10_000.0, 10_290.0));
        assert!(holds(0.04, 10_000.0, 10_300.0));
        // Faster on the night is no hold.
        assert!(!holds(0.10, 10_000.0, 9_000.0));
    }

    /// Commits named by their letter, each with the source hash `sources`
    /// gives it at the same place.
    fn commits(names: &str, sources: &str) -> Vec<Commit> {
        names
            .chars()
            .zip(sources.chars())
            .map(|(name, source)| Commit {
                commit: name.to_string(),
                source_hash: source.to_string(),
            })
            .collect()
    }

    fn next(names: &str, sources: &str) -> Result<String, String> {
        let commits = commits(names, sources);
        next_commit(&commits)
            .map(|commit| commit.commit.clone())
            .map_err(|commit| commit.commit.clone())
    }

    #[test]
    fn a_bisect_benches_the_middle_source_and_pins_the_commit_that_changed_it() {
        // Sources 1 to 5: the middle of 2, 3 and 4 is 3.
        assert_eq!(next("abcde", "12345"), Ok("c".to_owned()));
        // One source between the ends: it is benched.
        assert_eq!(next("abc", "123"), Ok("b".to_owned()));
        // A commit that changed no measured path is not a source of its own.
        assert_eq!(next("abcd", "1223"), Ok("b".to_owned()));
        // None between: the bad end's source came with its first commit.
        assert_eq!(next("ab", "12"), Err("b".to_owned()));
        assert_eq!(next("abcd", "1222"), Err("b".to_owned()));
        assert_eq!(next("abcd", "1112"), Err("d".to_owned()));
    }

    /// A Verdict on a Suspect `d`, marked 10% slower after `a`.
    fn night() -> Verdict {
        Verdict::of(Suspect {
            scene: "town".to_owned(),
            metric: "cpu_frame_microseconds".to_owned(),
            shift: 0.10,
            suspect: commits("d", "4").remove(0),
            predecessor: commits("a", "1").remove(0),
            committed_unix_seconds: 0,
        })
    }

    fn runs(runs: &[(&str, f64)]) -> Vec<Run> {
        runs.iter()
            .map(|&(commit, median)| Run {
                commit: commit.to_owned(),
                median,
            })
            .collect()
    }

    #[test]
    fn a_night_benches_its_ends_interleaved_before_it_judges() {
        let first_parent = commits("abcd", "1234");
        let mut so_far = Vec::new();
        for (expected, median) in [
            ("a", 10_000.0),
            ("d", 11_000.0),
            ("a", 10_000.0),
            ("d", 11_000.0),
        ] {
            assert_eq!(
                step(&night(), &runs(&so_far), &first_parent),
                (None, Some(expected.to_owned()))
            );
            so_far.push((expected, median));
        }
        let (verdict, next) = step(&night(), &runs(&so_far), &first_parent);
        assert_eq!(verdict.unwrap().state, Some(State::Held));
        // Sources 2 and 3 lie between: the middle is 3's commit, c.
        assert_eq!(next.as_deref(), Some("c"));
    }

    #[test]
    fn a_re_bench_that_does_not_hold_clears_the_suspect() {
        let ends = runs(&[
            ("a", 10_000.0),
            ("d", 10_200.0),
            ("a", 10_100.0),
            ("d", 10_300.0),
        ]);
        let (verdict, next) = step(&night(), &ends, &commits("abcd", "1234"));
        let verdict = verdict.unwrap();
        assert_eq!(
            (verdict.state, verdict.pinned, next),
            (Some(State::Cleared), None, None)
        );
    }

    #[test]
    fn a_bisect_moves_its_ends_by_the_nearer_median_and_pins_the_commit() {
        let first_parent = commits("abcd", "1234");
        let mut night_runs = vec![
            ("a", 10_000.0),
            ("d", 11_000.0),
            ("a", 10_100.0),
            ("d", 11_100.0),
        ];
        // c runs at d's level, so the bad end moves to c, and b is next.
        night_runs.push(("c", 10_900.0));
        let (verdict, next) = step(&night(), &runs(&night_runs), &first_parent);
        assert_eq!(
            (verdict.unwrap().bad, next.as_deref()),
            ("c".to_owned(), Some("b"))
        );
        // b runs at a's level: c changed it.
        night_runs.push(("b", 10_200.0));
        let (verdict, next) = step(&night(), &runs(&night_runs), &first_parent);
        let verdict = verdict.unwrap();
        assert_eq!(
            (verdict.good.as_str(), verdict.bad.as_str(), next),
            ("b", "c", None)
        );
        assert_eq!(verdict.pinned.unwrap().commit, commits("c", "3")[0]);
    }

    /// A Series row of `commit` with source `source_hash`, its suspects
    /// `suspects`.
    fn row(commit: &str, suspects: &[(&str, f64)]) -> Row {
        Row {
            commit: commit.to_owned(),
            source_hash: format!("source-{commit}"),
            committed_unix_seconds: 0,
            suspects: suspects
                .iter()
                .map(|&(metric, shift)| (metric.to_owned(), shift))
                .collect(),
        }
    }

    #[test]
    fn a_slower_suspect_is_listed_with_its_predecessor_until_a_verdict_spans_it() {
        let rows = [
            row("a", &[]),
            row("b", &[("gpu_frame_microseconds", -0.08)]),
            row("c", &[]),
            row(
                "d",
                &[
                    ("cpu_frame_microseconds", 0.04),
                    ("gpu_frame_microseconds", 0.06),
                ],
            ),
            row("e", &[("cpu_frame_microseconds", 0.05)]),
        ];
        let listed = |verdicts: &[Verdict]| -> Vec<(String, String, String)> {
            unjudged("town", &rows, verdicts)
                .into_iter()
                .map(|suspect| {
                    (
                        suspect.suspect.commit,
                        suspect.metric,
                        suspect.predecessor.commit,
                    )
                })
                .collect()
        };
        // b got faster, so only d and e are listed, d by its larger shift.
        let gpu = "gpu_frame_microseconds".to_owned();
        let cpu = "cpu_frame_microseconds".to_owned();
        assert_eq!(
            listed(&[]),
            [
                ("d".to_owned(), gpu, "c".to_owned()),
                ("e".to_owned(), cpu.clone(), "d".to_owned())
            ]
        );
        // A Verdict on e from b spans d too.
        let verdict = Verdict::of(Suspect {
            scene: "town".to_owned(),
            metric: cpu,
            shift: 0.05,
            suspect: Commit {
                commit: "e".to_owned(),
                source_hash: "source-e".to_owned(),
            },
            predecessor: Commit {
                commit: "b".to_owned(),
                source_hash: "source-b".to_owned(),
            },
            committed_unix_seconds: 0,
        });
        assert_eq!(listed(&[verdict]), []);
    }
}
