# Praxis Engine

A 3D game engine written in Rust, focusing on learning game engine fundamentals while building a practical engine for game development.

## Project Status: Phase 1 Complete ✅

**Current Version:** 0.1.0 (Foundation Phase)

Praxis has successfully completed its foundational phase with all core subsystems operational and integrated. The engine is now suitable for small 3D game prototypes and continued development.

### Completed Phase 1 Milestones

- ✅ **Core Architecture**: 11-crate modular workspace design
- ✅ **Rendering System**: Vulkan-based forward renderer with PBR materials
- ✅ **ECS Integration**: bevy_ecs with transform hierarchy system
- ✅ **Physics Engine**: Rapier3D integration with collision detection
- ✅ **Input System**: Keyboard, mouse, and gamepad support
- ✅ **Asset Pipeline**: OBJ model loading and texture management
- ✅ **Scene Graph**: Hierarchical transforms with automatic propagation
- ✅ **Debug UI**: egui integration for runtime inspection
- ✅ **Testing**: Comprehensive integration test suite (50+ tests)
- ✅ **Benchmarks**: 4 performance benchmark suites with Criterion.rs
- ✅ **Documentation**: Extensive rustdoc and architectural guides
- ✅ **CI/CD**: Automated testing, linting, and formatting checks

## Current Capabilities

### Graphics & Rendering
- **Vulkan-based rendering** via vulkano
- **Forward rendering pipeline** with dynamic lighting
- **PBR materials** (metallic-roughness workflow)
- **Blinn-Phong lighting** (directional and point lights)
- **Texture support** (PNG, JPEG via image crate)
- **Multiple mesh rendering** with batching
- **Camera system** with perspective projection and FPS controller
- **Custom shader support** via GLSL

### Physics Simulation
- **Rigid body dynamics** (dynamic, static, kinematic)
- **Collision detection** with multiple primitive shapes
- **Fixed timestep integration** (60 Hz default, deterministic)
- **Collision events** (started, stopped, persisted)
- **Spatial queries** (raycasting, shape casting, point tests)
- **Force and velocity control** for dynamic bodies
- **Bidirectional ECS-Physics sync**

### Scene Management
- **Hierarchical transforms** (position, rotation, scale)
- **Automatic transform propagation** through parent-child relationships
- **Global transform computation** for world-space operations
- **Dynamic reparenting** support
- **Component-based entity system** via bevy_ecs

### Asset Loading
- **OBJ model loading** with custom parser
- **Texture loading** (PNG/JPEG formats)
- **Mesh data management** with CPU and GPU representations
- **Built-in primitives** (cube, pyramid, quad generators)
- **Asset path resolution** with flexible path types

### Input Handling
- **Keyboard input** with key state tracking
- **Mouse input** (position, delta, buttons)
- **Gamepad support** via gilrs
- **Input state resource** for ECS integration

### Developer Tools
- **Debug GUI** with egui for runtime inspection
- **Frame timing** with FPS tracking
- **Comprehensive logging** via tracing
- **Error reporting** with color-eyre
- **Performance benchmarks** for critical systems

## Project Goals

- Learn about Game Engine Foundations, 3D space, and systems programming using Rust.
- Create a game engine using idiomatic Rust practices.
- Build a practical engine capable of supporting game development within a 1-2 year timeframe.
- Eventually support complex game worlds and interactions.

## Project Rules

- Use free/open, battle-proven libraries (crates) only.
- Avoid proprietary or costly tools.
- Prioritize simplicity and clarity in design.
- Focus on pragmatic solutions and iterative feature development.
- Minimize unnecessary abstractions.

## Architecture

Praxis uses a Cargo workspace with 11 specialized crates:

- **praxis_core**: Engine lifecycle, main loop coordination, initialization
- **praxis_window**: Window management via winit, event loop handling
- **praxis_graphics**: Vulkan rendering, shader compilation, mesh/texture management
- **praxis_ecs**: Entity-Component-System using bevy_ecs
- **praxis_math**: Math utilities, re-exports glam types
- **praxis_scene**: Scene graph and spatial organization
- **praxis_assets**: Asset loading/management (OBJ models, textures, config files)
- **praxis_input**: Keyboard/mouse/gamepad handling
- **praxis_gui**: Debug/editor GUI via egui
- **praxis_physics**: Physics simulation using Rapier3D
- **praxis_utils**: Shared utilities, logging, error handling, frame timing

See [Architecture Docs](docs/architecture.md) for detailed design information.

## Quick Start

### Prerequisites

