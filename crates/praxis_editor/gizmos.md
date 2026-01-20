# Transform Gizmos

This document describes the transform gizmo system for visual 3D manipulation of entities in the editor.

## Overview

The gizmo system provides interactive visual tools for transforming entities in the 3D viewport. Gizmos are rendered as colored lines and cones, allowing intuitive manipulation via ray-based interaction with axis-constrained movement, rotation, and scaling.

## Features

- **Visual 3D Gizmos**: Rendered as colored lines (X=red, Y=green, Z=blue) with arrow heads for translate mode
- **Ray-based Interaction**: Click and drag axes to manipulate transforms
- **Axis Constraints**: Operations are constrained to the selected axis (X, Y, or Z)
- **Local/World Space**: Toggle between local and world space transformation modes
- **Three Modes**: Translate (move), Rotate, and Scale
- **Undo/Redo**: All operations are integrated with the undo/redo system
- **Multi-entity Support**: Manipulate multiple selected entities simultaneously
- **Hover Feedback**: Axes highlight when hovered over

## Architecture

### Core Components

#### `GizmoSystem` (Resource)

The main system resource that manages all gizmo state and interaction:

```rust
pub struct GizmoSystem {
    mode: GizmoMode,              // Current mode (translate/rotate/scale)
    space: GizmoSpace,            // Current space (world/local)
    active_gizmo: Option<Gizmo>,  // Active gizmo for selection
    interaction: Option<GizmoInteraction>, // Current drag state
    enabled: bool,                // Whether gizmos are enabled
}
```

**Key Methods:**
- `set_mode(mode)` / `cycle_mode()` - Change gizmo mode
- `set_space(space)` / `toggle_space()` - Change coordinate space
- `update_gizmo_for_selection(entities)` - Update gizmo based on selection
- `start_interaction(...)` - Begin dragging an axis
- `update_interaction(...)` - Update drag with new mouse position
- `end_interaction()` - Finish drag and return interaction data
- `update_hover(...)` - Update hover state for visual feedback

#### `Gizmo`

Individual gizmo instance representing the visual tool:

```rust
pub struct Gizmo {
    position: Vec3,                  // Center position in world space
    rotation: Quat,                  // Rotation (for local space)
    size: f32,                       // Visualization size
    hovered_axis: Option<GizmoAxis>, // Currently hovered axis
}
```

**Key Methods:**
- `raycast(...)` - Performs ray-axis intersection testing
- `get_lines(...)` - Returns lines to render for visualization
- `from_transform(...)` - Create from entity transform

#### `GizmoMode`

Determines which transform property to manipulate:

- **`Translate`**: Move entities along axis
- **`Rotate`**: Rotate entities around axis
- **`Scale`**: Scale entities along axis

#### `GizmoSpace`

Coordinate space for transformations:

- **`World`**: Gizmo axes align with world X/Y/Z coordinates
- **`Local`**: Gizmo axes align with the entity's local rotation

#### `GizmoAxis`

Individual axis identifier:

- **`X`**: Red axis
- **`Y`**: Green axis
- **`Z`**: Blue axis

Each axis provides:
- `color()` - Base color (red/green/blue)
- `highlight_color()` - Brighter color when hovered
- `direction()` - Unit vector direction

#### `GizmoInteraction`

State for an active drag operation:

```rust
pub struct GizmoInteraction {
    axis: GizmoAxis,                      // Axis being dragged
    start_screen_pos: Vec2,               // Initial mouse position
    start_gizmo_position: Vec3,           // Initial gizmo position
    initial_transforms: Vec<(Entity, Transform)>, // Pre-drag transforms
    drag_delta: f32,                      // Current drag amount
}
```

#### `TransformGizmo` (Component)

Optional component that can be attached to entities to enable gizmo rendering:

```rust
#[derive(Component)]
pub struct TransformGizmo {
    visible: bool,          // Whether gizmo is visible
    size_multiplier: f32,   // Custom size scaling
}
```

