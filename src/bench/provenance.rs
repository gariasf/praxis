//! Where and when a bench result was measured, so that it can become a
//! Bench record (ADR 0024): the source it measured, the machine and how
//! quiet it was, and what its metrics mean.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;

use super::Spread;
use crate::preflight::{BUSY_GPU_PERCENT, gpu_load_percent, other_praxis_processes};

/// Names what the bench's metrics mean: raised by any change to how a
/// metric is measured, so that two Bench records compare only when they
/// share it.
pub const HARNESS_VERSION: u32 = 1;

/// The top-level paths whose changes can change a measurement, which the
/// source hash covers. Every other tracked top-level path is in the tests'
/// `UNMEASURED_PATHS`.
const MEASURED_PATHS: [&str; 5] = ["src", "assets", "Cargo.toml", "Cargo.lock", ".cargo"];

/// A scene's GPU frame p95 over its median at or above which its run was
/// not quiet, whatever preflight saw at its start.
const QUIET_P95_OVER_MEDIAN: f64 = 1.06;

/// Where and when a bench result was measured.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Provenance {
    #[serde(flatten)]
    pub checkout: Checkout,
    /// The driver and its version, as wgpu's adapter info reports them;
    /// empty when the backend reports none.
    pub driver: String,
    pub driver_info: String,
    pub harness_version: u32,
    /// The Cooked cache's content hash over the Blobs the scene loaded.
    pub scene_content_hash: String,
    /// The busiest GPU's utilisation at the bench's start, when known.
    pub gpu_load_percent: Option<u32>,
    /// Other praxis processes at the bench's start, when listed.
    pub other_processes: Option<Vec<String>>,
    /// The scene's GPU frame p95 over its median; none when the median is
    /// zero.
    pub gpu_frame_p95_over_median: Option<f64>,
    /// Whether the machine was quiet, from the three fields above.
    pub quiet: bool,
    /// When the scene's bench started, in seconds since the Unix epoch.
    pub started_unix_seconds: u64,
}

/// What a bench reads once before its first scene: the checkout, and the
/// machine's load while no scene runs, since the GPU utilisation a reading
/// right after a scene reports would include that scene's.
#[derive(Clone, Debug)]
pub struct BenchStart {
    /// None when the checkout is no git repository.
    pub checkout: Option<Checkout>,
    pub gpu_load_percent: Option<u32>,
    pub other_processes: Option<Vec<String>>,
}

impl BenchStart {
    /// Reads the checkout in the working directory, and the machine's
    /// load; a checkout git cannot read warns and leaves results without
    /// Provenance.
    pub fn read() -> Self {
        let checkout = Checkout::read(Path::new("."))
            .inspect_err(|error| {
                tracing::warn!(
                    error = format!("{error:#}"),
                    "bench results carry no provenance"
                )
            })
            .ok();
        Self {
            checkout,
            gpu_load_percent: gpu_load_percent(),
            other_processes: other_praxis_processes(),
        }
    }

    /// The Provenance of a scene's run that started at `started` and
    /// loaded `scene_content_hash`, on `adapter`, whose GPU frame spread
    /// `gpu_frame`.
    pub(crate) fn provenance(
        &self,
        adapter: &wgpu::AdapterInfo,
        scene_content_hash: String,
        gpu_frame: Spread,
        started: SystemTime,
    ) -> Option<Provenance> {
        let checkout = self.checkout.clone()?;
        let gpu_frame_p95_over_median =
            (gpu_frame.median > 0.0).then(|| gpu_frame.p95 / gpu_frame.median);
        Some(Provenance {
            checkout,
            driver: adapter.driver.clone(),
            driver_info: adapter.driver_info.clone(),
            harness_version: HARNESS_VERSION,
            scene_content_hash,
            gpu_load_percent: self.gpu_load_percent,
            other_processes: self.other_processes.clone(),
            gpu_frame_p95_over_median,
            quiet: is_quiet(
                self.gpu_load_percent,
                self.other_processes.as_deref(),
                gpu_frame_p95_over_median,
            ),
            started_unix_seconds: started
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since_epoch| since_epoch.as_secs()),
        })
    }
}

/// Whether a run was quiet: preflight would have warned of neither the GPU
/// load nor other praxis processes at its start, and its GPU frames spread
/// less than `QUIET_P95_OVER_MEDIAN`. A load or listing that could not be
/// read is a preflight note, not a warning, so it passes.
fn is_quiet(
    gpu_load_percent: Option<u32>,
    other_processes: Option<&[String]>,
    gpu_frame_p95_over_median: Option<f64>,
) -> bool {
    gpu_load_percent.is_none_or(|percent| percent <= BUSY_GPU_PERCENT)
        && other_processes.is_none_or(<[String]>::is_empty)
        && gpu_frame_p95_over_median.is_some_and(|ratio| ratio < QUIET_P95_OVER_MEDIAN)
}

