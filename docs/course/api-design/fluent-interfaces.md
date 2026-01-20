# Fluent Interfaces

Fluent interfaces use method chaining to create readable, expressive APIs that flow like natural language. They're pervasive in game engines for configuring objects, building queries, and composing transformations.

## The Core Problem

Game engines involve many sequential operations:

```rust
// Imperative style - verbose and disconnected
let mut transform = Transform::default();
transform.translate(Vec3::new(10.0, 0.0, 0.0));
transform.rotate(Quat::from_rotation_y(PI / 4.0));
transform.scale(Vec3::splat(2.0));
entity.set_transform(transform);

// Fluent style - reads like a sentence
entity
    .transform()
    .translate(Vec3::new(10.0, 0.0, 0.0))
    .rotate(Quat::from_rotation_y(PI / 4.0))
    .scale(Vec3::splat(2.0));
```

Fluent interfaces improve readability and reduce intermediate variables while maintaining type safety.

## Pattern Variants

### 1. Self-Returning Methods

**Concept**: Each method returns `self` (or `&mut self`) to enable chaining.

=== "Rust (Praxis)"

    ```rust
    // From Praxis Transform API
    impl Transform {
        pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
            Self {
                translation: Vec3::new(x, y, z),
                ..Default::default()
            }
        }

        pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
            let direction = (target - self.translation).normalize();
            self.rotation = Quat::from_rotation_arc(Vec3::Z, direction);
            self
        }

        pub fn with_scale(mut self, scale: Vec3) -> Self {
            self.scale = scale;
            self
        }
    }

    // Usage:
    let transform = Transform::from_xyz(0.0, 5.0, 10.0)
        .looking_at(Vec3::ZERO, Vec3::Y)
        .with_scale(Vec3::splat(1.5));
    ```

    **Key pattern**: `mut self` (owned), not `&mut self` (borrowed), to consume and return.

=== "C# (Unity)"

    ```csharp
    // Unity Transform (imperative, but chainable extension methods possible)
    public static class TransformExtensions 
    {
        public static Transform SetPosition(this Transform t, Vector3 pos) 
        {
            t.position = pos;
            return t;
        }

        public static Transform LookAt(this Transform t, Vector3 target) 
        {
            t.LookAt(target);
            return t;
        }

        public static Transform SetScale(this Transform t, Vector3 scale) 
        {
            t.localScale = scale;
            return t;
        }
    }

    // Usage:
    transform
        .SetPosition(new Vector3(0, 5, 10))
        .LookAt(Vector3.zero)
        .SetScale(Vector3.one * 1.5f);
    ```

    **Unity pattern**: Extension methods add fluent APIs to existing types.

=== "Rust (Bevy)"

    ```rust
    // Bevy's extensive use of fluent interfaces
    commands.spawn(Camera3dBundle::default())
        .insert(Name::new("MainCamera"))
        .insert(CameraController::default())
        .with_children(|parent| {
            parent.spawn(PointLightBundle {
                transform: Transform::from_xyz(4.0, 8.0, 4.0),
                ..default()
            });
        });

    // Query builder
    let query = world.query_filtered::<&Transform, With<Player>>()
        .iter(&world)
        .map(|transform| transform.translation)
        .collect::<Vec<_>>();
    ```

    **Bevy pattern**: Builder pattern combined with fluent interface for entity construction.

=== "C++ (Unreal)"

    ```cpp
    // Unreal's Blueprint node connections use fluent-style
    UK2Node* Node = Graph->CreateNode<UK2Node_CallFunction>();
    Node->SetFromFunction(Function)
        ->AllocateDefaultPins()
        ->FindPinChecked(TEXT("ReturnValue"))
        ->MakeLinkTo(TargetPin);

    // Actor component attachment (imperative but could be fluent)
    UStaticMeshComponent* Mesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("Mesh"));
    Mesh->SetStaticMesh(MeshAsset);
    Mesh->SetMaterial(0, Material);
    Mesh->SetRelativeLocation(FVector(0, 0, 100));
    ```

    **Unreal pattern**: Less common due to pointer semantics, but used in editor APIs.

**Trade-offs**:

