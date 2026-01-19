# Editor

Documentation for the Praxis editor system (`praxis_editor` crate).

## Contents

### Overview
- **[Editor Overview](editor-overview.md)** - Architecture, panels, and editor components

### Core Systems
- [Selection System](selection-system.md) - Multi-entity selection, raycast picking, marquee selection
- [Undo/Redo](undo-redo.md) - Command history and state management
- [Editor Camera](editor-camera.md) - Orbit camera controls and focus-on-selection
- [Gizmos](gizmos.md) - Transform manipulation tools

### Panels
- [Hierarchy Panel](hierarchy-panel.md) - Entity tree with drag-and-drop reparenting
- [Inspector Panel](inspector.md) - Component editing and properties
- [Asset Browser](asset-browser.md) - Asset management with thumbnails and drag-and-drop
- [Scene View](panels.md#scene-view) - 3D viewport with grid and camera controls
- [Console](panels.md#console) - Debug output and logging

### Interface
- [Menu Bar](menu-bar.md) - File, Edit, Entity, View, Help menus with shortcuts
- [Panels Overview](panels.md) - Dockable panel system and layout

## Overview

The Praxis editor provides a comprehensive development environment built on `egui` with dockable panels via `egui_dock`.

### Key Features

- **Dockable Panels**: Flexible UI layout with drag-and-drop
- **Scene Editing**: Create, modify, and delete entities
- **Selection System**: Click, marquee, and keyboard shortcuts
- **Undo/Redo**: Full command history with Ctrl+Z/Ctrl+Y
- **Transform Gizmos**: Visual 3D manipulation tools
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

## Architecture

The editor is organized into several key subsystems:

- **EditorState**: Root coordinator managing all panels and modes
- **Panels**: Modular UI components (hierarchy, inspector, scene view, etc.)
- **Systems**: ECS systems for selection, gizmos, camera control
- **Commands**: Undoable operations via command pattern

See [Editor Overview](editor-overview.md) for detailed architecture.

## Documentation Structure

The editor documentation is organized into two complementary sets:

### User Guides (docs/editor/)
High-level guides focused on **using** the editor:
- Architecture overview and design philosophy
- Usage patterns and best practices
- Keyboard shortcuts and controls
- Troubleshooting and common issues

### Technical Documentation (crates/praxis_editor/)
Implementation-focused documentation for **extending** the editor:
- Implementation details and algorithms
- API reference and method signatures
- Integration patterns and system internals
- Advanced customization

## Related

- [praxis_editor crate](../../crates/praxis_editor/README.md) - Crate-level documentation and API reference
- **Technical Documentation** in `crates/praxis_editor/`:
  - [SELECTION_SYSTEM.md](../../crates/praxis_editor/SELECTION_SYSTEM.md) - Selection implementation
  - [UNDO_REDO_SYSTEM.md](../../crates/praxis_editor/UNDO_REDO_SYSTEM.md) - Command pattern implementation
  - [EDITOR_CAMERA.md](../../crates/praxis_editor/EDITOR_CAMERA.md) - Camera controller implementation
  - [GIZMOS.md](../../crates/praxis_editor/GIZMOS.md) - Gizmo system implementation
  - [MENU_BAR.md](../../crates/praxis_editor/MENU_BAR.md) - Menu system implementation
  - [VIEWPORT_PANEL.md](../../crates/praxis_editor/VIEWPORT_PANEL.md) - Viewport rendering
  - [COMMAND_SYSTEM.md](../../crates/praxis_editor/COMMAND_SYSTEM.md) - Command architecture
  - [PLAY_MODE_SYSTEM.md](../../crates/praxis_editor/PLAY_MODE_SYSTEM.md) - Play mode system
  - [TOOLBAR_SYSTEM.md](../../crates/praxis_editor/TOOLBAR_SYSTEM.md) - Toolbar system
- **Examples**:
  - `examples/editor_demo.rs` - Full editor demonstration
  - `examples/selection_demo.rs` - Selection system demo
  - `examples/command_system_demo.rs` - Command pattern demo
  - `examples/undo_redo_system_demo.rs` - Undo/redo demo
  - `examples/editor_camera_demo.rs` - Camera controls demo
