## Praxis — Copilot Instructions

See `CLAUDE.md` in the project root for full AI guidelines.

Key points:

- Rust edition 2024, wgpu 29, winit 0.30, glam, WGSL shaders
- Use `tracing` for logging, not `log` or `println!`
- Keep suggestions small and focused — this is a learning project
- Don't suggest adding abstractions, extra error handling, or features beyond what's being worked on
- Dependencies should be minimal with explicit feature flags
- No `vec3<f32>` in WGSL uniform structs — it pads to 16 bytes; use `vec4<f32>` or wrap with an explicit pad field
- ECS components hold resource handles (`u32` or generational), not owned `wgpu::Buffer`/`wgpu::Texture` or references
- No test framework yet — don't scaffold `#[cfg(test)]` blocks unless explicitly asked
