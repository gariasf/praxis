# Selection System

Comprehensive entity selection functionality for the Praxis editor with multi-entity selection, raycast picking, marquee selection, and keyboard shortcuts.

## Overview

The Selection System enables intuitive entity selection in the editor through multiple interaction methods. It uses a marker component pattern with ECS integration for efficient queries and clean separation of concerns.

## Key Features

- **Multi-Entity Selection**: Select single or multiple entities with modifier keys
- **Raycast Picking**: Click entities in 3D viewport to select them
- **Marquee Selection**: Drag selection box to select multiple entities
- **Keyboard Shortcuts**: Ctrl+A (select all), Ctrl+D (deselect all)
- **Selection Events**: Event system for UI updates
- **Modifier Key Support**: Shift (add), Ctrl (remove), Alt (toggle)

## Architecture

### Components

#### `Selectable` (Marker)
Must be present on entities that can be selected.

```rust
world.spawn((
    Transform::default(),
    Selectable,  // Makes this entity selectable
));
```

#### `Selected` (Marker)
Automatically added/removed by the selection system when entities are selected/deselected.

```rust
// Query selected entities
fn highlight_selected(query: Query<&Transform, With<Selected>>) {
    for transform in query.iter() {
        // Render selection highlight
    }
}
```

### Resource

#### `SelectionSystem`
Main resource managing selection state and operations.

**Key Methods**:
- `select_entity(entity, mode)` - Select single entity
- `select_entities(entities, mode)` - Select multiple entities
- `clear()` - Clear all selections
- `is_selected(entity)` - Check if entity is selected
- `selected_entities()` - Iterator over selected entities
- `raycast_pick(...)` - Find entity at screen position
- `start_marquee(pos)` / `update_marquee(pos)` / `end_marquee()` - Box selection

## Selection Modes

### Replace (Default)
Clears existing selection and selects new entities.

**Input**: Click (no modifiers)  
**Use case**: Selecting a single entity

### Add
Adds entities to existing selection without clearing.

**Input**: Shift+Click  
**Use case**: Building up a selection set

### Remove
Removes entities from selection.

**Input**: Ctrl+Click  
**Use case**: Deselecting specific entities

### Toggle
Toggles entity selection state.

**Input**: Alt+Click  
**Use case**: Quick selection adjustments

## Usage

### Basic Setup

```rust
use praxis_editor::{SelectionSystem, Selectable, update_selection_system};

// Create world and add selection system
let mut world = World::new();
world.insert_resource(SelectionSystem::new());

// Add system to schedule
schedule.add_systems(update_selection_system);

// Spawn selectable entities
world.spawn((Transform::default(), Selectable));
```

### Programmatic Selection

```rust
use praxis_editor::{SelectionSystem, SelectionMode};

let mut selection = world.resource_mut::<SelectionSystem>();

// Select single entity
selection.select_entity(entity, SelectionMode::Replace);

// Select multiple entities
selection.select_entities(vec![entity1, entity2], SelectionMode::Add);

// Check selection
if selection.is_selected(entity) {
    println!("Selected!");
}

// Clear all
selection.clear();
```

### Raycast Picking

Convert mouse clicks to entity selections:

```rust
// On mouse click in viewport
if let Some(entity) = selection.raycast_pick(
    mouse_pos,
    viewport_size,
    &camera,
    &camera_matrices,
    &camera_transform,
    &selectable_query,
) {
    let mode = if shift_pressed {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    };
    selection.select_entity(entity, mode);
}
```

### Marquee (Box) Selection

Drag to select multiple entities:

```rust
// On mouse down - start marquee
if mouse_down {
    selection.start_marquee(mouse_pos);
}

// During drag - update marquee
if dragging {
    selection.update_marquee(mouse_pos);
}

// On mouse up - complete selection
if mouse_up {
    let entities = selection.end_marquee(
        &camera,
        &camera_matrices,
        &camera_transform,
        &selectable_query,
        SelectionMode::Replace,
    );
}
```

## Selection Events

React to selection changes:

```rust
use praxis_editor::SelectionEvent;

for event in selection.drain_events() {
    match event {
        SelectionEvent::Selected(entities) => {
            // Update inspector to show selected entities
        }
        SelectionEvent::Deselected(entities) => {
            // Clear inspector or update UI
        }
        SelectionEvent::Cleared => {
            // Hide selection-dependent UI
        }
        SelectionEvent::Changed => {
            // Generic selection change
        }
    }
}
```

