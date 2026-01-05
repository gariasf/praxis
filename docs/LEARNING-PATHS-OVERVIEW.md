# Learning Paths System Overview

Complete documentation navigation for the Praxis learning paths system.

## What Are Learning Paths?

Learning paths are structured, progressive guides that take you from beginner to advanced mastery of Praxis engine subsystems. Unlike traditional documentation that explains individual features, learning paths provide:

- **Clear progression**: Beginner → Intermediate → Advanced
- **Prerequisites**: Know what to learn first
- **Learning outcomes**: Measurable goals for each section
- **Time estimates**: Plan your learning schedule
- **Hands-on exercises**: Practice what you learn
- **Cross-references**: Connect related concepts
- **Checkpoints**: Verify understanding before proceeding

## Documentation Structure

```
Learning Paths Documentation
│
├── Main Documents
│   ├── learning-paths.md ..................... Main overview (start here!)
│   ├── learning-paths-quick-reference.md ..... Quick lookup guide
│   └── LEARNING-PATHS-OVERVIEW.md ............ This document
│
├── Learning Paths Directory (learning-paths/)
│   ├── README.md ............................. Directory index
│   ├── roadmap.md ............................ Visual progressions
│   ├── glossary.md ........................... Terms and definitions
│   │
│   ├── Core Systems
│   │   ├── rendering.md ...................... Graphics pipeline (3-6 weeks)
│   │   ├── animation.md ...................... Character animation (2-4 weeks)
│   │   ├── physics.md ........................ Rigid body simulation (2-3 weeks)
│   │   ├── scripting.md ...................... Lua integration (1-2 weeks)
│   │   └── networking.md ..................... Multiplayer systems (2-3 weeks)
│   │
│   ├── Supporting Systems
│   │   ├── audio.md .......................... Spatial audio (1 week)
│   │   ├── editor.md ......................... Editor tools (1-2 weeks)
│   │   └── assets.md ......................... Asset pipeline (4-6 days)
│   │
│   └── Cross-Cutting
│       └── performance.md .................... Optimization (1-2 weeks)
│
└── Integration with Existing Docs
    ├── beginners-guide.md .................... Detailed system explanations
    ├── guides/ ............................... Task-oriented tutorials
    ├── concepts/ ............................. Theoretical foundations
    └── reference/ ............................ API documentation
```

## How to Use Learning Paths

### For New Users

1. **Start here**: [learning-paths.md](learning-paths.md)
2. **Choose your role**: See "Quick Start by Role" section
3. **Select a path**: Based on your goals
4. **Read prerequisites**: Ensure you have required knowledge
5. **Follow progression**: Complete Beginner before Intermediate
6. **Run examples**: Hands-on learning is critical
7. **Complete exercises**: Reinforce understanding
8. **Track progress**: Use checkpoints

### For Experienced Users

1. **Quick Reference**: [learning-paths-quick-reference.md](learning-paths-quick-reference.md)
2. **Skip to level**: Jump to Intermediate or Advanced
3. **Focus on gaps**: Identify and fill knowledge gaps
4. **Cross-reference**: Connect new learning to existing knowledge

### For Specific Roles

See role-based roadmaps in [roadmap.md](learning-paths/roadmap.md):
- Game Developer
- Graphics Programmer
- Gameplay Programmer
- Multiplayer Developer
- Tools Developer

## Learning Path Components

Each path includes these elements:

### Path Overview
- Total time investment
- Prerequisites
- Final learning goal
- Progression map (visual)

### For Each Level (Beginner/Intermediate/Advanced)

**Goal Statement**
- What you'll achieve at this level

**Prerequisites**
- Required knowledge before starting
- Links to prerequisite material

**Theory Sections**
- Conceptual understanding
- Links to relevant documentation
- Time estimates

**Practice Sections**
- Hands-on coding
- Code examples and patterns
- Time estimates

**Exercises**
- Structured challenges
- Progressive difficulty
- Clear expected outcomes

**Checkpoint**
- Self-assessment questions
- Capstone project
- Learning outcomes checklist
- Time to complete

**Cross-References**
- Related systems
- Performance considerations
- Integration topics

## Navigation Quick Reference

| Document | Purpose | When to Use |
|----------|---------|-------------|
| [learning-paths.md](learning-paths.md) | Main overview with all paths | Starting learning journey |
| [quick-reference.md](learning-paths-quick-reference.md) | Condensed info, time estimates | Quick lookup, planning |
| [roadmap.md](learning-paths/roadmap.md) | Visual progressions, role-based | Understanding connections |
| [glossary.md](learning-paths/glossary.md) | Term definitions | Looking up unfamiliar terms |
| [rendering.md](learning-paths/rendering.md) | Detailed rendering path | Learning graphics |
| [animation.md](learning-paths/animation.md) | Detailed animation path | Learning animation |
| [physics.md](learning-paths/physics.md) | Detailed physics path | Learning physics |
| [scripting.md](learning-paths/scripting.md) | Detailed scripting path | Learning Lua integration |
| [networking.md](learning-paths/networking.md) | Detailed networking path | Learning multiplayer |
| [audio.md](learning-paths/audio.md) | Detailed audio path | Learning sound systems |
| [editor.md](learning-paths/editor.md) | Detailed editor path | Learning tools |
| [assets.md](learning-paths/assets.md) | Detailed assets path | Learning asset pipeline |
| [performance.md](learning-paths/performance.md) | Detailed optimization path | Optimizing systems |

## Integration with Other Documentation

Learning paths complement existing documentation:

