# Builder Patterns

Builder patterns provide ergonomic APIs for constructing complex objects with many optional parameters. They're ubiquitous in game engines for configuring entities, components, rendering pipelines, and other complex subsystems.

## The Core Problem

Game engines frequently need to create objects with:

1. **Many optional parameters** - Dozens of configuration options
2. **Sensible defaults** - Most parameters have reasonable defaults
3. **Validation** - Some combinations of parameters are invalid
4. **Type safety** - Catch configuration errors at compile time
5. **Discoverability** - Users should easily find available options

Traditional constructors with positional parameters don't scale:

```cpp
// Nightmare: What do these parameters mean?
AudioSource src("sound.ogg", 0.7, true, 50.0, 5.0, 1.0, false, 0.5);
```

Builders solve this with named parameters and method chaining.

## Pattern Variants

### 1. Basic Builder Pattern

**Concept**: Separate builder object accumulates configuration, then builds the final object.

=== "Rust (Praxis)"

    ```rust
    // Praxis AudioSource builder from examples/audio_simple.rs
    let audio_source = AudioSource::new("assets/sounds/test.ogg")
        .with_volume(0.7)
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(50.0)
        .with_reference_distance(5.0);

    // Spawn entity with configured component
    world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        audio_source,
    ));
    ```

    **Implementation pattern**:
    ```rust
    pub struct AudioSource {
        path: String,
        volume: f32,
        spatial: bool,
        looping: bool,
        max_distance: f32,
        reference_distance: f32,
    }

    impl AudioSource {
        pub fn new(path: impl Into<String>) -> Self {
            Self {
                path: path.into(),
                volume: 1.0,           // Sensible defaults
                spatial: false,
                looping: false,
                max_distance: 100.0,
                reference_distance: 1.0,
            }
        }

        pub fn with_volume(mut self, volume: f32) -> Self {
            self.volume = volume;
            self  // Return self for chaining
        }

        pub fn with_spatial(mut self, spatial: bool) -> Self {
            self.spatial = spatial;
            self
        }

        // ... more builder methods
    }
    ```

=== "C# (Unity)"

    ```csharp
    // Unity doesn't use builders for components (mutable after creation)
    // But uses them for complex operations like asset loading
    var request = Resources.LoadAsync<AudioClip>("sounds/test");
    request.completed += (op) => {
        var clip = request.asset as AudioClip;
        audioSource.clip = clip;
        audioSource.volume = 0.7f;
        audioSource.spatialBlend = 1.0f; // 3D sound
        audioSource.loop = true;
        audioSource.maxDistance = 50.0f;
        audioSource.minDistance = 5.0f;
        audioSource.Play();
    };
    ```

    **Unity pattern**: Components are mutable, so direct property assignment is used.
    Builders appear in asset loading, query building, and editor utilities.

=== "Rust (Bevy)"

    ```rust
    // Bevy heavily uses builders via bundles
    commands.spawn(AudioBundle {
        source: asset_server.load("sounds/test.ogg"),
        settings: PlaybackSettings {
            volume: Volume::new(0.7),
            mode: PlaybackMode::Loop,
            spatial: true,
            ..default()
        },
        spatial: SpatialSettings {
            max_distance: 50.0,
            reference_distance: 5.0,
            ..default()
        },
        ..default()
    });
    ```

    **Bevy pattern**: Uses Rust struct update syntax (`..default()`) for builders.
    Bundles are the primary builder pattern for entity creation.

=== "C++ (Unreal)"

    ```cpp
    // Unreal uses separate builder classes for complex objects
    USoundCue* Sound = LoadObject<USoundCue>(
        nullptr, 
        TEXT("/Game/Audio/TestSound.TestSound")
    );
    
    UAudioComponent* AudioComponent = 
        UGameplayStatics::SpawnSoundAtLocation(
            WorldContext,
            Sound,
            FVector(1000.f, 0.f, 0.f),
            FRotator::ZeroRotator,
            0.7f,  // VolumeMultiplier
            1.0f,  // PitchMultiplier
            0.0f,  // StartTime
            nullptr,  // AttenuationSettings
            nullptr,  // ConcurrencySettings
            true   // bAutoDestroy
        );
    ```

    **Unreal pattern**: Factory functions with many parameters. Uses default arguments
    and overloads for optionality. Builder classes for complex subsystems.

**Trade-offs**:

✅ **Strengths**:
- Clear, self-documenting API
- Flexible—add new options without breaking existing code
- Compile-time type safety
- Enforces defaults for unspecified options
- Chainable for readability

❌ **Weaknesses**:
- More verbose than positional parameters
- Requires separate builder methods for each option
- Validation happens at build time (or runtime), not during configuration
- May allocate intermediate objects

