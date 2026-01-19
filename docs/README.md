# Praxis Documentation

Welcome to the Praxis game engine documentation.

## Quick Start

- **[Getting Started](getting-started/README.md)** - Installation, setup, and first steps
- **[Beginners Guide](beginners-guide.md)** - Learn Praxis concepts through hands-on examples
- **[Learning Paths](learning-paths/)** - Structured progressions from beginner to advanced

## Documentation Sections

### [Guides](guides/README.md)
Task-oriented tutorials for implementing features:

**Rendering**
- [Rendering Overview](guides/rendering.md) - Forward and deferred pipelines
- [Rendering Guides](guides/rendering/) - Comprehensive rendering documentation:
  - [Forward Rendering](guides/rendering/forward-rendering.md) - Basic forward rendering pipeline
  - [Deferred Rendering](guides/rendering/deferred-rendering.md) - Multi-pass rendering with G-buffer
  - [HDR and Tone Mapping](guides/rendering/hdr-tonemapping.md) - High dynamic range rendering
  - [Shadows](guides/rendering/shadows.md) - Cascaded shadow maps with PCF
  - [Post-Processing](guides/rendering/post-processing.md) - Bloom, color grading, effects
  - [Environment Probes](guides/rendering/environment-probes.md) - Image-based lighting and reflections
  - [Particles](guides/rendering/particles.md) - Particle effect examples
  - [Advanced Lighting](guides/rendering/advanced-lighting.md) - Light probes and volumetric effects
  - [Advanced Materials](guides/rendering/advanced-materials.md) - PBR and material techniques
  - [LOD System](guides/rendering/lod.md) - Level of detail management
  - [GPU Culling](guides/rendering/gpu-culling.md) - GPU-driven culling techniques
  - [Temporal Anti-Aliasing (TAA)](guides/rendering/taa.md) - Temporal anti-aliasing with velocity buffers
  - [Screen-Space Reflections (SSR)](guides/rendering/ssr.md) - Real-time reflections via ray marching
- [Spatial Optimization](guides/spatial-optimization.md) - Frustum culling, LOD, and occlusion culling

For comprehensive particle rendering documentation, see [crates/praxis_graphics/PARTICLES.md](../crates/praxis_graphics/PARTICLES.md).

**Animation**
- [Animation Overview](guides/animation.md) - Quick start guide
- [Animation Guides](guides/animation/) - Comprehensive animation documentation:
  - [Skeletal Basics](guides/animation/skeletal-basics.md) - Core architecture and fundamentals
  - [Skeletal Animation](guides/animation/skeletal-animation.md) - Complete skeletal animation system
  - [Blending](guides/animation/blending.md) - Cross-fades, blend trees, and layered animation
  - [Advanced Features](guides/animation/advanced-features.md) - IK, retargeting, root motion
  - [Advanced Integration](guides/animation/advanced-integration.md) - Physics and scripting integration

**Assets**
- [Assets Guides](guides/assets/) - Asset pipeline documentation:
  - [Assets Overview](guides/assets/README.md) - Asset loading and management
  - [GLTF Loading](guides/assets/gltf.md) - GLTF format support
  - [OBJ Loading](guides/assets/obj.md) - OBJ format support
  - [Procedural Textures](guides/assets/procedural-textures.md) - Runtime texture generation
  - [Async Asset Loading](guides/async-assets.md) - Non-blocking asset loading with tokio

**Systems**
- [Audio](guides/audio.md) - Spatial audio with Kira
- [Physics](guides/physics.md) - Rigid body dynamics with Rapier3D
- [Input](guides/input.md) - Keyboard, mouse, and gamepad handling
- [Scripting](guides/scripting.md) - Lua scripting integration
- [Terrain](guides/terrain.md) - Terrain generation and rendering
- [Networking](guides/systems/networking.md) - Multiplayer client-server architecture
- [Serialization](guides/serialization.md) - Save/load system with versioning
- [Console](guides/console.md) - In-game console with Lua REPL and ECS introspection