✅ **Strengths**:
- Highly readable—operations read left-to-right, top-to-bottom
- Reduces intermediate variables
- Natural grouping of related operations
- IDE autocomplete guides usage
- Type-safe at each step

❌ **Weaknesses**:
- Can encourage long chains that are hard to debug
- Ownership semantics can be tricky (Rust)
- May allocate if each step clones
- Error handling is awkward (breaks the chain)
- Can hide performance implications

### 2. Builder-Style Fluent Interfaces

**Concept**: Combine builders with fluent chaining for complex configuration.

=== "Rust (Praxis)"

    ```rust
    // Praxis AudioSource from examples/audio_simple.rs
    world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/test.ogg")
            .with_volume(0.7)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));

    // Praxis ScriptingConfig
    let config = ScriptingConfig::default()
        .with_sandbox_level(SandboxLevel::Moderate)
        .with_max_execution_time(100)
        .with_enable_hot_reload(true);
    ```

=== "C# (Unity)"

    ```csharp
    // Unity UI builder pattern
    var button = new GameObject("Button")
        .AddComponent<Button>()
        .Setup(btn => {
            btn.onClick.AddListener(() => Debug.Log("Clicked!"));
        })
        .GetComponent<Image>()
        .Setup(img => {
            img.color = Color.blue;
            img.sprite = buttonSprite;
        });

    // LINQ query builder (built into C#)
    var enemies = entities
        .Where(e => e.HasComponent<Enemy>())
        .Where(e => e.health > 0)
        .OrderBy(e => Vector3.Distance(player.position, e.position))
        .Take(10)
        .ToList();
    ```

    **C# pattern**: LINQ provides fluent query API throughout the language.

=== "Rust (Bevy)"

    ```rust
    // Bevy app builder
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            movement_system,
            collision_system,
        ).chain())
        .add_state::<GameState>()
        .run();

    // Material builder
    materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        metallic: 0.1,
        perceptual_roughness: 0.5,
        ..default()
    });
    ```

**Bevy pattern**: Heavy use of fluent APIs for app configuration and system scheduling.

### 3. Closure-Based Fluent Interfaces

**Concept**: Accept closures to configure nested objects fluently.

=== "Rust (Bevy)"

    ```rust
    // Bevy's with_children for hierarchy
    commands.spawn(SpatialBundle::default())
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                ..default()
            });
        })
        .insert(Name::new("Parent"));
    ```

    The closure receives a builder for the child scope, enabling nested configuration.

=== "C#"

    ```csharp
    // C# fluent API with Action<T>
    public class UIBuilder 
    {
        public UIBuilder Panel(Action<PanelBuilder> configure) 
        {
            var panelBuilder = new PanelBuilder();
            configure(panelBuilder);
            AddChild(panelBuilder.Build());
            return this;
        }
    }

    // Usage:
    new UIBuilder()
        .Panel(panel => {
            panel.SetColor(Color.white);
            panel.Button(btn => {
                btn.SetText("Click Me");
                btn.OnClick(() => Debug.Log("Clicked"));
            });
        });
    ```

=== "Rust"

    ```rust
    // Praxis-style approach
    pub fn with_children<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut ChildBuilder),
    {
        let mut builder = ChildBuilder::new(&mut self.world);
        f(&mut builder);
        builder.attach_to(self.entity);
        self
    }
    ```

**Trade-offs**:

✅ **Strengths**:
- Natural nesting for hierarchical data
- Keeps related configuration together
- Type-safe scope management
- Clear parent-child relationships

❌ **Weaknesses**:
- Can be deeply nested and hard to follow
- Error handling across closures is complex
- Harder to test individual pieces
- Closure capture can be confusing

### 4. Method Reference Chaining

**Concept**: Chain methods that return different types, each with their own fluent API.

=== "Rust"

    ```rust
    // Each method returns a different type
    struct QueryBuilder<'w> { world: &'w World }
    struct FilteredQuery<'w, T> { /* ... */ }
    struct SortedQuery<'w, T> { /* ... */ }

    impl<'w> QueryBuilder<'w> {
        fn filter<T>(self) -> FilteredQuery<'w, T> {
            FilteredQuery { /* ... */ }
        }
    }

    impl<'w, T> FilteredQuery<'w, T> {
        fn sort_by(self) -> SortedQuery<'w, T> {
            SortedQuery { /* ... */ }
        }
    }

    impl<'w, T> SortedQuery<'w, T> {
        fn iter(self) -> impl Iterator<Item = T> + 'w {
            // ...
        }
    }

    // Usage - each step changes type
    let results = query_builder
        .filter::<Transform>()
        .sort_by(|a, b| a.translation.x.cmp(&b.translation.x))
        .iter()
        .collect::<Vec<_>>();
    ```

