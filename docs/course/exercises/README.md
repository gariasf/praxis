# Praxis Engine Exercises

**60 specification-based exercises** designed to teach game engine development through hands-on implementation. Learn by building, not just reading!

## 🚀 Quick Start

- **New to game engines?** → [GETTING_STARTED.md](./GETTING_STARTED.md)
- **Want structured learning?** → [CATALOG.md](./CATALOG.md) (8 curated paths)
- **Ready to dive in?** → [Exercise 01: Fixed Timestep Game Loop](./01-fixed-timestep-game-loop.md)

## What You'll Learn

Each exercise provides:

- **Clear Requirements**: Detailed specification of what to build
- **Validation Criteria**: How to verify your implementation works correctly
- **Expected Behavior**: What the final result should look like
- **Performance Targets**: Benchmarks to aim for (where applicable)
- **Reference Implementations**: Solutions in Rust, C++, and Python

## Exercise Organization

Exercises are organized by subsystem and difficulty:

### 1. Core Engine (01-10)
Fundamental engine architecture, main loop, timing, and lifecycle management.

### 2. Rendering (11-20)
Vulkan rendering, pipelines, shaders, lighting, and optimization.

### 3. ECS & Scene Management (21-30)
Entity-Component-System patterns, transforms, hierarchies, and queries.

### 4. Physics & Spatial (31-40)
Physics integration, collision detection, spatial partitioning.

### 5. Assets & Resources (41-47)
Asset loading, management, caching, and streaming.

### 6. Advanced Features (48-60)
Animation, audio, scripting, networking, and editor tools.

## Difficulty Levels

- 🟢 **Beginner**: Foundation concepts, guided implementation
- 🟡 **Intermediate**: Requires understanding of subsystem interactions
- 🔴 **Advanced**: Complex systems, performance-critical code

## How to Use These Exercises

1. **Read the Specification**: Understand requirements and validation criteria
2. **Plan Your Approach**: Sketch out the architecture before coding
3. **Implement Iteratively**: Start simple, add features progressively
4. **Validate**: Test against all validation criteria
5. **Benchmark**: Compare performance against targets (where applicable)
6. **Review Reference**: Study reference implementations to learn alternative approaches

## Exercise List

