# Capstone Projects

This directory contains language-agnostic capstone projects designed to teach game engine development through practical, milestone-based implementations. Each project builds on fundamental concepts and can be implemented in any engine or language.

## Project Overview

| Project | Difficulty | Duration | Core Concepts |
|---------|-----------|----------|---------------|
| [01: 3D Model Viewer](01-3d-model-viewer.md) | Beginner | 1-2 weeks | Asset loading, camera controls, basic lighting |
| [02: First-Person Explorer](02-first-person-explorer.md) | Beginner | 2-3 weeks | Input handling, camera movement, collision detection |
| [03: Physics Playground](03-physics-playground.md) | Intermediate | 2-3 weeks | Physics simulation, rigid bodies, constraints |
| [04: Animation Showcase](04-animation-showcase.md) | Intermediate | 3-4 weeks | Skeletal animation, blending, state machines |
| [05: Procedural Terrain Generator](05-procedural-terrain.md) | Intermediate | 3-4 weeks | Terrain generation, LOD, procedural textures |
| [06: Particle Effects System](06-particle-effects.md) | Intermediate | 2-3 weeks | Particle systems, GPU compute, effects composition |
| [07: Multiplayer Arena](07-multiplayer-arena.md) | Advanced | 4-6 weeks | Networking, replication, lag compensation |
| [08: Scene Editor](08-scene-editor.md) | Advanced | 4-6 weeks | Editor tools, undo/redo, serialization |
| [09: Audio-Reactive Visualizer](09-audio-visualizer.md) | Intermediate | 2-3 weeks | Audio processing, visual effects, synchronization |
| [10: Mini Game Engine](10-mini-game-engine.md) | Advanced | 6-8 weeks | Engine architecture, scripting, complete pipeline |

## How to Use These Projects

### Learning Approach

1. **Start with Requirements**: Read the project specification thoroughly
2. **Follow Milestones**: Complete each milestone before moving to the next
3. **Reference Architecture**: Use the architectural guidance as a design template
4. **Implement Incrementally**: Build and test each feature independently
5. **Review References**: Study reference implementations for patterns and techniques

### Project Structure

Each project contains:

- **Overview**: Project goals and learning objectives
- **Feature Requirements**: Detailed specifications for what to build
- **Architecture Guidance**: Recommended system design and data structures
- **Milestones**: Step-by-step implementation plan with deliverables
- **Technical Challenges**: Key problems to solve and approaches
- **Reference Implementations**: Links to examples in multiple engines/languages
- **Extension Ideas**: Optional features to deepen learning

### Adapting to Your Stack

These projects are **language-agnostic** and can be implemented in:

- **Game Engines**: Unity, Unreal, Godot, custom engines
- **Graphics APIs**: Vulkan, OpenGL, DirectX, Metal, WebGPU
- **Languages**: C++, Rust, C#, Python, JavaScript/TypeScript
- **Frameworks**: SDL, GLFW, raylib, Three.js, etc.

Translate the concepts and architecture to your chosen technology stack.

### Assessment Criteria

Evaluate your implementation against:

1. **Functionality**: Does it meet all milestone requirements?
2. **Code Quality**: Is the code well-structured and maintainable?
3. **Performance**: Does it run smoothly at target frame rates?
4. **Robustness**: Does it handle errors and edge cases gracefully?
5. **Documentation**: Is the code documented and usage clear?

## Recommended Progression

### Path 1: Graphics-Focused
1. 3D Model Viewer
2. Animation Showcase
3. Particle Effects System
4. Procedural Terrain Generator

### Path 2: Gameplay-Focused
1. First-Person Explorer
2. Physics Playground
3. Multiplayer Arena
4. Scene Editor

### Path 3: Systems Programming
1. 3D Model Viewer
2. Physics Playground
3. Scene Editor
4. Mini Game Engine

### Path 4: Complete Engine Development
Complete projects 1-10 in order for comprehensive coverage of all engine subsystems.

## Additional Resources

- **Praxis Examples**: See `examples/` directory for reference implementations
- **Crate Documentation**: Each Praxis crate README contains subsystem details
- **Guides**: `docs/guides/` covers specific techniques in depth
- **Architecture**: `docs/architecture.md` explains overall engine structure

## Contributing

Found an issue or have an improvement suggestion? Projects are designed to be evergreen educational resources. Submit feedback or enhancements through the main repository.
