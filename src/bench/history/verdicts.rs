//! Verdicts (ADR 0024): what the sweep found when it re-benched a slower
//! Suspect beside the record before it and bisected the commits between.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context;

use super::{
    BRANCH, JUDGED_TIMINGS, Publication, README, RELEASE_BUILD, adapter_key, fetch_main, git, push,
    remote_tip, source_hash,
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
    /// What the night that pinned it measured.
    pub measured: Measured,
}

/// The ends of the night that pinned a commit, and their mean medians by
/// judged timing.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct Measured {
    pub good: String,
    pub bad: String,
    pub medians: BTreeMap<String, Ends>,
}

/// A night's mean medians of one timing at its good and bad ends.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct Ends {
    pub good_median: f64,
    pub bad_median: f64,
}

impl Ends {
    /// How much slower the bad end ran, as a share of the good end's median.
    fn shift(self) -> f64 {
        (self.bad_median - self.good_median) / self.good_median
    }
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
    /// The `needs-triage` issue a pinned Regression opened, or shares with
    /// another Verdict of its scene pinned to its commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<u64>,
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
            issue: None,
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

/// What a night does next for `night_start`, its Verdict as the night started,
/// after `runs`, the night's runs so far in the order benched, along
/// `first_parent`: `main`'s first-parent commits from the night's good
/// end to its bad end. Until each end has `END_RUNS` runs, the next run
/// is the end with fewer, the good end first, and nothing is judged.
/// Then a Suspect not yet judged is cleared unless the ends' mean medians
/// hold the shift, and a held bisect waits for another night; held, each
/// later run moves the good or bad end to its commit,
/// by which end's median it lies nearer, and the bisect benches the next
/// commit or pins one. Returns the Verdict once judged, and the commit to
/// bench next, none once the night is done with this Suspect.
pub(super) fn step(
    night_start: &Verdict,
    runs: &[Run],
    first_parent: &[Commit],
) -> (Option<Verdict>, Option<String>) {
    let medians = |commit: &str| -> Vec<f64> {
        runs.iter()
            .filter(|run| run.commit == commit)
            .map(|run| run.median)
            .collect()
    };
    let (good_runs, bad_runs) = (medians(&night_start.good), medians(&night_start.bad));
    if good_runs.len() < END_RUNS || bad_runs.len() < END_RUNS {
        let next = if good_runs.len() <= bad_runs.len() {
            &night_start.good
        } else {
            &night_start.bad
        };
        return (None, Some(next.clone()));
    }
    let (good_median, bad_median) = (mean(&good_runs), mean(&bad_runs));
    let mut verdict = night_start.clone();
    if !holds(night_start.shift, good_median, bad_median) {
        // A bisect's held shift was judged once; a night whose ends do not
        // show it judges no commit and leaves the Verdict for the next.
        if night_start.state == Some(State::Held) {
            return (None, None);
        }
        verdict.state = Some(State::Cleared);
        return (Some(verdict), None);
    }
    verdict.state = Some(State::Held);
    for run in runs
        .iter()
        .filter(|run| run.commit != night_start.good && run.commit != night_start.bad)
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
                measured: Measured {
                    good: night_start.good.clone(),
                    bad: night_start.bad.clone(),
                    medians: [(
                        night_start.metric.clone(),
                        Ends {
                            good_median,
                            bad_median,
                        },
                    )]
                    .into(),
                },
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
    let (verdict, next) = step(
        &start,
        &night_runs(night, &start.scene, &start.metric)?,
        &first_parent,
    );
    if let Some(mut verdict) = verdict {
        if let Some(pinned) = &mut verdict.pinned {
            // Git names paths with `/` on every platform.
            let baseline: Vec<_> = baseline_directory(adapter, backend)
                .iter()
                .map(|component| component.to_string_lossy().into_owned())
                .chain([format!("{}.json", verdict.scene)])
                .collect();
            let commit = &pinned.commit.commit;
            for (metric, _) in JUDGED_TIMINGS {
                let runs = night_runs(night, &verdict.scene, metric)?;
                let medians = |commit: &str| -> Vec<f64> {
                    runs.iter()
                        .filter(|run| run.commit == commit)
                        .map(|run| run.median)
                        .collect()
                };
                let (good, bad) = (medians(&start.good), medians(&start.bad));
                if !good.is_empty() && !bad.is_empty() {
                    pinned.measured.medians.insert(
                        metric.to_owned(),
                        Ends {
                            good_median: mean(&good),
                            bad_median: mean(&bad),
                        },
                    );
                }
            }
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

/// The label a Regression's issue opens with.
const TRIAGE_LABEL: &str = "needs-triage";

/// Opens one `needs-triage` issue, with `gh` in `repository`, per scene
/// and commit the Verdicts of `adapter` on `backend` pinned that no
/// re-recorded Baseline covers, and records each issue's number on its
/// Verdicts on the remote's `bench-history`, so a later run opens none.
/// Returns each Verdict's scene, pinned commit and issue, opened or shared.
pub fn file_regressions(
    repository: &Path,
    adapter: &str,
    backend: &str,
) -> anyhow::Result<Vec<(String, String, u64)>> {
    let Some(tip) = remote_tip(repository)? else {
        return Ok(Vec::new());
    };
    fetch_main(repository)?;
    let key = adapter_key(adapter, Some(backend));
    let mut verdicts = stored(repository, &tip, &format!("{VERDICTS}/{key}"))?;
    let mut filed = Vec::new();
    for index in 0..verdicts.len() {
        let verdict = &verdicts[index];
        let issue = match filing(verdict, &verdicts) {
            Filing::Done => continue,
            Filing::Shared(issue) => issue,
            Filing::Open => {
                let pinned = verdict.pinned.as_ref().context("a Regression is pinned")?;
                let subject = git(
                    repository,
                    &["show", "--no-patch", "--format=%s", &pinned.commit.commit],
                    None,
                )?;
                let issue = issue(verdict, &key, &subject).context("a Regression is pinned")?;
                open_issue(repository, &issue)?
            }
        };
        let verdict = &mut verdicts[index];
        verdict.issue = Some(issue);
        record(repository, &key, verdict)?;
        let commit = verdict
            .pinned
            .as_ref()
            .map(|pinned| pinned.commit.commit.clone());
        filed.push((verdict.scene.clone(), commit.unwrap_or_default(), issue));
    }
    Ok(filed)
}

/// Opens `issue` with `gh` in `repository`, labelled `TRIAGE_LABEL`, and
/// returns its number.
fn open_issue(repository: &Path, issue: &Issue) -> anyhow::Result<u64> {
    let output = std::process::Command::new("gh")
        .current_dir(repository)
        .args(["issue", "create", "--label", TRIAGE_LABEL, "--title"])
        .arg(&issue.title)
        .arg("--body")
        .arg(&issue.body)
        .output()
        .context("running gh")?;
    anyhow::ensure!(
        output.status.success(),
        "gh issue create: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    // gh prints the new issue's URL, which ends in its number.
    let url = String::from_utf8(output.stdout)?;
    url.trim()
        .rsplit('/')
        .next()
        .and_then(|number| number.parse().ok())
        .with_context(|| format!("an issue number at the end of {url:?}"))
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

/// The runs in `night`'s directories, in name order, of `scene`: the
/// commit each benched and its median of `metric`.
fn night_runs(night: &Path, scene: &str, metric: &str) -> anyhow::Result<Vec<Run>> {
    let mut directories: Vec<PathBuf> = std::fs::read_dir(night)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    directories.sort();
    let mut runs = Vec::new();
    for directory in directories {
        let path = directory.join(format!("{scene}.json"));
        if !path.exists() {
            continue;
        }
        // Read loosely: an earlier commit's binary wrote it.
        let result: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)
            .with_context(|| format!("parsing {}", path.display()))?;
        let commit = result["provenance"]["commit"].as_str();
        let median = result[metric]["median"].as_f64();
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

/// The bench page, which draws a machine's Series of a scene.
const BENCH_PAGE: &str = "https://gariasf.github.io/praxis/bench/";

/// A `needs-triage` issue's title and body.
#[derive(Debug, PartialEq)]
pub(super) struct Issue {
    pub title: String,
    pub body: String,
}

/// The issue `verdict`'s Regression opens on the Series of `adapter_key`,
/// its pinned commit's subject `subject`: the commit and its PR, the
/// Series and its chart, and each judged timing the night measured slower
/// at its bad end, the Verdict's always and any other by
/// `SMALLEST_HELD_SHIFT`. None unless the Verdict is pinned.
pub(super) fn issue(verdict: &Verdict, adapter_key: &str, subject: &str) -> Option<Issue> {
    let scene = &verdict.scene;
    let Pinned {
        commit: Commit { commit, .. },
        measured,
        ..
    } = verdict.pinned.as_ref()?;
    let short = &commit[..commit.len().min(8)];
    let pull_request = pull_request(subject);
    let title = match pull_request {
        Some(number) => format!("Regression in {scene} on {adapter_key} from {short} (#{number})"),
        None => format!("Regression in {scene} on {adapter_key} from {short}"),
    };
    let pull_request = pull_request.map_or("none named in its subject".to_owned(), |number| {
        format!("#{number}")
    });
    let mut body = format!(
        "The nightly sweep pinned a Regression that no re-recorded Baseline covers.\n\n\
         - Commit: {commit}, {subject}\n\
         - PR: {pull_request}\n\
         - Series: `{scene}` on `{adapter_key}`\n\
         - Chart: {BENCH_PAGE}?machine={adapter_key}&scene={scene}\n\n\
         | Metric | Good end median | Bad end median | Shift |\n\
         |---|---|---|---|\n"
    );
    for (metric, ends) in measured
        .medians
        .iter()
        .filter(|(metric, ends)| **metric == verdict.metric || ends.shift() >= SMALLEST_HELD_SHIFT)
    {
        body += &format!(
            "| `{metric}` | {:.1} | {:.1} | {:+.1}% |\n",
            ends.good_median,
            ends.bad_median,
            ends.shift() * 100.0
        );
    }
    body += &format!(
        "\nMeasured the night it was pinned, from {END_RUNS} interleaved runs of each \
         end: {} (good) and {} (bad). The Suspect was {}, marked {:+.1}% on `{}` \
         after {}. Its Verdict is `{VERDICTS}/{adapter_key}/{scene}/{}.json` on \
         `{BRANCH}`.\n",
        measured.good,
        measured.bad,
        verdict.suspect.commit,
        verdict.shift * 100.0,
        verdict.metric,
        verdict.predecessor.commit,
        verdict.suspect.source_hash,
    );
    Some(Issue { title, body })
}

/// What a Verdict's Regression needs of its issue.
#[derive(Debug, PartialEq)]
pub(super) enum Filing {
    /// No Regression, or its issue is already recorded.
    Done,
    /// Another Verdict of its scene pinned to its commit opened this issue.
    Shared(u64),
    /// No issue names it yet.
    Open,
}

/// What `verdict` needs, beside `verdicts`, the others of its adapter and
/// backend: a pinned commit no re-recorded Baseline covers needs an issue,
/// one per scene and commit.
pub(super) fn filing(verdict: &Verdict, verdicts: &[Verdict]) -> Filing {
    let Some(pinned) = &verdict.pinned else {
        return Filing::Done;
    };
    if pinned.covered_by_baseline || verdict.issue.is_some() {
        return Filing::Done;
    }
    verdicts
        .iter()
        .filter(|other| other.scene == verdict.scene)
        .filter(|other| {
            other.pinned.as_ref().map(|pinned| &pinned.commit.commit) == Some(&pinned.commit.commit)
        })
        .find_map(|other| other.issue)
        .map_or(Filing::Open, Filing::Shared)
}

/// The PR a squash merge's `subject` names at its end, as `(#<number>)`.
fn pull_request(subject: &str) -> Option<u64> {
    subject.strip_suffix(')')?.rsplit_once("(#")?.1.parse().ok()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
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
    fn a_nights_runs_are_read_from_the_results_the_bench_writes() {
        let night_directory =
            std::env::temp_dir().join(format!("praxis-night-{}", std::process::id()));
        std::fs::remove_dir_all(&night_directory).ok();
        let mut result = crate::bench::tests::baseline();
        let checkout = crate::bench::Checkout {
            source_hash: "1".to_owned(),
            commit: "a".to_owned(),
            dirty: false,
            branch: "HEAD".to_owned(),
        };
        result.provenance = Some(crate::bench::tests::provenance(checkout, 1.01));
        for (run, median) in [("002", 200.0), ("001", 100.0)] {
            result.cpu_frame_microseconds.median = median;
            let directory = night_directory.join(run);
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                directory.join(format!("{}.json", result.scene)),
                serde_json::to_string(&result).unwrap(),
            )
            .unwrap();
        }
        let verdict = Verdict {
            scene: result.scene.clone(),
            ..night()
        };
        let read = night_runs(&night_directory, &verdict.scene, &verdict.metric).unwrap();
        std::fs::remove_dir_all(&night_directory).ok();
        assert_eq!(read, runs(&[("a", 100.0), ("a", 200.0)]));
    }

    #[test]
    fn a_held_bisect_waits_for_a_night_whose_ends_show_its_shift() {
        let held = Verdict {
            state: Some(State::Held),
            ..night()
        };
        let ends = runs(&[
            ("a", 10_000.0),
            ("d", 10_200.0),
            ("a", 10_100.0),
            ("d", 10_300.0),
        ]);
        assert_eq!(step(&held, &ends, &commits("abcd", "1234")), (None, None));
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
        let pinned = verdict.pinned.unwrap();
        assert_eq!(pinned.commit, commits("c", "3")[0]);
        // Measured at the night's ends, not where the bisect moved them.
        assert_eq!(
            (pinned.measured.good, pinned.measured.bad),
            ("a".to_owned(), "d".to_owned())
        );
        assert_eq!(
            pinned.measured.medians["cpu_frame_microseconds"],
            Ends {
                good_median: 10_050.0,
                bad_median: 11_050.0
            }
        );
    }

    /// `commit` with its source as `row` names it.
    fn row_commit(commit: &str) -> Commit {
        Commit {
            commit: commit.to_owned(),
            source_hash: format!("source-{commit}"),
        }
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
            suspect: row_commit("e"),
            predecessor: row_commit("b"),
            committed_unix_seconds: 0,
        });
        assert_eq!(listed(&[verdict]), []);
    }

    /// `night`'s Verdict pinned to `c`, uncovered, measured on its CPU frame
    /// 10% and its GPU frame 1% slower.
    fn regression() -> Verdict {
        let mut verdict = night();
        verdict.state = Some(State::Held);
        verdict.pinned = Some(Pinned {
            commit: Commit {
                commit: "c0ffee1234".to_owned(),
                source_hash: "3".to_owned(),
            },
            covered_by_baseline: false,
            measured: Measured {
                good: "b".to_owned(),
                bad: "d".to_owned(),
                medians: [
                    (
                        "cpu_frame_microseconds".to_owned(),
                        Ends {
                            good_median: 10_000.0,
                            bad_median: 11_000.0,
                        },
                    ),
                    (
                        "gpu_frame_microseconds".to_owned(),
                        Ends {
                            good_median: 8_000.0,
                            bad_median: 8_080.0,
                        },
                    ),
                ]
                .into(),
            },
        });
        verdict
    }

    #[test]
    fn a_regressions_issue_names_its_commit_pr_series_delta_and_chart() {
        let issue = issue(
            &regression(),
            "apple-m4-pro-metal",
            "Spin the orrery twice (#412)",
        )
        .unwrap();
        assert_eq!(
            issue.title,
            "Regression in town on apple-m4-pro-metal from c0ffee12 (#412)"
        );
        assert_eq!(
            issue.body,
            "The nightly sweep pinned a Regression that no re-recorded Baseline covers.

- Commit: c0ffee1234, Spin the orrery twice (#412)
- PR: #412
- Series: `town` on `apple-m4-pro-metal`
- Chart: https://gariasf.github.io/praxis/bench/?machine=apple-m4-pro-metal&scene=town

| Metric | Good end median | Bad end median | Shift |
|---|---|---|---|
| `cpu_frame_microseconds` | 10000.0 | 11000.0 | +10.0% |

Measured the night it was pinned, from 2 interleaved runs of each end: b \
(good) and d (bad). The Suspect was d, marked +10.0% on \
`cpu_frame_microseconds` after a. Its Verdict is \
`verdicts/apple-m4-pro-metal/town/4.json` on `bench-history`.
"
        );
    }

    #[test]
    fn a_commit_with_no_pr_in_its_subject_is_named_alone() {
        let issue = issue(&regression(), "apple-m4-pro-metal", "Spin the orrery").unwrap();
        assert_eq!(
            issue.title,
            "Regression in town on apple-m4-pro-metal from c0ffee12"
        );
        assert!(issue.body.contains("- PR: none named in its subject\n"));
    }

    #[test]
    fn one_issue_opens_per_scene_and_pinned_commit_unless_a_baseline_covers_it() {
        let open = regression();
        assert_eq!(filing(&open, &[]), Filing::Open);
        let filed = Verdict {
            issue: Some(7),
            ..regression()
        };
        assert_eq!(filing(&filed, &[]), Filing::Done);
        // Another Suspect of the scene pinned to the same commit shares its issue.
        assert_eq!(
            filing(&open, std::slice::from_ref(&filed)),
            Filing::Shared(7)
        );
        // Another scene's issue names only that scene.
        let elsewhere = Verdict {
            scene: "crowd".to_owned(),
            ..filed
        };
        assert_eq!(filing(&open, &[elsewhere]), Filing::Open);
        let mut covered = regression();
        covered.pinned.as_mut().unwrap().covered_by_baseline = true;
        assert_eq!(filing(&covered, &[]), Filing::Done);
        // Held but not pinned yet, or cleared: no Regression.
        assert_eq!(filing(&night(), &[]), Filing::Done);
    }
}
