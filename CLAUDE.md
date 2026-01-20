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
cargo run --example complete_features_demo
cargo run --example skeletal_animation_demo
cargo run --example animation_blending_demo
cargo run --example animation_advanced_demo
cargo run --example gltf_animation_loader_demo
cargo run --example audio_demo
cargo run --example audio_simple
cargo run --example editor_demo
cargo run --example editor_camera_demo
cargo run --example gui_demo
cargo run --example console_demo
cargo run --example menu_bar_demo
cargo run --example scripting_demo
cargo run --example scripting_advanced_demo
cargo run --example scripting_console_demo
cargo run --example networking_demo
cargo run --example terrain_demo
cargo run --example spatial_partitioning_demo
cargo run --example spatial_optimization_demo
cargo run --example scene_demo
cargo run --example scene_serialization_demo
cargo run --example save_load_demo
cargo run --example material_demo
cargo run --example material_instancing_demo
cargo run --example advanced_lighting_demo
cargo run --example environment_probe_demo
cargo run --example particles_demo
cargo run --example profiling_demo
cargo run --release --example performance_profiling_comprehensive
cargo run --example selection_demo
cargo run --example command_system_demo
cargo run --example command_serialization_demo
cargo run --example undo_redo_system_demo
cargo run --example transform_propagation_demo
cargo run --example gpu_culling_demo
cargo run --example lod_gpu_demo
cargo run --example mesh_streaming_demo
cargo run --example multi_mesh_demo
cargo run --example input_integration
cargo run --example fps_camera_controller
cargo run --example ecs_integration
cargo run --example texture_compression_demo
cargo run --example hiz_occlusion_demo
cargo run --example rendering_stress_test
cargo run --example optimization_showcase_demo
cargo run --example render_stats_demo
cargo run --example hardware_tier_demo

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

**Details**: `docs/guides/rendering.md`, `docs/guides/rendering/hdr-tonemapping.md`

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

### Debug Rendering
- **`DebugRenderer`**: Visual debugging for optimization systems
- **Culling Visualization**: Wireframe bounding spheres (green=visible, red=culled)
- **LOD Heat Map**: Color-coded LOD levels (blue=high detail, red=low detail)
- **Occlusion Buffer**: Hierarchical Z-buffer visualization
- **Mesh Streaming**: Loading progress and state indicators

```rust
// Create debug renderer
let debug_renderer = DebugRenderer::new(device, allocator, render_pass, [1920, 1080])?;

// Enable debug modes
debug_renderer.enable_mode(DebugRenderMode::CullingResults);
debug_renderer.enable_mode(DebugRenderMode::LodHeatMap);

// Render overlays
debug_renderer.render_all_debug(&mut cmd_builder, &culling_info, &lod_info, &streaming_info, view_proj)?;
```

**Details**: `crates/praxis_graphics/DEBUG_RENDERING.md`

## Educational Value & Design Rationale

Praxis is designed as an educational 3D game engine. Each subsystem exists to teach specific concepts while maintaining production-quality patterns. Use this rationale when evaluating feature additions:

### Core Engine (`praxis_core`, `praxis_window`, `praxis_utils`)
**What it teaches**: Engine architecture fundamentals, application lifecycle management, cross-cutting concerns (logging, error handling, timing).  
**Why it exists**: Demonstrates how to structure a game engine's foundation, manage the main loop, handle platform abstraction (windowing), and implement essential utilities that all subsystems depend on.  
**Evaluation criteria**: New features should focus on foundational patterns that apply across all engine types, not game-specific logic.

### Rendering (`praxis_graphics`)
**What it teaches**: Modern Vulkan rendering, forward/deferred pipelines, HDR tone mapping, shadow mapping, GPU-driven optimization (culling, LOD, occlusion), debug visualization.  
**Why it exists**: Modern graphics APIs like Vulkan are complex but powerful. Shows how to build safe Rust abstractions over Vulkan using `vulkano`, implement common rendering techniques, and visualize optimization systems for learning.  
**Evaluation criteria**: Additions should demonstrate fundamental rendering techniques or modern GPU-driven approaches, not bleeding-edge research or engine-specific hacks.

### ECS (`praxis_ecs`)
**What it teaches**: Entity Component System architecture using `bevy_ecs`, data-oriented design, composition over inheritance.  
**Why it exists**: ECS is a proven pattern for game engines. Demonstrates how to structure game logic around components and systems rather than traditional OOP hierarchies.  
**Evaluation criteria**: Components should represent reusable, composable data. Systems should be focused and demonstrate clear ECS patterns.