## Keyboard Shortcuts

Built-in keyboard shortcuts (handled by `handle_selection_input_system`):

| Shortcut | Action |
|----------|--------|
| **Ctrl+A** | Select all selectable entities |
| **Ctrl+D** | Deselect all entities |

### Custom Shortcuts

Disable built-in input and implement your own:

```rust
selection.set_input_enabled(false);

// Custom handling
if input.ctrl() && input.just_pressed(KeyCode::KeyA) {
    let all: Vec<Entity> = selectable_query.iter().collect();
    selection.select_entities(all, SelectionMode::Replace);
}
```

## ECS Integration

### Systems

**`update_selection_system`**: Synchronizes `Selected` components with `SelectionSystem` state. Run this every frame.

**`handle_selection_input_system`**: Processes keyboard shortcuts (Ctrl+A, Ctrl+D). Add to your schedule.

```rust
schedule.add_systems((
    handle_selection_input_system,
    update_selection_system,
).chain());
```

### Visual Feedback

Use the `Selected` component for rendering:

```rust
fn highlight_selected_system(
    mut materials: Query<&mut MaterialProperties>,
    selected: Query<Entity, With<Selected>>,
) {
    let selected_set: HashSet<_> = selected.iter().collect();
    
    for (entity, mut material) in materials.iter_mut() {
        if selected_set.contains(&entity) {
            material.base_color = [1.0, 1.0, 0.0, 1.0]; // Yellow highlight
        }
    }
}
```

## Advanced Features

### Hierarchical Selection

Select entity and all children:

```rust
fn select_hierarchy(
    entity: Entity,
    selection: &mut SelectionSystem,
    children_query: &Query<&Children>,
) {
    let mut entities = vec![entity];
    collect_children_recursive(entity, &mut entities, children_query);
    selection.select_entities(entities, SelectionMode::Add);
}
```

### Selection Filtering

Filter selection by component type:

```rust
// Select only entities with Light component
let lights: Vec<Entity> = selectable_query
    .iter()
    .filter(|&(e, _)| world.get::<Light>(e).is_some())
    .map(|(e, _)| e)
    .collect();

selection.select_entities(lights, SelectionMode::Replace);
```

## Performance Considerations

- **Selected entities**: O(1) lookup via HashSet, efficient for 1-1000 entities
- **Selectable entities**: O(n) iteration for raycast/marquee, optimize with spatial partitioning
- **Event buffer**: Ring buffer with 100 event capacity

### Optimization Tips

1. **Spatial Partitioning**: Use octree/BVH to reduce raycast candidates
2. **Batch Operations**: Use `select_entities()` instead of multiple `select_entity()` calls
3. **Cull Off-Screen**: Skip raycast tests for entities outside camera frustum
4. **Event Processing**: Drain events once per frame to avoid accumulation

## Troubleshooting

### Selection Not Working
- Verify entity has `Selectable` component
- Check `update_selection_system` is in schedule
- Ensure `SelectionSystem` resource exists
- Verify camera matrices are correct for raycast

### Multiple Entities Selected When Clicking One
- Entity bounds may be too large (adjust picking radius)
- Implement depth sorting to pick closest entity
- Use more accurate bounds (AABB vs sphere)

### Events Not Firing
- Check if events were already drained elsewhere
- Use `events()` to inspect without consuming
- Verify event buffer isn't full

## Examples

See `examples/selection_demo.rs` for a complete working example demonstrating:
- Click-to-select with raycast picking
- Multi-entity selection with modifier keys
- Marquee selection
- Keyboard shortcuts
- Visual feedback for selected entities

## Technical Details

For implementation details, see:
- [crates/praxis_editor/SELECTION_SYSTEM.md](../../crates/praxis_editor/SELECTION_SYSTEM.md) - Complete implementation documentation
- Raycast picking algorithm details
- Marquee selection algorithm details
- Event system internals

## See Also

- [Gizmos](gizmos.md) - Transform manipulation of selected entities
- [Hierarchy Panel](hierarchy-panel.md) - Tree view with selection integration
- [Inspector Panel](inspector.md) - Edit components of selected entities
- [Editor Camera](editor-camera.md) - Focus camera on selection
