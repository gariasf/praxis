//! The Bench history (ADR 0024): bench results published as Bench records
//! on the orphan `bench-history` branch, beside the Series files the page
//! reads, rebuilt in the same commit.

mod import;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Context;

use super::provenance::{git, source_hash};
use super::{Configuration, Measurements, Spread, adapter_key};

pub use import::import_baselines;

const REMOTE: &str = "origin";
const BRANCH: &str = "bench-history";

/// The branch's directories of records on a Series and of the rest.
const ON_SERIES: &str = "records";
const TAGGED: &str = "tagged";

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
  in the order their commits were made, rebuilt in the commit that changes them.

A record in `records/` holds its key against a tagged one. Otherwise a record
whose path exists replaces it only when its GPU frame p95 over its median is
lower. A bad publish is undone with `git revert`.

`praxis history import` added the Baselines committed to `main` before the
history existed: each Baseline file a commit changed is a record of that
commit's source, its `provenance.source` `baseline-commit`. It holds what the
bench measured then, so a metric added later is absent from it and null in
its Series row.
";

/// A Bench record to publish: the file the branch keeps under its key, and
/// what decides between it and a record already there.
#[derive(Debug)]
struct BenchRecord {
    /// `<adapter>-<backend>`, as the branch's paths name them.
    adapter_key: String,
    scene: String,
    source_hash: String,
    /// Empty for a record on a Series.
    tags: Vec<String>,
    /// The GPU frame p95 over its median; none when not measured.
    p95_over_median: Option<f64>,
    /// The file's JSON object.
    file: serde_json::Map<String, serde_json::Value>,
}

impl BenchRecord {
    /// The record of `file`, a bench result's JSON object naming its
    /// adapter, backend and scene, measured on `source_hash` committed at
    /// `committed_unix_seconds`.
    fn new(
        mut file: serde_json::Map<String, serde_json::Value>,
        source_hash: &str,
        committed_unix_seconds: u64,
        tags: Vec<String>,
        p95_over_median: Option<f64>,
    ) -> anyhow::Result<Self> {
        let field = |name: &str| {
            file.get(name)
                .and_then(serde_json::Value::as_str)
                .with_context(|| format!("a bench result names its {name}"))
        };
        let (adapter_key, scene) = (
            adapter_key(field("adapter")?, Some(field("backend")?)),
            field("scene")?.to_owned(),
        );
        file.insert(
            "committed_unix_seconds".to_owned(),
            committed_unix_seconds.into(),
        );
        if !tags.is_empty() {
            file.insert("tags".to_owned(), tags.clone().into());
        }
        Ok(Self {
            adapter_key,
            scene,
            source_hash: source_hash.to_owned(),
            tags,
            p95_over_median,
            file,
        })
    }

    /// The record's path under `directory`, `ON_SERIES` or `TAGGED`: its
    /// key. The Harness version it holds is not in the path, as it is
    /// defined under `src/`, so a source hash names one.
    fn path(&self, directory: &str) -> PathBuf {
        Path::new(directory)
            .join(&self.adapter_key)
            .join(&self.scene)
            .join(format!("{}.json", self.source_hash))
    }

    /// The GPU frame p95 over its median: lower is quieter.
    fn p95_over_median(&self) -> f64 {
        self.p95_over_median.unwrap_or(f64::INFINITY)
    }
}

/// A Bench record as the branch stores it, whether a bench published it or
/// the import made it from a Baseline, which lacks every metric added
/// after it was recorded.
#[derive(Debug, serde::Deserialize)]
struct StoredRecord {
    /// When the measured commit was committed, which orders a Series.
    committed_unix_seconds: u64,
    provenance: StoredProvenance,
    cpu_frame_microseconds: Option<Spread>,
    dynamic_tlas_cpu_microseconds: Option<Spread>,
    gpu_frame_microseconds: Option<Spread>,
    gate_microseconds: Option<Spread>,
    shadow_ray_microseconds: Option<Spread>,
    #[serde(default)]
    gpu_pass_microseconds: BTreeMap<String, Spread>,
    counter_frames: Option<StoredCounterFrames>,
    aliasing_rmse: Option<u64>,
    aliasing_p99: Option<u64>,
}

/// What every stored record's provenance holds, whichever form wrote it.
#[derive(Debug, serde::Deserialize)]
struct StoredProvenance {
    commit: String,
    source_hash: String,
    harness_version: u32,
    /// `baseline-commit` for an imported Baseline; none for a bench's
    /// result.
    source: Option<String>,
    /// None for an imported Baseline.
    scene_content_hash: Option<String>,
    gpu_frame_p95_over_median: Option<f64>,
}

