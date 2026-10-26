# Workflow

How work moves through this repo. Read before starting, closing, or reordering a ticket or milestone.

The owner decides what is worked on. Agents implement, commit, open PRs and merge them without asking (see Review), then take the next frontier ticket. A PR waits for the owner only for visual or interactive judgement (a render or Golden to judge, a check only the owner can run) or an important roadmap decision: milestone order or exit criteria, a new or superseded ADR, a new dependency or version bump ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6154239897)). Any other choice the agent makes itself and states with its reason in the PR body. A decision that lives only in chat is not made: it lands as an ADR (`docs/adr/`), a comment on the map issue, or a ticket comment.

## The ladder

- Work is ordered as GitHub milestones (`M0`, `M1`, …). Exactly one is **live**: the lowest-numbered open milestone.
- Only the live milestone has tickets. Later milestones are **fog**: one line each under *Not yet specified* on the map issue (label `wayfinder:map`). Fog is charted into tickets when the milestone before it nears exit.
- A milestone's exit criteria are commands plus numbers, written on the milestone before its first ticket opens. It closes when every criterion has been run and its output pasted on the milestone. "Looks done" never closes a milestone.
- The ladder's order, each milestone's one-line goal and its provisional exit criteria are recorded in the [ladder decision](https://github.com/gariasf/praxis/issues/75#issuecomment-6141801746). A milestone implements one or two ADRs in their dependency order. Provisional criteria become firm at the boundary, when the milestone is created.
- Every milestone's exit also includes the standard clause: the verify command (`scripts/verify.sh`) exits 0 within 30 lines; a new or extended Reference scene shows the milestone's work, with its Goldens committed on Metal and software Vulkan; Baselines are re-recorded and no Budget regresses unless an ADR or map comment moved it; a profile of the Reference scenes is recorded in `docs/performance.md` and the top three offenders are ticketed; a literature scan for the next milestone's topics is checked against the ADRs, and that milestone's ADRs and research notes have their load-bearing claims verified against primary sources (#90's [rubric](https://github.com/gariasf/praxis/issues/90), [decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6269184423)).
- Reordering is a decision comment on the map issue: what moved, why, and what it displaces. The rail numbers in this file (size ceiling, review rounds) and the rule for PRs in flight change the same way: a decision comment, then an edit here.

## Tickets

- A ticket is a **tracer bullet**: a thin slice that is demoable or verifiable on its own and fits one fresh session.
- Size ceiling: about 400 changed lines, not counting goldens, lockfile, and generated or cooked data. A ticket that cannot fit splits along a seam. A wide mechanical change goes expand–contract: add the new form, migrate callers in batches, delete the old form.
- Acceptance criteria are the **ceiling**. Anything a reviewer wants beyond them becomes a note under *Not yet specified*, with the trigger that would revive it.
- Every ticket carries a verification seam agreed before coding (`/tdd`) and, for render or hot-path changes, the metric it must not regress.
- One ticket per session. Start from the ticket body, its linked ADRs, and `CONTEXT.md`; chat history is not a source.
- A technique decision (rendering, lighting, visibility, memory and streaming, threading, formats, physics, animation, audio, AI) is blocked by a `wayfinder:research` ticket titled "State of the art: …" ([rule](https://github.com/gariasf/praxis/issues/17#issuecomment-6124930282)). It surveys research since 2019 (SIGGRAPH/TOG, HPG, EGSR, I3D, JCGT, arXiv) plus each technique's foundational paper, and shipped practice (GDC, SIGGRAPH Advances in Real-Time Rendering, studio and vendor publications); shipped engines are evidence, not templates. Per technique the note gives what it buys, published cost, maturity, fit (`docs/research/wgpu-capability-matrix.md`, ADR 0013's budgets on the M4 Pro, ADR 0008's stance on temporal accumulation) and the licence of any reference code. It names no patents and runs no patent search (owner decision, [#91](https://github.com/gariasf/praxis/issues/91)). It ends in options, not a decision, and stays compact in `docs/research/`: one comparison table and at most a paragraph per technique.

## PRs

- One ticket per PR (`Closes #N`). Any number may be in flight; PRs that share a module need their tickets to name the merge order, and the later one rebases onto the earlier ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6172436344)). A ticket that records Baselines does so while no other session loads the machine, or takes them from CI's artifact.
- One review, the agent's own, before the PR opens (see Review). One fix commit, and a verification pass only when a fix touched correctness-critical logic. Findings still open after that become tickets or fog notes.
- The verify command, `scripts/verify.sh`, exits 0 before the PR opens: formatting, lint, build, tests (Goldens required) and the bench counters against Baselines and Budgets, one line per step, about a minute once built. Its output is the Verification block.
- A session that introduces important new concepts (a technique, a model, a pass) makes a field guide for them without being asked, while the work happens ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6336434405)): an explainer artifact in the series that the pinned "Praxis Field Guides" hub lists, with the series' look, live demos, concept tooltips that define terms as `CONTEXT.md` does, a reading list and the series' navigation, added to the hub when published.
- Description follows `.github/pull_request_template.md`: what changed, how it was verified (command and output, 30 lines at most), metric deltas, and under Decide only the owner items named above.
- A PR that changes what a Reference scene renders, or that asks the owner to look at something, shows images in its description whenever they can be made ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6262013466)): each affected scene before and after, a difference image where the change is subtle, a zoom where it is small. `praxis capture <scene> <png> [<width>x<height>]` renders a scene headless. The images go on the `pr-images` branch, one directory per PR, never merged, and the description links them by commit SHA. Where no image can be made, the description says why.
- Commit subject 72 characters at most; the body says why, not what.

