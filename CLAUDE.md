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
cargo run --example shadow_demo
cargo run --example audio_demo

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

Praxis uses a Cargo workspace with 12 crates organized by subsystem. The root `praxis` crate coordinates all subsystems:

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
- **praxis_audio**: Audio system using `kira`, spatial audio, sound management
- **praxis_utils**: Shared utilities, logging (`tracing`), error handling, frame timing

### Initialization Flow

The engine follows a specific initialization sequence in `praxis_core::run()`:

1. `praxis_utils::init()` - Sets up logging and error reporting
2. `praxis_ecs::init()` - Initializes ECS system
3. `praxis_input::init()` - Initializes input system
4. `praxis_audio::init()` - Initializes audio system
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

### Shadow Mapping System

The shadow mapping system provides realistic shadows using cascaded shadow maps (CSM):

- **ShadowMapManager**: Manages shadow map resources and light-space matrix calculation
- **ShadowConfig**: Configures shadow quality, cascade count, distances, and PCF filtering
- **ShadowUniforms**: Shadow data passed to shaders (light-space matrices, cascade info)
- **Cascaded Shadow Maps (CSM)**: Multiple shadow maps at different distances for quality
- **PCF Filtering**: Percentage Closer Filtering for soft shadow edges

#### Shadow Mapping Overview

Shadow mapping uses a two-pass rendering technique:

1. **Shadow Pass**: Render scene from light's perspective to depth texture (shadow map)
2. **Main Pass**: Sample shadow maps to determine if fragments are shadowed

#### Cascade Configuration

CSM divides the view frustum into multiple cascades:
- **Near cascade**: High detail for close objects (e.g., 0-20m)
- **Mid cascades**: Medium detail for mid-range objects (e.g., 20-100m)
- **Far cascade**: Lower detail for distant objects (e.g., 100-500m)

Default configuration: 3 cascades at [20.0, 100.0, 500.0] meters

#### PCF Filtering

PCF samples multiple shadow map points and averages results:
- **1 sample**: Hard shadows (best performance)
- **4 samples**: 2×2 filter (soft shadows, good performance)
- **9 samples**: 3×3 filter (softer shadows, medium performance)
- **16 samples**: 4×4 filter (softest shadows, lower performance)

#### Key Features

**Light-Space Matrix Calculation**: Automatically computes view and projection matrices
for rendering from light's perspective for each cascade, fitting frustum bounds tightly.

**Shadow Bias**: Configurable bias to prevent shadow acne (self-shadowing artifacts).
Default: 0.005, with additional hardware depth bias in shadow pipeline.

**Cascade Selection**: Fragment shader automatically selects appropriate cascade based
on distance from camera, ensuring optimal shadow quality at all ranges.

See `praxis_graphics::shadow` documentation and `examples/shadow_demo.rs` for usage.

### Asset Loading

The asset system supports loading various file formats:

- **OBJ Models**: Via `tobj` crate in `praxis_assets`
- **GLTF/GLB Models**: Via `gltf` crate in `praxis_assets`, supporting:
  - Meshes with positions, normals, UVs, and tangents
  - Node hierarchies with transforms
  - PBR materials (base color, metallic, roughness)
  - Embedded and external textures
  - Multiple primitives per mesh
  - Scene graph structure
- **Textures**: PNG/JPEG via `image` crate in `praxis_graphics`

#### GLTF Loader Usage

```rust
use praxis_assets::{GltfLoader, GltfAssetManager};

// Direct loading
let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/scene.gltf")?;

// Cached loading
let mut manager = GltfAssetManager::new();
let asset = manager.load("assets/models/scene.gltf")?;

// Access loaded data
for (node_index, node) in asset.nodes_with_meshes() {
    let (translation, rotation, scale) = node.decompose_transform();
    let mesh = &asset.meshes[node.mesh_index.unwrap()];
    // Upload mesh to GPU, spawn entities, etc.
}
```

The `GltfAssetManager` caches loaded assets by file path to avoid redundant loading operations.

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
- Shadow mapping requires separate render pass and pipeline
- Shadow shaders in `src/shaders/shadow.vert` and `src/shaders/shadow.frag`
- Main shaders include shadow sampling at bindings 4-8

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

### Audio System (praxis_audio)

The audio system provides sound playback and spatial audio using Kira:

- **AudioManager**: ECS resource managing the Kira audio backend and loaded sounds
- **AudioSource**: Component for spatial audio attached to entities with Transform
- **AudioListener**: Component marking the audio listener (typically the camera)
- **play_sound_system**: System that processes audio playback and spatial audio updates
- **update_spatial_audio_system**: Optimized system for updating spatial audio on transform changes

#### Audio Components

**AudioSource** properties:
- `path`: Path to audio file (OGG, MP3, WAV, FLAC)
- `volume`: Volume level (0.0 to 1.0)
- `spatial`: Enable 3D spatial audio positioning
- `looping`: Whether the audio loops continuously
- `max_distance`: Distance beyond which sound is inaudible
- `reference_distance`: Distance at which volume is at specified level
- `state`: PlaybackState (Playing, Paused, Stopped)

#### Spatial Audio

Spatial audio uses inverse square law for distance attenuation:
`volume = base_volume * (reference_distance / distance)^2`

When `spatial` is true, the audio system:
- Calculates distance between AudioSource and AudioListener
- Applies distance-based attenuation
- Adjusts panning based on relative position
- Updates volume in real-time as entities move

#### Usage Example

```rust
use praxis_audio::{AudioManager, AudioSource, AudioListener, play_sound_system};
use praxis_ecs::{World, Schedule, Transform};

let mut world = World::new();
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);

// Attach listener to camera
world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    AudioListener,
));

// Spawn spatial audio source
world.spawn((
    Transform::from_xyz(10.0, 0.0, 0.0),
    AudioSource::new("assets/sounds/ambient.ogg")
        .with_volume(0.7)
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(50.0),
));

let mut schedule = Schedule::default();
schedule.add_systems(play_sound_system);
```

See `praxis_audio` documentation and `examples/audio_demo.rs` for detailed usage.

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
- **Audio**: `kira` (audio playback, spatial audio)

## Project Philosophy

From README.md and docs/architecture.md:
- Use only free/open, battle-proven libraries
- Prioritize simplicity and clarity over abstraction
- Focus on pragmatic, iterative feature development
- Leverage Rust's safety and performance characteristics
- Build toward supporting real game development