/// The source a checkout holds.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Checkout {
    /// The git tree of the measured paths, as `source_hash` gives it.
    pub source_hash: String,
    pub commit: String,
    /// Whether a measured path differed from `commit`, untracked files
    /// included.
    pub dirty: bool,
    /// `HEAD` when detached.
    pub branch: String,
}

impl Checkout {
    /// The checkout holding `repository`: its commit, and the source hash
    /// of its working tree, which uncommitted changes to the measured
    /// paths move, untracked files included.
    pub fn read(repository: &Path) -> anyhow::Result<Self> {
        // Pathspecs are relative to the working directory, so git runs at
        // the checkout's root.
        let root = git(repository, &["rev-parse", "--show-toplevel"], None)?;
        let repository = Path::new(&root);
        let commit = git(repository, &["rev-parse", "HEAD"], None)?;
        let branch = git(repository, &["rev-parse", "--abbrev-ref", "HEAD"], None)?;
        let status = git(
            repository,
            &[
                &["status", "--porcelain", "--untracked-files=all", "--"][..],
                &MEASURED_PATHS[..],
            ]
            .concat(),
            None,
        )?;
        let dirty = !status.is_empty();
        let revision = if dirty {
            working_tree(repository)?
        } else {
            commit.clone()
        };
        Ok(Self {
            source_hash: source_hash(repository, &revision)?,
            commit,
            dirty,
            branch,
        })
    }
}

/// The git tree of `HEAD` with the measured paths as `git add --all` would
/// stage them from the working tree, ignored files left out, written
/// through a temporary index so the checkout's own index stays as it is.
fn working_tree(repository: &Path) -> anyhow::Result<String> {
    // A pathspec that matches neither the working tree nor the index fails
    // the add, so a measured path that is in neither is left out.
    let in_head = git(
        repository,
        &[
            &["ls-tree", "--name-only", "HEAD", "--"][..],
            &MEASURED_PATHS[..],
        ]
        .concat(),
        None,
    )?;
    let present: Vec<&str> = MEASURED_PATHS
        .into_iter()
        .filter(|path| repository.join(path).exists() || in_head.lines().any(|line| line == *path))
        .collect();
    let index = std::env::temp_dir().join(format!("praxis-source-index-{}", std::process::id()));
    let staged = |arguments: &[&str]| {
        let mut command = git_command(repository);
        command.env("GIT_INDEX_FILE", &index);
        run(command, arguments, None)
    };
    let tree = staged(&["read-tree", "HEAD"])
        .and_then(|_| staged(&[&["add", "--all", "--"][..], &present].concat()))
        .and_then(|_| staged(&["write-tree"]));
    std::fs::remove_file(&index).ok();
    tree
}

/// The source hash of `revision` in `repository`: the git tree holding
/// only its measured paths, so a commit that changes none of them has its
/// parent's.
pub fn source_hash(repository: &Path, revision: &str) -> anyhow::Result<String> {
    let listing = git(
        repository,
        &[
            &["ls-tree", "--full-tree", revision, "--"][..],
            &MEASURED_PATHS[..],
        ]
        .concat(),
        None,
    )?;
    // An empty tree is no input at all: mktree refuses a blank line.
    let entries = if listing.is_empty() {
        listing
    } else {
        listing + "\n"
    };
    git(repository, &["mktree"], Some(&entries))
}

/// The trimmed output of git with `arguments` in `repository`, given
/// `input` on its standard input.
fn git(repository: &Path, arguments: &[&str], input: Option<&str>) -> anyhow::Result<String> {
    run(git_command(repository), arguments, input)
}

fn git_command(repository: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repository);
    command
}

