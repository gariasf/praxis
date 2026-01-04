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

### Animation
- Skeletal animation with keyframe interpolation
- Animation blending (cross-fade, 1D/2D blend trees, layers)
- Bone masking for partial-body animation
- GLTF animation loading

### Physics
- Rapier3D integration with fixed timestep (60 Hz)
- Rigid bodies (dynamic, static, kinematic)
- Collision detection and events
- Raycasting and spatial queries

### Audio
- 3D spatial audio with distance attenuation
- Doppler effect simulation
- Support for OGG, MP3, WAV, FLAC

### Assets
- GLTF/GLB loading with materials and animations
- OBJ model support
- PNG/JPEG textures

## Quick Start

```bash
# Clone and build
git clone https://github.com/gariasf/praxis
cd praxis
cargo build

# Run examples
cargo run --example comprehensive_scene_demo
cargo run --example fps_camera_controller
cargo run --example audio_demo
cargo run --example skeletal_animation_demo
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
│   └── praxis_utils      # Logging, timing, error handling
├── examples/             # Runnable demos
├── assets/               # Models, textures, sounds
└── docs/                 # Architecture and guides
```

## Examples

| Example | Description |
|---------|-------------|
| `comprehensive_scene_demo` | Scene loading with lighting and textures |
| `fps_camera_controller` | First-person camera movement |
| `audio_demo` | Spatial audio with doppler effect |
| `skeletal_animation_demo` | Bone-based character animation |
| `animation_blending_demo` | Blend trees and cross-fades |
| `multi_mesh_demo` | Multiple meshes with PBR materials |
| `input_integration` | Keyboard and mouse input handling |
| `environment_probe_demo` | IBL reflections with environment probes |
| `editor_demo` | Full editor interface with undo/redo |
| `particles_demo` | GPU-accelerated particle system |

### Planned Examples
The following examples are planned for future implementation:
- `deferred_demo` - Deferred rendering with many lights
- `hdr_demo` - HDR with tone mapping and exposure
- `shadow_demo` - Cascaded shadow maps demonstration
- `skybox_demo` - Cubemap skybox rendering
- `physics_demo` - Rigid body physics with collisions
- `obj_loader_demo` - OBJ mesh file loading

Run any example with:
```bash
cargo run --example <name>
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

- `docs/ARCHITECTURE.md` — High-level system design
- `docs/BEGINNERS_GUIDE.md` — Step-by-step introduction
- `docs/RENDERING_EXPLAINED.md` — Deep dive into the renderer
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
