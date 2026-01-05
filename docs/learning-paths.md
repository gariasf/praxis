# Learning Paths

This document provides structured learning tracks for mastering Praxis engine, organized by subsystem and skill level. Follow these paths to progress from beginner to advanced proficiency.

**Quick Reference**: See [Learning Paths Quick Reference](learning-paths-quick-reference.md) for a condensed overview with time estimates, milestones, and role-based recommendations.

## Overview

Each learning path is structured as:

- **Beginner**: Core concepts and basic usage
- **Intermediate**: Integration, optimization, and common patterns
- **Advanced**: Architecture, custom extensions, and performance tuning

## Quick Navigation

| Subsystem | Beginner | Intermediate | Advanced |
|-----------|----------|--------------|----------|
| [Rendering](#rendering-path) | Forward pipeline basics | Deferred & HDR | Custom pipelines |
| [Animation](#animation-path) | Skeletal basics | Blending & layers | IK & retargeting |
| [Physics](#physics-path) | Rigid bodies | Events & queries | Custom integration |
| [Scripting](#scripting-path) | Lua basics | ECS access | Hot-reload & sandboxing |
| [Networking](#networking-path) | Client-server setup | Replication | Lag compensation |
| [Audio](#audio-path) | Playback basics | Spatial audio | Performance optimization |
| [Editor](#editor-path) | Basic tools | Custom panels | Command system |

---

## Rendering Path

**Goal**: Master the graphics pipeline from basic rendering to advanced techniques.

### Beginner: Forward Rendering Fundamentals

**Prerequisites**: None (start here!)

**Core Concepts** (Read first):
- [Vulkan Rendering](concepts/vulkan-rendering.md) - Understand the graphics pipeline
- [PBR Materials](concepts/pbr-materials.md) - Physically-based rendering theory
- [Lighting](concepts/lighting.md) - Light types and calculations

**Practical Guides**:
1. [Rendering Overview](guides/rendering.md) - Forward pipeline basics
2. [Beginner's Guide: Rendering Pipeline](beginners-guide.md#rendering-pipeline-flow) - Detailed flow
3. Run `cargo run --example scene_demo` - See it in action

**Learning Outcomes**:
- ✓ Understand Vulkan rendering flow
- ✓ Create basic meshes and materials
- ✓ Add directional and point lights
- ✓ Use the unified rendering API

**Next Steps**: Once comfortable, proceed to Intermediate

### Intermediate: Advanced Rendering Techniques

**Prerequisites**: Beginner rendering complete

**Practical Guides**:
1. [Deferred Rendering](guides/deferred-rendering.md) - Multi-pass G-buffer pipeline
2. [HDR and Tone Mapping](guides/hdr-and-tonemapping.md) - High dynamic range
3. [Shadows](guides/shadows.md) - Cascaded shadow maps
4. [Environment Probes](guides/environment-probes.md) - Image-based lighting
5. [Post-Processing](guides/post-processing.md) - Bloom, color grading

**Examples to Run**:
- `cargo run --example advanced_lighting_demo`
- `cargo run --example environment_probe_demo`
- `cargo run --example material_demo`

**Learning Outcomes**:
- ✓ Switch between forward and deferred rendering
- ✓ Implement HDR with tone mapping
- ✓ Add realistic shadows
- ✓ Create complex materials with IBL
- ✓ Apply post-processing effects

**Cross-References**:
- [Spatial Optimization](guides/spatial-optimization.md) - For rendering performance
- [Profiling](profiling.md) - Identify rendering bottlenecks

### Advanced: Custom Pipeline Development

**Prerequisites**: Intermediate rendering mastery

**Advanced Topics**:
1. [Architecture: Render Pipeline](architecture/render-pipeline.md) - Internal architecture
2. [Beginner's Guide: Dynamic Uniform Buffers](beginners-guide.md#dynamic-uniform-buffer-ring-system)
3. [Beginner's Guide: Vulkano Abstractions](beginners-guide.md#vulkanvulkano-abstractions)
4. [Shaders Reference](reference/shaders.md) - Shader conventions

**Examples**:
- `cargo run --example advanced_material_demo`
- `cargo run --example advanced_rendering_demo`

**Learning Outcomes**:
- ✓ Understand Vulkano abstraction layers
- ✓ Create custom shaders
- ✓ Optimize descriptor set usage
- ✓ Implement custom rendering pipelines
- ✓ Debug GPU performance issues

**Expert Resources**:
- [Particles](guides/particles.md) - Complex GPU-driven effects
- [Procedural Textures](procedural-textures.md) - Runtime texture generation

---

## Animation Path

**Goal**: Create lifelike character movement from basic skeletal animation to advanced techniques.

### Beginner: Skeletal Animation Basics

**Prerequisites**: Basic understanding of transforms

**Core Concepts**:
- [Animation Concepts](concepts/animation.md) - Theory and design
- [Transform Hierarchy](concepts/transform-hierarchy.md) - Parent-child relationships
- [Beginner's Guide: Transform Propagation](beginners-guide.md#transform-hierarchy-propagation)

**Practical Guides**:
1. [Animation Overview](guides/animation.md) - Quick start
2. [Skeletal Basics](guides/animation/skeletal-basics.md) - Core architecture
3. Run `cargo run --example skeletal_animation_demo`

**Learning Outcomes**:
- ✓ Load skeletal meshes from GLTF
- ✓ Play animation clips
- ✓ Understand skeleton hierarchy
- ✓ Control animation playback

**Next Steps**: Progress to blending for smooth transitions

### Intermediate: Animation Blending & Control

**Prerequisites**: Beginner animation complete

**Practical Guides**:
1. [Blending Guide](guides/animation/blending.md) - Cross-fades and blend trees
2. [Skeletal Animation Complete](guides/animation/skeletal-animation.md) - Full system
3. Run `cargo run --example animation_blending_demo`

**Examples**:
- `cargo run --example animation_demo` - Basic blending
- `cargo run --example animation_advanced_demo` - Complex scenarios

**Learning Outcomes**:
- ✓ Cross-fade between animations
- ✓ Build blend trees (walk → run → sprint)
- ✓ Layer animations (upper body + lower body)
- ✓ Create animation state machines
- ✓ Control blend weights dynamically

**Cross-References**:
- [Input System](guides/input.md) - Connect controls to animations
- [Scripting](guides/scripting.md) - Script-driven animation logic

### Advanced: IK, Retargeting, and Root Motion

**Prerequisites**: Intermediate animation mastery

**Advanced Topics**:
1. [Advanced Features](guides/animation/advanced-features.md) - Complete guide
2. [Quick Reference](quick-reference-advanced-animation.md) - API cheat sheet

**Learning Outcomes**:
- ✓ Implement inverse kinematics (IK)
- ✓ Retarget animations between skeletons
- ✓ Apply additive animation blending
- ✓ Handle root motion for movement
- ✓ Optimize animation performance

**Expert Techniques**:
- Procedural animation generation
- Animation compression
- Multi-threaded skeleton updates

---

## Physics Path

**Goal**: Create realistic physics simulations using Rapier3D integration.

### Beginner: Rigid Body Fundamentals

**Prerequisites**: Basic understanding of transforms

**Core Concepts**:
- [Physics Concepts](concepts/physics.md) - Rigid body theory
- Crate docs: `crates/praxis_physics/README.md`

**Practical Guides**:
1. [Physics Guide](guides/physics.md) - Complete walkthrough
2. [Beginner's Guide: Physics System](beginners-guide.md#physics-system)

**Learning Outcomes**:
- ✓ Create dynamic, static, and kinematic bodies
- ✓ Add colliders (sphere, box, capsule)
- ✓ Configure physics properties (mass, friction)
- ✓ Understand ECS-physics sync

**Next Steps**: Learn collision handling

### Intermediate: Collisions and Interactions

**Prerequisites**: Beginner physics complete

**Practical Topics**:
- Collision events and queries
- Physics materials (friction, restitution)
- Joints and constraints
- Raycasting for gameplay

**Learning Outcomes**:
- ✓ Handle collision events
- ✓ Query physics world
- ✓ Create complex compound colliders
- ✓ Implement character controllers
- ✓ Use raycasts for hit detection

**Cross-References**:
- [Spatial Optimization](guides/spatial-optimization.md) - Broad-phase culling
- [Networking](guides/systems/networking.md) - Physics replication

### Advanced: Custom Integration

**Prerequisites**: Intermediate physics mastery

**Advanced Topics**:
- Custom physics materials
- Advanced joint configurations
- Physics debugging and profiling
- Integration with animation (ragdolls)

**Learning Outcomes**:
- ✓ Create custom physics behaviors
- ✓ Optimize physics performance
- ✓ Debug physics issues effectively
- ✓ Integrate physics with animation systems

---

## Scripting Path

**Goal**: Add runtime flexibility with Lua scripting and hot-reload capabilities.

### Beginner: Lua Basics

**Prerequisites**: None

**Practical Guides**:
1. [Scripting Guide](guides/scripting.md) - Complete guide
2. Crate docs: `crates/praxis_scripting/README.md`
3. Run `cargo run --example scripting_demo`

**Learning Outcomes**:
- ✓ Setup scripting context
- ✓ Load and execute Lua scripts
- ✓ Call Lua functions from Rust
- ✓ Pass data between Rust and Lua

**Next Steps**: Learn ECS access from scripts

### Intermediate: ECS Integration

**Prerequisites**: Beginner scripting + ECS understanding

**Core Concepts**:
- [ECS Architecture](concepts/ecs-architecture.md) - Understand ECS first
- [Beginner's Guide: ECS Data Flow](beginners-guide.md#ecs-data-flow)

**Practical Topics**:
1. Continue [Scripting Guide: ECS Integration](guides/scripting.md#ecs-integration)
2. Run `cargo run --example scripting_advanced_demo`

**Learning Outcomes**:
- ✓ Access entities from Lua
- ✓ Query and modify components
- ✓ Create game logic in Lua
- ✓ Handle component lifecycle

**Cross-References**:
- [Transform System](concepts/transform-hierarchy.md) - Script transforms
- [Input System](guides/input.md) - Script input handling

### Advanced: Hot-Reload and Performance

**Prerequisites**: Intermediate scripting complete

**Advanced Topics**:
- Hot-reload configuration
- Sandboxing levels (security)
- Performance monitoring
- Script debugging techniques

**Learning Outcomes**:
- ✓ Enable hot-reload for rapid iteration
- ✓ Configure sandbox restrictions
- ✓ Monitor script performance
- ✓ Optimize Lua code
- ✓ Debug script errors effectively

**Expert Resources**:
- Custom Lua bindings
- Script compilation
- LuaJIT integration (future)

---

## Networking Path

**Goal**: Build multiplayer games with client-server architecture and lag compensation.

### Beginner: Client-Server Setup

**Prerequisites**: Basic ECS understanding

**Core Concepts**:
- Crate docs: `crates/praxis_networking/README.md`

**Practical Guides**:
1. [Networking Guide](guides/systems/networking.md) - Complete guide
2. Run `cargo run --example networking_demo`

**Learning Outcomes**:
- ✓ Setup server and client
- ✓ Establish connections
- ✓ Handle basic message passing
- ✓ Understand network architecture

**Next Steps**: Learn entity replication

### Intermediate: Entity Replication

**Prerequisites**: Beginner networking complete

**Practical Topics**:
- Component registration
- Automatic synchronization
- Transform interpolation
- Bandwidth optimization

**Learning Outcomes**:
- ✓ Replicate entities across network
- ✓ Configure component sync
- ✓ Smooth remote entity movement
- ✓ Handle late-joining clients
- ✓ Monitor network bandwidth

**Cross-References**:
- [Physics](guides/physics.md) - Replicate physics state
- [Animation](guides/animation.md) - Sync animations

### Advanced: Lag Compensation

**Prerequisites**: Intermediate networking mastery

**Advanced Topics**:
- Server-side rewind
- Client prediction
- Input reconciliation
- Network profiler usage

**Learning Outcomes**:
- ✓ Implement lag compensation
- ✓ Handle high-latency scenarios
- ✓ Debug network issues
- ✓ Optimize for different game types
- ✓ Profile network performance

**Expert Techniques**:
- Custom replication strategies
- Advanced interpolation
- Cheat prevention
- Scalability optimization

---

## Audio Path

**Goal**: Create immersive soundscapes with spatial audio.

### Beginner: Audio Playback

**Prerequisites**: None

**Core Concepts**:
- [Spatial Audio Concepts](concepts/spatial-audio.md) - Theory

**Practical Guides**:
1. [Audio Guide](guides/audio.md) - Complete guide
2. Run `cargo run --example audio_simple`

**Learning Outcomes**:
- ✓ Load audio files
- ✓ Play sounds and music
- ✓ Control volume and pitch
- ✓ Handle audio resources

**Next Steps**: Learn spatial positioning

### Intermediate: Spatial Audio

**Prerequisites**: Beginner audio complete

**Practical Topics**:
1. Continue [Audio Guide: Spatial Audio](guides/audio.md)
2. Run `cargo run --example audio_demo`

**Learning Outcomes**:
- ✓ Position sounds in 3D space
- ✓ Configure listener (camera)
- ✓ Use distance attenuation
- ✓ Add reverb and effects

**Cross-References**:
- [Transform System](concepts/transform-hierarchy.md) - Audio positions
- [Physics](guides/physics.md) - Sound on collision

### Advanced: Performance Optimization

**Prerequisites**: Intermediate audio mastery

**Advanced Topics**:
- Audio streaming
- Sound pooling
- LOD for sounds
- CPU optimization

**Learning Outcomes**:
- ✓ Optimize audio memory usage
- ✓ Manage many simultaneous sounds
- ✓ Implement audio LOD
- ✓ Profile audio performance

---

## Editor Path

**Goal**: Master the editor tools for level design and debugging.

### Beginner: Core Tools

**Prerequisites**: Basic engine usage

**Practical Guides**:
1. [Editor Overview](editor/README.md)
2. [Editor Camera](editor/editor-camera.md)
3. [Hierarchy Panel](editor/hierarchy-panel.md)
4. Run `cargo run --example editor_demo`

**Learning Outcomes**:
- ✓ Navigate with editor camera
- ✓ Select and manipulate entities
- ✓ Use hierarchy panel
- ✓ Basic inspector usage

**Next Steps**: Learn advanced tools

### Intermediate: Advanced Editor Features

**Prerequisites**: Beginner editor tools

**Practical Guides**:
1. [Selection System](editor/selection-system.md)
2. [Asset Browser](editor/asset-browser.md)
3. [Gizmos](editor/gizmos.md)
4. Run `cargo run --example selection_demo`

**Learning Outcomes**:
- ✓ Multi-entity selection
- ✓ Transform gizmos
- ✓ Asset drag-and-drop
- ✓ Entity parenting

**Cross-References**:
- [Scene Format](scene-format-v2.md) - Save/load scenes
- [Input System](guides/input.md) - Editor shortcuts

### Advanced: Extending the Editor

**Prerequisites**: Intermediate editor mastery

**Advanced Topics**:
1. [Undo/Redo System](editor/undo-redo.md)
2. [Command System](editor/README.md)
3. [Custom Panels](editor/panels.md)
4. Run `cargo run --example undo_redo_system_demo`

**Learning Outcomes**:
- ✓ Implement undo/redo
- ✓ Create custom editor panels
- ✓ Extend inspector
- ✓ Add custom tools

---

## Cross-Cutting Concerns

These topics apply across multiple subsystems:

### Performance and Optimization

**Essential Reading**:
- [Profiling](profiling.md) - Performance analysis
- [Spatial Optimization](guides/spatial-optimization.md) - Frustum culling, LOD
- [Beginner's Guide: Memory Management](beginners-guide.md#memory-management-patterns)

**Examples**:
- `cargo run --example profiling_demo`
- `cargo run --example profiling_advanced_demo`
- `cargo run --example spatial_optimization_demo`
- `cargo run --example lod_demo`

### Architecture and Design

**For Engine Contributors**:
- [Architecture](architecture.md) - High-level design
- [ECS Patterns](architecture/ecs-patterns.md) - ECS best practices
- [Engine Lifecycle](architecture/engine-lifecycle.md) - Initialization flow
- [Crates Reference](reference/crates.md) - Workspace organization

### Assets and Resources

**Asset Pipeline**:
- [Assets Guide](guides/assets.md) - Loading meshes, textures, audio
- [GLTF Loading](gltf-loading.md) - GLTF asset format
- [Procedural Textures](procedural-textures.md) - Runtime generation

**Examples**:
- `cargo run --example comprehensive_scene_demo`
- `cargo run --example procedural_texture_demo`

---

## Recommended Learning Sequences

### For Game Developers

**Week 1-2: Core Systems**
1. Rendering (Beginner) → Run `scene_demo`
2. Input (Beginner) → Run `input_integration`
3. Physics (Beginner) → Combine with rendering
4. Audio (Beginner) → Add sound effects

**Week 3-4: Advanced Features**
1. Animation (Beginner + Intermediate)
2. Rendering (Intermediate) - HDR, Shadows
3. Scripting (Beginner + Intermediate)
4. Editor (Beginner) - Level design

**Week 5+: Specialization**
- Multiplayer? → Networking path
- Complex visuals? → Rendering advanced
- AI/Gameplay? → Scripting advanced
- Tools? → Editor advanced

### For Engine Contributors

**Core Architecture** (2-3 weeks):
1. [Beginner's Guide](beginners-guide.md) - Complete read
2. [Architecture](architecture.md) - System design
3. [ECS Patterns](architecture/ecs-patterns.md) - Data flow
4. [Render Pipeline](architecture/render-pipeline.md) - Graphics internals

**Subsystem Deep Dives**:
1. Choose subsystem of interest
2. Read concepts + guides + crate README
3. Study examples and tests
4. Review internal implementation
5. Contribute improvements!

### For Graphics Programmers

**Rendering Focus**:
1. [Vulkan Rendering Concepts](concepts/vulkan-rendering.md)
2. [Beginner's Guide: Rendering sections](beginners-guide.md)
3. All rendering guides (beginner → advanced)
4. [Shaders Reference](reference/shaders.md)
5. Study `praxis_graphics` crate internals

---

## Getting Help

- **Examples**: All examples in `examples/` directory
- **API Docs**: `cargo doc --workspace --no-deps --open`
- **Crate READMEs**: Detailed subsystem documentation
- **Issues**: Check GitHub for common problems

## Contributing to Learning Paths

Found a gap or have suggestions? Consider:
- Adding more examples
- Writing tutorials
- Improving documentation
- Sharing your learning experience
