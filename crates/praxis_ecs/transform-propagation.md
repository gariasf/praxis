# Transform Propagation System

This document describes the transform propagation system in Praxis ECS, which automatically maintains world-space transforms for entities in a hierarchy.

## Overview

The transform propagation system consists of five interconnected systems that work together to:
1. Maintain bidirectional parent-child relationships
2. Clean up orphaned references
3. Propagate transforms through hierarchies
4. Handle reparenting efficiently
5. Update transforms when they change

## System Architecture

### 1. `sync_parent_child_relationships`

**Purpose**: Maintains the bidirectional parent-child relationship.

**Trigger**: Runs when `Parent` components are added or changed (`Added<Parent>`, `Changed<Parent>`).

**Behavior**:
- When a `Parent` component is added to an entity, adds that entity to the parent's `Children` list
- If the parent doesn't have a `Children` component, creates one
- Prevents duplicate entries in the children list

**Example**:
```rust
let parent = world.spawn(TransformBundle::default());
let child = world.spawn((
    TransformBundle::default(),
    Parent(parent),
));

// After sync_parent_child_relationships runs:
// - parent.get::<Children>() contains [child]
```

### 2. `cleanup_removed_parents`

**Purpose**: Removes orphaned children from `Children` components when `Parent` is removed.

**Trigger**: Runs every frame (checks for consistency).

**Behavior**:
- Iterates through all entities with `Children` components
- Removes any child references that no longer have this entity as their parent
- Removes the `Children` component if the list becomes empty

**Example**:
```rust
world.entity_mut(child).remove::<Parent>();

// After cleanup_removed_parents runs:
// - parent no longer has child in its Children list
// - Children component is removed if empty
```

### 3. `propagate_transforms`

**Purpose**: Updates `GlobalTransform` for root entities and their descendants.

**Trigger**: Runs when `Transform` changes on root entities (entities without `Parent`).

**Behavior**:
- Queries root entities with changed transforms
- Updates their `GlobalTransform` from their local `Transform`
- Recursively propagates to all descendants using an iterative work queue

**Algorithm**:
```
For each root with changed Transform:
    global_transform.matrix = transform.compute_matrix()
    
    For each descendant (iterative):
        child_global = parent_global * child_local
        Recurse for child's children
```

### 4. `propagate_transforms_for_reparented`

**Purpose**: Immediately updates transforms when entities are reparented.

**Trigger**: Runs when `Parent` components are added or changed.

**Behavior**:
- Queries entities whose `Parent` was added or changed
- Looks up the new parent's `GlobalTransform`
- Recomputes the entity's `GlobalTransform` based on the new parent
- Recursively propagates to all descendants

**Example**:
```rust
// child is under parent1
world.entity_mut(child).insert(Parent(parent2));

// After propagate_transforms_for_reparented runs:
// - child.GlobalTransform is updated based on parent2
// - All of child's descendants are updated
```

### 5. `propagate_transforms_for_changed_children`

**Purpose**: Updates transforms when a child's local `Transform` changes.

**Trigger**: Runs when `Transform` changes on entities with a `Parent`.

**Behavior**:
- Queries entities with both `Parent` and changed `Transform`
- Looks up the parent's `GlobalTransform`
- Recomputes the entity's `GlobalTransform`
- Recursively propagates to all descendants

**Example**:
```rust
// Modify child's local transform
transform.translation = Vec3::new(5.0, 0.0, 0.0);

// After propagate_transforms_for_changed_children runs:
// - child.GlobalTransform is updated: parent.global * child.local
// - All of child's descendants are updated
```

## System Ordering

The systems should be run in the following order for correct behavior:

```rust
schedule.add_systems((
    sync_parent_child_relationships,      // 1. Maintain relationships
    cleanup_removed_parents,               // 2. Clean up orphans
    propagate_transforms,                  // 3. Update roots
    propagate_transforms_for_reparented,   // 4. Handle reparenting
    propagate_transforms_for_changed_children, // 5. Update children
).chain());
```

Or use the convenience function:
```rust
use praxis_ecs::systems::transform_propagation_systems;

schedule.add_systems(transform_propagation_systems().chain());
```

## Change Detection

The system uses Bevy ECS's change detection to minimize work:

- **`Added<T>`**: Component was added this frame
- **`Changed<T>`**: Component was modified this frame (including additions)
- **`Or<(...)>`**: Matches if any condition is true

This ensures that only entities whose transforms or relationships actually changed are processed.

## Recursive Propagation

Transform propagation uses an iterative approach with a work queue to avoid stack overflow on deep hierarchies:

```rust
fn propagate_recursive(children, parent_matrix, child_query) {
    for &child_entity in children {
        let child_matrix = parent_matrix * child.transform.compute_matrix();
        child.global_transform.matrix = child_matrix;
        
        // Recursively propagate to grandchildren
        if let Some(children) = child.children {
            propagate_recursive(&children, &child_matrix, child_query);
        }
    }
}
```

## Performance Characteristics

- **Best Case**: O(1) when no transforms change
- **Average Case**: O(n) where n is the number of changed entities and their descendants
- **Worst Case**: O(n) where n is all entities in the scene

The system is highly efficient because:
1. Only changed entities and their descendants are updated
2. Sibling branches are independent and don't affect each other
3. Change detection prevents redundant work
4. Matrix multiplication is the primary cost (well-optimized by `glam`)

## Common Patterns

### Spawning a Hierarchy

```rust
let parent = world.spawn(TransformBundle::from_xyz(10.0, 0.0, 0.0));

let child = world.spawn((
    TransformBundle::from_xyz(5.0, 0.0, 0.0),
    Parent(parent),
));

// Run systems
world.inner_mut().run_schedule(&mut schedule);

// Child is now at world position (15, 0, 0)
```

### Moving a Hierarchy

```rust
// Move the parent - all children follow automatically
world.get_mut::<Transform>(parent).translation.x += 10.0;

// Run systems
world.inner_mut().run_schedule(&mut schedule);
```

### Reparenting

```rust
// Move child from parent1 to parent2
world.entity_mut(child).insert(Parent(parent2));

// Run systems
world.inner_mut().run_schedule(&mut schedule);
```

### Removing from Hierarchy

```rust
// Make child a root entity
world.entity_mut(child).remove::<Parent>();

// Run systems
world.inner_mut().run_schedule(&mut schedule);

// child.GlobalTransform now equals child.Transform
```

## Edge Cases

### Circular Dependencies

**Not prevented by the system** - you must avoid creating circular parent-child relationships:

```rust
// BAD: This creates a cycle
world.entity_mut(parent).insert(Parent(child));
```

This will cause infinite recursion or incorrect transforms.

### Missing Components

If an entity has `Parent` but the parent entity doesn't exist or doesn't have `GlobalTransform`, the system will skip that entity (query won't match).

Best practice: Always spawn entities with complete transform components using `TransformBundle`.

### Deep Hierarchies

The system handles deep hierarchies efficiently, but extremely deep nesting (>100 levels) may impact performance. For most games, hierarchies are typically 3-10 levels deep.

## Integration with Rendering

The rendering system should:
1. Query entities with `GlobalTransform` (and optionally `Visibility`)
2. Use `global_transform.matrix` directly for rendering
3. Run after all transform propagation systems

```rust
fn render_system(query: Query<(&GlobalTransform, &Mesh, &Visibility)>) {
    for (global, mesh, visibility) in query.iter() {
        if visibility.is_visible() {
            render_mesh(mesh, global.matrix);
        }
    }
}
```

## Debugging

Use the `debug_transform_hierarchy` system (debug builds only) to visualize the hierarchy:

```rust
#[cfg(debug_assertions)]
schedule.add_systems(debug_transform_hierarchy);
```

This will log the entity hierarchy with their transforms to help diagnose issues.

## Future Enhancements

Potential improvements for the future:

1. **Dirty Flagging**: Only propagate when needed, skip unchanged subtrees
2. **Parallel Propagation**: Process independent subtrees in parallel
3. **Transform Interpolation**: Smooth movement between frames
4. **Previous Transform**: Store last frame's transform for velocity computation
5. **Transform Events**: Emit events when transforms change significantly

## References

- Bevy ECS: https://docs.rs/bevy_ecs
- Transform Hierarchies: https://www.haroldserrano.com/blog/understanding-the-4x4-transformation-matrix
- Scene Graphs: https://webglfundamentals.org/webgl/lessons/webgl-scene-graph.html
