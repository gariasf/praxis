# Language Constraints and API Design

Programming language features fundamentally shape API design. Templates, traits, generics, extension methods, and type systems enable different patterns and impose different constraints. Understanding these language-specific capabilities is essential for designing effective game engine APIs.

## The Core Insight

Game engines solve the same problems across languages, but express solutions differently:

```rust
// Rust: Trait-based generic query
fn system(query: Query<(&Transform, &Velocity)>) {
    for (transform, velocity) in query.iter() { }
}
```

```cpp
// C++: Template-based query
template<typename... Components>
void System(EntityQuery<Components...>& query) {
    query.Each([](Components&... comps) { });
}
```

```csharp
// C#: LINQ-based query
var entities = world.Query<Transform, Velocity>()
    .ForEach((ref Transform t, ref Velocity v) => { });
```

Same concept, different language affordances.

## Rust: Traits and Ownership

### Zero-Cost Abstractions with Traits

Rust's trait system enables compile-time polymorphism without runtime overhead:

=== "Trait-Based Component System"

    ```rust
    // Component marker trait
    pub trait Component: Send + Sync + 'static {}

    // Automatically implement for any suitable type
    impl<T: Send + Sync + 'static> Component for T {}

    // Generic function works with any Component
    pub fn insert<T: Component>(world: &mut World, entity: Entity, component: T) {
        world.insert_component(entity, component);
    }

    // Usage:
    insert(&mut world, entity, Transform::default());
    insert(&mut world, entity, Velocity::default());

    // Compiles to specialized versions—no vtable!
    ```

    **Key advantage**: Type safety + zero overhead. The compiler generates specialized code for each component type.

=== "Trait Bounds for Safety"

    ```rust
    // Require multiple traits
    pub trait SerializableComponent: Component + Serialize + Deserialize {}

    // Only types implementing all three can be used
    pub fn save_component<T: SerializableComponent>(
        component: &T,
        writer: &mut dyn Write,
    ) -> Result<()> {
        serde_json::to_writer(writer, component)?;
        Ok(())
    }

    // Won't compile if Transform doesn't implement Serialize
    // save_component(&transform, &mut file)?;
    ```

### Ownership-Based API Design

Rust's ownership system prevents entire classes of bugs but requires careful API design:

=== "Owned vs Borrowed"

    ```rust
    // Take ownership: Consumes the value
    impl World {
        pub fn spawn(&mut self, bundle: impl Bundle) -> Entity {
            let entity = self.spawn_empty();
            bundle.insert(entity, self);  // bundle is moved
            entity
        }
    }

    // Usage:
    let entity = world.spawn((
        Transform::default(),
        Velocity::default(),
    ));
    // Can't use bundle after this—it was moved

    // Borrow: Doesn't consume
    impl World {
        pub fn query<'a, T: Component>(&'a self) -> Query<'a, T> {
            Query::new(self)
        }
    }

    // Usage:
    let query = world.query::<Transform>();  // Borrows world
    // Can still use world after (when query is dropped)
    ```

=== "Interior Mutability"

    ```rust
    // Problem: Need shared access to World but want to mutate
    pub struct System {
        world: &World,  // Shared reference
    }

    impl System {
        pub fn run(&self) {
            // Can't mutate through &self!
            // self.world.spawn(bundle);  // Error!
        }
    }

    // Solution: Interior mutability with RefCell/Mutex
    use std::cell::RefCell;

    pub struct World {
        entities: RefCell<Vec<Entity>>,  // Inner mutability
    }

    impl World {
        pub fn spawn(&self, bundle: impl Bundle) -> Entity {
            let mut entities = self.entities.borrow_mut();
            // Can mutate through shared reference
            entities.push(Entity::new());
            // ...
        }
    }
    ```

### Lifetime Elision and Query Ergonomics

