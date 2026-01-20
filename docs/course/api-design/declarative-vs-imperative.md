# Declarative vs Imperative APIs

One of the most fundamental API design decisions in game engines is whether to expose functionality declaratively (describing *what* you want) or imperatively (describing *how* to do it). This choice profoundly affects how developers think about and build games.

## The Core Distinction

**Imperative APIs** specify step-by-step instructions:

```rust
// Imperative: Tell the engine HOW to do it
let entity = world.spawn_empty();
world.insert(entity, Transform::from_xyz(0.0, 0.0, 0.0));
world.insert(entity, MeshRenderer::new(mesh_handle));
world.insert(entity, Material::new(material_handle));

for _ in 0..60 {
    let delta = time.delta_seconds();
    transform.translation.y += velocity * delta;
    schedule.run(&mut world);
}
```

**Declarative APIs** specify desired outcomes:

```rust
// Declarative: Tell the engine WHAT you want
scene! {
    Entity {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        mesh: mesh_handle,
        material: material_handle,
        velocity: Velocity::new(0.0, 10.0, 0.0),
    }
}

// Engine handles when and how to update
app.run();
```

## Pattern Comparison

### Imperative APIs

**Philosophy**: Explicit control over execution flow and state changes.

=== "Rust (Praxis)"

    ```rust
    // Imperative ECS from examples/ecs_integration.rs
    let mut world = World::new();

    // Explicitly spawn entities
    let cube = world.spawn((
        Name::from("Center Cube"),
        TransformBundle::from_xyz(0.0, 0.0, 0.0),
        Active,
    ));

    // Explicitly define system logic
    fn rotation_system(mut query: Query<(&Name, &mut Transform)>, time: Res<Time>) {
        for (name, mut transform) in query.iter_mut() {
            if name.as_str().starts_with("Orbiter") {
                let rotation = Quat::from_rotation_y(time.delta_seconds() * 2.0);
                let pos = transform.translation;
                transform.translation = rotation * pos;
            }
        }
    }

    // Explicitly run systems
    let mut schedule = Schedule::default();
    schedule.add_systems(rotation_system);
    schedule.run(world.inner_mut());
    ```

    **Characteristics**:
    - Full control over execution order
    - Explicit state management
    - Clear data flow
    - Verbose but transparent

=== "C# (Unity)"

    ```csharp
    // Imperative MonoBehaviour scripts
    public class PlayerController : MonoBehaviour 
    {
        public float speed = 5f;

        void Update() 
        {
            // Explicitly read input
            float horizontal = Input.GetAxis("Horizontal");
            float vertical = Input.GetAxis("Vertical");

            // Explicitly calculate movement
            Vector3 direction = new Vector3(horizontal, 0, vertical);
            
            // Explicitly apply to transform
            transform.position += direction * speed * Time.deltaTime;

            // Explicitly handle collision
            if (Physics.Raycast(transform.position, direction, out RaycastHit hit)) 
            {
                // Handle collision
            }
        }
    }
    ```

    Unity's MonoBehaviour is fundamentally imperative—you write the logic.

=== "C++ (Unreal)"

    ```cpp
    // Imperative actor tick
    void AMyActor::Tick(float DeltaTime)
    {
        Super::Tick(DeltaTime);

        // Explicitly calculate new position
        FVector NewLocation = GetActorLocation();
        NewLocation.Z += FMath::Sin(RunningTime) * 100.0f;
        
        // Explicitly set location
        SetActorLocation(NewLocation);

        // Explicitly check overlaps
        TArray<AActor*> OverlappingActors;
        GetOverlappingActors(OverlappingActors, AEnemy::StaticClass());
        
        for (AActor* Actor : OverlappingActors) 
        {
            // Handle overlap
        }
    }
    ```

**Trade-offs**:

✅ **Strengths**:
- Complete control over execution
- Easy to debug (step through code)
- Performance optimization opportunities
- Familiar to most programmers
- Flexible—can implement any logic

❌ **Weaknesses**:
- Verbose and repetitive
- Easy to make mistakes (forgot to update something?)
- Hard to parallelize automatically
- Mixing logic with execution flow
- Difficult to serialize/save state

### Declarative APIs

**Philosophy**: Describe desired state; let the engine determine execution.