### [Concepts](concepts/README.md)
Educational explanations of engine design:
- [Architecture](architecture.md) - Overall engine design and crate organization
- [ECS Architecture](concepts/ecs-architecture.md) - Entity-Component-System patterns
- [Vulkan Rendering](concepts/vulkan-rendering.md) - Graphics pipeline fundamentals
- [Rendering Pipeline](concepts/rendering-pipeline.md) - Detailed rendering explanation
- [Transform Hierarchy](concepts/transform-hierarchy.md) - Scene graphs and spatial relationships
- [PBR Materials](concepts/pbr-materials.md) - Physically-based rendering theory
- [Lighting](concepts/lighting.md) - Directional and point light systems
- [Animation](concepts/animation.md) - Skeletal animation and blending
- [Physics](concepts/physics.md) - Rigid body simulation with Rapier3D
- [Input](concepts/input.md) - Keyboard, mouse, and gamepad handling
- [Spatial Audio](concepts/spatial-audio.md) - 3D audio positioning

### [Reference](reference/README.md)
API documentation and specifications:
- [Components](reference/components.md) - ECS component reference
- [Crates](reference/crates.md) - Workspace crate overview
- [Crate README Index](reference/crate-readme-index.md) - Quick reference to all crate documentation
- [Shaders](reference/shaders.md) - Shader reference
- [Configuration](reference/configuration.md) - Engine configuration
- [Scene Format](reference/scene-format.md) - Scene file specification
- [Animation API](reference/animation-api.md) - Animation system reference
- [Audio API](reference/audio-api.md) - Audio system reference
- [Camera API](reference/camera-api.md) - Camera system reference
- [GUI API](reference/gui-api.md) - GUI system reference
- [Input API](reference/input-api.md) - Input system reference
- [Mesh API](reference/mesh-api.md) - Mesh system reference

### [Learning Paths](learning-paths/)
Structured progressions for mastering Praxis subsystems:
- **Core Systems**: [Rendering](learning-paths/rendering.md), [Animation](learning-paths/animation.md), [Physics](learning-paths/physics.md), [Scripting](learning-paths/scripting.md), [Networking](learning-paths/networking.md)
- **Supporting Systems**: [Audio](learning-paths/audio.md), [Editor](learning-paths/editor.md), [Assets](learning-paths/assets.md)
- **Cross-Cutting**: [Performance Optimization](learning-paths/performance.md)

Each path provides:
- Clear prerequisites and learning outcomes
- Beginner → Intermediate → Advanced progression
- Hands-on exercises and examples
- Cross-references to related systems

### [Editor](editor/README.md)
Editor tools and workflows:
- [Editor Overview](editor/editor-overview.md) - Panels and editor architecture
- [Selection System](editor/selection-system.md) - Multi-entity selection and raycast picking
- [Asset Browser](editor/asset-browser.md) - Asset management with drag-and-drop
- [Editor Camera](editor/editor-camera.md) - Orbit camera controls and focus
- [Menu Bar](editor/menu-bar.md) - Menu system with keyboard shortcuts
- [Hierarchy Panel](editor/hierarchy-panel.md) - Entity tree with drag-and-drop reparenting
- [Gizmos](editor/gizmos.md) - Transform manipulation gizmos
- [Inspector](editor/inspector.md) - Component property editing
- [Undo/Redo](editor/undo-redo.md) - Command history system

### [Internals](internals/)
Implementation documentation for engine developers and contributors, including:
- Current implementation details for complex subsystems
- Historical documentation and archived audits
- Development notes and design decisions

**Note:** Comprehensive audit reports have been archived in [internals/archived-audits/](internals/archived-audits/) to preserve them as historical reference while acknowledging that line numbers and temporal assessments may become outdated.

## Examples

