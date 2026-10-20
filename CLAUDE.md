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
  every step.
- **Goals are explicit.** Work traces to a ticket in the live milestone
  (`docs/agents/workflow.md`).
- Teaching happens on request.

## Documents

- `docs/agents/workflow.md` — how work moves: ladder, tickets, PRs,
  budgets, session start. Read before starting, closing, or reordering
  work.
- `docs/agents/sessions.md` — which skill starts which kind of session.
  Read when the owner asks what to run, or when unsure which flow fits.
- `CONTEXT.md` — glossary; its terms are the vocabulary of code,
  tickets, and reviews.
- `docs/adr/` — decisions that are hard to reverse; a change that
  contradicts one says so.
- `ROADMAP.md` — long-range vision: target shape, design principles,
  scripting plan. Decisions are ADRs; the order of work is the ladder.

If any of these conflicts with current code, **the code is the source
of truth**. Flag the drift to the owner explicitly; do not silently
follow stale guidance.

## Agent skills

### Issue tracker

GitHub Issues for this repo, via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`); each label string equals its role name. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` plus `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Stack (frozen)

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
decisions: propose them in the ticket, justify them in the PR.

## Project shape (feature-based)

```
src/
  lib.rs, state.rs             # library root (window shell) + orchestrator
  bin/praxis.rs                # thin binary: command dispatch
  gpu/                         # GPU context, windowed or headless
  capture/                     # offscreen render read back as an image
  bench/                       # Reference scene medians, compared with
                                 Baselines and Budgets
  stats/                       # frame stats: CPU and GPU times, counts,
                                 GPU memory, counting allocator
  scenes/                      # Reference scenes: generated content,
                                 camera paths in frames, frame deltas,
                                 the orrery's Spin system
  components.rs                # ECS components (Transform, MeshRef)
  threading.rs                 # compute pool, schedule executor (ADR 0016)
  camera/, time/, input/,      # feature modules: resource (+ systems)
    helmet/, transform/           per domain in one mod.rs
  assets/                      # asset types, pools, the Cooked cache
                                 reader (Asset IDs, Blobs), and the
                                 loader from one into the other
  cook/                        # the cooker behind the default-on `cook`
                                 feature: `praxis cook` turns manifest
                                 sources and Reference scenes into Blobs
                                 in git-ignored `cooked/` (ADR 0018)
  render/                      # Renderer (owns the Pools, reads no
                                 ECS World, calls the passes in order),
                                 a module per pass (main_pass: its
                                 pipeline and recording), targets (every
                                 size-dependent target, the depth
                                 conventions), prepare (ECS World to
                                 Frame data), bind_groups (each defined
                                 once), instance, uniforms, staging,
                                 shaders/ (WGSL parts and the modules
                                 each pipeline assembles from them)
tests/                         # integration tests at agreed seams
  goldens/<backend>[-cpu]/     # committed Goldens; PRAXIS_UPDATE_GOLDENS=1
                                 cargo test --test goldens rewrites them
bench/                         # budgets.json (ADR 0013) and baselines/
                                 <adapter>/<scene>.json: the M4 Pro's,
                                 and CI's lavapipe (`--ci`); `praxis
                                 bench <dir>` then `praxis compare <dir>`
scripts/fetch-assets.sh        # downloads the manifest's fetched assets
scripts/verify.sh              # the verify command: fmt, clippy, build,
                                 tests with Goldens, bench counters
                                 against Baselines and Budgets
```

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
  interactive checks, with a recommendation attached. A rendering change
  or a visual check shows before and after images in its PR
  (`docs/agents/workflow.md`, PRs).
- Work the live milestone's frontier. Work that is not a ticket becomes
  a proposed ticket first.
- Review before the PR opens: `/implement` closes with a local
  `/code-review`; run it and fix what it finds. It is the only
  semantic review (`docs/agents/workflow.md`, Review).
- A bug-fix is not a refactor and a feature is not a cleanup. Do what
  the ticket says; its acceptance criteria are the ceiling.
- Don't add comments, docstrings, or type annotations to code you
  didn't change.
