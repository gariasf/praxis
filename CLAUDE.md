<!--
For AI assistants: This file describes durable conventions, not
phase-specific work. If you observe drift between this file and the
code, surface it. If you make changes that warrant a CLAUDE.md update,
update it in the same change. Don't let this file go stale.
-->

# Praxis — AI Guidelines

Praxis is a 3D engine built from scratch in Rust, aimed at a production
open-world engine (Skyrim / RDR class) on cutting-edge hardware. The
owner coordinates what is worked on; agents implement, commit, open PRs
and merge them once reviewed, without asking, and only wait for visual
or interactive checks and roadmap decisions (`docs/agents/workflow.md`).

## Project posture

- **Efficiency and correctness are paramount**: CPU, GPU, memory, and
  tokens. Every change is validated; evidence beats assertion.
- **Architecture is production-grade now.** Load-bearing decisions
  (`docs/adr/`) are made for the eventual scope. Implementation depth
  follows the live milestone.
- **No throwaway code.** Clean naming, structure, and conventions at
  every step. The one exception is a prototype ticket's branch, which
  is never merged (`docs/agents/workflow.md`, Tickets).
- **Goals are explicit.** Work traces to a ticket in the live milestone
  (`docs/agents/workflow.md`).
- Teaching happens on request, and the standing request is a field guide
  for each session's important new concepts (`docs/agents/workflow.md`).

## Documents

- `docs/agents/workflow.md` — how work moves: ladder, tickets, PRs,
  budgets, session start. Read before starting, closing, or reordering
  work.
- `docs/agents/sessions.md` — which skill starts which kind of session.
  Read when the owner asks what to run, or when unsure which flow fits.
- `docs/agents/machine.md` — how sessions share one machine: claims,
  worktrees, quiet turns. Read before the first build, bench or worktree
  removal while other sessions run.
- `CONTEXT.md` — glossary; its terms are the vocabulary of code,
  tickets, and reviews.
- `docs/adr/` — decisions that are hard to reverse; a change that
  contradicts one says so.
- `docs/references.toml` — the external sources the engine relies on,
  each with its section and a verbatim quote; code, ADRs and notes cite
  its keys (`[ref:<key>]`) beside the prose citation.
- `docs/guides/` — the field guides: the engine's reference, a page per
  mechanism and per concept, which GitHub Pages serves from `main`. Read
  when you need one of those concepts; a page describes the commit its
  *Describes* line names and may be stale since. `docs/agents/workflow.md`
  says when to write or rewrite one.
- `docs/bench/` — the page that draws the Bench history's Series from
  `bench-history` in the browser (ADR 0024), on Pages beside the guides.
- `docs/audit/` — the audits: `deviations.md`, where the code should not
  match the literature and why, read before judging code against a
  source; `<milestone>/<lane>.md`, each Audit lane's report;
  `strategy/<year>-<month>.md`, a strategy review: the ladder, the ADRs
  and the way of working against the owner's goals, with its Decision
  queue. `docs/agents/audit.md` says how a lane runs.
- `ROADMAP.md` — long-range vision: target shape, design principles,
  scripting plan. Decisions are ADRs; the order of work is the ladder.

If any of these conflicts with current code, **the code is the source
of truth**. Flag the drift to the owner explicitly; do not silently
follow stale guidance.

## Agent skills

### Issue tracker

GitHub Issues for this repo, via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`); each label string equals its role name. `fog` marks a ticket placed at a later milestone's charting, off the triage queue and the frontier. See `docs/agents/triage-labels.md`.

### Audit

An Audit lane of a Consolidation milestone, and the standing form of a milestone exit's visual audit and source check: shared brief, parallel read-only readers, adversarial verifier, report, triage. See `docs/agents/audit.md`.

### Domain docs

Single-context: one `CONTEXT.md` plus `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Stack (frozen between milestone exits)

- **Language:** Rust (edition 2024)
- **Graphics:** wgpu 30, WGSL shaders
- **Windowing:** winit 0.30
- **Math:** glam
- **ECS:** `bevy_ecs` (sub-crate only — no full Bevy stack), with
  `bevy_tasks` for its compute pool
- **Assets:** glTF via `gltf` crate, images via `image` crate,
  tangents via `bevy_mikktspace`, BC encoding via `intel_tex_2`, all
  four behind the `cook` feature; Blobs via `ktx2`, `zstd`, `meshopt`
  and `sha2`, Arenas via `offset-allocator` (ADRs 0017, 0018)
- **Logging:** `tracing` + `tracing-subscriber` (with `env-filter`)
- **Structure:** Single crate: a library plus a thin binary.

