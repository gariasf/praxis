# Component APIs

Component-based architectures are fundamental to modern game engines. How components are created, accessed, and queried determines both the developer experience and runtime performance. This document analyzes component API design patterns across different engine architectures.

## The Core Problem

Game engines need to:

1. **Create entities** with diverse combinations of components
2. **Query entities** by component combinations (e.g., "all entities with Position + Velocity")
3. **Access components** efficiently (both random access and iteration)
4. **Add/remove components** dynamically at runtime
5. **Ensure safety** (no dangling references, no data races)

Different engines solve these problems with radically different APIs.

## Entity Creation Patterns

### 1. Constructor-Based (Traditional OOP)

=== "C++ (Unreal)"

    ```cpp
    // Unreal: Inherit from base class, components as members
    class AMyActor : public AActor {
        GENERATED_BODY()

    public:
        AMyActor() {
            // Create components in constructor
            MeshComponent = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("Mesh"));
            CollisionComponent = CreateDefaultSubobject<USphereComponent>(TEXT("Collision"));
            
            // Configure components
            MeshComponent->SetupAttachment(RootComponent);
            CollisionComponent->InitSphereRadius(50.0f);
        }

    private:
        UPROPERTY(VisibleAnywhere)
        UStaticMeshComponent* MeshComponent;
        
        UPROPERTY(VisibleAnywhere)
        USphereComponent* CollisionComponent;
    };

    // Usage:
    AMyActor* Actor = World->SpawnActor<AMyActor>();
    ```

    **Characteristics**:
    - Components created in constructor
    - Strong typing via class members
    - Inheritance-based hierarchy
    - Pointer-based component access

=== "C# (Unity - Traditional)"

    ```csharp
    // Unity: GameObject + Component pattern
    public class Enemy : MonoBehaviour 
    {
        private Transform transformComponent;
        private Rigidbody rbComponent;
        
        void Awake() 
        {
            // Get required components
            transformComponent = GetComponent<Transform>();
            rbComponent = GetComponent<Rigidbody>();
            
            if (rbComponent == null) 
            {
                rbComponent = gameObject.AddComponent<Rigidbody>();
            }
        }

        void Update() 
        {
            // Use cached components
            transformComponent.position += Vector3.forward * Time.deltaTime;
        }
    }

    // Usage:
    GameObject enemy = new GameObject("Enemy");
    enemy.AddComponent<Enemy>();
    ```

    **Characteristics**:
    - Components added to GameObject
    - Runtime component lookup
    - Component dependencies checked at runtime
    - Reference-based access

**Trade-offs**:

✅ **Strengths**:
- Familiar OOP patterns
- Strong type safety
- Clear ownership
- IDE support (autocomplete)

❌ **Weaknesses**:
- Inflexible—fixed component set per class
- Poor cache locality
- Verbose component access
- Hard to query arbitrary combinations

### 2. Tuple-Based Spawning (ECS)

=== "Rust (Praxis/Bevy)"

    ```rust
    // Praxis: Spawn with tuple of components
    use praxis_ecs::{World, Transform, Name};

    let mut world = World::new();

    // Spawn entity with components as tuple
    let entity = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity::new(1.0, 0.0, 0.0),
        Name::from("Player"),
        Health::new(100),
    ));

    // Spawning many entities
    for i in 0..1000 {
        world.spawn((
            Transform::from_xyz(i as f32, 0.0, 0.0),
            Enemy::default(),
        ));
    }
    ```

    **Characteristics**:
    - Components as values, not pointers
    - Any combination of components
    - Compile-time type checking
    - Zero allocations (components stored inline)

=== "Rust (Bevy Commands)"

    ```rust
    // Bevy: Deferred spawning via Commands
    fn setup(mut commands: Commands) {
        commands.spawn((
            Camera3dBundle::default(),
            Name::new("MainCamera"),
        ));

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(0.5, 0.5, 0.5),
        ));
    }
    ```

    Commands buffer spawning operations and execute them between systems.

**Trade-offs**:

✅ **Strengths**:
- Flexible—any component combination
- Type-safe—compile errors for wrong types
- Efficient—packed storage
- Composable—easy to add/remove components

❌ **Weaknesses**:
- Unfamiliar pattern for OOP developers
- Can't enforce required components at compile time
- No autocomplete for component combinations
- Must query to access components

### 3. Bundle Pattern

