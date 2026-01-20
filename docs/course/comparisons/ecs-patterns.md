# ECS Patterns: Multi-Engine Comparison

**Complexity**: Intermediate  
**Curriculum Module**: [Module 3 - Entity Management Systems](../modules/03-entity-management-systems.md)

## Problem Statement

Game engines must manage thousands of game objects (entities) with varying behaviors and data. The fundamental challenge is:

- How do we represent diverse game objects (player, enemies, bullets, particles)?
- How do we efficiently update and query these objects each frame?
- How do we maximize cache performance and enable parallelization?
- How do we provide flexible composition without deep inheritance hierarchies?

## Design Philosophy Comparison

| Engine | Approach | Core Philosophy |
|--------|----------|-----------------|
| **Unity** | Hybrid OOP + ECS (DOTS) | Traditional GameObject/Component model + optional high-performance ECS |
| **Unreal** | Actor-Component Model | Object-oriented with Component composition; performance through Blueprint optimization |
| **Godot** | Node-based Scene Tree | Hierarchical Node system with signals; simplicity over maximum performance |
| **Praxis** | Pure ECS (bevy_ecs) | Data-oriented design; archetype storage for cache efficiency |

## Implementation Examples

### Creating an Entity with Components

#### Unity (C# - Classic GameObject)

```csharp
// Classic GameObject approach (Unity pre-DOTS)
public class PlayerHealth : MonoBehaviour
{
    public float health = 100f;
    public float maxHealth = 100f;

    void Update()
    {
        if (health <= 0)
            Destroy(gameObject);
    }
}

// Usage
GameObject player = new GameObject("Player");
player.AddComponent<PlayerHealth>();
player.AddComponent<Rigidbody>();
player.AddComponent<MeshRenderer>();
```

#### Unity (C# - DOTS/ECS)

```csharp
// Pure ECS approach (Unity DOTS)
using Unity.Entities;

public struct Health : IComponentData
{
    public float Value;
    public float MaxValue;
}

public struct Velocity : IComponentData
{
    public float3 Value;
}

// System to process entities
public partial class HealthSystem : SystemBase
{
    protected override void OnUpdate()
    {
        // Query all entities with Health component
        Entities.ForEach((ref Health health) =>
        {
            if (health.Value <= 0)
            {
                // Entity destruction handled separately
            }
        }).ScheduleParallel();
    }
}

// Creating entities
EntityManager entityManager = World.DefaultGameObjectInjectionWorld.EntityManager;
Entity player = entityManager.CreateEntity();
entityManager.AddComponentData(player, new Health { Value = 100f, MaxValue = 100f });
entityManager.AddComponentData(player, new Velocity { Value = float3.zero });
```

#### Unreal (C++)

```cpp
// Actor-Component architecture
UCLASS()
class APlayerCharacter : public ACharacter
{
    GENERATED_BODY()

public:
    // Component-based approach
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly)
    class UHealthComponent* HealthComponent;

    APlayerCharacter()
    {
        HealthComponent = CreateDefaultSubobject<UHealthComponent>(TEXT("HealthComponent"));
    }
};

// Health Component
UCLASS()
class UHealthComponent : public UActorComponent
{
    GENERATED_BODY()

public:
    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    float Health = 100.0f;

    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    float MaxHealth = 100.0f;

    virtual void TickComponent(float DeltaTime, ELevelTick TickType, 
                               FActorComponentTickFunction* ThisTickFunction) override
    {
        if (Health <= 0.0f)
        {
            GetOwner()->Destroy();
        }
    }
};

// Usage
UWorld* World = GetWorld();
APlayerCharacter* Player = World->SpawnActor<APlayerCharacter>();
```

#### Godot (GDScript)

```gdscript
# Node-based approach
extends CharacterBody3D

class_name Player

# Properties are node-local
var health: float = 100.0
var max_health: float = 100.0
var velocity: Vector3 = Vector3.ZERO

# Child nodes handle specific functionality
@onready var health_bar = $HealthBar
@onready var mesh = $MeshInstance3D

func _ready():
    health_bar.max_value = max_health
    health_bar.value = health

func _process(delta):
    if health <= 0:
        queue_free()  # Deferred deletion

# Usage in scene
# Player (CharacterBody3D)
#   ├─ MeshInstance3D
#   ├─ CollisionShape3D
#   └─ HealthBar (ProgressBar)
```

