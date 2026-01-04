# Editor Panels

Dockable panel system using `egui_dock` for flexible editor layouts.

## Built-in Panels

### Hierarchy Panel
Tree view of all scene entities. Shows parent-child relationships.

### Inspector Panel
Component editing for selected entities. Displays and modifies properties.

### Console Panel
Log output and command execution.

### Assets Panel
Project asset browser with drag-and-drop support.

### Scene View
3D viewport for visualizing and interacting with scenes.

## Panel Trait

```rust
pub trait EditorPanel {
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut egui::Ui);
}
```

## Layout Features

Panels can be:
- Dragged and rearranged
- Split horizontally or vertically
- Tabbed together
- Closed and reopened

## Usage

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// In your game loop
editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
```

## See Also

- [Selection](selection.md) - Selection in hierarchy
- [crates/praxis_gui/HIERARCHY_PANEL.md](../../crates/praxis_gui/HIERARCHY_PANEL.md) - Hierarchy details