=== "Rust (Bevy Bundles)"

    ```rust
    // Bundle: Reusable component groups
    #[derive(Bundle)]
    struct PlayerBundle {
        player: Player,
        transform: Transform,
        global_transform: GlobalTransform,
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        health: Health,
        inventory: Inventory,
    }

    impl Default for PlayerBundle {
        fn default() -> Self {
            Self {
                player: Player,
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
                mesh: Handle::default(),
                material: Handle::default(),
                health: Health::new(100),
                inventory: Inventory::new(),
            }
        }
    }

    // Usage: Spawn with bundle
    commands.spawn(PlayerBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        health: Health::new(150),
        ..default()
    });
    ```

    **Advantage**: Reusable, named component groups with defaults.

=== "Praxis (Transform Bundle)"

    ```rust
    // Praxis uses bundles for transform hierarchy
    use praxis_ecs::TransformBundle;

    let entity = world.spawn((
        TransformBundle::from_xyz(10.0, 0.0, 0.0),
        MeshRenderer::new(mesh_handle),
        Material::new(material_handle),
    ));

    // TransformBundle includes:
    // - Transform (local transform)
    // - GlobalTransform (world transform)
    // - Parent (optional)
    // - Children (optional)
    ```

**Trade-offs**:

✅ **Strengths**:
- Named component groups (self-documenting)
- Sensible defaults
- Reduces boilerplate
- Type-safe

❌ **Weaknesses**:
- Must define bundle types upfront
- Can't compose bundles dynamically
- More types to maintain

### 4. Prefab/Template Pattern

=== "Unity (Prefabs)"

    ```csharp
    // Unity: Prefabs are serialized GameObjects
    // Create prefab in editor with:
    // - Transform
    // - MeshRenderer
    // - BoxCollider
    // - Enemy script

    // Instantiate at runtime:
    public GameObject enemyPrefab;

    void SpawnEnemy(Vector3 position) 
    {
        GameObject enemy = Instantiate(enemyPrefab, position, Quaternion.identity);
        
        // Prefab instance has all components
        enemy.GetComponent<Enemy>().health = 100;
    }
    ```

    **Characteristics**:
    - Defined in editor (visual)
    - Serialized to file
    - Runtime instantiation
    - Can override component values

=== "Praxis (Scene Definitions)"

    ```rust
    // Praxis: Declarative scene files (RON format)
    SceneDefinition(
        name: "Enemy Template",
        entities: [
            (
                name: Some("Enemy"),
                transform: Some((
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                )),
                components: [
                    Enemy(health: 100, speed: 3.0),
                    AI(behavior: "aggressive"),
                ],
            ),
        ],
    )

    // Load and spawn:
    let scene = SceneLoader::new().load_from_file("enemy_template.ron")?;
    let handle = scene_manager.spawn_scene(&mut world, &scene)?;
    ```

**Trade-offs**:

✅ **Strengths**:
- Visual editor workflow
- Serializable/saveable
- Non-programmers can create entities
- Easy to tweak values

❌ **Weaknesses**:
- Less flexible than code
- Merge conflicts in version control
- Can diverge from code expectations
- Runtime overhead (deserialization)

## Component Access Patterns

### 1. Direct Access (OOP)

=== "Unity"

    ```csharp
    public class PlayerController : MonoBehaviour 
    {
        private Transform transformCache;
        private Rigidbody rbCache;

        void Awake() 
        {
            // Cache components for performance
            transformCache = GetComponent<Transform>();
            rbCache = GetComponent<Rigidbody>();
        }

        void Update() 
        {
            // Direct access via cached references
            transformCache.position += Vector3.forward * Time.deltaTime;
            rbCache.AddForce(Vector3.up * 10f);
        }
    }
    ```

    **Pattern**: Each script manages its own component references.

=== "Unreal"

    ```cpp
    class AMyActor : public AActor {
        UPROPERTY()
        UStaticMeshComponent* MeshComponent;

    public:
        void Tick(float DeltaTime) override {
            // Direct access via member pointer
            FVector Location = MeshComponent->GetComponentLocation();
            MeshComponent->SetWorldLocation(Location + FVector(0, 0, 1) * DeltaTime);
        }
    };
    ```

**Trade-offs**:

✅ **Strengths**:
- Simple and direct
- No lookup overhead
- Clear ownership
- IDE autocomplete

❌ **Weaknesses**:
- Must manage references manually
- Nullable (component might not exist)
- Poor cache locality
- Can't easily query multiple entities

### 2. Query-Based Access (ECS)

