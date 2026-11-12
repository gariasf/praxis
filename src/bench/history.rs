//! The Bench history (ADR 0024): bench results published as Bench records
//! on the orphan `bench-history` branch, beside the Series files the page
//! reads, rebuilt in the same commit.

mod import;
mod night;
mod suspects;
mod verdicts;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::Context;

use super::provenance::{Checkout, git, source_hash};
use super::{Configuration, Counters, Measurements, Spread, adapter_key};
use crate::scenes::ReferenceScene;

pub use import::import_baselines;
pub use night::{NextRun, next_run, publish_night};
pub use verdicts::{GitHubIssues, IssueTracker, Suspect, bisect, file_regressions, suspects};

const REMOTE: &str = "origin";
const BRANCH: &str = "bench-history";

/// The branch's directories of records on a Series and of the rest.
const ON_SERIES: &str = "records";
const TAGGED: &str = "tagged";

/// The branch's directory of Experiments, one directory each by name.
const EXPERIMENTS: &str = "experiments";

/// The directory, in place of a scene's, of the release builds published
/// beside an adapter's results.
const RELEASE_BUILD: &str = "release-build";

/// How Mesa's lavapipe, CI's software Vulkan, names its adapter.
const LAVAPIPE_ADAPTER_PREFIX: &str = "llvmpipe";

/// How many times a publish pushes: each attempt after the first rebuilds
/// its commit on the branch's new tip, as another publish pushed first.
const PUSH_ATTEMPTS: u32 = 5;

/// What the branch's README says: how it is laid out.
const README: &str = "# Bench history

Bench records published by `praxis publish` and `praxis bench`, and the
Baselines `praxis history import` added from `main`'s history (ADR 0024).
This branch holds data only and is never merged.

- `records/<adapter>-<backend>/<scene>/<source hash>.json`: the records on a
  Series, clean and release-profile runs of `main`'s source in their Series'
  configuration: 2560x1440 over 1800 frames on a quiet machine,
  `world` over the 3352 frames its bench segment measures, or on lavapipe
  CI's 640x360 over 64 frames, quiet or not.
- `tagged/<adapter>-<backend>/<scene>/<source hash>.json`: every other record,
  its `tags` saying why it is kept off the Series.
- `records/<adapter>-<backend>/release-build/<source hash>.json`: the release
  build whose binary ran that source's results, its binary's size and its wall
  time, published by CI beside lavapipe's records.
- `series/<adapter>-<backend>/<scene>.json`: one row per record in `records/`,
  in the order their commits were made, every one rebuilt by each publish. A
  row holds the counter frames' counters by name and the measured frames'
  under `measured_counters`.
- `experiments/<name>/<adapter>-<backend>/<scene>/<source hash>.json`: an
  Experiment's records, published with `--experiment <name>` from any branch,
  on no Series. Each names its `experiment.merge_base` with `main` and that
  commit's source hash, whose record on the Series is what it compares with.

A machine is quiet when preflight warned of no GPU load over 10%, one-minute
load average over 2, other praxis process, battery power or busy display at
the run's start, no process outside praxis held 50% of a core or more at the
scene's start or end, and the run's GPU frames, each over the median of
the 31 around it, stayed under 1.06 at their p95.

lavapipe's records hold CPU, GPU and pass times like any other, but lavapipe
renders on a shared runner's CPU, so the page draws only its counters and its
binary's size, which load cannot change, and its build time, which a shared
runner's load and cache hits move whatever a quiet test says. So its records
join its Series from CI's configuration whether or not the runner was quiet.

A Series row's `suspects` names each frame median, CPU or GPU, that makes its
record a suspect, with its shift as a share of the median of up to 8 records
before it on its line: it lies outside that median ± the larger of 4 median
absolute deviations and 3% for the CPU frame, 1.25% for the GPU frame, unless
it only continues a suspect among those 8:
on the same side, within that suspect's tolerance of its median. A new Harness
version or scene content, an imported or re-recorded Baseline, or a commit a
Verdict pinned starts a line. lavapipe's records are never suspects.

- `verdicts/<adapter>-<backend>/<scene>/<source hash>.json`: what the sweep
  found of the slower suspect of that source: the night re-benched it and the
  record before it on its line, interleaved, from release binaries of their
  commits, and the shift `held` when their mean medians still differed in its
  direction by half its marked shift and by its timing's 3% or 1.25%, else
  `cleared`. A held shift
  is bisected along `main`'s first-parent commits of other source hashes
  between `good` and `bad`, each commit's median judged by which end's median
  of that night it lies nearer, over as many nights as it takes, until
  `pinned` names the commit that changed it, whether that commit re-recorded the
  scene's Baseline for the adapter (`covered_by_baseline`), and the mean
  medians of that night's ends (`measured`). A pinned commit no Baseline
  covers is a Regression: it opens one `needs-triage` issue per scene, whose
  number `issue` records. A suspect from the record after a Verdict's
  predecessor to its suspect has that Verdict.

A record in `records/` holds its key against a tagged one. Otherwise a record
whose path exists replaces it only when its GPU frame p95 over its median is
lower, and a lavapipe or release build record never does: a source publishes
there once. A bad publish is undone with `git revert`.

`praxis history import` added the Baselines committed to `main` before the
history existed, each a record of its commit's source, its `provenance.source`
`baseline-commit`. A Baseline file a commit on `main`'s first-parent line
added or changed is a record, unless the commit only renamed it or left every
timing it held (a field ending `_microseconds`) and its Harness version as they
were: new fields or changed counters alone measured nothing again. Two
Baselines of one source meet the rules above with the later first: it stays
unless the earlier goes on a Series and it does not, or the earlier's GPU frame
spread is lower and neither is lavapipe's. A Baseline counts as quiet unless the
GPU frame spread it recorded reached 1.06, each frame's over the 31 around it
where it recorded that, else the run's p95 over its median: one that recorded
none counts as quiet unchecked. Each import rewrites the records an earlier one wrote, with
the tags the rules of its day give them. A record holds what the bench
measured then, so a metric added later is absent from it: its Series row
holds null for a time, Aliasing figure or scene content hash it lacks, and
leaves out a counter or pass it lacks. A Baseline `main` re-recorded since
is a record the same way, its `provenance.source` `baseline-rerecorded` and its
Harness version the one its commit defined; the sweep imports each night.
Its mark stays with its key: a quieter record of its source that holds the key
in its place carries `rerecorded_baseline`, and starts the line in its place.
";

/// A Bench record to publish: the file the branch keeps under its key, and
/// what decides between it and a record already there.
#[derive(Debug)]
struct BenchRecord {
    /// `<adapter>-<backend>`, as the branch's paths name them.
    adapter_key: String,
    /// The scene measured, or `RELEASE_BUILD`.
    subject: String,
    source_hash: String,
    /// Empty for a record on a Series.
    tags: Vec<String>,
    /// The GPU frame p95 over its median; none when not measured.
    p95_over_median: Option<f64>,
    /// Whether the first record under its key stays: lavapipe's and a
    /// release build's, whose Series read no time.
    publishes_once: bool,
    /// The Experiment it belongs to, which keeps it off every Series.
    experiment: Option<String>,
    /// The file's JSON object.
    file: serde_json::Map<String, serde_json::Value>,
}

