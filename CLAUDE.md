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
cargo run --example skeletal_animation_demo
cargo run --example animation_blending_demo
cargo run --example gltf_animation_loader_demo
cargo run --example deferred_demo
cargo run --example hdr_demo
cargo run --example environment_probe_demo

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
- **praxis_assets**: Asset loading/management (OBJ/GLTF models, skeletal animations, textures, config files)
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

#### Deferred Rendering

The engine provides a complete deferred rendering pipeline alongside forward rendering:

- **`DeferredRenderer`**: Manages G-buffer and lighting passes
- **`GBuffer`**: Multiple render targets (albedo, normal, metallic-roughness, depth)
- **Geometry Pass**: Renders scene geometry to G-buffer
- **Lighting Pass**: Full-screen accumulation of all lights from G-buffer data

**Benefits:**
- **Many Lights**: O(lights × pixels) instead of O(lights × triangles)
- **Efficient Culling**: Only visible pixels are lit
- **Decoupled Shading**: Geometry and lighting are independent

**Trade-offs:**
- Higher memory usage (multiple full-screen render targets)
- Higher bandwidth (multiple render target writes/reads)
- Transparency requires hybrid forward rendering
- MSAA is expensive with multiple render targets

Applications can choose between forward and deferred rendering or use both (e.g., deferred for opaque, forward for transparent).

See `praxis_graphics::deferred` and `examples/deferred_demo.rs` for usage.

#### HDR Rendering

The HDR (High Dynamic Range) rendering system provides floating-point rendering with tone mapping:

- **`HdrRenderTarget`**: Floating-point render targets (R16G16B16A16_SFLOAT)
- **`ToneMapper`**: Complete tone mapping system with exposure control
- **`ToneMappingOperator`**: Multiple operators (ACES, Reinhard, Uncharted 2)
- **`ExposureCalculator`**: Automatic and manual exposure calculation

**HDR Pipeline:**
1. Render scene to floating-point HDR target (values can exceed 1.0)
2. Calculate average scene luminance for auto-exposure
3. Apply exposure adjustment based on luminance or manual value
4. Apply tone mapping to convert HDR to displayable LDR [0,1]
5. Apply gamma correction (typically 2.2)

**Tone Mapping Operators:**
- **Reinhard**: Simple and fast, `color / (color + 1)`
- **ACES**: Industry-standard filmic curve (default, recommended)
- **Uncharted 2**: High contrast, dramatic look (Hable tone mapping)

**Exposure Modes:**
- **Manual**: Fixed exposure value for artistic control
- **Automatic**: Dynamic exposure based on scene luminance with smooth adaptation

**Usage:**
```rust
// Create HDR render target
let hdr_render_pass = render_context.create_hdr_render_pass()?;
let hdr_target = HdrRenderTarget::new(memory_allocator, hdr_render_pass, [1920, 1080])?;

// Create tone mapper with ACES
let mut tone_mapper = ToneMapper::new(device, memory_allocator, format, ToneMappingOperator::ACES)?;

// Set automatic exposure
tone_mapper.set_exposure_mode(ExposureMode::Automatic { speed: 2.0 });

// Apply tone mapping
tone_mapper.apply(builder, &hdr_target, output_framebuffer, extent, average_luminance, delta_time)?;
```

See `praxis_graphics::hdr`, `crates/praxis_graphics/HDR_RENDERING.md`, and `examples/hdr_demo.rs` for usage.

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

### Skeletal Animation System

The skeletal animation system provides keyframe-based animation for bone hierarchies:

- **Skeleton**: Defines bone hierarchy and bind poses for skeletal animation
- **Bone**: Individual bone with parent relationship and bind pose transform
- **AnimationClip**: Stores keyframe animation data for multiple bones
- **AnimationPlayer**: Controls animation playback, looping, speed, and blending
- **AnimatedPose**: Computed bone transforms after animation evaluation
- **BoneTrack**: Keyframe tracks for translation, rotation, and scale channels

#### Animation Components

