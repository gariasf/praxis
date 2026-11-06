//! A Night's directory: the Verdict it started from, then one directory per
//! run, numbered in the order benched, each holding the commit it benched
//! and the results its bench wrote.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::Context;

use super::verdicts::{Run, Verdict};
use super::{Publication, publish};
use crate::bench::Measurements;

/// The file in a Night's directory holding the Verdict as the Night started.
pub(super) const NIGHT_VERDICT: &str = "verdict.json";

/// The file in a run's directory naming the commit it benched, for a binary
/// too old to stamp its results with provenance.
const RUN_COMMIT_FILE: &str = "commit";

/// The commit that added `bench --no-publish`: a binary of a commit before
/// it refuses the flag, and publishes nothing without it.
const NO_PUBLISH_ADDED: &str = "3c6b9d92";

/// The Verdict `night` started from, none before it starts.
pub(super) fn started(night: &Path) -> anyhow::Result<Option<Verdict>> {
    let path = night.join(NIGHT_VERDICT);
    if !path.exists() {
        return Ok(None);
    }
    let verdict = serde_json::from_slice(&std::fs::read(&path)?)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(verdict))
}

/// Starts `night` from `verdict`.
pub(super) fn start(night: &Path, verdict: &Verdict) -> anyhow::Result<()> {
    std::fs::create_dir_all(night)?;
    std::fs::write(
        night.join(NIGHT_VERDICT),
        serde_json::to_string_pretty(verdict)? + "\n",
    )?;
    Ok(())
}

/// The run a Night benches next: its directory, and the rest of `bench`'s
/// arguments after it.
#[derive(Debug, PartialEq)]
pub struct NextRun {
    pub directory: PathBuf,
    pub arguments: Vec<String>,
}

/// Creates `night`'s next run directory, for a bench of `scene` at
/// `commit`, a commit of the checkout holding `repository`.
pub fn next_run(
    repository: &Path,
    night: &Path,
    scene: &str,
    commit: &str,
) -> anyhow::Result<NextRun> {
    run_after(repository, night, scene, commit, NO_PUBLISH_ADDED)
}

/// `next_run`, `--no-publish` added by the commit `no_publish_added`.
fn run_after(
    repository: &Path,
    night: &Path,
    scene: &str,
    commit: &str,
    no_publish_added: &str,
) -> anyhow::Result<NextRun> {
    let directory = night.join(format!("{:03}", run_directories(night)?.len() + 1));
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join(RUN_COMMIT_FILE), format!("{commit}\n"))?;
    let mut arguments = Vec::new();
    if is_ancestor(repository, no_publish_added, commit)? {
        arguments.push("--no-publish".to_owned());
    } else {
        tracing::warn!(
            scene,
            commit,
            "re-bench without --no-publish: its binary predates the flag and publishes nothing"
        );
    }
    arguments.push(scene.to_owned());
    Ok(NextRun {
        directory,
        arguments,
    })
}

/// Whether `ancestor` is `commit` or comes before it.
fn is_ancestor(repository: &Path, ancestor: &str, commit: &str) -> anyhow::Result<bool> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["merge-base", "--is-ancestor", ancestor, commit])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .context("running git")?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => anyhow::bail!("no ancestry of {commit} known"),
    }
}

/// `night`'s run directories, in the order benched; none before it starts.
fn run_directories(night: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !night.exists() {
        return Ok(Vec::new());
    }
    let mut directories = Vec::new();
    for entry in std::fs::read_dir(night)? {
        let path = entry?.path();
        if path.is_dir() {
            directories.push(path);
        }
    }
    directories.sort();
    Ok(directories)
}

/// `night`'s runs of `scene`, in the order benched: the commit each
/// benched, as its provenance or else its `RUN_COMMIT_FILE` names it, and
/// its median of `metric`.
pub(super) fn runs(night: &Path, scene: &str, metric: &str) -> anyhow::Result<Vec<Run>> {
    let mut runs = Vec::new();
    for directory in run_directories(night)? {
        let path = directory.join(format!("{scene}.json"));
        if !path.exists() {
            continue;
        }
        // Read loosely: an earlier commit's binary wrote it.
        let result = read_result(&path)?;
        let commit = match result["provenance"]["commit"].as_str() {
            Some(commit) => Some(commit.to_owned()),
            None => std::fs::read_to_string(directory.join(RUN_COMMIT_FILE))
                .ok()
                .map(|commit| commit.trim().to_owned()),
        };
        let median = result[metric]["median"].as_f64();
        if let (Some(commit), Some(median)) = (commit, median) {
            runs.push(Run { commit, median });
        }
    }
    Ok(runs)
}

fn read_result(path: &Path) -> anyhow::Result<serde_json::Value> {
    serde_json::from_slice(&std::fs::read(path)?)
        .with_context(|| format!("parsing {}", path.display()))
}

/// Publishes every result in `night`'s runs, measured in the checkout
/// holding `repository`, in one commit to its remote's `bench-history`; a
/// result without provenance, an old binary's, is skipped with a warning.
/// None when nothing was publishable.
pub fn publish_night(repository: &Path, night: &Path) -> anyhow::Result<Option<Publication>> {
    let mut results = Vec::new();
    for directory in run_directories(night)? {
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<_>>()?;
        paths.retain(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        });
        paths.sort();
        for path in paths {
            let result = read_result(&path)?;
            if result["provenance"].is_null() {
                tracing::warn!(path = %path.display(), "not published, no provenance");
                continue;
            }
            let measurements: Measurements = serde_json::from_value(result)
                .with_context(|| format!("parsing {}", path.display()))?;
            results.push(measurements);
        }
    }
    if results.is_empty() {
        return Ok(None);
    }
    publish(repository, &results, None).map(Some)
}

