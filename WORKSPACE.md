# Praxis Workspace Structure

This document describes the workspace structure and organization of the Praxis game engine.

## Workspace Overview

Praxis is organized as a Cargo workspace with 19 crates, each focused on a specific subsystem of the engine.

## Workspace Configuration

### Lints

All crates in the workspace inherit the following lint configuration:

```toml
[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"

[workspace.lints.rust]
unsafe_code = "warn"
missing_docs = "warn"
```

To apply workspace lints in a crate's `Cargo.toml`:

```toml
[lints]
workspace = true
```

### Feature Flags

The root package defines these feature flags:

- `default`: No features enabled by default
- `editor`: Enables the editor subsystem (`praxis_editor`)
- `networking`: Enables networking and multiplayer (`praxis_networking`)
- `scripting`: Enables Lua scripting (`praxis_scripting` and GUI scripting support)
- `terrain`: Enables terrain generation and rendering (`praxis_terrain`)
- `headless`: For CI/testing without GPU

## Crate Structure

### Core Engine

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_core` | `crates/praxis_core` | Engine lifecycle, main loop |
| `praxis_utils` | `crates/praxis_utils` | Logging, errors, timing |
| `praxis_window` | `crates/praxis_window` | Window management (winit) |

### Mathematics & Data Structures

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_math` | `crates/praxis_math` | Math library (glam wrapper) |
| `praxis_spatial` | `crates/praxis_spatial` | Octrees, BVH, spatial queries |

### ECS & Scene

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_ecs` | `crates/praxis_ecs` | ECS (bevy_ecs wrapper) |
| `praxis_scene` | `crates/praxis_scene` | Transform hierarchy, animation |

### Rendering

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_graphics` | `crates/praxis_graphics` | Vulkan rendering |
| `praxis_gui` | `crates/praxis_gui` | ImGui (egui) |
| `praxis_procedural` | `crates/praxis_procedural` | Procedural textures |
| `praxis_terrain` | `crates/praxis_terrain` | Terrain generation & LOD |

### Assets & Resources

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_assets` | `crates/praxis_assets` | OBJ/GLTF loading |

### Input & Interaction

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_input` | `crates/praxis_input` | Keyboard, mouse, gamepad |
| `praxis_editor` | `crates/praxis_editor` | Editor tools |

### Simulation

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_physics` | `crates/praxis_physics` | Physics (Rapier3D) |
| `praxis_audio` | `crates/praxis_audio` | Audio (Kira) |

### Optional Features

| Crate | Path | Description | Feature Flag |
|-------|------|-------------|--------------|
| `praxis_scripting` | `crates/praxis_scripting` | Lua integration | `scripting` |
| `praxis_networking` | `crates/praxis_networking` | Multiplayer | `networking` |

### Development Tools

| Crate | Path | Description |
|-------|------|-------------|
| `praxis_profiling` | `crates/praxis_profiling` | Performance profiling |

## Adding a New Crate

1. Create directory: `crates/your_crate_name`
2. Add `Cargo.toml`:
```toml
[package]
name = "your_crate_name"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Brief description"

[lints]
workspace = true

[dependencies]
# Add dependencies here
```

3. Create `src/lib.rs`:
```rust
//! Crate documentation.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
```

4. Add to workspace in root `Cargo.toml`:
```toml
[workspace]
members = [
    # ... existing crates ...
    "crates/your_crate_name",
]
```

## Dependency Guidelines

### Internal Dependencies

- Use relative paths: `praxis_utils = { path = "../praxis_utils", version = "0.1.0" }`
- Always specify version for forward compatibility
- Avoid circular dependencies

### External Dependencies

- Prefer widely-used, battle-tested libraries
- Specify versions explicitly
- Use features to minimize dependency footprint
- Document why each dependency is needed

### Common External Dependencies

- **Math**: `glam` (SIMD-optimized)
- **ECS**: `bevy_ecs` (composition over inheritance)
- **Graphics**: `vulkano` (safe Vulkan)
- **Physics**: `rapier3d` (high-performance)
- **Audio**: `kira` (easy-to-use)
- **GUI**: `egui` (immediate-mode)
- **Errors**: `color-eyre` (beautiful errors)
- **Logging**: `tracing` (structured logging)

## CI Configuration

The workspace uses GitHub Actions for CI (`.github/workflows/rust-ci.yml`):

### Jobs

1. **check**: `cargo check --all --all-features` and `--no-default-features`
2. **fmt**: `cargo fmt --all -- --check`
3. **clippy**: `cargo clippy --all --all-features -- -D warnings`
4. **test**: `cargo test --workspace --all-features`
5. **build_examples**: `cargo build --examples --features headless`

### Running Locally

```bash
# Check all crates
cargo check --all --all-features

# Format check
cargo fmt --all -- --check

# Lint (must pass with no warnings)
cargo clippy --all --all-features -- -D warnings

# Run tests
cargo test --workspace --all-features
```

## Build Profiles

### Development

```toml
[profile.dev]
opt-level = 1  # Faster debug builds
```

### Release

```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
```

## Best Practices

### Code Organization

- Keep crates focused on a single responsibility
- Use modules to organize code within crates
- Export public APIs through `lib.rs`
- Document all public items

### Documentation

- All public items must have rustdoc comments
- Use `//!` for module-level docs
- Use `///` for item-level docs
- Include examples in docs where helpful

### Testing

- Write unit tests in the same file as code
- Put integration tests in `tests/` directory
- Use doc tests for examples
- Run tests with `cargo test --workspace`

### Feature Flags

- Make optional features truly optional
- Document feature requirements
- Use `?` for weak dependencies: `praxis_editor?/terrain`

## Troubleshooting

### Build Errors

```bash
# Clean build
cargo clean

# Update dependencies
cargo update

# Check specific crate
cargo check -p praxis_graphics
```

### Lint Issues

```bash
# Auto-fix formatting
cargo fmt --all

# See all clippy suggestions
cargo clippy --all --all-features
```

### Dependency Conflicts

```bash
# Show dependency tree
cargo tree

# Show duplicate dependencies
cargo tree -d
```

## Further Reading

- [CLAUDE.md](CLAUDE.md) - Development guidelines
- [docs/architecture.md](docs/architecture.md) - Architecture details
- [docs/reference/crates.md](docs/reference/crates.md) - Crate reference
