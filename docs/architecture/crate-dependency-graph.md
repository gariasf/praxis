# Crate Dependency Graph

This document provides a visual representation of the dependencies between Praxis engine crates, showing how the workspace is organized into layers and the data flow between subsystems.

## Overview

The Praxis engine consists of 18 crates organized into four architectural layers:

1. **Foundation Layer**: Core utilities, math, and logging
2. **Platform Layer**: Window management and input handling
3. **Engine Layer**: Graphics, ECS, physics, audio, and assets
4. **Application Layer**: Editor, GUI, and specialized systems

## Dependency Graph

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          APPLICATION LAYER                                    │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────┐           ┌─────────────────┐                          │
│  │ praxis_editor   │◄──────────│ praxis_terrain  │                          │
│  │                 │           │                 │                          │
│  │ - Selection     │           │ - Heightmaps    │                          │
│  │ - Undo/Redo     │           │ - LOD System    │                          │
│  │ - Gizmos        │           │ - Generation    │                          │
│  │ - Commands      │           └────────┬────────┘                          │
│  └────────┬────────┘                    │                                    │
│           │                             │                                    │
└───────────┼─────────────────────────────┼────────────────────────────────────┘
            │                             │
            │         ┌───────────────────┘
            │         │
            ▼         ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                            ENGINE LAYER                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐  │
│  │  praxis_gui     │        │ praxis_scripting│        │ praxis_network  │  │
│  │                 │        │                 │        │                 │  │
│  │ - egui         │        │ - Lua Runtime   │        │ - TCP/UDP       │  │
│  │ - Panels        │        │ - Hot-reload    │        │ - Replication   │  │
│  │ - Rendering     │        │ - Sandbox       │        │ - Lag Comp      │  │
│  └────────┬────────┘        └────────┬────────┘        └────────┬────────┘  │
│           │                          │                          │           │
│           │         ┌────────────────┴──────────────┬───────────┘           │
│           │         │                               │                       │
│           ▼         ▼                               ▼                       │
│  ┌──────────────────────────────────────────────────────────────────┐      │
│  │                       praxis_ecs                                  │      │
│  │                                                                   │      │
│  │  - World & Entities      - Components       - Resources          │      │
│  │  - Systems & Schedule    - Queries          - Events             │      │
│  │  - Change Detection      - Commands         - bevy_ecs backend   │      │
│  └────────┬──────────────────────────────────────────────┬──────────┘      │
│           │                                              │                  │
│           │         ┌─────────────────────┬──────────────┘                  │
│           │         │                     │                                 │
│           ▼         ▼                     ▼                                 │
│  ┌──────────────┐ ┌──────────────┐  ┌──────────────┐                      │
│  │praxis_scene  │ │praxis_spatial│  │praxis_assets │                      │
│  │              │ │              │  │              │                      │
│  │- Transforms  │ │- Octree      │  │- OBJ/GLTF    │                      │
│  │- Hierarchy   │ │- BVH         │  │- Textures    │                      │
│  │- Animation   │ │- Frustum     │  │- Materials   │                      │
│  │- Cameras     │ │- Culling     │  │- Caching     │                      │
│  └──────┬───────┘ └──────┬───────┘  └──────┬───────┘                      │
│         │                │                  │                              │
│         │         ┌──────┴──────────────────┴─────────────┐                │
│         │         │                                        │                │
│         ▼         ▼                                        ▼                │
│  ┌──────────────────────┐         ┌──────────────────────────────────┐    │
│  │  praxis_graphics     │         │    praxis_procedural             │    │
│  │                      │         │                                  │    │
│  │ - Vulkan Context    │◄────────│ - Texture Graphs                 │    │
│  │ - Pipelines         │         │ - Noise Functions                │    │
│  │ - Forward Renderer  │         │ - GPU Generation                 │    │
│  │ - Deferred Renderer │         │ - GLSL Compilation               │    │
│  │ - Shadows           │         │ - LRU Cache                      │    │
│  │ - HDR/Tonemapping   │         └──────────────────────────────────┘    │
│  │ - Post-processing   │                                                  │
│  └──────────┬───────────┘                                                  │
│             │                                                              │
│             │         ┌────────────────────┬───────────────────┐          │
│             │         │                    │                   │          │
│             ▼         ▼                    ▼                   ▼          │
│       ┌──────────────────┐      ┌──────────────────┐  ┌──────────────┐   │
│       │ praxis_physics   │      │  praxis_audio    │  │praxis_profile│   │
│       │                  │      │                  │  │              │   │
│       │ - Rapier3D       │      │ - Kira Backend   │  │- Frame Time  │   │
│       │ - RigidBody      │      │ - 3D Spatial     │  │- Statistics  │   │
│       │ - Colliders      │      │ - Attenuation    │  │- Tracking    │   │
│       │ - Raycasting     │      │ - Doppler Effect │  └──────────────┘   │
│       └──────────┬───────┘      └──────────┬───────┘                      │
│                  │                         │                              │
└──────────────────┼─────────────────────────┼──────────────────────────────┘
                   │                         │
                   │         ┌───────────────┘
                   │         │
                   ▼         ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                          PLATFORM LAYER                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌──────────────────┐           ┌──────────────────┐                        │