=== "C# (LINQ)"

    ```csharp
    // LINQ methods return different IEnumerable types
    var query = entities
        .Where(e => e.HasComponent<Enemy>())    // Returns IEnumerable<Entity>
        .Select(e => e.GetComponent<Transform>()) // Returns IEnumerable<Transform>
        .OrderBy(t => t.position.x)              // Returns IOrderedEnumerable<Transform>
        .ToList();                                // Returns List<Transform>
    ```

**Key insight**: Type changes guide valid operations. Can't call `Select` after `ToList`.

## Real-World Examples

### Praxis: Scene Hierarchy

```rust
// From examples/scene_demo.rs
let scene = SceneDefinition {
    name: "Example Scene".to_string(),
    metadata: SceneMetadata {
        version: "1.0.0".to_string(),
        description: Some("Demo scene".to_string()),
        ..Default::default()
    },
    entities: vec![
        EntityDefinition {
            name: Some("Player".to_string()),
            transform: Some(TransformDef::from_xyz(0.0, 0.0, 0.0)),
            children: vec![
                EntityDefinition {
                    name: Some("Camera".to_string()),
                    transform: Some(TransformDef::from_xyz(0.0, 1.8, 0.0)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    ],
};
```

Struct literals with `..Default::default()` provide a fluent-like experience.

### Unity: Animation Curves

```csharp
// Unity AnimationCurve fluent manipulation
var curve = new AnimationCurve();
curve.AddKey(0f, 0f);
curve.AddKey(0.5f, 1f);
curve.AddKey(1f, 0f);

// Each AddKey returns int (key index), breaking fluency
// Extension methods can restore it:
public static AnimationCurve AddKeyFluent(
    this AnimationCurve curve, float time, float value)
{
    curve.AddKey(time, value);
    return curve;
}

var curve = new AnimationCurve()
    .AddKeyFluent(0f, 0f)
    .AddKeyFluent(0.5f, 1f)
    .AddKeyFluent(1f, 0f);
```

### Bevy: System Scheduling

```rust
// Bevy's schedule builder is deeply fluent
app.add_systems(
    Update,
    (
        player_input_system,
        player_movement_system,
        camera_follow_system,
    )
        .chain()
        .run_if(in_state(GameState::Playing))
        .before(render_system)
        .after(physics_system)
);

// Conditional system sets
app.add_systems(
    Update,
    (health_regen_system, mana_regen_system)
        .distributive_run_if(resource_exists::<Player>())
        .in_set(RegenerationSet)
);
```

### Unreal: Widget Blueprint

```cpp
// Unreal widget construction (fluent-style factory)
UUserWidget* Widget = CreateWidget<UMyWidget>(GetWorld());
Widget->AddToViewport(0);

// Property chaining for widget components
UTextBlock* Text = WidgetTree->ConstructWidget<UTextBlock>(UTextBlock::StaticClass());
Text->SetText(FText::FromString("Hello"))
    ->SetColorAndOpacity(FLinearColor::White)
    ->SetFont(FSlateFontInfo(Font, 24));
```

## Design Guidelines

### When to Use Fluent Interfaces

✅ **Use fluent interfaces when**:
- Operations naturally form a sequence
- Users configure complex objects step-by-step
- Readability is more important than terseness
- Each step has a clear, single responsibility
- You want to guide users through a workflow

❌ **Avoid fluent interfaces when**:
- Operations are independent (no natural order)
- Error handling is complex (breaks the chain)
- Performance is critical (chaining may allocate)
- Methods have no natural grouping
- APIs are already simple enough

### Best Practices

**1. Keep chains reasonably short**