=== "Rust (Bevy)"

    ```rust
    // Declarative app builder
    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_systems(Startup, setup)
            .add_systems(Update, (
                movement_system,
                collision_system,
            ).chain())
            .run();
    }

    // Declarative entity spawning
    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.spawn(Camera3dBundle::default());
        
        commands.spawn(SceneBundle {
            scene: asset_server.load("models/player.gltf#Scene0"),
            ..default()
        });
    }

    // Systems declare dependencies via parameters
    fn movement_system(
        time: Res<Time>,
        input: Res<Input<KeyCode>>,
        mut query: Query<&mut Transform, With<Player>>,
    ) {
        // Bevy automatically provides time, input, and query
        // Execution order inferred from parameter dependencies
    }
    ```

    **Characteristics**:
    - Declare what systems need
    - Engine schedules execution
    - Automatic parallelization
    - Concise and composable

=== "Rust (Praxis Scene Files)"

    ```rust
    // Declarative scene definition (RON format)
    SceneDefinition(
        name: "Example Scene",
        metadata: (
            version: "1.0.0",
            description: Some("A declarative scene"),
        ),
        entities: [
            (
                name: Some("Player"),
                transform: Some((
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                )),
                components: [],
                children: [
                    (
                        name: Some("Camera"),
                        transform: Some((
                            position: [0.0, 1.8, 0.0],
                        )),
                    ),
                ],
            ),
        ],
    )
    ```

    Scene files are purely declarative—describe the scene, not how to create it.

=== "HTML/CSS (Web Analogy)"

    ```html
    <!-- Declarative UI -->
    <div class="player" style="position: 0 0 0">
        <camera position="0 1.8 0"></camera>
        <mesh src="player.gltf"></mesh>
    </div>

    <style>
        .player {
            animation: float 2s infinite;
        }

        @keyframes float {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(10px); }
        }
    </style>
    ```

    Describe appearance and behavior; browser handles rendering.

**Trade-offs**:

✅ **Strengths**:
- Concise—focus on *what*, not *how*
- Engine optimizes execution
- Automatic parallelization
- Easy to serialize (save/load)
- Composable and reusable
- Separates data from logic

