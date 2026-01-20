# Exercise Catalog

Complete catalog of all exercises with metadata, dependencies, and learning paths.

## Quick Reference

### By Subsystem

#### Core Engine (01-10)
| # | Title | Difficulty | Time | Key Concepts |
|---|-------|------------|------|--------------|
| 01 | Fixed Timestep Game Loop | 🟢 | 2-3h | Game loops, timing, interpolation |
| 02 | Frame Time Profiler | 🟢 | 1-2h | Performance measurement, statistics |
| 03 | Resource Manager Pattern | 🟡 | 3-4h | Resource management, handles, caching |
| 04 | Event System | 🟡 | 2-3h | Pub-sub, type-safe events |
| 05 | Multi-threaded Task Queue | 🔴 | 4-6h | Parallelism, work stealing |
| 06 | Memory Pool Allocator | 🔴 | 4-5h | Custom allocation, performance |
| 07 | Hot-Reload System | 🔴 | 5-6h | File watching, dynamic loading |
| 08 | Configuration Management | 🟢 | 2h | Serialization, settings |
| 09 | Plugin Architecture | 🔴 | 6-8h | Dynamic loading, API design |
| 10 | Crash Reporter | 🟡 | 3-4h | Error handling, diagnostics |

#### Graphics & Rendering (11-20)
| # | Title | Difficulty | Time | Key Concepts |
|---|-------|------------|------|--------------|
| 11 | Triangle Renderer | 🟢 | 2-3h | Vulkan basics, pipelines, shaders |
| 12 | Texture Loading & Sampling | 🟢 | 2-3h | Texture management, samplers |
| 13 | Basic Material System | 🟡 | 3-4h | Material properties, uniform buffers |
| 14 | Directional Light | 🟡 | 3-4h | Lighting math, Phong shading |
| 15 | Shadow Mapping | 🔴 | 5-6h | Depth buffers, shadow techniques |
| 16 | Deferred Rendering | 🔴 | 6-8h | G-buffer, multi-pass rendering |
| 17 | HDR & Tone Mapping | 🟡 | 3-4h | Color spaces, tone mapping operators |
| 18 | Frustum Culling | 🟡 | 3-4h | Spatial queries, optimization |
| 19 | GPU Instancing | 🔴 | 4-5h | Instanced rendering, performance |
| 20 | PBR Material | 🔴 | 5-6h | Physically based rendering, BRDF |

#### ECS & Scene Management (21-30)
| # | Title | Difficulty | Time | Key Concepts |
|---|-------|------------|------|--------------|
| 21 | Component Registration | 🟢 | 1-2h | ECS basics, type systems |
| 22 | System Scheduling | 🟡 | 3-4h | System ordering, dependencies |
| 23 | Entity Queries | 🟢 | 2-3h | Query patterns, iteration |
| 24 | Transform Hierarchy | 🔴 | 5-6h | Transform propagation, matrices |
| 25 | Parent-Child Relationships | 🟡 | 3-4h | Hierarchy management |
| 26 | Component Serialization | 🟡 | 3-4h | Save/load, data formats |
| 27 | Prefab System | 🔴 | 5-6h | Templates, instantiation |
| 28 | Entity Archetypes | 🟡 | 3-4h | ECS optimization, archetypes |
| 29 | Change Detection | 🟡 | 2-3h | Dirty flags, reactivity |
| 30 | Scene Graph | 🔴 | 6-8h | Scene organization, traversal |

