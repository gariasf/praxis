# ECS vs OOP: Architectural Comparison

<span class="difficulty-badge difficulty-beginner">Beginner</span>

Understanding the fundamental difference between Entity-Component-System (ECS) and Object-Oriented Programming (OOP) architectures is crucial for modern game engine design.

## Core Philosophy

=== "ECS (Data-Oriented)"

    **Composition over Inheritance**
    
    - Entities are just IDs
    - Components are pure data
    - Systems operate on component queries
    - Focus on data layout and cache efficiency
    
    ```rust
    // Entity: just an ID
    let player = world.spawn();
    
    // Components: pure data
    world.insert(player, Position { x: 0.0, y: 0.0 });
    world.insert(player, Velocity { x: 1.0, y: 0.0 });
    world.insert(player, Health { current: 100, max: 100 });
    
    // System: operates on queries
    fn movement_system(query: Query<(&mut Position, &Velocity)>) {
        for (mut pos, vel) in query.iter_mut() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
    ```

=== "OOP (Object-Oriented)"

    **Inheritance and Polymorphism**
    
    - Objects encapsulate data and behavior
    - Inheritance creates type hierarchies
    - Virtual methods enable polymorphism
    - Focus on object relationships
    
    ```cpp
    // Base class
    class Actor {
    protected:
        FVector Position;
        FVector Velocity;
    public:
        virtual void Update(float DeltaTime);
    };
    
    // Derived class
    class Character : public Actor {
    private:
        int32 Health;
    public:
        void Update(float DeltaTime) override {
            Position += Velocity * DeltaTime;
        }
        void TakeDamage(int32 Amount);
    };
    
    // Usage
    Character* Player = new Character();
    Player->Update(0.016f);
    ```

=== "Component-Based (Unity-style)"

    **Hybrid Approach**
    
    - GameObjects hold components
    - Components can have behavior
    - Composition without pure ECS
    - Focus on modularity
    
    ```csharp
    // Component with behavior
    public class MovementComponent : MonoBehaviour {
        public Vector2 velocity;
        
        void Update() {
            transform.position += (Vector3)velocity * Time.deltaTime;
        }
    }
    
    // Component with data
    public class HealthComponent : MonoBehaviour {
        public int current = 100;
        public int max = 100;
        
        public void TakeDamage(int amount) {
            current = Mathf.Max(0, current - amount);
        }
    }
    
    // Usage
    GameObject player = new GameObject();
    player.AddComponent<MovementComponent>();
    player.AddComponent<HealthComponent>();
    ```

## Memory Layout

### ECS: Cache-Friendly Arrays

```
Position Components: [P1, P2, P3, P4, P5, P6...]
Velocity Components: [V1, V2, V3, V4, V5, V6...]
Health Components:   [H1, H2, H3, H4, H5, H6...]
```

✅ Sequential memory access  
✅ Cache-friendly iteration  
✅ Easy to parallelize  
⚠️ Requires component lookup

### OOP: Object Pointers

```
Object1 -> [Position, Velocity, Health, Methods...]
Object2 -> [Position, Velocity, Health, Methods...]
Object3 -> [Position, Velocity, Health, Methods...]
```

✅ Direct data access  
✅ Encapsulation  
⚠️ Cache misses from pointer chasing  
⚠️ Difficult to parallelize

## Feature Comparison

| Feature | ECS | OOP | Unity-Style |
|---------|-----|-----|-------------|
| **Data Layout** | Arrays of components | Objects with pointers | Components on GameObjects |
| **Iteration Speed** | ⚡ Very Fast | 🐌 Slower | 🏃 Medium |
| **Cache Efficiency** | ✅ Excellent | ❌ Poor | 🤔 Varies |
| **Parallelization** | ✅ Easy | ❌ Difficult | ⚠️ Limited |
| **Code Reuse** | Composition | Inheritance | Composition |
| **Learning Curve** | Steep | Gentle | Moderate |
| **Debugging** | Harder | Easier | Moderate |
| **Flexibility** | ✅ High | ⚠️ Limited by hierarchy | ✅ High |

## Example: Player Character

=== "ECS (Rust)"

    ```rust
    // Pure data components
    #[derive(Component)]
    struct Player;

    #[derive(Component)]
    struct Position(Vec2);

    #[derive(Component)]
    struct Velocity(Vec2);

    #[derive(Component)]
    struct Health { current: i32, max: i32 }

    #[derive(Component)]
    struct Sprite(String);

    // Systems operate on queries
    fn player_movement_system(
        input: Res<Input>,
        mut query: Query<&mut Velocity, With<Player>>,
    ) {
        for mut velocity in query.iter_mut() {
            if input.is_pressed(Key::W) {
                velocity.0.y += 1.0;
            }
            // ...
        }
    }

    fn apply_velocity_system(
        mut query: Query<(&mut Position, &Velocity)>,
    ) {
        for (mut pos, vel) in query.iter_mut() {
            pos.0 += vel.0;
        }
    }

    // Create player
    world.spawn((
        Player,
        Position(Vec2::ZERO),
        Velocity(Vec2::ZERO),
        Health { current: 100, max: 100 },
        Sprite("player.png".to_string()),
    ));
    ```

