# Decision Tree: ECS vs Inheritance

```
┌─────────────────────────────────────────────────┐
│ Should I use ECS or Inheritance for my engine? │
└─────────────────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────┐
        │ What's your primary language? │
        └───────────────────────────────┘
                /               \
               /                 \
     Rust/Zig/C              C++/C#/Java
         │                        │
         ▼                        ▼
    ┌─────────┐          ┌──────────────────┐
    │   ECS   │          │ More questions → │
    │ (strong │          └──────────────────┘
    │  rec.)  │                   │
    └─────────┘                   ▼
         │              ┌─────────────────────────┐
         │              │ What's your entity      │
         │              │ count expectation?      │
         │              └─────────────────────────┘
         │                     /          \
         │                    /            \
         │              < 1000 entities   > 10000 entities
         │                   │                     │
         │                   ▼                     ▼
         │          ┌─────────────────┐    ┌─────────┐
         │          │ Either works    │    │   ECS   │
         │          │ More questions→ │    │ (strong │
         │          └─────────────────┘    │  rec.)  │
         │                   │             └─────────┘
         │                   ▼
         │          ┌──────────────────────┐
         │          │ Do you need frequent │
         │          │ behavior changes at  │
         │          │ runtime?             │
         │          └──────────────────────┘
         │                 /        \
         │                /          \
         │              Yes           No
         │               │             │
         │               ▼             ▼
         │          ┌─────────┐  ┌──────────────┐
         │          │   ECS   │  │ Inheritance  │
         │          └─────────┘  │ (simpler for │
         │                       │ fixed types) │
         │                       └──────────────┘
         │
         ▼
    [See detailed comparison below]
```

## Quick Decision Matrix