#### Physics & Spatial (31-40)
| # | Title | Difficulty | Time | Key Concepts |
|---|-------|------------|------|--------------|
| 31 | AABB Collision | 🟢 | 2-3h | Bounding volumes, intersection tests |
| 32 | Sphere-Sphere Collision | 🟢 | 1-2h | Collision math, response |
| 33 | Raycast System | 🟡 | 3-4h | Ray intersection, queries |
| 34 | Physics Integration | 🔴 | 5-6h | Rigid body dynamics, integration |
| 35 | Collision Response | 🔴 | 4-5h | Impulse resolution, friction |
| 36 | Octree Partitioning | 🔴 | 5-6h | Spatial data structures |
| 37 | BVH Construction | 🔴 | 6-8h | Bounding volume hierarchies |
| 38 | Spatial Queries | 🟡 | 3-4h | Range queries, k-NN |
| 39 | Broad Phase Optimization | 🔴 | 4-5h | Collision detection optimization |
| 40 | Continuous Collision | 🔴 | 6-8h | Swept collision, TOI |

#### Assets & Resources (41-47)
| # | Title | Difficulty | Time | Key Concepts |
|---|-------|------------|------|--------------|
| 41 | OBJ Parser | 🟡 | 4-5h | File parsing, mesh data |
| 42 | Texture Atlas Generator | 🟡 | 3-4h | Texture packing, optimization |
| 43 | Async Asset Loading | 🔴 | 5-6h | Asynchronous I/O, futures |
| 44 | Asset Dependency Graph | 🔴 | 6-8h | Dependency tracking, loading order |
| 45 | LRU Cache | 🟡 | 2-3h | Cache eviction, data structures |
| 46 | Mesh LOD Generator | 🔴 | 6-8h | Level of detail, mesh simplification |
| 47 | Asset Streaming | 🔴 | 8-10h | Streaming, memory management |

#### Advanced Features (48-60)
| # | Title | Difficulty | Time | Key Concepts |
|---|-------|------------|------|--------------|
| 48 | Skeletal Animation | 🔴 | 6-8h | Bone hierarchies, skinning |
| 49 | Animation Blending | 🔴 | 5-6h | Blend trees, state machines |
| 50 | IK Solver | 🔴 | 8-10h | Inverse kinematics, constraints |
| 51 | Audio Spatialization | 🟡 | 3-4h | 3D audio, attenuation |
| 52 | Audio Mixer | 🔴 | 4-5h | Audio routing, effects |
| 53 | Lua Script Integration | 🔴 | 5-6h | Scripting, FFI, sandboxing |
| 54 | Entity Replication | 🔴 | 6-8h | Networking, state sync |
| 55 | Lag Compensation | 🔴 | 8-10h | Client prediction, rewind |
| 56 | Selection System | 🟡 | 3-4h | Editor tools, picking |
| 57 | Undo/Redo Commands | 🔴 | 5-6h | Command pattern, history |
| 58 | Transform Gizmos | 🔴 | 6-8h | Editor manipulation, handles |
| 59 | Procedural Noise | 🟡 | 3-4h | Noise functions, generation |
| 60 | Terrain LOD | 🔴 | 8-10h | Terrain rendering, chunking |

## Learning Paths

### Path 1: Engine Fundamentals (Beginner)
Core concepts for understanding game engine architecture.

1. Exercise 01: Fixed Timestep Game Loop
2. Exercise 02: Frame Time Profiler
3. Exercise 08: Configuration Management
4. Exercise 21: Component Registration
5. Exercise 23: Entity Queries
6. Exercise 31: AABB Collision
7. Exercise 32: Sphere-Sphere Collision
8. Exercise 11: Triangle Renderer
9. Exercise 12: Texture Loading & Sampling

**Est. Total Time**: 16-20 hours

### Path 2: Graphics Programming (Intermediate)
Focus on rendering and visual effects.

**Prerequisites**: Path 1 or equivalent experience

1. Exercise 13: Basic Material System
2. Exercise 14: Directional Light
3. Exercise 17: HDR & Tone Mapping
4. Exercise 18: Frustum Culling
5. Exercise 15: Shadow Mapping
6. Exercise 19: GPU Instancing
7. Exercise 20: PBR Material
8. Exercise 16: Deferred Rendering

**Est. Total Time**: 30-40 hours