│  │ praxis_window    │           │  praxis_input    │                        │
│  │                  │           │                  │                        │
│  │ - winit         │           │ - Keyboard       │                        │
│  │ - Event Loop     │──────────►│ - Mouse          │                        │
│  │ - Surface        │           │ - Gamepad        │                        │
│  └────────┬─────────┘           └────────┬─────────┘                        │
│           │                              │                                   │
└───────────┼──────────────────────────────┼───────────────────────────────────┘
            │                              │
            └──────────────┬───────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                        FOUNDATION LAYER                                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌──────────────────┐           ┌──────────────────┐                        │
│  │  praxis_utils    │           │  praxis_math     │                        │
│  │                  │           │                  │                        │
│  │ - Logging        │           │ - glam (Vec3)    │                        │
│  │ - Errors         │           │ - Mat4, Quat     │                        │
│  │ - Timing         │           │ - Transforms     │                        │
│  │ - color-eyre     │           └──────────────────┘                        │
│  └──────────────────┘                                                        │
│                                                                               │
│       ┌────────────────────────────────────────────┐                         │
│       │          praxis_core                       │                         │
│       │                                            │                         │
│       │ - Engine Lifecycle    - Initialization     │                         │
│       │ - Main Loop           - Shutdown           │                         │
│       │ - Configuration       - Entry Point        │                         │
│       └────────────────────────────────────────────┘                         │
│                                                                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Dependency Rules

### Layer Dependencies

**Allowed**: Layers may only depend on layers below them:
- Application Layer → Engine Layer → Platform Layer → Foundation Layer
- Same-layer dependencies are allowed where appropriate

**Prohibited**: Reverse dependencies (lower layers depending on higher layers)

### Key Principles

1. **Foundation Independence**: `praxis_utils` and `praxis_math` have no internal dependencies
2. **ECS Centrality**: Most engine systems depend on `praxis_ecs` for component storage and system execution
3. **Graphics Hub**: `praxis_graphics` is the main integration point for rendering subsystems
4. **Optional Features**: Some crates (`praxis_editor`, `praxis_scripting`, `praxis_networking`, `praxis_terrain`) are optional features

## Detailed Dependency Matrix

| Crate | Direct Dependencies |
|-------|-------------------|
| **praxis_utils** | *(none - foundation)* |
| **praxis_math** | *(none - foundation)* |
| **praxis_core** | utils, window, graphics, ecs, input, audio |
| **praxis_window** | utils |
| **praxis_input** | utils, window |
| **praxis_ecs** | utils, math, graphics |
| **praxis_graphics** | utils, math, procedural |
| **praxis_procedural** | utils, math |
| **praxis_scene** | ecs, math, utils |
| **praxis_spatial** | ecs, math, utils |
| **praxis_assets** | utils, ecs |
| **praxis_physics** | ecs, math, utils |
| **praxis_audio** | ecs, math, utils |
| **praxis_profiling** | ecs, utils |
| **praxis_gui** | core, graphics, window, ecs, input |
| **praxis_scripting** | core, ecs, assets, input, scene, math |
| **praxis_networking** | core, ecs, math |
| **praxis_terrain** | core, graphics, math, ecs |
| **praxis_editor** | core, gui, ecs, graphics, scene, spatial |

## Critical Paths

