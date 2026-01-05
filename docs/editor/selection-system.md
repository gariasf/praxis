# Selection System Guide

## Overview

The Selection System provides comprehensive entity selection functionality for the Praxis editor, including multi-entity selection, raycast picking, marquee selection, keyboard shortcuts, and event notifications.

## Architecture

### Core Components

The selection system consists of three main parts:

1. **SelectionSystem** (Resource): Manages selection state and operations
2. **Selectable** (Component): Marks entities that can be selected
3. **Selected** (Component): Marks currently selected entities

### Design Rationale

The dual-component design separates concerns:
- `Selectable`: Persistent capability marker
- `Selected`: Transient state marker (added/removed frequently)

This enables efficient queries and clear intent:
```rust
// Query all selectable entities
Query<Entity, With<Selectable>>

// Query currently selected entities
Query<Entity, With<Selected>>

// Query selectable but not selected
Query<Entity, (With<Selectable>, Without<Selected>)>
```

## Selection Modes

The system supports four selection modes for different use cases:

### Replace Mode

Clears existing selection and selects new entities:

```rust
selection.select_entity(entity, SelectionMode::Replace);
```

**Use Cases**: Primary selection, single-click without modifiers

### Add Mode

Adds entities to existing selection:

```rust
selection.select_entity(entity, SelectionMode::Add);
```

**Use Cases**: Shift+Click, building selection set

### Remove Mode

Removes entities from selection:

```rust
selection.select_entity(entity, SelectionMode::Remove);
```

**Use Cases**: Ctrl+Click on selected entity, deselecting

### Toggle Mode

Toggles entity selection state:

```rust
selection.select_entity(entity, SelectionMode::Toggle);
```

**Use Cases**: Alt+Click, quick selection changes

## API Reference

### SelectionSystem

Resource managing selection state:

```rust
pub struct SelectionSystem {
    selected: HashSet<Entity>,
    events: RingBuffer<SelectionEvent>,
    marquee: Option<MarqueeSelection>,
    input_enabled: bool,
}
```

**Core Methods**:

```rust
// Single entity operations
pub fn select_entity(&mut self, entity: Entity, mode: SelectionMode);
pub fn deselect_entity(&mut self, entity: Entity);
pub fn is_selected(&self, entity: Entity) -> bool;

// Batch operations
pub fn select_entities(&mut self, entities: Vec<Entity>, mode: SelectionMode);
pub fn clear(&mut self);

// Query operations
pub fn selected_entities(&self) -> impl Iterator<Item = Entity>;
pub fn selected_count(&self) -> usize;

// Event handling
pub fn events(&self) -> &[SelectionEvent];
pub fn drain_events(&mut self) -> Vec<SelectionEvent>;

// Input configuration
pub fn set_input_enabled(&mut self, enabled: bool);
pub fn is_input_enabled(&self) -> bool;
```

### Raycast Picking

Click-to-select functionality using ray-sphere intersection:

```rust
pub fn raycast_pick(
    &self,
    mouse_pos: Vec2,
    viewport_size: Vec2,
    camera: &Camera,
    camera_matrices: &CameraMatrices,
    camera_transform: &GlobalTransform,
    selectable_query: &Query<(Entity, &GlobalTransform), With<Selectable>>,
) -> Option<Entity>;
```

**Algorithm**:
1. Convert mouse screen position to NDC [-1, 1]
2. Unproject to view space using inverse projection
3. Transform to world space using camera rotation
4. Test all selectable entities for intersection
5. Return closest entity along ray

**Example**:

```rust
if input.mouse_button_just_pressed(MouseButton::Left) {
    let picked = selection.raycast_pick(
        input.mouse_position(),
        viewport_size,
        &camera,
        &camera_matrices,
        &camera_transform,
        &selectable_query,
    );
    
    if let Some(entity) = picked {
        let mode = if input.key_pressed(KeyCode::Shift) {
            SelectionMode::Add
        } else {
            SelectionMode::Replace
        };
        selection.select_entity(entity, mode);
    }
}
```

### Marquee Selection

