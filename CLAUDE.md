# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Praxis is a 3D game engine written in Rust, focusing on learning game engine fundamentals while building a practical engine for game development within a 1-2 year timeframe. The project uses idiomatic Rust practices and free/open battle-proven libraries.

## Key Commands

### Building and Running
```bash
# Build the entire workspace
cargo build

# Build in release mode
cargo build --release

# Run examples
cargo run --example ecs_integration
cargo run --example transform_propagation_demo
cargo run --example multi_mesh_demo
cargo run --example input_integration
cargo run --example fps_camera_controller
cargo run --example obj_loader_demo
cargo run --example comprehensive_scene_demo
cargo run --example scene_demo
cargo run --example gui_demo

# Check code without building
cargo check --all
```

### Testing and Quality
```bash
# Run all tests in workspace
cargo test --workspace

# Format code
cargo fmt --all

# Check formatting (without modifying files)
cargo fmt --all -- --check

# Run clippy lints (fail on warnings)
cargo clippy --all -- -D warnings

# Run clippy in specific crate
cargo clippy -p praxis_core -- -D warnings
```

### Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate docs for all workspace crates
cargo doc --workspace --no-deps
```

## Architecture

### Workspace Structure

Praxis uses a Cargo workspace with 10 crates organized by subsystem. The root `praxis` crate coordinates all subsystems:

- **praxis_core**: Engine lifecycle, main loop coordination, initialization sequence
- **praxis_window**: Window management via `winit`, event loop handling
- **praxis_graphics**: Vulkan rendering via `vulkano`, shader compilation, render context, mesh/texture management
- **praxis_ecs**: Entity-Component-System using `bevy_ecs`
- **praxis_math**: Math utilities, re-exports `glam` types (Vec3, Mat4, etc.)
- **praxis_scene**: Scene graph and spatial organization with transform hierarchy
- **praxis_assets**: Asset loading/management (OBJ models, textures, config files)
- **praxis_input**: Keyboard/mouse/gamepad handling
- **praxis_gui**: Debug/editor GUI via `egui`
- **praxis_utils**: Shared utilities, logging (`tracing`), error handling, frame timing

### Initialization Flow

The engine follows a specific initialization sequence in `praxis_core::run()`:

1. `praxis_utils::init()` - Sets up logging and error reporting
2. `praxis_ecs::init()` - Initializes ECS system
3. `praxis_input::init()` - Initializes input system
4. `praxis_window::run()` - Creates event loop and window, then:
   - Window creation (default 1920x1080)
   - `State::new()` creates `RenderContext` asynchronously
   - Event loop starts with `ControlFlow::Poll`
   - First `RedrawRequested` triggers rendering

### Rendering Architecture

Graphics rendering uses Vulkano for Vulkan abstraction:

- **RenderContext** (`praxis_graphics`): Manages device, surface, pipeline, and rendering
- **State** (`praxis_window`): Owns RenderContext, handles window events and frame timing
- **Resize handling**: Debounced (16ms) to avoid excessive reconfigurations
- **Frame timing**: Uses `FrameTimer` from `praxis_utils` for delta time and FPS tracking

#### Mesh System

The mesh system provides complete support for loading and rendering 3D geometry:

- **MeshData**: CPU-side mesh definition with vertices, indices, and attributes
- **GpuMesh**: GPU-side mesh containing Vulkan buffers
- **MeshAssetManager**: Central manager for loaded meshes
- **Primitive Generators**: Built-in functions for common shapes (cubes, pyramids, quads)

Meshes support both colored vertices and UV-mapped textures. See `praxis_graphics::mesh` for details.

#### Texture System

The texture system provides support for loading and managing textures:

- **Texture**: GPU-side texture with image view and sampler
- **TextureManager**: Central manager for cached textures
- **Format Support**: PNG and JPEG via the `image` crate
- **Default White Texture**: Automatically created fallback texture

The graphics pipeline supports texture sampling through UV coordinates in the vertex format.

#### Rendering Method

`RenderContext` provides a single unified rendering method:

- **`render()`**: Unified rendering supporting multiple meshes, optional textures, and optional materials per object

The method automatically handles:
- Multiple mesh types per frame
- Optional custom textures (defaults to white if not specified)
- Optional PBR material properties (defaults to standard if not specified)
- Automatic material batching and sorting for optimal performance
- Dynamic lighting updates

Examples demonstrate various usage patterns.

### ECS Integration

Built on `bevy_ecs`, providing:
- Entity spawning via `World::spawn()`
- Component derivation with `#[derive(Component)]`
- Systems that operate on queries
- Re-exported types: `Component`, `Entity`, `Query`, `Commands`, `Res/ResMut`, `Resource`, etc.

