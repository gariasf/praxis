# Internal Implementation Documentation

This directory contains internal implementation documentation for the Praxis engine. These documents detail the technical implementation of specific features and subsystems, and are intended for engine developers and contributors.

## Purpose

Internal documentation serves to:
- Record implementation decisions and technical details
- Provide reference for future maintenance and enhancements
- Document the current state of complex subsystems
- Track implementation progress and checklists

## Documents

### Animation System
- **[ANIMATION_ENHANCEMENTS_CHANGELOG.md](ANIMATION_ENHANCEMENTS_CHANGELOG.md)** - Changelog for advanced animation features including IK, retargeting, additive blending, and root motion
- **[IMPLEMENTATION_CHECKLIST.md](IMPLEMENTATION_CHECKLIST.md)** - Comprehensive checklist for advanced animation features implementation
- **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Summary of animation system enhancements implementation

### Graphics & Rendering
- **[ADVANCED_LIGHTING_IMPLEMENTATION.md](ADVANCED_LIGHTING_IMPLEMENTATION.md)** - Implementation details for light probes, volumetric fog, and advanced lighting features
- **[VISUAL_FEEDBACK_SYSTEMS.md](VISUAL_FEEDBACK_SYSTEMS.md)** - Line rendering system, grid renderer, and selection highlighting implementation

### Editor
- **[DRAG_DROP_IMPLEMENTATION.md](DRAG_DROP_IMPLEMENTATION.md)** - Drag-and-drop asset instantiation system implementation
- **[VIEWPORT_INTEGRATION.md](VIEWPORT_INTEGRATION.md)** - Viewport panel integration with render context, gizmos, and raycasting

### Performance & Optimization
- **[PROFILING_IMPLEMENTATION.md](PROFILING_IMPLEMENTATION.md)** - Profiling system implementation including CPU, GPU, memory tracking, and Chrome tracing
- **[SPATIAL_IMPLEMENTATION_SUMMARY.md](SPATIAL_IMPLEMENTATION_SUMMARY.md)** - Spatial partitioning implementation (Octree, BVH) with ECS integration

## Difference from User-Facing Documentation

**Internal docs** (`docs/internals/`):
- Implementation-specific technical details
- Code structure and architecture decisions
- Implementation checklists and status tracking
- Developer-oriented context

**User-facing docs** (`docs/`, `docs/guides/`, etc.):
- How-to guides for engine users
- Conceptual explanations of engine features
- API reference and usage examples
- Getting started tutorials

## Contributing

When adding new features to the Praxis engine:

1. Create implementation documentation in this directory during development
2. Once the feature is stable, create or update user-facing documentation in the main `docs/` directory
3. Keep internal docs updated as implementation details change
4. Archive or remove internal docs when they become outdated or the feature is fully documented elsewhere

## See Also

- [Main Documentation](../README.md) - Index of all documentation
- [Architecture](../ARCHITECTURE.md) - Overall engine architecture
- [Developer Guides](../guides/) - Guides for using the engine
