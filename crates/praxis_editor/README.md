# praxis_editor

Editor tools for Praxis engine.

## Overview

Provides editor functionality including selection, undo/redo, gizmos, and transform tools.

## Features

### Selection System

- Multi-select support
- Outline rendering for selected objects
- Hierarchy-aware selection

### Undo/Redo System

- Command pattern implementation
- Serializable command history
- Grouping commands into transactions

### Transform Gizmos

- Translation, rotation, scale gizmos
- World and local space modes
- Snap to grid support

### Editor Camera

- Orbit controller
- Pan and zoom
- Focus on selection

### Inspector

- Component editing
- Property grids
- Real-time updates

## Example

```rust
use praxis_editor::{Selection, UndoRedoSystem, EditorCamera};

// Selection
let mut selection = Selection::new();
selection.select(entity);
selection.toggle(other_entity);

// Undo/Redo
let mut undo_redo = UndoRedoSystem::new();
undo_redo.execute(TranslateCommand::new(entity, new_position));
undo_redo.undo();
undo_redo.redo();

// Editor camera
let mut camera = EditorCamera::new();
camera.orbit(delta_x, delta_y);
camera.focus_on(target_position);
```

## Architecture

```
EditorSystem
    ├── Selection
    ├── UndoRedo
    ├── Gizmos
    ├── Camera
    └── Inspector
```

## Dependencies

- `egui`: GUI
- `serde`: Command serialization
- `rustc-hash`: Fast hash maps

## Usage

```toml
praxis_editor = { path = "../praxis_editor", version = "0.1.0" }

# In root Cargo.toml
[features]
editor = ["praxis_editor"]
```