Box selection for selecting multiple entities at once:

```rust
// Start marquee (on mouse down)
pub fn start_marquee(&mut self, screen_pos: Vec2);

// Update during drag
pub fn update_marquee(&mut self, screen_pos: Vec2);

// Complete and select (on mouse up)
pub fn end_marquee(
    &mut self,
    camera: &Camera,
    camera_matrices: &CameraMatrices,
    camera_transform: &GlobalTransform,
    selectable_query: &Query<(Entity, &GlobalTransform), With<Selectable>>,
    mode: SelectionMode,
) -> Vec<Entity>;
```

**Algorithm**:
1. Track start position on mouse down
2. Update end position during drag
3. Compute screen-space rectangle
4. Project each entity to screen space
5. Select entities within rectangle

**Example**:

```rust
// On mouse down
if input.mouse_button_just_pressed(MouseButton::Left) {
    selection.start_marquee(input.mouse_position());
}

// During drag
if input.mouse_button_pressed(MouseButton::Left) {
    selection.update_marquee(input.mouse_position());
}

// On mouse up
if input.mouse_button_just_released(MouseButton::Left) {
    let mode = if input.key_pressed(KeyCode::Shift) {
        SelectionMode::Add
    } else {
        SelectionMode::Replace
    };
    
    let selected = selection.end_marquee(
        &camera,
        &camera_matrices,
        &camera_transform,
        &selectable_query,
        mode,
    );
    
    println!("Selected {} entities", selected.len());
}
```

**Click vs. Drag Detection**:

The system distinguishes clicks from drags:
- **Click**: Start and end positions within 5 pixels → Raycast pick
- **Drag**: Distance > 5 pixels → Marquee selection

## ECS Integration

### Components

Mark entities as selectable:

```rust
use praxis_editor::{Selectable, Selected};

// Spawn selectable entity
world.spawn((
    Transform::default(),
    Mesh::default(),
    Selectable,  // Can be selected
));

// Check if entity is selected
fn highlight_selected(query: Query<&MeshRenderer, With<Selected>>) {
    for renderer in query.iter() {
        renderer.set_color(Color::BLUE);
    }
}
```

### Systems

Two ECS systems synchronize selection state:

#### `update_selection_system`

Synchronizes `Selected` components with SelectionSystem:

```rust
pub fn update_selection_system(
    mut commands: Commands,
    selection: Res<SelectionSystem>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
)
```

**Behavior**:
- Adds `Selected` to newly selected entities
- Removes `Selected` from deselected entities
- Validates entities still exist

#### `handle_selection_input_system`

Processes keyboard shortcuts:

```rust
pub fn handle_selection_input_system(
    input: Res<InputState>,
    mut selection: ResMut<SelectionSystem>,
    selectable_query: Query<Entity, With<Selectable>>,
)
```

**Shortcuts**:
- `Ctrl+A`: Select all selectable entities
- `Ctrl+D`: Deselect all entities

### System Setup

Add systems to your schedule:

```rust
use praxis_editor::{update_selection_system, handle_selection_input_system};
use bevy_ecs::schedule::Schedule;

let mut schedule = Schedule::default();
schedule.add_systems((
    handle_selection_input_system,
    update_selection_system,
).chain());
```

## Keyboard Shortcuts

### Built-in Shortcuts

| Shortcut | Action | Mode |
|----------|--------|------|
| Click | Select entity | Replace |
| Shift+Click | Add to selection | Add |
| Ctrl+Click | Remove from selection | Remove |
| Alt+Click | Toggle selection | Toggle |
| Ctrl+A | Select all | Replace |
| Ctrl+D | Deselect all | Clear |

### Custom Shortcuts

Disable built-in input and implement custom shortcuts:

```rust
selection.set_input_enabled(false);

// Custom shortcut handling
if input.key_pressed(KeyCode::A) && input.key_pressed(KeyCode::LControl) {
    // Your custom select-all logic
    let entities: Vec<Entity> = selectable_query.iter().collect();
    selection.select_entities(entities, SelectionMode::Replace);
}
```

## Selection Events