=== "Rust (Praxis)"

    ```rust
    // System queries components from World
    fn movement_system(
        mut query: Query<(&mut Transform, &Velocity)>,
        time: Res<Time>,
    ) {
        // Iterate all entities with Transform + Velocity
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.0 * time.delta_seconds();
        }
    }

    // Can have multiple queries
    fn collision_system(
        players: Query<(&Transform, &Collider), With<Player>>,
        enemies: Query<(&Transform, &Collider), With<Enemy>>,
    ) {
        for (player_transform, player_collider) in players.iter() {
            for (enemy_transform, enemy_collider) in enemies.iter() {
                if check_collision(player_collider, enemy_collider) {
                    // Handle collision
                }
            }
        }
    }
    ```

    **Characteristics**:
    - Query declares component access
    - Automatic iteration over matching entities
    - Compile-time safety (borrow checker)
    - Parallel execution when queries don't conflict

=== "Rust (Bevy Filters)"

    ```rust
    // Advanced query filtering
    fn damage_system(
        mut query: Query<
            (&mut Health, &Transform),
            (With<Enemy>, Without<Dead>, Without<Invulnerable>)
        >,
    ) {
        for (mut health, transform) in query.iter_mut() {
            // Only enemies that are:
            // - Not dead
            // - Not invulnerable
            health.current -= 10.0;
        }
    }
    ```

=== "C# (Unity DOTS)"

    ```csharp
    partial struct DamageSystem : ISystem 
    {
        public void OnUpdate(ref SystemState state) 
        {
            // Query with filters
            foreach (var (transform, health) in
                SystemAPI.Query<
                    RefRO<LocalTransform>,
                    RefRW<Health>
                >()
                .WithAll<Enemy>()
                .WithNone<Dead, Invulnerable>())
            {
                health.ValueRW.Current -= 10;
            }
        }
    }
    ```

**Trade-offs**:

✅ **Strengths**:
- Excellent cache locality (packed arrays)
- Automatic parallelization
- Declarative—what, not how
- No null checks (query guarantees components exist)

❌ **Weaknesses**:
- Learning curve (different mental model)
- Can't access specific entity easily
- Query overhead for small iterations
- Harder to debug

### 3. Hybrid Access

=== "Praxis (Entity + Query)"

    ```rust
    // Random access by entity ID
    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.translation.y += 1.0;
    }

    // Batch processing via query
    let mut query = world.query::<(&mut Transform, &Velocity)>();
    for (mut transform, velocity) in query.iter_mut(world.inner_mut()) {
        transform.translation += velocity.0 * time.delta_seconds();
    }
    ```

    Supports both random access (for specific entities) and queries (for batch processing).

=== "Unity (Hybrid)"

    ```csharp
    // GameObject/MonoBehaviour for random access
    public Transform player;
    player.position = newPosition;

    // EntityQuery for batch processing (DOTS)
    EntityQuery query = EntityManager.CreateEntityQuery(
        typeof(Translation), 
        typeof(Velocity)
    );
    
    var entities = query.ToEntityArray(Allocator.Temp);
    foreach (var entity in entities) 
    {
        // Batch processing
    }
    ```

## Query Filtering Patterns

### Positive Filters (With)

```rust
// Only entities WITH these components
Query<&Transform, With<Enemy>>
Query<(&Transform, &Health), (With<Player>, With<Alive>)>
```

Matches entities that have Enemy component, but doesn't provide access to it.

### Negative Filters (Without)

```rust
// Only entities WITHOUT these components
Query<&Transform, Without<Dead>>
Query<(&Transform, &Health), (With<Enemy>, Without<Boss>)>
```

Excludes entities with certain components.

### Optional Components

```rust
// Access component if it exists
Query<(&Transform, Option<&Velocity>)>

for (transform, maybe_velocity) in query.iter() {
    let pos = transform.translation;
    
    if let Some(velocity) = maybe_velocity {
        // Entity has velocity
    } else {
        // Entity doesn't have velocity
    }
}
```

### Change Detection

=== "Bevy"

    ```rust
    // Only entities where Transform changed
    fn sync_system(
        query: Query<&Transform, Changed<Transform>>,
    ) {
        for transform in query.iter() {
            // Only process entities with modified Transform
            sync_to_physics(transform);
        }
    }
    ```

=== "Unity DOTS"

    ```csharp
    partial struct SyncSystem : ISystem 
    {
        public void OnUpdate(ref SystemState state) 
        {
            foreach (var transform in
                SystemAPI.Query<RefRO<LocalTransform>>()
                    .WithChangeFilter<LocalTransform>())
            {
                // Only changed entities
            }
        }
    }
    ```