Pinned in `Cargo.toml`. Version bumps and new dependencies are owner
decisions: propose them in the ticket, justify them in the PR. Each
milestone's exit reviews the releases published since the last one
(`docs/agents/workflow.md`, The ladder), and checks ADR 0007's reopen
trigger there. One
standing exception: until wgpu 31 ships, a fix Praxis needs from wgpu's
trunk is carried on a vendored 30.0.2 crate under `vendor/`, behind
`[patch.crates-io]`, without a further decision; `vendor/README.md` names
each upstream commit applied.

## Project shape (feature-based)

```
src/
  lib.rs, state.rs   # library root (window shell) + orchestrator
  bin/praxis.rs      # thin binary: command dispatch
  gpu/               # GPU context, windowed or headless
  capture/           # offscreen captures, Goldens, image difference
  preflight/         # `praxis preflight`
  bench/             # bench, compare, report, traverse, the Bench history
  stats/             # frame stats, GPU memory ledger, counting allocator
  streaming/         # the Chunk streamer, its IO pool, mip priority
  scenes/            # Reference scenes and their systems
  components.rs      # ECS components
  threading.rs       # compute pool, schedule executor
  camera/, time/, input/, helmet/, transform/, environment/
                     # feature modules: resource (+ systems) in one mod.rs
  assets/            # asset types, Pools, the Cooked cache reader, loader
  cook/              # the cooker (`cook` feature): cook, fetch, export-scene
  render/            # Renderer, a module per pass, targets, prepare,
                       bind_groups, readback, shaders/
tests/               # integration tests at agreed seams; goldens/
bench/               # budgets.json, baselines/<adapter>/, cycles/
scripts/             # verify, gallery, pr-lint, sweep, lavapipe-files,
                       cycles-reference/
vendor/              # crates.io crates carrying upstream fixes
```

What each holds, file by file and pass by pass: `docs/agents/project-shape.md`.

