# Project Structure

Praxis uses a Cargo workspace with 19 crates organized by subsystem. Most crates are always available, while some require [feature flags](feature-flags.md) to be enabled.

## Workspace Layout

```
praxis/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── praxis_core/        # Engine lifecycle, main loop
│   ├── praxis_window/      # Window management (winit)
│   ├── praxis_graphics/    # Vulkan rendering (vulkano)
│   ├── praxis_ecs/         # Entity-Component-System (bevy_ecs)
│   ├── praxis_math/        # Math utilities (glam)
│   ├── praxis_scene/       # Scene graph, transforms, animation
│   ├── praxis_assets/      # Asset loading (OBJ, GLTF)
│   ├── praxis_input/       # Input handling
│   ├── praxis_gui/         # Debug GUI (egui)
│   ├── praxis_physics/     # Physics (Rapier3D)
│   ├── praxis_audio/       # Audio (Kira)
│   ├── praxis_spatial/     # Octree, BVH, frustum culling
│   ├── praxis_procedural/  # Procedural texture generation
│   ├── praxis_profiling/   # Performance monitoring
│   ├── praxis_utils/       # Shared utilities
│   ├── praxis_editor/      # Editor tools (optional: requires `editor` feature)
│   ├── praxis_scripting/   # Lua scripting (optional: requires `scripting` feature)
│   ├── praxis_networking/  # Multiplayer (optional: requires `networking` feature)
│   └── praxis_terrain/     # Terrain system (optional: requires `terrain` feature)
├── examples/               # Runnable demos
├── assets/                 # Textures, models, sounds
└── docs/                   # Documentation
```

## Crate Dependencies

```
praxis (root)
├── praxis_core         # Entry point
│   ├── praxis_window   # Creates window
│   ├── praxis_graphics # Rendering
│   ├── praxis_ecs      # World management
│   ├── praxis_input    # Input processing
│   └── praxis_utils    # Logging, errors
├── praxis_scene        # Uses ECS, Math
├── praxis_assets       # File loading
├── praxis_physics      # Rapier integration
├── praxis_audio        # Kira integration
└── praxis_editor       # Development tools
```

## Core Crates (Always Available)

These crates are part of the [default build](core-features.md):

### praxis_core
Engine entry point. Coordinates initialization and main loop.

### praxis_graphics
Vulkan rendering via `vulkano`. Manages pipelines, meshes, textures, particle systems, LOD.

### praxis_ecs
Re-exports `bevy_ecs`. Provides components and system scheduling.

### praxis_scene
Transform hierarchy, scene serialization, skeletal animation.

### praxis_physics
3D physics simulation via Rapier3D. Rigid bodies, colliders, constraints.

### praxis_audio
Spatial audio playback via Kira. 3D positioning and sound effects.

### praxis_assets
Asset loading for OBJ, glTF, textures, and animations.

### praxis_input
Keyboard, mouse, and gamepad handling across platforms.

### praxis_gui
Immediate-mode GUI via egui. Inspector panels and debug overlays.

### praxis_spatial
Spatial acceleration structures: octree, BVH, frustum culling.

### praxis_procedural
GPU-based procedural texture generation with noise functions and texture graphs.

### praxis_profiling
CPU and GPU performance monitoring.

### praxis_math
Math types and utilities via glam.

### praxis_window
Window creation and management via winit.

### praxis_utils
Logging, error handling, timing utilities.

## Optional Crates (Feature Flags Required)

These crates require [feature flags](feature-flags.md) to be enabled:

### praxis_editor (requires `editor` feature)
Editor tools: selection, undo/redo, gizmos, hierarchy panel, inspector.

### praxis_scripting (requires `scripting` feature)
Lua scripting integration with ECS access and hot reload.

### praxis_networking (requires `networking` feature)
Multiplayer networking with entity replication and lag compensation.

### praxis_terrain (requires `terrain` feature)
Heightmap-based terrain with LOD and procedural generation.

## Running Examples

```bash
# Core feature examples (no flags needed)
cargo run --example comprehensive_scene_demo
cargo run --example animation_blending_demo

# Optional feature examples (flags required)
cargo run --features editor --example editor_demo
cargo run --features scripting --example scripting_demo
cargo run --features networking --example networking_demo
cargo run --features terrain --example terrain_demo
```

See [Feature Flags](feature-flags.md) for more examples.

## See Also

- [Core Features](core-features.md) - What's included by default
- [Feature Flags](feature-flags.md) - Enable optional systems
- [Architecture](../architecture.md) - Detailed design decisions
- [Beginners Guide](../beginners-guide.md) - Hands-on learning
