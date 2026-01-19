# Reference

API documentation, specifications, and configuration reference.

## Core Reference

- [Crates](crates.md) - All workspace crates, their purposes, and dependencies
- [Crate README Index](crate-readme-index.md) - Quick reference to all crate documentation
- [Components](components.md) - ECS components reference
- [Shaders](shaders.md) - Shader bindings and conventions
- [Configuration](configuration.md) - Configurable constants and settings

## File Formats

- [Scene Format](scene-format.md) - Scene file specification and serialization

## System APIs

Comprehensive API reference for each subsystem:

### Core Systems

- [Rendering API](rendering-api.md) - Vulkan rendering, meshes, materials, lighting, post-processing
- [Camera API](camera-api.md) - Camera types, projections, and controllers
- [Physics API](physics-api.md) - Rigid bodies, colliders, forces, and collision detection
- [Input API](input-api.md) - Keyboard, mouse, gamepad handling and action mapping

### Scene & Animation

- [Animation API](animation-api.md) - Skeletal animation, blending, IK, and root motion
- [Mesh API](mesh-api.md) - Geometry loading and management

### Audio & Media

- [Audio API](audio-api.md) - Audio playback and spatial audio

### Assets & Generation

- [Procedural Textures API](procedural-textures-api.md) - GPU-accelerated texture generation
- [Terrain API](terrain-api.md) - Heightmap terrain, LOD, materials, and vegetation

### Optimization

- [Spatial API](spatial-api.md) - Culling, LOD, octree, BVH, and occlusion queries
- [Profiling API](profiling-api.md) - CPU/GPU profiling, memory tracking, and bottleneck detection

### Multiplayer & Scripting

- [Networking API](networking-api.md) - Client-server, entity replication, and lag compensation
- [Scripting API](scripting-api.md) - Lua integration, ECS access, and hot-reload

### Editor

- [Editor API](editor-api.md) - Selection, undo/redo, gizmos, and panels
- [GUI API](gui-api.md) - GUI system and widgets

## Rustdoc API Documentation

For complete API documentation with all types and methods:

```bash
cargo doc --workspace --no-deps --open
```

## Related

- [Guides](../guides/README.md) - How to use these APIs
- [Concepts](../concepts/README.md) - Theory behind the APIs
- [Learning Paths](../learning-paths/README.md) - Structured learning for specific topics