### 2. Type-State Builder Pattern

**Concept**: Use types to enforce required parameters and valid state transitions at compile time.

=== "Rust"

    ```rust
    // Type-state builder ensures you can't build without required params
    use std::marker::PhantomData;

    struct NeedsPath;
    struct HasPath;
    struct NeedsVolume;
    struct HasVolume;

    struct AudioSourceBuilder<P, V> {
        path: Option<String>,
        volume: Option<f32>,
        spatial: bool,
        _path_state: PhantomData<P>,
        _volume_state: PhantomData<V>,
    }

    impl AudioSourceBuilder<NeedsPath, NeedsVolume> {
        fn new() -> Self {
            Self {
                path: None,
                volume: None,
                spatial: false,
                _path_state: PhantomData,
                _volume_state: PhantomData,
            }
        }
    }

    impl<V> AudioSourceBuilder<NeedsPath, V> {
        fn with_path(self, path: String) 
            -> AudioSourceBuilder<HasPath, V> 
        {
            AudioSourceBuilder {
                path: Some(path),
                volume: self.volume,
                spatial: self.spatial,
                _path_state: PhantomData,
                _volume_state: PhantomData,
            }
        }
    }

    impl<P> AudioSourceBuilder<P, NeedsVolume> {
        fn with_volume(self, volume: f32) 
            -> AudioSourceBuilder<P, HasVolume> 
        {
            AudioSourceBuilder {
                path: self.path,
                volume: Some(volume),
                spatial: self.spatial,
                _path_state: PhantomData,
                _volume_state: PhantomData,
            }
        }
    }

    // Can only build when both required fields are set
    impl AudioSourceBuilder<HasPath, HasVolume> {
        fn build(self) -> AudioSource {
            AudioSource {
                path: self.path.unwrap(),
                volume: self.volume.unwrap(),
                spatial: self.spatial,
            }
        }
    }

    // Usage:
    let source = AudioSourceBuilder::new()
        .with_path("sound.ogg".to_string())
        .with_volume(0.7)
        .build();  // Compile error if path or volume not set!
    ```

    **Zero-cost abstraction**: All the type-state machinery is eliminated at compile time.
    The generated code is identical to a direct struct initialization.

=== "C++ (Template Metaprogramming)"

    ```cpp
    // Similar approach using template parameters
    template<bool HasPath = false, bool HasVolume = false>
    class AudioSourceBuilder {
        std::optional<std::string> path_;
        std::optional<float> volume_;
        bool spatial_ = false;

    public:
        AudioSourceBuilder() = default;

        auto withPath(std::string path) 
            -> AudioSourceBuilder<true, HasVolume> 
        {
            AudioSourceBuilder<true, HasVolume> builder;
            builder.path_ = std::move(path);
            builder.volume_ = volume_;
            builder.spatial_ = spatial_;
            return builder;
        }

        auto withVolume(float volume) 
            -> AudioSourceBuilder<HasPath, true> 
        {
            AudioSourceBuilder<HasPath, true> builder;
            builder.path_ = path_;
            builder.volume_ = volume;
            builder.spatial_ = spatial_;
            return builder;
        }

        // Only available when both are set
        template<bool P = HasPath, bool V = HasVolume>
        std::enable_if_t<P && V, AudioSource> build() {
            return AudioSource(
                std::move(*path_),
                *volume_,
                spatial_
            );
        }
    };

    // Usage:
    auto source = AudioSourceBuilder()
        .withPath("sound.ogg")
        .withVolume(0.7f)
        .build();  // Compile error if either missing
    ```

**Trade-offs**:

✅ **Strengths**:
- Compile-time enforcement of required parameters
- Invalid states are unrepresentable
- Zero runtime overhead
- Excellent for complex initialization sequences
- Documents state machine in types

❌ **Weaknesses**:
- Complex implementation
- Difficult to understand for beginners
- Long compile times with many type parameters
- Poor error messages when used incorrectly
- Not practical in languages without zero-cost generics

**When to use**:
- Safety-critical initialization (rendering pipelines, network protocols)
- Complex state machines
- Rust libraries where compile-time guarantees are valued
- Not practical for simple objects or scripting-exposed APIs

### 3. Derive-Based Builders

**Concept**: Use macros/code generation to automatically create builder APIs.

