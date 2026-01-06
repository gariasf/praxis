# Guides

Task-oriented guides for implementing features with Praxis.

**New to Praxis?** Check out [Learning Paths](../learning-paths/) for structured progressions from beginner to advanced, with clear prerequisites and cross-references.

## Rendering

- [Rendering Overview](rendering.md) - Forward and deferred rendering pipelines
- [Rendering Guides](rendering/) - Comprehensive rendering documentation:
  - [Forward Rendering](rendering/forward-rendering.md) - Basic forward rendering pipeline
  - [Deferred Rendering](rendering/deferred-rendering.md) - Multi-pass rendering with G-buffer architecture
  - [HDR and Tone Mapping](rendering/hdr-tonemapping.md) - High dynamic range rendering
  - [Shadows](rendering/shadows.md) - Cascaded shadow mapping with PCF
  - [Post-Processing](rendering/post-processing.md) - Bloom, color grading, effects
  - [Environment Probes](rendering/environment-probes.md) - Image-based lighting and PBR reflections
  - [Particles](rendering/particles.md) - Particle effect examples
  - [Line Rendering](rendering/line-rendering.md) - Debug and gizmo line rendering
- [Spatial Optimization](spatial-optimization.md) - Frustum culling, LOD, and occlusion culling

## Animation

- [Animation Overview](animation.md) - Quick start and practical examples
- [Animation Guides](animation/) - Comprehensive animation documentation:
  - [Skeletal Basics](animation/skeletal-basics.md) - Core architecture and fundamentals
  - [Skeletal Animation](animation/skeletal-animation.md) - Complete skeletal animation system guide
  - [Blending](animation/blending.md) - Cross-fades, blend trees, and layered animation
  - [Advanced Features](animation/advanced-features.md) - IK, retargeting, additive blending, and root motion
  - [Advanced Integration](animation/advanced-integration.md) - Physics and scripting integration

## Simulation

- [Physics](physics.md) - Rigid body dynamics with Rapier3D
- [Audio](audio.md) - Spatial audio with Kira

## Scripting & Networking

- [Scripting](scripting.md) - Lua scripting integration
- [Networking](systems/networking.md) - Multiplayer client-server architecture

## Input & Assets

- [Input](input.md) - Keyboard, mouse, and gamepad handling
- [Assets](assets/) - Asset pipeline documentation:
  - [Assets Overview](assets/README.md) - Loading GLTF, OBJ, textures, and audio

## Related

- [Concepts](../concepts/README.md) - Understand the theory behind these systems
- [Reference](../reference/README.md) - API and configuration details
- [Editor](../editor/README.md) - Editor-specific documentation
