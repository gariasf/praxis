# Transform Hierarchy

Scene graphs and spatial relationships in Praxis.

## Core Components

### Transform
Local position, rotation, and scale relative to parent (or world if no parent):

```rust
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

### GlobalTransform
Computed world-space transform after hierarchy propagation:

```rust
#[derive(Component)]
pub struct GlobalTransform(pub Mat4);
```

### Parent / Children
Define hierarchy relationships:

```rust
// Parent points to parent entity
#[derive(Component)]
pub struct Parent(pub Entity);

// Children lists all child entities
#[derive(Component)]
pub struct Children(pub Vec<Entity>);
```

## Transform Propagation

The `transform_propagation_system` automatically computes `GlobalTransform`:

```
World
└── Car (Transform: pos(0,0,0))
    ├── Body (Transform: pos(0,1,0))    → GlobalTransform: pos(0,1,0)
    └── Wheel (Transform: pos(1,0,0))   → GlobalTransform: pos(1,0,0)
        └── Hub (Transform: pos(0,0,0.1)) → GlobalTransform: pos(1,0,0.1)
```

When parent transforms change, all descendants update automatically.

### Propagation Flow

The system processes entities in a top-down, breadth-first manner:

```
┌─────────────────────────────────────────────────────────────┐
│ Transform Propagation System                                │
│                                                               │
│  Step 1: Process Root Entities                              │
│  ┌─────────────────────────────────────────┐                │
│  │ Root Entity (no Parent)                 │                │
│  │  - Read: Transform                      │                │
│  │  - Write: GlobalTransform = Transform   │                │
│  └─────────────────────────────────────────┘                │
│                    │                                         │
│                    ▼                                         │
│  Step 2: Process Children (Level 1)                         │
│  ┌─────────────────────────────────────────┐                │
│  │ Child Entity                            │                │
│  │  - Read: Transform, Parent              │                │
│  │  - Get: Parent.GlobalTransform          │                │
│  │  - Write: GlobalTransform =             │                │
│  │           Parent.GlobalTransform *      │                │
│  │           Transform                     │                │
│  └─────────────────────────────────────────┘                │
│                    │                                         │
│                    ▼                                         │
│  Step 3: Process Grandchildren (Level 2)                    │
│  ┌─────────────────────────────────────────┐                │
│  │ Grandchild Entity                       │                │
│  │  - Read: Transform, Parent              │                │
│  │  - Get: Parent.GlobalTransform          │                │
│  │  - Write: GlobalTransform =             │                │
│  │           Parent.GlobalTransform *      │                │
│  │           Transform                     │                │
│  └─────────────────────────────────────────┘                │
│                    │                                         │
│                    ▼                                         │
│  Repeat for all hierarchy levels...                         │
│                                                               │
└─────────────────────────────────────────────────────────────┘

Matrix Multiplication Chain:
═══════════════════════════════

Root:        GlobalTransform = LocalTransform
             [identity] * [local]

Child:       GlobalTransform = Parent.GlobalTransform * LocalTransform
             [parent_world] * [local] = [child_world]

Grandchild:  GlobalTransform = Parent.GlobalTransform * LocalTransform
             [parent_world * parent_local] * [local] = [grandchild_world]

Example with actual transforms:
═══════════════════════════════

Root (Car at origin):
  LocalTransform:  translate(0, 0, 0)
  GlobalTransform: translate(0, 0, 0)

Child (Wheel offset):
  LocalTransform:  translate(1, 0, 0)
  GlobalTransform: translate(0,0,0) * translate(1,0,0) = translate(1, 0, 0)

Grandchild (Hub offset from wheel):
  LocalTransform:  translate(0, 0, 0.1)
  GlobalTransform: translate(1,0,0) * translate(0,0,0.1) = translate(1, 0, 0.1)

If Car moves to (5, 0, 0):
═══════════════════════════════
  Car.LocalTransform = translate(5, 0, 0)
  → System propagates to all descendants:
  
  Car.GlobalTransform:   translate(5, 0, 0)
  Wheel.GlobalTransform: translate(5,0,0) * translate(1,0,0) = translate(6, 0, 0)
  Hub.GlobalTransform:   translate(6,0,0) * translate(0,0,0.1) = translate(6, 0, 0.1)
```

## Creating Hierarchies

```rust
// Spawn parent
let parent = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

// Spawn child with parent relationship
let child = world.spawn((
    Transform::from_xyz(1.0, 0.0, 0.0),
    Parent(parent),
)).id();

// Update parent's children list
world.get_mut::<Children>(parent)
    .map(|mut c| c.0.push(child));
```

Or use commands for automatic hierarchy management:

```rust
commands.spawn(TransformBundle::default())
    .with_children(|parent| {
        parent.spawn(TransformBundle::from_xyz(1.0, 0.0, 0.0));
        parent.spawn(TransformBundle::from_xyz(-1.0, 0.0, 0.0));
    });
```

## Common Use Cases

### Character Skeletons
```
Character
└── Spine
    ├── Head
    ├── LeftArm → LeftHand
    └── RightArm → RightHand
```

### Vehicle Parts
```
Vehicle
├── Chassis
├── Wheel_FL
├── Wheel_FR
├── Wheel_BL
└── Wheel_BR
```

### UI Layouts
```
Window
├── TitleBar
└── Content
    ├── Button
    └── Label
```

## Transform Operations

### Combining Transforms
```rust
let world_matrix = parent_global * child_local;
```

### Decomposing
```rust
let (translation, rotation, scale) = transform.to_scale_rotation_translation();
```

### Look-at
```rust
let rotation = Quat::from_rotation_arc(Vec3::Z, (target - position).normalize());
```

## Performance Tips

1. **Flatten when possible**: Deep hierarchies have propagation overhead
2. **Static hierarchies**: Mark unchanging transforms to skip updates
3. **Batch updates**: Change multiple transforms, then propagate once

## See Also

- [Beginner's Guide: Transform Hierarchy Propagation](../beginners-guide.md#transform-hierarchy-propagation) - Deep dive with matrix math
- [Animation Concepts](animation.md) - Skeletal hierarchies
- [ECS Architecture](ecs-architecture.md) - Component patterns
- [praxis_scene Crate](../../crates/praxis_scene/README.md) - Crate documentation
