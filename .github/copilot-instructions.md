## Praxis — Copilot Instructions

See `CLAUDE.md` in the project root for full AI guidelines.

Key points:

- Rust edition 2024, wgpu 29, winit 0.30, glam, WGSL shaders
- Use `tracing` for logging, not `log` or `println!`
- Keep suggestions small and focused — this is a learning project
- Don't suggest adding abstractions, extra error handling, or features beyond what's being worked on
- wgpu 29: `depth_write_enabled` and `depth_compare` are `Option` types
- Dependencies should be minimal with explicit feature flags