### [Beginner's Guide](beginners-guide.md)
- **Beginner's Guide**: Deep explanations of how systems work internally
- **Learning Paths**: Structured progression for learning those systems
- **Use together**: Read concepts in Beginner's Guide, practice in Learning Paths

### [Guides](guides/README.md)
- **Guides**: Task-oriented "how to" instructions
- **Learning Paths**: Progressive learning with context
- **Use together**: Learning Paths reference Guides for specific tasks

### [Concepts](concepts/README.md)
- **Concepts**: Theory and design decisions
- **Learning Paths**: Practical application of theory
- **Use together**: Read Concepts for "why", Learning Paths for "how to learn"

### [Reference](reference/README.md)
- **Reference**: API documentation and specs
- **Learning Paths**: When and why to use APIs
- **Use together**: Learning Paths teach usage, Reference gives details

### Examples
- All learning paths reference relevant examples
- Run examples immediately after reading theory
- Study example code as reference implementations

## Learning Path Statistics

### Coverage
- **9 complete paths**: Rendering, Animation, Physics, Scripting, Networking, Audio, Editor, Assets, Performance
- **3 skill levels**: Beginner, Intermediate, Advanced per path
- **75+ sections** across all paths
- **100+ exercises** with clear outcomes
- **30+ example programs** referenced

### Time Investment
- **Complete Beginner**: 60-80 hours (1-2 months part-time)
- **Complete Intermediate**: 100-140 hours (2-3 months part-time)
- **Complete Advanced**: 100-150 hours (2-4 months part-time)
- **Total Mastery**: 400-600 hours (6-12 months part-time)

### By Path
| Path | Time | Difficulty | Priority for |
|------|------|------------|--------------|
| Rendering | 50-100 hrs | High | Graphics programmers |
| Animation | 60-90 hrs | Medium-High | Game developers |
| Physics | 60-80 hrs | Medium | Game developers |
| Scripting | 50-70 hrs | Medium | Gameplay programmers |
| Networking | 45-75 hrs | High | Multiplayer developers |
| Audio | 24-30 hrs | Low | Everyone |
| Editor | 33-50 hrs | Medium | Tools developers |
| Assets | 22-30 hrs | Low-Medium | Tools developers |
| Performance | 44-60 hrs | High | Everyone (after other paths) |

## Recommended Learning Sequences

### First-Time User (8 weeks)
```
Week 1-2: Rendering (Beginner)
Week 3: Physics (Beginner)
Week 4: Animation (Beginner)
Week 5: Scripting (Beginner)
Week 6: Audio (Beginner)
Week 7-8: Build complete game
```

### Graphics Focus (10 weeks)
```
Week 1-3: Rendering (Beginner)
Week 4-6: Rendering (Intermediate)
Week 7-10: Rendering (Advanced) + Performance
```

### Gameplay Focus (8 weeks)
```
Week 1-2: Scripting (All levels)
Week 3-4: Physics (Beginner-Intermediate)
Week 5-7: Animation (Beginner-Intermediate)
Week 8: Integration project
```

### Multiplayer Focus (6 weeks)
```
Week 1-2: Networking (Beginner-Intermediate)
Week 3-4: Networking (Advanced)
Week 5: Physics (Intermediate - for replication)
Week 6: Performance tuning
```

## Support and Resources

### When You Get Stuck

1. **Check glossary**: [glossary.md](learning-paths/glossary.md)
2. **Review prerequisites**: Make sure you completed required sections
3. **Run examples**: Study working code
4. **Read concepts**: Understand theory in [concepts/](concepts/README.md)
5. **Check guides**: Step-by-step instructions in [guides/](guides/README.md)

### Additional Resources

- **API Documentation**: `cargo doc --workspace --no-deps --open`
- **Crate READMEs**: Detailed info in `crates/praxis_*/README.md`
- **Examples**: All in `examples/` directory
- **Tests**: Unit tests show API usage

## Maintenance and Updates

Learning paths are living documents that evolve with the engine:

- **New features**: Paths updated when features added
- **User feedback**: Improvements based on learner experience
- **Example updates**: Examples kept in sync with API
- **Cross-references**: Updated when documentation restructures

## Contributing

Help improve learning paths:

1. **Report issues**: Unclear sections, broken links
2. **Suggest exercises**: Additional hands-on practice
3. **Share experience**: What worked, what didn't
4. **Add examples**: More demonstration code
5. **Write tutorials**: Expand on topics

## Success Criteria

You've successfully used learning paths when you can:

- Navigate documentation efficiently
- Choose appropriate systems for tasks
- Understand prerequisites and dependencies
- Build features independently
- Debug issues systematically
- Optimize for performance
- Extend engine functionality

## Next Steps

**New to Praxis?**
1. Read [Getting Started](getting-started/README.md)
2. Skim [Beginner's Guide](beginners-guide.md)
3. Choose role in [quick-reference.md](learning-paths-quick-reference.md)
4. Start first learning path!

**Ready to learn?**
- [View All Learning Paths](learning-paths.md)
- [Quick Reference Guide](learning-paths-quick-reference.md)
- [Visual Roadmap](learning-paths/roadmap.md)

**Looking for something specific?**
- [Rendering Path](learning-paths/rendering.md)
- [Animation Path](learning-paths/animation.md)
- [Physics Path](learning-paths/physics.md)
- [Scripting Path](learning-paths/scripting.md)
- [Networking Path](learning-paths/networking.md)

---

**Happy Learning!** 🚀

The Praxis learning paths are designed to take you from beginner to expert systematically and efficiently. Follow the progression, practice diligently, and you'll master the engine in no time.
