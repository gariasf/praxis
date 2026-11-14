# The repo is public under MIT

The repository is public on GitHub and everything in it is published under the MIT licence (`LICENSE`, `Cargo.toml`): `main`, every PR's branch, `bench-history` and `pr-images` alike. The owner kept MIT on 2026-10-20 ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6285721095)) and kept the repo public on 2026-11-14 ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6878038353), from queue item 10 of `docs/audit/strategy/2026-11.md`, which showed no cost of delay in either choice). What is published stays MIT: a later change of licence, or of visibility, covers new work only. The decision is revisited before the first outside contribution is accepted, since relicensing one later needs its author's agreement, and before a first public release.

## Considered options

- Taking new work private. What is published would stay public and MIT, and the review showed no cost in leaving the choice for later.
- A more protective licence for new work: MPL-2.0, GPL-3.0, PolyForm Noncommercial 1.0.0 or BUSL-1.1, noted on 2026-10-20 for a later change. None is taken.

## Consequences

What being public under MIT rules out:

- Anything the owner would not publish, on any branch, issue or PR: a credential, a token, private data. A Bench record and a PR's images are published as a commit is.
- A committed file whose licence forbids redistribution or commercial use. Every file under `assets/` has a manifest entry and a credit, which `cargo test` checks (`docs/research/test-assets.md`); a larger source is fetched from its publisher into a git-ignored cache, and a file under a noncommercial or restrictive term is dev-only: never committed, and no committed Golden is rendered from it.
- Code copied from a noncommercial or unclear licence. Adapted code cites its source and licence, a vendored crate keeps its licence files (`vendor/README.md`), and every dependency's licence expression is on the allowlist in `tests/dependency_licences.rs`.
- Self-hosted Actions runners on the owner's machines, which a pull request from anyone could run code on (ADR 0024). CI runs on GitHub's hosted runners, without a GPU, and local scheduled tasks bench the real ones.
