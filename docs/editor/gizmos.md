# Transform Gizmos

Interactive 3D visual tools for manipulating entity transforms in the editor viewport.

## Overview

The gizmo system provides intuitive visual manipulation of entities through colored 3D axes. Gizmos appear when entities are selected and allow direct manipulation via mouse interaction.

## Features

- **Visual 3D Gizmos**: Rendered as colored axes (X=red, Y=green, Z=blue)
- **Three Modes**: Translate (move), Rotate, Scale
- **Two Coordinate Spaces**: World space or Local space
- **Ray-based Interaction**: Click and drag individual axes
- **Multi-entity Support**: Transform multiple selected entities simultaneously
- **Undo/Redo Integration**: All operations create undoable commands
- **Visual Feedback**: Axes highlight on hover

## Controls

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **W** | Switch to Translate mode |
| **E** | Switch to Rotate mode |
| **R** | Switch to Scale mode |
| **X** | Toggle between World/Local space |

### Mouse Controls

1. **Hover over axis**: Axis highlights in lighter color
2. **Click axis**: Begin manipulation
3. **Drag**: Transform along the selected axis
4. **Release**: Complete operation and create undo command
5. **Esc**: Cancel manipulation (revert to original transform)

## Gizmo Modes

### Translate Mode (W)
Move entities along a single axis (X, Y, or Z).

**Visual**: Lines with arrow heads  
**Interaction**: Drag axis to slide entity along that direction  
**Use case**: Positioning objects in 3D space

### Rotate Mode (E)
Rotate entities around a single axis.

**Visual**: Circular arcs around each axis  
**Interaction**: Drag axis to rotate around that axis  
**Use case**: Orienting objects, adjusting facing direction

### Scale Mode (R)
Scale entities along a single axis.

**Visual**: Lines with cube handles  
**Interaction**: Drag axis to stretch/shrink along that axis  
**Use case**: Adjusting object dimensions

## Coordinate Spaces

### World Space
Gizmo axes align with the world coordinate system (X, Y, Z always point the same direction).

**Best for**:
- Aligning objects to world grid
- Moving objects in cardinal directions
- Placing objects at specific world coordinates

### Local Space
Gizmo axes rotate with the entity's orientation.

**Best for**:
- Moving objects relative to their facing direction
- Rotating objects around their own axes
- Natural manipulation of rotated objects

Press **X** to toggle between spaces.

## Usage Example

```rust
use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};

// Initialize gizmo system
world.insert_resource(GizmoSystem::new());

// Configure mode and space
let mut gizmo = world.resource_mut::<GizmoSystem>();
gizmo.set_mode(GizmoMode::Translate);
gizmo.set_space(GizmoSpace::World);

// Update gizmo for current selection
let selected: Vec<(Entity, Transform)> = /* get selected entities */;
gizmo.update_gizmo_for_selection(&selected);
```

## Interaction Flow

The gizmo system follows a standard interaction pattern:

1. **Update Hover**: Call `update_hover()` on mouse move to highlight axes under cursor
2. **Start Interaction**: Call `start_interaction()` on mouse down over an axis
3. **Update Drag**: Call `update_interaction()` during mouse drag to update transforms
4. **End Interaction**: Call `end_interaction()` on mouse up to finalize and create undo command

This pattern is handled automatically by `EditorState` when rendering the scene view.

## Multi-Entity Manipulation

When multiple entities are selected:
- Gizmo appears at the average position of all selected entities
- All selected entities transform together maintaining their relative positions
- Undo/redo affects all entities as a single operation

## Visual Customization

Gizmo appearance can be configured:
- **Size**: Auto-scales based on camera distance
- **Colors**: X=red, Y=green, Z=blue (standard convention)
- **Highlight**: Lighter shade when hovered
- **Thickness**: Line width for visibility

## Integration with Selection

Gizmos automatically update when selection changes:
```rust
// Selection changed - gizmo updates automatically
selection.select_entity(entity, SelectionMode::Replace);
// Gizmo now appears at selected entity's position
```

## Integration with Undo/Redo

All gizmo operations create undo commands:
```rust
// User drags gizmo
// -> TransformEditCommand created automatically
// -> Can undo with Ctrl+Z
undo_system.undo(&mut world)?; // Reverts transform
```

## Best Practices

1. **Disable during play mode**: Gizmos should only be active in edit mode
2. **Clear on empty selection**: Hide gizmos when nothing is selected
3. **Respect camera focus**: Don't process gizmo input when viewport isn't focused
4. **Cancel on mode change**: Switching gizmo mode cancels active manipulation

## Technical Details

For implementation details, see:
- [crates/praxis_editor/GIZMOS.md](../../crates/praxis_editor/GIZMOS.md) - Complete implementation documentation
- Ray-line distance calculations
- Axis constraint mathematics
- Rendering pipeline integration

## Troubleshooting

### Gizmos Not Appearing
- Verify `GizmoSystem` resource exists in world
- Check that entities are selected
- Ensure gizmos are enabled: `gizmo_system.set_enabled(true)`
- Verify entities have `Transform` component

### Can't Drag Gizmo
- Check if viewport has focus
- Verify mouse input is being processed
- Ensure gizmo interaction hasn't been disabled
- Check for conflicting input handlers (camera controls, etc.)

### Gizmo in Wrong Position
- Verify entity transforms are up-to-date
- Check transform propagation system is running
- For multi-entity selection, position is averaged

## See Also

- [Selection System](selection-system.md) - Entity selection required for gizmos
- [Undo/Redo System](undo-redo.md) - Command history for gizmo operations
- [Scene View Panel](panels.md#scene-view) - Viewport where gizmos appear
- [Editor Camera](editor-camera.md) - Camera controls in the viewport