/// The counter frames' counters by name, as many as the bench counted.
#[derive(Debug, serde::Deserialize)]
struct StoredCounterFrames {
    #[serde(rename = "frames")]
    _frames: serde::de::IgnoredAny,
    #[serde(flatten)]
    counters: BTreeMap<String, u64>,
}

impl StoredRecord {
    /// The GPU frame p95 over its median: lower is quieter.
    fn p95_over_median(&self) -> f64 {
        self.provenance
            .gpu_frame_p95_over_median
            .unwrap_or(f64::INFINITY)
    }
}

/// One Series file's row: a record's source and its metrics, without the
/// rest of its provenance. A metric its bench did not measure is null, and
/// a counter it did not count is absent.
#[derive(serde::Serialize)]
struct SeriesRow<'record> {
    commit: &'record str,
    source_hash: &'record str,
    committed_unix_seconds: u64,
    harness_version: u32,
    source: Option<&'record str>,
    scene_content_hash: Option<&'record str>,
    cpu_frame_microseconds: Option<Spread>,
    dynamic_tlas_cpu_microseconds: Option<Spread>,
    gpu_frame_microseconds: Option<Spread>,
    gate_microseconds: Option<Spread>,
    shadow_ray_microseconds: Option<Spread>,
    gpu_pass_median_microseconds: BTreeMap<&'record str, f64>,
    /// The counter frames' counters.
    #[serde(flatten)]
    counters: BTreeMap<&'record str, u64>,
    aliasing_rmse: Option<u64>,
    aliasing_p99: Option<u64>,
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
    /// Records already on the branch given the current Harness version.
    pub restamped: usize,
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
        if self.restamped > 0 {
            write!(formatter, ", re-stamped {}", self.restamped)?;
        }
        match &self.commit {
            Some(commit) => write!(formatter, " ({commit})"),
            None => write!(formatter, " (nothing changed)"),
        }
    }
}