#[cfg(test)]
mod tests {
    use super::super::tests::{Fixture, commit};
    use super::super::verdicts::{Commit, Suspect};
    use super::*;
    use crate::bench::Checkout;
    use crate::bench::tests::{baseline, provenance};

    fn head(checkout: &Path) -> String {
        super::super::git(checkout, &["rev-parse", "HEAD"], None).unwrap()
    }

    /// A Night's directory beside `fixture`'s checkout.
    fn night(fixture: &Fixture) -> PathBuf {
        fixture.checkout.with_file_name("night")
    }

    #[test]
    fn run_directories_number_in_the_order_benched_and_name_their_commit() {
        let fixture = Fixture::new("night-numbered");
        let checkout = &fixture.checkout;
        let night = night(&fixture);
        let commit = head(checkout);
        let commit_of = |commit: &str| Commit {
            commit: commit.to_owned(),
            source_hash: "1".to_owned(),
        };
        let verdict = Verdict::of(Suspect {
            scene: "town".to_owned(),
            metric: "cpu_frame_microseconds".to_owned(),
            shift: 0.10,
            suspect: commit_of(&commit),
            predecessor: commit_of(&commit),
            committed_unix_seconds: 0,
        });
        start(&night, &verdict).unwrap();
        let directories: Vec<PathBuf> = (0..3)
            .map(|_| {
                run_after(checkout, &night, "town", &commit, &commit)
                    .unwrap()
                    .directory
            })
            .collect();
        assert_eq!(
            directories,
            ["001", "002", "003"].map(|name| night.join(name))
        );
        assert_eq!(
            std::fs::read_to_string(directories[2].join(RUN_COMMIT_FILE)).unwrap(),
            format!("{commit}\n")
        );
    }

    #[test]
    fn a_commit_before_no_publish_was_added_is_benched_without_it() {
        let fixture = Fixture::new("night-flag");
        let checkout = &fixture.checkout;
        let night = night(&fixture);
        let before = head(checkout);
        commit(checkout, "src/flag.rs", "fn flag() {}");
        let added = head(checkout);
        commit(checkout, "src/after.rs", "fn after() {}");
        let after = head(checkout);
        let arguments = |commit: &str| {
            run_after(checkout, &night, "town", commit, &added)
                .unwrap()
                .arguments
        };
        assert_eq!(arguments(&before), ["town"]);
        assert_eq!(arguments(&added), ["--no-publish", "town"]);
        assert_eq!(arguments(&after), ["--no-publish", "town"]);
        assert!(run_after(checkout, &night, "town", "0000000", &added).is_err());
    }

    fn write_run(night: &Path, run: &str, result: &serde_json::Value) {
        let directory = night.join(run);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("town.json"), result.to_string()).unwrap();
    }

    #[test]
    fn a_nights_runs_are_read_from_the_results_the_bench_writes() {
        let fixture = Fixture::new("night-read");
        let night = night(&fixture);
        for (run, median) in [("002", 200.0), ("001", 100.0)] {
            let result = serde_json::json!({
                "provenance": { "commit": "a" },
                "cpu_frame_microseconds": { "median": median },
            });
            write_run(&night, run, &result);
        }
        let read = runs(&night, "town", "cpu_frame_microseconds").unwrap();
        assert_eq!(
            read,
            [("a", 100.0), ("a", 200.0)].map(|(commit, median)| Run {
                commit: commit.to_owned(),
                median
            })
        );
    }

    /// A binary older than the bench's provenance stamps no commit, so the
    /// run's directory names it.
    #[test]
    fn a_run_without_provenance_is_of_the_commit_its_directory_names() {
        let fixture = Fixture::new("night-unstamped");
        let checkout = &fixture.checkout;
        let night = night(&fixture);
        let commit = head(checkout);
        let result = serde_json::json!({ "cpu_frame_microseconds": { "median": 1000.0 } });
        let run = run_after(checkout, &night, "town", &commit, &commit).unwrap();
        std::fs::write(run.directory.join("town.json"), result.to_string()).unwrap();
        write_run(&night, "002", &result);
        let read = runs(&night, "town", "cpu_frame_microseconds").unwrap();
        assert_eq!(
            read,
            [Run {
                commit,
                median: 1000.0
            }]
        );
    }

    #[test]
    fn publishing_a_night_skips_a_run_without_provenance_and_pushes_the_rest_once() {
        let fixture = Fixture::new("night-publish");
        let checkout = &fixture.checkout;
        let night = night(&fixture);
        assert_eq!(publish_night(checkout, &night).unwrap(), None);
        let stamped = |p95_over_median| Measurements {
            adapter: "Apple M4 Pro".to_owned(),
            provenance: Some(provenance(
                Checkout::read(checkout).unwrap(),
                p95_over_median,
            )),
            ..baseline()
        };
        write_run(&night, "001", &serde_json::to_value(stamped(1.02)).unwrap());
        write_run(
            &night,
            "002",
            &serde_json::json!({ "cpu_frame_microseconds": { "median": 1000.0 } }),
        );
        let mut crowd = stamped(1.01);
        crowd.scene = "crowd".to_owned();
        write_run(&night, "003", &serde_json::to_value(crowd).unwrap());
        let publication = publish_night(checkout, &night).unwrap().unwrap();
        assert_eq!(publication.on_series + publication.tagged, 2);
        assert_eq!(fixture.commits(), 1);
    }
}