```rust
// Good: 3-4 steps is readable
let transform = Transform::from_xyz(0.0, 5.0, 10.0)
    .looking_at(Vec3::ZERO, Vec3::Y)
    .with_scale(Vec3::splat(2.0));

// Bad: Too long, hard to debug
let entity = world.spawn_empty()
    .insert(Transform::default())
    .insert(Velocity::default())
    .insert(Health::new(100))
    .insert(Name::new("Player"))
    .insert(Inventory::new())
    .insert(Equipped::default())
    .insert(Skills::default())
    .with_children(|parent| { /* ... */ });

// Better: Break into logical groups
let player = world.spawn((
    Transform::default(),
    Velocity::default(),
    Health::new(100),
    Name::new("Player"),
));

player.insert(Inventory::new())
    .insert(Equipped::default())
    .insert(Skills::default());
```

**2. Provide both fluent and non-fluent alternatives**

```rust
// Fluent
let source = AudioSource::new("sound.ogg")
    .with_volume(0.7)
    .with_looping(true);

// Direct field access for advanced users
let mut source = AudioSource::new("sound.ogg");
source.volume = 0.7;
source.looping = true;
```

**3. Use meaningful method names**

```rust
// Good: Verbs that describe actions
transform.translate(delta).rotate(angle).scale(factor);

// Bad: Generic names
transform.set(delta).apply(angle).modify(factor);
```

**4. Return `self` consistently**

```rust
impl Transform {
    // Good: Always returns Self for chaining
    pub fn translate(mut self, delta: Vec3) -> Self {
        self.translation += delta;
        self
    }

    pub fn rotate(mut self, rotation: Quat) -> Self {
        self.rotation *= rotation;
        self
    }
}

// Bad: Inconsistent returns break chaining
impl Transform {
    pub fn translate(mut self, delta: Vec3) -> Self { /* ... */ }
    pub fn rotate(mut self, rotation: Quat) { /* void return! */ }
}
```

**5. Handle errors gracefully**

```rust
// Option 1: Return Result, breaking the chain
pub fn with_valid_volume(mut self, volume: f32) -> Result<Self> {
    if volume < 0.0 || volume > 1.0 {
        return Err(Error::InvalidVolume);
    }
    self.volume = volume;
    Ok(self)
}

// Usage:
let source = AudioSource::new("sound.ogg")
    .with_valid_volume(0.7)?;  // Propagates error

// Option 2: Panic on invalid input (debug only)
pub fn with_volume(mut self, volume: f32) -> Self {
    debug_assert!(volume >= 0.0 && volume <= 1.0);
    self.volume = volume.clamp(0.0, 1.0);
    self
}

// Option 3: Return Option, use ? operator
pub fn with_parent(mut self, parent: Entity) -> Option<Self> {
    if self.world.contains(parent) {
        self.parent = Some(parent);
        Some(self)
    } else {
        None
    }
}
```

## Performance Considerations

### Zero-Cost Fluent Chains

Well-designed fluent interfaces compile to the same code as direct mutation:

```rust
// Fluent
let transform = Transform::default()
    .with_translation(Vec3::new(1.0, 2.0, 3.0))
    .with_rotation(Quat::IDENTITY)
    .with_scale(Vec3::ONE);

// Compiles to same code as:
let mut transform = Transform::default();
transform.translation = Vec3::new(1.0, 2.0, 3.0);
transform.rotation = Quat::IDENTITY;
transform.scale = Vec3::ONE;
```

**Key**: Use `mut self`, not `&mut self`, and return by value. The compiler elides copies.

### When Fluent Interfaces Have Overhead

**1. Unnecessary clones**

```rust
// Bad: Clones on each step
impl Transform {
    pub fn translate(&self, delta: Vec3) -> Self {
        let mut result = self.clone();  // Allocation!
        result.translation += delta;
        result
    }
}

// Good: Move self
impl Transform {
    pub fn translate(mut self, delta: Vec3) -> Self {
        self.translation += delta;
        self  // No allocation
    }
}
```

**2. Heap allocations in builders**

```rust
// Potential allocations
let query = entities
    .filter(predicate)      // May allocate Vec internally
    .map(transform_fn)      // May allocate intermediate results
    .collect::<Vec<_>>();   // Final allocation

// Optimized: Use iterators, lazy evaluation
let query = entities
    .filter(predicate)
    .map(transform_fn);  // No allocation yet

for item in query {  // Evaluate lazily
    process(item);
}
```