### Rendering Path
```
Application
    ↓
praxis_core → praxis_graphics → vulkano
    ↓              ↓
praxis_ecs    praxis_math
```

### Physics Path
```
Application
    ↓
praxis_physics → rapier3d
    ↓
praxis_ecs ← Transform Sync → praxis_scene
```

### Animation Path
```
Application
    ↓
praxis_scene → Animation System
    ↓
praxis_ecs → Skeleton & Bones
    ↓
praxis_graphics → Rendering
```

## External Dependencies

### Major External Crates

| Subsystem | External Dependency | Purpose |
|-----------|-------------------|---------|
| **Graphics** | `vulkano` 0.35.1 | Vulkan API wrapper |
| **ECS** | `bevy_ecs` 0.14 | Entity-Component-System |
| **Math** | `glam` | SIMD math (Vec3, Mat4, Quat) |
| **Physics** | `rapier3d` 0.22 | Physics simulation |
| **Audio** | `kira` | Audio playback and 3D sound |
| **Window** | `winit` 0.30.11 | Cross-platform windowing |
| **GUI** | `egui` 0.29 | Immediate-mode GUI |
| **Scripting** | `mlua` | Lua 5.4 runtime |
| **Networking** | `tokio` 1.40 | Async runtime |
| **Logging** | `tracing` | Structured logging |

## Feature Flags

Optional crates can be enabled via Cargo features:

```toml
[features]
default = []
editor = ["praxis_editor"]
networking = ["praxis_networking"]
scripting = ["praxis_scripting", "praxis_gui/scripting"]
terrain = ["praxis_terrain", "praxis_editor?/terrain"]
```

**Usage**:
```bash
# Full engine with all features
cargo build --all-features

# Minimal build
cargo build --no-default-features

# Editor only
cargo build --features editor

# Game build with networking and scripting
cargo build --features "networking,scripting"
```

## Data Flow Patterns

### Frame Update Flow
```
1. praxis_core (main loop tick)
   ↓
2. praxis_input (collect events)
   ↓
3. praxis_ecs (run systems)
   ├─→ praxis_physics (step simulation)
   ├─→ praxis_scene (update transforms)
   ├─→ praxis_audio (update 3D positions)
   └─→ praxis_scripting (execute scripts)
   ↓
4. praxis_graphics (render frame)
   ├─→ praxis_spatial (frustum culling)
   ├─→ praxis_procedural (generate textures)
   └─→ GPU submission
```

### Asset Loading Flow
```
Application
   ↓
praxis_assets (load OBJ/GLTF)
   ↓
Parse & Deserialize
   ├─→ Mesh data → praxis_graphics (upload to GPU)
   ├─→ Material data → praxis_graphics (create pipeline)
   ├─→ Animation data → praxis_scene (animation clips)
   └─→ Texture data → praxis_graphics (GPU textures)
   ↓
praxis_ecs (spawn entities with loaded components)
```

## Build Considerations

### Parallel Compilation

Crates with no dependencies can compile in parallel:
- `praxis_utils` and `praxis_math` compile first
- `praxis_window`, `praxis_input`, `praxis_graphics`, `praxis_ecs` can build in parallel after foundation

### Incremental Builds

Modifying a crate triggers rebuilds of:
- All crates that directly depend on it
- All crates that transitively depend on it

**Example**: Changing `praxis_ecs` requires rebuilding:
- `praxis_scene`, `praxis_spatial`, `praxis_physics`, `praxis_audio`
- `praxis_gui`, `praxis_editor`, `praxis_scripting`, `praxis_networking`

### Optimization

**Release builds**: Use `--release` flag for production builds
```bash
cargo build --release --all-features
```

**Link-Time Optimization** (enabled in workspace `Cargo.toml`):
```toml
[profile.release]
lto = "thin"
codegen-units = 1
```

## Related Documentation

- [Crates Reference](../reference/crates.md) - Detailed per-crate documentation
- [Architecture Overview](../architecture.md) - High-level engine design
- [Project Structure](../getting-started/project-structure.md) - Workspace layout
- [ECS Patterns](ecs-patterns.md) - Component and system design patterns
- [Engine Lifecycle](engine-lifecycle.md) - Initialization and main loop
- [Render Pipeline](render-pipeline.md) - Graphics subsystem architecture