**Optimization**: Avoid processing unchanged entities.

## Component Lifetime Management

### Adding Components

=== "Runtime Addition (Unity)"

    ```csharp
    // Add component at runtime
    GameObject obj = new GameObject();
    Rigidbody rb = obj.AddComponent<Rigidbody>();
    rb.mass = 10f;

    // Check if has component
    if (obj.TryGetComponent<Rigidbody>(out var rigidbody)) 
    {
        rigidbody.AddForce(Vector3.up * 10f);
    }
    ```

=== "Runtime Addition (Praxis)"

    ```rust
    // Insert component into existing entity
    world.insert(entity, Velocity::new(1.0, 0.0, 0.0));

    // Insert multiple
    world.insert(entity, (
        Health::new(100),
        Armor::new(50),
    ));

    // Check if has component
    if world.contains::<Velocity>(entity) {
        // Entity has velocity
    }
    ```

### Removing Components

=== "Unity"

    ```csharp
    // Remove component
    Rigidbody rb = obj.GetComponent<Rigidbody>();
    Destroy(rb);  // Deferred destruction

    // Immediate removal (DOTS)
    EntityManager.RemoveComponent<Velocity>(entity);
    ```

=== "Praxis"

    ```rust
    // Remove single component
    world.remove::<Velocity>(entity);

    // Remove multiple
    world.remove::<(Health, Armor)>(entity);
    ```

### Component Events

=== "Unity (Callbacks)"

    ```csharp
    public class Health : MonoBehaviour 
    {
        // Called when component added
        void Awake() { }

        // Called when component enabled
        void OnEnable() { }

        // Called when component disabled
        void OnDisable() { }

        // Called when component destroyed
        void OnDestroy() { }
    }
    ```

=== "Praxis (Systems)"

    ```rust
    // Detect added components
    fn on_spawn_system(
        query: Query<Entity, Added<Health>>,
    ) {
        for entity in query.iter() {
            println!("Entity {:?} spawned with Health", entity);
        }
    }

    // Detect removed components (via resource or events)
    ```

## Performance Considerations

### Cache Locality

=== "Poor Locality (GameObject)"

    ```
    GameObject A:
      Transform: [ptr] → [x, y, z, rx, ry, rz]
      Velocity:  [ptr] → [vx, vy, vz]
      
    GameObject B:
      Transform: [ptr] → [x, y, z, rx, ry, rz]
      Velocity:  [ptr] → [vx, vy, vz]

    Iteration:
    A.Transform → cache miss
    A.Velocity  → cache miss
    B.Transform → cache miss
    B.Velocity  → cache miss
    ```

    Each component access may be a cache miss (pointer chasing).

=== "Good Locality (ECS)"

    ```
    Transform array: [A.xyz, A.rot, B.xyz, B.rot, C.xyz, C.rot, ...]
    Velocity array:  [A.vel, B.vel, C.vel, ...]

    Iteration:
    Read Transform[0-2] → cache line (3 entities)
    Read Velocity[0-2]  → cache line (3 entities)
    ```

    Sequential access, few cache misses.

**Benchmark results** (from literature):

- ECS iteration: **~10x faster** for component processing
- Random access: **~2x slower** (indirection through entity ID)

### Query Overhead

```rust
// Small queries have overhead
fn process_one_entity(query: Query<&Transform>) {
    for transform in query.iter() {
        // Query setup cost dominates for 1 entity
    }
}

// Amortized over many entities
fn process_many_entities(query: Query<&Transform>) {
    for transform in query.iter() {
        // Query cost amortized over 10,000 entities
    }
}
```

**Rule of thumb**: ECS wins for >100 entities, random access wins for <10.

## Real-World Examples

### Praxis: ECS Integration

```rust
// From examples/ecs_integration.rs
use praxis_ecs::{World, Schedule, Query, Res, Name, Transform};

fn main() -> Result<()> {
    let mut world = World::new();

    // Spawn entities with components
    let cube = world.spawn((
        Name::from("Cube"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Active,
    ));

    // Define system
    fn rotation_system(
        mut query: Query<(&Name, &mut Transform)>,
        time: Res<Time>,
    ) {
        for (name, mut transform) in query.iter_mut() {
            if name.as_str().starts_with("Orbiter") {
                // Rotate orbiters
                let rotation = Quat::from_rotation_y(time.delta_seconds());
                transform.translation = rotation * transform.translation;
            }
        }
    }

    // Schedule and run
    let mut schedule = Schedule::default();
    schedule.add_systems(rotation_system);
    schedule.run(world.inner_mut());

    Ok(())
}
```

