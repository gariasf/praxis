//! The Bench history (ADR 0024): bench results published as Bench records
//! on the orphan `bench-history` branch, beside the Series files the page
//! reads, rebuilt in the same commit.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Context;

use super::provenance::{git, source_hash};
use super::{Aliasing, Counters, Measurements, Spread, adapter_key};

const REMOTE: &str = "origin";
const BRANCH: &str = "bench-history";

/// How many times a publish rebuilds its commit on the branch's new tip
/// after another publish pushed first.
const PUSH_ATTEMPTS: u32 = 5;

/// What the branch's README says: how it is laid out.
const README: &str = "# Bench history

Bench records published by `praxis publish` and `praxis bench` (ADR 0024).
This branch holds data only and is never merged.

- `records/<adapter>-<backend>/<scene>/<source hash>.json`: the records on a
  Series, clean and release-profile runs of `main`'s source on a quiet machine.
- `tagged/<adapter>-<backend>/<scene>/<source hash>.json`: every other record,
  its `tags` saying why it is kept off the Series.
- `series/<adapter>-<backend>/<scene>.json`: one row per record in `records/`,
  in source order, rebuilt in the commit that changes them.

A record whose path exists replaces it only when its GPU frame p95 over its
median is lower. A bad publish is undone with `git revert`.
";

/// A Bench record as the branch stores it: a bench result, when its commit
/// was made, and why it is kept off every Series.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BenchRecord {
    #[serde(flatten)]
    measurements: Measurements,
    /// When the measured commit was committed, which orders a Series.
    committed_unix_seconds: u64,
    /// Empty for a record on a Series.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

impl BenchRecord {
    /// The record's path under `records/` or `tagged/`: its key, but for
    /// the Harness version it holds.
    fn path(&self) -> anyhow::Result<PathBuf> {
        let measurements = &self.measurements;
        let provenance = measurements
            .provenance
            .as_ref()
            .context("a Bench record has provenance")?;
        Ok(PathBuf::from(if self.tags.is_empty() {
            "records"
        } else {
            "tagged"
        })
        .join(adapter_key(
            &measurements.adapter,
            Some(&measurements.backend),
        ))
        .join(&measurements.scene)
        .join(format!("{}.json", provenance.checkout.source_hash)))
    }

    /// The GPU frame p95 over its median: lower is quieter.
    fn spread(&self) -> f64 {
        self.measurements
            .provenance
            .as_ref()
            .and_then(|provenance| provenance.gpu_frame_p95_over_median)
            .unwrap_or(f64::INFINITY)
    }
}

/// One Series file's row: a record's source and its metrics, without the
/// rest of its provenance.
#[derive(serde::Serialize)]
struct SeriesRow<'record> {
    commit: &'record str,
    source_hash: &'record str,
    committed_unix_seconds: u64,
    harness_version: u32,
    scene_content_hash: &'record str,
    cpu_frame_microseconds: Spread,
    dynamic_tlas_cpu_microseconds: Spread,
    gpu_frame_microseconds: Spread,
    gate_microseconds: Spread,
    shadow_ray_microseconds: Spread,
    gpu_pass_median_microseconds: BTreeMap<&'record str, f64>,
    /// The counter frames' counters.
    #[serde(flatten)]
    counters: Counters,
    #[serde(flatten)]
    aliasing: Option<Aliasing>,
}

/// What a publish did.
#[derive(Debug, PartialEq)]
pub struct Publication {
    /// Records written under `records/`, so on a Series.
    pub on_series: usize,
    /// Records written under `tagged/`.
    pub tagged: usize,
    /// Records dropped for one at least as quiet under their key.
    pub dropped: usize,
    /// The commit pushed; none when nothing changed.
    pub commit: Option<String>,
}