/// Publishes `results`, measured in the checkout holding `repository`, as
/// Bench records on its remote's `bench-history` branch, which it creates
/// as an orphan when the remote lacks it. A result without provenance is
/// skipped with a warning.
pub fn publish(repository: &Path, results: &[Measurements]) -> anyhow::Result<Publication> {
    let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
    let repository = Path::new(&root);
    let main = fetch_main(repository)?;
    let main_source_hash = source_hash(repository, &main)?;
    let records: Vec<BenchRecord> = results
        .iter()
        .filter_map(|measurements| {
            record(repository, measurements, &main, &main_source_hash)
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
    let adapter_keys: BTreeSet<_> = records
        .iter()
        .map(|record| record.adapter_key.as_str())
        .collect();
    let message = format!(
        "Publish {} Bench records from {}",
        records.len(),
        adapter_keys.into_iter().collect::<Vec<_>>().join(", ")
    );
    push(repository, &message, |worktree| worktree.apply(&records))
}

/// Fetches the remote's `main`, and returns its remote-tracking ref.
fn fetch_main(repository: &Path) -> anyhow::Result<String> {
    let main = format!("{REMOTE}/main");
    git(
        repository,
        &[
            "fetch",
            "--quiet",
            REMOTE,
            &format!("+refs/heads/main:refs/remotes/{main}"),
        ],
        None,
    )?;
    Ok(main)
}

/// Commits what `write` writes into a worktree of the branch, outside the
/// checkout, and pushes it with `message`. When another publish pushes
/// first, it writes and commits again on the new tip.
fn push(
    repository: &Path,
    message: &str,
    write: impl Fn(&Worktree) -> anyhow::Result<Publication>,
) -> anyhow::Result<Publication> {
    let mut attempt = 1;
    let mut tip = remote_tip(repository)?;
    loop {
        let worktree = Worktree::add(repository, tip.as_deref())?;
        let mut publication = write(&worktree)?;
        publication.commit = worktree.commit(tip.as_deref(), message)?;
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
        let Err(error) = pushed else {
            return Ok(publication);
        };
        // Only a tip that moved since the fetch is a race worth another try.
        let current = remote_tip(repository)?;
        if current == tip || attempt == PUSH_ATTEMPTS {
            return Err(error);
        }
        tracing::info!(
            attempt,
            "bench-history moved; publishing again on its new tip"
        );
        tip = current;
        attempt += 1;
    }
}

/// `measurements` as a Bench record, tagged with every reason it is kept
/// off a Series.
fn record(
    repository: &Path,
    measurements: &Measurements,
    main: &str,
    main_source_hash: &str,
) -> anyhow::Result<BenchRecord> {
    let provenance = measurements
        .provenance
        .as_ref()
        .context("no provenance: it ran outside a git checkout")?;
    let checkout = &provenance.checkout;
    let mains_source = checkout.source_hash == main_source_hash
        || git(
            repository,
            &["merge-base", "--is-ancestor", &checkout.commit, main],
            None,
        )
        .is_ok();
    let serde_json::Value::Object(file) = serde_json::to_value(measurements)? else {
        anyhow::bail!("a bench result is a JSON object");
    };
    BenchRecord::new(
        file,
        &checkout.source_hash,
        committed_unix_seconds(repository, &checkout.commit)?,
        tags(
            checkout.dirty,
            &measurements.profile,
            provenance.quiet,
            mains_source,
            (
                measurements.width,
                measurements.height,
                measurements.measured_frames,
            ),
        ),
        provenance.gpu_frame_p95_over_median,
    )
}

/// Every reason a run is kept off a Series: a dirty tree, a profile other
/// than release, a machine that was not quiet, a source other than
/// `main`'s, and a size and frame count other than the reference
/// configuration's.
fn tags(
    dirty: bool,
    profile: &str,
    quiet: bool,
    mains_source: bool,
    (width, height, measured_frames): (u32, u32, u32),
) -> Vec<String> {
    let reference = Configuration::REFERENCE;
    [
        (dirty, "dirty tree".to_owned()),
        (profile != "release", format!("{profile} profile")),
        (!quiet, "machine not quiet".to_owned()),
        (!mains_source, "source not main's".to_owned()),
        (
            (width, height, measured_frames)
                != (reference.width, reference.height, reference.measured_frames),
            format!(
                "{width}x{height} over {measured_frames} frames, not the reference configuration"
            ),
        ),
    ]
    .into_iter()
    .filter_map(|(applies, tag)| applies.then_some(tag))
    .collect()
}

/// When `commit` was committed, in seconds since the Unix epoch.
fn committed_unix_seconds(repository: &Path, commit: &str) -> anyhow::Result<u64> {
    git(
        repository,
        &["show", "--no-patch", "--format=%ct", commit],
        None,
    )?
    .parse()
    .context("a commit time")
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
        // A process killed mid-publish leaves its worktree, which a later
        // process of the same id would collide with.
        std::fs::remove_dir_all(&directory).ok();
        git(repository, &["worktree", "prune"], None)?;
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

    /// Writes `records` into the worktree and rebuilds the Series files
    /// they join. A record on a Series holds its key against a tagged one;
    /// otherwise a record replaces one under its key only when quieter.
    fn apply(&self, records: &[BenchRecord]) -> anyhow::Result<Publication> {
        let mut publication = Publication {
            on_series: 0,
            tagged: 0,
            dropped: 0,
            restamped: 0,
            commit: None,
        };
        let mut series = BTreeSet::new();
        for record in records {
            let on_series = record.tags.is_empty();
            let series_path = self.directory.join(record.path(ON_SERIES));
            let tagged_path = self.directory.join(record.path(TAGGED));
            let relative_path = record.path(if on_series { ON_SERIES } else { TAGGED });
            if !on_series && series_path.exists() {
                tracing::info!(
                    record = %relative_path.display(),
                    "Bench record dropped: one on a Series holds its key"
                );
                publication.dropped += 1;
                continue;
            }
            let path = self.directory.join(&relative_path);
            let contents = serde_json::to_string_pretty(&record.file)? + "\n";
            if let Ok(json) = std::fs::read(&path) {
                let existing: StoredRecord = serde_json::from_slice(&json)
                    .with_context(|| format!("parsing {}", relative_path.display()))?;
                // serde_json may parse a float back a bit off what it wrote,
                // so the same record is found by its contents.
                if json == contents.as_bytes()
                    || record.p95_over_median() >= existing.p95_over_median()
                {
                    tracing::info!(
                        record = %relative_path.display(),
                        p95_over_median = record.p95_over_median(),
                        kept_p95_over_median = existing.p95_over_median(),
                        "Bench record dropped: the one under its key is as quiet"
                    );
                    publication.dropped += 1;
                    continue;
                }
            }
            let directory = relative_path.parent().context("a record has a directory")?;
            std::fs::create_dir_all(self.directory.join(directory))?;
            std::fs::write(&path, contents)?;
            if on_series {
                if tagged_path.exists() {
                    std::fs::remove_file(&tagged_path)?;
                }
                publication.on_series += 1;
                series.insert(directory.to_owned());
            } else {
                publication.tagged += 1;
            }
        }
        for directory in series {
            self.rebuild_series(&directory)?;
        }
        std::fs::write(self.directory.join("README.md"), README)?;
        Ok(publication)
    }

    /// Rebuilds the Series file of the records in `directory`,
    /// `records/<adapter>-<backend>/<scene>`, one row per line in the
    /// order their commits were made.
    fn rebuild_series(&self, directory: &Path) -> anyhow::Result<()> {
        let mut records = Vec::new();
        for entry in std::fs::read_dir(self.directory.join(directory))? {
            let path = entry?.path();
            let json = std::fs::read(&path)?;
            let record: StoredRecord = serde_json::from_slice(&json)
                .with_context(|| format!("parsing {}", path.display()))?;
            records.push(record);
        }
        let mut rows: Vec<_> = records.iter().map(SeriesRow::of).collect();
        rows.sort_by_key(|row| (row.committed_unix_seconds, row.commit));
        let lines = rows
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?;
        let series = Path::new("series")
            .join(directory.strip_prefix(ON_SERIES)?)
            .with_extension("json");
        let path = self.directory.join(series);
        std::fs::create_dir_all(path.parent().context("a Series has a directory")?)?;
        std::fs::write(path, format!("[\n{}\n]\n", lines.join(",\n")))?;
        Ok(())
    }

    /// Commits the worktree's files onto `tip` with `message`, or as an
    /// orphan with no tip; none when its tree is `tip`'s.
    fn commit(&self, tip: Option<&str>, message: &str) -> anyhow::Result<Option<String>> {
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
        let mut arguments = vec!["commit-tree", &tree, "-m", message];
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
    fn of(record: &'record StoredRecord) -> Self {
        let provenance = &record.provenance;
        Self {
            commit: &provenance.commit,
            source_hash: &provenance.source_hash,
            committed_unix_seconds: record.committed_unix_seconds,
            harness_version: provenance.harness_version,
            source: provenance.source.as_deref(),
            scene_content_hash: provenance.scene_content_hash.as_deref(),
            cpu_frame_microseconds: record.cpu_frame_microseconds,
            dynamic_tlas_cpu_microseconds: record.dynamic_tlas_cpu_microseconds,
            gpu_frame_microseconds: record.gpu_frame_microseconds,
            gate_microseconds: record.gate_microseconds,
            shadow_ray_microseconds: record.shadow_ray_microseconds,
            gpu_pass_median_microseconds: record
                .gpu_pass_microseconds
                .iter()
                .map(|(pass, spread)| (pass.as_str(), spread.median))
                .collect(),
            counters: record
                .counter_frames
                .iter()
                .flat_map(|frames| &frames.counters)
                .map(|(counter, value)| (counter.as_str(), *value))
                .collect(),
            aliasing_rmse: record.aliasing_rmse,
            aliasing_p99: record.aliasing_p99,
        }
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
    /// `p95_over_median`, quiet below `QUIET_P95_OVER_MEDIAN`.
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
            let record: StoredRecord = serde_json::from_str(&fixture.read(&path)).unwrap();
            record.p95_over_median()
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
            let record: serde_json::Value = serde_json::from_str(&fixture.read(&format!(
                "tagged/apple-m4-pro-metal/pbr-sweep/{source_hash}.json"
            )))
            .unwrap();
            assert_eq!(record["tags"], serde_json::json!(tags));
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
    #[test]
    fn a_record_on_a_series_holds_its_key_against_a_tagged_one() {
        let fixture = Fixture::new("held");
        let paths = |directory: &str| {
            fixture
                .files()
                .into_iter()
                .filter(|path| path.starts_with(directory))
                .count()
        };
        let quiet = result(&fixture.checkout, "Apple M4 Pro", 1.02);
        let noisy = result(&fixture.checkout, "Apple M4 Pro", 1.10);
        let counters = Measurements {
            measured_frames: Configuration::COUNTERS.measured_frames,
            ..result(&fixture.checkout, "Apple M4 Pro", 1.01)
        };
        let publication = publish(&fixture.checkout, std::slice::from_ref(&noisy)).unwrap();
        assert_eq!((publication.on_series, publication.tagged), (0, 1));
        // A quiet record of the same key replaces the tagged one.
        publish(&fixture.checkout, &[quiet]).unwrap();
        assert_eq!((paths("records/"), paths("tagged/")), (1, 0));
        // And neither a noisy run nor a quieter one of another configuration
        // displaces it.
        let publication = publish(&fixture.checkout, &[noisy, counters]).unwrap();
        assert_eq!(publication.dropped, 2);
        assert_eq!((paths("records/"), paths("tagged/")), (1, 0));
    }

    #[test]
    fn a_run_of_another_configuration_is_tagged_with_it() {
        let fixture = Fixture::new("configuration");
        let counters = Measurements {
            measured_frames: Configuration::COUNTERS.measured_frames,
            ..result(&fixture.checkout, "Apple M4 Pro", 1.01)
        };
        let source_hash = &counters.provenance.as_ref().unwrap().checkout.source_hash;
        publish(&fixture.checkout, std::slice::from_ref(&counters)).unwrap();
        let record: serde_json::Value = serde_json::from_str(&fixture.read(&format!(
            "tagged/apple-m4-pro-metal/pbr-sweep/{source_hash}.json"
        )))
        .unwrap();
        assert_eq!(
            record["tags"],
            serde_json::json!(["2560x1440 over 64 frames, not the reference configuration"])
        );
    }

    #[test]
    fn an_import_records_each_changed_baseline_once_and_restamps_earlier_records() {
        let fixture = Fixture::new("import");
        let checkout = &fixture.checkout;
        // A record a bench published at Harness version 1, before the import.
        let live = result(checkout, "Apple M4 Pro", 1.02);
        publish(checkout, std::slice::from_ref(&live)).unwrap();
        let baseline = |adapter: &str, width: u32, median: f64| {
            format!(
                r#"{{"scene": "pbr-sweep", "adapter": "{adapter}", "profile": "release",
                "width": {width}, "height": 1440, "measured_frames": 1800,
                "cpu_frame_microseconds": {{"median": {median}, "p95": 40.0}}}}"#
            )
        };
        let metal = "bench/baselines/apple-m4-pro/pbr-sweep.json";
        let lavapipe = "bench/baselines/llvmpipe/pbr-sweep.json";
        commit(checkout, "src/lib.rs", "fn main() { first() }");
        commit(checkout, metal, &baseline("Apple M4 Pro", 2560, 30.0));
        commit(
            checkout,
            lavapipe,
            &baseline("llvmpipe (LLVM 20.1.2)", 640, 900.0),
        );
        commit(checkout, "src/bench.rs", "// a metric's meaning changes");
        let harness_break = git(checkout, &["rev-parse", "HEAD"], None).unwrap();
        commit(checkout, metal, &baseline("Apple M4 Pro", 2560, 20.0));
        // A rename alone measures nothing again.
        git(
            checkout,
            &["mv", lavapipe, "bench/baselines/llvmpipe/renamed.json"],
            None,
        )
        .unwrap();
        git(
            checkout,
            &["commit", "--quiet", "--message", "rename"],
            None,
        )
        .unwrap();
        commit(checkout, "src/history.rs", "// the history starts");
        let history_started = git(checkout, &["rev-parse", "HEAD"], None).unwrap();
        commit(checkout, metal, &baseline("Apple M4 Pro", 2560, 10.0));

        let first = import::import_up_to(checkout, &history_started, &[&harness_break]).unwrap();
        assert_eq!(
            (
                first.on_series,
                first.tagged,
                first.dropped,
                first.restamped
            ),
            (2, 1, 0, 1)
        );
        let series = fixture.read("series/apple-m4-pro-metal/pbr-sweep.json");
        let rows: Vec<serde_json::Value> = serde_json::from_str(&series).unwrap();
        // The fixture's commits share a second, so the Series' order ties.
        let mut summary: Vec<_> = rows
            .iter()
            .map(|row| {
                (
                    row["harness_version"].as_u64().unwrap(),
                    row["source"].as_str(),
                    row["cpu_frame_microseconds"]["median"].as_f64().unwrap(),
                )
            })
            .collect();
        summary.sort_by(|left, right| left.partial_cmp(right).unwrap());
        assert_eq!(
            summary,
            [
                (1, Some("baseline-commit"), 30.0),
                (2, None, 1000.0),
                (2, Some("baseline-commit"), 20.0),
            ]
        );

        let commits = fixture.commits();
        let second = import::import_up_to(checkout, &history_started, &[&harness_break]).unwrap();
        assert_eq!(
            (second.dropped, second.restamped, second.commit),
            (3, 0, None)
        );
        assert_eq!(fixture.commits(), commits);
    }
}
