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
| **Tools Developer** | Editor → Assets → Serialization | Start with [Editor](editor.md) |

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
| [**Rendering**](rendering.md) | 4-8 weeks | High | Forward/deferred rendering, PBR, shadows, HDR, post-processing, TAA, SSR, GPU culling |
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
**Time**: 15-20 hours  
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
**Time**: 30-40 hours  
**Prerequisites**: Beginner rendering complete

**Practical Guides**:
1. [Deferred Rendering](../guides/rendering/deferred-rendering.md) - Multi-pass G-buffer pipeline
2. [HDR and Tone Mapping](../guides/rendering/hdr-tonemapping.md) - High dynamic range
3. [Shadows](../guides/rendering/shadows.md) - Cascaded shadow maps
4. [Environment Probes](../guides/rendering/environment-probes.md) - Image-based lighting
5. [Post-Processing](../guides/rendering/post-processing.md) - Bloom, color grading

**Examples**: `advanced_lighting_demo`, `environment_probe_demo`, `material_demo`

#### Advanced: Custom Pipeline Development
**Time**: 60-80 hours  
**Prerequisites**: Intermediate rendering mastery

**Topics**:
- [Architecture: Render Pipeline](../architecture/render-pipeline.md)
- [Shaders Reference](../reference/shaders.md)
- [Temporal Anti-Aliasing (TAA)](../guides/rendering/taa.md) - High-quality edge smoothing
- [Screen-Space Reflections (SSR)](../guides/rendering/ssr.md) - Realistic reflections
- [GPU Culling](../guides/rendering/gpu-culling.md) - GPU-driven rendering
- Custom shader development
- Bindless rendering concepts

**Examples**: `complete_features_demo`, `gpu_culling_demo`

---

### Animation Path

**Goal**: Create lifelike character movement from basic skeletal animation to advanced techniques.

#### Beginner: Skeletal Animation Basics
**Time**: 8-12 hours  
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
**Time**: 15-20 hours  
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
**Time**: 20-30 hours  
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
**Time**: 8-12 hours  
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
**Time**: 10-15 hours  

- Collision events and queries
- Raycasting for gameplay
- Character controllers
- Joints and constraints

#### Advanced: Custom Integration
**Time**: 12-18 hours  

- Advanced joint configurations
- Physics debugging and profiling
- Ragdoll integration with animation

---

### Scripting Path

**Goal**: Add runtime flexibility with Lua scripting and hot-reload capabilities.

#### Beginner: Lua Basics
**Time**: 5-8 hours  
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
**Time**: 8-12 hours  
**Prerequisites**: Beginner scripting + ECS understanding

**Read First**: [ECS Architecture](../concepts/ecs-architecture.md)

**Learning Outcomes**:
- Access entities from Lua
- Query and modify components
- Create game logic in Lua

#### Advanced: Hot-Reload and Performance
**Time**: 6-10 hours  

- Hot-reload configuration
- Sandboxing levels (security)
- Performance monitoring
- Script debugging techniques

---

### Networking Path

**Goal**: Build multiplayer games with client-server architecture and lag compensation.

#### Beginner: Client-Server Setup
**Time**: 8-12 hours  
**Prerequisites**: Basic ECS understanding

**Practical Guides**:
1. [Networking Guide](../guides/systems/networking.md)
2. Run `cargo run --example networking_demo`

**Learning Outcomes**:
- Setup server and client
- Establish connections
- Handle basic message passing

#### Intermediate: Entity Replication
**Time**: 12-16 hours  

- Component registration
- Automatic synchronization
- Transform interpolation
- Bandwidth optimization

#### Advanced: Lag Compensation
**Time**: 12-18 hours  

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
| Beginner | 2-3 days (6-8 hours) | Audio playback, volume control |
| Intermediate | 2-3 days (6-8 hours) | Spatial positioning, attenuation |
| Advanced | 2-3 days (6-8 hours) | Pooling, LOD, optimization |

**Examples**: `audio_simple`, `audio_demo`

---

### Editor Path

**Goal**: Master the editor tools for level design and debugging.