**Skeleton**: Component containing bone hierarchy and inverse bind matrices
- `new(bones: Vec<Bone>)`: Creates skeleton from bone list
- `bone_count()`: Returns number of bones
- `find_bone(name: &str)`: Finds bone index by name
- `inverse_bind_matrices()`: Gets matrices for skinning

**AnimationClip**: Keyframe animation data for bone transforms
- `new(name: String, duration: f32)`: Creates clip with duration
- `add_bone_track(bone_index: usize)`: Adds track for a bone
- `add_translation_keyframe(bone_index, time, translation)`: Adds translation key
- `add_rotation_keyframe(bone_index, time, rotation)`: Adds rotation key
- `add_scale_keyframe(bone_index, time, scale)`: Adds scale key

**AnimationPlayer**: Controls animation playback
- `new()`: Creates empty player
- `add_clip(name, clip)`: Adds clip to library
- `play(name)`: Starts playing animation
- `pause(name)` / `resume(name)` / `stop(name)`: Playback control
- `set_looping(name, looping)`: Controls looping behavior
- `set_speed(name, speed)`: Sets playback speed multiplier
- `set_weight(name, weight)`: Sets blend weight (0.0-1.0)
- `update(delta_time)`: Advances animation time
- `evaluate(skeleton)`: Produces AnimatedPose from current state

**AnimatedPose**: Final computed bone transforms
- `new(bone_count)`: Creates pose for skeleton
- `local_transforms()`: Gets local bone transforms
- `world_transforms()`: Gets world bone transforms
- `skinning_matrices()`: Gets final matrices for GPU skinning

#### Keyframe Interpolation

The system supports automatic interpolation between keyframes:
- **Translation**: Linear interpolation (lerp) between Vec3 keyframes
- **Rotation**: Spherical linear interpolation (slerp) between Quat keyframes
- **Scale**: Linear interpolation between Vec3 keyframes

Keyframes are automatically sorted by time when added to tracks.

#### Animation Blending

Multiple animations can play simultaneously with weights:
```rust
player.play("Walk");
player.set_weight("Walk", 0.7);
player.play("Run");
player.set_weight("Run", 0.3);
```

Transforms are blended using weighted lerp/slerp based on animation weights.

#### Advanced Animation Blending System

The `AnimationBlender` component provides sophisticated animation blending capabilities:

**Core Features**:
- **Cross-fade Transitions**: Smooth transitions between animations over time
- **Blend Trees**: 1D/2D blend spaces for parameter-driven animation blending
- **Layered Animation**: Multiple animation layers with bone masking
- **Additive Blending**: Add animations on top of base animations

**AnimationBlender**: Advanced blending component
- `new()`: Creates new blender
- `add_clip(name, clip)`: Adds clip to library
- `play(clip_name)`: Plays animation on base layer
- `cross_fade(from, to, duration)`: Cross-fades between animations
- `add_blend_tree(name, node)`: Adds a blend tree
- `activate_blend_tree(name)`: Activates a blend tree
- `set_blend_parameter(tree_name, value)`: Sets 1D blend parameter
- `set_blend_parameters_2d(tree_name, x, y)`: Sets 2D blend parameters
- `add_layer(layer)`: Adds an animation layer
- `play_on_layer(layer_index, clip_name)`: Plays clip on specific layer
- `update(delta_time)`: Updates blending state
- `evaluate(skeleton)`: Produces final AnimatedPose

**Blend Tree Types**:
- **BlendNode1D**: 1D blend space (e.g., idle->walk->run based on speed)
  - `add_clip(name, parameter)`: Adds clip at parameter value
  - `set_parameter(value)`: Sets current parameter for blending
- **BlendNode2D**: 2D blend space (e.g., 8-directional movement)
  - `add_clip(name, x, y)`: Adds clip at 2D position
  - `set_parameters(x, y)`: Sets current position for blending
- **AdditiveBlendNode**: Additive blending
  - `set_base(clip_name)`: Sets base animation
  - `set_additive(clip_name)`: Sets additive animation
  - `set_weight(weight)`: Sets additive weight

**Animation Layers**:
- **AnimationLayer**: Layer for partial skeleton animation
  - `new(weight)`: Creates layer with weight
  - `set_mask(mask)`: Sets bone mask for this layer
  - `set_blend_mode(mode)`: Sets Override or Additive mode
  - `play(clip_name)`: Plays clip on this layer
