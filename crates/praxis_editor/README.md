# Praxis Editor

Editor system with dockable panels, selection, undo/redo, and gizmos for the Praxis game engine.

## Overview

Comprehensive development environment with scene editing, component inspection, and powerful command system.

**Key Features:**
- Dockable panels (hierarchy, inspector, console, assets, scene view)
- Multi-entity selection with raycast and marquee picking
- Full undo/redo system (100 command history)
- Transform gizmos (translate, rotate, scale)
- Dirty state tracking for unsaved changes
- Play mode with scene snapshot/restore
- Console with log filtering and search

## Quick Start

```rust
use praxis_editor::{EditorState, UndoRedoSystem};

// Initialize
praxis_editor::init()?;
let mut editor = EditorState::new();
world.insert_resource(UndoRedoSystem::new());

// In game loop
let mut undo_system = world.remove_resource::<UndoRedoSystem>().unwrap();
editor.ui(&egui_ctx, Some(&mut undo_system), Some(&mut world));
world.insert_resource(undo_system);
```

## Core Systems

### Selection System

```rust
use praxis_editor::{SelectionSystem, Selectable, Selected};

world.insert_resource(SelectionSystem::new());

// Mark entities as selectable
world.spawn((Transform::default(), Selectable));

// Selected entities have Selected component
fn highlight_selected(query: Query<Entity, With<Selected>>) {
    // ...
}
```

**See**: [SELECTION_SYSTEM.md](SELECTION_SYSTEM.md) for implementation details.

### Undo/Redo Commands

```rust
use praxis_editor::{CommandHistory, TransformEditCommand};

let mut history = CommandHistory::new();

let command = TransformEditCommand::new(entity, old_transform, new_transform);
history.execute(&mut world, Box::new(command))?;

history.undo(&mut world)?;
history.redo(&mut world)?;
```

**Available Commands:**
- `TransformEditCommand` - Transform changes
- `CreateEntityCommand` / `DeleteEntityCommand` - Entity lifecycle
- `AddComponentCommand` / `RemoveComponentCommand` - Component management
- `SetParentCommand` - Hierarchy changes
- `CompositeCommand` - Grouped operations

**See**: [UNDO_REDO_SYSTEM.md](UNDO_REDO_SYSTEM.md) for implementation details.

### Editor Camera

```rust
use praxis_editor::{EditorCameraController, EditorCamera};

world.insert_resource(EditorCameraController::new());

// Spawn editor camera
world.spawn((
    PerspectiveCameraBundle::new(Vec3::new(0.0, 5.0, 10.0), fov, aspect),
    EditorCamera,
));
```

**See**: [EDITOR_CAMERA.md](EDITOR_CAMERA.md) for implementation details.

### Transform Gizmos

```rust
use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};

world.insert_resource(GizmoSystem::new());

let mut gizmo = world.resource_mut::<GizmoSystem>();
gizmo.set_mode(GizmoMode::Translate);
gizmo.set_space(GizmoSpace::World);
```

**See**: [GIZMOS.md](GIZMOS.md) for implementation details.

### Menu Bar

```rust
use praxis_editor::menu_bar::{render_menu_bar, check_keyboard_shortcuts, handle_menu_action};

let actions = render_menu_bar(&ctx, &mut menu_state, Some(&undo_system));
// Handle actions...
```

**See**: [MENU_BAR.md](MENU_BAR.md) for implementation details.

## Documentation

### User Guides (docs/editor/)

High-level guides for using the editor:
- [Editor Overview](../../docs/editor/editor-overview.md) - Architecture and features
- [Selection System Guide](../../docs/editor/selection-system.md) - Using selection
- [Undo/Redo Guide](../../docs/editor/undo-redo.md) - Using undo/redo
- [Inspector Panel](../../docs/editor/inspector.md) - Component editing
- [Hierarchy Panel](../../docs/editor/hierarchy-panel.md) - Entity tree
- [Asset Browser](../../docs/editor/asset-browser.md) - Asset management
- [Editor Camera](../../docs/editor/editor-camera.md) - Camera controls
- [Gizmos](../../docs/editor/gizmos.md) - Transform tools
- [Menu Bar](../../docs/editor/menu-bar.md) - Menu system
- [Panels](../../docs/editor/panels.md) - Panel overview

### Technical Documentation (crates/praxis_editor/)

Implementation details and API reference:
- [SELECTION_SYSTEM.md](SELECTION_SYSTEM.md) - Selection implementation
- [UNDO_REDO_SYSTEM.md](UNDO_REDO_SYSTEM.md) - Command pattern implementation
- [EDITOR_CAMERA.md](EDITOR_CAMERA.md) - Camera controller implementation
- [GIZMOS.md](GIZMOS.md) - Gizmo system implementation
- [MENU_BAR.md](MENU_BAR.md) - Menu system implementation
- [VIEWPORT_PANEL.md](VIEWPORT_PANEL.md) - Viewport rendering
- [COMMAND_SYSTEM.md](COMMAND_SYSTEM.md) - Command architecture
- [COMMANDS_OVERVIEW.md](COMMANDS_OVERVIEW.md) - Command catalog
- [PLAY_MODE_SYSTEM.md](PLAY_MODE_SYSTEM.md) - Play mode implementation
- [TOOLBAR_SYSTEM.md](TOOLBAR_SYSTEM.md) - Toolbar system

## Examples

```bash
# Full editor demonstration
cargo run --example editor_demo

# System-specific examples
cargo run --example selection_demo
cargo run --example undo_redo_system_demo
cargo run --example command_system_demo
cargo run --example editor_camera_demo
```

## Architecture

The editor is organized into several subsystems:

- **EditorState**: Central coordinator managing panels, modes, and state
- **Panels**: Dockable UI components (hierarchy, inspector, console, etc.)
- **Systems**: ECS systems for selection, gizmos, camera
- **Commands**: Undoable operations via command pattern
- **Menu/Toolbar**: Standard UI controls with shortcuts

All panels implement the `EditorPanel` trait for consistent integration with the docking system.

## Dependencies

- `egui` 0.29 - Immediate mode GUI
- `egui_dock` - Dockable panels
- `bevy_ecs` 0.14 - ECS integration
- `ron` 0.8 - Command serialization
- `praxis_graphics` - Rendering integration
- `praxis_scene` - Transform hierarchy
- `praxis_input` - Input handling

## Features

- **Default**: Core editor functionality
- `terrain` - Terrain editing panel (requires terrain features in other crates)

## Related Crates

- `praxis_gui` - Base GUI components used by editor panels
- `praxis_graphics` - Rendering backend for viewports
- `praxis_scene` - Scene graph and transform hierarchy
- `praxis_input` - Input handling for camera and selection

## License

See `LICENSE` file in repository root.
