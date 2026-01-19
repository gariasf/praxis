# Learning Paths Roadmap

Visual guide to navigating the Praxis learning paths with recommended progressions.

## Complete Learning Graph

```
                    ┌─────────────────┐
                    │ Getting Started │
                    │  Installation   │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Beginner's Guide│
                    │  Read Overview  │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐    ┌───────────────┐    ┌──────────────┐
│   Rendering   │    │   Physics     │    │  Animation   │
│   (Beginner)  │    │  (Beginner)   │    │  (Beginner)  │
└───────┬───────┘    └───────┬───────┘    └──────┬───────┘
        │                    │                    │
        │    ┌───────────────┴────────────────┐   │
        │    │                                 │   │
        ▼    ▼                                 ▼   ▼
┌───────────────────┐                  ┌──────────────────┐
│    Scripting      │                  │     Audio        │
│    (Beginner)     │                  │   (Beginner)     │
└─────────┬─────────┘                  └──────────────────┘
          │
          │
          ▼
┌─────────────────────────────────────────────────────────┐
│              INTERMEDIATE LEVEL                          │
│  (Choose paths based on project needs)                  │
└─────────────────────────────────────────────────────────┘
          │
          │
    ┌─────┴──────┬─────────────┬──────────────┬──────────┐
    │            │             │              │          │
    ▼            ▼             ▼              ▼          ▼
Rendering   Animation      Physics      Networking   Editor
(Inter.)    (Inter.)       (Inter.)     (Beginner)   (Beginner)
    │            │             │              │          │
    │            │             │              │          │
    └────────────┴─────┬───────┴──────────────┴──────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                ADVANCED LEVEL                            │
│  (Specialization and production-ready systems)          │
└─────────────────────────────────────────────────────────┘
                       │
                       │
          ┌────────────┴───────────────┐
          │                            │
          ▼                            ▼
    ┌──────────────┐          ┌──────────────────┐
    │ Performance  │          │ Custom Systems    │
    │ Optimization │          │ & Extensions      │
    └──────────────┘          └──────────────────┘
```

## Role-Based Roadmaps

### Game Developer Path

```
Week 1-2: Foundation
┌─────────────────────────────────────┐
│ 1. Rendering (Beginner)             │
│    └─ Learn forward rendering       │
│    └─ Materials and lighting        │
│                                     │
│ 2. Input System                     │
│    └─ Keyboard, mouse, gamepad      │
└─────────────────────────────────────┘
              ↓
Week 3-4: Core Gameplay
┌─────────────────────────────────────┐
│ 3. Physics (Beginner)               │
│    └─ Rigid bodies and colliders    │
│    └─ Character controller          │
│                                     │
│ 4. Animation (Beginner)             │
│    └─ Load and play animations      │
│    └─ Basic blending                │
└─────────────────────────────────────┘
              ↓
Week 5-6: Game Logic
┌─────────────────────────────────────┐
│ 5. Scripting (Beginner-Inter.)      │
│    └─ Lua integration               │
│    └─ ECS access                    │
│    └─ Game systems in Lua           │
└─────────────────────────────────────┘
              ↓
Week 7-8: Polish
┌─────────────────────────────────────┐
│ 6. Audio (Beginner-Inter.)          │
│    └─ Sound effects                 │
│    └─ Spatial audio                 │
│                                     │
│ 7. Build Complete Game              │
└─────────────────────────────────────┘
```

### Graphics Programmer Path

```
Week 1-3: Rendering Fundamentals
┌─────────────────────────────────────┐
│ 1. Rendering (Beginner)             │
│    └─ Forward pipeline              │
│    └─ PBR materials                 │
│    └─ Lighting                      │
└─────────────────────────────────────┘
              ↓
Week 4-6: Advanced Techniques
┌─────────────────────────────────────┐
│ 2. Rendering (Intermediate)         │
│    └─ Deferred rendering            │
│    └─ HDR and tone mapping          │
│    └─ Shadow mapping                │
│    └─ Environment probes            │
│    └─ Post-processing               │
└─────────────────────────────────────┘
              ↓
Week 7-10: Custom Pipelines
┌─────────────────────────────────────┐
│ 3. Rendering (Advanced)             │
│    └─ Custom shaders                │
│    └─ Pipeline optimization         │
│    └─ GPU-driven rendering          │
│                                     │
│ 4. Performance (Advanced)           │
│    └─ GPU profiling                 │
│    └─ LOD systems                   │
└─────────────────────────────────────┘
```

