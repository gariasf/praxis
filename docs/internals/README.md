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
- **[animation-enhancements-changelog.md](animation-enhancements-changelog.md)** - Changelog for advanced animation features including IK, retargeting, additive blending, and root motion
- **[implementation-checklist.md](implementation-checklist.md)** - Comprehensive checklist for advanced animation features implementation
- **[implementation-summary.md](implementation-summary.md)** - Summary of animation system enhancements implementation

### Graphics & Rendering
- **[advanced-lighting-implementation.md](advanced-lighting-implementation.md)** - Implementation details for light probes, volumetric fog, and advanced lighting features
- **[visual-feedback-systems.md](visual-feedback-systems.md)** - Line rendering system, grid renderer, and selection highlighting implementation

### Editor
- **[drag-drop-implementation.md](drag-drop-implementation.md)** - Drag-and-drop asset instantiation system implementation
- **[viewport-integration.md](viewport-integration.md)** - Viewport panel integration with render context, gizmos, and raycasting

### Performance & Optimization
- **[profiling-implementation.md](profiling-implementation.md)** - Profiling system implementation including CPU, GPU, memory tracking, and Chrome tracing
- **[spatial-implementation-summary.md](spatial-implementation-summary.md)** - Spatial partitioning implementation (Octree, BVH) with ECS integration

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
