# Selection System

A comprehensive entity selection system for the Praxis editor with support for multi-entity selection, raycast picking, marquee selection, and keyboard shortcuts.

## Features

### Multi-Entity Selection
- Select single or multiple entities
- Four selection modes:
  - **Replace**: Clear existing selection and select new entities
  - **Add**: Add entities to existing selection
  - **Remove**: Remove entities from selection
  - **Toggle**: Toggle entity selection state

### Raycast Picking
- Click entities in the 3D viewport to select them
- Uses camera projection to convert screen coordinates to world-space rays
- Sphere-based intersection testing (easily extensible to use actual entity bounds)
- Finds the closest entity along the ray

### Marquee (Box) Selection
- Click and drag to create a selection rectangle
- All entities within the rectangle are selected
- Works in screen space for intuitive 2D-style selection in 3D viewport
- Supports all selection modes (add, remove, toggle)

### Keyboard Shortcuts
- **Ctrl+A**: Select all selectable entities
- **Ctrl+D**: Deselect all entities
- Modifier keys for click selection:
  - **Shift+Click**: Add to selection
  - **Ctrl+Click**: Remove from selection
  - **Alt+Click**: Toggle selection

### Selection Events
- `SelectionEvent::Selected(Vec<Entity>)`: Entities were selected
- `SelectionEvent::Deselected(Vec<Entity>)`: Entities were deselected
- `SelectionEvent::Cleared`: All selections were cleared
- `SelectionEvent::Changed`: Generic change notification
- Events are collected in a ring buffer for history tracking
- Can be drained for UI updates

## Architecture

### Components

#### `Selectable`
Marker component that must be added to entities that can be selected. Only entities with this component will be considered by the selection system.

```rust
world.spawn((
    Transform::default(),
    Selectable,  // Make this entity selectable
));
```

#### `Selected`
Marker component automatically added/removed by the selection system when entities are selected/deselected. Query for this component to implement selection-specific rendering.

```rust
fn highlight_selected(query: Query<Entity, With<Selected>>) {
    for entity in query.iter() {
        // Render highlight for selected entity
    }
}
```

### Resource

#### `SelectionSystem`
Main resource managing selection state, operations, and events.

**Key Methods:**
- `select_entity(entity, mode)`: Select a single entity
- `select_entities(entities, mode)`: Select multiple entities
- `deselect_entity(entity)`: Deselect a single entity
- `clear()`: Clear all selections
- `is_selected(entity)`: Check if entity is selected
- `selected_entities()`: Iterator over selected entities
- `drain_events()`: Consume all pending selection events
- `raycast_pick(...)`: Find entity at screen position
- `marquee_pick(...)`: Find entities in screen rectangle
- `start_marquee(pos)`: Begin marquee selection
- `update_marquee(pos)`: Update marquee rectangle
- `end_marquee()`: Finish marquee and get rectangle

### Systems

#### `update_selection_system`
Synchronizes `Selected` components with the selection state. Automatically adds/removes the `Selected` component as entities are selected/deselected.

**Schedule:**
```rust
schedule.add_systems(update_selection_system);
```

#### `handle_selection_input_system`
Handles keyboard shortcuts (Ctrl+A, Ctrl+D). Does not handle mouse input, as that requires viewport context.

**Schedule:**
```rust
schedule.add_systems(handle_selection_input_system);
```

## Usage

### Basic Setup

```rust
use praxis_editor::{SelectionSystem, Selectable, update_selection_system};
use praxis_ecs::{World, Schedule, Transform};

// Create world and insert selection system
let mut world = World::new();
world.insert_resource(SelectionSystem::new());

// Add systems
let mut schedule = Schedule::default();
schedule.add_systems(update_selection_system);

// Spawn selectable entities
world.spawn((
    Transform::default(),
    Selectable,
));
```

### Programmatic Selection

```rust
use praxis_editor::{SelectionSystem, SelectionMode};

let mut selection = SelectionSystem::new();

// Select single entity
selection.select_entity(entity, SelectionMode::Replace);

// Add to selection
selection.select_entity(another_entity, SelectionMode::Add);

// Select multiple entities
let entities = vec![entity1, entity2, entity3];
selection.select_entities(entities, SelectionMode::Replace);

// Check selection
if selection.is_selected(entity) {
    println!("Entity is selected!");
}

// Clear selection
selection.clear();
```

### Raycast Picking

```rust
use praxis_editor::{SelectionSystem, SelectionMode};
use praxis_ecs::{Query, Transform, CameraMatrices, GlobalTransform, With};

fn handle_click_selection(
    mut selection: ResMut<SelectionSystem>,
    camera_query: Query<(&Transform, &CameraMatrices), With<Camera>>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    let (camera_transform, camera_matrices) = camera_query.single();
    
    let screen_pos = Vec2::new(mouse_x, mouse_y);
    let viewport_size = Vec2::new(1920.0, 1080.0);
    
    if let Some(entity) = selection.raycast_pick(
        screen_pos,
        viewport_size,
        camera_transform,
        camera_matrices,
        &selectable_query,
    ) {
        selection.select_entity(entity, SelectionMode::Replace);
    }
}
```