## Usage

### Basic Setup

```rust
use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};
use praxis_ecs::World;

// Initialize in your world
world.insert_resource(GizmoSystem::new());

// Configure gizmo
let mut gizmo_system = world.resource_mut::<GizmoSystem>();
gizmo_system.set_mode(GizmoMode::Translate);
gizmo_system.set_space(GizmoSpace::World);
```

### Updating Gizmo for Selection

When selection changes, update the gizmo to match:

```rust
// Assuming you have selected entities with transforms
let selected: Vec<(Entity, &Transform)> = /* ... */;

let mut gizmo_system = world.resource_mut::<GizmoSystem>();
gizmo_system.update_gizmo_for_selection(&selected);
```

### Interaction Flow

1. **Update Hover**: Call on mouse move to highlight axes
```rust
gizmo_system.update_hover(mouse_pos, camera_matrices, camera_pos);
```

2. **Start Interaction**: Call on mouse down
```rust
let entities = selected.iter()
    .map(|(e, t)| (*e, **t))
    .collect();

if gizmo_system.start_interaction(
    mouse_pos,
    camera_matrices,
    camera_pos,
    entities
) {
    // Interaction started successfully
}
```

3. **Update Interaction**: Call on mouse drag
```rust
if let Some(new_transforms) = gizmo_system.update_interaction(
    current_mouse_pos,
    camera_matrices,
    camera_pos,
) {
    // Apply new transforms to entities
    for (entity, transform) in new_transforms {
        world.get_mut::<Transform>(entity).map(|mut t| *t = transform);
    }
}
```

4. **End Interaction**: Call on mouse up
```rust
if let Some(interaction) = gizmo_system.end_interaction() {
    // Create undo command
    let old_transforms: Vec<Transform> = interaction.initial_transforms
        .iter()
        .map(|(_, t)| *t)
        .collect();
    
    let new_transforms: Vec<Transform> = /* get current transforms */;
    let entities: Vec<Entity> = interaction.initial_transforms
        .iter()
        .map(|(e, _)| *e)
        .collect();
    
    let command = TransformCommand::new(entities, old_transforms, new_transforms);
    undo_system.execute_command(Box::new(command));
}
```

### Rendering Gizmos

To render gizmos, extract line data and render with debug drawing:

```rust
if let Some(gizmo) = gizmo_system.active_gizmo() {
    let lines = gizmo.get_lines(gizmo_system.mode(), gizmo_system.space());
    
    for (start, end, color) in lines {
        // Render line from start to end with color
        debug_draw.line(start, end, color);
    }
}
```

### Keyboard Shortcuts

Common shortcuts to implement:

- **W**: Set translate mode
- **E**: Set rotate mode  
- **R**: Set scale mode
- **Tab**: Cycle through modes
- **Spacebar**: Toggle world/local space

```rust
if input.is_key_just_pressed(KeyCode::KeyW) {
    gizmo_system.set_mode(GizmoMode::Translate);
}
if input.is_key_just_pressed(KeyCode::KeyE) {
    gizmo_system.set_mode(GizmoMode::Rotate);
}
if input.is_key_just_pressed(KeyCode::KeyR) {
    gizmo_system.set_mode(GizmoMode::Scale);
}
if input.is_key_just_pressed(KeyCode::Tab) {
    gizmo_system.cycle_mode();
}
if input.is_key_just_pressed(KeyCode::Space) {
    gizmo_system.toggle_space();
}
```

## Ray-based Interaction

### Ray Casting

The gizmo system uses ray-line distance calculations to determine if the mouse is over an axis:

1. Convert screen coordinates to a ray in world space
2. For each axis, calculate the closest distance between the ray and the axis line
3. If distance is below threshold, the axis is considered "hit"
4. Return the closest hit axis

### Axis Constraints

When dragging, all movement is constrained to the selected axis:

- **Translate**: Movement along the axis direction
- **Rotate**: Rotation around the axis
- **Scale**: Scaling along the axis

