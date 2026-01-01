# Praxis ECS

Entity Component System for the Praxis game engine, built on top of `bevy_ecs`.

## Features

- **Transform Hierarchy**: Automatic transform propagation through parent-child relationships
- **Built-in Components**: Common components like Transform, GlobalTransform, Parent, Children, Name, Visibility
- **Change Detection**: Efficient systems that only update when components change
- **Parent-Child Relationships**: Bidirectional parent-child tracking with automatic maintenance

## Transform Propagation System

The transform propagation system is one of the core features of Praxis ECS. It automatically computes world-space transforms (GlobalTransform) from local-space transforms (Transform) and the parent-child hierarchy.

### Key Concepts

- **Transform**: Local-space position, rotation, and scale of an entity
- **GlobalTransform**: World-space transformation matrix computed from Transform and parent hierarchy
- **Parent**: Component that references the parent entity
- **Children**: Component that contains a list of child entities

### Systems

The transform propagation system consists of five main systems that work together:

1. **`sync_parent_child_relationships`**: Maintains bidirectional parent-child relationships by automatically updating the Children component when Parent components are added or changed.

2. **`cleanup_removed_parents`**: Cleans up orphaned references in Children components when Parent components are removed.

3. **`propagate_transforms`**: Updates GlobalTransform for root entities (entities without parents) when their Transform changes, and recursively propagates to all descendants.

4. **`propagate_transforms_for_reparented`**: Immediately updates GlobalTransform for entities whose Parent component was added or changed.

5. **`propagate_transforms_for_changed_children`**: Updates GlobalTransform for entities with parents when their local Transform changes.

### Usage Example

```rust
use praxis_ecs::{World, Schedule, IntoSystemConfigs};
use praxis_ecs::{Transform, GlobalTransform, Parent, Children};
use praxis_ecs::systems::*;
use praxis_math::Vec3;

// Create world and schedule
let mut world = World::new();
let mut schedule = Schedule::default();

// Add all transform propagation systems in the correct order
schedule.add_systems((
    sync_parent_child_relationships,
    cleanup_removed_parents,
    propagate_transforms,
    propagate_transforms_for_reparented,
    propagate_transforms_for_changed_children,
).chain());

// Create a parent entity at (10, 0, 0)
let parent = world.spawn((
    Transform::from_xyz(10.0, 0.0, 0.0),
    GlobalTransform::default(),
));

// Create a child entity at local position (5, 0, 0)
let child = world.spawn((
    Transform::from_xyz(5.0, 0.0, 0.0),
    GlobalTransform::default(),
    Parent(parent),
));

// Run the schedule to propagate transforms
world.inner_mut().run_schedule(&mut schedule);

// The child's GlobalTransform will now be at world position (15, 0, 0)
let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
assert_eq!(child_global.translation(), Vec3::new(15.0, 0.0, 0.0));
```

### How It Works

1. **Hierarchy Setup**: When you add a `Parent` component to an entity, the `sync_parent_child_relationships` system automatically adds the entity to its parent's `Children` component.

2. **Transform Changes**: When a `Transform` is modified:
   - If the entity has no parent, `propagate_transforms` updates its `GlobalTransform` directly
   - If the entity has a parent, `propagate_transforms_for_changed_children` computes the new `GlobalTransform` by multiplying the parent's `GlobalTransform` with the local `Transform`
   - The change is then recursively propagated to all descendants

3. **Reparenting**: When a `Parent` component is added or changed, `propagate_transforms_for_reparented` immediately updates the entity and its descendants based on the new parent.

4. **Cleanup**: When a `Parent` component is removed, `cleanup_removed_parents` removes the entity from its old parent's `Children` list.

### Performance Considerations

- The system uses change detection (`Changed<Transform>`, `Added<Parent>`, etc.) to minimize unnecessary computations
- Only entities whose transforms actually changed, or whose parents changed, are updated
- Propagation is done recursively within each affected subtree only
- Deep hierarchies are handled efficiently using an iterative approach with a work queue

### Best Practices

1. **Always use TransformBundle**: When spawning entities that need transforms, use `TransformBundle` to ensure both `Transform` and `GlobalTransform` are present.

2. **Parent first, then children**: It's more efficient to spawn parents before children, as this allows the system to propagate transforms correctly from the start.

3. **Batch reparenting**: If you need to reparent multiple entities, try to do it in the same frame to minimize propagation passes.

4. **Avoid deep hierarchies**: While the system handles deep hierarchies efficiently, extremely deep nesting (>10-15 levels) can impact performance.

## Built-in Components

### Transform

Local-space transformation with position, rotation, and scale.

```rust
let transform = Transform {
    translation: Vec3::new(10.0, 0.0, 0.0),
    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
    scale: Vec3::ONE,
};
```

### GlobalTransform

World-space transformation matrix, automatically computed by the propagation system.

```rust
let global = GlobalTransform::from_matrix(Mat4::IDENTITY);
let world_pos = global.translation();
```

### Parent

References the parent entity in the hierarchy.

```rust
let parent_entity = world.spawn(TransformBundle::default());
let child_entity = world.spawn((
    TransformBundle::default(),
    Parent(parent_entity),
));
```

### Children

Contains a list of child entities. Usually managed automatically by the system.

```rust
// Automatically maintained by sync_parent_child_relationships
let children = Children::with_children(vec![child1, child2, child3]);
```

### Name

A debug-friendly name for entities.

```rust
world.spawn((
    Name::from("Player"),
    TransformBundle::default(),
));
```

### Visibility

Controls whether an entity should be rendered.

```rust
world.spawn((
    TransformBundle::default(),
    Visibility::Visible, // or Visibility::Hidden
));
```

## Examples

See `examples/transform_propagation_demo.rs` for a comprehensive demonstration of the transform propagation system.

```bash
cargo run --example transform_propagation_demo
```