### Path 3: Systems Programming (Intermediate)
Advanced engine systems and patterns.

**Prerequisites**: Path 1

1. Exercise 03: Resource Manager Pattern
2. Exercise 04: Event System
3. Exercise 22: System Scheduling
4. Exercise 24: Transform Hierarchy
5. Exercise 05: Multi-threaded Task Queue
6. Exercise 06: Memory Pool Allocator
7. Exercise 07: Hot-Reload System

**Est. Total Time**: 28-38 hours

### Path 4: Physics & Spatial (Advanced)
Physics simulation and spatial optimization.

**Prerequisites**: Path 1, basic physics knowledge

1. Exercise 33: Raycast System
2. Exercise 34: Physics Integration
3. Exercise 35: Collision Response
4. Exercise 36: Octree Partitioning
5. Exercise 38: Spatial Queries
6. Exercise 37: BVH Construction
7. Exercise 39: Broad Phase Optimization
8. Exercise 40: Continuous Collision

**Est. Total Time**: 40-52 hours

### Path 5: Content Pipeline (Intermediate)
Asset loading and management.

**Prerequisites**: Path 1

1. Exercise 41: OBJ Parser
2. Exercise 45: LRU Cache
3. Exercise 42: Texture Atlas Generator
4. Exercise 43: Async Asset Loading
5. Exercise 44: Asset Dependency Graph
6. Exercise 46: Mesh LOD Generator
7. Exercise 47: Asset Streaming

**Est. Total Time**: 38-50 hours

### Path 6: Animation & Character (Advanced)
Character animation systems.

**Prerequisites**: Path 1, Exercise 24

1. Exercise 48: Skeletal Animation
2. Exercise 49: Animation Blending
3. Exercise 50: IK Solver

**Est. Total Time**: 19-24 hours

### Path 7: Multiplayer (Expert)
Networked gameplay systems.

**Prerequisites**: Path 1, networking fundamentals

1. Exercise 04: Event System
2. Exercise 54: Entity Replication
3. Exercise 55: Lag Compensation

**Est. Total Time**: 16-21 hours

### Path 8: Editor Tools (Advanced)
Building editor functionality.

**Prerequisites**: Path 2

1. Exercise 56: Selection System
2. Exercise 57: Undo/Redo Commands
3. Exercise 58: Transform Gizmos

**Est. Total Time**: 14-18 hours

## Exercise Dependencies

### Dependency Graph

```
01 (Game Loop) → 02 (Profiler)
                → 34 (Physics Integration)
                → 48 (Skeletal Animation)

03 (Resource Manager) → 41 (OBJ Parser)
                       → 43 (Async Asset Loading)
                       → 45 (LRU Cache)

04 (Event System) → 54 (Entity Replication)

21 (Component Registration) → 22 (System Scheduling)
                            → 23 (Entity Queries)
                            → 24 (Transform Hierarchy)
                            → 25 (Parent-Child)

24 (Transform Hierarchy) → 48 (Skeletal Animation)
                         → 58 (Transform Gizmos)

11 (Triangle Renderer) → 12 (Texture Loading)
                       → 13 (Material System)
                       → 14 (Directional Light)

14 (Directional Light) → 15 (Shadow Mapping)
                       → 20 (PBR Material)

31 (AABB Collision) → 34 (Physics Integration)
                    → 36 (Octree)
                    → 37 (BVH)

36 (Octree) → 18 (Frustum Culling)
           → 38 (Spatial Queries)

48 (Skeletal Animation) → 49 (Animation Blending)
                        → 50 (IK Solver)

54 (Entity Replication) → 55 (Lag Compensation)
```

## By Difficulty

### Beginner (🟢)
Quick wins for learning fundamentals: 01, 02, 08, 11, 12, 21, 23, 31, 32