The system generates events for selection changes:

### Event Types

```rust
pub enum SelectionEvent {
    Selected(Vec<Entity>),    // Entities were selected
    Deselected(Vec<Entity>),  // Entities were deselected
    Cleared,                   // All selections cleared
    Changed,                   // Generic change notification
}
```

### Event Handling

Two ways to access events:

#### Inspect Events (Non-consuming)

```rust
// Read events without removing them
for event in selection.events() {
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
        }
    }
}
```

#### Drain Events (Consuming)

```rust
// Process and remove events
for event in selection.drain_events() {
    match event {
        SelectionEvent::Selected(entities) => {
            update_inspector(entities);
        }
        _ => {}
    }
}
```

### Event Usage Examples

**Update Inspector Panel**:
```rust
for event in selection.drain_events() {
    if let SelectionEvent::Selected(entities) = event {
        inspector.set_targets(entities);
    }
}
```

**Update Outline Renderer**:
```rust
for event in selection.events() {
    match event {
        SelectionEvent::Selected(entities) => {
            outline_renderer.add_entities(entities);
        }
        SelectionEvent::Deselected(entities) => {
            outline_renderer.remove_entities(entities);
        }
        SelectionEvent::Cleared => {
            outline_renderer.clear();
        }
        _ => {}
    }
}
```

**Analytics/Logging**:
```rust
for event in selection.events() {
    analytics.log_editor_event("selection_changed", event);
}
```

## Advanced Usage

### Selection Filtering

Filter selection by component type:

```rust
// Select only entities with Light component
let lights: Vec<Entity> = selectable_query
    .iter()
    .filter(|(entity, _)| world.get::<Light>(*entity).is_some())
    .map(|(entity, _)| entity)
    .collect();

selection.select_entities(lights, SelectionMode::Replace);
```

### Hierarchical Selection

Select entity and all children:

```rust
fn select_hierarchy(
    entity: Entity,
    selection: &mut SelectionSystem,
    children_query: &Query<&Children>,
) {
    let mut entities = vec![entity];
    
    // Recursively collect children
    fn collect_children(
        entity: Entity,
        entities: &mut Vec<Entity>,
        children_query: &Query<&Children>,
    ) {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                entities.push(*child);
                collect_children(*child, entities, children_query);
            }
        }
    }
    
    collect_children(entity, &mut entities, children_query);
    selection.select_entities(entities, SelectionMode::Add);
}
```

### Selection Groups

Save and restore selection sets:

```rust
use std::collections::HashMap;

struct SelectionGroups {
    groups: HashMap<String, Vec<Entity>>,
}

impl SelectionGroups {
    pub fn save(&mut self, name: String, selection: &SelectionSystem) {
        let entities = selection.selected_entities().collect();
        self.groups.insert(name, entities);
    }
    
    pub fn restore(&self, name: &str, selection: &mut SelectionSystem) {
        if let Some(entities) = self.groups.get(name) {
            selection.select_entities(entities.clone(), SelectionMode::Replace);
        }
    }
}
```

### Custom Intersection Tests

Extend raycast picking with custom bounds:

```rust
// Use AABB instead of sphere
pub fn raycast_pick_aabb(
    ray_origin: Vec3,
    ray_direction: Vec3,
    query: &Query<(Entity, &GlobalTransform, &Aabb), With<Selectable>>,
) -> Option<Entity> {
    let mut closest_entity = None;
    let mut closest_distance = f32::MAX;
    
    for (entity, transform, aabb) in query.iter() {
        if let Some(distance) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            aabb,
            transform.translation(),
        ) {
            if distance < closest_distance {
                closest_distance = distance;
                closest_entity = Some(entity);
            }
        }
    }
    
    closest_entity
}
```

## Performance Considerations

### Selection Limits

The system is optimized for typical editor use:
- **Selected entities**: 1-1000 (HashSet lookup: O(1))
- **Selectable entities**: 1-100,000 (Query iteration: O(n))
- **Event buffer**: 100 events (ring buffer, configurable)

### Optimization Tips

