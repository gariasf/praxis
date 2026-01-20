# Learning Paths by Background

Role-specific guides that map familiar concepts to universal engine architecture patterns, showing how to leverage your existing expertise to master game engine development.

## Overview

These learning paths are designed for developers with specific backgrounds, mapping concepts you already know to universal engine design patterns. Each path translates familiar language-specific idioms to engine architecture fundamentals, using Praxis as a reference implementation.

**Purpose**: Bridge the gap between your current expertise and game engine architecture knowledge by:
1. Mapping familiar concepts to universal patterns
2. Explaining what high-level engines abstract away
3. Showing idiomatic implementations across languages
4. Providing concrete exercises and projects

---

## Available Paths

### [For Unity Developers Learning Engine Architecture](unity-developers.md)

**Best for**: Unity developers (C# / MonoBehaviour) who want to understand engine internals

**You'll Learn**:
- GameObject/Component architecture → Universal ECS patterns
- What Unity automates (rendering, physics, transforms)
- Mapping Unity DOTS to Praxis ECS
- Converting MonoBehaviour logic to systems
- Asset pipeline differences
- Physics integration patterns

**Key Mappings**:
- `GameObject` → Entity (ECS)
- `MonoBehaviour` → Component (data) + System (logic)
- `Update()` / `FixedUpdate()` → System scheduling
- `Resources.Load()` → Asset loading patterns
- `Rigidbody` → Physics integration

**Time**: 4-12 weeks depending on depth

---

### [For C++ Programmers Building Custom Engines](cpp-programmers.md)

**Best for**: C++ programmers building engines, Unreal developers exploring internals, or transitioning to Rust

**You'll Learn**:
- C++ vs Rust memory management (RAII, smart pointers, ownership)
- Implementing ECS in C++ (EnTT, custom archetypes)
- Vulkan rendering patterns
- Transform hierarchy optimization
- Physics integration (PhysX, Jolt)
- Job systems and multithreading
- Custom memory allocators

**Key Translations**:
- `std::unique_ptr<T>` ↔ `Box<T>`
- `std::shared_ptr<T>` ↔ `Arc<T>`
- Templates ↔ Generics
- Virtual functions ↔ Trait objects
- Manual memory ↔ Ownership system

**Time**: 4-13 weeks depending on depth

---

### [For Rust Developers Using Praxis](rust-developers.md)

**Best for**: Rust developers building games, contributing to Praxis, or learning engine architecture

**You'll Learn**:
- bevy_ecs mastery (archetypes, queries, resources)
- Vulkano rendering patterns
- Transform hierarchy and animation systems
- Rapier3D physics integration
- Lua scripting integration (mlua)
- Tokio async networking
- SIMD and performance optimization
- Praxis subsystem architecture

**Unique Focus**:
- Leveraging Rust's strengths (fearless concurrency, zero-cost abstractions)
- Understanding Praxis's 19-crate architecture
- Compile-time safety in real-time systems
- Advanced optimization techniques

**Time**: 4-12 weeks depending on depth

---

## Choosing Your Path

### I'm coming from Unity and want to understand engines
→ Start with **[Unity Developers](unity-developers.md)**

**Recommended Order**:
1. ECS fundamentals (understand GameObject → Entity mapping)
2. Rendering pipeline (see what MeshRenderer automates)
3. Transform hierarchy (manual propagation vs automatic)
4. Physics integration (bidirectional sync patterns)

### I'm building a custom engine in C++
→ Start with **[C++ Programmers](cpp-programmers.md)**

**Recommended Order**:
1. Game loop implementation (fixed timestep patterns)
2. Vulkan rendering architecture
3. ECS implementation (EnTT or custom)
4. Transform hierarchy optimization
5. Physics integration (PhysX/Jolt)

### I'm using Praxis or learning Rust gamedev
→ Start with **[Rust Developers](rust-developers.md)**

**Recommended Order**:
1. bevy_ecs mastery
2. Vulkano rendering
3. Transform + animation systems
4. Physics (Rapier3D)
5. Scripting (mlua) or Networking (tokio)

### I want language-agnostic engine concepts
→ Use **[Course Curriculum](../CURRICULUM.md)** directly

The curriculum teaches universal patterns applicable to any language, while these paths show language-specific implementations.

---

## How These Paths Work

### Structure

Each path follows this pattern:

1. **Conceptual Mappings**: Translate familiar concepts to universal patterns
2. **Code Comparisons**: Side-by-side examples in your language vs Praxis
3. **Learning Modules**: Align with [Course Curriculum](../CURRICULUM.md) modules
4. **Practical Exercises**: Hands-on tasks to reinforce learning
5. **Projects**: Build real systems to apply knowledge
6. **Best Practices**: Language-specific idioms and gotchas

### Integration with Main Documentation

These paths complement existing Praxis documentation:

- **Paths** (this section): Role-specific entry points with language mappings
- **[Course Curriculum](../CURRICULUM.md)**: Universal engine concepts (language-agnostic)
- **[Code Examples](../CODE_EXAMPLES.md)**: Side-by-side implementations (Rust, C++, C#)
- **[Language Guide](../LANGUAGE_GUIDE.md)**: Detailed language translation reference
- **[Guides](../../guides/)**: Task-oriented how-to documentation
- **[Concepts](../../concepts/)**: Theoretical foundations
- **[Reference](../../reference/)**: API documentation

### Prerequisites

All paths assume:
- ✅ Programming proficiency in your chosen language
- ✅ Basic 3D math (vectors, matrices, quaternions)
- ✅ Understanding of game development concepts
- ✅ Familiarity with your language's ecosystem (build tools, package managers)

Specific prerequisites per path listed in each guide.

---

## Cross-Path Comparison

### Feature Coverage

| Topic | Unity Path | C++ Path | Rust Path |
|-------|------------|----------|-----------|
| ECS Architecture | ✅ GameObject → ECS | ✅ EnTT implementation | ✅ bevy_ecs mastery |
| Rendering | ✅ MeshRenderer comparison | ✅ Vulkan direct API | ✅ Vulkano patterns |
| Transform Hierarchy | ✅ Automatic vs manual | ✅ Dirty flag optimization | ✅ Change detection |
| Physics Integration | ✅ Rigidbody sync | ✅ PhysX/Jolt | ✅ Rapier3D |
| Memory Management | ✅ GC vs ownership | ✅ Smart pointers, RAII | ✅ Ownership deep dive |
| Scripting | ✅ C# vs Lua | ✅ Lua integration | ✅ mlua bindings |
| Networking | ✅ Netcode patterns | ✅ ENet/custom | ✅ Tokio async |
| Multithreading | ✅ DOTS parallelism | ✅ Job systems | ✅ Rayon, channels |

### Time Investment

| Path | Fast Track | Deep Dive | Mastery |
|------|------------|-----------|---------|
| Unity Developers | 4 weeks | 8 weeks | 12 weeks |
| C++ Programmers | 4 weeks | 12 weeks | 13 weeks |
| Rust Developers | 4 weeks | 8 weeks | 12 weeks |

### Difficulty Curve

**Unity Developers**:
- Easy: Understanding ECS concepts (familiar with DOTS)
- Medium: Manual rendering pipeline (no automatic features)
- Hard: Memory management (no GC)

**C++ Programmers**:
- Easy: Memory management concepts (familiar with RAII)
- Medium: ECS architecture (new paradigm)
- Hard: Rust ownership system (borrow checker)

**Rust Developers**:
- Easy: Ownership and safety (native to Rust)
- Medium: Game engine patterns (domain-specific knowledge)
- Hard: Real-time optimization (performance constraints)

---

## Universal Patterns Reference

All paths converge on these universal engine patterns:

### Core Architecture
- **Game Loop**: Fixed timestep, variable timestep, accumulator pattern
- **ECS**: Entity-Component-System, archetype storage, queries
- **Transform Hierarchy**: Local/global transforms, dirty propagation
- **Asset Pipeline**: Loading, caching, hot-reload

### Rendering
- **Forward Rendering**: One pass per light
- **Deferred Rendering**: G-buffer + lighting pass
- **PBR Materials**: Metallic-roughness workflow
- **Shadow Mapping**: Cascaded shadow maps

### Simulation
- **Physics Integration**: Bidirectional sync, fixed timestep
- **Animation**: Skeletal animation, blending, IK
- **Scripting**: Embedded languages, hot-reload, ECS bindings
- **Networking**: Client-server, replication, prediction

See [Universal Patterns](../patterns/) for detailed discussions.

---

## Learning Resources

### Documentation
- [Course Curriculum](../CURRICULUM.md) - Universal concepts
- [Code Examples](../CODE_EXAMPLES.md) - Multi-language comparisons
- [Language Guide](../LANGUAGE_GUIDE.md) - Translation reference
- [Glossary](../glossary.md) - Terminology definitions

### Praxis-Specific
- [Architecture](../../architecture.md) - System design
- [Beginner's Guide](../../beginners-guide.md) - Comprehensive intro
- [Guides](../../guides/) - Task-oriented tutorials
- [Examples](../../../examples/) - Runnable code samples

### External Resources
- **Unity DOTS**: [Official Docs](https://docs.unity3d.com/Packages/com.unity.entities@latest)
- **EnTT**: [GitHub](https://github.com/skypjack/entt) - C++ ECS library
- **Bevy**: [Official Site](https://bevyengine.org/) - Rust game engine
- **Game Engine Architecture**: Book by Jason Gregory
- **Foundations of Game Engine Development**: Series by Eric Lengyel

---

## Contribution Guidelines

### Adding a New Path

To create a path for a new language or role:

1. **Identify Target Audience**: Who is this for? What do they already know?
2. **Map Concepts**: Create translation table (their concepts → universal patterns)
3. **Code Examples**: Show side-by-side comparisons
4. **Align with Curriculum**: Reference relevant modules from [Curriculum](../CURRICULUM.md)
5. **Practical Exercises**: Provide hands-on tasks
6. **Best Practices**: Include language-specific gotchas and idioms

### Template Structure

```markdown
# For [Language/Role] [Learning Goal]

## Overview
- Target audience
- Prerequisites
- Learning approach

## Key Conceptual Mappings
- Familiar Concept → Universal Pattern

## Learning Path by Module
- Align with Curriculum modules
- Show language-specific implementations

## Practical Exercises
- Hands-on tasks
- Comparative analysis

## Projects
- Real-world applications

## Resources
- Language-specific libraries
- Ecosystem tools

## Next Steps
- Where to go after completing path
```

---

## Feedback and Updates

These paths are living documents that evolve based on:
- User feedback and common questions
- New language features and ecosystem changes
- Praxis architecture updates
- Industry best practices

**Contribute**: Submit issues or PRs to improve clarity, add examples, or suggest new paths.

---

## Quick Navigation

**By Language/Background**:
- [Unity Developers](unity-developers.md)
- [C++ Programmers](cpp-programmers.md)
- [Rust Developers](rust-developers.md)

**Universal Concepts**:
- [Course Curriculum](../CURRICULUM.md)
- [Code Examples](../CODE_EXAMPLES.md)
- [Universal Patterns](../patterns/)

**Praxis Documentation**:
- [Architecture](../../architecture.md)
- [Beginner's Guide](../../beginners-guide.md)
- [Guides](../../guides/)
- [Reference](../../reference/)