Run examples to see features in action:
```bash
# Core Demos
cargo run --example comprehensive_scene_demo  # Complete scene with assets
cargo run --example scene_demo                # Basic scene rendering
cargo run --example multi_mesh_demo           # Multiple mesh rendering

# Animation
cargo run --example skeletal_animation_demo   # Skeletal animation
cargo run --example animation_demo            # Basic animation
cargo run --example animation_blending_demo   # Animation blending
cargo run --example animation_advanced_demo   # IK, retargeting, root motion

# Audio
cargo run --example audio_demo                # Spatial audio
cargo run --example audio_simple              # Simple audio playback

# Editor
cargo run --example editor_demo               # Full editor interface
cargo run --example editor_camera_demo        # Editor camera controls
cargo run --example selection_demo            # Entity selection
cargo run --example undo_redo_system_demo     # Undo/redo system

# Effects & Optimization
cargo run --example particles_demo            # Particle rendering effects
cargo run --example spatial_optimization_demo # Frustum culling and LOD
cargo run --example spatial_partitioning_demo # Spatial partitioning
cargo run --example environment_probe_demo    # IBL reflections
cargo run --example advanced_lighting_demo    # Advanced lighting

# Systems
cargo run --example gui_demo                  # GUI system
cargo run --example material_demo             # Material system
cargo run --example terrain_demo              # Terrain rendering
cargo run --example scripting_demo            # Lua scripting
cargo run --example scripting_advanced_demo   # Advanced scripting
cargo run --example scripting_console_demo    # Console with Lua REPL
cargo run --example networking_demo           # Networking features

# Performance & Tools
cargo run --example profiling_demo            # Performance profiling
cargo run --example profiling_advanced_demo   # Advanced profiling
```

## Task Index

**Quick reference for finding documentation by what you want to do:**

