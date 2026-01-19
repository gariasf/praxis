# Components Reference

ECS components available in Praxis.

## Transform Components

### Transform
Local position, rotation, and scale.

```rust
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

**Constructors:**
- `Transform::default()` - Identity transform
- `Transform::from_xyz(x, y, z)` - Position only
- `Transform::from_translation(vec)` - Position from Vec3
- `Transform::from_rotation(quat)` - Rotation only
- `Transform::from_scale(vec)` - Scale only

### GlobalTransform
Computed world-space transform (read-only, set by systems).

```rust
#[derive(Component)]
pub struct GlobalTransform(pub Mat4);
```

### Parent / Children
Hierarchy relationships.

```rust
#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);
```

## Rendering Components

### Mesh / MeshHandle
Reference to a loaded mesh.

```rust
#[derive(Component)]
pub struct MeshHandle { pub id: String }
```

**See**: [Mesh API](mesh-api.md) for details.

### Material
PBR material properties.

```rust
#[derive(Component)]
pub struct Material {
    pub albedo: Vec4,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
}
```

## Camera Components

### Camera
Marks entity as a camera with active/priority control.

```rust
#[derive(Component)]
pub struct Camera {
    pub is_active: bool,
    pub priority: i32,
}
```

**See**: [Camera API](camera-api.md) for projection types and usage.

### EditorCamera
Marker for the editor camera entity.

```rust
#[derive(Component)]
pub struct EditorCamera;
```

## Physics Components

### RigidBody
Physics body type.

```rust
#[derive(Component)]
pub enum RigidBody {
    Dynamic,    // Affected by forces
    Static,     // Never moves
    Kinematic,  // Moved by code
}
```

### Collider
Collision shape.

```rust
#[derive(Component)]
pub enum Collider {
    Cuboid { hx: f32, hy: f32, hz: f32 },
    Sphere { radius: f32 },
    Capsule { half_height: f32, radius: f32 },
    // ...
}
```

### PhysicsVelocity
Linear and angular velocity.

```rust
#[derive(Component)]
pub struct PhysicsVelocity {
    pub linear: Vec3,
    pub angular: Vec3,
}
```

## Audio Components

### AudioSource
Spatial audio emitter.

```rust
#[derive(Component)]
pub struct AudioSource { /* ... */ }
```

**See**: [Audio API](audio-api.md) for properties and usage.

### AudioListener
Marks the audio listener entity.

```rust
#[derive(Component)]
pub struct AudioListener;
```

## Animation Components

### Skeleton
Bone hierarchy for skeletal animation.

```rust
#[derive(Component)]
pub struct Skeleton { /* ... */ }
```

**See**: [Animation API](animation-api.md) for methods and usage.

### AnimationPlayer
Controls animation playback.

```rust
#[derive(Component)]
pub struct AnimationPlayer { /* ... */ }
```

**See**: [Animation API](animation-api.md) for methods and usage.

### AnimatedPose
Current computed bone transforms.

```rust
#[derive(Component)]
pub struct AnimatedPose { /* ... */ }
```

**See**: [Animation API](animation-api.md) for methods and usage.

## Editor Components

### Selectable
Marks entity as selectable in editor.

```rust
#[derive(Component)]
pub struct Selectable;
```

### Selected
Automatically added to selected entities.

```rust
#[derive(Component)]
pub struct Selected;
```

### TransformGizmo
Enables gizmo rendering for entity.

```rust
#[derive(Component)]
pub struct TransformGizmo {
    pub visible: bool,
    pub size_multiplier: f32,
}
```

## Marker Components

Marker components have no data—they just tag entities:

```rust
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Projectile;
```

Use with `With<T>` and `Without<T>` in queries:

```rust
fn player_system(query: Query<&Transform, With<Player>>) { ... }
```

## See Also

- [ECS Architecture](../concepts/ecs-architecture.md)
- [praxis_ecs Crate](../../crates/praxis_ecs/README.md)