- **BoneMask**: Controls which bones a layer affects
  - `with_bone_count(count)`: Creates mask for skeleton
  - `enable_bone(index)`: Enables specific bone
  - `enable_bone_and_children(index)`: Enables bone hierarchy

**Cross-Fade Transitions**:
- **CrossFadeTransition**: Smooth transition state
  - `new(from, to, duration)`: Creates transition
  - `blend_weight()`: Gets current blend weight (0.0 to 1.0)
  - `is_complete()`: Checks if transition finished

#### Animation Blending Examples

**Cross-fade transition**:
```rust
let mut blender = AnimationBlender::new();
blender.add_clip("Idle", idle_clip);
blender.add_clip("Walk", walk_clip);

blender.play("Idle");
// Later, smoothly transition to walk over 0.3 seconds
blender.cross_fade("Idle", "Walk", 0.3);
```

**1D Blend Tree (speed-based)**:
```rust
let mut blend_tree = BlendNode1D::new();
blend_tree.add_clip("Idle", 0.0);
blend_tree.add_clip("Walk", 0.5);
blend_tree.add_clip("Run", 1.0);

blender.add_blend_tree("Movement", blend_tree.into());
blender.activate_blend_tree("Movement");
blender.set_blend_parameter("Movement", 0.75); // 75% between walk and run
```

**2D Blend Tree (directional movement)**:
```rust
let mut blend_tree = BlendNode2D::new();
blend_tree.add_clip("Forward", 0.0, 1.0);
blend_tree.add_clip("Back", 0.0, -1.0);
blend_tree.add_clip("Left", -1.0, 0.0);
blend_tree.add_clip("Right", 1.0, 0.0);

blender.add_blend_tree("Locomotion", blend_tree.into());
blender.activate_blend_tree("Locomotion");
blender.set_blend_parameters_2d("Locomotion", 0.5, 0.5); // Forward-right
```

**Layered animation with bone masking**:
```rust
// Base layer: full body walk
blender.play("Walk");

// Upper body layer: aim animation
let mut upper_body_mask = BoneMask::with_bone_count(skeleton.bone_count());
upper_body_mask.enable_bone(spine_index);
upper_body_mask.enable_bone_and_children(left_arm_index);
upper_body_mask.enable_bone_and_children(right_arm_index);

let mut upper_layer = AnimationLayer::new(1.0);
upper_layer.set_mask(upper_body_mask);
upper_layer.set_blend_mode(LayerBlendMode::Override);

blender.add_layer(upper_layer);
blender.play_on_layer(0, "Aim");
// Result: Character walks while aiming upper body
```

**Additive blending**:
```rust
let mut additive_node = AdditiveBlendNode::new();
additive_node.set_base("Walk");
additive_node.set_additive("Recoil");
additive_node.set_weight(1.0);

blender.add_blend_tree("CombatMovement", additive_node.into());
blender.activate_blend_tree("CombatMovement");
// Result: Walk animation with recoil added on top
```

**Update system**:
```rust
fn blending_system(
    mut query: Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>
) {
    let delta_time = 0.016;
    praxis_scene::update_animation_blenders(delta_time, &mut query);
}
```

#### Usage Example

```rust
use praxis_scene::{Skeleton, AnimationClip, AnimationPlayer, AnimatedPose, Bone};
use praxis_math::{Vec3, Quat};

// Create skeleton
let skeleton = Skeleton::new(vec![
    Bone::with_bind_pose("Root".to_string(), None, Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    Bone::with_bind_pose("Arm".to_string(), Some(0), Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE),
]);

// Create animation
let mut clip = AnimationClip::new("Wave".to_string(), 1.0);
clip.add_rotation_keyframe(1, 0.0, Quat::IDENTITY);
clip.add_rotation_keyframe(1, 0.5, Quat::from_rotation_z(PI / 2.0));
clip.add_rotation_keyframe(1, 1.0, Quat::IDENTITY);

// Setup player
let mut player = AnimationPlayer::new();
player.add_clip("Wave".to_string(), clip);
player.play("Wave");

// Spawn entity
let pose = AnimatedPose::new(skeleton.bone_count());
world.spawn((skeleton, player, pose));

// Update in game loop
fn animation_system(mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>) {
    let delta_time = 0.016; // From timing system
    praxis_scene::update_animations(delta_time, &mut query);
}
```

