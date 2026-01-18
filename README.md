# Praxis

A 3D game engine written in Rust, focused on learning game engine fundamentals while building something practical.

## What is Praxis?

Praxis is an educational yet capable game engine that prioritizes:

- **Clarity over abstraction** — Code is extensively documented to help you understand how game engines work
- **Idiomatic Rust** — Leverages Rust's safety and performance without fighting the language
- **Battle-tested libraries** — Built on vulkano, bevy_ecs, rapier3d, kira, and other proven crates

## Features

### Rendering
- Vulkan-based forward and deferred rendering pipelines via vulkano
- Cascaded shadow mapping with PCF soft shadows
- PBR materials with normal mapping
- HDR rendering with tone mapping (ACES, Reinhard, Uncharted 2)
- Automatic and manual exposure control
- Skybox and environment rendering with IBL support
- Environment probes for physically-based reflections
- GPU-accelerated particle system with emission, forces, and collision
- GPU-driven frustum culling and LOD selection

### Animation
- Skeletal animation with keyframe interpolation
- Animation blending (cross-fade, 1D/2D blend trees, layers)
- Bone masking for partial-body animation
- GLTF animation loading
- IK, retargeting, additive blending, root motion

### Physics
- Rapier3D integration with fixed timestep (60 Hz)
- Rigid bodies (dynamic, static, kinematic)
- Collision detection and events
- Raycasting and spatial queries

### Spatial Optimization
- Octree-based spatial partitioning
- Frustum culling for efficient rendering
- LOD (Level of Detail) system with GPU-driven selection
- Broad-phase optimization for physics and rendering

### Audio
- 3D spatial audio with distance attenuation
- Doppler effect simulation
- Support for OGG, MP3, WAV, FLAC

### Assets
- GLTF/GLB loading with materials and animations
- OBJ model support
- PNG/JPEG textures

### Scripting
- Lua 5.4 integration with ECS access
- Hot-reload support
- Configurable sandboxing
- Performance monitoring

### Networking
- Client-server architecture
- Entity replication with interpolation/extrapolation
- Lag compensation
- Network profiling

### Editor Tools
- Selection and raycast picking
- Undo/redo system
- Transform gizmos
- Orbit camera controller

### Performance
- CPU/GPU profiling
- Memory tracking
- Chrome trace export
- Mesh streaming

## Quick Start

```bash
# Clone and build
git clone https://github.com/gariasf/praxis
cd praxis
cargo build

# Run examples
cargo run --example hello_triangle
cargo run --example comprehensive_scene_demo
cargo run --example animation_demo
cargo run --example editor_demo --features editor
```

## Requirements

