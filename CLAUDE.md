# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Project Overview

Praxis is a 3D game engine in Rust using `vulkano` (Vulkan), `bevy_ecs`, and `rapier3d`.

**Documentation**: See `docs/README.md` for comprehensive guides, concepts, and reference.

## Essential Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build
cargo build -p praxis_editor   # Specific crate

# Test & Quality
cargo test --workspace         # All tests
cargo fmt --all                # Format code
cargo clippy --all -- -D warnings  # Lint (must pass)

# Run Examples
cargo run --example hello_triangle
cargo run --example comprehensive_scene_demo
cargo run --example skeletal_animation_demo
cargo run --example animation_demo
cargo run --example animation_blending_demo
cargo run --example animation_advanced_demo
cargo run --example audio_demo
cargo run --example audio_simple
cargo run --example editor_demo
cargo run --example editor_camera_demo
cargo run --example gui_demo
cargo run --example scripting_demo
cargo run --example scripting_advanced_demo
cargo run --example networking_demo
cargo run --example terrain_demo
cargo run --example spatial_partitioning_demo
cargo run --example spatial_optimization_demo
cargo run --example scene_demo
cargo run --example scene_serialization_demo
cargo run --example material_demo
cargo run --example advanced_lighting_demo
cargo run --example environment_probe_demo
cargo run --example particles_demo
cargo run --example profiling_demo
cargo run --example profiling_advanced_demo
cargo run --example spatial_optimization_demo
cargo run --example selection_demo
cargo run --example command_system_demo
cargo run --example undo_redo_system_demo
cargo run --example transform_propagation_demo

# Documentation
cargo doc --workspace --no-deps --open

# Benchmarks
cargo bench                          # Run all benchmarks
cargo bench --bench asset_loading    # Asset loading (OBJ/GLTF)
cargo bench --bench scene_serialization  # Scene serialization/deserialization
cargo bench --bench mesh_upload      # GPU mesh upload
cargo bench --bench physics_step     # Physics simulation
cargo bench --bench transform_propagation  # Transform hierarchy
cargo bench --bench render_loop      # Camera and rendering
```

## Architecture

19-crate workspace organized by subsystem:

| Crate | Purpose |
|-------|---------|
| `praxis_core` | Engine lifecycle, main loop |
| `praxis_window` | Window management (winit) |
| `praxis_graphics` | Vulkan rendering |
| `praxis_ecs` | ECS (bevy_ecs) |
| `praxis_math` | Math (glam) |
| `praxis_scene` | Transform hierarchy, animation |
| `praxis_spatial` | Spatial data structures (octree, BVH) |
| `praxis_assets` | Asset loading (OBJ, GLTF) |
| `praxis_input` | Keyboard, mouse, gamepad |
| `praxis_gui` | Editor GUI (egui) |
| `praxis_physics` | Physics (Rapier3D) |
| `praxis_audio` | Audio (Kira) |
| `praxis_procedural` | Procedural textures |
| `praxis_terrain` | Terrain generation and LOD |
| `praxis_profiling` | Performance profiling |
| `praxis_scripting` | Lua scripting integration |
| `praxis_networking` | Networking and multiplayer |
| `praxis_editor` | Editor tools |
| `praxis_utils` | Logging, errors, timing |

**Details**: `docs/reference/crates.md`, `docs/architecture.md`

## Key Patterns

### ECS
```rust
#[derive(Component)]
struct Health(f32);

fn damage_system(mut query: Query<&mut Health>) {
    for mut hp in query.iter_mut() {
        hp.0 -= 1.0;
    }
}
```

### Transform Hierarchy
- `Transform`: Local position, rotation, scale
- `GlobalTransform`: Computed world-space
- `Parent`/`Children`: Hierarchy relationships

### Rendering
- Forward rendering: `RenderContext::render()`
- Deferred rendering: `DeferredRenderer`
- HDR: `ToneMapper` with ACES/Reinhard
- Shadows: Cascaded shadow maps

**Details**: `docs/guides/rendering.md`, `docs/guides/hdr-and-tonemapping.md`

### Physics
- `RigidBody`: Dynamic, Static, Kinematic
- `Collider`: Boxes, spheres, capsules
- Fixed timestep (60 Hz default)
- Bidirectional transform sync with ECS

**Details**: `crates/praxis_physics/README.md`

### Procedural Textures
- **`TextureGraph`**: Node-based texture composition
- **Noise functions**: Perlin, Simplex, Worley
- **GPU generation**: Runtime GLSL-to-SPIR-V compute shader compilation and dispatch
- **Caching**: Automatic LRU cache with configurable limits
- **Performance**: 5-10ms for 512x512 textures on GPU
```rust
let mut graph = TextureGraph::new();
let noise = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0, octaves: 4,
    persistence: 0.5, lacunarity: 2.0,
});
graph.set_output(noise);
let params = TextureGenerationParams { width: 512, height: 512, seed: 0 };
let texture = manager.generate_texture(&graph, params)?;
```

**Details**: `crates/praxis_procedural/README.md`

### Animation
- `Skeleton`, `AnimationClip`, `AnimationPlayer`
- Blend trees, layered animation, cross-fading

**Details**: `docs/guides/animation/` (skeletal-basics.md, blending.md, advanced-features.md)

### Scripting
- **Lua 5.4** integration via `mlua`
- **ECS access**: Query/modify entities and components from scripts
- **Hot-reload**: Auto-reload scripts on file changes
- **Sandboxing**: Configurable security levels (None/Moderate/Strict)
- **Performance monitoring**: Track execution time, warn on slow scripts
```rust
let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;
context.load_script("game_logic", "scripts/game.lua")?;
context.enable_hot_reload("scripts")?;
```

**Details**: `crates/praxis_scripting/README.md`, `docs/guides/scripting.md`

### Networking
- **Client-Server**: TCP/UDP transport with connection management
- **Entity Replication**: Automatic component synchronization
- **Interpolation/Extrapolation**: Smooth remote entity movement
- **Lag Compensation**: Server-side rewind for fair hit detection
- **Network Profiler**: Bandwidth and latency monitoring
```rust
let config = NetworkConfig::default();
let mut server = NetworkServer::new(config).await?;
server.start().await?;