impl std::fmt::Display for Publication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "published {} Bench records on a Series and {} tagged to {BRANCH}, dropped {}",
            self.on_series, self.tagged, self.dropped
        )?;
        match &self.commit {
            Some(commit) => write!(formatter, " ({commit})"),
            None => write!(formatter, " (nothing changed)"),
        }
    }
}

/// Publishes `results`, measured in the checkout holding `repository`, as
/// Bench records on its remote's `bench-history` branch, which it creates
/// as an orphan when the remote lacks it. It works in a worktree of the
/// branch outside the checkout, and when another publish pushes first it
/// builds its commit again on the new tip. A result without provenance is
/// skipped with a warning.
pub fn publish(repository: &Path, results: &[Measurements]) -> anyhow::Result<Publication> {
    let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
    let repository = Path::new(&root);
    git(
        repository,
        &[
            "fetch",
            "--quiet",
            REMOTE,
            "+refs/heads/main:refs/remotes/origin/main",
        ],
        None,
    )?;
    let main_source_hash = source_hash(repository, "origin/main")?;
    let records: Vec<BenchRecord> = results
        .iter()
        .filter_map(|measurements| {
            record(repository, measurements, &main_source_hash)
                .inspect_err(|error| {
                    tracing::warn!(
                        scene = measurements.scene,
                        error = format!("{error:#}"),
                        "bench result not published"
                    )
                })
                .ok()
        })
        .collect();
    let mut attempt = 1;
    loop {
        let tip = remote_tip(repository)?;
        let worktree = Worktree::add(repository, tip.as_deref())?;
        let mut publication = worktree.apply(&records)?;
        publication.commit = worktree.commit(tip.as_deref(), &records)?;
        let Some(commit) = &publication.commit else {
            return Ok(publication);
        };
        let pushed = git(
            repository,
            &[
                "push",
                "--quiet",
                REMOTE,
                &format!("{commit}:refs/heads/{BRANCH}"),
            ],
            None,
        );
        match pushed {
            Ok(_) => return Ok(publication),
            Err(error) if attempt < PUSH_ATTEMPTS => {
                tracing::info!(
                    attempt,
                    error = format!("{error:#}"),
                    "bench-history moved; publishing again on its new tip"
                );
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

/// `measurements` as a Bench record, tagged with every reason it is kept
/// off a Series.
fn record(
    repository: &Path,
    measurements: &Measurements,
    main_source_hash: &str,
) -> anyhow::Result<BenchRecord> {
    let provenance = measurements
        .provenance
        .as_ref()
        .context("no provenance: it ran outside a git checkout")?;
    let checkout = &provenance.checkout;
    let committed_unix_seconds = git(
        repository,
        &["show", "--no-patch", "--format=%ct", &checkout.commit],
        None,
    )?
    .parse()
    .context("a commit time")?;
    let mains_source = checkout.source_hash == main_source_hash
        || git(
            repository,
            &[
                "merge-base",
                "--is-ancestor",
                &checkout.commit,
                "origin/main",
            ],
            None,
        )
        .is_ok();
    let tags = [
        (checkout.dirty, "dirty tree".to_owned()),
        (
            measurements.profile != "release",
            format!("{} profile", measurements.profile),
        ),
        (!provenance.quiet, "machine not quiet".to_owned()),
        (!mains_source, "source not main's".to_owned()),
    ]
    .into_iter()
    .filter_map(|(applies, tag)| applies.then_some(tag))
    .collect();
    Ok(BenchRecord {
        measurements: measurements.clone(),
        committed_unix_seconds,
        tags,
    })
}

/// The remote's `bench-history` tip, fetched; none when it has no such
/// branch.
fn remote_tip(repository: &Path) -> anyhow::Result<Option<String>> {
    let listing = git(repository, &["ls-remote", "--heads", REMOTE, BRANCH], None)?;
    let Some((tip, _)) = listing.split_once('\t') else {
        return Ok(None);
    };
    git(
        repository,
        &[
            "fetch",
            "--quiet",
            REMOTE,
            &format!("+refs/heads/{BRANCH}:refs/remotes/{REMOTE}/{BRANCH}"),
        ],
        None,
    )?;
    Ok(Some(tip.to_owned()))
}

/// A detached worktree of the branch in a temporary directory, removed on
/// drop.
struct Worktree<'repository> {
    repository: &'repository Path,
    directory: PathBuf,
}

impl<'repository> Worktree<'repository> {
    /// A worktree checked out at `tip`, or with no tip an empty one.
    fn add(repository: &'repository Path, tip: Option<&str>) -> anyhow::Result<Self> {
        static WORKTREES: AtomicU32 = AtomicU32::new(0);
        let directory = std::env::temp_dir().join(format!(
            "praxis-bench-history-{}-{}",
            std::process::id(),
            WORKTREES.fetch_add(1, Ordering::Relaxed)
        ));
        let path = directory.to_str().context("a UTF-8 temporary directory")?;
        match tip {
            Some(tip) => git(
                repository,
                &["worktree", "add", "--quiet", "--detach", path, tip],
                None,
            )?,
            None => git(
                repository,
                &[
                    "worktree",
                    "add",
                    "--quiet",
                    "--detach",
                    "--no-checkout",
                    path,
                ],
                None,
            )?,
        };
        let worktree = Self {
            repository,
            directory,
        };
        if tip.is_none() {
            git(&worktree.directory, &["read-tree", "--empty"], None)?;
        }
        Ok(worktree)
    }

    /// Writes `records` into the worktree, a record whose path exists only
    /// when it is quieter, and rebuilds the Series files they join.
    fn apply(&self, records: &[BenchRecord]) -> anyhow::Result<Publication> {
        let mut publication = Publication {
            on_series: 0,
            tagged: 0,
            dropped: 0,
            commit: None,
        };
        let mut series = BTreeSet::new();
        for record in records {
            let relative = record.path()?;
            let path = self.directory.join(&relative);
            if let Ok(json) = std::fs::read(&path) {
                let existing: BenchRecord = serde_json::from_slice(&json)
                    .with_context(|| format!("parsing {}", relative.display()))?;
                if record.spread() >= existing.spread() {
                    tracing::info!(
                        record = %relative.display(),
                        p95_over_median = record.spread(),
                        kept_p95_over_median = existing.spread(),
                        "Bench record dropped: the one under its key is as quiet"
                    );
                    publication.dropped += 1;
                    continue;
                }
            }
            std::fs::create_dir_all(path.parent().context("a record has a directory")?)?;
            std::fs::write(&path, serde_json::to_string_pretty(record)? + "\n")?;
            if record.tags.is_empty() {
                publication.on_series += 1;
                series.insert(relative.parent().map(Path::to_owned));
            } else {
                publication.tagged += 1;
            }
        }
        for directory in series.into_iter().flatten() {
            self.rebuild_series(&directory)?;
        }
        std::fs::write(self.directory.join("README.md"), README)?;
        Ok(publication)
    }

    /// Rebuilds the Series file of the records in `directory`,
    /// `records/<adapter>-<backend>/<scene>`, one row per line in source
    /// order.
    fn rebuild_series(&self, directory: &Path) -> anyhow::Result<()> {
        let mut records = Vec::new();
        for entry in std::fs::read_dir(self.directory.join(directory))? {
            let path = entry?.path();
            let json = std::fs::read(&path)?;
            let record: BenchRecord = serde_json::from_slice(&json)
                .with_context(|| format!("parsing {}", path.display()))?;
            records.push(record);
        }
        let mut rows: Vec<_> = records.iter().filter_map(SeriesRow::of).collect();
        rows.sort_by_key(|row| (row.committed_unix_seconds, row.commit));
        let lines = rows
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?;
        let series = Path::new("series")
            .join(directory.strip_prefix("records")?)
            .with_extension("json");
        let path = self.directory.join(series);
        std::fs::create_dir_all(path.parent().context("a Series has a directory")?)?;
        std::fs::write(path, format!("[\n{}\n]\n", lines.join(",\n")))?;
        Ok(())
    }

    /// Commits the worktree's files onto `tip`, or as an orphan with no
    /// tip; none when its tree is `tip`'s.
    fn commit(&self, tip: Option<&str>, records: &[BenchRecord]) -> anyhow::Result<Option<String>> {
        git(&self.directory, &["add", "--all"], None)?;
        let tree = git(&self.directory, &["write-tree"], None)?;
        if let Some(tip) = tip
            && git(
                &self.directory,
                &["rev-parse", &format!("{tip}^{{tree}}")],
                None,
            )? == tree
        {
            return Ok(None);
        }
        let sources: BTreeSet<_> = records
            .iter()
            .map(|record| {
                adapter_key(
                    &record.measurements.adapter,
                    Some(&record.measurements.backend),
                )
            })
            .collect();
        let message = format!(
            "Publish {} Bench records from {}",
            records.len(),
            sources.into_iter().collect::<Vec<_>>().join(", ")
        );
        let mut arguments = vec!["commit-tree", &tree, "-m", &message];
        if let Some(tip) = tip {
            arguments.extend(["-p", tip]);
        }
        git(&self.directory, &arguments, None).map(Some)
    }
}

impl Drop for Worktree<'_> {
    fn drop(&mut self) {
        let removed = self.directory.to_str().map(|path| {
            git(
                self.repository,
                &["worktree", "remove", "--force", path],
                None,
            )
        });
        if let Some(Err(error)) = removed {
            tracing::warn!(
                directory = %self.directory.display(),
                error = format!("{error:#}"),
                "bench-history worktree left behind"
            );
        }
    }
}

impl<'record> SeriesRow<'record> {
    /// `record`'s row; none for a record without provenance, which no
    /// publish writes.
    fn of(record: &'record BenchRecord) -> Option<Self> {
        let measurements = &record.measurements;
        let provenance = measurements.provenance.as_ref()?;
        Some(Self {
            commit: &provenance.checkout.commit,
            source_hash: &provenance.checkout.source_hash,
            committed_unix_seconds: record.committed_unix_seconds,
            harness_version: provenance.harness_version,
            scene_content_hash: &provenance.scene_content_hash,
            cpu_frame_microseconds: measurements.cpu_frame_microseconds,
            dynamic_tlas_cpu_microseconds: measurements.dynamic_tlas_cpu_microseconds,
            gpu_frame_microseconds: measurements.gpu_frame_microseconds,
            gate_microseconds: measurements.gate_microseconds,
            shadow_ray_microseconds: measurements.shadow_ray_microseconds,
            gpu_pass_median_microseconds: measurements
                .gpu_pass_microseconds
                .iter()
                .map(|(pass, spread)| (pass.as_str(), spread.median))
                .collect(),
            counters: measurements.counter_frames.counters,
            aliasing: measurements.aliasing,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::Checkout;
    use crate::bench::tests::{baseline, provenance};

    /// A bare remote with `main` holding one commit, and a clone of it to
    /// bench in, in a fresh temporary directory.
    struct Fixture {
        directory: PathBuf,
        remote: PathBuf,
        checkout: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let directory =
                std::env::temp_dir().join(format!("praxis-history-{}-{name}", std::process::id()));
            std::fs::remove_dir_all(&directory).ok();
            std::fs::create_dir_all(&directory).unwrap();
            let remote = directory.join("remote.git");
            git(
                &directory,
                &["init", "--quiet", "--bare", "-b", "main", "remote.git"],
                None,
            )
            .unwrap();
            let fixture = Self {
                checkout: directory.join("checkout"),
                directory,
                remote,
            };
            fixture.clone_into(&fixture.checkout);
            commit(&fixture.checkout, "src/lib.rs", "fn main() {}");
            git(
                &fixture.checkout,
                &["push", "--quiet", "origin", "main"],
                None,
            )
            .unwrap();
            fixture
        }

        /// Clones the remote into `path`, with an identity to commit as.
        fn clone_into(&self, path: &Path) {
            git(
                &self.directory,
                &[
                    "clone",
                    "--quiet",
                    self.remote.to_str().unwrap(),
                    path.to_str().unwrap(),
                ],
                None,
            )
            .unwrap();
            for (key, value) in [
                ("user.name", "Bench"),
                ("user.email", "bench@example.com"),
                ("commit.gpgsign", "false"),
            ] {
                git(path, &["config", key, value], None).unwrap();
            }
        }

        /// The remote branch's files, by path.
        fn files(&self) -> Vec<String> {
            git(
                &self.remote,
                &["ls-tree", "-r", "--name-only", BRANCH],
                None,
            )
            .unwrap()
            .lines()
            .map(str::to_owned)
            .collect()
        }

        fn commits(&self) -> usize {
            git(&self.remote, &["rev-list", "--count", BRANCH], None)
                .unwrap()
                .parse()
                .unwrap()
        }

        fn read(&self, path: &str) -> String {
            git(&self.remote, &["show", &format!("{BRANCH}:{path}")], None).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.directory).ok();
        }
    }

    fn commit(repository: &Path, path: &str, contents: &str) {
        let file = repository.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, contents).unwrap();
        git(repository, &["add", "--all"], None).unwrap();
        git(repository, &["commit", "--quiet", "--message", path], None).unwrap();
    }

    /// A result of `checkout` on `adapter` whose GPU frames spread
    /// `p95_over_median`, quiet below 1.06.
    fn result(checkout: &Path, adapter: &str, p95_over_median: f64) -> Measurements {
        Measurements {
            adapter: adapter.to_owned(),
            provenance: Some(provenance(
                Checkout::read(checkout).unwrap(),
                p95_over_median,
            )),
            ..baseline()
        }
    }

    #[test]
    fn publishing_the_same_results_twice_leaves_one_record_and_one_commit() {
        let fixture = Fixture::new("twice");
        let results = [result(&fixture.checkout, "Apple M4 Pro", 1.02)];
        let first = publish(&fixture.checkout, &results).unwrap();
        assert_eq!((first.on_series, first.tagged, first.dropped), (1, 0, 0));
        let second = publish(&fixture.checkout, &results).unwrap();
        assert_eq!((second.dropped, second.commit), (1, None));
        assert_eq!(fixture.commits(), 1);
        let source_hash = &results[0].provenance.as_ref().unwrap().checkout.source_hash;
        assert_eq!(
            fixture.files(),
            [
                "README.md".to_owned(),
                format!("records/apple-m4-pro-metal/pbr-sweep/{source_hash}.json"),
                "series/apple-m4-pro-metal/pbr-sweep.json".to_owned(),
            ]
        );
        let series = fixture.read("series/apple-m4-pro-metal/pbr-sweep.json");
        assert_eq!(series.lines().count(), 3, "{series}");
        assert!(series.contains(source_hash), "{series}");
        // The checkout's own branches and worktrees are as they were.
        assert_eq!(
            git(&fixture.checkout, &["worktree", "list"], None)
                .unwrap()
                .lines()
                .count(),
            1
        );
    }
    #[test]
    fn a_quieter_record_replaces_a_noisier_one_and_a_noisier_one_is_dropped() {
        let fixture = Fixture::new("quieter");
        let spread_on_the_branch = || {
            let path = fixture
                .files()
                .into_iter()
                .find(|path| path.starts_with("records/"))
                .unwrap();
            let record: BenchRecord = serde_json::from_str(&fixture.read(&path)).unwrap();
            record.spread()
        };
        for (p95_over_median, (on_series, dropped), kept) in [
            (1.04, (1, 0), 1.04),
            (1.02, (1, 0), 1.02),
            (1.03, (0, 1), 1.02),
        ] {
            let publication = publish(
                &fixture.checkout,
                &[result(&fixture.checkout, "Apple M4 Pro", p95_over_median)],
            )
            .unwrap();
            assert_eq!(
                (publication.on_series, publication.dropped),
                (on_series, dropped)
            );
            assert_eq!(spread_on_the_branch(), kept);
        }
        assert_eq!(fixture.commits(), 2);
    }
    #[test]
    fn a_dirty_or_branch_record_lands_in_tagged_with_its_reasons_and_on_no_series() {
        let fixture = Fixture::new("tagged");
        std::fs::write(fixture.checkout.join("src/lib.rs"), "fn main() { loop {} }").unwrap();
        let dirty = result(&fixture.checkout, "Apple M4 Pro", 1.02);
        git(
            &fixture.checkout,
            &["switch", "--quiet", "-c", "branch"],
            None,
        )
        .unwrap();
        commit(&fixture.checkout, "src/lib.rs", "fn main() { panic!() }");
        let branch = result(&fixture.checkout, "Apple M4 Pro", 1.10);
        let publication = publish(&fixture.checkout, &[dirty.clone(), branch.clone()]).unwrap();
        assert_eq!((publication.on_series, publication.tagged), (0, 2));
        assert!(
            fixture
                .files()
                .iter()
                .all(|path| path == "README.md" || path.starts_with("tagged/")),
            "{:?}",
            fixture.files()
        );
        for (result, tags) in [
            (dirty, &["dirty tree"][..]),
            (branch, &["machine not quiet", "source not main's"][..]),
        ] {
            let source_hash = &result.provenance.unwrap().checkout.source_hash;
            let record: BenchRecord = serde_json::from_str(&fixture.read(&format!(
                "tagged/apple-m4-pro-metal/pbr-sweep/{source_hash}.json"
            )))
            .unwrap();
            assert_eq!(record.tags, tags);
        }
    }
    #[test]
    fn two_publishes_from_different_adapters_racing_to_push_both_land() {
        let fixture = Fixture::new("race");
        let adapters = ["Apple M4 Pro", "NVIDIA GeForce RTX 2070 SUPER"];
        let checkouts = adapters.map(|adapter| {
            let checkout = fixture.directory.join(adapter_key(adapter, None));
            fixture.clone_into(&checkout);
            checkout
        });
        let scenes = ["pbr-sweep", "town", "crowd"];
        for scene in scenes {
            let start = std::sync::Barrier::new(adapters.len());
            std::thread::scope(|scope| {
                for (adapter, checkout) in adapters.iter().zip(&checkouts) {
                    let start = &start;
                    scope.spawn(move || {
                        let result = Measurements {
                            scene: scene.to_owned(),
                            ..result(checkout, adapter, 1.02)
                        };
                        start.wait();
                        publish(checkout, &[result]).unwrap();
                    });
                }
            });
        }
        let series: Vec<_> = fixture
            .files()
            .into_iter()
            .filter(|path| path.starts_with("series/"))
            .collect();
        let mut expected: Vec<_> = ["apple-m4-pro-metal", "nvidia-geforce-rtx-2070-super-metal"]
            .iter()
            .flat_map(|key| scenes.map(|scene| format!("series/{key}/{scene}.json")))
            .collect();
        expected.sort();
        assert_eq!(series, expected);
        assert_eq!(fixture.commits(), adapters.len() * scenes.len());
    }
    #[test]
    fn a_publish_that_cannot_reach_its_remote_fails() {
        let fixture = Fixture::new("unreachable");
        let results = [result(&fixture.checkout, "Apple M4 Pro", 1.02)];
        std::fs::rename(&fixture.remote, fixture.directory.join("moved.git")).unwrap();
        assert!(publish(&fixture.checkout, &results).is_err());
    }
}
