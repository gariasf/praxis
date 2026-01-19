# Architecture Documentation

This directory contains detailed architecture documentation for the Praxis game engine, including visual diagrams and in-depth technical explanations.

## Visual Architecture Diagrams

High-level visual representations of engine architecture:

- **[Crate Dependency Graph](crate-dependency-graph.md)** - Complete visual diagram showing how the 18 workspace crates depend on each other, organized into architectural layers (Foundation, Platform, Engine, Application)

- **[Rendering Pipeline Stages](rendering-pipeline-stages.md)** - Comprehensive breakdown of the rendering pipeline showing CPU preprocessing, GPU rendering stages, and post-processing for both forward and deferred rendering paths

- **[ECS System Execution Order](ecs-system-execution-order.md)** - Detailed visualization of ECS system execution flow per frame, showing system sets, dependencies, parallelization opportunities, and data flow between systems

- **[Multiplayer Data Flow](multiplayer-data-flow.md)** - Complete client-server networking architecture showing entity replication, client-side prediction, server reconciliation, lag compensation, and interpolation/extrapolation

## Detailed Architecture Documents

In-depth technical documentation:

- **[ECS Design Patterns](ecs-patterns.md)** - Common Entity-Component-System patterns used throughout Praxis, including component composition, system ordering, and performance best practices

- **[Engine Lifecycle](engine-lifecycle.md)** - Comprehensive guide to engine initialization, main loop execution, and shutdown, covering resource management and synchronization

- **[Render Pipeline Architecture](render-pipeline.md)** - Deep dive comparing forward and deferred rendering approaches, performance characteristics, and choosing the right pipeline for your game

## Usage

These documents are designed to be used together:

1. **Understanding the Overall Structure**: Start with the [Crate Dependency Graph](crate-dependency-graph.md) to understand how subsystems are organized

2. **Frame Execution Flow**: Read [ECS System Execution Order](ecs-system-execution-order.md) to understand what happens each frame

3. **Rendering Details**: Study [Rendering Pipeline Stages](rendering-pipeline-stages.md) for the complete rendering flow

4. **Multiplayer**: Explore [Multiplayer Data Flow](multiplayer-data-flow.md) for networking architecture

5. **Deep Dives**: Use the detailed documents for specific subsystems

## Related Documentation

- [Main Architecture Overview](../architecture.md) - High-level principles and design philosophy
- [Concepts](../concepts/) - Educational explanations of core concepts
- [Guides](../guides/) - Practical implementation guides
- [Reference](../reference/) - API documentation

## Contributing

When updating architecture:

1. Update relevant diagrams to reflect changes
2. Keep visual representations accurate
3. Cross-reference related documents
4. Update examples in guides if architecture changes affect APIs