- Rust (latest stable via [rustup](https://rustup.rs/))
- Vulkan-capable GPU and drivers

## Project Structure

```
praxis/
├── crates/
│   ├── praxis_core       # Engine lifecycle and initialization
│   ├── praxis_graphics   # Vulkan rendering, shaders, materials
│   ├── praxis_ecs        # Entity-Component-System (bevy_ecs)
│   ├── praxis_physics    # Physics simulation (Rapier3D)
│   ├── praxis_scene      # Scene graph, transforms, animation
│   ├── praxis_audio      # Audio playback and spatial sound (kira)
│   ├── praxis_assets     # Asset loading (GLTF, OBJ)
│   ├── praxis_input      # Keyboard, mouse, gamepad
│   ├── praxis_gui        # Debug UI (egui)
│   ├── praxis_window     # Window management (winit)
│   ├── praxis_math       # Math utilities (glam)
│   ├── praxis_spatial    # Spatial partitioning and optimization
│   ├── praxis_editor     # Editor tools and GUI
│   ├── praxis_procedural # Procedural generation (textures, terrain)
│   ├── praxis_profiling  # Performance profiling and monitoring
│   ├── praxis_scripting  # Lua scripting integration
│   ├── praxis_networking # Networking and multiplayer
│   ├── praxis_terrain    # Terrain generation and rendering
│   └── praxis_utils      # Logging, timing, error handling
├── examples/             # Runnable demos
├── assets/               # Models, textures, sounds
└── docs/                 # Architecture and guides
```

## Examples

See [examples/README.md](examples/README.md) for a complete catalog with detailed descriptions.

### Core Examples

| Example | Description |
|---------|-------------|
| `hello_triangle` | **START HERE** - Minimal rendering example (~200 lines) |
| `comprehensive_scene_demo` | Complete asset pipeline with OBJ loading and textures |
| `complete_features_demo` | **Full showcase** - All engine systems working together |

### Rendering Examples

| Example | Description |
|---------|-------------|
| `material_demo` | PBR materials with post-processing |
| `material_instancing_demo` | Efficient per-object material property overrides |
| `environment_probe_demo` | IBL reflections with environment probes |
| `particles_demo` | GPU-accelerated particle system |
| `advanced_lighting_demo` | Advanced lighting techniques |
| `gpu_culling_demo` | GPU-driven frustum culling (1000+ objects) |
| `lod_gpu_demo` | GPU-driven LOD selection with compute shaders |
| `terrain_demo` | Heightmap terrain with LOD and texture splatting |

### Animation Examples

| Example | Description |
|---------|-------------|
| `animation_demo` | Interactive animation with blend transitions |
| `skeletal_animation_demo` | Bone hierarchies and keyframe animation |
| `animation_blending_demo` | Cross-fading, blend trees, layers |
| `animation_advanced_demo` | IK, retargeting, additive blending, root motion |
| `gltf_animation_loader_demo` | Loading animations from GLTF files |

### Editor Examples

| Example | Description |
|---------|-------------|
| `editor_demo` | Full editor with panels and tools |
| `editor_camera_demo` | Orbit camera controller |
| `selection_demo` | Entity selection and raycast picking |
| `console_demo` | Console panel with log filtering |
| `menu_bar_demo` | Menu bar with file, edit, entity, view, help menus |

### System Examples

| Example | Description |
|---------|-------------|
| `audio_demo` | Spatial audio with doppler effect |
| `scripting_console_demo` | Interactive Lua REPL with ECS access |
| `networking_demo` | Client-server with entity replication |
| `spatial_optimization_demo` | Frustum culling, octree, BVH, LOD |
| `profiling_demo` | CPU/GPU profiling and trace export |
| `save_load_demo` | Game state persistence with metadata |

Run any example with:
```bash
cargo run --example <name>

# Some examples require feature flags:
cargo run --example editor_demo --features editor
cargo run --example terrain_demo --features terrain
cargo run --example scripting_console_demo --features scripting
cargo run --example networking_demo --features networking
cargo run --example complete_features_demo --features "terrain,networking,scripting"
```

## Development

```bash
# Run tests
cargo test --workspace

# Run benchmarks
cargo bench

# Check code quality
cargo fmt --all -- --check
cargo clippy --all -- -D warnings

# Generate docs
cargo doc --workspace --no-deps --open
```

## Documentation

The codebase is heavily documented. Start with:

- [Documentation Index](docs/README.md) — Complete documentation overview
- [Architecture](docs/architecture.md) — High-level system design
- [Beginner's Guide](docs/beginners-guide.md) — Step-by-step introduction to all systems
- [Getting Started](docs/getting-started/README.md) — Installation and setup
- [Guides](docs/guides/README.md) — Task-oriented tutorials
- [Learning Paths](docs/learning-paths/README.md) — Structured progressions from beginner to advanced
- Shader files in `crates/praxis_graphics/src/shaders/` — Extensive educational comments

## Goals

- Learn game engine architecture and 3D graphics programming
- Create an engine using idiomatic Rust practices
- Build something practical enough for actual game development
- Use only free, open-source, battle-tested libraries

## Contributing

Contributions welcome. Please:

1. Follow existing code style (clippy pedantic + nursery)
2. Add rustdoc comments for public items
3. Include tests for new functionality
4. Run `cargo fmt` and `cargo clippy` before submitting

## License

MIT

## Acknowledgments

Built on excellent open-source libraries:
- [vulkano](https://github.com/vulkano-rs/vulkano) — Vulkan wrapper
- [bevy_ecs](https://github.com/bevyengine/bevy) — Entity Component System
- [rapier3d](https://rapier.rs/) — Physics engine
- [kira](https://github.com/tesselode/kira) — Audio library
- [glam](https://github.com/bitshifter/glam-rs) — Linear algebra
- [winit](https://github.com/rust-windowing/winit) — Window creation
- [egui](https://github.com/emilk/egui) — Immediate mode GUI
