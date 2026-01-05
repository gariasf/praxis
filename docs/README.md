# Praxis Documentation

Welcome to the Praxis game engine documentation.

## Quick Start

- **[Getting Started](getting-started/README.md)** - Installation, setup, and first steps
- **[Beginners Guide](BEGINNERS_GUIDE.md)** - Learn Praxis concepts through hands-on examples

## Documentation Sections

### [Guides](guides/README.md)
Task-oriented tutorials for implementing features:
- [Rendering](guides/rendering.md) - Forward and deferred pipelines
- [Deferred Rendering](guides/deferred-rendering.md) - Multi-pass rendering with G-buffer
- [Environment Probes](guides/environment-probes.md) - Image-based lighting and reflections
- [HDR and Tone Mapping](guides/hdr-and-tonemapping.md) - High dynamic range rendering
- [Shadows](guides/shadows.md) - Cascaded shadow maps with PCF
- [Post-Processing](guides/post-processing.md) - Bloom, color grading, effects
- [Particles](guides/particles.md) - Practical particle effect examples
- [Spatial Optimization](guides/spatial-optimization.md) - Frustum culling, LOD, and occlusion culling
- [Animation](guides/animation.md) - Quick start guide
  - [Skeletal Animation](guides/animation/skeletal-animation.md) - Complete skeletal animation system
  - [Skeletal Basics](guides/animation/skeletal-basics.md) - Core architecture and fundamentals
  - [Blending](guides/animation/blending.md) - Cross-fades, blend trees, and layered animation
  - [Advanced Features](guides/animation/advanced-features.md) - IK, retargeting, root motion
- [Audio](guides/audio.md) - Spatial audio with Kira
- [Input](guides/input.md) - Keyboard, mouse, and gamepad handling
- [Profiling](profiling.md) - Performance analysis and optimization

For comprehensive particle system documentation, see [crates/praxis_graphics/PARTICLES.md](../crates/praxis_graphics/PARTICLES.md).

### [Concepts](concepts/README.md)
Educational explanations of engine design:
- [Architecture](ARCHITECTURE.md) - Overall engine design and crate organization
- [ECS Architecture](concepts/ecs-architecture.md) - Entity-Component-System patterns
- [Vulkan Rendering](concepts/vulkan-rendering.md) - Graphics pipeline fundamentals
- [Transform Hierarchy](concepts/transform-hierarchy.md) - Scene graphs and spatial relationships
- [PBR Materials](concepts/pbr-materials.md) - Physically-based rendering theory
- [Lighting](concepts/lighting.md) - Directional and point light systems
- [Animation](concepts/animation.md) - Skeletal animation and blending
- [Physics](concepts/physics.md) - Rigid body simulation with Rapier3D
- [Input](concepts/input.md) - Keyboard, mouse, and gamepad handling
- [Spatial Audio](concepts/spatial-audio.md) - 3D audio positioning

### [Reference](reference/README.md)
API documentation and specifications:
- [Input System](input_system.md) - Keyboard, mouse, gamepad handling
- [Camera System](camera_system.md) - Camera types and controllers
- [Mesh System](mesh_system.md) - Geometry loading and management
- [Testing](TESTING.md) - Test organization and running tests

### [Editor](editor/README.md)
Editor tools and workflows:
- [Selection System](editor/selection-system.md) - Multi-entity selection and raycast picking
- [Asset Browser](editor/asset-browser.md) - Asset management with drag-and-drop
- [Editor Camera](editor/editor-camera.md) - Orbit camera controls and focus
- [Menu Bar](editor/menu-bar.md) - Menu system with keyboard shortcuts
- [Hierarchy Panel](editor/hierarchy-panel.md) - Entity tree with drag-and-drop reparenting
- [Undo/Redo](editor/undo-redo.md) - Command history system
- [Editor Overview](editor_system.md) - Panels and editor architecture

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
cargo run --example gltf_animation_loader_demo # GLTF animation loading

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
cargo run --example lod_demo                  # Level of detail
cargo run --example environment_probe_demo    # IBL reflections
cargo run --example advanced_lighting_demo    # Advanced lighting

# Systems
cargo run --example gui_demo                  # GUI system
cargo run --example console_demo              # Debug console
cargo run --example material_demo             # Material system
cargo run --example procedural_texture_demo   # Procedural textures
cargo run --example terrain_demo              # Terrain rendering
cargo run --example scripting_demo            # Lua scripting
cargo run --example scripting_advanced_demo   # Advanced scripting
cargo run --example networking_demo           # Networking features

# Performance & Tools
cargo run --example profiling_demo            # Performance profiling
cargo run --example profiling_advanced_demo   # Advanced profiling

# Input & Camera
cargo run --example input_integration         # Input handling
cargo run --example fps_camera_controller     # FPS camera

# Low-level
cargo run --example ecs_integration           # ECS integration
cargo run --example transform_propagation_demo # Transform hierarchy
cargo run --example command_system_demo       # Command system
cargo run --example command_serialization_demo # Command serialization
cargo run --example scene_serialization_demo  # Scene serialization
cargo run --example menu_bar_demo             # Menu bar UI
```

## Development

- [Strategic Analysis](STRATEGIC_ANALYSIS_2026.md) - Project roadmap and priorities
- [Benchmarking](benchmarking.md) - Performance testing