## Language-Specific Patterns

### Rust: Ownership and Lifetimes

```rust
// Owned fluent interface (consumes self)
impl Transform {
    pub fn translate(mut self, delta: Vec3) -> Self {
        self.translation += delta;
        self
    }
}

// Borrowed fluent interface (mutation)
impl TransformMut<'a> {
    pub fn translate(&mut self, delta: Vec3) -> &mut Self {
        self.translation += delta;
        self
    }
}

// Usage:
let transform = Transform::default().translate(delta);  // Owned
transform_mut.translate(delta).rotate(angle);  // Borrowed
```

### C#: Extension Methods

```csharp
// Extension methods add fluent APIs to sealed types
public static class TransformExtensions 
{
    public static Transform WithPosition(this Transform t, Vector3 pos) 
    {
        t.position = pos;
        return t;
    }

    public static Transform WithRotation(this Transform t, Quaternion rot) 
    {
        t.rotation = rot;
        return t;
    }
}

// Works on Unity's sealed Transform class
transform
    .WithPosition(new Vector3(0, 5, 10))
    .WithRotation(Quaternion.identity);
```

### C++: Pointer Semantics

```cpp
// C++ fluent interfaces often return pointers or references
class Transform {
    glm::vec3 translation_;

public:
    Transform* translate(const glm::vec3& delta) {
        translation_ += delta;
        return this;  // Return pointer for chaining
    }

    Transform& rotate(const glm::quat& rotation) {
        // Return reference (preferred)
        return *this;
    }
};

// Usage:
transform.translate(delta)->rotate(rotation);  // Pointer
transform.translate(delta).rotate(rotation);   // Reference (better)
```

## Anti-Patterns to Avoid

### 1. Breaking the Chain Unexpectedly

```rust
// Bad: Inconsistent return types
impl Builder {
    pub fn with_name(mut self, name: String) -> Self { /* ... */ }
    pub fn with_age(mut self, age: u32) { /* void! */ }
}

// Can't chain:
let builder = Builder::new()
    .with_name("Alice")
    .with_age(30);  // Error: with_age returns ()
```

### 2. Hidden Mutations

```rust
// Bad: Fluent API with surprising side effects
impl World {
    pub fn spawn_entity(mut self) -> Self {
        self.entities.push(Entity::new());
        self.save_to_disk();  // Surprise! Hidden I/O
        self
    }
}
```

**Guideline**: Fluent methods should be pure or have obvious effects.

### 3. Overly Deep Nesting

```rust
// Bad: Callback hell
builder
    .with_child(|child| {
        child.with_component(|comp| {
            comp.with_property(|prop| {
                prop.with_value(|val| {
                    val.set(42);
                });
            });
        });
    });

// Better: Flatten with named builders
let child = ChildBuilder::new()
    .component(ComponentBuilder::new()
        .property("value", 42));
builder.add_child(child);
```

### 4. Unclear Ownership

```rust
// Bad: Ambiguous ownership in chain
impl EntityBuilder {
    pub fn with_component(self, component: impl Component) -> Self {
        // Does this move component? Clone it? Store reference?
        // Not clear from API
    }
}

// Good: Clear ownership
impl EntityBuilder {
    pub fn add_component(mut self, component: impl Component) -> Self {
        self.components.push(Box::new(component));  // Takes ownership
        self
    }
}
```

## Summary

| Pattern | Best For | Trade-off |
|---------|----------|-----------|
| Self-Returning | Configuration, transformation | Readability vs performance |
| Builder-Style | Complex object construction | Ergonomics vs verbosity |
| Closure-Based | Hierarchical configuration | Nesting vs clarity |
| Type-Changing | Query pipelines | Type safety vs flexibility |

Fluent interfaces excel at:
- Making APIs self-documenting
- Guiding users through workflows
- Reducing intermediate variables
- Creating DSL-like experiences

Choose fluent interfaces when readability and discoverability outweigh the costs of potential overhead and debugging complexity.

## Related Patterns

- [Builder Patterns](builder-patterns.md) - Fluent construction
- [Declarative APIs](declarative-vs-imperative.md) - Fluent composition
- [Language Constraints](language-constraints.md) - Ownership and method chaining