| Factor | ECS | Inheritance |
|--------|-----|-------------|
| **Performance (many entities)** | ✅ Excellent (cache-friendly) | ❌ Poor (pointer chasing) |
| **Flexibility** | ✅ High (mix components freely) | ❌ Low (rigid hierarchies) |
| **Learning curve** | ⚠️ Steep | ✅ Familiar |
| **Debugging** | ⚠️ Harder (data spread) | ✅ Easier (object-centric) |
| **Language fit (Rust)** | ✅ Natural fit | ❌ Fights borrow checker |
| **Language fit (C++/C#)** | ⚠️ Requires discipline | ✅ Native support |
| **Small projects (<1000 entities)** | ⚠️ Overkill | ✅ Simpler |
| **Large projects (>10000 entities)** | ✅ Scales well | ❌ Performance issues |

## Detailed Analysis

### Choose ECS If:

**✅ High Priority:**
- Working in **Rust** (ownership model aligns with ECS)
- Expecting **>10,000 entities** in scenes
- Need **dynamic composition** (entities gain/lose abilities at runtime)
- Performance is critical (**cache locality** matters)
- Team comfortable with **data-oriented design**

**Example Use Cases:**
- Open-world games with thousands of NPCs
- Particle systems with millions of particles
- Real-time strategy games with large armies
- Simulation games with complex entity interactions

**Pros:**
- **Performance**: Data laid out contiguously, excellent cache utilization
- **Flexibility**: Mix and match components without rigid hierarchies
- **Parallelization**: Systems naturally independent, easy to parallelize
- **Memory efficiency**: No virtual table overhead, tight data packing
- **Composition**: Avoid deep inheritance chains and diamond problems
- **Rust-friendly**: Aligns with ownership and borrowing semantics

**Cons:**
- **Learning curve**: Paradigm shift from OOP thinking
- **Debugging**: Entity state scattered across component tables
- **Boilerplate**: Requires more setup (queries, systems, resources)
- **Indirection**: Finding entity data requires lookups
- **Overkill**: For simple projects with few entity types

**Praxis Implementation:**
```rust
// Define components
#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Velocity(Vec3);

// Create entity with dynamic composition
let entity = commands.spawn()
    .insert(Health(100.0))
    .insert(Velocity(Vec3::ZERO))
    .id();

// System operates on components
fn damage_system(mut query: Query<&mut Health, With<Velocity>>) {
    for mut health in query.iter_mut() {
        health.0 -= 1.0;
    }
}
```

### Choose Inheritance If:

**✅ High Priority:**
- Working in **C++/C#/Java** with strong OOP traditions
- **Small project** (<1000 entities)
- Entity types are **well-defined and static**
- Team familiar with **OOP patterns**
- Need **fast prototyping** with familiar patterns

**Example Use Cases:**
- Small indie games with few entity types
- Educational projects teaching OOP
- Prototypes exploring gameplay concepts
- Games with fixed entity types (chess, card games)

**Pros:**
- **Familiarity**: Most developers know OOP
- **Simplicity**: Object-centric reasoning is intuitive
- **Debugging**: Easy to inspect object state
- **Encapsulation**: Behavior and data bundled together
- **Polymorphism**: Virtual functions for behavior variants
- **Quick start**: Less upfront design needed

**Cons:**
- **Performance**: Poor cache locality, virtual call overhead
- **Inflexibility**: Hard to add cross-cutting concerns
- **Rust incompatibility**: Fights borrow checker (shared mutable state)
- **Coupling**: Deep hierarchies create tight coupling
- **Diamond problem**: Multiple inheritance pitfalls
- **Scalability**: Performance degrades with many entities

**Example (C++):**
```cpp
class GameObject {
public:
    virtual void update(float dt) = 0;
    virtual void render() = 0;
    Transform transform;
};

class Enemy : public GameObject {
    Health health;
    AI ai;
public:
    void update(float dt) override {
        ai.think();
        // Update logic
    }
};
```

## Hybrid Approaches

Some engines combine both:

### Component-Based OOP (Unity classic)
```csharp
// GameObject has components (not full ECS)
public class HealthComponent : MonoBehaviour {
    public float health = 100f;
}

// Still uses inheritance for behavior
public class Enemy : MonoBehaviour {
    private HealthComponent healthComp;
}
```

**When to use:**
- C# projects needing more flexibility than pure inheritance
- Teams transitioning from OOP to ECS
- Games with moderate entity counts (1000-10000)

### ECS with Object Handles
```rust
// Store object-like handles in components
#[derive(Component)]
struct AIAgent {
    brain: Box<dyn AI>, // Object-oriented behavior
}

// But still benefit from ECS iteration
fn ai_system(query: Query<&AIAgent>) {
    for agent in query.iter() {
        agent.brain.think();
    }
}
```

**When to use:**
- Need complex behavior trees or state machines
- Prototyping with ECS structure
- Gradually migrating to full data-oriented design

## Language-Specific Guidance

### Rust
**Strong recommendation: ECS**

Rust's ownership model makes inheritance painful:
- **Borrow checker**: Can't easily have shared mutable references
- **No inheritance**: Traits don't support inheritance hierarchies
- **Move semantics**: Transferring ownership is complex with nested objects

ECS aligns naturally:
- Components are owned by World (single owner)
- Systems borrow components with clear lifetimes
- Queries enforce Rust's borrowing rules at compile time

**Example of inheritance pain in Rust:**
```rust
// This doesn't work well in Rust
trait GameObject {
    fn update(&mut self, dt: f32);
}

// Problems:
// - Trait objects require Box<dyn> (heap allocation)
// - Can't easily borrow multiple objects mutably
// - No inheritance chains
```

### C++
**Recommendation: Depends on project size**

C++ supports both well:
- **Inheritance**: Native support, familiar to most C++ devs
- **ECS**: Libraries like EnTT provide excellent performance

**Choose ECS if:**
- Large-scale project
- Performance-critical
- Team experienced with data-oriented design

**Choose inheritance if:**
- Small-medium project
- Rapid prototyping
- Team prefers OOP

### C#
**Recommendation: Component-based approach**

C# with Unity has popularized a middle ground:
- GameObject + Component model (not full ECS)
- Unity DOTS for high-performance ECS
- Traditional OOP for game logic

**Choose DOTS/ECS if:**
- Need >10,000 entities
- Performance bottlenecks in current approach
- Willing to invest in learning curve

**Choose Unity classic if:**
- Standard game scale
- Leveraging existing Unity assets
- Team familiar with Unity patterns

### Zig/C
**Strong recommendation: ECS or Data-Oriented Design**

Low-level languages benefit from explicit memory layout:
- Manual memory management suits ECS patterns
- No OOP features in C, limited in Zig
- Performance-focused development aligns with ECS

## Migration Strategies

### From Inheritance to ECS

**Step 1: Identify components**
```
GameObject hierarchy:
- Enemy
  - Position
  - Health
  - AI

Becomes:
- Entity with components: Position, Health, AI
```

**Step 2: Extract systems from methods**
```cpp
// Before: method on class
class Enemy {
    void update() { ai.think(); }
};

// After: system operating on components
fn ai_system(query: Query<&mut AI>) {
    for mut ai in query.iter_mut() {
        ai.think();
    }
}
```

**Step 3: Replace hierarchies with composition**
```
// Before: FlyingEnemy : Enemy : GameObject
// After: Entity with {Enemy, Flying, Transform}
```

### From ECS to Inheritance (rare)

Usually happens when:
- Project scope reduced significantly
- Team expertise shifted
- Language changed to OOP-heavy one

Group related components into objects:
```rust
// ECS components
struct Position(Vec3);
struct Rotation(Quat);
struct Scale(Vec3);

// Becomes inheritance
class Transform {
    Vec3 position;
    Quat rotation;
    Vec3 scale;
};
```

## Common Pitfalls

### ECS Pitfalls
1. **Over-componentization**: Making every field a component
   - ❌ Separate Position/Rotation/Scale components
   - ✅ Single Transform component
2. **Singletons as entities**: Using ECS for non-entity data
   - ❌ Camera as entity with special handling
   - ✅ Camera as resource outside ECS
3. **Ignoring cache patterns**: Randomly accessing components
   - ❌ Query many different component combinations
   - ✅ Group frequently accessed components

### Inheritance Pitfalls
1. **Deep hierarchies**: More than 3-4 levels deep
2. **God objects**: Base class with everything
3. **Inappropriate inheritance**: "is-a" vs "has-a" confusion
   - ❌ `Square : Rectangle` (Liskov violation)
   - ✅ `Square { Rectangle rect; }`

## Performance Comparison

### Updating 100,000 entities

**ECS (Praxis with bevy_ecs):**
```
Cache-friendly iteration: ~1-2ms per frame
Parallel systems: ~0.5ms per frame
Memory: ~1.2MB (tight packing)
```

**Inheritance (C++ virtual calls):**
```
Pointer chasing: ~8-15ms per frame
Single-threaded: ~8-15ms (hard to parallelize)
Memory: ~3-5MB (vtable pointers, padding)
```

**Real-world benchmark** (see `benches/ecs_vs_inheritance.md`):
- ECS: 5-10x faster for large entity counts
- Inheritance: Comparable for <1000 entities

## Decision Checklist

Mark your answers, then count checkmarks in each column:

| Question | ECS | Inheritance |
|----------|-----|-------------|
| Using Rust/Zig? | ✓ | |
| Using C++/C#/Java? | | ✓ |
| >10,000 entities expected? | ✓ | |
| <1000 entities expected? | | ✓ |
| Need runtime composition? | ✓ | |
| Fixed entity types? | | ✓ |
| Performance critical? | ✓ | |
| Rapid prototyping? | | ✓ |
| Team knows ECS? | ✓ | |
| Team knows OOP only? | | ✓ |
| Data-oriented focus? | ✓ | |
| Object-oriented focus? | | ✓ |

**Score:**
- **Mostly ECS**: Choose ECS architecture
- **Mostly Inheritance**: Choose inheritance architecture
- **Tied**: Start with simpler inheritance, migrate to ECS if needed

## Recommended Reading

- **ECS:**
  - [Data-Oriented Design](http://www.dataorienteddesign.com/dodbook/)
  - [Overwatch Gameplay Architecture](https://www.youtube.com/watch?v=W3aieHjyNvw)
  - Praxis: `docs/concepts/ecs.md`

- **Inheritance:**
  - Game Programming Patterns - Component Pattern
  - [Component-Based Engine Design](https://www.randygaul.net/2013/05/20/component-based-engine-design/)

- **Comparison:**
  - [ECS FAQ](https://github.com/SanderMertens/ecs-faq)
  - Praxis: `docs/course/comparisons/ecs-vs-oop.md`

## Conclusion

**TL;DR:**
- **Rust engine? → ECS**
- **C++/C# engine, large scale? → ECS**
- **C++/C# engine, small/medium? → Inheritance or component-based OOP**
- **Learning project? → Try both to understand trade-offs**

Both patterns are valid. Choose based on your constraints, not trends. Praxis chose ECS because:
1. Built in Rust (natural fit)
2. Educational focus (teaches modern patterns)
3. Scalability goals (thousands of entities)
4. Performance focus (cache-friendly iteration)

Your choice may differ based on your needs.