### Math (`praxis_math`)
**What it teaches**: 3D mathematics using `glam`, vectors, matrices, quaternions, coordinate spaces.  
**Why it exists**: Provides thin wrapper around battle-tested math library while demonstrating common 3D math patterns and conventions. Shows when to use library types vs. custom abstractions.  
**Evaluation criteria**: Only add math utilities that demonstrate common game engine patterns, not problem-specific calculations.

### Scene Management (`praxis_scene`)
**What it teaches**: Transform hierarchies, parent-child relationships, skeletal animation, animation blending, clip playback.  
**Why it exists**: Scene graphs and transform propagation are fundamental to 3D engines. Animation systems demonstrate complex state management and temporal interpolation.  
**Evaluation criteria**: Features should focus on core scene organization and animation techniques applicable to most 3D applications.

### Spatial Structures (`praxis_spatial`)
**What it teaches**: Spatial partitioning (octrees, BVH), spatial queries, performance optimization through data structure choice.  
**Why it exists**: Demonstrates how spatial data structures enable efficient queries and culling in large 3D worlds. Shows trade-offs between different structures.  
**Evaluation criteria**: Additions should demonstrate fundamental spatial algorithms with clear performance characteristics and use cases.

### Asset Pipeline (`praxis_assets`)
**What it teaches**: Asset loading (OBJ, GLTF), parsing file formats, async loading, resource management.  
**Why it exists**: Shows how to integrate standard 3D formats, handle I/O efficiently, and manage asset lifetime. Demonstrates practical parsing and data transformation.  
**Evaluation criteria**: Support formats that are industry-standard and demonstrate different design philosophies (simple vs. complex, text vs. binary).

### Input (`praxis_input`)
**What it teaches**: Input abstraction, keyboard/mouse/gamepad handling, input mapping, frame-by-frame state management.  
**Why it exists**: Demonstrates how to abstract platform-specific input and provide game-friendly APIs. Shows state vs. event-based input patterns.  
**Evaluation criteria**: Features should demonstrate input patterns common across games, not application-specific bindings.

### GUI (`praxis_gui`)
**What it teaches**: Immediate-mode GUI using `egui`, editor UI patterns, tool development.  
**Why it exists**: Shows how to integrate GUI systems into rendering pipeline, build editor tools, and handle UI state. Demonstrates immediate-mode vs. retained-mode trade-offs.  
**Evaluation criteria**: GUI additions should focus on editor/tool patterns, not game HUD (which belongs in separate examples).

### Physics (`praxis_physics`)
**What it teaches**: Physics integration using `rapier3d`, rigid body simulation, collision detection, ECS-physics synchronization.  
**Why it exists**: Demonstrates how to integrate a physics engine, sync with ECS transforms bidirectionally, handle fixed timesteps, and expose physics features through ECS.  
**Evaluation criteria**: Features should show practical physics integration patterns, not advanced simulation techniques better left to Rapier itself.

### Audio (`praxis_audio`)
**What it teaches**: Audio playback using `kira`, spatial audio, sound management, resource pooling.  
**Why it exists**: Shows how to integrate audio middleware, manage sound resources, and implement common audio patterns (background music, sound effects, spatial positioning).  
**Evaluation criteria**: Demonstrate common audio patterns in games, not advanced DSP or music production features.

### Procedural Generation (`praxis_procedural`)
**What it teaches**: Node-based texture generation, noise algorithms (Perlin, Simplex, Worley), GPU compute shaders, runtime GLSL compilation, LRU caching.  
**Why it exists**: Demonstrates GPU-accelerated procedural content, shader compilation pipeline, and cache management. Shows graph-based composition patterns.  
**Evaluation criteria**: Focus on foundational procedural techniques and GPU compute patterns, not specific artistic use cases.

### Terrain (`praxis_terrain`)
**What it teaches**: Terrain generation, height maps, LOD systems for terrain, chunk management.  
**Why it exists**: Terrain is a common game feature with unique challenges (scale, LOD, streaming). Demonstrates specialized rendering and data management.  
**Evaluation criteria**: Features should demonstrate scalable terrain techniques applicable to various games, not specific biome/game logic.

