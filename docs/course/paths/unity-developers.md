# For Unity Developers Learning Engine Architecture

A guided path for Unity developers to understand game engine architecture through Praxis, mapping familiar Unity concepts to universal engine design patterns.

## Overview

This guide helps you transition from Unity development to understanding engine architecture. You already know game development - this path shows you what's happening under the hood and how to apply those concepts in any engine context.

**Target Audience**: Unity developers (C#/MonoBehaviour) who want to understand engine internals, build custom engines, or work across multiple platforms.

**Prerequisites**: 
- Proficiency with Unity (GameObjects, Components, MonoBehaviours)
- C# programming experience
- Basic 3D math (vectors, matrices, quaternions)

**Learning Approach**: We'll map Unity concepts you know to their universal engine equivalents, then show Praxis's Rust implementation as a reference.

---

## Key Conceptual Mappings

### Unity → Universal Engine Concepts

| Unity Concept | Universal Pattern | Praxis Implementation |
|---------------|-------------------|------------------------|
| GameObject | Entity (ECS) | `Entity` (bevy_ecs) |
| Component (MonoBehaviour) | Component (data) + System (logic) | `#[derive(Component)]` struct + system functions |
| Scene hierarchy | Transform hierarchy | `Transform` + `Parent` + `Children` components |
| Update() / FixedUpdate() | Variable/Fixed timestep systems | System scheduling, `Res<Time>` |
| Prefab | Entity spawning pattern | `Commands::spawn()` with component bundle |
| Resources.Load() | Asset loading/caching | `MeshLoader`, `TextureManager` |
| Renderer components | Rendering commands | `DrawCommand` + `RenderContext` |
| Rigidbody | Physics integration | `RigidBody` component + Rapier3D |
| ScriptableObject | Resources (shared data) | `Res<T>` / `ResMut<T>` |
| Event system | Events | `EventReader<T>` / `EventWriter<T>` |

---

## Unity to Engine Architecture: Core Differences

### Architecture Paradigm

**Unity (GameObject/Component)**:
```csharp
public class PlayerController : MonoBehaviour {
    public float speed = 5.0f;
    private Rigidbody rb;
    
    void Start() {
        rb = GetComponent<Rigidbody>();
    }
    
    void Update() {
        float h = Input.GetAxis("Horizontal");
        rb.velocity = new Vector3(h * speed, rb.velocity.y, 0);
    }
}
```

**Universal ECS Pattern** (Praxis):
```rust
// Component: pure data
#[derive(Component)]
struct Velocity {
    value: Vec3,
}

#[derive(Component)]
struct PlayerController {
    speed: f32,
}

// System: pure logic
fn player_movement_system(
    mut query: Query<(&mut Velocity, &PlayerController)>,
    input: Res<InputState>,
    time: Res<Time>,
) {
    for (mut velocity, controller) in query.iter_mut() {
        let h = input.axis_value("Horizontal");
        velocity.value.x = h * controller.speed;
    }
}
```

**Key Differences**:
- Unity: Logic + data together in MonoBehaviour
- ECS: Components = data, Systems = logic, complete separation
- Unity: Object-oriented, inheritance-based
- ECS: Composition-based, archetype storage

**Why ECS?**
- Better cache locality (data-oriented design)
- Easier parallelization (systems don't own data)
- More flexible composition (no inheritance diamond problems)
- Modern engines trend toward ECS (Unity DOTS, Bevy, Flecs)

---

## Learning Path by Module

Refer to the [Course Curriculum](../CURRICULUM.md) for universal concepts. This guide shows Unity-specific mappings.

### Module 1: Game Loop (Maps to Curriculum Module 1)

**Unity Lifecycle**:
```csharp
Awake() → Start() → Update() + FixedUpdate() + LateUpdate() → OnDestroy()
```

**Universal Game Loop** (Praxis):
```rust
loop {
    // 1. Event handling (Unity: automatic)
    // 2. Fixed timestep physics (Unity: FixedUpdate)
    // 3. Variable timestep updates (Unity: Update)
    // 4. Rendering (Unity: automatic after LateUpdate)
}
```

**See**: [Game Loop Patterns](../../course/patterns/game-loop-patterns.md)

### Module 2: Rendering Architecture (Maps to Curriculum Module 2)

**Unity**: MeshRenderer + Material → Automatic rendering  
**Universal**: Explicit draw commands + pipeline management

**Key Learning**: Unity abstracts Vulkan/DX/Metal. Praxis shows direct Vulkan usage.

**See**: 
- [Vulkan Rendering](../../concepts/vulkan-rendering.md)
- [Rendering Guide](../../guides/rendering.md)

### Module 3: ECS Architecture (Maps to Curriculum Module 3)

**Unity GameObject** → **Unity DOTS / Praxis ECS**

| Unity GameObject | Unity DOTS | Praxis |
|------------------|------------|--------|
| `GetComponent<T>()` | `EntityManager.GetComponentData<T>()` | `Query<&T>` |
| `AddComponent<T>()` | `EntityManager.AddComponentData()` | `Commands.spawn()` |
| `foreach (var obj in objects)` | `Entities.ForEach()` | `Query<&T>.iter()` |

**See**: [ECS Architecture](../../concepts/ecs-architecture.md)

### Module 4: Transform Hierarchy (Maps to Curriculum Module 4)

**Unity**: Automatic transform propagation via Transform component  
**Universal**: Manual propagation system with dirty flagging

**Praxis Pattern**:
```rust
// Detect changed transforms
Query<&Transform, Changed<Transform>, Without<Parent>>

// Propagate to children recursively
parent.global * child.local = child.global
```

**See**: [Transform Hierarchy](../../concepts/transform-hierarchy.md)

### Module 5: Physics Integration (Maps to Curriculum Module 5)

**Unity**: Automatic sync between Transform and Rigidbody  
**Universal**: Bidirectional sync systems

**Sync Pattern**:
```
1. ECS → Physics (kinematic bodies)
2. Physics.Step()
3. Physics → ECS (dynamic bodies)
```

**See**: [Physics Guide](../../guides/physics.md)

### Module 6: Asset Pipeline (Maps to Curriculum Module 6)

**Unity**: Import pipeline (FBX → Unity format)  
**Universal**: Runtime loading + GPU upload

**Comparison**:
- Unity: `Resources.Load<T>()` or Addressables
- Praxis: `MeshLoader::load_obj()` + `mesh_manager.add()`

**See**: [Course Language Guide - Asset Loading](../LANGUAGE_GUIDE.md#asset-loading)

### Module 11: Scripting Integration (Maps to Curriculum Module 11)

**Unity**: C# is the engine language (compile-time scripting)  
**Universal**: Embedded language (Lua, Python) for runtime flexibility

**Why Lua in engines?**
- Hot-reload without recompile (Unity domain reload is slow)
- Modding support
- Designer-friendly iteration

**See**: [Scripting Guide](../../guides/scripting.md)

### Module 12: Networking (Maps to Curriculum Module 12)

**Unity Netcode**: NetworkObject + NetworkVariable + ServerRpc  
**Universal**: Entity replication + client prediction + lag compensation

**Pattern Equivalence**:
- Unity `NetworkVariable<T>` → Praxis replicated component
- Unity `[ServerRpc]` → Server command handling
- Unity client prediction → Praxis prediction system

**See**: [Networking Guide](../../guides/systems/networking.md)

---

## Practical Exercises

### Exercise 1: GameObject to ECS Conversion
**Time**: 2-4 hours

1. Create a Unity scene with:
   - Player GameObject with Health, Movement scripts
   - Enemy GameObjects with AI script
   - Item GameObjects with pickup logic

2. Convert to ECS (Unity DOTS or on paper):
   - Components: Health, Velocity, Player tag, Enemy tag
   - Systems: MovementSystem, HealthSystem, AISystem

3. Map to Praxis:
   - Implement components in Rust
   - Write equivalent systems
   - Spawn entities with component bundles

**Learning Outcome**: Understand component/system separation

### Exercise 2: Rendering Pipeline Exploration
**Time**: 3-5 hours

1. In Unity:
   - Create scene with 10 objects, 5 lights
   - Enable Frame Debugger
   - Observe draw calls, batching, culling

2. In Praxis:
   - Run `cargo run --example scene_demo`
   - Read `RenderContext::render()` implementation
   - Trace from `DrawCommand` to GPU submission

3. Compare:
   - Draw call count
   - Culling strategies
   - Material batching

**Learning Outcome**: Understand what Unity automates

### Exercise 3: Transform Hierarchy Implementation
**Time**: 2-3 hours

1. Create hierarchy in Unity: Parent → Child → Grandchild
2. Move parent, observe children in Inspector
3. In Praxis, run `cargo run --example transform_propagation_demo`
4. Study `propagate_transforms()` system
5. Implement simple hierarchy on paper (pseudocode)

**Learning Outcome**: Understand propagation algorithm

---

## Unity DOTS Quick Reference

Unity DOTS is Microsoft's modern ECS. Understanding DOTS helps understand Praxis.

### Component
```csharp
public struct Health : IComponentData {
    public float Current;
    public float Max;
}
```

```rust
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}
```

### System
```csharp
public partial class HealthSystem : SystemBase {
    protected override void OnUpdate() {
        Entities.ForEach((ref Health h) => {
            h.Current = math.min(h.Max, h.Current + 1.0f);
        }).ScheduleParallel();
    }
}
```

```rust
fn health_system(mut query: Query<&mut Health>) {
    for mut h in query.iter_mut() {
        h.current = h.current.min(h.max + 1.0);
    }
}
```

### Query
```csharp
EntityQuery query = GetEntityQuery(typeof(Transform), typeof(Velocity));
```

```rust
Query<(&Transform, &Velocity)>
```

---

## Common Pitfalls

### 1. Expecting Automatic Features
**Unity**: Rendering, physics, transform hierarchy automatic  
**Engine Dev**: Must implement explicitly

**Solution**: Study Praxis systems to see implementation patterns

### 2. Null References
**C#**: Objects can be null  
**Rust**: No null, use `Option<T>`

```csharp
if (player != null) { ... }
```

```rust
if let Some(player) = player_option { ... }
```

### 3. Garbage Collection
**C#**: GC manages memory  
**Rust**: Ownership system, compile-time safety

**Solution**: Learn Rust ownership (see [Language Guide](../LANGUAGE_GUIDE.md))

### 4. Mixing Logic and Data
**MonoBehaviour**: Fields + methods together  
**ECS**: Components = data only, Systems = logic only

**Solution**: Always separate data from behavior

---

## Recommended Study Order

### 4-Week Fast Track
```
Week 1: ECS fundamentals (Module 3)
Week 2: Rendering pipeline (Module 2)
Week 3: Transform + Physics (Modules 4-5)
Week 4: Assets + Scripting (Modules 6, 11)
```

### 8-Week Deep Dive
```
Weeks 1-2: ECS + Game Loop (Modules 1, 3)
Weeks 3-4: Rendering (Module 2 + advanced)
Weeks 5-6: Transform + Physics + Assets (Modules 4-6)
Weeks 7-8: Scripting + Networking (Modules 11-12)
```

### 12-Week Mastery
```
Follow complete [Course Curriculum](../CURRICULUM.md) with Unity comparisons
Build project: Recreate Unity game in Praxis ECS
```

---

## Resources

### Unity to ECS
- [Unity DOTS Documentation](https://docs.unity3d.com/Packages/com.unity.entities@latest)
- [Praxis ECS Concepts](../../concepts/ecs-architecture.md)
- [Code Examples](../CODE_EXAMPLES.md) - Side-by-side comparisons

### Rust for C# Developers
- [Rust Book](https://doc.rust-lang.org/book/)
- [Language Guide](../LANGUAGE_GUIDE.md#c-unity-style)
- Focus: Ownership (Ch 4), Traits (Ch 10), Error Handling (Ch 9)

### Engine Architecture
- [Praxis Architecture](../../architecture.md)
- [Game Engine Architecture by Jason Gregory](https://www.gameenginebook.com/)
- [Curriculum Modules](../CURRICULUM.md) - Universal concepts

---

## Next Steps

After this path:
- ✅ Understand Unity vs ECS architecture
- ✅ Know rendering pipeline internals
- ✅ Can implement transform hierarchy
- ✅ Understand physics integration
- ✅ Can design asset pipeline
- ✅ Know scripting integration patterns

**Continue to**:
- [For C++ Programmers](cpp-programmers.md) - If switching to C++
- [For Rust Developers](rust-developers.md) - Deeper Praxis patterns
- [Course Curriculum](../CURRICULUM.md) - Universal engine concepts
- Build your own engine with learned patterns
