# Learning Paths Quick Reference

Quick overview of all learning paths with time estimates and key milestones.

## Path Selection Guide

### By Role

| Role | Recommended Paths | Order |
|------|------------------|-------|
| **Game Developer** | Rendering → Physics → Animation → Scripting | 1. Rendering (Beginner)<br>2. Physics (Beginner)<br>3. Animation (Beginner-Intermediate)<br>4. Scripting (Beginner-Intermediate) |
| **Graphics Programmer** | Rendering (All) → Performance | 1. Rendering (All levels)<br>2. Performance (All levels) |
| **Gameplay Programmer** | Scripting → Physics → Animation | 1. Scripting (All levels)<br>2. Physics (Intermediate)<br>3. Animation (Intermediate) |
| **Multiplayer Developer** | Networking → Physics | 1. Networking (All levels)<br>2. Physics (Intermediate) for replication |
| **Tools Developer** | Editor → Assets | 1. Editor (All levels)<br>2. Assets (Intermediate-Advanced) |

### By Experience Level

| Experience | Start Here |
|------------|------------|
| **New to Game Engines** | [Rendering Path](learning-paths/rendering.md) (Beginner) |
| **Familiar with Unity/Unreal** | [Beginner's Guide](beginners-guide.md) → Any path (Intermediate) |
| **Experienced Game Developer** | Skip to Intermediate/Advanced in paths of interest |
| **Engine Contributor** | [Architecture](architecture.md) → Advanced sections |

## Path Summaries

### Core Systems

#### [Rendering Path](learning-paths/rendering.md)
**Time**: 3-6 weeks | **Prerequisites**: Basic 3D math

| Level | Duration | Key Topics | Outcome |
|-------|----------|------------|---------|
| Beginner | 1-2 weeks | Forward rendering, PBR materials, lighting | Render 3D scenes with materials and lights |
| Intermediate | 2-3 weeks | Deferred rendering, HDR, shadows, IBL, post-processing | Advanced visual quality |
| Advanced | 2-3 weeks | Custom shaders, pipeline optimization, GPU-driven | Custom rendering techniques |

**Exercises**: 15+ hands-on exercises  
**Examples**: `scene_demo`, `material_demo`, `advanced_lighting_demo`

---

#### [Animation Path](learning-paths/animation.md)
**Time**: 2-4 weeks | **Prerequisites**: Understanding of transforms

| Level | Duration | Key Topics | Outcome |
|-------|----------|------------|---------|
| Beginner | 1 week | Skeletal basics, animation clips, playback | Play skeletal animations |
| Intermediate | 1-2 weeks | Cross-fading, blend trees, layers, state machines | Responsive character animation |
| Advanced | 1-2 weeks | IK, retargeting, additive blending, root motion | Production-ready animation |

**Exercises**: 20+ hands-on exercises  
**Examples**: `skeletal_animation_demo`, `animation_blending_demo`, `animation_advanced_demo`

---

#### [Physics Path](learning-paths/physics.md)
**Time**: 2-3 weeks | **Prerequisites**: Basic 3D math

| Level | Duration | Key Topics | Outcome |
|-------|----------|------------|---------|
| Beginner | 1 week | Rigid bodies, basic colliders, ECS integration | Physics simulations |
| Intermediate | 1 week | Collision events, raycasting, character controllers | Interactive physics |
| Advanced | 1 week | Joints, ragdolls, optimization | Complex physics systems |

**Exercises**: 18+ hands-on exercises  
**Examples**: Physics examples via demos

---

#### [Scripting Path](learning-paths/scripting.md)
**Time**: 1-2 weeks | **Prerequisites**: Basic programming

| Level | Duration | Key Topics | Outcome |
|-------|----------|------------|---------|
| Beginner | 3-4 days | Lua basics, script loading, function calls | Execute Lua scripts |
| Intermediate | 4-5 days | ECS access, component manipulation, game logic | Build systems in Lua |
| Advanced | 4-5 days | Hot-reload, sandboxing, performance | Production Lua integration |

**Exercises**: 15+ hands-on exercises  
**Examples**: `scripting_demo`, `scripting_advanced_demo`

---

#### [Networking Path](learning-paths/networking.md)
**Time**: 2-3 weeks | **Prerequisites**: Async/await, ECS

| Level | Duration | Key Topics | Outcome |
|-------|----------|------------|---------|
| Beginner | 1 week | Client-server setup, connections, messaging | Basic multiplayer |
| Intermediate | 1 week | Entity replication, interpolation, bandwidth | Smooth multiplayer |
| Advanced | 1 week | Lag compensation, prediction, profiling | Production multiplayer |

**Exercises**: 12+ hands-on exercises  
**Examples**: `networking_demo`

---

### Supporting Systems

#### [Audio Path](learning-paths/audio.md)
**Time**: 1 week | **Prerequisites**: Basic 3D space

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 2-3 days | Audio playback, volume control |
| Intermediate | 2-3 days | Spatial positioning, attenuation |
| Advanced | 2-3 days | Pooling, LOD, optimization |

**Examples**: `audio_simple`, `audio_demo`

---

#### [Editor Path](learning-paths/editor.md)
**Time**: 1-2 weeks | **Prerequisites**: Basic engine usage

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 3-4 days | Navigation, selection, hierarchy |
| Intermediate | 4-5 days | Asset browser, gizmos, scenes |
| Advanced | 5-6 days | Undo/redo, custom panels, extensions |

**Examples**: `editor_demo`, `selection_demo`, `undo_redo_system_demo`

---

#### [Assets Path](learning-paths/assets.md)
**Time**: 4-6 days | **Prerequisites**: Basic file I/O

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 2 days | Loading meshes, textures, audio |
| Intermediate | 2 days | GLTF scenes, skeletal meshes |
| Advanced | 2 days | Custom loaders, hot-reload, pipeline |

---

### Cross-Cutting

#### [Performance Path](learning-paths/performance.md)
**Time**: 1-2 weeks | **Prerequisites**: Completed at least one other path

| Level | Duration | Key Topics |
|-------|----------|------------|
| Beginner | 4-5 days | Profiling, identifying bottlenecks |
| Intermediate | 4-5 days | Rendering, ECS, physics, memory optimization |
| Advanced | 4-5 days | GPU profiling, multi-threading, LOD, production |

**Goal**: 60+ FPS in production scenarios

---

## Learning Sequences

### Week-by-Week Plans

#### 4-Week Game Developer Fast Track
```
Week 1: Rendering (Beginner) + Input basics
Week 2: Physics (Beginner) + Animation (Beginner)  
Week 3: Scripting (Beginner-Intermediate)
Week 4: Build small game project
```

#### 8-Week Complete Mastery
```
Week 1-2: Rendering (Beginner + Intermediate)
Week 3: Physics (Beginner + Intermediate)
Week 4-5: Animation (All levels)
Week 6: Scripting (All levels)
Week 7: Networking or Editor (based on goals)
Week 8: Performance optimization + project polish
```

#### 2-Week Graphics Focus
```
Week 1: Rendering (Beginner + Intermediate)
Week 2: Rendering (Advanced) + Performance
```

#### 2-Week Multiplayer Focus
```
Week 1: Networking (Beginner + Intermediate)
Week 2: Networking (Advanced) + Physics replication
```

---

## Milestone Tracking

### Beginner Milestones (2-3 weeks total)
- [ ] Render 3D scene with lighting (Rendering Beginner)
- [ ] Create physics simulation (Physics Beginner)
- [ ] Play character animations (Animation Beginner)
- [ ] Execute Lua scripts (Scripting Beginner)

### Intermediate Milestones (4-6 weeks total)
- [ ] Implement deferred renderer with HDR (Rendering Intermediate)
- [ ] Build character controller (Physics Intermediate)
- [ ] Create animation state machine (Animation Intermediate)
- [ ] Access ECS from Lua (Scripting Intermediate)
- [ ] Setup multiplayer replication (Networking Intermediate)

### Advanced Milestones (4-8 weeks total)
- [ ] Create custom rendering pipeline (Rendering Advanced)
- [ ] Implement ragdoll physics (Physics Advanced)
- [ ] Build IK system (Animation Advanced)
- [ ] Hot-reload Lua scripts (Scripting Advanced)
- [ ] Lag compensation working (Networking Advanced)
- [ ] 60+ FPS with 100+ entities (Performance)

---

## Estimated Time Investment

### By Path
- **Rendering**: 50-100 hours (most comprehensive)
- **Animation**: 60-90 hours
- **Physics**: 60-80 hours
- **Scripting**: 50-70 hours
- **Networking**: 45-75 hours
- **Audio**: 24-30 hours
- **Editor**: 33-50 hours
- **Assets**: 22-30 hours
- **Performance**: 44-60 hours

### By Level
- **All Beginner**: 60-80 hours (1-2 months part-time)
- **All Intermediate**: 100-140 hours (2-3 months part-time)
- **All Advanced**: 100-150 hours (2-4 months part-time)

### Complete Engine Mastery
**Total**: 400-600 hours (6-12 months part-time, 3-6 months full-time)

---

## Prerequisites Map

```
Getting Started (Required for all)
    ↓
┌───┴─────────────────────────────────────┐
│   Core Foundation (Recommended first)   │
├─────────────────────────────────────────┤
│ • Rendering (Beginner)                  │
│ • Basic 3D math understanding           │
│ • ECS concepts                          │
└───┬─────────────────────────────────────┘
    ↓
┌───┴─────────────────────────────────────┐
│   Choose Your Path                      │
├─────────────────────────────────────────┤
│ Animation → requires transforms         │
│ Physics → requires transforms           │
│ Scripting → requires ECS knowledge      │
│ Networking → requires ECS + async       │
│ Audio → minimal prerequisites           │
│ Editor → basic engine usage            │
│ Assets → basic file I/O                │
└───┬─────────────────────────────────────┘
    ↓
┌───┴─────────────────────────────────────┐
│   Performance (After any other path)    │
└─────────────────────────────────────────┘
```

---

## Cross-Path Integration Points

These topics appear in multiple paths:

| Topic | Primary Path | Also In |
|-------|-------------|---------|
| **Transforms** | All paths | Core concept everywhere |
| **ECS Queries** | Concepts | Rendering, Physics, Animation, Scripting |
| **Profiling** | Performance | All Advanced sections |
| **GLTF Loading** | Assets | Rendering, Animation |
| **Character Controllers** | Physics | Animation (for movement) |
| **Spatial Audio** | Audio | Physics (collision sounds) |
| **Hot-Reload** | Scripting, Assets | Editor, Performance |
| **Component Replication** | Networking | Physics, Animation |

---

## Getting Started

1. **Choose your path** based on role/goals above
2. **Check prerequisites** in the selected path
3. **Follow the progression** (don't skip levels)
4. **Run examples** for each section
5. **Complete exercises** to reinforce learning
6. **Track milestones** to measure progress

## Navigation

- **[Main Learning Paths Document](learning-paths.md)** - Detailed overview
- **[Learning Paths Directory](learning-paths/)** - Individual path files
- **[Beginner's Guide](beginners-guide.md)** - Comprehensive introduction
- **[Examples List](README.md#examples)** - All runnable examples

---

Good luck on your learning journey! 🚀
