# Praxis Documentation

Welcome to the Praxis game engine documentation.

## Quick Start

- **[Getting Started](getting-started/README.md)** - Installation, setup, and first steps
- **[Beginners Guide](BEGINNERS_GUIDE.md)** - Learn Praxis concepts through hands-on examples

## Documentation Sections

### [Guides](guides/README.md)
Task-oriented tutorials for implementing features:
- [Rendering](guides/rendering.md) - Forward and deferred pipelines
- [HDR and Tone Mapping](guides/hdr-and-tonemapping.md) - High dynamic range rendering
- [Shadows](guides/shadows.md) - Cascaded shadow maps with PCF
- [Post-Processing](guides/post-processing.md) - Bloom, color grading, effects
- [Animation](animation_system.md) - Skeletal animation and blending
- [Audio](audio_system.md) - Spatial audio with Kira

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
- [Selection](editor/selection.md) - Multi-entity selection and picking
- [Undo/Redo](editor/undo-redo.md) - Command history system
- [Editor Overview](editor_system.md) - Panels and editor architecture

## Examples

Run examples to see features in action:
```bash
cargo run --example deferred_demo      # Deferred rendering
cargo run --example hdr_demo           # HDR with tone mapping
cargo run --example shadow_demo        # Cascaded shadow maps
cargo run --example physics_demo       # Rapier3D physics
cargo run --example audio_demo         # Spatial audio
cargo run --example editor_demo        # Full editor interface
```

## Development

- [Strategic Analysis](STRATEGIC_ANALYSIS_2026.md) - Project roadmap and priorities
- [Benchmarking](benchmarking.md) - Performance testing
