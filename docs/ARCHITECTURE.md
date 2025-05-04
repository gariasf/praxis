# Praxis Engine - Rust Architecture

## Core Principles
- **Language:** Rust (latest stable version).
- **Safety & Concurrency:** Leverage Rust's ownership, borrowing, and type system for memory safety and concurrency.
- **Modularity:** Design the engine as a collection of loosely coupled crates within a Cargo workspace.
- **Pragmatism & Iteration:** Focus on building a usable engine for game creation. Develop features iteratively based on practical game development needs, aiming for usability within 1-2 years.
- **Performance:** Optimize critical paths, leveraging Rust's performance characteristics and libraries like `glam` (SIMD).
- **Tooling:** Utilize the standard Rust toolchain: `cargo`, `rustfmt`, `clippy`, `rustdoc`.
- **Error Handling:** Employ `Result<T, E>` for recoverable errors, potentially using crates like `thiserror` and `color-eyre` for better ergonomics and reporting. Panics should be reserved for unrecoverable states (bugs).
- **Cross-Platform:** Target Windows, Linux, and macOS as primary platforms.

## Project Structure (Cargo Workspace)
The engine will be structured as a Cargo workspace, located at the root of the repository.

```
praxis/
├── Cargo.toml          # Workspace definition
├── crates/             # Engine modules (crates)
│   ├── praxis_core/
│   ├── praxis_window/
│   ├── praxis_input/
│   ├── praxis_graphics/
│   ├── praxis_ecs/
│   ├── praxis_scene/
│   ├── praxis_assets/
│   ├── praxis_math/
│   ├── praxis_audio/
│   ├── praxis_physics/
│   ├── praxis_gui/
│   └── praxis_utils/
├── examples/           # Example applications using the engine
├── tests/              # Integration tests
├── benches/            # Benchmarks (using criterion)
├── assets/             # Shared assets for examples/tests
├── docs/               # Documentation
└── .gitignore
└── README.md
```

## Core Crates (Modules)

- **`praxis`:**
  - Engine lifecycle management (startup, shutdown, main loop).
  - Configuration loading and management.
  - Time management (delta time, timers).
  - Core traits and types used across the engine.
- **`praxis_window`:**
  - Window creation and management.
  - Event loop integration.
  - Potential backend: `winit` (pure Rust). `winit` is often preferred for tighter integration with graphics APIs like Vulkan.
- **`praxis_input`:**
  - Handling keyboard, mouse, gamepad input.
  - Mapping raw input to game actions.
  - Backend tied to the chosen windowing library (`winit`).
- **`praxis_graphics`:**
  - Rendering abstraction layer. Defines traits for renderers, resources (textures, meshes, shaders), pipelines.
  - Proposed dependency: `wgpu`. `wgpu` provides an abstraction over native graphics APIs (Vulkan, Metal, DirectX, OpenGL) allowing for cross-platform rendering.
- **`praxis_math`:**
  - Provides core mathematical types and operations for graphics and physics.
  - Likely re-exports types from `glam` (Vector, Matrix, Quaternion).
  - May include engine-specific math utilities.
- **`praxis_ecs`:**
  - Entity-Component-System implementation.
  - Proposed dependency: `bevy_ecs` (known for performance and ergonomics).
  - Manages entities, components, and systems.
- **`praxis_scene`:**
  - Scene representation and management.
  - Scene graph for spatial organization.
  - Camera management.
  - Integrates with `praxis_ecs`.
- **`praxis_assets`:**
  - Asset loading, management, and caching.
  - Supports various formats:
    - Textures: `image` crate.
    - Models: `russimp` (Assimp bindings) or `gltf`.
    - Configuration/Data: `serde` (with `serde_json`, `serde_yaml`, etc.).
  - Asynchronous loading capabilities.
- **`praxis_audio`:**
  - Audio playback and spatialization.
  - Proposed dependency: `rodio` (pure Rust) or `kira`.
- **`praxis_physics`:**
  - Physics simulation (collision detection, rigid body dynamics).
  - Proposed dependency: `rapier` (pure Rust 2D/3D physics engine).
- **`praxis_gui`:**
  - Debugging and editor tools GUI.
  - Proposed dependency: `imgui-rs` with appropriate backend (`imgui-winit-support`, `imgui-ash-renderer`).
- **`praxis_utils`:**
  - Common utilities shared across crates.
  - Logging: `tracing` framework with subscribers like `tracing-subscriber`.
  - Error handling helpers: `color-eyre` or similar.
  - Filesystem utilities.

## Build System & Dependencies
- **Build System:** `cargo`.
- **Dependency Management:** `Cargo.toml` for each crate and the workspace root.
- **Key Proposed Dependencies:**
  - Windowing/Input: `winit`
  - Graphics (Vulkan): `vulkano`, `vulkano-shaders`
  - Math: `glam`
  - ECS: `bevy_ecs`
  - Assets: `image`, `russimp`/`gltf`, `serde`
  - Audio: `rodio` or `kira`
  - Physics: `rapier`
  - GUI: `imgui-rs` + backends
  - Logging: `tracing`, `tracing-subscriber`
  - Error Reporting: `color-eyre`
- **Dependency Versions:** Use latest stable versions where compatible and reasonable. Pin versions in `Cargo.lock` for reproducible builds.

## Testing
- **Unit Tests:** Written within each crate's source files (`#[cfg(test)] mod tests { ... }`).
- **Integration Tests:** Located in the workspace `tests/` directory.
- **Benchmarking:** Located in the workspace `benches/` directory, using `criterion`.
- **Coverage:** Aim for high test coverage, potentially using tools like `cargo-tarpaulin`.
- **CI:** Enforce tests (`cargo test --workspace`), formatting (`cargo fmt --check`), and lints (`cargo clippy --workspace -- -D warnings`).

## Documentation
- **API Documentation:** Use `rustdoc` comments (`///` and `//!`) for all public items (structs, enums, functions, traits, modules). Generate documentation using `cargo doc`.
- **Architectural Documentation:** Maintain high-level design documents in `docs/`, including this file.
- **Examples:** Provide clear, concise examples in the `examples/` directory demonstrating engine features.

## Coding Style & Idioms
- Follow standard Rust API guidelines and idioms.
- Enforce formatting using `rustfmt` (default settings are generally preferred).
- Enforce code quality and catch common mistakes using `clippy`.
