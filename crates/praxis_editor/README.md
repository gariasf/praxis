# Praxis Editor

Editor system with dockable panels, selection, undo/redo, and gizmos for the Praxis game engine.

## Overview

Comprehensive development environment with scene editing, component inspection, and powerful command system.

**Key Features:**
- Dockable panels (hierarchy, inspector, console, assets)
- Multi-entity selection with raycast picking
- Full undo/redo system (100 command history)
- Transform gizmos (translate, rotate, scale)
- Dirty state tracking for unsaved changes
- Console with Lua REPL integration

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

## Undo/Redo Commands

```rust
use praxis_editor::{CommandHistory, TransformEditCommand};

let mut history = CommandHistory::new();

let command = TransformEditCommand::new(
    entity,
    old_transform,
    new_transform,
);
history.execute(&mut world, Box::new(command))?;

history.undo(&mut world)?;
history.redo(&mut world)?;
```

**Available Commands:**
- `TransformEditCommand`
- `CreateEntityCommand`
- `DeleteEntityCommand`
- `AddComponentCommand`
- `RemoveComponentCommand`
- `SetParentCommand`
- `CompositeCommand`

## Selection System

```rust
use praxis_editor::{SelectionSystem, Selectable, Selected};

world.insert_resource(SelectionSystem::new());

// Mark entities as selectable
world.spawn((
    Transform::default(),
    Selectable,
));

// Selected entities have Selected component
```

## Documentation

**Comprehensive Guides:**
- [Editor Overview](../../docs/editor/README.md)
- [Selection System](../../docs/editor/selection-system.md)
- [Undo/Redo System](../../docs/editor/undo-redo.md)
- [Inspector Panel](../../docs/editor/inspector.md)
- [Hierarchy Panel](../../docs/editor/hierarchy-panel.md)

**Crate Documentation:**
- [Undo/Redo System Details](UNDO_REDO_SYSTEM.md)
- [Quick Start Guide](QUICK_START_UNDO_REDO.md)
- [Selection System Details](SELECTION_SYSTEM.md)

**Learning Path:**
- [Editor Learning Path](../../docs/learning-paths/editor.md)

## Examples

```bash
cargo run --example editor_demo
cargo run --example selection_demo
cargo run --example undo_redo_system_demo
cargo run --example command_system_demo
```

## Dependencies

- `egui` 0.29: Immediate mode GUI
- `egui_dock`: Dockable panels
- `bevy_ecs` 0.14: ECS integration
- `ron` 0.8: Command serialization
