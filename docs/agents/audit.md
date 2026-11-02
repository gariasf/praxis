# Audit

How an Audit lane runs: one shared brief, parallel read-only readers that file Findings with evidence, an adversarial verifier that reopens every one, a report that lands by PR, and a triage with the owner that places each Finding. Run it for each lane of a Consolidation milestone (`M3.5` first, phase A, every lane on one frozen commit), and afterwards as the standing form of every milestone exit's visual audit and source check (`docs/agents/workflow.md`, The ladder). Start with "Audit lane <n> per docs/agents/audit.md", or `/audit <lane>` where that skill is installed. One lane per session; a lane larger than one session splits by module as its brief says, one session per module, one verifier over all of them.

The lane briefs below were decided for `M3.5` and name its referents (its spec issue, its map's fog lines). A later audit replaces those referents with its own in the PR that opens it, and keeps the rest.

## Inputs

Every reader and the verifier receive the same things, and nothing from chat:

- **The frozen commit**: the commit of `main` the audit reads, named on the milestone's spec issue. Every `where` is a `path:line` at that commit. A reader works in a worktree checked out at it, never at `main`'s tip, so a fix landing mid-audit moves no line.
- **The lane brief** below: scope, evidence kind, tolerance, severity default and module split.
- `docs/audit/deviations.md`: where the code should not match the literature, and why. A reader who finds the code matching an entry files nothing; one who finds an entry wrong files a Finding against the entry, not the code.
- `scripts/cycles-reference/DIFFERENCES.md`: the image half of the same list; a Cycles difference it describes is known, not a defect.
- `docs/references.toml`, the registry of the sources the code cites, for the lanes whose evidence is a source quote (correctness, docs and rules drift) or a published cost (performance).

## The pipeline

1. **Brief.** The session writes the lane's brief into each reader's prompt verbatim from this file, adding only the frozen commit, the module that reader owns, the report's path and the Finding template. The brief is shared: every reader of a lane reads the same one, so no reader's findings are shaped by a prompt the others lacked.
2. **Readers.** One read-only sub-agent per module in the lane's split, in parallel, at high effort (`docs/agents/sessions.md`, Effort). Each files Findings in the template below, every one with evidence in the lane's kind; a claim without that evidence is not filed. Readers change no file and run nothing that writes to the tree. A lane whose evidence is a measurement benches on the frozen commit, on a quiet machine with no peer benching, and keeps the bench JSON beside the report; `docs/agents/workflow.md`, Performance and memory, says how a figure is quoted.
3. **Verifier.** One sub-agent after the readers, independent of them, at high effort, adversarial: it reopens every Finding's evidence itself (re-runs the bench line, opens the source, reads the code at the line), trusts no reader's quote, and writes `confirmed`, `narrowed` (the claim rewritten) or `killed`, with one reason each. A Finding without evidence in its lane's kind is killed on form. Killed Findings stay in the report's closing Killed section with their reason, so the next audit does not re-find them.
4. **Report.** `docs/audit/<milestone>/<lane>.md` on `main`, by PR that closes the lane's ticket, in the form below. The PR's own review (`docs/agents/workflow.md`, Review) runs as for any docs PR.
5. **Triage.** With the owner, in the milestone's triage sessions, each Finding is placed: `fix`, a ticket in the milestone that names the Finding's id and carries its evidence; `fog`, a line under *Not yet specified* on the ladder's map issue with the trigger that revives it; or `wontfix`, with its reason. None is left open. The triage's PR fills the report's `placement` column. Fix tickets are capped per milestone and ranked by impact at the triage: the class first, then the magnitude within it; what falls past the cap becomes fog with a trigger.

The image and the scalability lanes are prepared AFK to the point of judgement and judged with the owner in the room: no reader files an image Finding the owner has not seen, and the scalability lane's output is a decision comment the owner writes with the session. The other lanes run to the triage without the owner.

## The Finding

One Finding is one defect at one location: a Markdown table row, with a block under the table for evidence that does not fit a cell, headed by the Finding's id. Readers fill 1 to 6, the verifier 7, the triage 8.

1. `id`: `<lane>-<n>`, as `L3-07`; stable across the verifier and the triage.
2. `claim`: one falsifiable sentence.
3. `severity`: the class below, plus the magnitude in the class's unit.
4. `where`: `path:line` at the frozen commit, or the doc heading, or scene, frame and pixel box.
5. `evidence`: in the kind the lane names: a measurement with its run conditions (adapter, commit, resolution, frames, load average, battery state, peers), a source quote with its `docs/references.toml` key, a 2x image crop with its path, or a quoted comment beside the code it misdescribes.
6. `expected`: what the source, ADR, spec story, Budget or convention says it should be.
7. `verifier`: `confirmed`, `narrowed` (the claim rewritten) or `killed`, with one reason.
8. `placement`: `fix` (ticket), `fog` (with its trigger) or `wontfix` (with its reason).

## Severity

Five classes in fixed rank; the triage ranks by class, then by magnitude within the class. No numeric score.

| Class | What it marks | Magnitude |
|---|---|---|
| `image` | a defect a viewer sees at native resolution | pixels past tolerance, scenes affected |
| `budget` | a Budget or gate breached | µs over |
| `correctness` | code disagrees with its ADR, spec story or source, with no visible defect | relative error, or the clause broken |
| `drift` | a doc, comment, rule or reference says what the code does not | load-bearing or not; readers misled |
| `health` | over-engineering, vestigial code, naming, module depth | lines, call sites |

## Deviations

`docs/audit/deviations.md` holds, in one table, what should not match the literature: what deviates, the source key, why, and the trigger that revisits it. `DIFFERENCES.md` stays the image half and is linked from it. A Deviation is never a code comment (numbers are banned there, and nobody reads across the tree) and never a `references.toml` field (a source has many uses; a Deviation belongs to a code site). A deviation past a lane's tolerance that the owner accepts at the triage joins the table in the triage's PR.

## The report

`docs/audit/<milestone>/<lane>.md`:

1. A Summary of three sentences.
2. Counts per Severity class and per verifier verdict.
3. The Findings table, in the template's columns, with the evidence blocks under it.
4. The Killed section: each killed Finding with the verifier's reason.
5. The performance lane adds its architectural notes; the docs and rules drift lane adds its Rule candidates.

Prose names no ticket or PR number except in a Finding's `placement`.

## Lane briefs

### Lane 1, correctness against intent

- Scope: every ADR's decision clauses against the code, a verdict per clause (`holds`, `broken`, `superseded by ADR n`); the milestone spec's stories (for `M3.5`, #188); every function under an oracle against its literature number, with the WGSL accuracy floor for its expression chain written beside the test's constant; every registry key cited in code implemented as the registry's quote says.
- Evidence: `path:line` plus the source quote with its key; for a number, the test's constant and the measured error.
- Tolerance: the oracle's own named constant. A deviation past it that the owner accepts goes to `deviations.md`.
- Severity default: `correctness`; `image` when a capture shows it.
- Split: render; shaders; cook with assets and scenes; tooling (bench, capture, preflight, scripts). Four readers, one verifier.

### Lane 2, image quality

- Scope: every Reference scene through `scripts/gallery.sh` with `PRAXIS_BLENDER` at 2560×1440, and the 3x box-filtered supersampled reference where aliasing is the question.
- Method: readers run AFK and prepare the gallery, each scene's mean 8-bit difference and linear-light RMSE against Cycles (recorded as that scene's first Cycles Baseline; the Baseline mechanism itself is a fix ticket the spec charts), and candidate 2x crops of terminators, highlights, silhouettes and shadow edges. The owner judges every crop in the room; no reader files an image Finding the owner has not seen. Each `DIFFERENCES.md` item is tagged in that file `deliberate until <trigger>` or `ticketed #n`.
- Evidence: scene, frame, pixel box, 2x crop praxis | Cycles | difference.
- Severity: `image`.

### Lane 3, performance, where we left off

- Scope: each timed pass on each scene against (i) its source's published cost, as a 2x profiling trigger only, and (ii) its own hardware floor on the measuring adapter, as the verdict; the "revive when" fog lines on the ladder's map issue (#17; 27 of them at `M3.5`) re-judged `fired`, `waiting` or `dead`; `town`'s remaining gap decomposed into named options with a cost estimate each.
- Evidence: bench JSON on the frozen commit, Apple M4 Pro, three rounds interleaved, the machine's quiet stated (load average, `pmset -g batt`, no peer benching), plus the floor's arithmetic. RTX figures only where an RTX Baseline already exists.
- Severity: `budget` when over a Budget or gate, else `health`. A pass within 2x of its floor but over 2x its source is an architectural note, closed in the report, not a Finding.
- Split: dense scenes (`town`, `texture-wall`); the rest. One verifier re-runs every quoted line.

### Lane 4, scalability and reality check against the milestones ahead

- Scope: every fixed capacity, binding floor and buffer size with its arithmetic at target scale (10 × 10 km, streaming Chunks, LOD, GI on the same TLAS); per ADR, `survives` or `must change`, with the clause; which of the audited milestones' choices the next milestone must undo.
- Evidence: `path:line` of the constant or clause plus the arithmetic.
- Output: a decision comment on the ladder's map issue that the next milestone's charting consumes, written with the owner in the room; readers prepare the table AFK. It does not chart that milestone.
- Placement default: `fog` with the trigger "<next milestone> charting"; `fix` only where code must change before the next milestone can start.

### Lane 5, code health

- Axes: over-engineering (an abstraction with one use, flexibility nothing exercises); vestigial code (unreferenced items, Render switches no scene sets); cognitive complexity (clippy's `cognitive_complexity` lint run as a probe, findings above its default threshold only); naming against the full-word rule; module depth (interface width against what it hides); comment accuracy.
- Coverage: `src/render` and the shaders in full; the rest every fifth file in path order, a wrong sample opening its module.
- Evidence: `path:line` and the claim; for a comment, the comment quoted beside the line it misdescribes.
- Severity: `health`; `drift` for a wrong comment. Fix tickets carry the refactor licence (`CLAUDE.md`, How agents work here): a refactor with proof of unchanged behaviour, Goldens bit-exact, counters equal, timings within noise; one that changes an image is a defect ticket instead.
- Split: as lane 1's.

### Lane 6, tests and flakes

- Scope: every ticket's Verification seam since `M0` against a test that exercises it; CI's failed-then-passed runs over the audited milestones as the flake list; Golden tolerances kept at their values (1 of 255 and 0 pixels) and each loosening since `M0` explained; every WGSL shading function against an oracle in `src/render/shader_function_tests.rs`; every conventions test against the rule it enforces.
- Evidence: the test name and the seam or function it misses, or the CI run id and its failure text.
- Severity: `correctness` when an uncovered seam carries a correctness claim, else `health`.
- Split: one reader on the tests, one on CI history, one verifier.

### Lane 7, docs and rules drift

- Scope: the ADRs, `CONTEXT.md`, `CLAUDE.md`, `docs/agents/*` and `docs/references.toml` against the code, the code winning; the references by the source-check rubric (#90: load-bearing claims in full, every fifth of the rest, a wrong sample opening its section; verdicts `confirmed`, `wrong`, `unsupported`, `stale`, `dead`).
- Rule candidates: a slip a reader sees twice goes in the report's Rule candidates section with both Findings' ids; the milestone's rules ticket decides, each rule backed by at least two Findings of its kind.
- Evidence: the doc heading and quote beside `path:line`.
- Severity: `drift`.

### Lane 8, guides

- Not a defect lane. Readers inventory: per volume, the mechanism it describes, the page in the guide's shape (`docs/agents/workflow.md`, PRs) that absorbs it, and what is history; per mechanism in the frame, whether any volume covers it.
- Findings: a mechanism with no page; a volume claim the code no longer matches (`drift`).
- Output: the volume-to-page table the guide rewrite executes from.
