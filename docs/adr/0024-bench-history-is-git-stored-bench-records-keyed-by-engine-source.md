# Bench history is git-stored Bench records keyed by engine source

Every bench run is kept as a Bench record. A record is a file on an orphan branch, `bench-history`, in this repo, keyed by source hash × adapter × backend × scene × Harness version. The source hash is one git hash over the paths that can change a measurement. Only clean, release-profile records of `main`'s source taken by a quiet machine form a Series. The history observes and never gates: Baselines stay in `bench/baselines/` on `main`, and `verify` and `compare` judge against them as before (ADR 0013). Records publish themselves: CI's lavapipe on every push to `main`, `praxis bench` by default in sessions, and a nightly sweep on the reference machine and the RTX 2070 SUPER that benches `main` only when its source has no quiet record and the machine is quiet. A page in `docs/` draws the Series. Accepted 2026-10-31 ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6542262691)); the track is #427.

## Considered options

- A hosted benchmark service (Bencher.dev). Change detection and charts come ready-made, but the history moves outside the repo, behind an account and a token that every machine and CI would hold. The key, the quiet rule and the Harness version breaks below would still have to be layered on top.
- github-action-benchmark, which charts results on a `gh-pages` branch from Actions. Only CI publishes through it, CI has no GPU (lavapipe's timings mean nothing), and the key, the quiet rule and the breaks would again be ours to add.
- A time-series database with Grafana. A server to run and back up for a few KB per record, and agents could no longer query the history with `git` and `jq`.
- The commit or the whole tree hash as the key. PR branches need not be up to date with `main`, and merges are squashed, so a PR head's commit never lands. A docs-only commit gets a new tree, and the sweep would bench it for nothing.
- Self-hosted Actions runners on the two machines, so that CI benches real GPUs. GitHub's guidance is that "self-hosted runners should almost never be used for public repositories on GitHub", "because any user can open pull requests against the repository and compromise the environment" ([Security hardening, Hardening for self-hosted runners](https://docs.github.com/en/actions/reference/security/secure-use)). A local scheduled task runs no code from a pull request. A GPU runner for CI is #301's decision.

## Consequences

- The key is source hash × adapter × backend × scene × Harness version. The source hash covers `src/`, `assets/`, `Cargo.toml`, `Cargo.lock` and `.cargo/`, a list held in one place in code, and a test fails when a new top-level path is classified neither way. A commit whose source hash didn't change inherits the record before it.
- A record path is its key, so a duplicate is a file that already exists. When two records share a key, the quieter one stays: the one with the lower GPU p95/median.
- A record joins a Series only if its tree was clean, its profile release, the machine quiet (preflight warned nothing, and p95/median below about 1.06), and its source `main`'s. Every other record is kept, tagged with its reasons, and drawn only on request. An Experiment's records are kept under its name and never join a Series.
- The Harness version rises whenever a change alters what a metric means, and each record carries its scene's content hash. A chart breaks its line where either changes, so a new definition never reads as a Regression.
- The 61 commits that touched `bench/baselines/` before the history existed are imported as records, each with the Harness version in force at its commit.
- Publishing never blocks work. A failed publish warns and the bench still succeeds; `verify` never publishes. Baselines are still chosen by hand, and a re-recorded Baseline's record is marked as one.
- A timing record far from its Series' recent median is a suspect. The sweep re-benches it, bisects the commits behind it, and files a confirmed Regression that no re-recorded Baseline covers as one `needs-triage` issue.
- A bad publish is undone with `git revert` on `bench-history`. The branch holds only data and a README, is never merged, and CI writes it only from pushes to `main`, never from a pull request.