let mut registry = ReplicationRegistry::new();
registry.register_transform();
registry.register_velocity();
```

**Details**: `crates/praxis_networking/README.md`

### Editor
- Selection: `SelectionSystem`, `Selectable` component
- Undo/Redo: `UndoRedoSystem`, command history
- Gizmos: Transform manipulation
- Camera: Orbit controller

**Details**: `docs/editor/README.md`

## Naming Conventions

### Type Suffixes

Use these suffixes consistently to clarify responsibility:

| Suffix | Purpose | Examples | When to Use |
|--------|---------|----------|-------------|
| **Manager** | Resource caching, asset loading, lifetime management | `AudioManager`, `TextureManager`, `SceneManager` | Manages a pool/cache of resources, handles allocation/deallocation, provides retrieval APIs |
| **Renderer** | GPU rendering, draw calls, pipeline management | `DeferredRenderer`, `TerrainRenderer`, `ParticleRenderer` | Encapsulates Vulkan pipelines, command buffers, and draw logic; issues GPU commands |
| **System** | ECS behavior, component processing | `SelectionSystem`, `UndoRedoSystem`, `PhysicsSystem` | Processes ECS components/queries each frame; implements game logic or editor behavior |

**Functions** that act as systems use `_system` suffix (e.g., `physics_step_system`, `frustum_culling_system`).

**Anti-patterns to avoid**:
- Using `System` for non-ECS types (e.g., `ParticleSystem` that's actually a renderer)
- Using `Manager` for types that only render (should be `Renderer`)
- Using `Context` inconsistently (prefer `Manager` for resource management)

**Note**: Some existing types predate these conventions. See `dev-notes/NAMING_STANDARDIZATION.md` for migration tracking.

## Code Quality

### Linting (must pass CI)
```toml
clippy::all = "warn"
clippy::pedantic = "warn"
clippy::nursery = "warn"
unsafe_code = "warn"
missing_docs = "warn"
```

All public items require rustdoc comments.

### CI Checks
1. `cargo check --all`
2. `cargo fmt --all -- --check`
3. `cargo clippy --all -- -D warnings`

## Common Tasks

### Adding Components
1. Define in `crates/praxis_ecs/src/components.rs`
2. Derive `Component`
3. Add rustdoc
4. Export from `lib.rs`

### Graphics Changes
- Shaders: `crates/praxis_graphics/src/shaders/`
- Compiled via `vulkano-shaders` macro
- `RenderContext` manages Vulkan resources

### Physics
- Components: `RigidBody`, `Collider`, `PhysicsVelocity`
- Resources: `PhysicsWorld`, `PhysicsConfig`
- System order matters: sync → step → sync → events

## Dependencies

| Category | Crates |
|----------|--------|
| Graphics | vulkano, vulkano-shaders |
| Window | winit 0.30 |
| Math | glam |
| ECS | bevy_ecs |
| Physics | rapier3d |
| Audio | kira |
| GUI | egui, egui_vulkano |
| Logging | tracing |
| Errors | color-eyre |

## Documentation Structure

```
docs/
├── README.md              # Main index
├── beginners-guide.md     # Learning resource
├── getting-started/       # Installation, setup
├── guides/                # How-to guides
├── concepts/              # Theory explanations
├── reference/             # API reference
└── editor/                # Editor docs
```

## Philosophy

- Use battle-proven libraries
- Prioritize clarity over abstraction
- Pragmatic, iterative development
- Leverage Rust's safety guarantees