=== "OOP (C++)"

    ```cpp
    // Class hierarchy
    class PlayerCharacter : public Character {
    private:
        FVector2D Position;
        FVector2D Velocity;
        int32 Health;
        int32 MaxHealth;
        UTexture2D* Sprite;

    public:
        void HandleInput(const FInputState& Input) {
            if (Input.IsPressed(EKey::W)) {
                Velocity.Y += 1.0f;
            }
            // ...
        }

        void Update(float DeltaTime) override {
            Position += Velocity * DeltaTime;
            
            // Handle collisions, etc.
        }

        void TakeDamage(int32 Amount) {
            Health = FMath::Max(0, Health - Amount);
        }

        void Render(FCanvas* Canvas) {
            Canvas->DrawTexture(Sprite, Position);
        }
    };

    // Usage
    PlayerCharacter* Player = new PlayerCharacter();
    Player->HandleInput(InputState);
    Player->Update(DeltaTime);
    Player->Render(Canvas);
    ```

=== "Unity (C#)"

    ```csharp
    // Multiple components on GameObject
    public class PlayerController : MonoBehaviour {
        private Rigidbody2D rb;
        private SpriteRenderer sprite;
        private Health health;

        void Start() {
            rb = GetComponent<Rigidbody2D>();
            sprite = GetComponent<SpriteRenderer>();
            health = GetComponent<Health>();
        }

        void Update() {
            // Input handling
            Vector2 input = Vector2.zero;
            if (Input.GetKey(KeyCode.W)) {
                input.y += 1.0f;
            }
            // ...

            rb.velocity = input * 5.0f;
        }
    }

    public class Health : MonoBehaviour {
        public int current = 100;
        public int max = 100;

        public void TakeDamage(int amount) {
            current = Mathf.Max(0, current - amount);
            if (current == 0) {
                Destroy(gameObject);
            }
        }
    }

    // Setup
    GameObject player = new GameObject("Player");
    player.AddComponent<PlayerController>();
    player.AddComponent<Health>();
    player.AddComponent<SpriteRenderer>();
    player.AddComponent<Rigidbody2D>();
    ```

## When to Use Each

### Use ECS When:
- ✅ Performance is critical (10,000+ entities)
- ✅ Parallelization is important
- ✅ Data-oriented design fits your domain
- ✅ You have experienced ECS developers

### Use OOP When:
- ✅ Rapid prototyping is priority
- ✅ Team is familiar with OOP
- ✅ Entity count is low (< 1,000)
- ✅ Clear hierarchies exist

### Use Hybrid (Unity-style) When:
- ✅ Balance of flexibility and familiarity
- ✅ Moderate entity counts (100-10,000)
- ✅ You want composition without ECS complexity
- ✅ Using an engine that supports it

## Performance Comparison

Typical results for 10,000 entities with position, velocity, and health:

| Operation | ECS | OOP | Unity |
|-----------|-----|-----|-------|
| **Iteration** | 0.1ms | 2.5ms | 0.8ms |
| **Cache Misses** | ~5% | ~40% | ~20% |
| **Parallelization** | 8x speedup | 1.2x speedup | 2x speedup |
| **Memory Usage** | Low | High | Medium |

!!! info "Benchmarks are Approximate"
    Actual performance depends on workload, hardware, and implementation quality.

## Common Misconceptions

!!! warning "ECS is Always Faster"
    Not true! For small entity counts (< 100), OOP overhead is negligible. ECS shines with scale.

!!! warning "OOP Can't Be Fast"
    With careful design (data-oriented OOP), you can achieve good performance. It's harder though.

!!! warning "You Must Choose One"
    Hybrid approaches work well. Unity uses components, Unreal uses OOP with some ECS concepts.

## Further Reading

- [ECS Architecture Concepts](../../concepts/ecs-architecture.md)
- [Component Storage Strategies](../../patterns/component-storage-strategies.md)
- [Performance Optimization](../../learning-paths/performance.md)

## Exercises

1. **Implement same feature in both** - Compare code complexity
2. **Profile performance** - Measure iteration speed at different scales
3. **Add new behavior** - See which is easier to extend
4. **Refactor hierarchy** - Try converting OOP to ECS

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="fixed-timestep-physics.html" class="md-button">← Previous: Fixed Timestep Physics</a>
  <a href="../patterns/" class="md-button">Next: Explore Patterns →</a>
</div>
