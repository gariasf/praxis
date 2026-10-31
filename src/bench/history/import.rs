//! `praxis history import`: the Baselines committed to `main` before the
//! Bench history existed, each a Bench record of the commit that changed it
//! (ADR 0024).

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Context;

use super::{
    BenchRecord, ON_SERIES, Publication, TAGGED, Worktree, committed_unix_seconds, push, tags,
};
use crate::bench::Spread;
use crate::bench::provenance::{QUIET_P95_OVER_MEDIAN, git, source_hash};

/// Where `main` keeps its Baselines.
const BASELINES_DIRECTORY: &str = "bench/baselines";

/// The commit that started publishing Bench records: every Baseline
/// committed before it is imported, and every bench since has published
/// its own.
const HISTORY_STARTED: &str = "3c6b9d92";

/// The commits on `main` before `HISTORY_STARTED` that changed what a
/// metric means, oldest first, and what changed. Each starts the Harness
/// version after the one before it, so a Baseline committed before the
/// first was measured under version 1.
const HARNESS_BREAKS: [(&str, &str); 9] = [
    (
        "926fcc18",
        "gpu_main_pass_microseconds spans the culling pass's start to the main pass's end",
    ),
    (
        "0b29a3c0",
        "the GPU stats readback ring's own CPU time, allocations and memory join the metrics",
    ),
    (
        "80227bca",
        "gpu_frame_microseconds is the dynamic TLAS build plus the span of the frame's passes",
    ),
    (
        "c5a12eb2",
        "gpu_frame_microseconds spans its passes' earliest start to their latest end",
    ),
    (
        "800c0d21",
        "the dynamic TLAS build counts from the end of the frame before's last pass",
    ),
    (
        "5264094f",
        "gpu_memory_bytes leaves out the GPU stats readback",
    ),
    (
        "958ed4e9",
        "the bench spin-waits on hardware adapters, so CPU times are at full clock",
    ),
    (
        "a5280dce",
        "a render pass's time counts from the end of the passes before it",
    ),
    (
        "a00280dc",
        "the depth passes count only from the passes they wait on",
    ),
];

/// What an imported record's provenance names as its source.
const IMPORTED_SOURCE: &str = "baseline-commit";

/// Imports every Baseline file each commit before `HISTORY_STARTED`
/// changed into the remote's `bench-history` branch, and gives the records
/// a bench published before the import their era's Harness version. A
/// second import changes nothing.
pub fn import_baselines(repository: &Path) -> anyhow::Result<Publication> {
    import_up_to(
        repository,
        HISTORY_STARTED,
        &HARNESS_BREAKS.map(|(commit, _)| commit),
    )
}

/// Imports the Baselines committed before `history_started`, with the
/// Harness version raised at each of `breaks`.
pub(super) fn import_up_to(
    repository: &Path,
    history_started: &str,
    breaks: &[&str],
) -> anyhow::Result<Publication> {
    let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
    let repository = Path::new(&root);
    let records = baseline_records(repository, history_started, breaks)?;
    // The version of the method in force when the history started, which
    // records published before the import name 1.
    let started_harness_version = breaks.len() as u32 + 1;
    let message = format!(
        "Import {} Baselines committed before the Bench history",
        records.len()
    );
    push(repository, &message, |worktree| {
        let restamped = worktree.restamp(started_harness_version)?;
        Ok(Publication {
            restamped,
            ..worktree.apply(&records)?
        })
    })
}

/// A commit that changed Baselines.
struct BaselineCommit<'commit> {
    commit: &'commit str,
    source_hash: String,
    committed_unix_seconds: u64,
    harness_version: u32,
}