| # | Title | Subsystem | Difficulty | Est. Time |
|---|-------|-----------|------------|-----------|
| 01 | Fixed Timestep Game Loop | Core | 🟢 | 2-3h |
| 02 | Frame Time Profiler | Core | 🟢 | 1-2h |
| 03 | Resource Manager Pattern | Core | 🟡 | 3-4h |
| 04 | Event System | Core | 🟡 | 2-3h |
| 05 | Multi-threaded Task Queue | Core | 🔴 | 4-6h |
| 06 | Memory Pool Allocator | Core | 🔴 | 4-5h |
| 07 | Hot-Reload System | Core | 🔴 | 5-6h |
| 08 | Configuration Management | Core | 🟢 | 2h |
| 09 | Plugin Architecture | Core | 🔴 | 6-8h |
| 10 | Crash Reporter | Core | 🟡 | 3-4h |
| 11 | Triangle Renderer | Graphics | 🟢 | 2-3h |
| 12 | Texture Loading & Sampling | Graphics | 🟢 | 2-3h |
| 13 | Basic Material System | Graphics | 🟡 | 3-4h |
| 14 | Directional Light | Graphics | 🟡 | 3-4h |
| 15 | Shadow Mapping | Graphics | 🔴 | 5-6h |
| 16 | Deferred Rendering | Graphics | 🔴 | 6-8h |
| 17 | HDR & Tone Mapping | Graphics | 🟡 | 3-4h |
| 18 | Frustum Culling | Graphics | 🟡 | 3-4h |
| 19 | GPU Instancing | Graphics | 🔴 | 4-5h |
| 20 | PBR Material | Graphics | 🔴 | 5-6h |
| 21 | Component Registration | ECS | 🟢 | 1-2h |
| 22 | System Scheduling | ECS | 🟡 | 3-4h |
| 23 | Entity Queries | ECS | 🟢 | 2-3h |
| 24 | Transform Hierarchy | ECS | 🔴 | 5-6h |
| 25 | Parent-Child Relationships | ECS | 🟡 | 3-4h |
| 26 | Component Serialization | ECS | 🟡 | 3-4h |
| 27 | Prefab System | ECS | 🔴 | 5-6h |
| 28 | Entity Archetypes | ECS | 🟡 | 3-4h |
| 29 | Change Detection | ECS | 🟡 | 2-3h |
| 30 | Scene Graph | ECS | 🔴 | 6-8h |
| 31 | AABB Collision | Physics | 🟢 | 2-3h |
| 32 | Sphere-Sphere Collision | Physics | 🟢 | 1-2h |
| 33 | Raycast System | Physics | 🟡 | 3-4h |
| 34 | Physics Integration | Physics | 🔴 | 5-6h |
| 35 | Collision Response | Physics | 🔴 | 4-5h |
| 36 | Octree Partitioning | Spatial | 🔴 | 5-6h |
| 37 | BVH Construction | Spatial | 🔴 | 6-8h |
| 38 | Spatial Queries | Spatial | 🟡 | 3-4h |
| 39 | Broad Phase Optimization | Physics | 🔴 | 4-5h |
| 40 | Continuous Collision | Physics | 🔴 | 6-8h |
| 41 | OBJ Parser | Assets | 🟡 | 4-5h |
| 42 | Texture Atlas Generator | Assets | 🟡 | 3-4h |
| 43 | Async Asset Loading | Assets | 🔴 | 5-6h |
| 44 | Asset Dependency Graph | Assets | 🔴 | 6-8h |
| 45 | LRU Cache | Assets | 🟡 | 2-3h |
| 46 | Mesh LOD Generator | Assets | 🔴 | 6-8h |
| 47 | Asset Streaming | Assets | 🔴 | 8-10h |
| 48 | Skeletal Animation | Animation | 🔴 | 6-8h |
| 49 | Animation Blending | Animation | 🔴 | 5-6h |
| 50 | IK Solver | Animation | 🔴 | 8-10h |
| 51 | Audio Spatialization | Audio | 🟡 | 3-4h |
| 52 | Audio Mixer | Audio | 🔴 | 4-5h |
| 53 | Lua Script Integration | Scripting | 🔴 | 5-6h |
| 54 | Entity Replication | Networking | 🔴 | 6-8h |
| 55 | Lag Compensation | Networking | 🔴 | 8-10h |
| 56 | Selection System | Editor | 🟡 | 3-4h |
| 57 | Undo/Redo Commands | Editor | 🔴 | 5-6h |
| 58 | Transform Gizmos | Editor | 🔴 | 6-8h |
| 59 | Procedural Noise | Procedural | 🟡 | 3-4h |
| 60 | Terrain LOD | Terrain | 🔴 | 8-10h |

## 📚 Documentation

- **[GETTING_STARTED.md](./GETTING_STARTED.md)** - Complete guide for using these exercises
- **[CATALOG.md](./CATALOG.md)** - Comprehensive catalog with 8 learning paths
- **[EXERCISE_TEMPLATE.md](./EXERCISE_TEMPLATE.md)** - Template for creating new exercises
- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - What was created and why

## 🎯 Learning Paths

Choose a path based on your goals:

1. **Engine Fundamentals** (Beginner, 16-20h) - Core concepts
2. **Graphics Programming** (Intermediate, 30-40h) - Rendering pipeline
3. **Systems Programming** (Intermediate, 28-38h) - Advanced patterns
4. **Physics & Spatial** (Advanced, 40-52h) - Simulation systems
5. **Content Pipeline** (Intermediate, 38-50h) - Asset management
6. **Animation & Character** (Advanced, 19-24h) - Character systems
7. **Multiplayer** (Expert, 16-21h) - Networked gameplay
8. **Editor Tools** (Advanced, 14-18h) - Tool development

See [CATALOG.md](./CATALOG.md) for detailed path descriptions.

## 📊 Statistics

- **Total Exercises**: 60
- **Total Time**: 220-290 hours (full completion)
- **Difficulty**: 9 beginner, 20 intermediate, 31 advanced
- **Languages**: Rust (all), C++ (40%), Python (15%)
- **Subsystems**: Core, Graphics, ECS, Physics, Assets, Animation, Audio, Scripting, Networking, Editor

## Contributing

When adding new exercises:

1. Follow the template in [EXERCISE_TEMPLATE.md](./EXERCISE_TEMPLATE.md)
2. Include all required sections
3. Provide reference implementations in at least 2 languages (Rust + one other)
4. Test validation criteria thoroughly
5. Update this README and [CATALOG.md](./CATALOG.md)

## Resources

- [Praxis Documentation](../../README.md)
- [Beginners Guide](../../beginners-guide.md)
- [Architecture Overview](../../architecture.md)
- [Crate Reference](../../reference/crates.md)