### Gameplay Programmer Path

```
Week 1-2: Scripting Foundation
┌─────────────────────────────────────┐
│ 1. Scripting (All Levels)           │
│    └─ Lua basics                    │
│    └─ ECS integration               │
│    └─ Hot-reload                    │
│    └─ Game systems                  │
└─────────────────────────────────────┘
              ↓
Week 3-4: Physics & Interaction
┌─────────────────────────────────────┐
│ 2. Physics (Beginner-Inter.)        │
│    └─ Rigid bodies                  │
│    └─ Collision events              │
│    └─ Character controllers         │
└─────────────────────────────────────┘
              ↓
Week 5-7: Animation & AI
┌─────────────────────────────────────┐
│ 3. Animation (Intermediate)         │
│    └─ State machines                │
│    └─ Blend trees                   │
│    └─ Layered animation             │
│                                     │
│ 4. Script AI Behaviors              │
│    └─ Behavior trees in Lua         │
│    └─ State machines                │
└─────────────────────────────────────┘
              ↓
Week 8+: Game Systems
┌─────────────────────────────────────┐
│ 5. Build Complete Gameplay          │
│    └─ Combat system                 │
│    └─ Inventory                     │
│    └─ Quest system                  │
│    └─ Dialog system                 │
└─────────────────────────────────────┘
```

### Multiplayer Developer Path

```
Week 1-2: Foundation
┌─────────────────────────────────────┐
│ 1. Networking (Beginner)            │
│    └─ Client-server setup           │
│    └─ Connection management         │
│    └─ Basic messaging               │
└─────────────────────────────────────┘
              ↓
Week 3-4: Entity Replication
┌─────────────────────────────────────┐
│ 2. Networking (Intermediate)        │
│    └─ Component synchronization     │
│    └─ Interpolation                 │
│    └─ Bandwidth optimization        │
│                                     │
│ 3. Physics (Intermediate)           │
│    └─ Physics replication           │
│    └─ Networked character control   │
└─────────────────────────────────────┘
              ↓
Week 5-6: Production Features
┌─────────────────────────────────────┐
│ 4. Networking (Advanced)            │
│    └─ Lag compensation              │
│    └─ Client prediction             │
│    └─ Network profiling             │
│    └─ Security                      │
└─────────────────────────────────────┘
```

### Tools Developer Path

```
Week 1-2: Editor Basics
┌─────────────────────────────────────┐
│ 1. Editor (Beginner-Inter.)         │
│    └─ Navigation & selection        │
│    └─ Hierarchy panel               │
│    └─ Asset browser                 │
│    └─ Transform gizmos              │
└─────────────────────────────────────┘
              ↓
Week 3-4: Asset Pipeline
┌─────────────────────────────────────┐
│ 2. Assets (All Levels)              │
│    └─ Asset loading                 │
│    └─ GLTF workflow                 │
│    └─ Custom loaders                │
│    └─ Hot-reload                    │
└─────────────────────────────────────┘
              ↓
Week 5-7: Custom Tools
┌─────────────────────────────────────┐
│ 3. Editor (Advanced)                │
│    └─ Undo/redo system              │
│    └─ Custom panels                 │
│    └─ Command system                │
│    └─ Editor extensions             │
└─────────────────────────────────────┘
```

## Parallel Learning Paths

Some paths can be learned simultaneously:

### Recommended Parallel Combinations

**Rendering + Audio**
```
Morning: Rendering tutorials
Afternoon: Audio tutorials
→ Visuals and sound develop together
```

**Physics + Animation**
```
Morning: Physics simulation
Afternoon: Character animation
→ Natural integration for characters
```

**Scripting + Any System**
```
Learn system in Rust → Immediately script it in Lua
→ Reinforces both concepts
```

**Editor + Assets**
```
Learn editor tools → Use with asset workflow
→ Practical integration
```

## Milestone-Based Progression

Instead of time-based, track by milestones:

