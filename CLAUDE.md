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
cargo run --example physics_demo

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

Praxis uses a Cargo workspace with 11 crates organized by subsystem. The root `praxis` crate coordinates all subsystems:

- **praxis_core**: Engine lifecycle, main loop coordination, initialization sequence
- **praxis_window**: Window management via `winit`, event loop handling
- **praxis_graphics**: Vulkan rendering via `vulkano`, shader compilation, render context, mesh/texture management
- **praxis_ecs**: Entity-Component-System using `bevy_ecs`
- **praxis_math**: Math utilities, re-exports `glam` types (Vec3, Mat4, etc.)
- **praxis_scene**: Scene graph and spatial organization with transform hierarchy
- **praxis_assets**: Asset loading/management (OBJ models, textures, config files)
- **praxis_input**: Keyboard/mouse/gamepad handling
- **praxis_gui**: Debug/editor GUI via `egui`
- **praxis_physics**: Physics simulation using `Rapier3D`, collision detection, spatial queries
- **praxis_utils**: Shared utilities, logging (`tracing`), error handling, frame timing

### Initialization Flow

The engine follows a specific initialization sequence in `praxis_core::run()`:

1. `praxis_utils::init()` - Sets up logging and error reporting
2. `praxis_ecs::init()` - Initializes ECS system
3. `praxis_input::init()` - Initializes input system
4. `praxis_physics::init()` - Initializes physics system
5. `praxis_window::run()` - Creates event loop and window, then:
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

### Physics System

The physics system provides realistic physics simulation using Rapier3D:

- **PhysicsWorld**: ECS resource managing the Rapier physics pipeline
- **RigidBody**: Component defining physics behavior (Dynamic, Static, Kinematic)
- **Collider**: Component defining collision geometry (boxes, spheres, capsules, etc.)
- **PhysicsVelocity**: Linear and angular velocity tracking
- **ExternalForces**: Force and torque accumulation for dynamic bodies
- **Collision Events**: Event system for detecting and responding to collisions
- **Spatial Queries**: Raycasting, shape casting, and point intersection tests

The physics system uses fixed timestep integration (60 Hz by default) for deterministic,
stable simulation. Transform synchronization happens bidirectionally with the ECS.

#### Physics Systems

The physics simulation requires these systems to be scheduled in order:

1. **`clear_collision_event_receivers`**: Clears event buffers before physics step
2. **`sync_physics_transforms_system`**: Syncs ECS transforms to Rapier (runs before physics)
3. **`physics_step_system`**: Advances the simulation using fixed timestep
4. **`sync_physics_transforms_system`**: Syncs Rapier results back to ECS (runs after physics)
5. **`populate_collision_events`**: Distributes collision events to entity components

Alternative legacy systems:
- **`sync_transforms_to_physics`**: One-way sync (ECS → Rapier)
- **`step_physics_simulation`**: Simple physics step without fixed timestep
- **`sync_transforms_from_physics`**: One-way sync (Rapier → ECS)

Optional systems:
- **`apply_external_forces`**: Applies accumulated forces/torques to bodies
- **`sync_colliders`**: Creates/updates Rapier colliders from components
- **`sync_physics_properties`**: Updates velocities, friction, restitution

#### Key Physics Concepts

**Fixed Timestep Integration**: Physics runs at a constant rate (default 60 Hz) independent
of frame rate. This ensures deterministic, stable simulation. The `PhysicsTime` accumulator
tracks time between frames and steps the simulation multiple times if needed to catch up.

**Rigid Body Types**:
- **Dynamic**: Affected by forces, gravity, and collisions. Used for moving objects like
  balls, boxes, and physics-driven entities.
- **Static**: Never moves, has infinite mass. Used for terrain, walls, and fixed level geometry.
- **Kinematic**: Moved by code/animation, not physics. Affects dynamic bodies but isn't
  affected by them. Used for moving platforms, doors, and player-controlled objects.

**Transform Synchronization**: The system maintains bidirectional sync between ECS `Transform`
components and Rapier rigid body positions. Before physics: kinematic bodies push their
Transform to Rapier. After physics: dynamic bodies pull their position from Rapier.

**Collision Detection**: Rapier performs collision detection in multiple phases:
- **Broad Phase**: Spatial partitioning (AABB tree) to quickly find potentially colliding pairs
- **Narrow Phase**: Precise geometric tests (GJK, SAT) to determine actual collisions
- **Contact Generation**: Creates contact manifolds with points, normals, and penetration
- **Constraint Solver**: Applies impulses to resolve collisions and enforce joint constraints

**Collision Events**: The system provides three event types:
- `CollisionStarted`: Two bodies begin colliding (first contact)
- `CollisionStopped`: Two bodies stop colliding (contact lost)
- `CollisionPersisted`: Two bodies continue colliding (ongoing contact)

Events are stored in `CollisionEventReceiver` components on entities, allowing entity-centric
event handling. The `ContactEvents` resource collects global collision events from Rapier.

**Spatial Queries**: The `PhysicsWorld` provides efficient spatial queries:
- **Raycast**: Cast an infinitely thin ray to find the first hit
- **Raycast All**: Cast a ray and return all hits along the path
- **Shape Cast**: Sweep a 3D shape to detect collisions (useful for character controllers)
- **Point Inside**: Check if a point is inside any collider

These queries use spatial acceleration structures (BVH) for O(log n) performance.

See `praxis_physics` documentation and `examples/physics_demo.rs` for detailed usage patterns.

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

### Physics System (praxis_physics)
- Built on Rapier3D physics engine
- ECS-first design with components, resources, and systems
- Fixed timestep integration for deterministic simulation (60 Hz default)
- Bidirectional transform synchronization with ECS
- Collision event system with entity-centric event distribution
- Spatial queries (raycasting, shape casting, point tests)
- Components: RigidBody, Collider, PhysicsVelocity, ExternalForces, etc.
- Resources: PhysicsWorld, PhysicsConfig, PhysicsTime, ContactEvents
- System ordering critical: clear events → sync → step → sync → populate events

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
- **Physics**: `rapier3d` (rigid body dynamics, collision detection)

## Project Philosophy

From README.md and docs/architecture.md:
- Use only free/open, battle-proven libraries
- Prioritize simplicity and clarity over abstraction
- Focus on pragmatic, iterative feature development
- Leverage Rust's safety and performance characteristics
- Build toward supporting real game development