```rust
// Explicit lifetimes (verbose)
pub fn query<'a, T: Component>(&'a self) -> Query<'a, T> {
    Query { world: &self, marker: PhantomData }
}

// Lifetime elision (compiler infers)
pub fn query<T: Component>(&self) -> Query<'_, T> {
    Query { world: self, marker: PhantomData }
}

// Return type elision
pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.components.iter()
}

// Allows ergonomic chaining:
world.query::<Transform>()
    .iter()
    .filter(|t| t.translation.y > 0.0)
    .map(|t| t.translation)
    .collect()
```

### Proc Macros for Ergonomics

```rust
// Derive macro generates boilerplate
#[derive(Component, Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

// Expands to:
impl Component for Transform {}
// Plus Debug and Clone implementations

// Custom derive for bundles
#[derive(Bundle)]
pub struct TransformBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

// Generates Bundle trait implementation automatically
```

**Praxis example**:
```rust
// From Praxis ECS
#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

## C++: Templates and SFINAE

### Template Metaprogramming

C++ templates enable compile-time computation and type manipulation:

=== "Variadic Templates"

    ```cpp
    // Accept any number of component types
    template<typename... Components>
    class Query {
        template<typename Func>
        void Each(Func&& func) {
            for (auto entity : entities_) {
                // Get each component type
                func(entity.Get<Components>()...);
            }
        }
    };

    // Usage:
    Query<Transform, Velocity> query;
    query.Each([](Transform& t, Velocity& v) {
        t.position += v.linear * dt;
    });

    // Compiler generates specialized code for each component combination
    ```

=== "SFINAE (Substitution Failure Is Not An Error)"

    ```cpp
    // Enable function only if T has a Update method
    template<typename T>
    auto Update(T& component, float dt) 
        -> decltype(component.Update(dt), void())
    {
        component.Update(dt);
    }

    // Enable function only if T is copy-constructible
    template<typename T>
    std::enable_if_t<std::is_copy_constructible_v<T>, void>
    Clone(T& component) {
        T copy = component;
        // ...
    }

    // Won't compile if conditions not met
    ```

=== "Concepts (C++20)"

    ```cpp
    // Define requirements for components
    template<typename T>
    concept Component = std::is_copy_constructible_v<T> 
                     && std::is_move_constructible_v<T>
                     && sizeof(T) <= 256;  // Enforce size limit

    // Constrain template to Components
    template<Component T>
    void Register(World& world) {
        world.RegisterComponent<T>();
    }

    // Clear error if constraint violated
    Register<TooLargeType>(world);  
    // Error: TooLargeType does not satisfy Component concept
    ```

### CRTP (Curiously Recurring Template Pattern)

```cpp
// Base class that knows derived type
template<typename Derived>
class Component {
public:
    void Register(World& world) {
        world.RegisterComponent<Derived>();
    }
    
    Derived& AsDerived() {
        return static_cast<Derived&>(*this);
    }
};

// Derived class inherits from base with itself as parameter
class Transform : public Component<Transform> {
    glm::vec3 position;
    glm::quat rotation;
};

// Enables static polymorphism without virtual functions
```

**Unreal example**:
```cpp
// Unreal's template-based component system
template<typename T>
T* AActor::FindComponentByClass() const {
    for (UActorComponent* Component : Components) {
        if (T* TypedComponent = Cast<T>(Component)) {
            return TypedComponent;
        }
    }
    return nullptr;
}

// Usage:
UStaticMeshComponent* Mesh = Actor->FindComponentByClass<UStaticMeshComponent>();
```

### Perfect Forwarding

```cpp
// Forward arguments perfectly (preserving lvalue/rvalue-ness)
template<typename... Args>
Entity World::Spawn(Args&&... args) {
    Entity entity = CreateEntity();
    (entity.AddComponent(std::forward<Args>(args)), ...);
    return entity;
}