Rendering **pulls** from `World` (Design Principle #3 in `ROADMAP.md`):
prepare queries it into owned Frame data, and the Renderer draws only
that, reading no `World` (ADR 0004). ECS types never hold GPU buffers
(Design Principle #1).

When a feature folder grows past one file, split into
`feature/{resource,system,...}.rs`. Don't pre-split.

## How agents work here

- Implement tickets end to end: code, tests, verification, PR, merge.
  Make routine calls yourself and state them in the PR; bring the owner
  only roadmap decisions (milestones, ADRs, dependencies) and visual or
  interactive checks, with a recommendation attached. A PR whose gallery
  reports a difference, that moves a Golden or changes a `.wgsl` file, or
  that needs a visual check, shows before and after images
  (`docs/agents/workflow.md`, PRs).
- Work the live milestone's frontier. Work that is not a ticket becomes
  a proposed ticket first.
- Review before the PR opens: `/implement` closes with a local
  `/code-review`; run it and fix what it finds. It is the only
  semantic review (`docs/agents/workflow.md`, Review).
- A bug-fix is not a refactor and a feature is not a cleanup. Do what
  the ticket says; its acceptance criteria are the ceiling.
- The one licence to refactor is an audit's code-health fix ticket, with
  proof of unchanged behaviour in its PR: Goldens bit-exact, counters
  equal, timings within noise. One that changes an image is a defect
  ticket instead.
- Don't add comments, docstrings, or type annotations to code you
  didn't change.
- Reviews flag correctness, safety, and budget regressions.
  `cargo fmt` and `clippy` own style.
- Prefer deep modules behind small interfaces, tested at those seams
  (skills: `codebase-design`, `tdd`).

## Code conventions

- `cargo fmt` and `cargo clippy` must pass.
- Use `tracing::info!` / `error!` / etc. — never `println!` or `log` (tested).
- Prefer structured fields:
  `tracing::info!(field = value, "message")` over `format!`-style
  messages.
- Default log filter is `praxis=debug,wgpu=warn,naga=warn` via
  `.cargo/config.toml`; override per run with `RUST_LOG`.
- Bevy imports: `use bevy_ecs::prelude::*;` for what the prelude
  exports; items outside it by path.
- WGPU types: wgpu's own types by full path (`wgpu::Buffer`); wgpu-hal's
  `api::Metal` and traits imported `as _` excepted.
- Identifiers are full words, WGSL included (`normal_dot_view`, not
  `n_dot_v`; shading names tested). Put a textbook symbol in a comment
  where it helps.
- Pool lookups return `Option<&T>`. Callers handle `None`
  gracefully (warn + skip in render path); never `.unwrap()` in a
  hot loop.
- Shaders are WGSL parts in `src/render/shaders/`, assembled into modules
  by its `mod.rs` after the bindings and constants it declares from Rust;
  a function under an oracle lives in a shared part.
- No `unsafe` unless absolutely necessary and clearly explained.
- Keep dependencies minimal. Use feature flags
  (`image = { version = "0.25", default-features = false, features = ["png"] }`).
- Tests use the built-in harness (`cargo test`) at the seams agreed in
  the ticket. Test-framework crates are an owner decision.
- Every file under `assets/` has an entry in `assets/manifest.json` and
  its credit in `assets/ATTRIBUTION.md`; `cargo test` checks both.
  Policy: `docs/research/test-assets.md`.
- Every dependency's licence expression is on the allowlist in
  `tests/dependency_licences.rs`; `cargo test` checks it. A new
  expression is read before it joins.
- Docs, tickets and code comments name no patents and record no
  patent searches.
- Comments say what the code does and why in their own words, citing no
  ticket, PR or decision: only an upstream bug, adapted code's source and
  licence, a pinned third-party source or an ADR (ticket numbers tested).

## Review checklist

When reviewing changes, check for:

- [ ] A size-dependent target lives in the targets module
      (`src/render/targets.rs`), whose `RenderTargets::new` creates every
      target and runs again on resize. Bind groups are snapshots, not
      pointers: whatever recreates a resource rebuilds the groups holding
      it, so a pass builds the bind groups that read targets in one
      function, `target_groups`, and holds none of them: the Renderer's
      `BoundTargets` holds what it returns beside the targets, both built
      by one function at creation and on every resize, and hands it to
      each function that records the pass. No pass constructor takes the
      targets
- [ ] A binding is written once, in its bind group's definition
      (`src/render/bind_groups.rs`), which declares it in WGSL and gives
      its layout and its group's index in pipeline layouts and
      `set_bind_group`; no WGSL part declares one. A group a pass uses
      is listed in its shader module, its pipeline layout and its render
      pass. Rust-only padding fields start with `_`
- [ ] A pass constructor that binds a shared buffer into a group of its
      own takes `&SharedBuffers`, and one that binds a buffer another
      pass owns takes that pass and calls its getter, so a binding's name
      is spelled only in the file of the pass whose group it is. A shader
      module is assembled with a Render capacity only if one of its parts
      reads it
- [ ] No World position reaches the GPU in f32: only an anchor plus an
      f32 offset, composed camera-relative (ADR 0014)
- [ ] Depth has no far plane, and WGSL that compares or reduces depth
      (shadow, Hi-Z) keeps reversed depth's direction (ADR 0015)
- [ ] Light values are photometric as glTF defines them (lux, candela,
      cd/m²) and exposure is an EV100; HDR colour is pre-exposed before
      fp16 stores it and clamped to 65504, so no backend sees infinity
      (ADR 0023)
- [ ] Matrices reach WGSL column-major (glam `to_cols_array_2d`)
- [ ] A new or changed shading function in WGSL has an oracle test
      in `src/render/shader_function_tests.rs`
- [ ] Texture formats match between creation and surface config, and
      sRGB is encoded once: the tonemapping pass alone encodes it, in its
      shader, into 8-bit UNORM targets, so its dither lands on 8-bit steps
- [ ] Outside `src/render/`, no `unwrap()` on a GPU operation that could
      fail at runtime: `?` or explicit handling
- [ ] No GPU resources stored on ECS components — components are pure
      authoring data
- [ ] Code adapted from elsewhere cites its source and licence;
      noncommercial or unclear licences are never copied, and nothing
      is pushed, to any branch, that the public repo may not publish
      under MIT: a secret, or a file its licence keeps from
      redistribution (ADR 0025)
- [ ] A cost that rejects a technique, or decides a reopen trigger,
      names its adapter and is final only with the figure on
      the M4 Pro and the RTX 2070 SUPER from one engine source
      (ADR 0013)
- [ ] Every reference the change's code or ADRs rely on or add exists,
      says what the change claims, is cited to the right place, and is
      implemented as it says; one that cannot be opened and confirmed is
      fixed or removed; a survey's rows wait for its decision
      (`docs/agents/workflow.md`, Review: Sources)
- [ ] Every name the change renames, removes, changes the behaviour
      of or adds to a set a glossary entry lists (pass, buffer, format,
      Blob field, constant, Frame stats field, an ADR's status word),
      and every thread or machine it moves work to or off, is grepped
      in comments, ADRs, `CONTEXT.md`, `docs/agents/` and
      `docs/research/`; each clause the change makes false (a doc
      comment, an "until", "today", "yet", "planned" or count, a
      Consequence, a clause borrowing another ADR's status, a
      prediction, a research note's account of Praxis, or its open
      question a measure answers) is rewritten or dated in the same PR;
      a new glossary term is grepped against `pub struct`/`enum` names
      and a same-named item of another meaning goes in its `_Avoid_`
      line
- [ ] Frame data holds no borrow into the ECS World, and only the
      main thread submits, presents or polls (ADR 0016)
- [ ] Whatever changes what an Instance holds makes prepare write its
      slot; other slots keep what earlier frames wrote (ADR 0003)
- [ ] The GPU builds the frame's draw in slot order within each culling
      phase, never in the order atomics land: on the M4 Pro, which
      triangles precede others in a draw changes implicit derivatives, so
      captures would differ run to run
- [ ] Frame stats still see new work: GPU buffers, textures and
      acceleration structures are created through the GPU memory
      ledger (`GpuMemory`) under their kind, acceleration structures at
      the size wgpu-hal's query gives (`blas_bytes`, `tlas_bytes`), a
      compacted BLAS at its adapter's estimated fraction of it;
      the GPU stats readback's buffers stay out, as they measure the
      frame, and so does the load staging's belt, which `uploaded_bytes`
      counts (ADR 0013); per-frame GPU writes add to `uploaded_bytes`, and systems are added inside
      `SystemTimer::scope`
- [ ] Streaming uploads go through the load staging (`LoadStaging`: its
      kept slots, else a `StagingBelt`), never
      `Queue::write_texture`; no GPU buffer grows at runtime, and a
      request past a buffer's capacity is refused and logged. Only
      per-frame staging grows, up to the buffers a frame writes, which
      it asserts (ADRs 0003, 0017)
- [ ] The runtime loads cooked Blobs only and references Assets by
      Asset ID, never by source path (ADR 0018)
- [ ] A cooked Mesh Blob is the same bytes on every machine: the two
      hash tests (`src/cook/mesh.rs`, `src/cook/far_proxy.rs`) pass on
      each. A hash that moves on one machine alone has met glam's SIMD
      rounding (`Quat`, `Vec4`, `Vec3A`), a platform's f32 sine or
      cosine, or a C++ build that fuses a multiply and an add
      (`.cargo/config.toml`)
- [ ] The main pass reads no screen-space history, and no change rules
      out its 4x MSAA (ADRs 0008, 0019, 0020)
- [ ] No main-pass pipeline enables alpha-to-coverage. Partial coverage
      is a `sample_mask` the depth passes write, through pipelines of
      their own, the same samples in every fragment of a pixel, from a
      stage that reads a flat input (ADR 0027)
- [ ] A newly used wgpu feature or limit joins the Capability floor,
      or, outside the capability matrix's Tiers 1-2, has a path for
      adapters without it (ADR 0021)
- [ ] Game-state changes run in the fixed schedule, which reads no
      wall clock and has no ambiguous system order (ADR 0022); every
      entity whose World transform it changes, children included,
      carries `PreviousWorldTransform`, and a jump inserts `Teleported`
      before `reset_teleported` runs
- [ ] WGSL keeps sRGB and linear apart, has no divide-by-zero or NaN
      at grazing angles or zero roughness, and calls no `textureSample`
      under non-uniform control flow
- [ ] Workflows keep least-privilege permissions and pin each action
      to a commit SHA with its version in a comment, as `pr-lint.yml`
      does

## Maintenance

This file is durable guidance. Keep it short — the more rules, the
faster they drift.

**Update this file in the same PR when:**

- Folder layout changes: a directory here, a file in
  `docs/agents/project-shape.md`
- A new cross-cutting convention is adopted
- An ADR is added or superseded
- A repeated correction emerges — same mistake twice means it's a rule

**Don't update for:**

- Ticket-specific work (lives in tickets and the map issue)
- One-off bug fixes
- Style nits

If you (AI or human) notice this file conflicts with current code,
flag it explicitly in the response. Don't silently follow stale
guidance.

### Agent fleet

One agent reads this file: Claude Code. No `AGENTS.md`,
`.cursor/rules/`, `.junie/guidelines.md`, or similar — the project
doesn't use those tools. If a new agent is onboarded later, extract
shared content into `AGENTS.md` and have `CLAUDE.md` carry only the
Claude-specific addenda. Don't pre-create.