- **Rust**: Latest stable version (install via [rustup](https://rustup.rs/))
- **Vulkan**: Vulkan-capable GPU and drivers
  - Linux: Install vulkan-tools and your GPU vendor's drivers
  - Windows: Install latest GPU drivers
  - macOS: Install MoltenVK via Homebrew

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/praxis.git
cd praxis

# Build the entire workspace
cargo build

# Build in release mode for better performance
cargo build --release
```

### Running Examples

```bash
# Basic ECS integration
cargo run --example ecs_integration

# Transform hierarchy demonstration
cargo run --example transform_propagation_demo

# Multiple mesh rendering with PBR materials
cargo run --example multi_mesh_demo

# Input system demonstration
cargo run --example input_integration

# FPS camera controller
cargo run --example fps_camera_controller

# OBJ model loading
cargo run --example obj_loader_demo

# Comprehensive scene with lighting and textures
cargo run --example comprehensive_scene_demo

# Physics simulation with collision detection
cargo run --example physics_demo

# Debug GUI demonstration
cargo run --example gui_demo
```

## Development

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific test suite
cargo test --test integration_test

# Run with output
cargo test --workspace -- --nocapture
```

See [Testing Guide](docs/testing.md) for comprehensive testing documentation.

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench physics_step
cargo bench --bench transform_propagation

# Save baseline for comparison
cargo bench -- --save-baseline main
```

See [Benchmarking Guide](docs/benchmarking.md) for performance optimization details.

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy lints
cargo clippy --all -- -D warnings

# Generate documentation
cargo doc --workspace --no-deps --open
```

## Codebase Statistics

- **Total Lines**: ~25,000 lines of Rust code
- **Crates**: 11 specialized subsystem crates
- **Examples**: 10 demonstration programs
- **Integration Tests**: 5 test files with 50+ test cases
- **Benchmarks**: 4 comprehensive performance suites
- **Documentation**: 600+ line shader comments, beginner guides, architecture docs

## Roadmap

### Phase 2: Essential Rendering (Q1-Q2 2026)
- Shadow mapping (directional lights)
- Normal mapping support
- GLTF/GLB model loading
- Post-processing framework
- Bloom and HDR effects
- Skybox rendering

### Phase 3: Animation & Audio (Q3 2026)
- Skeletal animation system
- Animation blending
- Audio system integration (kira/rodio)
- 3D positional audio

### Phase 4: Advanced Rendering (Q4 2026)
- Deferred rendering option
- Clustered forward rendering
- SSAO (Screen Space Ambient Occlusion)
- Environment probes for reflections

### Phase 5: Editor & Tools (2027)
- Visual scene editor
- Asset hot-reload
- Material editor
- Animation preview
- Performance profiler

See [Strategic Analysis](docs/STRATEGIC_ANALYSIS_2026.md) for detailed roadmap and feature planning.

## Learning Resources (Rust Focus)

- [The Rust Programming Language Book ("The Book")](https://doc.rust-lang.org/book/)
- [VkGuide](https://vkguide.dev/docs/introduction/vulkan_overview/) 
- [The Book of Shaders](https://thebookofshaders.com/) by Patricio Gonzalez Vivo & Jen Lowe 
- [Vulkan Tutorial](https://vulkan-tutorial.com/) 
- [Game Engine Architecture](https://www.gameenginebook.com/) by Jason Gregory 
- [Real-Time Rendering](https://www.realtimerendering.com/) by Tomas Akenine-Möller 
- [Physically Based Rendering Book](https://www.pbr-book.org/) by Matt Pharr, Wenzel Jakob, and Greg Humphreys
- [Foundations of Game Engine Development](https://foundationsofgameenginedev.com/) series by Eric Lengyel

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Beginners Guide](docs/BEGINNERS_GUIDE.md)
- [Testing Guide](docs/TESTING.md)
- [Benchmarking Guide](docs/benchmarking.md)
- [Rendering Explained](docs/RENDERING_EXPLAINED.md)
- [Logging Guide](docs/LOGGING.md)
- [Strategic Analysis](docs/STRATEGIC_ANALYSIS_2026.md)
- [Camera System](docs/camera_system.md)
- [GUI System](docs/gui_system.md)
- [Input System](docs/input_system.md)
- [Mesh System](docs/mesh_system.md)
- [OBJ Loading](docs/obj_loading.md)

## Contributing

Praxis is primarily a learning project, but contributions are welcome! Please:

1. Follow Rust idioms and standard library patterns
2. Maintain existing code quality standards (clippy pedantic + nursery)
3. Add rustdoc comments for all public items
4. Include tests for new functionality
5. Update relevant documentation
6. Run `cargo fmt` and `cargo clippy` before submitting

## License

GPL-3.0-or-later

## Acknowledgments

This project builds upon excellent open-source libraries:
- **vulkano**: Rust wrapper for Vulkan
- **bevy_ecs**: High-performance Entity Component System
- **rapier3d**: 2D and 3D physics engine
- **glam**: Fast linear algebra library
- **winit**: Cross-platform window creation
- **egui**: Immediate mode GUI library
- **image**: Image encoding/decoding
- **tracing**: Application-level tracing
- **criterion**: Statistics-driven benchmarking

## Contact

Guillem Arias - hello@gariasf.com

Project Link: [https://github.com/yourusername/praxis](https://github.com/yourusername/praxis)