- Reviews flag correctness, safety, and budget regressions.
  `cargo fmt` and `clippy` own style.
- Prefer deep modules behind small interfaces, tested at those seams
  (skills: `codebase-design`, `tdd`).

## Code conventions

- `cargo fmt` and `cargo clippy` must pass.
- Use `tracing::info!` / `error!` / etc. — never `println!` or `log`.
- Prefer structured fields:
  `tracing::info!(field = value, "message")` over `format!`-style
  messages.
- Default log filter is `praxis=debug,wgpu=warn,naga=warn` via
  `.cargo/config.toml`; override per run with `RUST_LOG`.
- Bevy imports: `use bevy_ecs::prelude::*;` (not piecemeal).
- WGPU types: full path (`wgpu::Buffer`), don't import individual types.
- Identifiers are full words, WGSL included (`normal_dot_view`, not
  `n_dot_v`). Put a textbook symbol in a comment where it helps.
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
  patent searches (#91).

## Review checklist

When reviewing changes, check for:

- [ ] A size-dependent target lives in the targets module
      (`src/render/targets.rs`), whose `RenderTargets::new` creates every
      target and runs again on resize; a pass builds the bind groups that
      read targets in one function, called at creation and after every
      resize (bind groups are snapshots, not pointers)
- [ ] A binding is written once, in its bind group's definition
      (`src/render/bind_groups.rs`), which declares it in WGSL and gives
      its layout and its group's index in pipeline layouts and
      `set_bind_group`; no WGSL part declares one. A group a pass uses
      is listed in its shader module, its pipeline layout and its render
      pass. `cargo test` checks each binding's stages against the entry
      points that use it, and the structs the CPU writes field by field
      against their Rust twins (`src/render/shader_tests.rs`); Rust-only
      padding fields start with `_`
- [ ] No World position reaches the GPU in f32: only an anchor plus an
      f32 offset, composed camera-relative (ADR 0014)
- [ ] Depth is cleared to 0 and compared with `Greater`, with no far
      plane; shadow and Hi-Z passes follow the same direction (ADR 0015)
- [ ] Matrices reach WGSL column-major (glam `to_cols_array_2d`); a
      `mat3x3` or `array<vec3>` field is padded per column or element
      on the Rust side (the test compares sizes, not strides)
- [ ] A new or changed shading function in WGSL has an oracle test
      in `src/render/shader_function_tests.rs`
- [ ] Texture formats match between creation and surface config (sRGB
      consistency)
- [ ] `unwrap()` on GPU operations that could fail at runtime — prefer
      `?` or explicit handling
- [ ] No GPU resources stored on ECS components — components are pure
      authoring data
- [ ] Code adapted from elsewhere cites its source and licence;
      noncommercial or unclear licences are never copied
- [ ] Frame data holds no borrow into the ECS World, and only the
      main thread submits, presents or polls (ADR 0016)
- [ ] Whatever changes what an Instance holds makes prepare write its
      slot; other slots keep what earlier frames wrote (ADR 0003)
- [ ] Frame stats still see new work: GPU buffers and textures join
      their owner's `gpu_memory_bytes`, per-frame GPU writes add to
      `uploaded_bytes`, and systems are added inside
      `SystemTimer::scope`
- [ ] Streaming uploads go through the `StagingBelt`, never
      `Queue::write_texture`; no GPU buffer grows at runtime, and a
      request past a buffer's capacity is refused and logged. Only
      per-frame staging grows, up to the buffers a frame writes, which
      it asserts (ADRs 0003, 0017)
- [ ] The runtime loads cooked Blobs only and references Assets by
      Asset ID, never by source path (ADR 0018)
- [ ] The main pass reads no screen-space history, and no change rules
      out its 4x MSAA, which M3 turns on (ADRs 0008, 0019, 0020)
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
- [ ] Workflows keep least-privilege permissions and pinned action
      versions
- [ ] `cargo fmt` and `cargo clippy` pass

## Maintenance

This file is durable guidance. Keep it short — the more rules, the
faster they drift.

**Update this file in the same PR when:**

- Folder layout changes
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