#### Praxis (Rust)

```rust
// Pure ECS with bevy_ecs
use bevy_ecs::prelude::*;

// Components are pure data
#[derive(Component)]
struct Health {
    value: f32,
    max_value: f32,
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Player;

// Systems are pure logic
fn health_system(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Player>>
) {
    for (entity, health) in query.iter() {
        if health.value <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// Creating entities
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Health { value: 100.0, max_value: 100.0 },
        Velocity(Vec3::ZERO),
    ));
}
```

### Querying Entities

#### Unity (C# - DOTS)

```csharp
// Query entities with specific components
public partial class DamageSystem : SystemBase
{
    protected override void OnUpdate()
    {
        float deltaTime = Time.DeltaTime;

        // Query: All entities with Health but WITHOUT Shield
        Entities
            .WithNone<Shield>()
            .ForEach((ref Health health, in DamageOverTime dot) =>
            {
                health.Value -= dot.DamagePerSecond * deltaTime;
            })
            .ScheduleParallel();
    }
}
```

#### Unreal (C++)

```cpp
// Iterate all actors of a type
void ADamageSystem::Tick(float DeltaTime)
{
    TArray<AActor*> FoundActors;
    UGameplayStatics::GetAllActorsOfClass(GetWorld(), AEnemy::StaticClass(), FoundActors);

    for (AActor* Actor : FoundActors)
    {
        AEnemy* Enemy = Cast<AEnemy>(Actor);
        if (Enemy && Enemy->HealthComponent)
        {
            Enemy->HealthComponent->TakeDamage(DeltaTime * DamagePerSecond);
        }
    }
}

// Or use component iteration (more efficient)
for (TObjectIterator<UHealthComponent> Itr; Itr; ++Itr)
{
    if (UHealthComponent* HealthComp = *Itr)
    {
        // Process component
    }
}
```

#### Godot (GDScript)

```gdscript
# Get all nodes in a group
func apply_poison_damage(delta: float):
    var poisoned_entities = get_tree().get_nodes_in_group("poisoned")
    for entity in poisoned_entities:
        if entity.has_method("take_damage"):
            entity.take_damage(poison_damage * delta)

# Nodes must be added to groups
func _ready():
    add_to_group("poisoned")
    add_to_group("enemies")
```

#### Praxis (Rust)

```rust
// Query with filters
fn damage_system(
    mut query: Query<&mut Health, (With<Enemy>, Without<Shield>)>,
    time: Res<Time>,
) {
    for mut health in query.iter_mut() {
        health.value -= POISON_DAMAGE * time.delta_seconds();
    }
}

// Multiple queries in one system
fn combat_system(
    mut enemies: Query<(&mut Health, &Transform), With<Enemy>>,
    players: Query<&Transform, With<Player>>,
) {
    for (mut health, enemy_transform) in enemies.iter_mut() {
        for player_transform in players.iter() {
            let distance = enemy_transform.translation.distance(player_transform.translation);
            if distance < ATTACK_RANGE {
                health.value -= MELEE_DAMAGE;
            }
        }
    }
}
```

## Memory Layout & Performance

### Unity (DOTS)

```
Archetype: [Translation, Rotation, Health, Velocity]
┌──────────────────────────────────────────────┐
│ Chunk 0 (16KB, fits ~100 entities)           │
│ Entity IDs:   [E0] [E1] [E2] ... [E99]       │
│ Translations: [T0] [T1] [T2] ... [T99]       │ ← Contiguous
│ Rotations:    [R0] [R1] [R2] ... [R99]       │ ← Cache-friendly
│ Healths:      [H0] [H1] [H2] ... [H99]       │
│ Velocities:   [V0] [V1] [V2] ... [V99]       │
└──────────────────────────────────────────────┘
```

