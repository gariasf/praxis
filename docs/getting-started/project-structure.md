# Project Structure

Praxis uses a Cargo workspace with 12 crates organized by subsystem.

## Workspace Layout

```
praxis/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── praxis_core/    # Engine lifecycle, main loop
│   ├── praxis_window/  # Window management (winit)
│   ├── praxis_graphics/# Vulkan rendering (vulkano)
│   ├── praxis_ecs/     # Entity-Component-System (bevy_ecs)
│   ├── praxis_math/    # Math utilities (glam)
│   ├── praxis_scene/   # Scene graph, transforms
│   ├── praxis_assets/  # Asset loading (OBJ, GLTF)
│   ├── praxis_input/   # Input handling
│   ├── praxis_gui/     # Debug GUI (egui)
│   ├── praxis_physics/ # Physics (Rapier3D)
│   ├── praxis_audio/   # Audio (Kira)
│   ├── praxis_editor/  # Editor tools
│   └── praxis_utils/   # Shared utilities
├── examples/           # Runnable demos
├── assets/             # Textures, models, sounds
└── docs/               # Documentation
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

## Key Crates

### praxis_core
Engine entry point. Coordinates initialization and main loop.

### praxis_graphics
Vulkan rendering via `vulkano`. Manages pipelines, meshes, textures.

### praxis_ecs
Re-exports `bevy_ecs`. Provides components and system scheduling.

### praxis_scene
Transform hierarchy, scene serialization, animation.

## Running Examples

```bash
# List all examples
ls examples/

# Run specific example
cargo run --example deferred_demo
```

## See Also

- [Architecture](../ARCHITECTURE.md) - Detailed design decisions
- [Beginners Guide](../BEGINNERS_GUIDE.md) - Hands-on learning