**Documentation**: [Editor Overview](../editor/README.md)

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 3-4 days (10-12 hours) | Navigation, selection, hierarchy |
| Intermediate | 4-5 days (12-16 hours) | Asset browser, gizmos, scenes |
| Advanced | 5-6 days (16-20 hours) | Undo/redo, custom panels, extensions |

**Examples**: `editor_demo`, `selection_demo`, `undo_redo_system_demo`

---

### Assets Path

**Goal**: Master the asset pipeline for efficient resource management.

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 2 days (6-8 hours) | Loading meshes, textures, audio |
| Intermediate | 2 days (6-8 hours) | GLTF scenes, skeletal meshes |
| Advanced | 2 days (6-8 hours) | Custom loaders, hot-reload |

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

### 12-Week Modern Graphics Specialist
```
Week 1-2: Rendering (Beginner + Intermediate)
Week 3-4: Advanced deferred rendering + HDR + shadows
Week 5-6: Temporal Anti-Aliasing (TAA) implementation
Week 7-8: Screen-Space Reflections (SSR) + environment probes
Week 9-10: GPU culling + indirect drawing
Week 11: Bindless rendering concepts + descriptor optimization
Week 12: Complete features integration + performance tuning
```

---

## Recommended Project Progressions

### For Beginners

**Project 1: Simple Scene (Week 1)**
- 3-5 textured meshes
- 2-3 light sources
- Basic camera movement
- Focus: Fundamentals

**Project 2: Interactive Environment (Weeks 2-3)**
- Physics-enabled objects
- Player controller
- Basic Lua scripting for interactions
- Focus: System integration

**Project 3: Mini-Game (Week 4)**
- Game objective and rules
- Animation state machines
- Score tracking and UI
- Focus: Complete game loop

### For Intermediate Developers

**Project 1: Visually Rich Scene (Weeks 1-2)**
- Deferred rendering
- HDR with multiple tone mapping options
- Shadow mapping
- Post-processing stack
- Focus: Visual quality

**Project 2: Character Action Game (Weeks 3-5)**
- Animated player character
- Physics-based interactions
- Save/load system
- Combat or platforming mechanics
- Focus: Gameplay systems

**Project 3: Multiplayer Prototype (Weeks 6-8)**
- Client-server architecture
- Entity replication
- Networked physics
- Interpolation and prediction
- Focus: Networking fundamentals

### For Advanced Graphics Programmers

**Project 1: Modern Renderer (Weeks 1-6)**
- TAA implementation
- SSR with Hi-Z optimization
- GPU frustum culling
- Descriptor optimization
- Performance profiling
- Focus: Advanced rendering techniques

**Project 2: GPU-Driven Pipeline (Weeks 7-10)**
- Complete GPU culling system
- Indirect drawing
- Bindless materials (conceptual)
- 10,000+ object support
- Focus: GPU-driven architecture

**Project 3: Open World Tech Demo (Weeks 11-12)**
- Large-scale scene rendering
- Asset streaming
- LOD system integration
- Complete feature integration (TAA + SSR + GPU culling)
- 60 FPS target with all features
- Focus: Production-ready optimization

---

## Milestone Tracking

### Beginner Milestones
- [ ] Render 3D scene with lighting (Rendering Beginner)
- [ ] Create physics simulation (Physics Beginner)
- [ ] Play character animations (Animation Beginner)
- [ ] Execute Lua scripts (Scripting Beginner)
- [ ] Build interactive scene with 5+ objects

### Intermediate Milestones
- [ ] Implement deferred renderer with HDR (Rendering Intermediate)
- [ ] Build character controller (Physics Intermediate)
- [ ] Create animation state machine (Animation Intermediate)
- [ ] Access ECS from Lua (Scripting Intermediate)
- [ ] Setup multiplayer replication (Networking Intermediate)
- [ ] Save and load complete game state
- [ ] Complete a mini-game prototype

### Advanced Milestones

**Rendering Advanced**:
- [ ] Implement temporal anti-aliasing (TAA) with history rejection
- [ ] Build screen-space reflections (SSR) with Hi-Z optimization
- [ ] Create GPU culling system with indirect drawing
- [ ] Understand bindless rendering architecture
- [ ] Integrate TAA + SSR + GPU culling in single pipeline