/// The trimmed output of `command` with `arguments`, given `input` on its
/// standard input.
fn run(mut command: Command, arguments: &[&str], input: Option<&str>) -> anyhow::Result<String> {
    let mut child = command
        .args(arguments)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("running git")?;
    if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
        stdin.write_all(input.as_bytes())?;
    }
    let output = child.wait_with_output()?;
    anyhow::ensure!(
        output.status.success(),
        "git {}: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The top-level paths whose changes cannot change a measurement: the
    /// bench reads none of them.
    const UNMEASURED_PATHS: [&str; 11] = [
        ".github",
        ".gitignore",
        "CLAUDE.md",
        "CONTEXT.md",
        "LICENSE",
        "README.md",
        "ROADMAP.md",
        "bench",
        "docs",
        "scripts",
        "tests",
    ];

    #[test]
    fn every_tracked_top_level_path_is_measured_or_unmeasured_but_not_both() {
        let tracked = git(
            Path::new(env!("CARGO_MANIFEST_DIR")),
            &["ls-tree", "--name-only", "HEAD"],
            None,
        )
        .unwrap();
        for path in tracked.lines() {
            let (measured, unmeasured) = (
                MEASURED_PATHS.contains(&path),
                UNMEASURED_PATHS.contains(&path),
            );
            assert!(
                measured != unmeasured,
                "{path} must be in exactly one of MEASURED_PATHS and UNMEASURED_PATHS \
                 in src/bench/provenance.rs: can a change to it change a measurement?"
            );
        }
    }

    /// A git repository in a fresh temporary directory, with an identity
    /// to commit as.
    fn repository(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("praxis-provenance-{}-{name}", std::process::id()));
        std::fs::remove_dir_all(&directory).ok();
        std::fs::create_dir_all(&directory).unwrap();
        git(&directory, &["init", "--quiet"], None).unwrap();
        for (key, value) in [
            ("user.name", "Bench"),
            ("user.email", "bench@example.com"),
            ("commit.gpgsign", "false"),
        ] {
            git(&directory, &["config", key, value], None).unwrap();
        }
        directory
    }

    fn commit(repository: &Path, path: &str, contents: &str) {
        let file = repository.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, contents).unwrap();
        git(repository, &["add", "--all"], None).unwrap();
        git(repository, &["commit", "--quiet", "--message", path], None).unwrap();
    }

    #[test]
    fn only_a_measured_path_moves_the_source_hash() {
        let repository = repository("history");
        commit(&repository, "src/lib.rs", "fn main() {}");
        let first = source_hash(&repository, "HEAD").unwrap();
        commit(&repository, "docs/notes.md", "notes");
        assert_eq!(source_hash(&repository, "HEAD").unwrap(), first);
        commit(&repository, "Cargo.toml", "[package]");
        let second = source_hash(&repository, "HEAD").unwrap();
        assert_ne!(second, first);
        commit(&repository, "src/lib.rs", "fn main() { loop {} }");
        assert_ne!(source_hash(&repository, "HEAD").unwrap(), second);
        std::fs::remove_dir_all(repository).ok();
    }

    #[test]
    fn two_clean_checkouts_of_a_commit_share_its_source_hash() {
        let original = repository("original");
        commit(&original, "src/lib.rs", "fn main() {}");
        let clone =
            original.with_file_name(format!("praxis-provenance-{}-clone", std::process::id()));
        std::fs::remove_dir_all(&clone).ok();
        git(
            &original,
            &["clone", "--quiet", ".", clone.to_str().unwrap()],
            None,
        )
        .unwrap();
        let (original_checkout, clone_checkout) = (
            Checkout::read(&original).unwrap(),
            Checkout::read(&clone).unwrap(),
        );
        assert!(!original_checkout.dirty && !clone_checkout.dirty);
        assert_eq!(original_checkout.source_hash, clone_checkout.source_hash);
        assert_eq!(original_checkout.commit, clone_checkout.commit);
        std::fs::remove_dir_all(original).ok();
        std::fs::remove_dir_all(clone).ok();
    }

    #[test]
    fn a_dirty_or_untracked_measured_path_moves_the_source_hash_and_another_path_does_not() {
        let repository = repository("dirty");
        commit(&repository, "src/lib.rs", "fn main() {}");
        let clean = Checkout::read(&repository).unwrap();
        std::fs::write(repository.join("README.md"), "untracked, unmeasured").unwrap();
        assert_eq!(Checkout::read(&repository).unwrap(), clean);
        std::fs::write(repository.join("src/lib.rs"), "fn main() { loop {} }").unwrap();
        let dirty = Checkout::read(&repository).unwrap();
        assert!(dirty.dirty);
        assert_eq!(dirty.commit, clean.commit);
        assert_ne!(dirty.source_hash, clean.source_hash);
        std::fs::write(repository.join("src/new.rs"), "untracked, measured").unwrap();
        let untracked = Checkout::read(&repository).unwrap();
        assert!(untracked.dirty);
        assert_ne!(untracked.source_hash, dirty.source_hash);
        // From a subdirectory, the same checkout.
        assert_eq!(Checkout::read(&repository.join("src")).unwrap(), untracked);
        assert_eq!(
            git(&repository, &["status", "--porcelain"], None)
                .unwrap()
                .lines()
                .count(),
            3
        );
        std::fs::remove_dir_all(repository).ok();
    }

    #[test]
    fn a_run_is_quiet_only_when_preflight_would_warn_of_nothing_and_its_frames_hold_steady() {
        let none: &[String] = &[];
        let other = &["praxis 42".to_owned()][..];
        let steady = Some(1.02);
        assert!(is_quiet(Some(BUSY_GPU_PERCENT), Some(none), steady));
        // Unknown load and processes are preflight notes, not warnings.
        assert!(is_quiet(None, None, steady));
        assert!(!is_quiet(Some(BUSY_GPU_PERCENT + 1), Some(none), steady));
        assert!(!is_quiet(Some(0), Some(other), steady));
        assert!(!is_quiet(Some(0), Some(none), Some(QUIET_P95_OVER_MEDIAN)));
        assert!(!is_quiet(Some(0), Some(none), None));
    }
}