impl BenchRecord {
    /// The record of `file`, a JSON object naming its adapter and backend,
    /// of `subject` measured on `source_hash` committed at
    /// `committed_unix_seconds`.
    fn new(
        mut file: serde_json::Map<String, serde_json::Value>,
        subject: &str,
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
        let adapter = field("adapter")?;
        let publishes_once = is_lavapipe(adapter) || subject == RELEASE_BUILD;
        let adapter_key = adapter_key(adapter, Some(field("backend")?));
        file.insert(
            "committed_unix_seconds".to_owned(),
            committed_unix_seconds.into(),
        );
        if !tags.is_empty() {
            file.insert("tags".to_owned(), tags.clone().into());
        }
        Ok(Self {
            adapter_key,
            subject: subject.to_owned(),
            source_hash: source_hash.to_owned(),
            tags,
            p95_over_median,
            publishes_once,
            experiment: None,
            file,
        })
    }

    /// Moves the record of `commit` into the Experiment `name`, naming its
    /// merge base with `main` and that commit's source hash.
    fn join_experiment(
        &mut self,
        repository: &Path,
        name: &str,
        commit: &str,
        main: &str,
    ) -> anyhow::Result<()> {
        let merge_base = git(repository, &["merge-base", commit, main], None)?;
        let merge_base_source_hash = source_hash(repository, &merge_base)?;
        self.file.insert(
            "experiment".to_owned(),
            serde_json::json!({
                "name": name,
                "merge_base": merge_base,
                "merge_base_source_hash": merge_base_source_hash,
            }),
        );
        self.experiment = Some(name.to_owned());
        Ok(())
    }

    /// The record's path under `directory`, `ON_SERIES` or `TAGGED`: its
    /// key. The Harness version it holds is not in the path, as it is
    /// defined under `src/`, so a source hash names one.
    fn path(&self, directory: &str) -> PathBuf {
        Path::new(directory)
            .join(&self.adapter_key)
            .join(&self.subject)
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
    /// The adapter's name, one per Series.
    adapter: String,
    provenance: StoredProvenance,
    measured_frames: u32,
    cpu_frame_microseconds: Option<Spread>,
    dynamic_tlas_cpu_microseconds: Option<Spread>,
    /// Named `gpu_main_pass_microseconds` until the bench renamed it, the
    /// same span under a new name.
    #[serde(alias = "gpu_main_pass_microseconds")]
    gpu_frame_microseconds: Option<Spread>,
    gate_microseconds: Option<Spread>,
    shadow_ray_microseconds: Option<Spread>,
    #[serde(default)]
    gpu_pass_microseconds: BTreeMap<String, Spread>,
    counter_frames: Option<StoredCounterFrames>,
    aliasing_rmse: Option<u64>,
    aliasing_p99: Option<u64>,
    /// Whether it holds the key of a re-recorded Baseline it was quieter
    /// than, or that was quieter than it: the Baseline's mark stays with
    /// its key.
    #[serde(default)]
    rerecorded_baseline: bool,
    /// Every field not named above, the measured frames' counters among
    /// them under their names in `Counters::NAMES`: a map, as a record
    /// holds only the counters its bench counted.
    #[serde(flatten)]
    remaining_fields: serde_json::Map<String, serde_json::Value>,
}

/// What every stored record's provenance holds, whichever form wrote it.
#[derive(Debug, serde::Deserialize)]
struct StoredProvenance {
    commit: String,
    source_hash: String,
    harness_version: u32,
    /// `baseline-commit` for an imported Baseline, `baseline-rerecorded`
    /// for one re-recorded since; none for a bench's result.
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

/// The field a record holds, true, when it holds the key of a re-recorded
/// Baseline's source in its place.
const RERECORDED_BASELINE: &str = "rerecorded_baseline";

/// Whether the record `file` marks a re-recorded Baseline: it is one, or
/// holds its key in its place.
fn marks_rerecorded_baseline(file: &serde_json::Map<String, serde_json::Value>) -> bool {
    file.get(RERECORDED_BASELINE) == Some(&true.into())
        || file
            .get("provenance")
            .and_then(|provenance| provenance.get("source"))
            .is_some_and(|source| source == import::RERECORDED_SOURCE)
}

/// Whether `adapter` is lavapipe, whose GPU times mean nothing: its Series
/// read counters, binary size and build time only.
fn is_lavapipe(adapter: &str) -> bool {
    adapter.starts_with(LAVAPIPE_ADAPTER_PREFIX)
}

/// A release build's cost: its binary's size and its wall time.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReleaseBuild {
    pub binary_bytes: u64,
    pub build_seconds: u64,
}

/// A release build as a record holds it: under the adapter whose results
/// its binary measured, from their checkout.
#[derive(Debug, serde::Serialize)]
struct ReleaseBuildRecord<'results> {
    adapter: &'results str,
    backend: &'results str,
    checkout: &'results Checkout,
    #[serde(flatten)]
    build: ReleaseBuild,
}

/// A release build's record as the branch stores it.
#[derive(Debug, serde::Deserialize)]
struct StoredReleaseBuild {
    committed_unix_seconds: u64,
    checkout: Checkout,
    #[serde(flatten)]
    build: ReleaseBuild,
}

/// A release build's Series row.
#[derive(serde::Serialize)]
struct ReleaseBuildRow<'record> {
    commit: &'record str,
    source_hash: &'record str,
    committed_unix_seconds: u64,
    #[serde(flatten)]
    build: &'record ReleaseBuild,
}

/// A stored record as a row of its Series.
trait SeriesRecord: serde::de::DeserializeOwned {
    /// Where its row sorts: in the order commits were made.
    fn order(&self) -> (u64, &str);

    /// The rows of `records`, which are in order, each of the
    /// `line_starts` source hashes starting a line.
    fn rows<'records>(
        records: &'records [Self],
        line_starts: &BTreeSet<String>,
    ) -> Vec<impl serde::Serialize + 'records>;
}

impl SeriesRecord for StoredRecord {
    fn order(&self) -> (u64, &str) {
        (self.committed_unix_seconds, &self.provenance.commit)
    }

    fn rows<'records>(
        records: &'records [Self],
        line_starts: &BTreeSet<String>,
    ) -> Vec<impl serde::Serialize + 'records> {
        let mut rows: Vec<SeriesRow> = records.iter().map(SeriesRow::of).collect();
        // A Series is one adapter's, and lavapipe's times mean nothing, so
        // none of its records is a suspect.
        if records
            .first()
            .is_some_and(|record| !is_lavapipe(&record.adapter))
        {
            let starts_line: Vec<bool> = rows
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    (index > 0 && !rows[index - 1].same_definition(row))
                        || row.source.is_some_and(import::is_baseline_source)
                        || row.rerecorded_baseline
                        || line_starts.contains(row.source_hash)
                })
                .collect();
            for (metric, median, smallest_shift) in JUDGED_TIMINGS {
                let points: Vec<suspects::Point> = rows
                    .iter()
                    .zip(&starts_line)
                    .map(|(row, &starts_line)| suspects::Point {
                        median: median(row),
                        starts_line,
                    })
                    .collect();
                let shifts = suspects::suspects(&points, smallest_shift);
                for (row, shift) in rows.iter_mut().zip(shifts) {
                    if let Some(shift) = shift {
                        row.suspects.insert(metric, shift);
                    }
                }
            }
        }
        rows
    }
}

/// A timing's median in a row.
type MedianOf = fn(&SeriesRow) -> Option<f64>;

/// The smallest shift of a CPU frame median that makes a Suspect and that a
/// night's re-bench must still show, as a share of the median: a busy CPU
/// inflated the benched rows, so their spread sets no floor.
const CPU_FRAME_SMALLEST_SHIFT: f64 = 0.03;

/// The GPU frame's: above the farthest an M4 Pro benched row lies from the
/// median of its scene's other nights, minification at 8bc34eb1 against
/// 91fda48b, ec4dfd7c and 075919da, and under the smallest step it must
/// catch (measured: `docs/performance.md`, Measurements behind the code).
const GPU_FRAME_SMALLEST_SHIFT: f64 = 0.0125;