1. **Use spatial partitioning**: Reduce entities tested for picking
```rust
let candidates = octree.query_frustum(&camera_frustum);
let picked = raycast_pick_subset(ray, &candidates);
```

2. **Batch operations**: Use `select_entities()` instead of multiple `select_entity()` calls
```rust
// Slow
for entity in entities {
    selection.select_entity(entity, SelectionMode::Add);
}

// Fast
selection.select_entities(entities, SelectionMode::Add);
```

3. **Event processing**: Drain events when done processing
```rust
// Process events once per frame
for event in selection.drain_events() {
    handle_event(event);
}
```

4. **Cull off-screen entities**: Skip raycast tests for culled entities
```rust
let visible_entities: Vec<_> = selectable_query
    .iter()
    .filter(|(_, transform)| frustum.contains_point(transform.translation()))
    .collect();
```

## Troubleshooting

### Selection Not Working

**Problem**: Entities don't select when clicked
**Solutions**:
- Verify entity has `Selectable` component
- Check raycast is hitting the entity (add debug visualization)
- Ensure `update_selection_system` is running
- Verify camera matrices are correct

### Multiple Entities Selected When Clicking

**Problem**: Click selects multiple overlapping entities
**Solutions**:
- Check entity bounds (may be too large)
- Use depth sorting to pick closest entity
- Implement occlusion testing

### Performance Issues with Large Scenes

**Problem**: Selection is slow with many entities
**Solutions**:
- Implement spatial partitioning (octree, BVH)
- Cull off-screen entities before raycast
- Use simpler intersection tests (sphere vs. mesh)
- Batch selection operations

### Events Not Firing

**Problem**: SelectionEvent not received
**Solutions**:
- Check if events were already drained
- Use `events()` to inspect without consuming
- Verify selection actually changed
- Check event buffer isn't full (increase size)

## Complete Example

```rust
use praxis_editor::*;
use praxis_ecs::*;
use praxis_input::InputState;

// Setup
fn setup(mut world: World) {
    // Add selection resource
    world.insert_resource(SelectionSystem::new());
    
    // Spawn selectable entities
    for x in 0..10 {
        for z in 0..10 {
            world.spawn((
                Transform::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0),
                Mesh::cube(),
                Selectable,
            ));
        }
    }
    
    // Setup systems
    let mut schedule = Schedule::default();
    schedule.add_systems((
        handle_input_system,
        handle_selection_input_system,
        update_selection_system,
        highlight_selected_system,
    ).chain());
    
    world.insert_resource(schedule);
}

// Handle mouse input
fn handle_input_system(
    input: Res<InputState>,
    mut selection: ResMut<SelectionSystem>,
    camera_query: Query<(&Camera, &CameraMatrices, &GlobalTransform)>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    let (camera, matrices, transform) = camera_query.single();
    
    // Click selection
    if input.mouse_button_just_pressed(MouseButton::Left) {
        let picked = selection.raycast_pick(
            input.mouse_position(),
            Vec2::new(1920.0, 1080.0),
            camera,
            matrices,
            transform,
            &selectable_query,
        );
        
        if let Some(entity) = picked {
            let mode = if input.key_pressed(KeyCode::Shift) {
                SelectionMode::Add
            } else {
                SelectionMode::Replace
            };
            selection.select_entity(entity, mode);
        }
    }
}

// Visual feedback for selected entities
fn highlight_selected_system(
    mut query: Query<&mut Material>,
    selected_query: Query<Entity, With<Selected>>,
) {
    let selected: HashSet<_> = selected_query.iter().collect();
    
    for (entity, mut material) in query.iter_mut() {
        if selected.contains(&entity) {
            material.set_color(Color::rgb(0.5, 0.7, 1.0));
        } else {
            material.set_color(Color::rgb(0.7, 0.7, 0.7));
        }
    }
}
```

## See Also

- [Editor Camera Guide](camera.md)
- [Hierarchy Panel Guide](hierarchy.md)
- [Inspector Panel Guide](inspector.md)
- [Gizmos Guide](gizmos.md)
- [Undo/Redo System](undo-redo.md)
