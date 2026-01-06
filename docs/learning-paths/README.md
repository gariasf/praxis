# Learning Paths

Structured progressions for mastering Praxis engine subsystems, organized by skill level with clear prerequisites and learning outcomes.

**Visual Guide**: See [Learning Paths Roadmap](roadmap.md) for visual progressions and project-specific recommendations.

**Glossary**: Check [Learning Paths Glossary](glossary.md) for definitions of terms used throughout.

---

## Quick Start by Role

| Role | Recommended Paths | Order |
|------|------------------|-------|
| **Game Developer** | Rendering → Physics → Animation → Scripting | Start with [Rendering](rendering.md) (Beginner) |
| **Graphics Programmer** | Rendering (All) → Performance | Start with [Rendering](rendering.md) (All levels) |
| **Gameplay Programmer** | Scripting → Physics → Animation | Start with [Scripting](scripting.md) |
| **Multiplayer Developer** | Networking → Physics | Start with [Networking](networking.md) |
| **Tools Developer** | Editor → Assets | Start with [Editor](editor.md) |

### By Experience Level

| Experience | Start Here |
|------------|------------|
| **New to Game Engines** | [Rendering Path](rendering.md) (Beginner) |
| **Familiar with Unity/Unreal** | [Beginner's Guide](../beginners-guide.md) → Any path (Intermediate) |
| **Experienced Game Developer** | Skip to Intermediate/Advanced in paths of interest |
| **Engine Contributor** | [Architecture](../architecture.md) → Advanced sections |

---

## Path Summaries

### Core Systems

| Path | Time | Difficulty | Key Topics |
|------|------|------------|------------|
| [**Rendering**](rendering.md) | 3-6 weeks | High | Forward/deferred rendering, PBR, shadows, HDR, post-processing |
| [**Animation**](animation.md) | 2-4 weeks | Medium-High | Skeletal animation, blending, blend trees, IK, retargeting |
| [**Physics**](physics.md) | 2-3 weeks | Medium | Rigid bodies, colliders, events, raycasting, character controllers |
| [**Scripting**](scripting.md) | 1-2 weeks | Medium | Lua integration, ECS access, hot-reload, sandboxing |
| [**Networking**](networking.md) | 2-3 weeks | High | Client-server, replication, interpolation, lag compensation |

### Supporting Systems

| Path | Time | Difficulty | Key Topics |
|------|------|------------|------------|
| [**Audio**](audio.md) | 1 week | Low | Sound playback, spatial audio, attenuation, pooling |
| [**Editor**](editor.md) | 1-2 weeks | Medium | Selection, hierarchy, gizmos, undo/redo, custom panels |
| [**Assets**](assets.md) | 4-6 days | Low-Medium | GLTF loading, texture management, custom loaders |

### Cross-Cutting

| Path | Time | Difficulty | Prerequisites |
|------|------|------------|---------------|
| [**Performance**](performance.md) | 1-2 weeks | High | Complete at least one other path first |

---

## Available Paths (Detailed)

Each path is structured as:
- **Beginner**: Core concepts and basic usage
- **Intermediate**: Integration, optimization, and common patterns
- **Advanced**: Architecture, custom extensions, and performance tuning

### Rendering Path

**Goal**: Master the graphics pipeline from basic rendering to advanced techniques.

#### Beginner: Forward Rendering Fundamentals
**Prerequisites**: None (start here!)

**Read First**:
- [Vulkan Rendering](../concepts/vulkan-rendering.md) - Understand the graphics pipeline
- [PBR Materials](../concepts/pbr-materials.md) - Physically-based rendering theory
- [Lighting](../concepts/lighting.md) - Light types and calculations

**Practical Guides**:
1. [Rendering Overview](../guides/rendering.md) - Forward pipeline basics
2. [Beginner's Guide: Rendering Pipeline](../beginners-guide.md#rendering-pipeline-flow)
3. Run `cargo run --example scene_demo`

**Learning Outcomes**:
- Understand Vulkan rendering flow
- Create basic meshes and materials
- Add directional and point lights

#### Intermediate: Advanced Rendering Techniques
**Prerequisites**: Beginner rendering complete

**Practical Guides**:
1. [Deferred Rendering](../guides/rendering/deferred-rendering.md) - Multi-pass G-buffer pipeline
2. [HDR and Tone Mapping](../guides/rendering/hdr-tonemapping.md) - High dynamic range
3. [Shadows](../guides/rendering/shadows.md) - Cascaded shadow maps
4. [Environment Probes](../guides/rendering/environment-probes.md) - Image-based lighting
5. [Post-Processing](../guides/rendering/post-processing.md) - Bloom, color grading

**Examples**: `advanced_lighting_demo`, `environment_probe_demo`, `material_demo`

#### Advanced: Custom Pipeline Development
**Prerequisites**: Intermediate rendering mastery

**Topics**:
- [Architecture: Render Pipeline](../architecture/render-pipeline.md)
- [Shaders Reference](../reference/shaders.md)
- Custom shader development
- GPU-driven rendering techniques

---

### Animation Path

**Goal**: Create lifelike character movement from basic skeletal animation to advanced techniques.

#### Beginner: Skeletal Animation Basics
**Prerequisites**: Basic understanding of transforms

**Read First**:
- [Animation Concepts](../concepts/animation.md)
- [Transform Hierarchy](../concepts/transform-hierarchy.md)

**Practical Guides**:
1. [Animation Overview](../guides/animation/README.md)
2. [Skeletal Basics](../guides/animation/skeletal-basics.md)
3. Run `cargo run --example skeletal_animation_demo`

**Learning Outcomes**:
- Load skeletal meshes from GLTF
- Play animation clips
- Understand skeleton hierarchy

#### Intermediate: Animation Blending & Control
**Prerequisites**: Beginner animation complete

**Practical Guides**:
1. [Blending Guide](../guides/animation/blending.md) - Cross-fades and blend trees
2. Run `cargo run --example animation_blending_demo`

**Learning Outcomes**:
- Cross-fade between animations
- Build blend trees (walk → run → sprint)
- Layer animations (upper body + lower body)
- Create animation state machines

#### Advanced: IK, Retargeting, and Root Motion
**Prerequisites**: Intermediate animation mastery

**Topics**: [Advanced Features](../guides/animation/advanced-features.md)
- Inverse kinematics (IK)
- Animation retargeting
- Additive blending
- Root motion

---

### Physics Path

**Goal**: Create realistic physics simulations using Rapier3D integration.

#### Beginner: Rigid Body Fundamentals
**Prerequisites**: Basic understanding of transforms

**Read First**: [Physics Concepts](../concepts/physics.md)

**Practical Guides**:
1. [Physics Guide](../guides/physics.md)
2. [Beginner's Guide: Physics System](../beginners-guide.md#physics-system)

**Learning Outcomes**:
- Create dynamic, static, and kinematic bodies
- Add colliders (sphere, box, capsule)
- Configure physics properties
- Understand ECS-physics sync

#### Intermediate: Collisions and Interactions
- Collision events and queries
- Raycasting for gameplay
- Character controllers
- Joints and constraints

#### Advanced: Custom Integration
- Advanced joint configurations
- Physics debugging and profiling
- Ragdoll integration with animation

---

### Scripting Path

**Goal**: Add runtime flexibility with Lua scripting and hot-reload capabilities.

#### Beginner: Lua Basics
**Prerequisites**: None

**Practical Guides**:
1. [Scripting Guide](../guides/scripting.md)
2. Run `cargo run --example scripting_demo`

**Learning Outcomes**:
- Setup scripting context
- Load and execute Lua scripts
- Call Lua functions from Rust
- Pass data between Rust and Lua

#### Intermediate: ECS Integration
**Prerequisites**: Beginner scripting + ECS understanding

**Read First**: [ECS Architecture](../concepts/ecs-architecture.md)

**Learning Outcomes**:
- Access entities from Lua
- Query and modify components
- Create game logic in Lua

#### Advanced: Hot-Reload and Performance
- Hot-reload configuration
- Sandboxing levels (security)
- Performance monitoring
- Script debugging techniques

---

### Networking Path

**Goal**: Build multiplayer games with client-server architecture and lag compensation.

#### Beginner: Client-Server Setup
**Prerequisites**: Basic ECS understanding

**Practical Guides**:
1. [Networking Guide](../guides/systems/networking.md)
2. Run `cargo run --example networking_demo`

**Learning Outcomes**:
- Setup server and client
- Establish connections
- Handle basic message passing

#### Intermediate: Entity Replication
- Component registration
- Automatic synchronization
- Transform interpolation
- Bandwidth optimization

#### Advanced: Lag Compensation
- Server-side rewind
- Client prediction
- Input reconciliation
- Network profiling

---

### Audio Path

**Goal**: Create immersive soundscapes with spatial audio.

**Practical Guides**: [Audio Guide](../guides/audio.md)

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 2-3 days | Audio playback, volume control |
| Intermediate | 2-3 days | Spatial positioning, attenuation |
| Advanced | 2-3 days | Pooling, LOD, optimization |

**Examples**: `audio_simple`, `audio_demo`

---

### Editor Path

**Goal**: Master the editor tools for level design and debugging.

**Documentation**: [Editor Overview](../editor/README.md)

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 3-4 days | Navigation, selection, hierarchy |
| Intermediate | 4-5 days | Asset browser, gizmos, scenes |
| Advanced | 5-6 days | Undo/redo, custom panels, extensions |

**Examples**: `editor_demo`, `selection_demo`, `undo_redo_system_demo`

---

### Assets Path

**Goal**: Master the asset pipeline for efficient resource management.

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 2 days | Loading meshes, textures, audio |
| Intermediate | 2 days | GLTF scenes, skeletal meshes |
| Advanced | 2 days | Custom loaders, hot-reload |

---

## Learning Sequences

### 4-Week Game Developer Fast Track
```
Week 1: Rendering (Beginner) + Input basics
Week 2: Physics (Beginner) + Animation (Beginner)
Week 3: Scripting (Beginner-Intermediate)
Week 4: Build small game project
```

### 8-Week Complete Mastery
```
Week 1-2: Rendering (Beginner + Intermediate)
Week 3: Physics (Beginner + Intermediate)
Week 4-5: Animation (All levels)
Week 6: Scripting (All levels)
Week 7: Networking or Editor (based on goals)
Week 8: Performance optimization + project polish
```

### 2-Week Graphics Focus
```
Week 1: Rendering (Beginner + Intermediate)
Week 2: Rendering (Advanced) + Performance
```

---

## Milestone Tracking

### Beginner Milestones
- [ ] Render 3D scene with lighting (Rendering Beginner)
- [ ] Create physics simulation (Physics Beginner)
- [ ] Play character animations (Animation Beginner)
- [ ] Execute Lua scripts (Scripting Beginner)

### Intermediate Milestones
- [ ] Implement deferred renderer with HDR (Rendering Intermediate)
- [ ] Build character controller (Physics Intermediate)
- [ ] Create animation state machine (Animation Intermediate)
- [ ] Access ECS from Lua (Scripting Intermediate)
- [ ] Setup multiplayer replication (Networking Intermediate)

### Advanced Milestones
- [ ] Create custom rendering pipeline (Rendering Advanced)
- [ ] Implement ragdoll physics (Physics Advanced)
- [ ] Build IK system (Animation Advanced)
- [ ] Hot-reload Lua scripts (Scripting Advanced)
- [ ] Lag compensation working (Networking Advanced)
- [ ] 60+ FPS with 100+ entities (Performance)

---

## Time Investment

### By Level
- **All Beginner**: 60-80 hours (1-2 months part-time)
- **All Intermediate**: 100-140 hours (2-3 months part-time)
- **All Advanced**: 100-150 hours (2-4 months part-time)

### Complete Engine Mastery
**Total**: 400-600 hours (6-12 months part-time, 3-6 months full-time)

---

## How to Use Learning Paths

1. **Choose your path** based on your role/goals above
2. **Check prerequisites** before starting each level
3. **Follow the progression** (don't skip levels)
4. **Run examples** immediately after reading theory
5. **Complete exercises** to reinforce learning
6. **Track milestones** to measure progress
7. **Cross-reference** related systems when mentioned

---

## Navigation

- [Beginner's Guide](../beginners-guide.md) - Comprehensive introduction
- [Guides](../guides/README.md) - Task-oriented tutorials
- [Concepts](../concepts/README.md) - Theoretical foundations
- [Reference](../reference/README.md) - API documentation
- [Architecture](../architecture.md) - System design for contributors