```
Level 1: Basic Systems
├─ [ ] Render a 3D scene
├─ [ ] Play an animation
├─ [ ] Physics simulation running
└─ [ ] Execute Lua script
        ↓
Level 2: Integration
├─ [ ] Animated character with physics
├─ [ ] Scripted game logic
├─ [ ] Multi-light scene with shadows
└─ [ ] Spatial audio working
        ↓
Level 3: Complex Systems
├─ [ ] Character state machine
├─ [ ] Multiplayer replication
├─ [ ] Custom rendering effects
└─ [ ] Editor tools built
        ↓
Level 4: Production Ready
├─ [ ] 60 FPS with 100+ entities
├─ [ ] Complete game loop
├─ [ ] Polish and optimization
└─ [ ] Deployment ready
```

## Cross-System Dependencies

Understanding system interactions:

```
         ┌──────────┐
    ┌────┤ ECS Core ├────┐
    │    └──────────┘    │
    │                    │
    ▼                    ▼
┌─────────┐        ┌──────────┐
│Transform│◄───────┤ Rendering│
│Hierarchy│        └──────────┘
└────┬────┘              │
     │                   │
     ├───────────┬───────┴──────┬──────────┐
     ▼           ▼              ▼          ▼
┌─────────┐ ┌─────────┐  ┌─────────┐ ┌─────────┐
│Animation│ │ Physics │  │  Audio  │ │ Editor  │
└─────────┘ └─────────┘  └─────────┘ └─────────┘
     │           │              │          │
     └───────────┴──────┬───────┴──────────┘
                        ▼
                  ┌──────────┐
                  │Scripting │
                  │  (Glue)  │
                  └──────────┘
                        │
                        ▼
                  ┌──────────┐
                  │Networking│
                  │(Sync All)│
                  └──────────┘
```

## Recommended Order by Project Type

### Single-Player Action Game
1. Rendering (Beginner → Intermediate)
2. Physics (Beginner → Intermediate)
3. Animation (Beginner → Intermediate)
4. Audio (Beginner → Intermediate)
5. Scripting (for game logic)
6. Performance optimization

### Multiplayer Shooter
1. Rendering (Beginner)
2. Physics (Beginner → Intermediate)
3. Networking (All levels) ← Priority!
4. Animation (Beginner)
5. Audio (Beginner)
6. Performance optimization

### Puzzle Game
1. Rendering (Beginner)
2. Scripting (All levels) ← Priority!
3. Physics (Beginner)
4. Audio (Beginner)
5. Editor (for level creation)

### RPG
1. Rendering (Beginner → Intermediate)
2. Animation (All levels) ← Priority!
3. Scripting (All levels) ← Priority!
4. Physics (Beginner)
5. Audio (Intermediate)
6. Editor (for content creation)

### Racing Game
1. Rendering (Beginner → Intermediate)
2. Physics (All levels) ← Priority!
3. Audio (Intermediate with spatial)
4. Animation (Beginner for UI/effects)
5. Networking (if multiplayer)

## Learning Velocity Tips

### Fast Track (Intensive Learning)
- 4-6 hours/day of focused study
- Complete paths in 1-2 weeks each
- Run every example immediately
- Build mini-projects between paths
- **Estimate**: 3-4 months to proficiency

### Steady Pace (Sustainable Learning)
- 1-2 hours/day with weekends
- Complete paths in 3-4 weeks each
- Thorough understanding of concepts
- Build projects to practice
- **Estimate**: 6-9 months to proficiency

### Part-Time (Casual Learning)
- 30min-1 hour/day
- Complete paths in 6-8 weeks each
- Focus on one system at a time
- Small experiments and tests
- **Estimate**: 12-18 months to proficiency

## Next Steps

1. **Assess your goals**: What type of game/system are you building?
2. **Choose starting path**: Use role-based recommendations
3. **Set milestones**: Track progress with checkpoints
4. **Run examples**: Hands-on learning is critical
5. **Build projects**: Apply knowledge immediately
6. **Ask questions**: Use documentation and examples

## Navigation

- [Back to Learning Paths Overview](README.md)
- [Learning Paths Glossary](glossary.md) - Term definitions
- [Individual Path Files](README.md) - Detailed learning paths

---

Happy learning! 🎮