/// The timings a Series' records are judged on for suspects, each with its
/// median in a row and its smallest shift.
const JUDGED_TIMINGS: [(&str, MedianOf, f64); 2] = [
    (
        "cpu_frame_microseconds",
        |row| row.cpu_frame_microseconds.map(|spread| spread.median),
        CPU_FRAME_SMALLEST_SHIFT,
    ),
    (
        "gpu_frame_microseconds",
        |row| row.gpu_frame_microseconds.map(|spread| spread.median),
        GPU_FRAME_SMALLEST_SHIFT,
    ),
];

/// `metric`'s smallest shift, the largest of them for a timing not judged.
fn smallest_shift(metric: &str) -> f64 {
    JUDGED_TIMINGS
        .iter()
        .find(|(judged, ..)| *judged == metric)
        .map_or(CPU_FRAME_SMALLEST_SHIFT, |&(.., shift)| shift)
}

impl SeriesRecord for StoredReleaseBuild {
    fn order(&self) -> (u64, &str) {
        (self.committed_unix_seconds, &self.checkout.commit)
    }

    fn rows<'records>(
        records: &'records [Self],
        _line_starts: &BTreeSet<String>,
    ) -> Vec<impl serde::Serialize + 'records> {
        records
            .iter()
            .map(|record| ReleaseBuildRow {
                commit: &record.checkout.commit,
                source_hash: &record.checkout.source_hash,
                committed_unix_seconds: record.committed_unix_seconds,
                build: &record.build,
            })
            .collect()
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
    /// Whether a re-recorded Baseline's mark passed to its record.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    rerecorded_baseline: bool,
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
    /// How many frames `measured_counters` count: the configuration's, so
    /// two rows' compare only when these match.
    measured_frames: u32,
    /// The measured frames' counters, as many as the bench counted.
    measured_counters: BTreeMap<&'record str, u64>,
    aliasing_rmse: Option<u64>,
    aliasing_p99: Option<u64>,
    /// Each judged timing whose median makes this record a suspect, with its
    /// shift from the median of the records before it, as a share of that.
    suspects: BTreeMap<&'static str, f64>,
}

/// What a publish did.
#[derive(Debug, Default, PartialEq)]
pub struct Publication {
    /// Records written under `records/`, so on a Series.
    pub on_series: usize,
    /// Records written under `tagged/`.
    pub tagged: usize,
    /// Records written under `experiments/`.
    pub in_experiment: usize,
    /// The Experiment published to; none for a publish to the Series.
    pub experiment: Option<String>,
    /// Records dropped for the same record, or one at least as quiet, under
    /// their key.
    pub dropped: usize,
    /// Records already on the branch given the current Harness version.
    pub restamped: usize,
    /// The commit pushed; none when nothing changed.
    pub commit: Option<String>,
}

impl std::fmt::Display for Publication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(name) = &self.experiment {
            write!(
                formatter,
                "published {} Bench records to Experiment {name} on {BRANCH}, dropped {}",
                self.in_experiment, self.dropped
            )?;
        } else {
            write!(
                formatter,
                "published {} Bench records on a Series and {} tagged to {BRANCH}, dropped {}",
                self.on_series, self.tagged, self.dropped
            )?;
        }
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
/// skipped with a warning. A `release_build` joins them as the release
/// build whose binary measured them, all one adapter's from one checkout.
pub fn publish(
    repository: &Path,
    results: &[Measurements],
    release_build: Option<ReleaseBuild>,
) -> anyhow::Result<Publication> {
    let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
    let repository = Path::new(&root);
    let main = fetch_main(repository)?;
    let main_source_hash = source_hash(repository, &main)?;
    let release_build = release_build
        .map(|build| release_build_record(repository, results, build, &main, &main_source_hash))
        .transpose()?;
    let records: Vec<BenchRecord> =
        bench_records(repository, results, &main, &main_source_hash, None)
            .chain(release_build)
            .collect();
    push_records(repository, &records)
}

/// Publishes `results`, measured in the checkout holding `repository` from
/// any branch, as the Experiment `name`'s Bench records, which join no
/// Series. Each names its merge base with the remote's `main`, so that it
/// compares with the record of that commit's source.
pub fn publish_experiment(
    repository: &Path,
    results: &[Measurements],
    name: &str,
) -> anyhow::Result<Publication> {
    check_experiment_name(name)?;
    let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
    let repository = Path::new(&root);
    let main = fetch_main(repository)?;
    let main_source_hash = source_hash(repository, &main)?;
    let records: Vec<BenchRecord> =
        bench_records(repository, results, &main, &main_source_hash, Some(name)).collect();
    Ok(Publication {
        experiment: Some(name.to_owned()),
        ..push_records(repository, &records)?
    })
}

/// Fails unless `name` can name an Experiment: one directory on the branch,
/// of ASCII letters, digits, `-` and `_`.
pub fn check_experiment_name(name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character)),
        "an Experiment's name is ASCII letters, digits, `-` and `_`, not {name:?}"
    );
    Ok(())
}

/// `results` as Bench records, in the Experiment `experiment` when named; a
/// result that cannot be one is skipped with a warning.
fn bench_records<'results>(
    repository: &'results Path,
    results: &'results [Measurements],
    main: &'results str,
    main_source_hash: &'results str,
    experiment: Option<&'results str>,
) -> impl Iterator<Item = BenchRecord> + 'results {
    results.iter().filter_map(move |measurements| {
        record(repository, measurements, main, main_source_hash, experiment)
            .inspect_err(|error| {
                tracing::warn!(
                    scene = measurements.scene,
                    error = format!("{error:#}"),
                    "bench result not published"
                )
            })
            .ok()
    })
}

/// Pushes `records` to the branch in one commit naming their adapters.
fn push_records(repository: &Path, records: &[BenchRecord]) -> anyhow::Result<Publication> {
    let adapter_keys: BTreeSet<_> = records
        .iter()
        .map(|record| record.adapter_key.as_str())
        .collect();
    let message = format!(
        "Publish {} Bench records from {}",
        records.len(),
        adapter_keys.into_iter().collect::<Vec<_>>().join(", ")
    );
    push(repository, &message, |worktree| worktree.apply(records))
}

/// Which of `scenes` have no record on a Series for the source of the
/// remote's `main` on `adapter` with `backend`: the scenes a sweep of the
/// checkout holding `repository` benches.
pub fn unrecorded_scenes<'scene>(
    repository: &Path,
    adapter: &str,
    backend: &str,
    scenes: &[&'scene str],
) -> anyhow::Result<Vec<&'scene str>> {
    let main = fetch_main(repository)?;
    let main_source_hash = source_hash(repository, &main)?;
    // Git names paths with `/` on every platform.
    let directory = format!("{ON_SERIES}/{}", adapter_key(adapter, Some(backend)));
    let recorded = match remote_tip(repository)? {
        Some(tip) => git(
            repository,
            &["ls-tree", "-r", "--name-only", &tip, "--", &directory],
            None,
        )?,
        None => String::new(),
    };
    let recorded: BTreeSet<&str> = recorded.lines().collect();
    Ok(scenes
        .iter()
        .copied()
        .filter(|scene| {
            !recorded.contains(format!("{directory}/{scene}/{main_source_hash}.json").as_str())
        })
        .collect())
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
/// off a Series, in the Experiment `experiment` when named.
fn record(
    repository: &Path,
    measurements: &Measurements,
    main: &str,
    main_source_hash: &str,
    experiment: Option<&str>,
) -> anyhow::Result<BenchRecord> {
    let provenance = measurements
        .provenance
        .as_ref()
        .context("no provenance: it ran outside a git checkout")?;
    let checkout = &provenance.checkout;
    let serde_json::Value::Object(file) = serde_json::to_value(measurements)? else {
        anyhow::bail!("a bench result is a JSON object");
    };
    let mut record = BenchRecord::new(
        file,
        &measurements.scene,
        &checkout.source_hash,
        committed_unix_seconds(repository, &checkout.commit)?,
        tags(
            checkout.dirty,
            mains_source(repository, checkout, main, main_source_hash),
            Some(Run {
                scene: &measurements.scene,
                adapter: &measurements.adapter,
                profile: &measurements.profile,
                quiet: provenance.quiet,
                width: measurements.width,
                height: measurements.height,
                measured_frames: measurements.measured_frames,
            }),
        ),
        provenance.gpu_frame_p95_over_median,
    )?;
    if let Some(name) = experiment {
        record.join_experiment(repository, name, &checkout.commit, main)?;
    }
    Ok(record)
}

