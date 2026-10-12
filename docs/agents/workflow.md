# Workflow

How work moves through this repo. Read before starting, closing, or reordering a ticket or milestone.

The owner decides and merges. Agents implement and open PRs. A decision that lives only in chat is not made: it lands as an ADR (`docs/adr/`), a comment on the map issue, or a ticket comment.

## The ladder

- Work is ordered as GitHub milestones (`M0`, `M1`, …). Exactly one is **live**: the lowest-numbered open milestone.
- Only the live milestone has tickets. Later milestones are **fog**: one line each under *Not yet specified* on the map issue (label `wayfinder:map`). Fog is charted into tickets when the milestone before it nears exit.
- A milestone's exit criteria are commands plus numbers, written on the milestone before its first ticket opens. It closes when every criterion has been run and its output pasted on the milestone. "Looks done" never closes a milestone.
- Reordering is a decision comment on the map issue: what moved, why, and what it displaces.

## Tickets

- A ticket is a **tracer bullet**: a thin slice that is demoable or verifiable on its own and fits one fresh session.
- Size ceiling: about 400 changed lines, not counting goldens, lockfile, and generated or cooked data. A ticket that cannot fit splits along a seam. A wide mechanical change goes expand–contract: add the new form, migrate callers in batches, delete the old form.
- Acceptance criteria are the **ceiling**. Anything a reviewer wants beyond them becomes a note under *Not yet specified*, with the trigger that would revive it.
- Every ticket carries a verification seam agreed before coding (`/tdd`) and, for render or hot-path changes, the metric it must not regress.
- One ticket per session. Start from the ticket body, its linked ADRs, and `CONTEXT.md`; chat history is not a source.

## PRs

- One ticket per PR (`Closes #N`). At most two PRs open at once, on disjoint modules.
- One review → fix round. Findings still open after it become tickets or fog notes.
- Description: what changed, how it was verified (command and output, 30 lines at most), metric deltas, decisions needed from the owner.
- Commit subject 72 characters at most; the body says why, not what.

## Performance and memory

- Budgets and baselines live in the repo beside the benches (M0 creates them). A render or hot-path change reports its delta against baseline.
- A budget moves down freely and moves up only through an ADR or map comment.
- Milestone exit includes a profile of the reference scenes and the top three offenders ticketed.
- Measurement is always on; optimisation is evidence-driven: fix what measurement names, in the order it names them.

## Correctness

- Tests live at pre-agreed seams. Expected values come from an independent source: a literal, an analytic oracle, a reference tool, a golden produced by a different path.
- Mechanical rules belong in CI. `CODING_STANDARDS.md`, when it exists, carries judgement calls only.

## Session start

1. Orient: this file, `CLAUDE.md`, the map issue body (`gh issue list --label wayfinder:map`).
2. Find the live milestone: `gh api repos/gariasf/praxis/milestones --jq '.[] | "\(.title) open=\(.open_issues) closed=\(.closed_issues)"'`.
3. Take the frontier: `gh issue list --milestone <live> --label ready-for-agent --state open`; first unblocked ticket in milestone order.
4. Claim it (assign) before any work.

## Context hygiene

- `CLAUDE.md` holds pointers; reference material sits behind them.
- Validation commands print 30 lines at most on success and the failing detail on failure.
- Research results are cached in `docs/research/` and linked from tickets, so each is read once.