### Intermediate (🟡)
Requires solid understanding: 03, 04, 10, 13, 14, 17, 18, 22, 25, 26, 28, 29, 33, 38, 41, 42, 45, 51, 56, 59

### Advanced (🔴)
Complex systems requiring experience: 05, 06, 07, 09, 15, 16, 19, 20, 24, 27, 30, 34, 35, 36, 37, 39, 40, 43, 44, 46, 47, 48, 49, 50, 52, 53, 54, 55, 57, 58, 60

## By Estimated Time

### Quick (1-3h)
02, 08, 21, 23, 29, 32, 38, 45, 51, 56, 59

### Medium (3-6h)
01, 03, 04, 06, 10, 13, 14, 15, 17, 18, 22, 24, 25, 26, 28, 31, 33, 34, 35, 39, 41, 42, 43, 49, 52, 53, 54, 57, 58

### Long (6-10h)
05, 07, 09, 16, 19, 20, 27, 30, 36, 37, 40, 44, 46, 48, 55, 60

### Very Long (10+h)
47, 50

## Prerequisites

### Mathematical Foundation
- Linear algebra (vectors, matrices, quaternions): Required for 11+, 24, 31-40
- Calculus basics (derivatives, integration): Helpful for 34, 35, 40
- Probability/statistics: Helpful for 02, 59

### Programming Concepts
- Basic Rust: All exercises
- Unsafe Rust: 05, 06, 45 (optional)
- Async/await: 43, 54, 55
- Trait system: 03, 21-30
- Generics: 03, 21-30
- FFI: 53

### Domain Knowledge
- 3D graphics basics: 11-20
- Game loops: 01 (then builds on itself)
- ECS patterns: 21 (then builds on itself)
- Physics basics: 31-40
- Networking basics: 54-55

## Tool Requirements

### Required
- Rust toolchain (cargo, rustc)
- Code editor (VS Code, RustRover, etc.)
- Git

### Per Exercise Domain
- **Graphics (11-20)**: Vulkan SDK, GPU with Vulkan support
- **Networking (54-55)**: Network simulation tools (optional)
- **Audio (51-52)**: Audio device, headphones recommended
- **Scripting (53)**: Lua knowledge helpful

## Validation Tools

### Automated Testing
Most exercises include unit tests. Run with:
```bash
cargo test --test exercise_XX
```

### Performance Benchmarking
Exercises with performance targets:
```bash
cargo bench --bench exercise_XX
```

### Visual Validation
Graphics exercises require visual inspection:
- Run example program
- Compare with reference screenshots
- Check for artifacts

## Community & Support

### Getting Help
1. Review exercise hints section
2. Check reference implementations (but try first!)
3. Consult related resources links
4. Review Praxis codebase for patterns

### Sharing Solutions
- Create GitHub repo with your implementations
- Blog about your learning process
- Contribute improvements to exercises

## Exercise Authoring Guide

When creating new exercises, follow the template and include:

1. **Clear Learning Objectives**: What will students learn?
2. **Detailed Requirements**: Functional and non-functional
3. **Validation Criteria**: How to verify correctness
4. **Test Cases**: Concrete examples
5. **Performance Targets**: Measurable goals
6. **Hints & Guidance**: Help without spoiling
7. **Reference Implementations**: At least Rust + one other language
8. **Related Resources**: Links to learn more
9. **Next Steps**: Where to go after completing

## Statistics

- **Total Exercises**: 60
- **Total Estimated Time**: 220-290 hours (full completion)
- **Beginner Exercises**: 9 (15%)
- **Intermediate Exercises**: 20 (33%)
- **Advanced Exercises**: 31 (52%)
- **Subsystems Covered**: 9
- **Languages**: Rust (all), C++ (most), Python (some)

## Updates & Maintenance

This catalog is maintained alongside the exercises. When adding new exercises:

1. Update this catalog
2. Update main README.md
3. Add to appropriate learning path
4. Update dependency graph
5. Test all links and references
