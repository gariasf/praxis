# Selection System

Comprehensive entity selection for the Praxis editor with multi-selection, raycast picking, marquee selection, and keyboard shortcuts.

## Features

- **Multi-Entity Selection**: Select single or multiple entities
- **Selection Modes**: Replace, Add, Remove, Toggle
- **Raycast Picking**: Click to select in 3D viewport
- **Marquee Selection**: Drag to select multiple entities
- **Keyboard Shortcuts**: Ctrl+A (select all), Ctrl+D (deselect)
- **Selection Events**: React to selection changes

## Components

### Selectable

Marker component for entities that can be selected:

```rust
world.spawn((
    Transform::default(),
    Selectable,  // Make this entity selectable
));
```

### Selected

Automatically added/removed when entities are selected:

```rust
fn highlight_selected(query: Query<Entity, With<Selected>>) {
    for entity in query.iter() {
        // Render highlight
    }
}
```

## Selection Modes

| Mode | Behavior |
|------|----------|
| `Replace` | Clear existing, select new |
| `Add` | Add to existing selection |
| `Remove` | Remove from selection |
| `Toggle` | Toggle selection state |

## Usage

### Basic Setup

```rust
use praxis_editor::{SelectionSystem, Selectable, update_selection_system};
use praxis_ecs::{World, Schedule};

let mut world = World::new();
world.insert_resource(SelectionSystem::new());

let mut schedule = Schedule::default();
schedule.add_systems(update_selection_system);
```

### Programmatic Selection

```rust
use praxis_editor::{SelectionSystem, SelectionMode};

let mut selection = SelectionSystem::new();

// Select single entity
selection.select_entity(entity, SelectionMode::Replace);

// Add to selection
selection.select_entity(another, SelectionMode::Add);

// Select multiple
selection.select_entities(vec![e1, e2, e3], SelectionMode::Replace);

// Check selection
if selection.is_selected(entity) { ... }

// Clear
selection.clear();
```

### Raycast Picking

```rust
fn handle_click(
    mut selection: ResMut<SelectionSystem>,
    camera_query: Query<(&Transform, &CameraMatrices), With<Camera>>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    let (cam_transform, cam_matrices) = camera_query.single();

    if let Some(entity) = selection.raycast_pick(
        screen_pos,
        viewport_size,
        cam_transform,
        cam_matrices,
        &selectable_query,
    ) {
        selection.select_entity(entity, SelectionMode::Replace);
    }
}
```

### Marquee Selection

```rust
fn handle_marquee(
    mut selection: ResMut<SelectionSystem>,
    input: Res<InputState>,
    camera_query: Query<&CameraMatrices, With<Camera>>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    let mouse = input.mouse_position();

    // Start on mouse down
    if input.is_mouse_button_just_pressed(MouseButton::Left) {
        selection.start_marquee(mouse);
    }

    // Update while dragging
    if selection.is_marquee_active() {
        selection.update_marquee(mouse);
    }

    // End on mouse up
    if input.is_mouse_button_just_released(MouseButton::Left) {
        if let Some((min, max)) = selection.end_marquee() {
            let entities = selection.marquee_pick(
                min, max, viewport_size, cam_matrices, &selectable_query,
            );
            selection.select_entities(entities, SelectionMode::Replace);
        }
    }
}
```

### Selection Events

```rust
use praxis_editor::{SelectionSystem, SelectionEvent};

fn handle_events(mut selection: ResMut<SelectionSystem>) {
    for event in selection.drain_events() {
        match event {
            SelectionEvent::Selected(entities) => { ... }
            SelectionEvent::Deselected(entities) => { ... }
            SelectionEvent::Cleared => { ... }
            SelectionEvent::Changed => { ... }
        }
    }
}
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+A | Select all |
| Ctrl+D | Deselect all |
| Shift+Click | Add to selection |
| Ctrl+Click | Remove from selection |
| Alt+Click | Toggle selection |

## Systems

### update_selection_system

Synchronizes `Selected` components with selection state:

```rust
schedule.add_systems(update_selection_system);
```

### handle_selection_input_system

Handles keyboard shortcuts (Ctrl+A, Ctrl+D):

```rust
schedule.add_systems(handle_selection_input_system);
```

## Implementation Details

### Raycast Algorithm

1. Convert mouse position to NDC (-1 to 1)
2. Unproject to view-space ray using inverse projection
3. Rotate ray to world space
4. Test intersection with each selectable entity
5. Return closest hit

### Marquee Algorithm

1. Record start position on mouse down
2. Track current position while dragging
3. Project each entity to screen space
4. Check if within rectangle bounds

## Example

```bash
cargo run --example selection_demo
```

## See Also

- [Undo/Redo System](undo-redo.md) - Selection integrates with undo/redo
- [Editor Overview](README.md) - Complete editor documentation
