# Internal Implementation Documentation

This directory contains internal implementation documentation for engine developers and contributors. These documents detail the technical implementation of specific features and subsystems.

## Purpose

Internal documentation serves to:
- Record architecturally significant implementation decisions
- Provide technical context for complex subsystems
- Document design patterns and integration approaches
- Preserve historical context for understanding current architecture

## Documents

### Editor Systems
- **[viewport-integration.md](viewport-integration.md)** - Viewport panel integration with render context, gizmos, and raycasting
- **[visual-feedback-systems.md](visual-feedback-systems.md)** - Line rendering, grid renderer, and selection highlighting

### Historical Documentation
- **[archived-audits/](archived-audits/)** - Comprehensive audit reports from January 2026 covering all 19 crates

## Internal vs User-Facing Documentation

| Internal docs (`docs/internals/`) | User-facing docs (`docs/guides/`, etc.) |
|-----------------------------------|----------------------------------------|
| Implementation-specific details | How-to guides for engine users |
| Design decisions and rationale | Conceptual explanations |
| Developer-oriented context | API reference and usage examples |
| Historical snapshots | Current best practices |

## Guidelines for Internal Documentation

Documents in this directory should:
- Focus on **why** decisions were made, not just **what** was implemented
- Explain complex integration points between subsystems
- Document non-obvious design patterns
- Serve as reference for future maintenance and refactoring

Documents should **not** be:
- Pure work logs or task lists
- Duplicates of information in user-facing guides
- Step-by-step tutorials (those belong in `docs/guides/`)
- Simple API documentation (that belongs in rustdoc comments)

## See Also

- [Main Documentation](../README.md)
- [Architecture](../architecture.md)
- [Guides](../guides/)
- [Reference](../reference/)