### Unity: Traditional Components

```csharp
public class EnemyAI : MonoBehaviour 
{
    [SerializeField] private float moveSpeed = 5f;
    [SerializeField] private float detectionRadius = 10f;
    
    private Transform transformCache;
    private Transform playerTransform;

    void Start() 
    {
        transformCache = transform;
        playerTransform = GameObject.FindWithTag("Player").transform;
    }

    void Update() 
    {
        float distance = Vector3.Distance(
            transformCache.position, 
            playerTransform.position
        );

        if (distance < detectionRadius) 
        {
            // Chase player
            Vector3 direction = (playerTransform.position - transformCache.position).normalized;
            transformCache.position += direction * moveSpeed * Time.deltaTime;
        }
    }
}
```

### Bevy: Advanced Queries

```rust
fn complex_query_system(
    // Multiple queries with different access patterns
    mut players: Query<(&mut Transform, &mut Health), (With<Player>, Without<Dead>)>,
    enemies: Query<&Transform, (With<Enemy>, Without<Dead>)>,
    walls: Query<&Transform, With<Wall>>,
    
    // Resources
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    for (mut player_transform, mut player_health) in players.iter_mut() {
        // Check collisions with enemies
        for enemy_transform in enemies.iter() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            if distance < config.collision_radius {
                player_health.current -= 10.0 * time.delta_seconds();
            }
        }

        // Check collisions with walls
        for wall_transform in walls.iter() {
            // Wall collision logic
        }
    }
}
```

## Design Guidelines

### When to Use ECS

✅ **Use ECS when**:
- Processing many similar entities (>100)
- Entities have diverse component combinations
- Performance is critical (cache locality matters)
- Parallelization is important
- You want data-oriented design

❌ **Don't use ECS when**:
- Few entities (<10)
- Fixed component combinations (RPG character with 50 components)
- Programmers unfamiliar with pattern
- Need inheritance hierarchies

### API Design Principles

**1. Type-safe component access**

```rust
// Good: Compile error if component doesn't exist
let transform: &Transform = query.get(entity)?;

// Bad: Runtime error
let transform = query.get(entity) as Transform;
```

**2. Clear ownership semantics**

```rust
// Good: Clear mutability
Query<&mut Transform>      // Mutable access
Query<&Transform>          // Readonly access

// Bad: Unclear
Query<Transform>  // Is this by value? Reference?
```

**3. Efficient defaults**

```rust
// Good: Zero-cost iteration
for transform in query.iter() { }

// Bad: Unnecessary allocation
let transforms = query.collect::<Vec<_>>();
for transform in transforms { }
```

**4. Composable queries**

```rust
// Good: Combine filters
Query<&Transform, (With<Enemy>, Without<Dead>)>

// Good: Multiple independent queries
fn system(
    enemies: Query<&Transform, With<Enemy>>,
    players: Query<&Transform, With<Player>>,
) { }
```

## Summary

| Pattern | Access | Performance | Flexibility | Safety |
|---------|--------|-------------|-------------|--------|
| **OOP Components** | Direct | Random: Fast, Iteration: Slow | Low | Runtime |
| **ECS Queries** | Query | Random: Slow, Iteration: Fast | High | Compile-time |
| **Bundles** | Query | Iteration: Fast | Medium | Compile-time |
| **Prefabs** | Direct | Medium | High | Runtime |

**Key Insights**:

1. **No silver bullet**: Choose based on your needs
   - Many entities → ECS
   - Few entities → OOP
   - Data-driven → Prefabs

2. **Hybrid approaches work**:
   - Unity: GameObject for editor, ECS for runtime
   - Praxis: Both entity access and queries

3. **API ergonomics matter**:
   - Type safety catches errors early
   - Clear access patterns prevent bugs
   - Good defaults reduce boilerplate

4. **Performance vs usability**:
   - ECS: High performance, learning curve
   - OOP: Familiar, potentially slower
   - Balance based on team and project

## Related Patterns

- [Declarative vs Imperative](declarative-vs-imperative.md) - Entity spawning styles
- [Language Constraints](language-constraints.md) - How generics enable queries
- [Component Storage](../patterns/component-storage-strategies.md) - Memory layout
