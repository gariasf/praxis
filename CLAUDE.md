<!--
For AI assistants: This file describes durable conventions, not
phase-specific work. If you observe drift between this file and the
code, surface it. If you make changes that warrant a CLAUDE.md update,
update it in the same change. Don't let this file go stale.
-->

# Praxis — AI Guidelines

Praxis is a personal, learning-focused 3D rendering engine built from
scratch. Solo developer. The goal is to understand every line — not to
ship fast.

## Project posture

**Learning sets the pace, not the bar.**

Long-term destination: a production cutting-edge engine, foundations
for a Skyrim-shape RPG on modern hardware. Today's small, unoptimised
implementation reflects the early phase, not the eventual scope. See
`ROADMAP.md` for the phase plan and load-bearing decisions.

What this means in practice:

- **Architectural decisions are production-grade.** Load-bearing
  decisions in `ROADMAP.md` (ECS shape, asset pools, renderer pull,
  GPU resource ownership) are made for the eventual scope, not the
  current phase. Don't suggest shortcuts that would need rewriting at
  scale.
- **Implementation depth tracks the current phase.** Use the simplest
  version of the right shape. Complexity arrives on schedule, not
  speculation.
- **No throwaway code.** "We'll fix it later" is rejected. Clean
  naming, structure, conventions every step. If it's worth building,
  build it right within the phase's scope.
- **Performance optimisation waits its turn.** Profiling, batching,
  GPU-driven rendering — all real, all scheduled, none premature.

"Learning-focused" describes how the project is built (one person,
weekends, understanding every line). It does not describe what the
project is *for*.

## Documents

Read in this order before assisting:

- `ROADMAP.md` — phase plan, design principles, load-bearing decisions.
  Authoritative on **what** to build and **when**.
- `PLAN.md` — current phase scratch. Replaced each phase. Not durable.
- `CLAUDE.md` (this file) — **how** to work in this codebase.

If any of these conflicts with current code, **the code is the source
of truth**. Flag the drift to the user explicitly; do not silently
follow stale guidance.

## Stack (frozen)

- **Language:** Rust (edition 2024)
- **Graphics:** wgpu 29, WGSL shaders
- **Windowing:** winit 0.30
- **Math:** glam
- **ECS:** `bevy_ecs` (sub-crate only — no full Bevy stack)
- **Assets:** glTF via `gltf` crate, images via `image` crate
- **Logging:** `tracing` + `tracing-subscriber` (with `env-filter`)
- **Structure:** Single crate.

Pinned in `Cargo.toml`. Do not bump versions or add deps without
discussion.

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

## How to assist

- **Guide, explain, review.** Don't generate entire systems or scaffold
  modules. Write small, focused snippets when asked.
- Prefer teaching the concept over writing the code.
- Stay on the current phase. Don't suggest features from phases ahead.
- A bug-fix request is not a refactor request. A feature request is not
  a cleanup request. Do only what was asked.
- Don't add comments, docstrings, or type annotations to code you
  didn't change.
- If asked to review: flag correctness and safety issues, not style
  preferences. `cargo fmt` and `clippy` own style.

## Code conventions

- `cargo fmt` and `cargo clippy` must pass.
- Use `tracing::info!` / `error!` / etc. — never `println!` or `log`.
- Prefer structured fields:
  `tracing::info!(field = value, "message")` over `format!`-style
  messages.
- See `scripts/run-debug.{ps1,sh}` and `scripts/run-trace.{ps1,sh}` for
  log-filter recipes. Project default is
  `praxis=info,wgpu=warn,naga=warn` via `.cargo/config.toml`.
- Bevy imports: `use bevy_ecs::prelude::*;` (not piecemeal).
- WGPU types: full path (`wgpu::Buffer`), don't import individual types.
- Shaders are WGSL (`.wgsl` files in `src/`).
- No `unsafe` unless absolutely necessary and clearly explained.
- Keep dependencies minimal. Use feature flags
  (`image = { version = "0.25", default-features = false, features = ["png"] }`).
- No test framework yet. Verify changes by running the app — don't add
  `#[cfg(test)]` scaffolding unless explicitly asked.

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

- Folder layout changes (e.g., the Phase 5 feature-folder migration)
- A new cross-cutting convention is adopted
- A "load-bearing decision" in `ROADMAP.md` is added or revised
- A repeated correction emerges — same mistake twice means it's a rule

**Don't update for:**

- Phase-specific work (lives in `PLAN.md`)
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