=== "Rust (derive_builder)"

    ```rust
    use derive_builder::Builder;

    #[derive(Builder, Debug)]
    #[builder(setter(into))]
    struct AudioSource {
        path: String,
        
        #[builder(default = "1.0")]
        volume: f32,
        
        #[builder(default)]
        spatial: bool,
        
        #[builder(default)]
        looping: bool,
        
        #[builder(default = "100.0")]
        max_distance: f32,
    }

    // Generated builder API:
    let source = AudioSourceBuilder::default()
        .path("sound.ogg")
        .volume(0.7)
        .spatial(true)
        .looping(true)
        .max_distance(50.0)
        .build()?;  // Returns Result<AudioSource, Error>
    ```

    **Generated code**:
    - `AudioSourceBuilder` struct with `Option<T>` for each field
    - `default()` constructor
    - Setter methods for each field
    - `build()` method that validates and constructs

=== "C# (Source Generators)"

    ```csharp
    // C# 9+ source generators can create builders
    [GenerateBuilder]
    public partial class AudioSource 
    {
        public required string Path { get; init; }
        public float Volume { get; init; } = 1.0f;
        public bool Spatial { get; init; } = false;
        public bool Looping { get; init; } = false;
        public float MaxDistance { get; init; } = 100.0f;
    }

    // Usage:
    var source = new AudioSourceBuilder()
        .WithPath("sound.ogg")
        .WithVolume(0.7f)
        .WithSpatial(true)
        .Build();
    ```

**Trade-offs**:

✅ **Strengths**:
- Minimal boilerplate
- Consistent API across types
- Easy to add/remove fields
- Customizable via attributes
- Type-safe

❌ **Weaknesses**:
- Requires external dependencies or code generation
- Less flexible than hand-written builders
- Macro/generator complexity
- Can obscure what code actually runs

## Real-World Examples

### Praxis: Scene Definition Builder

```rust
// From crates/praxis_scene/src/scene_definition.rs
let scene = SceneDefinition {
    name: "Example Scene".to_string(),
    metadata: SceneMetadata {
        version: "1.0.0".to_string(),
        description: Some("A simple example scene".to_string()),
        author: Some("Praxis Engine".to_string()),
        ..Default::default()
    },
    entities: vec![
        EntityDefinition {
            name: Some("Player".to_string()),
            transform: Some(TransformDef {
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
            }),
            components: vec![],
            children: vec![],
        },
    ],
};
```

Praxis uses Rust's struct update syntax (`..Default::default()`) as a lightweight builder pattern.

### Unity: Animation Curve Editor

```csharp
// Unity's AnimationCurve builder
var curve = new AnimationCurve(
    new Keyframe(0f, 0f),
    new Keyframe(1f, 1f)
);

// Fluent API for modification
curve.AddKey(2f, 0.5f);
curve.SmoothTangents(0, 0.5f);

// Query builder for asset database
var guids = AssetDatabase.FindAssets("t:AudioClip")
    .Select(guid => AssetDatabase.GUIDToAssetPath(guid))
    .Where(path => path.Contains("Music"))
    .ToArray();
```

Unity uses builders for complex queries and gradual configuration of mutable objects.

### Bevy: Schedule Builder

```rust
// Bevy's schedule builder uses a fluent API
app.add_systems(
    Update,
    (
        movement_system,
        collision_system,
        audio_system,
    )
        .chain()  // Run in sequence
        .run_if(in_state(GameState::Playing))
        .before(render_system)
);

// Camera bundle builder
commands.spawn(Camera3dBundle {
    camera: Camera {
        order: 0,
        ..default()
    },
    transform: Transform::from_xyz(0.0, 5.0, 10.0)
        .looking_at(Vec3::ZERO, Vec3::Y),
    ..default()
});
```

Bevy extensively uses struct update syntax and method chaining for declarative APIs.

### Unreal: Blueprint Node Builder

```cpp
// Unreal's K2Node builders for Blueprint graphs
UK2Node_CallFunction* CallNode = 
    NewObject<UK2Node_CallFunction>(Graph);

CallNode->FunctionReference.SetExternalMember(
    GET_FUNCTION_NAME_CHECKED(UGameplayStatics, SpawnEmitterAtLocation),
    UGameplayStatics::StaticClass()
);

CallNode->AllocateDefaultPins();

// Fluent-style chaining via return values
UEdGraphPin* SpawnedPin = CallNode->GetReturnValuePin();
UEdGraphPin* WorldContextPin = CallNode->FindPinChecked(
    TEXT("WorldContextObject")
);
```

Unreal uses builders for editor functionality and complex object graphs.

## Design Guidelines

### When to Use Builders

✅ **Use builders when**:
- Object has >3 configuration parameters
- Most parameters have sensible defaults
- Users need clear, discoverable API
- Configuration is complex or stateful
- Validation logic is non-trivial

❌ **Don't use builders when**:
- Simple objects with 1-2 required fields
- All parameters are equally important (no defaults)
- Performance is absolutely critical (hot path)
- Scripting languages need simple APIs