The drag delta is calculated based on screen-space mouse movement and converted to world-space manipulation.

## Local vs World Space

### World Space

Gizmo axes align with the world coordinate system:
- X axis always points along world X
- Y axis always points along world Y
- Z axis always points along world Z

This is useful for:
- Aligning objects to the world grid
- Moving objects in cardinal directions
- Placing objects at specific world positions

### Local Space

Gizmo axes align with the entity's local coordinate system:
- Axes rotate with the entity
- Transformations are relative to the entity's orientation

This is useful for:
- Moving objects relative to their facing direction
- Rotating objects naturally
- Manipulating child objects in parent space

## Integration with Undo/Redo

All gizmo operations create `TransformCommand` objects that can be undone/redone:

```rust
pub struct TransformCommand {
    entities: Vec<Entity>,
    old_transforms: Vec<Transform>,
    new_transforms: Vec<Transform>,
}
```

The undo/redo system (see `UNDO_REDO.md`) manages the command history:

```rust
// Execute command (adds to undo stack)
undo_system.execute_command(Box::new(command));

// Undo
undo_system.undo();

// Redo
undo_system.redo();
```

## Implementation Details

### Ray-Line Distance

The raycast algorithm calculates the minimum distance between a ray and a line segment:

1. Project the gizmo position onto the ray to check if it's in front of camera
2. Find parametric closest points on both ray and axis line
3. Calculate distance between these points
4. Compare against pick threshold (20% of gizmo size)

### Multi-Entity Handling

When multiple entities are selected:
- Gizmo position is the average of all entity positions
- Gizmo rotation is the first entity's rotation (or identity for multiple)
- All entities are transformed relative to the gizmo
- Undo/redo works on all entities simultaneously

### Sensitivity and Scale

The drag sensitivity is tunable via the `sensitivity` constant in `update_interaction()`. Default is 0.01 which means:
- 100 pixels of mouse movement = 1 unit of translation/rotation/scale

The gizmo size is scaled based on:
- Distance from camera (optional - implement screen-space constant size)
- Custom size multiplier per entity
- Mode-specific length multipliers (translate: 2x, rotate: 1.5x, scale: 2x)

## Debug Drawing Requirements

To render gizmos, you need a debug drawing system that can:

1. Draw 3D lines in world space
2. Support colored lines
3. Render with depth testing (gizmos should be visible through objects with transparency)
4. Support multiple line segments per frame

Example debug draw API:

```rust
pub trait DebugDraw {
    fn line(&mut self, start: Vec3, end: Vec3, color: Vec3);
    fn lines(&mut self, lines: &[(Vec3, Vec3, Vec3)]);
}
```

## Future Enhancements

Potential improvements to the gizmo system:

1. **Screen-space Constant Size**: Scale gizmo based on camera distance
2. **Plane Dragging**: Add support for dragging on planes (XY, XZ, YZ)
3. **Snapping**: Grid snapping for translate, angle snapping for rotate
4. **Visual Feedback**: Show numeric feedback during drag
5. **Multi-axis Operations**: Support for 2-axis or 3-axis simultaneous operations
6. **Custom Gizmo Shapes**: Circles for rotation, boxes for scale
7. **Gizmo Stencil**: Render gizmos on top of geometry always
8. **Touch Support**: Support for touch-based manipulation on mobile/tablets
9. **Gizmo History**: Visualize previous transform states

## Testing

The gizmo module includes comprehensive tests covering:

- Mode cycling and space toggling
- Gizmo creation and initialization
- Axis color and direction mappings
- Component builder patterns
- Interaction state management
- Mode/space changes canceling interactions

Run tests with:
```bash
cargo test -p praxis_editor gizmo
```

## See Also

- `SELECTION_SYSTEM.md` - Entity selection integration
- `UNDO_REDO.md` - Undo/redo command system
- `ecs/components.rs` - Transform component definition