// Usage:
auto entity = world.Spawn(
    Transform{},              // rvalue (moved)
    Velocity{1.0, 2.0, 3.0},  // rvalue (moved)
    my_health                 // lvalue (copied)
);
```

## C#: Generics and Extension Methods

### Generic Constraints

C# generics are more restricted than C++ templates but provide clearer errors:

=== "Where Clauses"

    ```csharp
    // Constrain to classes only
    public class ComponentStore<T> where T : class, IComponent
    {
        private Dictionary<int, T> components = new();

        public void Add(int entityId, T component) 
        {
            components[entityId] = component;
        }

        public T Get(int entityId) 
        {
            return components[entityId];
        }
    }

    // Constrain to value types
    public struct ComponentArray<T> where T : struct, IComponent
    {
        private T[] components;

        public ref T Get(int index) 
        {
            return ref components[index];
        }
    }

    // Constrain to types with parameterless constructor
    public T Create<T>() where T : new()
    {
        return new T();
    }
    ```

=== "Multiple Constraints"

    ```csharp
    public class ComponentQuery<T> 
        where T : class, IComponent, ISerializable, new()
    {
        // T must be:
        // - A class (reference type)
        // - Implement IComponent
        // - Implement ISerializable
        // - Have parameterless constructor
    }
    ```

**Unity example**:
```csharp
// Unity's generic component access
public T GetComponent<T>() where T : Component
{
    // Runtime type check + cast
    foreach (var component in components) 
    {
        if (component is T typedComponent)
            return typedComponent;
    }
    return null;
}

// Usage:
Transform transform = gameObject.GetComponent<Transform>();
```

### Extension Methods

C# extension methods add functionality to existing types without inheritance:

=== "Query Extensions"

    ```csharp
    // Add methods to IEnumerable<Entity>
    public static class EntityQueryExtensions 
    {
        public static IEnumerable<T> WithComponent<T>(
            this IEnumerable<Entity> entities) 
            where T : IComponent
        {
            return entities
                .Where(e => e.HasComponent<T>())
                .Select(e => e.GetComponent<T>());
        }

        public static IEnumerable<Entity> InRadius(
            this IEnumerable<Entity> entities,
            Vector3 center,
            float radius)
        {
            return entities.Where(e => {
                var transform = e.GetComponent<Transform>();
                return Vector3.Distance(transform.Position, center) <= radius;
            });
        }
    }

    // Usage—fluent API on any IEnumerable<Entity>
    var nearbyEnemies = world.Entities
        .WithComponent<Enemy>()
        .InRadius(player.Position, 10f)
        .OrderBy(e => Vector3.Distance(e.Position, player.Position))
        .Take(5);
    ```

=== "Transform Extensions"

    ```csharp
    // Unity Transform is sealed—can't inherit
    // Extension methods add functionality
    public static class TransformExtensions 
    {
        public static Transform SetPosition(
            this Transform transform, Vector3 position)
        {
            transform.position = position;
            return transform;  // Enable chaining
        }

        public static Transform LookAt(
            this Transform transform, Transform target)
        {
            transform.LookAt(target);
            return transform;
        }

        public static Transform SetScale(
            this Transform transform, float scale)
        {
            transform.localScale = Vector3.one * scale;
            return transform;
        }
    }

    // Fluent API on sealed type:
    transform
        .SetPosition(new Vector3(0, 5, 10))
        .LookAt(target)
        .SetScale(2f);
    ```

### Ref Returns and In Parameters

C# 7+ added references for performance-critical code:

```csharp
// Return by reference—avoid copying large structs
public ref Transform GetTransformRef(int entityId)
{
    return ref transformArray[entityId];
}

// Modify in-place
ref Transform transform = ref GetTransformRef(entityId);
transform.Position += velocity * deltaTime;

// In parameter—readonly reference (no copy)
public void ProcessTransform(in Transform transform)
{
    // Can read transform without copying it
    // transform.Position = ...; // Error—readonly!
}
```

## Language Comparison: Query APIs

Same concept, different implementations:

=== "Rust (Bevy)"

    ```rust
    fn movement_system(
        time: Res<Time>,
        mut query: Query<(&mut Transform, &Velocity), Without<Frozen>>,
    ) {
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.linear * time.delta_seconds();
        }
    }
    ```

    **Features used**:
    - Trait bounds (`Query<T: Component>`)
    - Lifetime elision (`Query<'_, ...>`)
    - Tuple types for multiple components
    - Borrow checker ensures safety

