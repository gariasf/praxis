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

### Mesh
Reference to a loaded mesh.

```rust
#[derive(Component)]
pub struct Mesh(pub String);  // Mesh ID/name
```

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
Projection settings.

```rust
#[derive(Component)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}
```

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
pub struct AudioSource {
    pub path: String,
    pub volume: f32,
    pub spatial: bool,
    pub looping: bool,
    pub max_distance: f32,
    pub reference_distance: f32,
    pub doppler_enabled: bool,
    pub doppler_scale: f32,
    pub state: PlaybackState,
}
```

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
pub struct Skeleton {
    bones: Vec<Bone>,
    inverse_bind_matrices: Vec<Mat4>,
}
```

### AnimationPlayer
Controls animation playback.

```rust
#[derive(Component)]
pub struct AnimationPlayer {
    clips: HashMap<String, AnimationClip>,
    active: HashSet<String>,
    // ...
}
```

### AnimatedPose
Current computed bone transforms.

```rust
#[derive(Component)]
pub struct AnimatedPose {
    local_transforms: Vec<Transform>,
    world_transforms: Vec<Mat4>,
    skinning_matrices: Vec<Mat4>,
}
```

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
- [praxis_ecs crate](../../crates/praxis_ecs/README.md)