### Marquee Selection

```rust
use praxis_editor::{SelectionSystem, SelectionMode};

fn handle_marquee_selection(
    mut selection: ResMut<SelectionSystem>,
    input: Res<InputState>,
    camera_query: Query<&CameraMatrices, With<Camera>>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    let mouse_pos = Vec2::new(input.mouse_position().0 as f32, input.mouse_position().1 as f32);
    
    // Start on mouse down
    if input.is_mouse_button_just_pressed(MouseButton::Left) {
        selection.start_marquee(mouse_pos);
    }
    
    // Update while dragging
    if selection.is_marquee_active() {
        selection.update_marquee(mouse_pos);
    }
    
    // End on mouse up
    if input.is_mouse_button_just_released(MouseButton::Left) {
        if let Some((rect_min, rect_max)) = selection.end_marquee() {
            let camera_matrices = camera_query.single();
            let viewport_size = Vec2::new(1920.0, 1080.0);
            
            let entities = selection.marquee_pick(
                rect_min,
                rect_max,
                viewport_size,
                camera_matrices,
                &selectable_query,
            );
            
            selection.select_entities(entities, SelectionMode::Replace);
        }
    }
}
```

### Selection Events

```rust
use praxis_editor::{SelectionSystem, SelectionEvent};

fn handle_selection_events(mut selection: ResMut<SelectionSystem>) {
    for event in selection.drain_events() {
        match event {
            SelectionEvent::Selected(entities) => {
                println!("Selected {} entities", entities.len());
            }
            SelectionEvent::Deselected(entities) => {
                println!("Deselected {} entities", entities.len());
            }
            SelectionEvent::Cleared => {
                println!("Selection cleared");
            }
            SelectionEvent::Changed => {
                println!("Selection changed");
                // Update UI, etc.
            }
        }
    }
}
```

### Visual Feedback

```rust
use praxis_editor::{Selected, Selectable};
use praxis_ecs::{Query, Commands, With, Without};

fn update_selected_visuals(
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    not_selected_query: Query<Entity, (With<Selectable>, Without<Selected>)>,
) {
    // Highlight selected entities
    for entity in selected_query.iter() {
        commands.entity(entity).insert(
            MaterialPropertiesComponent::default()
                .with_base_color([1.0, 1.0, 0.0, 1.0]) // Yellow
        );
    }
    
    // Reset non-selected entities
    for entity in not_selected_query.iter() {
        commands.entity(entity).insert(
            MaterialPropertiesComponent::default()
                .with_base_color([1.0, 1.0, 1.0, 1.0]) // White
        );
    }
}
```

## Implementation Details

### Raycast Picking Algorithm

1. **Screen to NDC**: Convert mouse position from screen space (pixels) to Normalized Device Coordinates (-1 to 1)
2. **Unproject**: Use inverse projection matrix to convert NDC to view-space ray direction
3. **Transform to World**: Rotate ray by camera rotation to get world-space direction
4. **Intersection Test**: For each selectable entity:
   - Calculate closest point on ray to entity position
   - Check if distance to ray is within picking radius
   - Track closest entity along ray
5. **Return Result**: Return closest entity, or None if no hits

### Marquee Selection Algorithm

1. **Start**: Record mouse position when left button pressed
2. **Update**: Track current mouse position while dragging
3. **End**: On mouse release, compute rectangle from start and current positions
4. **Project**: For each selectable entity:
   - Project entity world position to screen space using view-projection matrix
   - Check if screen position is within marquee rectangle
5. **Select**: Add all entities within rectangle to selection

### Event System

The selection system maintains a ring buffer of recent events with a configurable maximum size (default 100). Events are added as selection state changes and can be consumed by:
- Calling `events()` to inspect without consuming
- Calling `drain_events()` to consume and process

This allows UI panels and other systems to react to selection changes asynchronously.

## Example

See `examples/selection_demo.rs` for a complete working example demonstrating:
- Click-to-select with raycast picking
- Multi-entity selection with modifier keys
- Marquee selection by dragging
- Keyboard shortcuts (Ctrl+A, Ctrl+D)
- Visual feedback for selected entities

## Future Enhancements

- **Better Bounds Detection**: Currently uses sphere-based picking with fixed radius. Should use actual entity bounds (AABB, OBB, or mesh bounds)
- **Physics Integration**: Optionally use physics raycasts for more accurate picking
- **Selection Outline**: Dedicated outline rendering for selected entities instead of just color change
- **Selection Box Rendering**: Draw the marquee selection rectangle in the viewport
- **Undo/Redo**: Track selection history for undo/redo operations
- **Selection Groups**: Named selection sets that can be saved and restored
- **Filter by Type**: Only select entities of certain types (e.g., lights only)
