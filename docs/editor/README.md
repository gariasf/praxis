# Editor

Documentation for the Praxis editor system (`praxis_editor` crate).

## Contents

- [Panels](panels.md) - Hierarchy, Inspector, Assets, Console, Viewport
- [Selection](selection.md) - Multi-entity selection, raycast picking, marquee selection
- [Undo/Redo](undo-redo.md) - Command history and state management
- [Gizmos](gizmos.md) - Transform manipulation tools
- [Camera](camera.md) - Editor camera controller

## Overview

The Praxis editor provides a comprehensive development environment built on `egui` with dockable panels via `egui_dock`.

### Key Features

- **Dockable Panels**: Flexible UI layout with drag-and-drop
- **Scene Editing**: Create, modify, and delete entities
- **Selection System**: Click, marquee, and keyboard shortcuts
- **Undo/Redo**: Full command history with Ctrl+Z/Ctrl+Y
- **Play Mode**: Toggle between edit and play modes

### Quick Start

```rust
use praxis_editor::{EditorState, UndoRedoSystem};
use praxis_ecs::World;

let mut world = World::new();
world.insert_resource(UndoRedoSystem::new());

let mut editor = EditorState::new();
// In your render loop:
editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
```

## Related

- [praxis_editor crate](../../crates/praxis_editor/README.md) - Crate-level documentation
- [Examples](../../examples/README.md) - editor_demo, selection_demo, undo_redo_system_demo