/// A record per Baseline file per commit on the first-parent line before
/// `history_started` that added or changed it, newest first: of two
/// Baselines of one source, the quieter stays, and when neither measured
/// its spread, the later, which replaced the earlier. A file only renamed,
/// or given fields without a value it held changing, was not measured
/// again.
fn baseline_records(
    repository: &Path,
    history_started: &str,
    breaks: &[&str],
) -> anyhow::Result<Vec<BenchRecord>> {
    let breaks = breaks
        .iter()
        .map(|commit| {
            git(
                repository,
                &["rev-parse", "--verify", &format!("{commit}^{{commit}}")],
                None,
            )
        })
        .collect::<anyhow::Result<BTreeSet<_>>>()?;
    let commits = git(
        repository,
        &["rev-list", "--reverse", "--first-parent", history_started],
        None,
    )?;
    let changed_baselines = git(
        repository,
        &[
            "log",
            "--first-parent",
            "--format=%H",
            history_started,
            "--",
            BASELINES_DIRECTORY,
        ],
        None,
    )?;
    let changed_baselines: BTreeSet<_> = changed_baselines.lines().collect();
    let mut harness_version = 1;
    let mut records = Vec::new();
    for commit in commits.lines() {
        if breaks.contains(commit) {
            harness_version += 1;
        }
        if !changed_baselines.contains(commit) {
            continue;
        }
        let baseline_commit = BaselineCommit {
            commit,
            source_hash: source_hash(repository, commit)?,
            committed_unix_seconds: committed_unix_seconds(repository, commit)?,
            harness_version,
        };
        let paths = git(
            repository,
            &[
                "diff-tree",
                "-r",
                "--no-commit-id",
                "--name-only",
                "--find-renames=100%",
                "--diff-filter=AM",
                commit,
                "--",
                BASELINES_DIRECTORY,
            ],
            None,
        )?;
        for path in paths.lines().filter(|path| path.ends_with(".json")) {
            let json = git(repository, &["show", &format!("{commit}:{path}")], None)?;
            let earlier = git(repository, &["show", &format!("{commit}^:{path}")], None).ok();
            if let Some(earlier) = earlier
                && !remeasured(&earlier, &json)
                    .with_context(|| format!("comparing {path} at {commit} with its parent's"))?
            {
                tracing::info!(
                    path,
                    commit,
                    "Baseline not imported: its values are its parent's"
                );
                continue;
            }
            records.push(
                imported(&baseline_commit, path, &json)
                    .with_context(|| format!("importing {path} at {commit}"))?,
            );
        }
    }
    anyhow::ensure!(
        harness_version as usize == breaks.len() + 1,
        "a Harness version break is not on the first-parent line before {history_started}"
    );
    records.reverse();
    Ok(records)
}

/// Whether the Baseline `current` measured again what `earlier`, the same
/// file in the commit before, held: whether any value `earlier` held
/// differs or is gone, rather than only fields added.
fn remeasured(earlier: &str, current: &str) -> anyhow::Result<bool> {
    let earlier: serde_json::Map<String, serde_json::Value> = serde_json::from_str(earlier)?;
    let current: serde_json::Map<String, serde_json::Value> = serde_json::from_str(current)?;
    Ok(earlier
        .iter()
        .any(|(key, value)| current.get(key) != Some(value)))
}

/// What the import reads from every Baseline, whichever bench wrote it.
#[derive(serde::Deserialize)]
struct Baseline {
    adapter: String,
    /// Absent before Baselines were keyed by backend.
    backend: Option<String>,
    profile: String,
    width: u32,
    height: u32,
    measured_frames: u32,
    /// Absent before the bench timed the whole frame.
    gpu_frame_microseconds: Option<Spread>,
}

/// An imported record's provenance: the Baseline and the commit it came
/// from, and the Harness version then.
#[derive(serde::Serialize)]
struct ImportedProvenance<'baseline> {
    source: &'static str,
    baseline: &'baseline str,
    commit: &'baseline str,
    source_hash: &'baseline str,
    harness_version: u32,
    gpu_frame_p95_over_median: Option<f64>,
}

/// The Baseline `json` that `commit` committed at `path`, as a Bench
/// record of that commit's source: the file as it was, with the backend it
/// ran on and an imported provenance. Baselines were recorded on a quiet
/// machine, so only their own GPU frame spread can tag one as not quiet.
fn imported(commit: &BaselineCommit, path: &str, json: &str) -> anyhow::Result<BenchRecord> {
    let baseline: Baseline = serde_json::from_str(json)?;
    let serde_json::Value::Object(mut file) = serde_json::from_str(json)? else {
        anyhow::bail!("a Baseline is a JSON object");
    };
    if baseline.backend.is_none() {
        file.insert(
            "backend".to_owned(),
            backend_before_keyed(&baseline.adapter)?.into(),
        );
    }
    let p95_over_median = baseline
        .gpu_frame_microseconds
        .filter(|spread| spread.median > 0.0)
        .map(|spread| spread.p95 / spread.median);
    file.insert(
        "provenance".to_owned(),
        serde_json::to_value(ImportedProvenance {
            source: IMPORTED_SOURCE,
            baseline: path,
            commit: commit.commit,
            source_hash: &commit.source_hash,
            harness_version: commit.harness_version,
            gpu_frame_p95_over_median: p95_over_median,
        })?,
    );
    let quiet = p95_over_median.is_none_or(|ratio| ratio < QUIET_P95_OVER_MEDIAN);
    BenchRecord::new(
        file,
        &commit.source_hash,
        commit.committed_unix_seconds,
        tags(
            false,
            &baseline.profile,
            quiet,
            true,
            (baseline.width, baseline.height, baseline.measured_frames),
        ),
        p95_over_median,
    )
}

