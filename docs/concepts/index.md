# Concepts

Deep dives into the theoretical foundations of game engine architecture. These pages explain the "why" behind design decisions.

## Core Concepts

### Architecture
- [ECS Architecture](ecs-architecture.md) - Entity-Component-System design
- [Rendering Pipeline](rendering-pipeline.md) - Graphics pipeline flow
- [Transform Hierarchy](transform-hierarchy.md) - Scene graphs and spatial relationships

### Graphics
- [Vulkan Rendering](vulkan-rendering.md) - Low-level graphics API
- [PBR Materials](pbr-materials.md) - Physically-based rendering
- [Lighting](lighting.md) - Light types and calculations

### Simulation
- [Animation](animation.md) - Skeletal animation theory
- [Physics](physics.md) - Rigid body simulation
- [Spatial Audio](spatial-audio.md) - 3D audio positioning

### Input & Interaction
- [Input](input.md) - Input abstraction and handling

## Concept Categories

### Foundation Concepts
Essential understanding for all engine work:

- [ECS Architecture](ecs-architecture.md) - Data-oriented design
- [Transform Hierarchy](transform-hierarchy.md) - Spatial relationships
- [Rendering Pipeline](rendering-pipeline.md) - Graphics flow

### Graphics Concepts
Understanding rendering and visuals:

- [Vulkan Rendering](vulkan-rendering.md) - Modern graphics APIs
- [PBR Materials](pbr-materials.md) - Realistic materials
- [Lighting](lighting.md) - Illumination models

### Simulation Concepts
Physics, animation, and interaction:

- [Animation](animation.md) - Character animation
- [Physics](physics.md) - Physical simulation
- [Spatial Audio](spatial-audio.md) - Sound positioning

## How Concepts Work

### Theory to Practice
Each concept page:

1. **Explains the theory** - Why does this exist?
2. **Shows the math** - What are the equations?
3. **Links to implementation** - How do engines implement this?
4. **Provides examples** - See it in action

### Comparison to Guides

| Concepts | Guides |
|----------|--------|
| **Why** it works | **How** to implement |
| Theory and math | Step-by-step code |
| Background knowledge | Practical application |
| General principles | Specific features |

### Multi-Engine Perspective

Concepts are taught engine-agnostically:

- How **Praxis** approaches the problem (Rust + ECS)
- How **Unreal** solves it (C++ + OOP)
- How **Unity** implements it (C# + Components)
- How **others** might do it (alternatives)

## Learning Order

### For Beginners
Start with foundational concepts:

1. [ECS Architecture](ecs-architecture.md)
2. [Transform Hierarchy](transform-hierarchy.md)
3. [Rendering Pipeline](rendering-pipeline.md)

### For Graphics Programmers
Focus on rendering concepts:

1. [Vulkan Rendering](vulkan-rendering.md)
2. [PBR Materials](pbr-materials.md)
3. [Lighting](lighting.md)

### For Gameplay Programmers
Simulation and interaction:

1. [Physics](physics.md)
2. [Animation](animation.md)
3. [Input](input.md)

## Visual Learning

Concepts include:

- 📊 **Diagrams** - Visualize data flow and relationships
- 📐 **Equations** - Mathematical foundations
- 🎨 **Illustrations** - See the results
- 💻 **Code snippets** - Connect theory to practice

## Related Resources

- [Guides](../guides/) - Implement the concepts
- [Code Examples](../course/CODE_EXAMPLES.md) - See multiple implementations
- [Patterns](../course/patterns/) - Architectural decisions
- [Learning Paths](../learning-paths/) - Structured progressions

## Contributing

Found an explanation unclear? Have a better diagram?

- Submit an issue with feedback
- Propose improvements via pull request
- Add alternative explanations

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="ecs-architecture.html" class="md-button md-button--primary">Start with ECS</a>
  <a href="vulkan-rendering.html" class="md-button">Graphics Fundamentals</a>
</div>