### API Design Principles

**1. Start simple, evolve to builders**

```rust
// Start with this
impl Transform {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self { ... }
}

// Add builder methods as needs grow
impl Transform {
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }
    
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }
}

// Usage:
let transform = Transform::from_xyz(1.0, 2.0, 3.0)
    .with_rotation(Quat::from_rotation_y(PI / 4.0))
    .with_scale(Vec3::splat(2.0));
```

**2. Provide both builders and direct construction**

```rust
// Direct construction for common case
let source = AudioSource::new("sound.ogg");

// Builder for complex case
let source = AudioSource::new("sound.ogg")
    .with_volume(0.7)
    .with_spatial(true);
```

**3. Use type-state only when safety justifies complexity**

Type-state builders are powerful but complex. Reserve for:
- Initialization sequences that can fail silently
- State machines with invalid transitions
- Safety-critical configuration

**4. Document builder methods clearly**

```rust
impl AudioSource {
    /// Sets the playback volume.
    /// 
    /// # Arguments
    /// * `volume` - Volume multiplier from 0.0 (silent) to 1.0 (full volume).
    ///              Values >1.0 will amplify the sound.
    /// 
    /// # Example
    /// ```
    /// let source = AudioSource::new("sound.ogg").with_volume(0.5);
    /// ```
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }
}
```

## Performance Considerations

### Zero-Cost Abstractions

Well-designed builders compile to identical code as direct construction:

```rust
// Builder syntax
let source = AudioSource::new("sound.ogg")
    .with_volume(0.7)
    .with_spatial(true);

// Compiles to same machine code as:
let source = AudioSource {
    path: "sound.ogg".to_string(),
    volume: 0.7,
    spatial: true,
    looping: false,
    max_distance: 100.0,
    reference_distance: 1.0,
};
```

The compiler inlines all builder methods and eliminates intermediate copies.

### When Builders Have Overhead

Builders may introduce overhead when:

1. **Heap allocations**: Building collections incrementally
2. **Validation**: Runtime checks during construction
3. **Separate builder type**: Extra copy to final type

**Example with overhead**:

```rust
// Builder allocates Vec, then copies to final object
struct MeshBuilder {
    vertices: Vec<Vertex>,  // Grows during building
}

impl MeshBuilder {
    fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);  // Allocation
    }
    
    fn build(self) -> Mesh {
        Mesh {
            vertices: self.vertices,  // Move, no copy
        }
    }
}
```

**Optimization**: Use `with_capacity` to pre-allocate:

```rust
fn new() -> Self {
    Self {
        vertices: Vec::with_capacity(1024),  // Avoid reallocation
    }
}
```

## Language-Specific Patterns

### Rust: Ownership-Based Builders

```rust
// Consume self to prevent use-after-build
impl Builder {
    pub fn build(self) -> Object {  // Takes ownership
        Object { ... }
    }
}

let builder = Builder::new();
let object = builder.build();  // builder is moved
// builder.build();  // Compile error: value used after move
```

### C#: Object Initializers

```csharp
// C# object initializers are lightweight builders
var source = new AudioSource {
    Path = "sound.ogg",
    Volume = 0.7f,
    Spatial = true,
    Looping = true
};

// With required properties (C# 11)
public required string Path { get; init; }
```

### C++: Named Parameters Idiom

```cpp
// C++ doesn't have named parameters, use builder
class AudioSource {
    std::string path_;
    float volume_ = 1.0f;
    
public:
    AudioSource& path(std::string p) { 
        path_ = std::move(p); 
        return *this; 
    }
    
    AudioSource& volume(float v) { 
        volume_ = v; 
        return *this; 
    }
};

// Usage:
AudioSource source;
source.path("sound.ogg").volume(0.7f);
```

## Summary

| Pattern | Language | Use Case | Trade-off |
|---------|----------|----------|-----------|
| Basic Builder | All | General configuration | Simple but manual |
| Type-State | Rust, C++ | Safety-critical | Complex but safe |
| Derive/Generated | Rust, C# | Reduce boilerplate | Dependency cost |
| Struct Update | Rust | Lightweight defaults | Limited flexibility |
| Object Initializer | C# | Simple objects | No validation |

Choose builders based on:
- **Complexity**: How many options?
- **Safety**: Can invalid states cause crashes?
- **Audience**: Experienced or novice users?
- **Language**: What features are available?

When in doubt, start with simple builders and evolve as needs emerge.

## Related Patterns

- [Fluent Interfaces](fluent-interfaces.md) - Method chaining beyond construction
- [Declarative APIs](declarative-vs-imperative.md) - Builder-style scene composition
- [Language Constraints](language-constraints.md) - How traits/templates enable builders