/// `build` as a Bench record beside `results`, which must all be one
/// adapter's on one backend from one checkout.
fn release_build_record(
    repository: &Path,
    results: &[Measurements],
    build: ReleaseBuild,
    main: &str,
    main_source_hash: &str,
) -> anyhow::Result<BenchRecord> {
    let mut sources = results.iter().map(|measurements| {
        (
            &measurements.adapter,
            &measurements.backend,
            measurements
                .provenance
                .as_ref()
                .map(|provenance| &provenance.checkout),
        )
    });
    let first = sources
        .next()
        .context("a release build is published beside the results its binary measured")?;
    anyhow::ensure!(
        sources.all(|source| source == first),
        "a release build is published beside one adapter's results from one checkout"
    );
    let (adapter, backend, checkout) = first;
    let checkout = checkout.context("a release build's results have no provenance")?;
    let serde_json::Value::Object(file) = serde_json::to_value(ReleaseBuildRecord {
        adapter,
        backend,
        checkout,
        build,
    })?
    else {
        anyhow::bail!("a release build is a JSON object");
    };
    BenchRecord::new(
        file,
        RELEASE_BUILD,
        &checkout.source_hash,
        committed_unix_seconds(repository, &checkout.commit)?,
        tags(
            checkout.dirty,
            mains_source(repository, checkout, main, main_source_hash),
            None,
        ),
        None,
    )
}

/// Whether `checkout` holds `main`'s source: the same source hash, or a
/// commit `main` contains.
fn mains_source(
    repository: &Path,
    checkout: &Checkout,
    main: &str,
    main_source_hash: &str,
) -> bool {
    checkout.source_hash == main_source_hash
        || git(
            repository,
            &["merge-base", "--is-ancestor", &checkout.commit, main],
            None,
        )
        .is_ok()
}

/// What a run measured on, as far as its tags tell.
struct Run<'run> {
    scene: &'run str,
    adapter: &'run str,
    profile: &'run str,
    quiet: bool,
    width: u32,
    height: u32,
    measured_frames: u32,
}