See `praxis_scene::animation`, `examples/skeletal_animation_demo.rs`, and `examples/animation_blending_demo.rs` for complete usage.

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
  - Skeletal animations with keyframe interpolation
  - Skins/skeletons with bone hierarchies and inverse bind matrices
- **Textures**: PNG/JPEG via `image` crate in `praxis_graphics`

#### GLTF Loader Usage

```rust
use praxis_assets::{GltfLoader, GltfAssetManager};
use praxis_scene::{AnimationPlayer, AnimatedPose};

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

// Access skeletal animations
for animation in &asset.animations {
    println!("Animation: {:?}, duration: {}s", animation.name, animation.duration);
}

// Load skeleton and animations for animated characters
if let Some(skin) = asset.skins.first() {
    let skeleton = skin.skeleton.clone();
    let mut player = AnimationPlayer::new();
    
    // Add all animations to the player
    for animation in &asset.animations {
        let name = animation.name.clone().unwrap_or_else(|| "Unnamed".to_string());
        player.add_clip(name, animation.clip.clone());
    }
    
    // Spawn entity with animation
    let pose = AnimatedPose::new(skeleton.bone_count());
    world.spawn((skeleton, player, pose));
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
- **update_listener_system**: System that updates spatial audio when listener transform changes

#### Audio Components

**AudioSource** properties:
- `path`: Path to audio file (OGG, MP3, WAV, FLAC)
- `volume`: Volume level (0.0 to 1.0)
- `spatial`: Enable 3D spatial audio positioning
- `looping`: Whether the audio loops continuously
- `max_distance`: Distance beyond which sound is inaudible
- `reference_distance`: Distance at which volume is at specified level
- `doppler_enabled`: Enable doppler effect for pitch shifting
- `doppler_scale`: Scale factor for doppler effect (0.0 to disable, 1.0 for normal)
- `state`: PlaybackState (Playing, Paused, Stopped)

#### Spatial Audio

Spatial audio uses inverse square law for distance attenuation:
`volume = base_volume * (reference_distance / distance)^2`

When `spatial` is true, the audio system:
- Calculates distance between AudioSource and AudioListener
- Applies distance-based attenuation
- Adjusts stereo panning based on relative X-axis position
- Updates volume and panning in real-time as entities move

#### Doppler Effect

The doppler effect simulates realistic pitch changes based on relative velocity:
- Pitch increases when source approaches listener (higher frequency)
- Pitch decreases when source moves away (lower frequency)
- Uses classic doppler formula: `f' = f * c / (c - v_radial)`
- Speed of sound: 343.0 world units/second (configurable in systems.rs)
- Playback rate clamped to 0.5-2.0 range for stability

Enable doppler with `.with_doppler(true)` and adjust intensity with `.with_doppler_scale(factor)`.

#### Listener Transform Synchronization

The audio system tracks the AudioListener component (typically on the camera):
- Listener position used as reference for all spatial calculations
- `update_listener_system` efficiently updates all audio sources when listener moves
- Change detection ensures minimal overhead when listener is stationary
- Multiple listeners supported (first found is used)

#### Usage Example

```rust
use praxis_audio::{AudioManager, AudioSource, AudioListener, play_sound_system, update_listener_system};
use praxis_ecs::{World, Schedule, Transform, IntoSystemConfigs};

let mut world = World::new();
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);

// Attach listener to camera
world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    AudioListener,
));

// Spawn spatial audio source with doppler effect
world.spawn((
    Transform::from_xyz(10.0, 0.0, 0.0),
    AudioSource::new("assets/sounds/ambient.ogg")
        .with_volume(0.7)
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(50.0)
        .with_doppler(true)
        .with_doppler_scale(1.0),
));

let mut schedule = Schedule::default();
schedule.add_systems((play_sound_system, update_listener_system).chain());
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
