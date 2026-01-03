# Praxis Engine - Rust Architecture

## Core Principles

- **Language:** Rust (latest stable version).
- **Safety & Concurrency:** Leverage Rust's ownership, borrowing, and type system for memory safety and concurrency.
- **Modularity:** Design the engine as a collection of loosely coupled crates within a Cargo workspace.
- **Pragmatism & Iteration:** Focus on building a usable engine for game creation. Develop features iteratively based on practical game development needs, aiming for usability within 1-2 years.
- **Performance:** Optimize critical paths, leveraging Rust's performance characteristics and libraries like `glam` (SIMD).
- **Tooling:** Utilize the standard Rust toolchain: `cargo`, `rustfmt`, `clippy`, `rustdoc`.
- **Error Handling:** Employ `Result<T, E>` for recoverable errors, potentially using crates like `thiserror` and `color-eyre` for better ergonomics and reporting. Panics should be reserved for unrecoverable states (bugs).

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
│   ├── praxis_gui/
│   ├── praxis_audio/
│   ├── praxis_physics/
│   ├── praxis_scripting/
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

- **`praxis_core`:**
  - Engine lifecycle management (startup, shutdown, main loop).
  - Configuration loading and management.
  - Time management (delta time, timers).
  - Core traits and types used across the engine.
- **`praxis_window`:**
  - Window creation and management.
  - Event loop integration.
  - Backend: `winit` (pure Rust, for tight integration with graphics APIs).
  - Depends on: `praxis_core`.
- **`praxis_input`:**
  - Handling keyboard, mouse, gamepad input.
  - Mapping raw input to game actions.
  - Backend tied to `winit`.
  - Depends on: `praxis_window`, `praxis_core`.
- **`praxis_graphics`:**
  - Rendering abstraction layer. Defines traits for renderers, resources (textures, meshes, shaders), pipelines.
  - Backend: `vulkano` (abstraction over Vulkan).
  - Depends on: `praxis_core`, `praxis_window`, `praxis_math`.
- **`praxis_math`:**
  - Provides core mathematical types and operations for graphics and physics.
  - Re-exports types from `glam` (Vector, Matrix, Quaternion).
  - May include engine-specific math utilities.
- **`praxis_ecs`:**
  - Entity-Component-System implementation.
  - Dependency: `bevy_ecs` (known for performance and ergonomics).
  - Manages entities, components, and systems.
  - Depends on: `praxis_core`.
- **`praxis_scene`:**
  - Scene representation and management.
  - Scene graph for spatial organization.
  - Camera management.
  - Skeletal animation system with bone hierarchies.
  - Animation blending (cross-fade, blend trees, layered animation).
  - Transform propagation for both regular and animated entities.
  - Integrates with `praxis_ecs`.
  - Depends on: `praxis_ecs`, `praxis_core`, `praxis_math`.
- **`praxis_assets`:**
  - Asset loading, management, and caching.
  - Supports various formats:
    - Textures: `image` crate (PNG, JPEG).
    - Models: `gltf` (GLTF 2.0/GLB with PBR materials, animations), `tobj` (OBJ).
    - Audio: OGG, MP3, WAV, FLAC via `kira`.
    - Configuration/Data: `serde` (with `serde_json`, `serde_yaml`, etc.).
  - GLTF loader includes material, texture, animation, and node hierarchy support.
  - Asset caching via `GltfAssetManager` and `ObjAssetManager`.
  - Depends on: `praxis_core`, `praxis_utils`.
- **`praxis_audio`:**
  - Audio playback and management using Kira.
  - 3D spatial audio with distance attenuation and doppler effect.
  - Audio source components for ECS integration.
  - Listener positioning (typically follows camera).
  - Volume, pitch, and playback controls.
  - Multiple audio format support (OGG, MP3, WAV, FLAC).
  - Depends on: `praxis_core`, `praxis_math`, `praxis_ecs`.
- **`praxis_gui`:**
  - Debugging and editor tools GUI.
  - Dependency: `egui`.
  - Depends on: `praxis_core`, `praxis_graphics`, `praxis_window`.
- **`praxis_scripting`:**
  - Manages Lua VMs and provides bindings for scripting game logic.
  - Handles loading/execution of Lua scripts (via `praxis_assets`).
  - Dependency: `mlua`.
  - Integrates with `praxis_ecs` for script components and entity manipulation.
  - Exposes other engine functionalities (input, scene, math, etc.) to Lua.
  - Depends on: `praxis_core`, `praxis_ecs`, `praxis_assets`, `praxis_input`, `praxis_scene`, `praxis_math`.
- **`praxis_utils`:**
  - Common utilities shared across crates.
  - Logging: `tracing` framework with subscribers like `tracing-subscriber`.
  - Error handling helpers: `color-eyre` or similar.
  - Filesystem utilities.
  - Depends on: `praxis_core` (for common error types, if any).

## Build System & Dependencies

- **Build System:** `cargo`.
- **Dependency Management:** `Cargo.toml` for each crate and the workspace root.
- **Key Dependencies:**
  - Windowing/Input: `winit`, `gilrs`
  - Graphics: `vulkano`, `vulkano-shaders`
  - Math: `glam`
  - ECS: `bevy_ecs`
  - Physics: `rapier3d`
  - Audio: `kira`
  - Assets:
    - Textures: `image`
    - Models: `gltf` (GLTF 2.0/GLB with animations), `tobj` (OBJ)
    - Audio: `kira` (OGG, MP3, WAV, FLAC)
    - Data: `serde`
  - GUI: `egui`, `egui-winit`, `egui_vulkano`
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