❌ **Weaknesses**:
- Less control over execution
- Harder to debug (what's running when?)
- Performance may be opaque
- Learning curve (new mental model)
- May not support all use cases

## Hybrid Approaches

Most modern game engines blend both paradigms:

### Praxis: Imperative Systems, Declarative Data

```rust
// Declarative scene definition
let scene = SceneDefinition {
    name: "Game Level".to_string(),
    entities: vec![/* ... */],
};

// Imperative loading
let scene_handle = scene_manager.spawn_scene(&mut world, &scene)?;

// Imperative system definition
fn player_movement_system(
    time: Res<Time>,
    input: Res<Input>,
    mut query: Query<(&mut Transform, &Velocity), With<Player>>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

// Declarative system scheduling
let mut schedule = Schedule::default();
schedule.add_systems((
    player_movement_system,
    camera_follow_system,
));
```

### Unity: Declarative Inspector, Imperative Scripts

```csharp
// Declarative: Expose properties in Inspector
public class Enemy : MonoBehaviour 
{
    [SerializeField] private float health = 100f;
    [SerializeField] private float speed = 3f;
    [SerializeField] private GameObject bulletPrefab;

    // Imperative: Write behavior
    void Update() 
    {
        // Manual logic
        if (health <= 0) 
        {
            Destroy(gameObject);
        }
    }

    // Declarative: Unity calls this automatically
    void OnTriggerEnter(Collider other) 
    {
        if (other.CompareTag("Bullet")) 
        {
            health -= 10f;
        }
    }
}
```

**Unity pattern**: Inspector provides declarative configuration, scripts provide imperative logic.

### Bevy: Declarative Scheduling, Imperative Systems

```rust
// Declarative: Describe system graph
app.add_systems(
    Update,
    (
        input_system,
        (movement_system, animation_system).chain(),
        collision_system,
    )
        .chain()
        .run_if(in_state(GameState::Playing))
);

// Imperative: Write system logic
fn movement_system(/* ... */) {
    for (mut transform, velocity) in query.iter_mut() {
        // Explicit logic
        transform.translation += velocity.linear * time.delta_seconds();
    }
}
```

**Bevy pattern**: Declarative scheduling, imperative behavior.

## Use Case Analysis

### When to Use Imperative APIs

**1. Performance-Critical Code**

```rust
// Imperative gives full control for optimization
fn particle_update_system(
    mut particles: Query<(&mut Transform, &mut Velocity, &mut Lifetime)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    // Manually optimized loop
    for (mut transform, mut velocity, mut lifetime) in particles.iter_mut() {
        // Inline calculations for performance
        velocity.0.y -= 9.81 * dt;  // Gravity
        transform.translation += velocity.0 * dt;
        lifetime.remaining -= dt;
    }
}
```

**2. Complex State Machines**

```csharp
// Imperative is clearer for complex logic
void Update() 
{
    switch (currentState) 
    {
        case State.Idle:
            if (Input.GetKeyDown(KeyCode.Space))
                TransitionTo(State.Jumping);
            break;

        case State.Jumping:
            velocity.y += jumpForce * Time.deltaTime;
            if (IsGrounded())
                TransitionTo(State.Idle);
            break;

        case State.Falling:
            velocity.y -= gravity * Time.deltaTime;
            if (IsGrounded())
                TransitionTo(State.Idle);
            break;
    }
}
```

**3. Dynamic Behavior**

```rust
// Imperative for runtime-determined logic
fn ai_system(
    mut enemies: Query<(&Transform, &mut Velocity, &AIState)>,
    player: Query<&Transform, With<Player>>,
) {
    let player_pos = player.single().translation;
    
    for (transform, mut velocity, ai_state) in enemies.iter_mut() {
        // Dynamic decision making
        let distance = transform.translation.distance(player_pos);
        
        if distance < 5.0 {
            // Chase player
            let direction = (player_pos - transform.translation).normalize();
            velocity.0 = direction * ai_state.chase_speed;
        } else if distance < 20.0 {
            // Patrol
            velocity.0 = ai_state.patrol_direction * ai_state.patrol_speed;
        } else {
            // Idle
            velocity.0 = Vec3::ZERO;
        }
    }
}
```

### When to Use Declarative APIs

**1. Scene Composition**

```rust
// Declarative scene files are more maintainable
SceneDefinition(
    name: "Level 1",
    entities: [
        (
            name: Some("SpawnPoint"),
            transform: Some((position: [0.0, 0.0, 0.0])),
        ),
        (
            name: Some("Enemy1"),
            transform: Some((position: [10.0, 0.0, 5.0])),
            components: [
                EnemyAI(patrol_radius: 10.0),
                Health(100),
            ],
        ),
    ],
)
```

**2. UI Layouts**

```rust
// Declarative UI is more readable
ui.vertical(|ui| {
    ui.heading("Settings");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Volume:");
        ui.add(egui::Slider::new(&mut volume, 0.0..=1.0));
    });
    
    ui.checkbox(&mut fullscreen, "Fullscreen");
    
    if ui.button("Apply").clicked() {
        apply_settings();
    }
});
```

**3. Data Pipelines**

```rust
// Declarative queries are more composable
let results = query
    .filter::<With<Enemy>>()
    .filter::<Without<Dead>>()
    .sorted_by(|a, b| a.distance_to_player.cmp(&b.distance_to_player))
    .take(10);
```

## Real-World Example: Animation

Different engines take different approaches to the same problem:

=== "Declarative (Animation Graph)"

    ```yaml
    # Declarative animation blend tree
    animation_graph:
      blend_tree:
        type: blend2d
        parameters:
          x: movement_speed
          y: movement_direction
        animations:
          - idle: { position: [0, 0] }
          - walk_forward: { position: [1, 0] }
          - walk_backward: { position: [-1, 0] }
          - walk_left: { position: [0, -1] }
          - walk_right: { position: [0, 1] }
    ```

    The engine automatically blends animations based on parameter values.

=== "Imperative (Manual Blending)"

    ```rust
    fn animation_system(
        mut query: Query<(&mut AnimationPlayer, &MovementState)>,
    ) {
        for (mut player, movement) in query.iter_mut() {
            // Manually calculate blend weights
            let speed = movement.velocity.length();
            let direction = movement.velocity.normalize();
            
            if speed < 0.1 {
                player.play("idle");
            } else {
                // Manual blend calculation
                let forward_weight = direction.dot(Vec3::Z).max(0.0);
                let right_weight = direction.dot(Vec3::X).max(0.0);
                
                player.start_blend("walk_forward", forward_weight);
                player.start_blend("walk_right", right_weight);
            }
        }
    }
    ```

    Full control, but more code and potential for bugs.

## Design Guidelines

### Choosing an Approach

**Use Declarative APIs when**:
- ✅ Configuration over computation
- ✅ Data can be serialized/deserialized
- ✅ Users are designers, not programmers
- ✅ Automatic optimization is valuable
- ✅ Composability is important

**Use Imperative APIs when**:
- ✅ Performance is critical
- ✅ Logic is complex or dynamic
- ✅ Users are experienced programmers
- ✅ Debugging clarity is essential
- ✅ Full control is required

**Use Hybrid Approach when**:
- ✅ Both configuration and logic are needed
- ✅ Supporting diverse user skill levels
- ✅ Balancing flexibility and usability
- ✅ Different parts of engine have different needs

### API Evolution Strategy

Start imperative, evolve to declarative as patterns emerge:

```rust
// Phase 1: Imperative
world.spawn_empty();
world.insert(entity, Transform::default());
world.insert(entity, MeshRenderer::new(mesh));

// Phase 2: Helpers reduce boilerplate
world.spawn((
    Transform::default(),
    MeshRenderer::new(mesh),
));

// Phase 3: Bundles for common combinations
world.spawn(MeshBundle {
    transform: Transform::default(),
    mesh: mesh_handle,
    material: material_handle,
});

// Phase 4: Declarative scene files
load_scene("level1.ron");
```

## Performance Implications

### Declarative Overhead

Declarative APIs may introduce overhead:

```rust
// Declarative: Engine decides execution order
app.add_systems(Update, (system_a, system_b, system_c));

// May insert synchronization points between systems
// to ensure safety, even if not strictly necessary
```

### Imperative Optimization

Imperative gives full control:

```rust
// Imperative: Manual optimization
fn combined_system(mut query: Query<(&mut A, &mut B, &mut C)>) {
    // Process all components in one pass
    // No synchronization overhead
    for (mut a, mut b, mut c) in query.iter_mut() {
        update_a(&mut a);
        update_b(&mut b);
        update_c(&mut c);
    }
}
```

**Trade-off**: Declarative sacrifices some performance for safety and ergonomics.

## Language Constraints

### Rust: Declarative Safety

Rust's type system enables safe declarative APIs:

```rust
// Compiler ensures systems can run in parallel
fn system_a(mut a: Query<&mut ComponentA>) { }
fn system_b(mut b: Query<&mut ComponentB>) { }

// These can run in parallel (access different components)
app.add_systems(Update, (system_a, system_b));

// Compiler error if systems conflict
fn system_c(mut a: Query<&mut ComponentA>) { }
// system_a and system_c access same component
// app.add_systems(Update, (system_a, system_c)); // Error!
```

### C#: Declarative Attributes

C# uses attributes for declarative metadata:

```csharp
[RequireComponent(typeof(Rigidbody))]
[RequireComponent(typeof(Collider))]
public class PlayerController : MonoBehaviour 
{
    [SerializeField, Range(0, 10)]
    private float speed = 5f;

    [Header("Audio")]
    [SerializeField]
    private AudioClip jumpSound;

    // Unity uses reflection to enforce requirements
}
```

### C++: Declarative Templates

C++ uses templates for compile-time declarative patterns:

```cpp
// Declarative component requirements
template<typename... Components>
class System {
    void execute(EntityManager& entities) {
        entities.each<Components...>([](auto entity, Components&... comps) {
            // Process entities with all specified components
        });
    }
};

// Usage:
System<Transform, Velocity, Renderable> movementSystem;
```

## Summary

| Aspect | Imperative | Declarative | Hybrid |
|--------|-----------|-------------|--------|
| **Control** | Full | Limited | Balanced |
| **Conciseness** | Verbose | Concise | Medium |
| **Debuggability** | Easy | Hard | Medium |
| **Optimization** | Manual | Automatic | Both |
| **Learning Curve** | Shallow | Steep | Medium |
| **Best For** | Complex logic | Configuration | Most engines |

**Key Insight**: Most successful game engines use hybrid approaches—declarative for configuration and composition, imperative for behavior and logic.

Choose based on:
- **Audience**: Designers prefer declarative, programmers prefer imperative
- **Use case**: Configuration vs logic
- **Performance**: Critical paths may need imperative control
- **Maintainability**: Declarative scales better for large projects

## Related Patterns

- [Builder Patterns](builder-patterns.md) - Declarative construction
- [Fluent Interfaces](fluent-interfaces.md) - Declarative method chains
- [Component APIs](component-apis.md) - Declarative entity composition
- [Script Bindings](script-bindings.md) - Imperative scripting APIs
