# Universal Game Engine Patterns

This directory documents fundamental patterns and approaches used across game engines, independent of any specific implementation. These patterns represent decades of collective industry knowledge and trade-offs that apply regardless of programming language or engine architecture.

## Purpose

The documents in this directory teach **concepts and trade-offs**, not specific implementations. Each pattern is presented with:

- **What it is**: Core concept and variants
- **Why it exists**: Problems it solves
- **Trade-offs**: Strengths, weaknesses, and when to use each variant
- **Examples**: Conceptual examples across different engines/languages
- **Further reading**: Academic papers and industry resources

## Pattern Categories

### Temporal Patterns

- **[Game Loop Patterns](game-loop-patterns.md)**: Fixed timestep, variable timestep, semi-fixed timestep
  - How engines handle time progression
  - Determinism vs. smoothness trade-offs
  - Integration with rendering and physics

### Data Organization Patterns

- **[Component Storage Strategies](component-storage-strategies.md)**: Table-based, archetype, sparse set
  - How ECS systems organize component data
  - Memory layout and cache efficiency
  - Iteration vs. random access trade-offs

### Rendering Patterns

- **[Rendering Architecture Patterns](rendering-architecture-patterns.md)**: Forward, deferred, forward+, tiled
  - How engines organize draw calls and lighting
  - Trade-offs between flexibility and performance
  - Modern GPU-driven approaches

### Memory Management Patterns

- **[Memory Management Approaches](memory-management-approaches.md)**: Manual, reference counting, garbage collection, ownership
  - How engines manage object lifetime
  - Performance vs. safety trade-offs
  - Language-specific considerations

## How to Use These Documents

1. **Learning**: Read sequentially to understand fundamental engine architecture choices
2. **Decision-making**: Compare patterns when designing your own engine
3. **Analysis**: Use as framework to understand existing engines
4. **Teaching**: Reference when explaining engine architecture concepts

## What This Is Not

These documents are **not**:

- Praxis-specific implementation guides (see `/docs/guides/`)
- Code tutorials (see `/docs/course/modules/`)
- API documentation (see `/docs/reference/`)
- Optimization guides (see `/docs/performance_profiling_guide.md`)

## Contributing

When adding new patterns:

- Focus on **universal principles**, not implementation details
- Include **multiple variants** with clear trade-offs
- Provide **cross-engine examples** (Unity, Unreal, Godot, custom engines)
- Cite **academic sources** where applicable
- Keep language and engine agnostic where possible

## Further Reading

- **Game Engine Architecture** by Jason Gregory
- **Game Programming Patterns** by Robert Nystrom
- **Data-Oriented Design** by Richard Fabian
- GDC talks on engine architecture
- SIGGRAPH papers on rendering techniques