**Other Advanced**:
- [ ] Create custom rendering pipeline (Rendering Advanced)
- [ ] Implement ragdoll physics (Physics Advanced)
- [ ] Build IK system (Animation Advanced)
- [ ] Hot-reload Lua scripts (Scripting Advanced)
- [ ] Lag compensation working (Networking Advanced)
- [ ] 60+ FPS with 100+ entities (Performance)
- [ ] Support 10,000+ objects with GPU culling

**Integration Milestones**:
- [ ] TAA eliminates aliasing without visible blur
- [ ] SSR shows accurate reflections with environment probe fallback
- [ ] GPU culling handles 5,000+ objects with < 1ms CPU overhead
- [ ] Save/load system persists complex scenes with metadata
- [ ] Complete feature demo runs at 60+ FPS

---

## Time Investment

### By Level (Updated for Modern Features)
- **All Beginner**: 60-80 hours (1-2 months part-time)
- **All Intermediate**: 110-150 hours (2.5-3.5 months part-time)
- **All Advanced**: 130-180 hours (3-4.5 months part-time)

### Complete Engine Mastery
**Total**: 450-700 hours (9-14 months part-time, 4-7 months full-time)

### Modern Graphics Specialization
**Total**: 150-200 hours (3-5 months part-time, 1.5-2 months full-time)
- Rendering Beginner + Intermediate: 45-60 hours
- TAA Implementation: 15-20 hours
- SSR Implementation: 20-30 hours
- GPU Culling: 25-35 hours
- Bindless Concepts: 15-20 hours
- Integration & Optimization: 30-45 hours

---

## Self-Study Guidance

### Effective Learning Strategies

1. **Follow the Progression**: Don't skip levels - each builds on the previous
2. **Run Examples First**: See working code before reading theory
3. **Modify, Don't Just Read**: Change values, break things, fix them
4. **Build Small Projects**: Apply concepts immediately in practice
5. **Track Your Progress**: Use milestones to measure advancement
6. **Take Breaks**: Complex topics need processing time
7. **Ask for Help**: Use documentation, examples, and architecture docs

### Study Habits for Success

**Daily Practice (1-2 hours)**:
- 20-30 min: Read theory/concepts
- 30-60 min: Hands-on exercises
- 10-20 min: Review and note-taking

**Weekly Practice (10-15 hours)**:
- 3-4 hours: Theory and concepts
- 5-8 hours: Practical exercises and examples
- 2-3 hours: Project work applying new concepts

### When You Get Stuck

1. **Re-read Prerequisites**: Missing foundation often causes confusion
2. **Run Related Examples**: Working code shows the correct approach
3. **Check Architecture Docs**: Understanding design helps debug
4. **Simplify**: Strip down to minimal working case
5. **Profile**: Measure don't guess (use profiling tools)
6. **Compare**: Look at example code vs your implementation
7. **Document**: Explain the problem to yourself in writing

### Recommended Order for New Features

**For Advanced Graphics Students**:
1. Complete Intermediate rendering first (deferred, HDR, shadows, post-processing)
2. Learn TAA next (builds on temporal concepts, simpler than SSR)
3. Then SSR (requires TAA understanding for stability)
4. Then GPU culling (independent system, good break from screen-space effects)
5. Finally integration (combine all systems)

**Why this order?**:
- TAA teaches temporal techniques needed for SSR stability
- SSR is most complex, benefits from TAA experience
- GPU culling provides different skillset (compute shaders, indirect drawing)
- Integration reinforces all concepts together

---

## How to Use Learning Paths

1. **Choose your path** based on your role/goals above
2. **Check prerequisites** before starting each level
3. **Follow the progression** (don't skip levels)
4. **Run examples** immediately after reading theory
5. **Complete exercises** to reinforce learning
6. **Build small projects** to apply concepts
7. **Track milestones** to measure progress
8. **Cross-reference** related systems when mentioned
9. **Profile your work** to verify optimizations
10. **Iterate and refine** your understanding

---

## Navigation

- [Beginner's Guide](../beginners-guide.md) - Comprehensive introduction
- [Guides](../guides/README.md) - Task-oriented tutorials
- [Concepts](../concepts/README.md) - Theoretical foundations
- [Reference](../reference/README.md) - API documentation
- [Architecture](../architecture.md) - System design for contributors
