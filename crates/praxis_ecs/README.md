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

### Mesh and MeshHandle

Components for rendering 3D geometry.

**MeshHandle**: References a mesh asset by ID (preferred for shared meshes).

```rust
use praxis_ecs::{MeshHandle, Transform};

world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("cube"),
));
```

**Mesh**: Stores mesh data directly on the entity (for procedural/dynamic meshes).

```rust
use praxis_ecs::Mesh;

let vertices = vec![
    [0.0, 1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
];
let indices = vec![0, 1, 2];

world.spawn((
    Transform::default(),
    Mesh::new(vertices, indices),
));
```

The `Mesh` component supports optional attributes:

```rust
let mut mesh = Mesh::new(vertices, indices);
mesh.set_colors(colors);  // Optional vertex colors
mesh.set_normals(normals); // Optional vertex normals
mesh.set_uvs(uvs);        // Optional texture coordinates
```

See the [Mesh System Documentation](../../docs/mesh_system.md) for complete details on using meshes.

## Transform Propagation Implementation Details

### System Design

The transform propagation system consists of five interconnected systems that maintain world-space transforms automatically:

1. **`sync_parent_child_relationships`**
   - Triggers on `Added<Parent>` and `Changed<Parent>`
   - Automatically adds entities to parent's `Children` component
   - Creates `Children` component if it doesn't exist
   - Maintains bidirectional relationships

2. **`cleanup_removed_parents`**
   - Runs every frame to ensure consistency
   - Removes orphaned children from `Children` components when `Parent` is removed
   - Cleans up empty `Children` components

3. **`propagate_transforms`**
   - Updates root entities (without parents) when `Transform` changes
   - Uses change detection: `Changed<Transform>` and `Added<Transform>`
   - Recursively propagates to all descendants
   - Efficient iterative implementation to avoid stack overflow

4. **`propagate_transforms_for_reparented`**
   - Handles entities whose `Parent` was added or changed
   - Triggers on `Added<Parent>` and `Changed<Parent>`
   - Immediately updates entity and all descendants based on new parent
   - Critical for proper reparenting behavior

5. **`propagate_transforms_for_changed_children`**
   - Updates entities with parents when local `Transform` changes
   - Triggers on `Changed<Transform>` for entities with `Parent`
   - Propagates changes to all descendants
   - Ensures child transform changes ripple through hierarchy

### Change Detection

The system uses Bevy ECS's change detection to minimize unnecessary computation:
- Only processes entities whose transforms or relationships changed
- Uses `Added<T>` and `Changed<T>` queries
- Propagation only occurs through affected subtrees

### Recursive Propagation Algorithm

Uses an iterative work queue to avoid stack overflow with deep hierarchies:

```rust
fn propagate_recursive(
    entity: Entity,
    parent_global: &GlobalTransform,
    world: &mut World,
) {
    let mut work_queue = vec![(entity, parent_global.clone())];
    
    while let Some((current_entity, current_parent_global)) = work_queue.pop() {
        // Update current entity's GlobalTransform
        // Add children to work queue with updated parent transform
    }
}
```

### Performance Characteristics

- **O(1)** when nothing changes (thanks to change detection)
- **O(n)** where n is changed entities and their descendants
- Sibling branches are independent and don't affect each other
- Deep hierarchies handled efficiently with iterative approach

## Testing

The system includes comprehensive tests covering:
- Basic parent-child propagation
- Deep hierarchies (3+ levels)
- Multiple children per parent
- Entity reparenting
- Complex transforms (rotation, scale)
- Parent removal and cleanup
- Full system chain integration

Run tests with:
```bash
cargo test -p praxis_ecs
```

## Examples

See the transform propagation demo:

```bash
cargo run --example transform_propagation_demo
```

## Dependencies

- `bevy_ecs` 0.14: Core Entity-Component-System
- `praxis_math`: Math types (Vec3, Quat, Mat4)
- `praxis_utils`: Error handling

## See Also

- [Transform Propagation Demo](../../examples/transform_propagation_demo.rs)
- [ECS Integration Example](../../examples/ecs_integration.rs)
- [Mesh System Documentation](../../docs/mesh_system.md)
- [bevy_ecs Documentation](https://docs.rs/bevy_ecs)