/// Every reason a record is kept off a Series: a dirty tree, a source other
/// than `main`'s, and for a `run` a profile other than release, a machine
/// that was not quiet, and a size and frame count other than its Series'
/// configuration, as its scene runs it. lavapipe's Series read counters
/// only, which load cannot change, at the configuration CI benches.
fn tags(dirty: bool, mains_source: bool, run: Option<Run>) -> Vec<String> {
    let mut tags = vec![
        (dirty, "dirty tree".to_owned()),
        (!mains_source, "source not main's".to_owned()),
    ];
    if let Some(Run {
        scene,
        adapter,
        profile,
        quiet,
        width,
        height,
        measured_frames,
    }) = run
    {
        let lavapipe = is_lavapipe(adapter);
        let (configuration, configuration_name) = if lavapipe {
            (Configuration::CI_COUNTERS, "CI's configuration")
        } else {
            (Configuration::REFERENCE, "the reference configuration")
        };
        let configuration = (ReferenceScene::from_name(scene))
            .map_or(configuration, |scene| configuration.for_scene(scene));
        tags.extend([
            (profile != "release", format!("{profile} profile")),
            (!lavapipe && !quiet, "machine not quiet".to_owned()),
            (
                (width, height, measured_frames)
                    != (
                        configuration.width,
                        configuration.height,
                        configuration.measured_frames,
                    ),
                format!("{width}x{height} over {measured_frames} frames, not {configuration_name}"),
            ),
        ]);
    }
    tags.into_iter()
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

    /// Writes `records` into the worktree and rebuilds every Series file,
    /// so rows take the form this bench writes whichever records they hold.
    /// A record on a Series holds its key against a tagged one;
    /// otherwise a record replaces one under its key only when quieter. An
    /// Experiment's records join no Series.
    fn apply(&self, records: &[BenchRecord]) -> anyhow::Result<Publication> {
        let mut publication = Publication {
            on_series: 0,
            tagged: 0,
            in_experiment: 0,
            experiment: None,
            dropped: 0,
            restamped: 0,
            commit: None,
        };
        for record in records {
            let on_series = record.tags.is_empty() && record.experiment.is_none();
            let series_path = self.directory.join(record.path(ON_SERIES));
            let tagged_path = self.directory.join(record.path(TAGGED));
            let relative_path = match &record.experiment {
                Some(name) => record.path(&format!("{EXPERIMENTS}/{name}")),
                None if on_series => record.path(ON_SERIES),
                None => record.path(TAGGED),
            };
            if !on_series && record.experiment.is_none() && series_path.exists() {
                tracing::info!(
                    record = %relative_path.display(),
                    "Bench record dropped: one on a Series holds its key"
                );
                publication.dropped += 1;
                continue;
            }
            let path = self.directory.join(&relative_path);
            let mut file = record.file.clone();
            let mut contents = serde_json::to_string_pretty(&file)? + "\n";
            if let Ok(json) = std::fs::read(&path) {
                let mut existing: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_slice(&json)
                        .with_context(|| format!("parsing {}", relative_path.display()))?;
                // A re-recorded Baseline starts its Series' line whichever
                // record of its source holds the key.
                if marks_rerecorded_baseline(&existing) && !marks_rerecorded_baseline(&file) {
                    file.insert(RERECORDED_BASELINE.to_owned(), true.into());
                    contents = serde_json::to_string_pretty(&file)? + "\n";
                }
                // serde_json may parse a float back a bit off what it wrote,
                // so the same record is found by its contents.
                let stays = json == contents.as_bytes()
                    || record.publishes_once
                    || record.p95_over_median()
                        >= serde_json::from_slice::<StoredRecord>(&json)
                            .with_context(|| format!("parsing {}", relative_path.display()))?
                            .p95_over_median();
                if stays {
                    if marks_rerecorded_baseline(&file) && !marks_rerecorded_baseline(&existing) {
                        existing.insert(RERECORDED_BASELINE.to_owned(), true.into());
                        std::fs::write(&path, serde_json::to_string_pretty(&existing)? + "\n")?;
                    }
                    tracing::info!(
                        record = %relative_path.display(),
                        publishes_once = record.publishes_once,
                        p95_over_median = record.p95_over_median(),
                        "Bench record dropped: the one under its key stays"
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
            } else if record.experiment.is_some() {
                publication.in_experiment += 1;
            } else {
                publication.tagged += 1;
            }
        }
        self.rebuild_every_series()?;
        std::fs::write(self.directory.join("README.md"), README)?;
        Ok(publication)
    }

    /// Rebuilds the Series file of each directory of records on a Series.
    fn rebuild_every_series(&self) -> anyhow::Result<()> {
        let records = self.directory.join(ON_SERIES);
        if !records.exists() {
            return Ok(());
        }
        for adapter_key in std::fs::read_dir(&records)? {
            let adapter_key = adapter_key?.file_name();
            for subject in std::fs::read_dir(records.join(&adapter_key))? {
                let subject = subject?.file_name();
                self.rebuild_series(&Path::new(ON_SERIES).join(&adapter_key).join(subject))?;
            }
        }
        Ok(())
    }

    /// Rebuilds the Series file of the records in `directory`,
    /// `records/<adapter>-<backend>/<scene>`, one row per line in the
    /// order their commits were made.
    fn rebuild_series(&self, directory: &Path) -> anyhow::Result<()> {
        let series = directory.strip_prefix(ON_SERIES)?;
        let lines = if directory.ends_with(RELEASE_BUILD) {
            self.series_lines::<StoredReleaseBuild>(directory, &BTreeSet::new())?
        } else {
            let pinned =
                verdicts::pinned_sources(&self.directory.join(verdicts::VERDICTS).join(series))?;
            self.series_lines::<StoredRecord>(directory, &pinned)?
        };
        let series = Path::new("series").join(series).with_extension("json");
        let path = self.directory.join(series);
        std::fs::create_dir_all(path.parent().context("a Series has a directory")?)?;
        std::fs::write(path, format!("[\n{}\n]\n", lines.join(",\n")))?;
        Ok(())
    }

    /// The rows of the records in `directory`, each a line of JSON, in the
    /// order their commits were made, each of the `line_starts` source
    /// hashes starting a line.
    fn series_lines<Record: SeriesRecord>(
        &self,
        directory: &Path,
        line_starts: &BTreeSet<String>,
    ) -> anyhow::Result<Vec<String>> {
        let mut records = Vec::new();
        for entry in std::fs::read_dir(self.directory.join(directory))? {
            let path = entry?.path();
            let json = std::fs::read(&path)?;
            let record: Record = serde_json::from_slice(&json)
                .with_context(|| format!("parsing {}", path.display()))?;
            records.push(record);
        }
        records.sort_by(|left, right| left.order().cmp(&right.order()));
        Ok(Record::rows(&records, line_starts)
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<_, _>>()?)
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
            rerecorded_baseline: record.rerecorded_baseline,
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
            measured_frames: record.measured_frames,
            measured_counters: Counters::NAMES
                .into_iter()
                .filter_map(|counter| {
                    let value = record.remaining_fields.get(counter)?.as_u64()?;
                    Some((counter, value))
                })
                .collect(),
            aliasing_rmse: record.aliasing_rmse,
            aliasing_p99: record.aliasing_p99,
            suspects: BTreeMap::new(),
        }
    }

    /// Whether `later` measures what this row does: one Harness version, and
    /// one scene content where both know it, as a chart's line runs: the
    /// twin of the bench page's `sameDefinition`.
    fn same_definition(&self, later: &Self) -> bool {
        self.harness_version == later.harness_version
            && (self.scene_content_hash.is_none()
                || later.scene_content_hash.is_none()
                || self.scene_content_hash == later.scene_content_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::Checkout;
    use crate::bench::tests::{baseline, provenance};

    /// A bare remote with `main` holding one commit, and a clone of it to
    /// bench in, in a fresh temporary directory.
    pub(super) struct Fixture {
        directory: PathBuf,
        remote: PathBuf,
        pub(super) checkout: PathBuf,
    }

    impl Fixture {
        pub(super) fn new(name: &str) -> Self {
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

        pub(super) fn commits(&self) -> usize {
            git(&self.remote, &["rev-list", "--count", BRANCH], None)
                .unwrap()
                .parse()
                .unwrap()
        }

        pub(super) fn read(&self, path: &str) -> String {
            git(&self.remote, &["show", &format!("{BRANCH}:{path}")], None).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.directory).ok();
        }
    }

    pub(super) fn commit(repository: &Path, path: &str, contents: &str) {
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
        let first = publish(&fixture.checkout, &results, None).unwrap();
        assert_eq!((first.on_series, first.tagged, first.dropped), (1, 0, 0));
        let second = publish(&fixture.checkout, &results, None).unwrap();
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
    fn a_publish_rebuilds_every_series_so_rows_take_the_current_form() {
        let fixture = Fixture::new("rebuild");
        let mut results = [result(&fixture.checkout, "Apple M4 Pro", 1.02)];
        results[0].counters.triangles_drawn *= 27;
        publish(&fixture.checkout, &results, None).unwrap();
        let series_path = "series/apple-m4-pro-metal/pbr-sweep.json";
        let rebuilt = fixture.read(series_path);
        // A Series an older bench wrote, in a form rows no longer take.
        let clone = fixture.directory.join("stale");
        fixture.clone_into(&clone);
        git(&clone, &["checkout", "--quiet", BRANCH], None).unwrap();
        commit(&clone, series_path, "[\n{}\n]\n");
        git(&clone, &["push", "--quiet", "origin", BRANCH], None).unwrap();
        let town = Measurements {
            scene: "town".to_owned(),
            ..result(&fixture.checkout, "Apple M4 Pro", 1.02)
        };
        let publication = publish(&fixture.checkout, &[town], None).unwrap();
        assert_eq!(publication.on_series, 1);
        assert_eq!(fixture.read(series_path), rebuilt);
        // A published record's row holds both kinds of frame's counters.
        let rows: serde_json::Value = serde_json::from_str(&rebuilt).unwrap();
        assert_eq!(rows[0]["measured_frames"], 1800);
        assert_eq!(rows[0]["measured_counters"]["triangles_drawn"], 22_080 * 27);
        assert_eq!(rows[0]["triangles_drawn"], 22_080);
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
                None,
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
        let publication =
            publish(&fixture.checkout, &[dirty.clone(), branch.clone()], None).unwrap();
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
            (branch, &["source not main's", "machine not quiet"][..]),
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
                        publish(checkout, &[result], None).unwrap();
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
        assert!(publish(&fixture.checkout, &results, None).is_err());
    }

    #[test]
    fn a_scene_is_unrecorded_until_mains_source_has_a_record_on_its_series() {
        let fixture = Fixture::new("unrecorded");
        let scenes = ["pbr-sweep", "town"];
        let unrecorded =
            || unrecorded_scenes(&fixture.checkout, "Apple M4 Pro", "metal", &scenes).unwrap();
        assert_eq!(unrecorded(), scenes, "no branch yet");
        publish(
            &fixture.checkout,
            &[result(&fixture.checkout, "Apple M4 Pro", 1.10)],
            None,
        )
        .unwrap();
        assert_eq!(unrecorded(), scenes, "a tagged record is no record");
        publish(
            &fixture.checkout,
            &[result(&fixture.checkout, "Apple M4 Pro", 1.02)],
            None,
        )
        .unwrap();
        assert_eq!(unrecorded(), ["town"]);
        assert_eq!(
            unrecorded_scenes(&fixture.checkout, "Apple M4 Pro", "vulkan", &scenes).unwrap(),
            scenes,
            "another backend is another machine"
        );
        // Another session pushes to main: a docs change keeps the source,
        // a source change does not.
        let peer = fixture.directory.join("peer");
        fixture.clone_into(&peer);
        commit(&peer, "docs/notes.md", "notes");
        git(&peer, &["push", "--quiet", "origin", "main"], None).unwrap();
        assert_eq!(unrecorded(), ["town"]);
        commit(&peer, "src/lib.rs", "fn main() { loop {} }");
        git(&peer, &["push", "--quiet", "origin", "main"], None).unwrap();
        assert_eq!(unrecorded(), scenes);
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
        let publication = publish(&fixture.checkout, std::slice::from_ref(&noisy), None).unwrap();
        assert_eq!((publication.on_series, publication.tagged), (0, 1));
        // A quiet record of the same key replaces the tagged one.
        publish(&fixture.checkout, &[quiet], None).unwrap();
        assert_eq!((paths("records/"), paths("tagged/")), (1, 0));
        // And neither a noisy run nor a quieter one of another configuration
        // displaces it.
        let publication = publish(&fixture.checkout, &[noisy, counters], None).unwrap();
        assert_eq!(publication.dropped, 2);
        assert_eq!((paths("records/"), paths("tagged/")), (1, 0));
    }

    #[test]
    fn an_experiment_publishes_under_its_name_beside_its_base_and_on_no_series() {
        let fixture = Fixture::new("experiment");
        let on_main = result(&fixture.checkout, "Apple M4 Pro", 1.02);
        publish(&fixture.checkout, std::slice::from_ref(&on_main), None).unwrap();
        let series = fixture.read("series/apple-m4-pro-metal/pbr-sweep.json");
        git(
            &fixture.checkout,
            &["switch", "--quiet", "-c", "faster-cull"],
            None,
        )
        .unwrap();
        commit(&fixture.checkout, "src/lib.rs", "fn main() { cull() }");
        let on_branch = result(&fixture.checkout, "Apple M4 Pro", 1.02);
        let publication = publish_experiment(
            &fixture.checkout,
            std::slice::from_ref(&on_branch),
            "faster-cull",
        )
        .unwrap();
        assert_eq!(
            (
                publication.in_experiment,
                publication.on_series,
                publication.tagged
            ),
            (1, 0, 0)
        );
        assert!(
            publication
                .to_string()
                .contains("to Experiment faster-cull"),
            "{publication}"
        );
        let source_hash = |measurements: &Measurements| {
            measurements
                .provenance
                .as_ref()
                .unwrap()
                .checkout
                .source_hash
                .clone()
        };
        let path = format!(
            "experiments/faster-cull/apple-m4-pro-metal/pbr-sweep/{}.json",
            source_hash(&on_branch)
        );
        assert!(fixture.files().contains(&path), "{:?}", fixture.files());
        assert!(
            fixture
                .files()
                .iter()
                .all(|path| !path.starts_with("tagged/")),
            "{:?}",
            fixture.files()
        );
        assert_eq!(
            fixture.read("series/apple-m4-pro-metal/pbr-sweep.json"),
            series
        );
        let record: serde_json::Value = serde_json::from_str(&fixture.read(&path)).unwrap();
        let main = git(&fixture.checkout, &["rev-parse", "main"], None).unwrap();
        assert_eq!(
            record["experiment"],
            serde_json::json!({
                "name": "faster-cull",
                "merge_base": main,
                "merge_base_source_hash": source_hash(&on_main),
            })
        );
        // Even main's own clean, quiet source stays off the Series.
        git(&fixture.checkout, &["switch", "--quiet", "main"], None).unwrap();
        let publication = publish_experiment(&fixture.checkout, &[on_main], "control").unwrap();
        assert_eq!((publication.in_experiment, publication.on_series), (1, 0));
        // A record dropped as no quieter still reads as the Experiment's.
        let again = publish_experiment(&fixture.checkout, &[on_branch], "faster-cull").unwrap();
        assert_eq!((again.in_experiment, again.dropped), (0, 1));
        assert!(
            again.to_string().contains("to Experiment faster-cull"),
            "{again}"
        );
        assert_eq!(
            fixture.read("series/apple-m4-pro-metal/pbr-sweep.json"),
            series
        );
        for name in ["", "..", "a/b", "with space"] {
            assert!(
                publish_experiment(&fixture.checkout, &[], name).is_err(),
                "{name:?}"
            );
        }
    }

    #[test]
    fn the_readme_names_the_series_configurations_and_quiet_thresholds() {
        let configuration = |configuration: Configuration| {
            format!(
                "{}x{} over {} frames",
                configuration.width, configuration.height, configuration.measured_frames
            )
        };
        for fact in [
            configuration(Configuration::REFERENCE),
            configuration(Configuration::CI_COUNTERS),
            format!(
                "`world` over the {} frames its bench segment measures",
                Configuration::REFERENCE
                    .for_scene(ReferenceScene::World)
                    .measured_frames
            ),
            format!("GPU load over {}%", crate::preflight::BUSY_GPU_PERCENT),
            format!("load average over {}", crate::preflight::BUSY_LOAD_AVERAGE),
            format!("held {}% of a core", crate::preflight::BUSY_PROCESS_PERCENT),
            format!("under {}", crate::bench::provenance::QUIET_P95_OVER_MEDIAN),
            format!(
                "the {} around it",
                crate::bench::provenance::NEIGHBOUR_FRAMES
            ),
            format!(
                "reached {}",
                crate::bench::provenance::QUIET_P95_OVER_MEDIAN
            ),
        ] {
            assert!(README.contains(&fact), "{fact}");
        }
    }

    #[test]
    fn a_run_of_another_configuration_is_tagged_with_it() {
        let fixture = Fixture::new("configuration");
        let counters = Measurements {
            measured_frames: Configuration::COUNTERS.measured_frames,
            ..result(&fixture.checkout, "Apple M4 Pro", 1.01)
        };
        let source_hash = &counters.provenance.as_ref().unwrap().checkout.source_hash;
        publish(&fixture.checkout, std::slice::from_ref(&counters), None).unwrap();
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
    fn the_world_runs_the_reference_configuration_over_its_bench_segment() {
        let run = |scene, measured_frames| Run {
            scene,
            adapter: "Apple M4 Pro",
            profile: "release",
            quiet: true,
            width: 2560,
            height: 1440,
            measured_frames,
        };
        assert_eq!(
            tags(false, true, Some(run("world", 3352))),
            [] as [String; 0]
        );
        assert_eq!(
            tags(false, true, Some(run("world", 1800))),
            ["2560x1440 over 1800 frames, not the reference configuration"]
        );
        assert_eq!(
            tags(false, true, Some(run("town", 1800))),
            [] as [String; 0]
        );
    }

    #[test]
    fn an_import_records_each_changed_baseline_once_and_restamps_earlier_records() {
        let fixture = Fixture::new("import");
        let checkout = &fixture.checkout;
        // A record a bench published at Harness version 1, before the import,
        // with its release build, which holds no Harness version to restamp.
        let live = result(checkout, "Apple M4 Pro", 1.02);
        let build = ReleaseBuild {
            binary_bytes: 1,
            build_seconds: 1,
        };
        publish(checkout, std::slice::from_ref(&live), Some(build)).unwrap();
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
        // It writes again every record the first wrote, to the same tree.
        assert_eq!(
            (
                second.on_series,
                second.tagged,
                second.dropped,
                second.restamped,
                second.commit
            ),
            (2, 1, 0, 0, None)
        );
        assert_eq!(fixture.commits(), commits);

        // An import under older rules tagged a record today's rules put on
        // its Series, and gave a lavapipe record, which publishes once, tags
        // today's rules do not: the next import writes both as today's would.
        let imported = |directory: &str, adapter_key: &str| {
            fixture
                .files()
                .into_iter()
                .find(|path| {
                    path.starts_with(&format!("{directory}/{adapter_key}/pbr-sweep/"))
                        && fixture
                            .read(path)
                            .contains(r#""source": "baseline-commit""#)
                })
                .unwrap()
        };
        let metal_path = imported(ON_SERIES, "apple-m4-pro-metal");
        let lavapipe_path = imported(TAGGED, "llvmpipe-llvm-20-1-2-vulkan");
        let (metal_record, lavapipe_record) =
            (fixture.read(&metal_path), fixture.read(&lavapipe_path));
        let stale = |record: &str, tags: &[&str]| {
            let mut file: serde_json::Value = serde_json::from_str(record).unwrap();
            file["tags"] = serde_json::json!(tags);
            file.to_string()
        };
        let metal_stale_path = metal_path.replacen(ON_SERIES, TAGGED, 1);
        let editor = fixture.directory.join("editor");
        fixture.clone_into(&editor);
        git(&editor, &["checkout", "--quiet", BRANCH], None).unwrap();
        std::fs::remove_file(editor.join(&metal_path)).unwrap();
        let metal_stale_file = editor.join(&metal_stale_path);
        std::fs::create_dir_all(metal_stale_file.parent().unwrap()).unwrap();
        std::fs::write(
            metal_stale_file,
            stale(&metal_record, &["machine not quiet"]),
        )
        .unwrap();
        commit(
            &editor,
            &lavapipe_path,
            &stale(
                &lavapipe_record,
                &[
                    "machine not quiet",
                    "640x1440 over 1800 frames, not the reference configuration",
                ],
            ),
        );
        git(&editor, &["push", "--quiet", "origin", BRANCH], None).unwrap();

        let third = import::import_up_to(checkout, &history_started, &[&harness_break]).unwrap();
        assert!(third.commit.is_some());
        let files = fixture.files();
        assert!(!files.contains(&metal_stale_path), "{files:?}");
        assert_eq!(fixture.read(&metal_path), metal_record);
        assert_eq!(fixture.read(&lavapipe_path), lavapipe_record);
    }

    /// A fixture whose `main` re-recorded pbr-sweep's M4 Pro Baseline, with
    /// no GPU frame spread, after the commit it returns, which starts the
    /// history; the re-record is pushed when `pushed`.
    fn re_recorded(name: &str, pushed: bool) -> (Fixture, String) {
        let fixture = Fixture::new(name);
        let checkout = &fixture.checkout;
        commit(checkout, "src/history.rs", "// the history starts");
        let history_started = git(checkout, &["rev-parse", "HEAD"], None).unwrap();
        commit(
            checkout,
            "src/bench/provenance.rs",
            "pub const HARNESS_VERSION: u32 = 7;\n",
        );
        commit(
            checkout,
            "bench/baselines/apple-m4-pro/pbr-sweep.json",
            r#"{"scene": "pbr-sweep", "adapter": "Apple M4 Pro", "backend": "metal",
            "profile": "release", "width": 2560, "height": 1440, "measured_frames": 1800,
            "cpu_frame_microseconds": {"median": 30.0, "p95": 40.0}}"#,
        );
        if pushed {
            git(checkout, &["push", "--quiet", "origin", "main"], None).unwrap();
        }
        (fixture, history_started)
    }

    /// The Series rows' sources, Harness versions and re-recorded marks.
    fn marks(fixture: &Fixture) -> Vec<(Option<String>, Option<u64>, bool)> {
        let series = fixture.read("series/apple-m4-pro-metal/pbr-sweep.json");
        let rows: Vec<serde_json::Value> = serde_json::from_str(&series).unwrap();
        rows.iter()
            .map(|row| {
                (
                    row["source"].as_str().map(str::to_owned),
                    row["harness_version"].as_u64(),
                    row["rerecorded_baseline"] == true,
                )
            })
            .collect()
    }

    #[test]
    fn a_baseline_re_recorded_after_the_history_started_is_marked_as_one() {
        let (fixture, history_started) = re_recorded("re-recorded", false);
        let checkout = &fixture.checkout;
        // Unpushed, a Baseline is not yet `main`'s.
        let unpushed = import::import_up_to(checkout, &history_started, &[]).unwrap();
        assert_eq!(unpushed.on_series + unpushed.tagged, 0);
        git(checkout, &["push", "--quiet", "origin", "main"], None).unwrap();

        let publication = import::import_up_to(checkout, &history_started, &[]).unwrap();
        assert_eq!((publication.on_series, publication.tagged), (1, 0));
        assert_eq!(
            marks(&fixture),
            [(Some("baseline-rerecorded".to_owned()), Some(7), false)]
        );
    }

    /// A Baseline without a spread is never quieter than a bench of its
    /// source: whichever comes first, the bench holds the key, and the mark.
    #[test]
    fn a_re_recorded_baselines_mark_stays_with_its_key_whichever_record_holds_it() {
        let bench_after = |fixture: &Fixture| {
            let bench = result(&fixture.checkout, "Apple M4 Pro", 1.01);
            publish(&fixture.checkout, &[bench], None).unwrap()
        };
        let carried = [(None, Some(1), true)];

        let (fixture, history_started) = re_recorded("mark-replaced", true);
        import::import_up_to(&fixture.checkout, &history_started, &[]).unwrap();
        assert_eq!(bench_after(&fixture).on_series, 1);
        assert_eq!(marks(&fixture), carried);
        // The next night's import changes nothing.
        let again = import::import_up_to(&fixture.checkout, &history_started, &[]).unwrap();
        assert_eq!((again.dropped, again.commit), (1, None));

        let (fixture, history_started) = re_recorded("mark-kept", true);
        bench_after(&fixture);
        let import = import::import_up_to(&fixture.checkout, &history_started, &[]).unwrap();
        assert_eq!(import.dropped, 1);
        assert_eq!(marks(&fixture), carried);
    }

    #[test]
    fn a_series_marks_a_step_as_a_suspect_unless_a_baseline_a_pinned_commit_or_lavapipe_takes_it() {
        let rows = |adapter: &str, step_source: Option<&str>, line_starts: &[&str]| {
            let records: Vec<StoredRecord> = [10_000.0, 10_050.0, 9_950.0, 10_020.0, 11_000.0]
                .iter()
                .enumerate()
                .map(|(index, median)| {
                    serde_json::from_value(serde_json::json!({
                        "adapter": adapter,
                        "committed_unix_seconds": index,
                        "provenance": {
                            "commit": format!("commit-{index}"),
                            "source_hash": format!("source-{index}"),
                            "harness_version": 10,
                            "source": if index == 4 { step_source } else { None },
                            "scene_content_hash": "content",
                        },
                        "measured_frames": 1800,
                        "gpu_frame_microseconds": { "median": median, "p95": median },
                    }))
                    .unwrap()
                })
                .collect();
            let line_starts = line_starts
                .iter()
                .map(|&source| source.to_owned())
                .collect();
            serde_json::to_value(StoredRecord::rows(&records, &line_starts)).unwrap()
        };
        let suspects = |rows: serde_json::Value| {
            rows.as_array()
                .unwrap()
                .iter()
                .map(|row| row["suspects"].clone())
                .collect::<Vec<_>>()
        };
        let unmarked = vec![serde_json::json!({}); 5];
        let mut stepped = unmarked.clone();
        // 11,000 on the four medians before it, whose median is 10,000.
        stepped[4] = serde_json::json!({ "gpu_frame_microseconds": 0.1 });
        assert_eq!(suspects(rows("Apple M4 Pro", None, &[])), stepped);
        assert_eq!(
            suspects(rows("Apple M4 Pro", Some(import::IMPORTED_SOURCE), &[])),
            unmarked
        );
        assert_eq!(
            suspects(rows("Apple M4 Pro", Some(import::RERECORDED_SOURCE), &[])),
            unmarked
        );
        // A bisect pinned the step's commit: its line starts there.
        assert_eq!(
            suspects(rows("Apple M4 Pro", None, &["source-4"])),
            unmarked
        );
        assert_eq!(
            suspects(rows("llvmpipe (LLVM 20.1.2)", None, &[])),
            unmarked
        );
    }

    /// Basis points each benched M4 Pro GPU frame median lies from its
    /// scene's median, every scene benched on three nights or more, in scene
    /// then commit order.
    const BENCHED_NOISE: [f64; 40] = [
        -25.0, -14.0, 42.0, 0.0, 7.0, 0.0, 35.0, -83.0, -6.0, 107.0, 1.0, -1.0, -1.0, 112.0, -13.0,
        1.0, -11.0, 28.0, 4.0, -4.0, 0.0, 70.0, -1.0, 0.0, -41.0, -3.0, 3.0, 54.0, -39.0, 37.0,
        30.0, -30.0, -8.0, 2.0, 26.0, -2.0, 11.0, 80.0, -19.0, -11.0,
    ];

    /// The timings each row of a Series of `medians`, CPU and GPU frames
    /// alike, on the M4 Pro, is a Suspect on.
    fn suspect_timings(medians: &[f64]) -> Vec<Vec<String>> {
        let records: Vec<StoredRecord> = medians
            .iter()
            .enumerate()
            .map(|(index, median)| {
                serde_json::from_value(serde_json::json!({
                    "adapter": "Apple M4 Pro",
                    "committed_unix_seconds": index,
                    "provenance": {
                        "commit": format!("commit-{index}"),
                        "source_hash": format!("source-{index}"),
                        "harness_version": 10,
                        "scene_content_hash": "content",
                    },
                    "measured_frames": 1800,
                    "cpu_frame_microseconds": { "median": median, "p95": median },
                    "gpu_frame_microseconds": { "median": median, "p95": median },
                }))
                .unwrap()
            })
            .collect();
        let rows = serde_json::to_value(StoredRecord::rows(&records, &BTreeSet::new())).unwrap();
        rows.as_array()
            .unwrap()
            .iter()
            .map(|row| {
                row["suspects"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .cloned()
                    .collect()
            })
            .collect()
    }

    fn benched_noise() -> Vec<f64> {
        BENCHED_NOISE
            .iter()
            .map(|basis_points| 10_000.0 * (1.0 + basis_points / 10_000.0))
            .collect()
    }

    #[test]
    fn benched_noise_alone_marks_no_suspect() {
        assert!(suspect_timings(&benched_noise()).iter().all(Vec::is_empty));
    }

    #[test]
    fn a_gpu_frame_step_of_one_and_a_half_percent_after_benched_nights_is_a_suspect() {
        let quiet_nights = 12;
        let mut medians = benched_noise();
        for median in &mut medians[quiet_nights..] {
            *median *= 1.015;
        }
        let marked: Vec<(usize, Vec<String>)> = suspect_timings(&medians)
            .into_iter()
            .enumerate()
            .filter(|(_, timings)| !timings.is_empty())
            .collect();
        assert_eq!(
            marked,
            [(quiet_nights, vec!["gpu_frame_microseconds".to_owned()])]
        );
    }

    /// lavapipe's result of `checkout` in CI's configuration.
    fn lavapipe_result(checkout: &Path, p95_over_median: f64) -> Measurements {
        let ci_configuration = Configuration::CI_COUNTERS;
        Measurements {
            backend: "vulkan".to_owned(),
            width: ci_configuration.width,
            height: ci_configuration.height,
            measured_frames: ci_configuration.measured_frames,
            ..result(checkout, LAVAPIPE, p95_over_median)
        }
    }

    const LAVAPIPE: &str = "llvmpipe (LLVM 20.1.2, 256 bits)";
    const LAVAPIPE_KEY: &str = "llvmpipe-llvm-20-1-2-256-bits-vulkan";

    #[test]
    fn lavapipe_in_ci_configuration_joins_its_series_quiet_or_not_and_publishes_once() {
        let fixture = Fixture::new("lavapipe");
        let noisy = lavapipe_result(&fixture.checkout, 1.14);
        let source_hash = &noisy.provenance.as_ref().unwrap().checkout.source_hash;
        let first = publish(&fixture.checkout, std::slice::from_ref(&noisy), None).unwrap();
        assert_eq!((first.on_series, first.tagged), (1, 0));
        assert!(fixture.files().contains(&format!(
            "records/{LAVAPIPE_KEY}/pbr-sweep/{source_hash}.json"
        )));
        // A quieter run of the same source adds nothing: its counters repeat.
        let quieter = lavapipe_result(&fixture.checkout, 1.01);
        let second = publish(&fixture.checkout, &[quieter], None).unwrap();
        assert_eq!((second.dropped, second.commit), (1, None));
        assert_eq!(fixture.commits(), 1);
    }

    #[test]
    fn lavapipe_in_another_configuration_is_tagged_with_it() {
        let fixture = Fixture::new("lavapipe-configuration");
        let reference = Measurements {
            backend: "vulkan".to_owned(),
            ..result(&fixture.checkout, LAVAPIPE, 1.01)
        };
        let source_hash = &reference.provenance.as_ref().unwrap().checkout.source_hash;
        publish(&fixture.checkout, std::slice::from_ref(&reference), None).unwrap();
        let record: serde_json::Value = serde_json::from_str(&fixture.read(&format!(
            "tagged/{LAVAPIPE_KEY}/pbr-sweep/{source_hash}.json"
        )))
        .unwrap();
        assert_eq!(
            record["tags"],
            serde_json::json!(["2560x1440 over 1800 frames, not CI's configuration"])
        );
    }

    #[test]
    fn a_release_build_lands_beside_its_results_with_a_series_and_publishes_once() {
        let fixture = Fixture::new("release-build");
        let results = [lavapipe_result(&fixture.checkout, 1.03)];
        let source_hash = &results[0].provenance.as_ref().unwrap().checkout.source_hash;
        let build = ReleaseBuild {
            binary_bytes: 12_293_984,
            build_seconds: 412,
        };
        let first = publish(&fixture.checkout, &results, Some(build.clone())).unwrap();
        assert_eq!((first.on_series, first.tagged), (2, 0));
        let record: StoredReleaseBuild = serde_json::from_str(&fixture.read(&format!(
            "records/{LAVAPIPE_KEY}/release-build/{source_hash}.json"
        )))
        .unwrap();
        assert_eq!(record.build, build);
        let series = fixture.read(&format!("series/{LAVAPIPE_KEY}/release-build.json"));
        assert!(
            series.contains(r#""binary_bytes":12293984,"build_seconds":412"#),
            "{series}"
        );
        let second = publish(&fixture.checkout, &results, Some(build)).unwrap();
        assert_eq!((second.dropped, second.commit), (2, None));
        assert_eq!(fixture.commits(), 1);
    }

    #[test]
    fn a_release_build_needs_results_of_one_adapter_from_one_checkout() {
        let fixture = Fixture::new("release-build-adapters");
        let build = ReleaseBuild {
            binary_bytes: 1,
            build_seconds: 1,
        };
        let results = [
            lavapipe_result(&fixture.checkout, 1.03),
            result(&fixture.checkout, "Apple M4 Pro", 1.02),
        ];
        assert!(publish(&fixture.checkout, &results, Some(build.clone())).is_err());
        assert!(publish(&fixture.checkout, &[], Some(build)).is_err());
        assert!(git(&fixture.remote, &["rev-parse", "--verify", BRANCH], None).is_err());
    }
}