### Profiling (`praxis_profiling`)
**What it teaches**: Performance measurement, timing, profiling integration, identifying bottlenecks, Chrome trace export, rendering statistics integration.  
**Why it exists**: Performance is critical in game engines. Shows how to instrument code, measure frame time, identify optimization opportunities, and export comprehensive profiling data. Demonstrates integration between subsystems (profiling + graphics) for holistic performance analysis.  
**Evaluation criteria**: Additions should help users understand engine performance, not solve specific optimization problems. Integration features should demonstrate cross-cutting profiling patterns.

### Scripting (`praxis_scripting`)
**What it teaches**: Lua integration via `mlua`, script-ECS bridge, hot-reload, sandboxing, performance monitoring.  
**Why it exists**: Demonstrates how to embed scripting, expose engine functionality safely, enable rapid iteration, and monitor script performance.  
**Evaluation criteria**: Focus on engine-script integration patterns and safety, not Lua language features or game-specific scripts.

### Networking (`praxis_networking`)
**What it teaches**: Client-server architecture, entity replication, interpolation/extrapolation, lag compensation, network profiling.  
**Why it exists**: Multiplayer is complex. Shows how to synchronize game state, handle latency, implement server authority, and monitor network performance.  
**Evaluation criteria**: Demonstrate foundational networking patterns for games, not specific game genres or protocols.

### Editor (`praxis_editor`)
**What it teaches**: Editor architecture, selection systems, undo/redo, gizmos, transform tools, command pattern.  
**Why it exists**: Shows how to build editor tools, implement robust undo/redo, handle user manipulation, and separate editor from runtime.  
**Evaluation criteria**: Features should demonstrate general editor patterns useful across projects, not application-specific tools.

### General Principles for Feature Evaluation

When considering new features or subsystems:

1. **Educational First**: Does it teach a fundamental concept or pattern?
2. **Broad Applicability**: Is it useful across multiple game types/projects?
3. **Production Quality**: Does it demonstrate industry patterns, not toy examples?
4. **Appropriate Scope**: Does it fit the subsystem's educational focus?
5. **Clear Trade-offs**: Does it illustrate design decisions and their consequences?
6. **Avoid Over-Engineering**: Keep implementations clear and maintainable over maximally abstract.
7. **Complement, Don't Duplicate**: Does it teach something distinct from existing subsystems?

**Anti-patterns to reject**:
- Game-specific logic (belongs in examples, not engine crates)
- Bleeding-edge research without proven educational value
- Features that obscure rather than illuminate underlying concepts
- Duplicate approaches without clear pedagogical distinction
- Dependencies on non-standard or unstable libraries without strong justification

## Naming Conventions

### Type Suffixes

Use these suffixes consistently to clarify responsibility:

| Suffix | Purpose | Examples | When to Use |
|--------|---------|----------|-------------|
| **Manager** | Resource caching, asset loading, lifetime management | `AudioManager`, `TextureManager`, `SceneManager` | Manages a pool/cache of resources, handles allocation/deallocation, provides retrieval APIs |
| **Renderer** | GPU rendering, draw calls, pipeline management | `DeferredRenderer`, `TerrainRenderer`, `ParticleRenderer` | Encapsulates Vulkan pipelines, command buffers, and draw logic; issues GPU commands |
| **System** | ECS behavior, component processing | `SelectionSystem`, `UndoRedoSystem`, `PhysicsSystem` | Processes ECS components/queries each frame; implements game logic or editor behavior |
| **Context** | Top-level API coordinator (rare) | `RenderContext` | Manages API lifecycle, coordinates multiple subsystems, serves as primary entry point |

**Functions** that act as systems use `_system` suffix (e.g., `physics_step_system`, `frustum_culling_system`).

**Context types** are reserved for top-level types that manage entire API lifecycles and coordinate multiple subsystems. Most types should use Manager/Renderer/System. Only use Context for top-level API abstractions like `RenderContext` (which manages Vulkan lifecycle, coordinates managers/renderers, and handles synchronization).

**Anti-patterns to avoid**:
- Using `System` for non-ECS types (should use `Renderer` or `Manager` instead)
- Using `Manager` for types that only render (should be `Renderer`)
- Using `Context` for simple resource management (should be `Manager`)
- Creating new Context types without clear justification (rare pattern)

**Note**: Some existing types predate these conventions. See `dev-notes/NAMING_STANDARDIZATION.md` for migration tracking and the RenderContext evaluation.

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
| Window | winit |
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