/// The backend a Baseline that names none ran on: until the RTX 2070
/// SUPER joined, the M4 Pro benched on Metal and CI's lavapipe on Vulkan.
fn backend_before_keyed(adapter: &str) -> anyhow::Result<&'static str> {
    if adapter.starts_with("Apple") {
        Ok("metal")
    } else if adapter.starts_with("llvmpipe") {
        Ok("vulkan")
    } else {
        anyhow::bail!("no backend known for {adapter}'s Baseline")
    }
}

impl Worktree<'_> {
    /// Gives every record a bench published at Harness version 1 the
    /// Harness version `version`: before the import numbered the eras before it,
    /// version 1 named the method in force. Rebuilds the Series they are
    /// on, and returns how many it changed.
    fn restamp(&self, version: u32) -> anyhow::Result<usize> {
        let listing = git(
            &self.directory,
            &["ls-files", "--", ON_SERIES, TAGGED],
            None,
        )?;
        let mut restamped = 0;
        let mut series = BTreeSet::new();
        for relative_path in listing.lines().map(Path::new) {
            let path = self.directory.join(relative_path);
            let mut file: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)
                .with_context(|| format!("parsing {}", relative_path.display()))?;
            let provenance = file
                .get_mut("provenance")
                .and_then(serde_json::Value::as_object_mut)
                .with_context(|| format!("{} has provenance", relative_path.display()))?;
            if provenance.contains_key("source")
                || provenance.get("harness_version") != Some(&1.into())
            {
                continue;
            }
            provenance.insert("harness_version".to_owned(), version.into());
            std::fs::write(&path, serde_json::to_string_pretty(&file)? + "\n")?;
            restamped += 1;
            if relative_path.starts_with(ON_SERIES) {
                series.insert(
                    relative_path
                        .parent()
                        .context("a record has a directory")?
                        .to_owned(),
                );
            }
        }
        for directory in series {
            self.rebuild_series(&directory)?;
        }
        Ok(restamped)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{SeriesRow, StoredRecord};
    use super::*;
    use crate::bench::provenance::HARNESS_VERSION;

    /// pbr-sweep's first Baseline, as `591433d7` committed it.
    const FIRST_BASELINE: &str = r#"{
  "scene": "pbr-sweep",
  "adapter": "Apple M4 Pro",
  "profile": "release",
  "width": 2560,
  "height": 1440,
  "measured_frames": 1800,
  "cpu_frame_microseconds": {
    "median": 33.041,
    "p95": 73.292
  },
  "gpu_main_pass_microseconds": {
    "median": 198.875,
    "p95": 211.209
  },
  "draw_calls": 10,
  "uploaded_bytes": 1712,
  "gpu_memory_bytes": 687168016,
  "heap_allocations": 83
}"#;

    /// pbr-sweep's Baseline as this checkout holds it.
    const CURRENT_BASELINE: &str =
        include_str!("../../../bench/baselines/apple-m4-pro/pbr-sweep.json");

    fn commit(harness_version: u32) -> BaselineCommit<'static> {
        BaselineCommit {
            commit: "591433d7",
            source_hash: "4b825dc6".to_owned(),
            committed_unix_seconds: 1_791_871_200,
            harness_version,
        }
    }

    /// `record`'s file as the branch would store it, read back.
    fn stored(record: &BenchRecord) -> StoredRecord {
        serde_json::from_value(serde_json::Value::Object(record.file.clone())).unwrap()
    }

    #[test]
    fn a_first_era_baseline_becomes_a_record_of_its_commit_with_what_it_measured() {
        let record =
            imported(&commit(1), "bench/baselines/pbr-sweep.json", FIRST_BASELINE).unwrap();
        assert_eq!(
            record.path(ON_SERIES),
            Path::new("records/apple-m4-pro-metal/pbr-sweep/4b825dc6.json")
        );
        assert!(record.tags.is_empty(), "{:?}", record.tags);
        assert_eq!(record.p95_over_median, None);
        assert_eq!(record.file["backend"], "metal");
        assert_eq!(record.file["gpu_main_pass_microseconds"]["median"], 198.875);
        let stored = stored(&record);
        assert_eq!(
            (
                stored.provenance.source.as_deref(),
                stored.provenance.commit.as_str(),
                stored.provenance.harness_version,
                stored.committed_unix_seconds,
            ),
            (Some(IMPORTED_SOURCE), "591433d7", 1, 1_791_871_200)
        );
        let row = serde_json::to_value(SeriesRow::of(&stored)).unwrap();
        assert_eq!(row["cpu_frame_microseconds"]["median"], 33.041);
        for metric in [
            "gpu_frame_microseconds",
            "gate_microseconds",
            "shadow_ray_microseconds",
            "scene_content_hash",
            "aliasing_rmse",
        ] {
            assert!(row[metric].is_null(), "{metric}: {row}");
        }
        assert!(row.get("draw_calls").is_none(), "{row}");
    }

    #[test]
    fn a_current_baseline_becomes_a_record_with_every_metric_and_its_spread() {
        let record = imported(
            &commit(HARNESS_BREAKS.len() as u32 + 1),
            "bench/baselines/apple-m4-pro/pbr-sweep.json",
            CURRENT_BASELINE,
        )
        .unwrap();
        let baseline: crate::bench::Measurements = serde_json::from_str(CURRENT_BASELINE).unwrap();
        let spread = baseline.gpu_frame_microseconds;
        assert_eq!(record.p95_over_median, Some(spread.p95 / spread.median));
        assert_eq!(record.adapter_key, "apple-m4-pro-metal");
        let row = serde_json::to_value(SeriesRow::of(&stored(&record))).unwrap();
        for metric in [
            "cpu_frame_microseconds",
            "dynamic_tlas_cpu_microseconds",
            "gpu_frame_microseconds",
            "gate_microseconds",
            "shadow_ray_microseconds",
            "aliasing_rmse",
            "draw_calls",
            "heap_allocations",
        ] {
            assert!(!row[metric].is_null(), "{metric}: {row}");
        }
        assert_eq!(
            row["harness_version"],
            HARNESS_BREAKS.len() as u64 + 1,
            "{row}"
        );
    }

    #[test]
    fn a_lavapipe_baseline_runs_on_vulkan_and_a_noisy_one_is_tagged() {
        let lavapipe = FIRST_BASELINE
            .replace("Apple M4 Pro", "llvmpipe (LLVM 20.1.2, 256 bits)")
            .replace("2560", "640")
            .replace("1440", "360")
            .replace("1800", "256");
        let record = imported(
            &commit(1),
            "bench/baselines/llvmpipe/pbr-sweep.json",
            &lavapipe,
        )
        .unwrap();
        assert_eq!(record.adapter_key, "llvmpipe-llvm-20-1-2-256-bits-vulkan");
        assert_eq!(
            record.tags,
            ["640x360 over 256 frames, not the reference configuration"]
        );
        let noisy = FIRST_BASELINE.replace(
            "\"draw_calls\"",
            "\"gpu_frame_microseconds\": { \"median\": 100.0, \"p95\": 110.0 },\n  \"draw_calls\"",
        );
        let record = imported(&commit(1), "bench/baselines/pbr-sweep.json", &noisy).unwrap();
        assert_eq!(record.tags, ["machine not quiet"]);
        let unknown = FIRST_BASELINE.replace("Apple M4 Pro", "Some GPU");
        assert!(imported(&commit(1), "bench/baselines/pbr-sweep.json", &unknown).is_err());
    }

    #[test]
    fn only_a_baseline_whose_held_values_changed_was_measured_again() {
        let earlier = r#"{"scene": "helmet", "cpu_frame_microseconds": {"median": 30.0}}"#;
        let with_backend = r#"{"scene": "helmet", "backend": "metal", "cpu_frame_microseconds": {"median": 30.0}}"#;
        let remeasured_baseline =
            r#"{"scene": "helmet", "cpu_frame_microseconds": {"median": 31.0}}"#;
        let renamed_metric = r#"{"scene": "helmet", "gpu_frame_microseconds": {"median": 30.0}}"#;
        assert!(!remeasured(earlier, with_backend).unwrap());
        assert!(remeasured(earlier, remeasured_baseline).unwrap());
        assert!(remeasured(earlier, renamed_metric).unwrap());
    }

    #[test]
    fn the_live_harness_version_is_no_older_than_the_last_imported_era() {
        assert!(HARNESS_VERSION as usize > HARNESS_BREAKS.len());
    }
}