=== "C++ (Unreal-style)"

    ```cpp
    template<typename... Components>
    void System(EntityQuery<Components...>& query) {
        query.Each([&](Entity entity, Components&... comps) {
            // Process components
        });
    }

    // Usage:
    System<Transform, Velocity>(query);
    ```

    **Features used**:
    - Variadic templates
    - Parameter packs
    - Lambda captures
    - Template type deduction

=== "C# (Unity ECS)"

    ```csharp
    partial struct MovementSystem : ISystem
    {
        public void OnUpdate(ref SystemState state)
        {
            foreach (var (transform, velocity) in
                SystemAPI.Query<
                    RefRW<LocalTransform>, 
                    RefRO<Velocity>
                >())
            {
                transform.ValueRW.Position += 
                    velocity.ValueRO.Linear * SystemAPI.Time.DeltaTime;
            }
        }
    }
    ```

    **Features used**:
    - Generic constraints
    - Ref struct (zero-copy iteration)
    - Source generators (SystemAPI)
    - Partial types

## Type System Trade-offs

### Rust: Compile-Time Safety

```rust
// Won't compile—borrow checker prevents data races
fn bad_system(mut query1: Query<&mut Transform>, mut query2: Query<&mut Transform>) {
    // Error: cannot borrow world mutably twice
}

// Solution: Ensure disjoint access
fn good_system(
    mut players: Query<&mut Transform, With<Player>>,
    mut enemies: Query<&mut Transform, With<Enemy>>,
) {
    // OK—different query filters guarantee no overlap
}
```

**Trade-off**: Safe by default, but requires fighting the borrow checker.

### C++: Compile-Time Flexibility

```cpp
// Compiles but unsafe—no protection against data races
void BadSystem(Query<Transform>& query1, Query<Transform>& query2) {
    // May access same entities from both queries
    // Undefined behavior if mutations overlap!
}
```

**Trade-off**: Flexible but can have subtle bugs.

### C#: Runtime Safety

```csharp
// Runtime check prevents conflicts
[BurstCompile]
partial struct BadSystem : ISystem 
{
    public void OnUpdate(ref SystemState state) 
    {
        // Throws exception at runtime if another system
        // is already accessing Transform mutably
        foreach (var transform in 
            SystemAPI.Query<RefRW<LocalTransform>>()) 
        {
            // ...
        }
    }
}
```

**Trade-off**: Catches errors late but provides clear error messages.

## Memory Layout and Language Features

### Rust: Repr and Alignment

```rust
// Control memory layout explicitly
#[repr(C)]
pub struct Transform {
    pub translation: Vec3,  // Offset 0
    pub rotation: Quat,     // Offset 12
    pub scale: Vec3,        // Offset 28
}

// Pack tightly (no padding)
#[repr(packed)]
pub struct CompactTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

// Align for SIMD
#[repr(align(16))]
pub struct AlignedVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _padding: f32,
}
```

### C++: Alignas and Packed

```cpp
// Control alignment
struct alignas(16) AlignedTransform {
    glm::vec3 position;
    glm::quat rotation;
};

// Pack without padding
#pragma pack(push, 1)
struct PackedTransform {
    float position[3];
    float rotation[4];
};
#pragma pack(pop)

// Static assertion to verify
static_assert(sizeof(PackedTransform) == 28);
```

### C#: StructLayout

```csharp
// Sequential layout (default for structs)
[StructLayout(LayoutKind.Sequential)]
public struct Transform 
{
    public Vector3 Position;  // 12 bytes
    public Quaternion Rotation;  // 16 bytes
}

// Explicit layout
[StructLayout(LayoutKind.Explicit)]
public struct TransformOptimized 
{
    [FieldOffset(0)]
    public Vector3 Position;
    
    [FieldOffset(16)]  // Explicit alignment
    public Quaternion Rotation;
}
```

