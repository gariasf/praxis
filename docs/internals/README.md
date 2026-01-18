# Internal Implementation Documentation

This directory contains internal implementation documentation for engine developers and contributors. These documents detail the technical implementation of specific features and subsystems.

## Purpose

Internal documentation serves to:
- Record implementation decisions and technical details
- Provide reference for future maintenance and enhancements
- Document the current state of complex subsystems

## Documents

### Graphics & Rendering
- **[advanced-lighting-implementation.md](advanced-lighting-implementation.md)** - Light probes, volumetric fog, and advanced lighting
- **[visual-feedback-systems.md](visual-feedback-systems.md)** - Line rendering, grid renderer, and selection highlighting

### Editor
- **[drag-drop-implementation.md](drag-drop-implementation.md)** - Drag-and-drop asset instantiation system
- **[viewport-integration.md](viewport-integration.md)** - Viewport panel integration with render context, gizmos, and raycasting

### Performance & Optimization
- **[profiling-implementation.md](profiling-implementation.md)** - Profiling system including CPU, GPU, memory tracking, and Chrome tracing
- **[spatial-implementation-summary.md](spatial-implementation-summary.md)** - Spatial partitioning (Octree, BVH) with ECS integration

### Historical Documentation
- **[archived-audits/](archived-audits/)** - Comprehensive audit reports from January 2026 covering all 19 crates
- **[implementation-history/](implementation-history/)** - Historical implementation notes from feature development

## Internal vs User-Facing Documentation

| Internal docs (`docs/internals/`) | User-facing docs (`docs/guides/`, etc.) |
|-----------------------------------|----------------------------------------|
| Implementation-specific details | How-to guides for engine users |
| Code structure and architecture | Conceptual explanations |
| Developer-oriented context | API reference and usage examples |

## See Also

- [Main Documentation](../README.md)
- [Architecture](../architecture.md)
- [Guides](../guides/)