### Getting Started Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Install Praxis and run my first example** | [Installation](getting-started/installation.md) | [Getting Started](getting-started/README.md) |
| **Understand the workspace structure** | [Project Structure](getting-started/project-structure.md) | [Crates Reference](reference/crates.md) |
| **Learn Praxis from scratch** | [Beginners Guide](beginners-guide.md) | [Learning Paths](learning-paths/README.md) |
| **Choose what to learn based on my role** | [Learning Paths Quick Start](learning-paths/README.md#quick-start-by-role) | Role-specific progressions |
| **Enable optional features (editor, scripting, etc.)** | [Feature Flags](getting-started/feature-flags.md) | [Configuration](reference/configuration.md) |

### Rendering Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Render my first 3D scene** | [Rendering Overview](guides/rendering.md) | Example: `scene_demo` |
| **Understand the graphics pipeline** | [Vulkan Rendering](concepts/vulkan-rendering.md) | [Rendering Pipeline](concepts/rendering-pipeline.md) |
| **Use forward vs deferred rendering** | [Forward Rendering](guides/rendering/forward-rendering.md), [Deferred Rendering](guides/rendering/deferred-rendering.md) | Example: `comprehensive_scene_demo` |
| **Create physically-based materials** | [PBR Materials](concepts/pbr-materials.md) | [Advanced Materials](guides/rendering/advanced-materials.md), Example: `material_demo` |
| **Add lighting (directional, point, spot)** | [Lighting Concepts](concepts/lighting.md) | [Advanced Lighting](guides/rendering/advanced-lighting.md), Example: `advanced_lighting_demo` |
| **Add shadows** | [Shadows Guide](guides/rendering/shadows.md) | [Rendering API](reference/rendering-api.md) |
| **Enable HDR and tone mapping** | [HDR and Tone Mapping](guides/rendering/hdr-tonemapping.md) | [Rendering Learning Path](learning-paths/rendering.md) |
| **Add post-processing effects** | [Post-Processing](guides/rendering/post-processing.md) | [Bloom](guides/rendering/bloom.md) |
| **Create particle effects** | [Particles Guide](guides/rendering/particles.md) | [PARTICLES.md](../crates/praxis_graphics/PARTICLES.md), Example: `particles_demo` |
| **Add environment-based lighting** | [Environment Probes](guides/rendering/environment-probes.md) | Example: `environment_probe_demo` |
| **Implement temporal anti-aliasing (TAA)** | [TAA Guide](guides/rendering/taa.md) | [Rendering Learning Path (Advanced)](learning-paths/rendering.md) |
| **Add screen-space reflections (SSR)** | [SSR Guide](guides/rendering/ssr.md) | [Rendering Learning Path (Advanced)](learning-paths/rendering.md) |
| **Optimize rendering with LOD** | [LOD System](guides/rendering/lod.md) | [Spatial Optimization](guides/spatial-optimization.md), Example: `lod_gpu_demo` |
| **Implement GPU-driven culling** | [GPU Culling](guides/rendering/gpu-culling.md) | Example: `gpu_culling_demo` |
| **Add frustum culling** | [Frustum Culling](guides/rendering/frustum-culling.md) | [Spatial Optimization](guides/spatial-optimization.md) |
| **Draw debug lines or gizmos** | [Line Rendering](guides/rendering/line-rendering.md) | [Line Rendering Quick Ref](guides/rendering/line-rendering-quick-ref.md) |
| **Create cinematic effects** | [Cinematic Effects](guides/rendering/cinematic-effects.md) | [Post-Processing](guides/rendering/post-processing.md) |
| **Understand material instancing** | [Material Instancing](guides/rendering/material-instancing.md) | Example: `material_instancing_demo` |
| **Build a complete rendering pipeline** | [Rendering Learning Path](learning-paths/rendering.md) | [Architecture: Render Pipeline](architecture/render-pipeline.md) |

### Animation Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Play skeletal animations** | [Animation Overview](guides/animation.md) | [Skeletal Basics](guides/animation/skeletal-basics.md), Example: `skeletal_animation_demo` |
| **Understand animation concepts** | [Animation Concepts](concepts/animation.md) | [Animation API](reference/animation-api.md) |
| **Blend between animations** | [Blending Guide](guides/animation/blending.md) | Example: `animation_blending_demo` |
| **Create animation state machines** | [Blending Guide](guides/animation/blending.md) | [Animation Learning Path](learning-paths/animation.md) |
| **Use inverse kinematics (IK)** | [Advanced Features](guides/animation/advanced-features.md) | Example: `animation_advanced_demo` |
| **Implement animation retargeting** | [Advanced Features](guides/animation/advanced-features.md) | [Animation API](reference/animation-api.md) |
| **Use root motion** | [Advanced Features](guides/animation/advanced-features.md) | Example: `animation_advanced_demo` |
| **Integrate animation with physics** | [Advanced Integration](guides/animation/advanced-integration.md) | [Physics Guide](guides/physics.md) |
| **Load animations from GLTF** | [GLTF Loading](guides/assets/gltf.md) | Example: `gltf_animation_loader_demo` |
| **Master animation system** | [Animation Learning Path](learning-paths/animation.md) | Complete beginner → advanced progression |

### Physics Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Add physics to my game** | [Physics Guide](guides/physics.md) | [Physics Concepts](concepts/physics.md) |
| **Create rigid bodies** | [Physics Guide](guides/physics.md) | [Physics API](reference/physics-api.md) |
| **Add colliders (box, sphere, capsule)** | [Physics Guide](guides/physics.md) | [Physics API](reference/physics-api.md) |
| **Handle collision events** | [Physics Guide](guides/physics.md) | [Physics Learning Path](learning-paths/physics.md) |
| **Implement raycasting** | [Physics API](reference/physics-api.md) | [Physics Learning Path](learning-paths/physics.md) |
| **Create character controllers** | [Physics Learning Path (Intermediate)](learning-paths/physics.md) | [Physics Guide](guides/physics.md) |
| **Use joints and constraints** | [Physics Learning Path (Intermediate)](learning-paths/physics.md) | [Physics API](reference/physics-api.md) |
| **Integrate physics with animation (ragdolls)** | [Physics Learning Path (Advanced)](learning-paths/physics.md) | [Advanced Integration](guides/animation/advanced-integration.md) |
| **Understand ECS-physics synchronization** | [Physics Concepts](concepts/physics.md) | [Crate README](../crates/praxis_physics/README.md) |

### Asset Loading Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Load 3D models (GLTF, OBJ)** | [Assets Overview](guides/assets/README.md) | [GLTF Guide](guides/assets/gltf.md), [OBJ Guide](guides/assets/obj.md) |
| **Load textures** | [Assets Overview](guides/assets/README.md) | [Mesh API](reference/mesh-api.md) |
| **Load audio files** | [Audio Guide](guides/audio.md) | [Audio API](reference/audio-api.md) |
| **Generate procedural textures** | [Procedural Textures](guides/assets/procedural-textures.md) | [Procedural Textures API](reference/procedural-textures-api.md), [Crate README](../crates/praxis_procedural/README.md) |
| **Load assets asynchronously** | [Async Asset Loading](guides/async-assets.md) | [Assets Learning Path](learning-paths/assets.md) |
| **Create custom asset loaders** | [Assets Learning Path (Advanced)](learning-paths/assets.md) | [Assets Overview](guides/assets/README.md) |
| **Implement asset hot-reload** | [Assets Learning Path (Advanced)](learning-paths/assets.md) | [Async Asset Loading](guides/async-assets.md) |

### Scene & Transform Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Understand scene graphs** | [Transform Hierarchy](concepts/transform-hierarchy.md) | Example: `transform_propagation_demo` |
| **Work with transforms** | [Transform Hierarchy](concepts/transform-hierarchy.md) | [Beginners Guide: Transform System](beginners-guide.md) |
| **Create parent-child hierarchies** | [Transform Hierarchy](concepts/transform-hierarchy.md) | [ECS Architecture](concepts/ecs-architecture.md) |
| **Save and load scenes** | [Serialization Guide](guides/serialization.md) | [Scene Format](reference/scene-format.md), Example: `scene_serialization_demo` |
| **Manage scene files** | [Scene Format](reference/scene-format.md) | Example: `save_load_demo` |

### Audio Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Play sound effects** | [Audio Guide](guides/audio.md) | Example: `audio_simple` |
| **Add 3D spatial audio** | [Spatial Audio Concepts](concepts/spatial-audio.md) | [Audio Guide](guides/audio.md), Example: `audio_demo` |
| **Configure audio attenuation** | [Audio API](reference/audio-api.md) | [Audio Learning Path](learning-paths/audio.md) |
| **Optimize audio performance** | [Audio Learning Path (Advanced)](learning-paths/audio.md) | [Audio API](reference/audio-api.md) |

### Input Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Handle keyboard input** | [Input Guide](guides/input.md) | [Input Concepts](concepts/input.md), Example: `input_integration` |
| **Handle mouse input** | [Input Guide](guides/input.md) | Example: `input_integration` |
| **Handle gamepad input** | [Input Guide](guides/input.md) | [Input API](reference/input-api.md) |
| **Create action mappings** | [Input API](reference/input-api.md) | [Input Learning Path](learning-paths/README.md) |
| **Build camera controllers** | [Camera API](reference/camera-api.md) | Example: `fps_camera_controller` |

### Scripting Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Add Lua scripting** | [Scripting Guide](guides/scripting.md) | [Scripting API](reference/scripting-api.md), Example: `scripting_demo` |
| **Access ECS from scripts** | [Scripting Guide](guides/scripting.md) | Example: `scripting_advanced_demo` |
| **Enable hot-reload for scripts** | [Scripting API](reference/scripting-api.md) | [Crate README](../crates/praxis_scripting/README.md) |
| **Configure script sandboxing** | [Scripting API](reference/scripting-api.md) | [Scripting Learning Path](learning-paths/scripting.md) |
| **Debug script performance** | [Scripting Learning Path (Advanced)](learning-paths/scripting.md) | [Scripting API](reference/scripting-api.md) |
| **Create in-game console** | [Console Guide](guides/console.md) | Example: `console_demo`, `scripting_console_demo` |

### Networking Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Build multiplayer games** | [Networking Guide](guides/systems/networking.md) | [Networking API](reference/networking-api.md), Example: `networking_demo` |
| **Setup client-server architecture** | [Networking Guide](guides/systems/networking.md) | [Networking Learning Path](learning-paths/networking.md) |
| **Replicate entities over network** | [Networking API](reference/networking-api.md) | [Crate README](../crates/praxis_networking/README.md) |
| **Implement lag compensation** | [Networking Learning Path (Advanced)](learning-paths/networking.md) | [Networking API](reference/networking-api.md) |
| **Handle network interpolation** | [Networking API](reference/networking-api.md) | [Networking Learning Path](learning-paths/networking.md) |
| **Monitor network performance** | [Networking API](reference/networking-api.md) | [Profiling](profiling.md) |

### Editor Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Use the editor** | [Editor Overview](editor/editor-overview.md) | Example: `editor_demo` |
| **Select entities** | [Selection System](editor/selection-system.md) | Example: `selection_demo` |
| **Use transform gizmos** | [Gizmos](editor/gizmos.md) | [Editor Overview](editor/editor-overview.md) |
| **Navigate with editor camera** | [Editor Camera](editor/editor-camera.md) | Example: `editor_camera_demo` |
| **Undo/redo operations** | [Undo/Redo](editor/undo-redo.md) | Example: `undo_redo_system_demo` |
| **Manage hierarchy** | [Hierarchy Panel](editor/hierarchy-panel.md) | [Editor Overview](editor/editor-overview.md) |
| **Browse and import assets** | [Asset Browser](editor/asset-browser.md) | [Editor Overview](editor/editor-overview.md) |
| **Edit component properties** | [Inspector](editor/inspector.md) | [Editor Overview](editor/editor-overview.md) |
| **Use menu bar and shortcuts** | [Menu Bar](editor/menu-bar.md) | [Editor Overview](editor/editor-overview.md) |
| **Customize editor panels** | [Editor Learning Path (Advanced)](learning-paths/editor.md) | [Editor API](reference/editor-api.md) |
| **Understand command pattern** | [Undo/Redo](editor/undo-redo.md) | Example: `command_system_demo`, [Crate: COMMAND_SYSTEM.md](../crates/praxis_editor/COMMAND_SYSTEM.md) |

### GUI Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Create in-game UI** | [GUI API](reference/gui-api.md) | Example: `gui_demo` |
| **Build menus** | [Menu Bar](editor/menu-bar.md) | Example: `menu_bar_demo` |
| **Use immediate mode GUI** | [GUI API](reference/gui-api.md) | [Editor Overview](editor/editor-overview.md) |

### Terrain Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Generate terrain** | [Terrain Guide](guides/terrain.md) | [Terrain API](reference/terrain-api.md), Example: `terrain_demo` |
| **Use heightmaps** | [Terrain Guide](guides/terrain.md) | [Terrain API](reference/terrain-api.md) |
| **Optimize terrain with LOD** | [Terrain API](reference/terrain-api.md) | [Spatial Optimization](guides/spatial-optimization.md) |

### Optimization Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Improve rendering performance** | [Spatial Optimization](guides/spatial-optimization.md) | [Performance Learning Path](learning-paths/performance.md) |
| **Profile CPU performance** | [Profiling](profiling.md) | [Profiling API](reference/profiling-api.md), Example: `profiling_demo` |
| **Profile GPU performance** | [Profiling API](reference/profiling-api.md) | Example: `profiling_advanced_demo` |
| **Use spatial partitioning (octree, BVH)** | [Spatial API](reference/spatial-api.md) | Example: `spatial_partitioning_demo` |
| **Implement frustum culling** | [Frustum Culling](guides/rendering/frustum-culling.md) | Example: `spatial_optimization_demo` |
| **Implement occlusion culling** | [Spatial Optimization](guides/spatial-optimization.md) | [Spatial API](reference/spatial-api.md) |
| **Optimize with GPU culling** | [GPU Culling](guides/rendering/gpu-culling.md) | Example: `gpu_culling_demo` |
| **Implement LOD system** | [LOD System](guides/rendering/lod.md) | Example: `lod_gpu_demo` |
| **Stream meshes efficiently** | Example: `mesh_streaming_demo` | [Assets Learning Path](learning-paths/assets.md) |
| **Handle 1000+ entities** | [Performance Learning Path](learning-paths/performance.md) | [Spatial Optimization](guides/spatial-optimization.md) |

### ECS & Architecture Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Understand ECS design** | [ECS Architecture](concepts/ecs-architecture.md) | Example: `ecs_integration` |
| **Learn ECS patterns** | [Architecture: ECS Patterns](architecture/ecs-patterns.md) | [ECS Architecture](concepts/ecs-architecture.md) |
| **Create components** | [Components Reference](reference/components.md) | [ECS Architecture](concepts/ecs-architecture.md) |
| **Write systems** | [ECS Architecture](concepts/ecs-architecture.md) | [Beginners Guide](beginners-guide.md) |
| **Understand engine architecture** | [Architecture](architecture.md) | [Architecture: Engine Lifecycle](architecture/engine-lifecycle.md) |
| **Understand crate organization** | [Crates Reference](reference/crates.md) | [Project Structure](getting-started/project-structure.md) |

### Development Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Run tests** | [Testing](testing.md) | `cargo test --workspace` |
| **Run benchmarks** | [Benchmarking](benchmarking.md) | `cargo bench` |
| **Configure logging** | [Logging](logging.md) | [Profiling](profiling.md) |
| **Contribute to the engine** | [Architecture](architecture.md) | [Crate README Audit](CRATE_README_AUDIT.md) |
| **Understand internal implementation** | [Internals](internals/README.md) | Crate-level documentation |

### Learning Path Tasks

| I want to... | Start here | Also see |
|--------------|-----------|----------|
| **Follow structured learning** | [Learning Paths](learning-paths/README.md) | Pick a path based on your role |
| **Learn rendering end-to-end** | [Rendering Learning Path](learning-paths/rendering.md) | Beginner → Intermediate → Advanced |
| **Learn animation end-to-end** | [Animation Learning Path](learning-paths/animation.md) | Beginner → Intermediate → Advanced |
| **Learn physics end-to-end** | [Physics Learning Path](learning-paths/physics.md) | Beginner → Intermediate → Advanced |
| **Learn networking end-to-end** | [Networking Learning Path](learning-paths/networking.md) | Beginner → Intermediate → Advanced |
| **Learn scripting end-to-end** | [Scripting Learning Path](learning-paths/scripting.md) | Beginner → Intermediate → Advanced |
| **Master editor tools** | [Editor Learning Path](learning-paths/editor.md) | Beginner → Intermediate → Advanced |
| **Optimize performance** | [Performance Learning Path](learning-paths/performance.md) | Requires completing another path first |
| **Track my progress** | [Learning Paths: Milestones](learning-paths/README.md#milestone-tracking) | Beginner/Intermediate/Advanced checklists |
| **Build a complete game** | [Learning Paths: Project Progressions](learning-paths/README.md#recommended-project-progressions) | Beginner/Intermediate/Advanced projects |

## Development

- [Testing](testing.md) - Test organization and running tests
- [Benchmarking](benchmarking.md) - Performance testing
- [Profiling](profiling.md) - Performance analysis and optimization
- [Logging](logging.md) - Logging configuration
- [Crate README Audit](CRATE_README_AUDIT.md) - Documentation completeness audit and maintenance