**Characteristics**:
- Chunk-based archetype storage
- 16KB chunks for cache efficiency
- Parallel job scheduling with safety checks
- Change filtering for delta updates

### Unreal

```
Traditional Object-Oriented Layout:
Heap Memory (scattered):
  Actor* ──> [VTable* | Components* | Properties | ...]
                  │
                  └──> UHealthComponent* ──> [VTable* | Health | MaxHealth | ...]
                       UMeshComponent*   ──> [VTable* | Mesh* | Materials | ...]
```

**Characteristics**:
- Pointer indirection (cache unfriendly for iteration)
- Garbage collection overhead
- Excellent for complex object behaviors
- Optimized Blueprint VM reduces script overhead

### Godot

```
Node Tree (hierarchical):
Player (Node3D)
  ├─ Health: float = 100.0      ← Properties stored in node
  ├─ MeshInstance3D*            ← Child node (separate allocation)
  └─ CollisionShape3D*          ← Child node (separate allocation)
```

**Characteristics**:
- Tree structure incurs traversal overhead
- Signals/callbacks for inter-node communication
- Optimized for small-to-medium entity counts
- GDScript JIT improves performance in 4.x

### Praxis

```
Archetype Table Storage (bevy_ecs):
Archetype: (Health, Velocity, Transform)
┌─────────────────────────────────────────┐
│ Entity IDs:  [E0] [E1] [E2] [E3] ...    │
│ Healths:     [H0] [H1] [H2] [H3] ...    │ ← SoA layout
│ Velocities:  [V0] [V1] [V2] [V3] ...    │ ← Optimal for SIMD
│ Transforms:  [T0] [T1] [T2] [T3] ...    │ ← Linear iteration
└─────────────────────────────────────────┘
```

**Characteristics**:
- Table-based archetype storage
- Zero-cost abstractions (compile-time)
- Parallel system execution with compile-time safety
- Change detection with minimal overhead

## Trade-Off Analysis

### Unity (Classic)

**Pros**:
- Familiar OOP patterns
- Designer-friendly inspector workflow
- Massive ecosystem and asset store
- Hot-reload in editor

**Cons**:
- Garbage collection pauses
- Poor cache performance at scale
- GetComponent lookups have overhead
- Difficult to parallelize safely

### Unity (DOTS)

**Pros**:
- Excellent cache performance (archetype chunks)
- Burst compiler generates SIMD code
- Safe parallelization via job system
- Scales to millions of entities

**Cons**:
- Steep learning curve
- Limited editor tooling (improving)
- C# value types can be verbose
- Ecosystem still maturing

### Unreal

**Pros**:
- Battle-tested in AAA production
- Blueprint visual scripting accessibility
- Powerful reflection system
- Excellent debugging tools

**Cons**:
- Iteration overhead from OOP design
- Manual memory management complexity
- C++ compile times
- Harder to parallelize actor iteration

### Godot

**Pros**:
- Intuitive scene tree model
- Lightweight and fast to prototype
- Signals provide loose coupling
- Easy to learn for beginners

**Cons**:
- Node overhead doesn't scale to massive entity counts
- GDScript slower than compiled languages (though improving)
- Tree traversal less cache-friendly
- Manual group management

### Praxis

**Pros**:
- Maximum performance from pure ECS + Rust
- Memory safety without garbage collection
- Compile-time parallelization safety
- Minimal runtime overhead

**Cons**:
- ECS mental model takes time to learn
- Rust ownership rules have learning curve
- Less mature tooling than Unity/Unreal
- Educational focus over production polish

## Performance Comparison

### Iteration Performance (10,000 entities)

| Engine | Time (ms) | Notes |
|--------|-----------|-------|
| Unity (Classic) | ~5-8 ms | GetComponent lookups, GC pressure |
| Unity (DOTS) | ~0.5-1 ms | Burst-compiled, parallel jobs |
| Unreal | ~3-6 ms | Actor iteration, virtual calls |
| Godot | ~2-4 ms | Tree traversal, GDScript JIT (4.x) |
| Praxis | ~0.3-0.7 ms | Pure ECS, zero-cost abstractions |

*Note: Benchmarks are illustrative; actual performance depends on workload complexity.*