## Review

CI owns mechanical checks. One semantic review follows, the agent's own, before the PR opens. No external reviewer runs (CodeRabbit was removed, 2026-10-14). The agent owns every fix and merges on evidence.

1. **Review.** When the build is done and local checks pass, run `/code-review` against the base branch: a Standards axis (this repo's conventions and the `CLAUDE.md` review checklist) and a Spec axis (the ticket's acceptance criteria), in parallel sub-agents, plus a third sub-agent beside them for a Sources axis ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6421792794)). The Standards sub-agent runs at medium effort, the Spec and Sources sub-agents at high (`docs/agents/sessions.md`, Effort). A docs-only PR gets the agent's own read of the diff against the same three axes, without sub-agents.
   - **Sources.** Every reference the diff relies on or adds (a paper, specification, book, vendor document, other codebase, or measurement) is checked for four things: the source exists; it is a primary or trusted source and says what the diff claims; the citation points to the right place (section, equation, version, commit) and quotes it faithfully; and the code implements it correctly (formula, constants, units, edge cases). The reviewer opens each source and never vouches from memory. A reference it cannot open and confirm is fixed or removed, never kept on trust. The PR lists each reference checked and its verdict under Verification.
2. **Disposition each finding.**
   - *Fix*: verified, and a correctness, safety, budget, or ADR problem, or a quick win inside the changed lines. All fixes land in one commit before the PR opens.
   - *Decline*: wrong, or beyond the acceptance criteria. One-line reason.
   - *Defer*: true but outside the ticket. A note under *Not yet specified* with its revival trigger.
3. **Open as draft, ready when green.** Fill the PR template, with the review's counts (found, fixed, declined, deferred), the deferred list and the sources checked under Verification. Open the PR as a draft and run `gh pr ready` once CI passes.
4. **Merge** when CI is green, the review's counts are in the PR, and no Decide item is unanswered. Any agent squash-merges any such PR, its own or another session's, without asking, then removes the worktree and branch. A PR that needs the owner's visual or interactive judgement, or carries a roadmap decision, waits for the owner instead.

A declined finding class that recurs becomes a line in the `CLAUDE.md` review checklist or a `CODING_STANDARDS.md` rule.

## Performance and memory

- Budgets and Baselines live in the repo beside the benches (`bench/`, since M0). A render or hot-path change reports its delta against its Baselines.
- A budget moves down freely and moves up only through an ADR or map comment.
- Measurement is always on; optimisation is evidence-driven: fix what measurement names, in the order it names them.

## Correctness

- Tests live at pre-agreed seams. Expected values come from an independent source: a literal, an analytic oracle, a reference tool, a reference output produced by a different path.
- Mechanical rules belong in CI. `CODING_STANDARDS.md`, when it exists, carries judgement calls only.

## Session start

1. Run `cargo run --release -- preflight` in the checkout. It prints the session's starting state in ten lines or fewer: the adapter and which machine it is, whether its Goldens, Baselines, fetched files and Cooked cache are present, GPU load and other praxis processes, the open PRs and the live milestone's assigned tickets. A `FAIL` line blocks `scripts/verify.sh` and names its fix; a `warn` line holds off a bench.
2. Orient: this file, `CLAUDE.md`, the map issue body (`gh issue list --label wayfinder:map`).
3. Take the frontier: `gh issue list --milestone <live> --label ready-for-agent --state open --limit 200 --json number,title --jq 'sort_by(.number)'`. Tickets are published blockers-first, so ascending number is a valid order. Take the lowest number whose `gh api repos/gariasf/praxis/issues/<n> --jq .issue_dependencies_summary.blocked_by` is 0.
4. Claim it (assign) before any work.
5. In the ticket's fresh worktree, run preflight again and the fixes its `FAIL` and `warn` lines name (a new worktree has no fetched files: `cargo run --release -- fetch`), then `scripts/verify.sh`. A red start is reported on the ticket before work begins; the failing step's log is in `target/verify/`.

## Context hygiene

- `CLAUDE.md` holds pointers; reference material sits behind them.
- Validation commands print 30 lines at most on success and the failing detail on failure.
- Research results are cached in `docs/research/` and linked from tickets, so each is read once.
