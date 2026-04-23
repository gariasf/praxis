# Praxis — AI Guidelines

## Project

Praxis is a personal, learning-focused 3D rendering engine built from scratch.
Solo developer. The goal is to understand every line — not to ship fast.

- **Language:** Rust (edition 2024)
- **Graphics:** wgpu 29, WGSL shaders
- **Windowing:** winit 0.30
- **Math:** glam
- **Assets:** glTF via `gltf` crate, images via `image` crate
- **Logging:** tracing + tracing-subscriber
- **Structure:** Single crate for now. Split when a module justifies it.

See `ROADMAP.md` for current progress and next steps.

## How to assist

- **Guide, explain, review.** Don't generate entire systems or scaffold modules.
- Write small, focused snippets when asked — not full implementations.
- Prefer teaching the concept over writing the code.
- If asked to review: flag correctness and safety issues, not style preferences.
- Don't add features, abstractions, or "improvements" beyond what was asked.
- Don't add comments, docstrings, or type annotations to code you didn't change.
- Follow `ROADMAP.md`'s Design Principles when suggesting architecture or
  refactors. Flag any proposed code that holds GPU buffers inside ECS
  components or couples ECS types to wgpu byte layouts.
- Stay on the current phase. Don't suggest features from phases ahead.
- A bug-fix request is not a refactor request. A feature request is not a
  cleanup request. Do only what was asked.

## Code conventions

- `cargo fmt` and `cargo clippy` must pass.
- Use `tracing::info!` / `tracing::error!` etc. for logging. Not `log`, not `println!`.
- Shaders are WGSL (`.wgsl` files in `src/`).
- No `unsafe` unless absolutely necessary and clearly explained.
- Keep dependencies minimal. Use feature flags to avoid pulling in the world
  (e.g. `image = { version = "0.25", default-features = false, features = ["png"] }`).
- No test framework yet. Verify changes by running the app — don't add
  `#[cfg(test)]` scaffolding unless explicitly asked.
- `PLAN.md` is phase-scratch: uncommitted, replaced each phase. Don't treat
  it as durable documentation.

## Review checklist

When reviewing PRs or changes, check for:

- [ ] GPU resources created in `State::new` are also recreated in `resize()`
- [ ] Shader types match Rust-side types (vertex layout, bind group bindings, uniforms)
- [ ] `unwrap()` on GPU operations that could fail at runtime — prefer `?` or explicit handling
- [ ] New bind groups are added to both the pipeline layout and set in the render pass
- [ ] Texture formats match between creation and surface config (sRGB consistency)
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] No `vec3<f32>` in WGSL uniform structs — it pads to 16 bytes and silently
      mismatches Rust-side layouts. Use `vec4<f32>` or add an explicit pad field.
- [ ] Rust-side uniform structs are `#[repr(C)]` with `bytemuck::Pod` + `Zeroable`,
      and match WGSL layout (16-byte alignment, column-major matrices).
