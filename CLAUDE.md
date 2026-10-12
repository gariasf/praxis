<!--
For AI assistants: This file describes durable conventions, not
phase-specific work. If you observe drift between this file and the
code, surface it. If you make changes that warrant a CLAUDE.md update,
update it in the same change. Don't let this file go stale.
-->

# Praxis — AI Guidelines

Praxis is a 3D engine built from scratch in Rust, aimed at a production
open-world engine (Skyrim / RDR class) on cutting-edge hardware. The
owner decides and merges; agents implement and open PRs.

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
- `ROADMAP.md` — long-range vision. Its load-bearing decisions moved to
  `docs/adr/`; its phase checklists are superseded by milestones.
- `PLAN.md` — Phase 6 scratch, paused. Superseded by the M0 tickets;
  Steps 5-6 return as tickets under a later milestone.

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
- **Graphics:** wgpu 29, WGSL shaders
- **Windowing:** winit 0.30
- **Math:** glam
- **ECS:** `bevy_ecs` (sub-crate only — no full Bevy stack)
- **Assets:** glTF via `gltf` crate, images via `image` crate,
  tangents via `bevy_mikktspace`
- **Logging:** `tracing` + `tracing-subscriber` (with `env-filter`)
- **Structure:** Single crate.

Pinned in `Cargo.toml`. Version bumps and new dependencies are owner
decisions: propose them in the ticket, justify them in the PR.

## Project shape (feature-based)

```
src/
  main.rs, state.rs            # entry + orchestrator
  shader.wgsl                  # WGSL shaders (split when justified)
  components.rs                # ECS components (Transform, MeshRef)
  camera/, time/, input/,      # feature modules: resource + system
    helmet/                       per domain in one mod.rs
  assets/                      # asset types, loaders, pools
  render/                      # renderer-internal: instance, uniforms,
                                 vertex, depth, prepare
```

Renderer **pulls** from `World` via queries (Design Principle #3 in
`ROADMAP.md`). ECS types never hold GPU buffers (Design Principle #2).

When a feature folder grows past one file, split into
`feature/{resource,system,...}.rs`. Don't pre-split.

## How agents work here

- Implement tickets end to end: code, tests, verification, PR. Bring
  the owner decisions (scope, trade-offs, dependencies) with a
  recommendation attached.
- Work the live milestone's frontier. Work that is not a ticket becomes
  a proposed ticket first.
- Review before the PR opens: `/implement` closes with a local
  `/code-review`; run it and fix what it finds. CodeRabbit's review of
  the PR is a second opinion, treated as suggestions
  (`docs/agents/workflow.md`, Review).
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
- Asset pool lookups return `Option<&T>`. Callers handle `None`
  gracefully (warn + skip in render path); never `.unwrap()` in a
  hot loop.
- Shaders are WGSL (`.wgsl` files in `src/`).
- No `unsafe` unless absolutely necessary and clearly explained.
- Keep dependencies minimal. Use feature flags
  (`image = { version = "0.25", default-features = false, features = ["png"] }`).
- Tests use the built-in harness (`cargo test`) at the seams agreed in
  the ticket. Test-framework crates are an owner decision.

## Review checklist

When reviewing changes, check for:

- [ ] GPU resources created in `State::new` are also recreated in
      `resize()`
- [ ] Shader types match Rust-side types (vertex layout, bind group
      bindings, uniforms)
- [ ] No `vec3<f32>` in WGSL uniform structs — it pads to 16 bytes and
      silently mismatches Rust-side layouts. Use `vec4<f32>` or
      explicit pad
- [ ] Rust-side uniform structs are `#[repr(C)]` with `bytemuck::Pod` +
      `Zeroable`, match WGSL layout (16-byte alignment, column-major
      matrices)
- [ ] When a buffer is recreated, the bind group referencing it is
      also recreated (bind groups are snapshots, not pointers)
- [ ] New bind groups are added to both the pipeline layout and set in
      the render pass
- [ ] Texture formats match between creation and surface config (sRGB
      consistency)
- [ ] `unwrap()` on GPU operations that could fail at runtime — prefer
      `?` or explicit handling
- [ ] No GPU resources stored on ECS components — components are pure
      authoring data
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

Two agents read this file: Claude Code (interactive) and Code Rabbit
(PR review, configured via `.coderabbit.yaml`). No `AGENTS.md`,
`.cursor/rules/`, `.junie/guidelines.md`, or similar — the project
doesn't use those tools. If a new agent is onboarded later, extract
shared content into `AGENTS.md` and have `CLAUDE.md` carry only the
Claude-specific addenda. Don't pre-create.
