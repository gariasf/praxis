# Core Features

Praxis provides a comprehensive set of default features that are always available without enabling any feature flags. These systems form the foundation of the engine and cover most common game development needs.

## What's Included by Default

When you build Praxis with `cargo build`, you get all of these systems:

### Rendering (praxis_graphics)

**Vulkan-based 3D rendering** with modern graphics capabilities:

- **Forward and Deferred Rendering**: Choose between forward rendering for simpler scenes or deferred rendering for complex lighting
- **HDR and Tone Mapping**: High dynamic range rendering with ACES and Reinhard tone mappers
- **Physically-Based Rendering (PBR)**: Metallic/roughness workflow with realistic material properties
- **Shadow Mapping**: Cascaded shadow maps with percentage-closer filtering (PCF)
- **Environment Mapping**: Image-based lighting with environment probes for realistic reflections
- **Particle Systems**: GPU-accelerated particle effects with emitters, behaviors, and render modes
- **Level of Detail (LOD)**: Automatic mesh switching based on distance for performance optimization
- **Post-Processing**: Bloom, color grading, and custom effect pipelines

See: [Rendering Guide](../guides/rendering.md), [Deferred Rendering Guide](../guides/deferred-rendering.md)

### Entity-Component-System (praxis_ecs)

**Battle-tested ECS** via `bevy_ecs` integration:

- **Entities and Components**: Flexible composition-based architecture
- **Systems**: Parallel execution with automatic dependency tracking
- **Queries**: Efficient iteration over component combinations
- **Resources**: Global state accessible to systems
- **Events**: Type-safe event passing between systems

See: [ECS Architecture](../concepts/ecs-architecture.md)

### Scene Management (praxis_scene)

**Hierarchical scene graphs** with full serialization support:

- **Transform Hierarchy**: Parent-child relationships with automatic propagation
- **Local and Global Transforms**: Position, rotation, scale in local and world space
- **Animation System**: Skeletal animation with blend trees, cross-fading, and layered animation
- **Scene Serialization**: Save and load complete scenes with JSON format
- **Bone Hierarchies**: Full skeletal animation support for characters and creatures

See: [Transform Hierarchy](../concepts/transform-hierarchy.md), [Animation Guides](../guides/animation/)

### Asset Loading (praxis_assets)

**File format support** for common 3D assets:

- **Mesh Formats**: OBJ and glTF 2.0 with PBR materials
- **Texture Formats**: PNG, JPEG, TGA, BMP
- **Animation Loading**: Skeletal animations from glTF files
- **Material Import**: PBR material properties from asset files

See: [GLTF Loading](../gltf-loading.md), [OBJ Loading](../obj-loading.md)

### Physics (praxis_physics)

**3D physics simulation** powered by Rapier3D:

- **Rigid Bodies**: Dynamic, static, and kinematic body types
- **Colliders**: Boxes, spheres, capsules, convex hulls, and triangle meshes
- **Joints**: Fixed, revolute, prismatic, and spherical constraints
- **Ray Casting**: Collision queries and raycasts
- **Collision Detection**: Broad and narrow phase with contact events
- **Fixed Timestep**: Consistent 60 Hz physics updates with ECS synchronization

See: [Physics Concepts](../concepts/physics.md), [praxis_physics README](../../crates/praxis_physics/README.md)

### Audio (praxis_audio)

**3D spatial audio** via Kira integration:

- **Sound Playback**: Play, pause, stop, and loop audio clips
- **Spatial Audio**: 3D positioning with distance attenuation
- **Audio Emitters**: Attach sounds to entities in the scene
- **Listener Control**: Camera-based audio listener positioning
- **Volume and Pitch**: Runtime control of playback parameters

See: [Audio Guide](../guides/audio.md), [Spatial Audio Concepts](../concepts/spatial-audio.md)

### Input Handling (praxis_input)

**Cross-platform input** for keyboard, mouse, and gamepads:

- **Keyboard**: Key press, release, and hold states
- **Mouse**: Button states, position, motion, and scroll
- **Gamepad**: Xbox/PlayStation controller support with analog sticks and triggers
- **Input Mapping**: Bind actions to multiple input sources

See: [Input Guide](../guides/input.md), [Input System](../input-system.md)

### GUI System (praxis_gui)

**Immediate-mode UI** with egui integration:

- **Debug UI**: Easily create inspector panels, property editors
- **Performance Overlays**: FPS counters, profiling displays
- **Vulkan Integration**: GPU-accelerated rendering via egui_vulkano
- **Layout**: Windows, panels, buttons, sliders, text input

See: [GUI System](../gui-system.md)

### Spatial Structures (praxis_spatial)

**Acceleration structures** for efficient scene queries:

- **Octree**: Hierarchical spatial partitioning for broad-phase queries
- **Bounding Volume Hierarchy (BVH)**: Fast ray-mesh intersection testing
- **Frustum Culling**: Automatic visibility determination for rendering optimization

See: [Spatial Optimization Guide](../guides/spatial-optimization.md)

### Procedural Generation (praxis_procedural)

**GPU-accelerated texture generation**:

- **Texture Graphs**: Node-based composition system
- **Noise Functions**: Perlin, Simplex, Worley, Voronoi
- **Image Operations**: Blend, transform, filter nodes
- **Compute Shaders**: GPU-based generation for real-time performance
- **LRU Caching**: Automatic texture reuse

See: [Procedural Textures](../procedural-textures.md), [praxis_procedural README](../../crates/praxis_procedural/README.md)

### Profiling (praxis_profiling)

**Performance monitoring** tools:

- **CPU Profiling**: Track system execution times
- **GPU Profiling**: Measure render pass durations
- **Frame Timing**: Monitor frame rates and frame time budgets
- **Memory Tracking**: Allocation and usage statistics
- **Hierarchical Scopes**: Nested profiling regions

See: [Profiling Guide](../profiling.md)

### Utilities (praxis_utils)

**Common infrastructure**:

- **Logging**: Structured logging with `tracing` crate
- **Error Handling**: Color-formatted error reports via `color-eyre`
- **Timing**: Frame time, delta time, and fixed timestep utilities

## Building with Default Features

Simply build Praxis normally:

```bash
cargo build
cargo run --example comprehensive_scene_demo
```

All core features are available without any configuration.

## Next Steps

- **Add Optional Features**: See [Feature Flags](feature-flags.md) to enable editor, scripting, networking, or terrain
- **Learn Core Concepts**: Read the [Beginner's Guide](../beginners-guide.md)
- **Try Examples**: Run examples to see features in action (see [docs README](../README.md))
- **Explore Guides**: Dive into specific topics in [Guides](../guides/README.md)