## Performance Implications

### Zero-Cost Abstractions (Rust, C++)

```rust
// High-level abstraction
for transform in query.iter_mut() {
    transform.translation.y += 1.0;
}

// Compiles to same assembly as:
for i in 0..transforms.len() {
    transforms[i].translation.y += 1.0;
}
```

**Key**: No runtime overhead for abstractions.

### Runtime Overhead (C#)

```csharp
// Virtual dispatch costs ~1-2ns per call
interface IUpdatable {
    void Update(float dt);
}

class Component : IUpdatable {
    public virtual void Update(float dt) { }
}

// Each call involves vtable lookup
component.Update(dt);  // Indirection through vtable
```

**Mitigation**: Use structs, avoid interfaces in hot paths, use Burst compiler.

## API Design Guidelines by Language

### Rust Best Practices

1. **Use trait bounds for flexibility**
   ```rust
   pub fn insert<T: Component>(entity: Entity, component: T);
   ```

2. **Return owned values for builders**
   ```rust
   pub fn with_scale(mut self, scale: Vec3) -> Self;
   ```

3. **Use lifetimes to prevent dangling references**
   ```rust
   pub fn query<'a>(&'a self) -> Query<'a, T>;
   ```

4. **Leverage derive macros to reduce boilerplate**
   ```rust
   #[derive(Component, Clone, Debug)]
   ```

### C++ Best Practices

1. **Use templates for type safety + performance**
   ```cpp
   template<typename T> void Register();
   ```

2. **Use concepts (C++20) for clear constraints**
   ```cpp
   template<Component T> void Process(T& comp);
   ```

3. **Perfect forwarding for efficiency**
   ```cpp
   template<typename... Args>
   Entity Spawn(Args&&... args);
   ```

4. **RAII for resource management**
   ```cpp
   class QueryGuard {
       ~QueryGuard() { /* cleanup */ }
   };
   ```

### C# Best Practices

1. **Use generics with constraints**
   ```csharp
   T GetComponent<T>() where T : Component;
   ```

2. **Extension methods for fluent APIs**
   ```csharp
   public static Transform SetPosition(this Transform t, Vector3 pos);
   ```

3. **Ref returns for performance**
   ```csharp
   public ref Transform GetTransformRef(int id);
   ```

4. **Source generators for code generation**
   ```csharp
   [GenerateComponent]
   partial struct Transform { }
   ```

## Summary

| Language | Key Features | API Style | Performance | Safety |
|----------|--------------|-----------|-------------|--------|
| **Rust** | Traits, ownership, lifetimes | Zero-cost generic | Excellent | Compile-time |
| **C++** | Templates, SFINAE, concepts | Compile-time polymorphism | Excellent | Opt-in |
| **C#** | Generics, extension methods | Runtime polymorphism | Good | Runtime |

**Key Takeaways**:

1. **Choose language features that match your goals**
   - Safety → Rust traits and ownership
   - Flexibility → C++ templates
   - Productivity → C# generics and extensions

2. **Understand the costs**
   - Zero-cost abstractions (Rust/C++) vs runtime overhead (C#)
   - Compile-time safety vs runtime flexibility

3. **Design APIs idiomatically**
   - Rust: Ownership-based builders, trait bounds
   - C++: Template metaprogramming, perfect forwarding
   - C#: Extension methods, LINQ-style queries

4. **Language constraints are features**
   - They guide you toward better designs
   - Work with the language, not against it

## Related Patterns

- [Builder Patterns](builder-patterns.md) - Language-specific builder idioms
- [Component APIs](component-apis.md) - How generics enable ECS
- [Fluent Interfaces](fluent-interfaces.md) - Method chaining across languages
- [Script Bindings](script-bindings.md) - FFI and language interop