### Entity Spawning (1,000 entities/frame)

| Engine | Approach | Performance Impact |
|--------|----------|-------------------|
| Unity (Classic) | GameObject.Instantiate() | High (GC allocation) |
| Unity (DOTS) | EntityManager.CreateEntity() | Low (chunk allocation) |
| Unreal | SpawnActor() | Medium (UObject construction) |
| Godot | Node.new() / scene.instantiate() | Medium (node construction) |
| Praxis | Commands.spawn() | Very Low (archetype insertion) |

## When to Use Each Approach

### Choose Unity (Classic) When:
- Building 2D/3D games with moderate entity counts
- Prioritizing designer workflows and rapid prototyping
- Leveraging Unity Asset Store ecosystem
- Team already familiar with C# and Unity

### Choose Unity (DOTS) When:
- Need to simulate tens of thousands of entities
- Performance is critical (large-scale RTS, simulations)
- Building data-oriented systems (AI, physics)
- Willing to invest in ECS learning curve

### Choose Unreal When:
- AAA graphics and production quality are priorities
- Non-programmers use Blueprint extensively
- Complex gameplay mechanics benefit from OOP
- Building first/third-person action games

### Choose Godot When:
- Indie game development with moderate scope
- Open-source philosophy is important
- Simplicity and ease of learning are priorities
- 2D games or stylized 3D graphics

### Choose Praxis (or Pure ECS) When:
- Learning game engine architecture
- Maximum performance is critical
- Memory safety guarantees are important
- Building custom engine from scratch

## Key Takeaways

### Universal Principles

1. **Data-Oriented Design Wins at Scale**: Pure ECS (Unity DOTS, Praxis) outperforms OOP for large entity counts due to cache efficiency

2. **Trade-Off: Flexibility vs. Performance**:
   - OOP (Unreal, Classic Unity) = More intuitive, slower
   - ECS (DOTS, Praxis) = Harder to learn, much faster

3. **Composition Over Inheritance**: All modern engines favor component composition, even if not pure ECS

4. **Archetype Storage is Optimal**: Grouping entities by component set maximizes cache performance

5. **Parallelization Needs Safety**: Rust/DOTS provide compile-time guarantees; Unreal requires manual care

6. **No One-Size-Fits-All**: Choose based on team size, experience, performance requirements, and project scope

### Design Patterns to Steal

- **Archetype Storage**: Group entities with identical component sets for cache efficiency
- **Query-Based Iteration**: Filter entities by component presence/absence
- **Change Detection**: Only update systems when relevant data changes
- **Deferred Operations**: Buffer spawns/despawns to avoid mid-iteration modifications
- **Parallel Systems**: Run independent systems simultaneously

### Common Pitfalls

- **Over-Granular Components**: Too many tiny components increase archetype count (bad for cache)
- **Singleton Components**: Storing global state in components defeats ECS benefits
- **Component Cross-References**: Holding Entity IDs or pointers couples components
- **Premature Optimization**: Start simple; profile before optimizing

## Further Reading

### Unity
- [Unity DOTS Documentation](https://docs.unity3d.com/Packages/com.unity.entities@latest)
- [Entity Component System Primer](https://unity.com/ecs)

### Unreal
- [Actor Component Architecture](https://docs.unrealengine.com/5.0/en-US/actors-and-components-in-unreal-engine/)
- [Gameplay Framework](https://docs.unrealengine.com/5.0/en-US/gameplay-framework-in-unreal-engine/)

### Godot
- [Godot's Node System](https://docs.godotengine.org/en/stable/getting_started/step_by_step/nodes_and_scenes.html)
- [Scene Architecture](https://docs.godotengine.org/en/stable/tutorials/best_practices/scene_organization.html)

### Praxis
- [Praxis ECS Documentation](../../guides/ecs.md)
- [bevy_ecs Book](https://github.com/bevyengine/bevy/tree/main/crates/bevy_ecs)

### General
- [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/)
- [Understanding Data-Oriented Design (Mike Acton)](https://www.youtube.com/watch?v=rX0ItVEVjHc)
