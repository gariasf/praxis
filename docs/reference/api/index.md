# API Reference

Complete API documentation for all Praxis crates. This page serves as an index to detailed rustdoc documentation.

## Core Crates

### praxis_core
**Engine lifecycle and main loop**

```bash
cargo doc --package praxis_core --open
```

[View Documentation →](https://docs.rs/praxis_core)

### praxis_ecs
**Entity-Component-System using bevy_ecs**

```bash
cargo doc --package praxis_ecs --open
```

[View Documentation →](https://docs.rs/praxis_ecs)

### praxis_math
**Mathematics library (glam wrapper)**

```bash
cargo doc --package praxis_math --open
```

[View Documentation →](https://docs.rs/praxis_math)

## Graphics

### praxis_graphics
**Vulkan rendering and graphics pipeline**

```bash
cargo doc --package praxis_graphics --open
```

[View Documentation →](https://docs.rs/praxis_graphics)

### praxis_window
**Window management (winit)**

```bash
cargo doc --package praxis_window --open
```

[View Documentation →](https://docs.rs/praxis_window)

## Scene & Animation

### praxis_scene
**Transform hierarchy and animation**

```bash
cargo doc --package praxis_scene --open
```

[View Documentation →](https://docs.rs/praxis_scene)

### praxis_spatial
**Spatial data structures (octree, BVH)**

```bash
cargo doc --package praxis_spatial --open
```

[View Documentation →](https://docs.rs/praxis_spatial)

## Systems

### praxis_physics
**Physics simulation (Rapier3D)**

```bash
cargo doc --package praxis_physics --open
```

[View Documentation →](https://docs.rs/praxis_physics)

### praxis_audio
**Audio playback (Kira)**

```bash
cargo doc --package praxis_audio --open
```

[View Documentation →](https://docs.rs/praxis_audio)

### praxis_input
**Input handling (keyboard, mouse, gamepad)**

```bash
cargo doc --package praxis_input --open
```

[View Documentation →](https://docs.rs/praxis_input)

### praxis_gui
**Editor GUI (egui)**

```bash
cargo doc --package praxis_gui --open
```

[View Documentation →](https://docs.rs/praxis_gui)

### praxis_scripting
**Lua scripting integration**

```bash
cargo doc --package praxis_scripting --open
```

[View Documentation →](https://docs.rs/praxis_scripting)

### praxis_networking
**Multiplayer networking**

```bash
cargo doc --package praxis_networking --open
```

[View Documentation →](https://docs.rs/praxis_networking)

## Assets

### praxis_assets
**Asset loading (OBJ, GLTF)**

```bash
cargo doc --package praxis_assets --open
```

[View Documentation →](https://docs.rs/praxis_assets)

### praxis_procedural
**Procedural texture generation**

```bash
cargo doc --package praxis_procedural --open
```

[View Documentation →](https://docs.rs/praxis_procedural)

## Specialized

### praxis_terrain
**Terrain generation and LOD**

```bash
cargo doc --package praxis_terrain --open
```

[View Documentation →](https://docs.rs/praxis_terrain)

### praxis_profiling
**Performance profiling**

```bash
cargo doc --package praxis_profiling --open
```

[View Documentation →](https://docs.rs/praxis_profiling)

### praxis_editor
**Editor tools and commands**

```bash
cargo doc --package praxis_editor --open
```

[View Documentation →](https://docs.rs/praxis_editor)

### praxis_utils
**Logging, errors, and utilities**

```bash
cargo doc --package praxis_utils --open
```

[View Documentation →](https://docs.rs/praxis_utils)

## Building All Documentation

Generate documentation for the entire workspace:

```bash
cargo doc --workspace --no-deps --open
```

## Documentation Features

### Code Examples
All public APIs include usage examples:

```rust
/// Spawns an entity with components.
///
/// # Examples
///
/// ```
/// let entity = world.spawn((
///     Transform::default(),
///     Velocity::default(),
/// ));
/// ```
pub fn spawn(&mut self, components: impl Bundle) -> Entity { ... }
```

### Cross-References
Documentation links to related types and traits automatically.

### Search
Use the search bar in generated docs to find types and functions.

## External Dependencies

Key external crates used by Praxis:

- [bevy_ecs](https://docs.rs/bevy_ecs) - Entity-Component-System
- [vulkano](https://docs.rs/vulkano) - Vulkan bindings
- [rapier3d](https://docs.rs/rapier3d) - Physics engine
- [glam](https://docs.rs/glam) - Math library
- [kira](https://docs.rs/kira) - Audio engine
- [egui](https://docs.rs/egui) - Immediate-mode GUI

## Contributing to Documentation

### Writing Rustdoc Comments

```rust
/// Brief description (one line).
///
/// Detailed explanation with multiple paragraphs.
///
/// # Examples
///
/// ```
/// // Example code here
/// ```
///
/// # Panics
///
/// When this function panics and why.
///
/// # Safety
///
/// For unsafe functions, explain safety requirements.
pub fn example() { ... }
```

### Documentation Standards

- ✅ All public items must have documentation
- ✅ Include at least one example per function
- ✅ Explain parameters and return values
- ✅ Document panics and errors
- ✅ Link to related types

### Building Locally

```bash
# Build docs
cargo doc --no-deps

# Build and open
cargo doc --no-deps --open

# Build with private items
cargo doc --document-private-items
```

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="../components.html" class="md-button">Component Reference</a>
  <a href="../crates.html" class="md-button">Crate Overview</a>
</div>
