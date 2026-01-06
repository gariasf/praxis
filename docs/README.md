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
- [Spatial Optimization](guides/spatial-optimization.md) - Frustum culling, LOD, and occlusion culling

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

**Systems**
- [Audio](guides/audio.md) - Spatial audio with Kira
- [Physics](guides/physics.md) - Rigid body dynamics with Rapier3D
- [Input](guides/input.md) - Keyboard, mouse, and gamepad handling
- [Scripting](guides/scripting.md) - Lua scripting integration
- [Terrain](guides/terrain.md) - Terrain generation and rendering
- [Networking](guides/systems/networking.md) - Multiplayer client-server architecture

For comprehensive particle system documentation, see [crates/praxis_graphics/PARTICLES.md](../crates/praxis_graphics/PARTICLES.md).

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

### [Audit](audit/)
Technical audits and analysis for each crate - useful for understanding implementation details and improvement opportunities.

### [Internals](internals/)
Implementation documentation for engine developers and contributors.

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
cargo run --example particles_demo            # Particle system effects
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
cargo run --example networking_demo           # Networking features

# Performance & Tools
cargo run --example profiling_demo            # Performance profiling
cargo run --example profiling_advanced_demo   # Advanced profiling
```

## Development

- [Testing](testing.md) - Test organization and running tests
- [Benchmarking](benchmarking.md) - Performance testing
- [Profiling](profiling.md) - Performance analysis and optimization
- [Logging](logging.md) - Logging configuration