See `praxis_ecs` documentation and `examples/ecs_integration.rs` for usage patterns.

### Scene Graph

The scene system provides hierarchical transform management:

- **Transform**: Local position, rotation, scale
- **GlobalTransform**: Computed world-space transform
- **Parent/Children**: Hierarchy relationships
- **Transform Propagation**: Automatic system that updates global transforms

See `praxis_scene` and `examples/transform_propagation_demo.rs` for details.

### Input System

The input system provides keyboard, mouse, and gamepad support:

- **InputState**: Global resource tracking all input state
- **Keyboard**: Key press/release tracking
- **Mouse**: Position, delta, and button state
- **Gamepad**: Button and axis support via `gilrs`

See `praxis_input` and `examples/input_integration.rs` for usage.

### Asset Loading

The asset system supports loading various file formats:

- **OBJ Models**: Via custom parser in `praxis_assets`
- **Textures**: PNG/JPEG via `image` crate
- **Future**: Plans for GLTF, materials, and other formats

## Code Quality Standards

The workspace enforces strict linting:
- `clippy::all = "warn"`
- `clippy::pedantic = "warn"`
- `clippy::nursery = "warn"`
- `unsafe_code = "warn"`
- `missing_docs = "warn"`

All public items must have rustdoc comments (`///` for items, `//!` for modules).

## CI/CD

GitHub Actions workflow (`.github/workflows/rust-ci.yml`) runs on PRs and main branch pushes:
1. `cargo check --all`
2. `cargo fmt --all -- --check`
3. `cargo clippy --all -- -D warnings`

All checks must pass before merging.

## Working with Specific Crates

### Adding New Components (praxis_ecs)
1. Define component struct in `crates/praxis_ecs/src/components.rs`
2. Derive `Component` trait
3. Add rustdoc comments
4. Export from `lib.rs` if needed

### Graphics Changes (praxis_graphics)
- Shaders compiled via `vulkano-shaders` macro
- Surface reconfiguration needed on window resize
- RenderContext manages Vulkan device, queues, swapchain
- All rendering operations return `Result<()>` for error handling
- Mesh and texture managers handle asset lifecycle

### Window/Event Handling (praxis_window)
- Uses winit 0.30.11 with `ApplicationHandler` trait
- Escape key exits application
- Resize events are debounced to avoid performance issues
- State machine: `None` -> `resumed()` -> `Some(State)`

### Scene Management (praxis_scene)
- Transform components provide position, rotation, scale
- Parent/Children components create hierarchies
- `transform_propagation_system` maintains global transforms
- Query patterns access transform data in systems

## Dependencies

Key external crates:
- **Graphics**: `vulkano` (Vulkan), `vulkano-shaders` (shader compilation)
- **Windowing**: `winit` 0.30.11
- **Math**: `glam` (SIMD-accelerated vector/matrix operations)
- **ECS**: `bevy_ecs`
- **Logging**: `tracing`, `tracing-subscriber`
- **Error Handling**: `color-eyre` (via praxis_utils)
- **Input**: `gilrs` (gamepad support)
- **Image Loading**: `image` (PNG/JPEG)
- **GUI**: `egui`, `egui-winit`, `egui_vulkano`

## Project Philosophy

From README.md and docs/architecture.md:
- Use only free/open, battle-proven libraries
- Prioritize simplicity and clarity over abstraction
- Focus on pragmatic, iterative feature development
- Leverage Rust's safety and performance characteristics
- Build toward supporting real game development
