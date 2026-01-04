# Transform Gizmos

Interactive 3D tools for visually manipulating entity transforms in the editor viewport.

## Features

- **Visual 3D Gizmos**: Colored axes (X=red, Y=green, Z=blue)
- **Three Modes**: Translate, Rotate, Scale
- **Local/World Space**: Toggle coordinate systems
- **Ray-based Interaction**: Click and drag axes
- **Multi-entity Support**: Manipulate multiple entities
- **Undo/Redo Integration**: All operations are undoable

## Controls

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| W | Translate mode |
| E | Rotate mode |
| R | Scale mode |
| Tab | Cycle modes |
| Space | Toggle world/local |

### Mouse Controls

- **Click axis**: Start manipulation
- **Drag**: Transform along axis
- **Release**: Complete operation

## Usage

```rust
use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};

// Initialize
world.insert_resource(GizmoSystem::new());

// Configure
let mut gizmo = world.resource_mut::<GizmoSystem>();
gizmo.set_mode(GizmoMode::Translate);
gizmo.set_space(GizmoSpace::World);

// Update for selection
gizmo.update_gizmo_for_selection(&selected_entities);
```

## Interaction Flow

1. **Hover**: Call `update_hover()` on mouse move
2. **Start**: Call `start_interaction()` on mouse down
3. **Update**: Call `update_interaction()` during drag
4. **End**: Call `end_interaction()` on mouse up, create undo command

## World vs Local Space

**World Space**: Axes align with world coordinates. Good for grid alignment.

**Local Space**: Axes rotate with entity. Good for relative movement.

## See Also

- [Selection System](selection.md) - Entity selection
- [Undo/Redo](undo-redo.md) - Command history
- [crates/praxis_editor/GIZMOS.md](../../crates/praxis_editor/GIZMOS.md) - Full documentation
