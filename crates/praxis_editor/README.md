# praxis_editor

Editor system for the Praxis game engine, providing a comprehensive interface for creating and managing game content.

## Features

- **Dockable Panel System**: Flexible UI layout using `egui_dock`
- **Edit/Play Modes**: Switch between editing and runtime testing
- **Scene View**: 3D viewport for visualizing and interacting with the scene
- **Hierarchy Panel**: Tree view of scene entities
- **Inspector Panel**: Component editing for selected entities
- **Console Panel**: Log output and command execution
- **Assets Panel**: Project asset browser

## Architecture

The editor is organized around several key components:

### EditorState

The root coordinator that manages all editor panels and modes. It provides:
- Mode switching (Edit/Play)
- Panel layout management via `egui_dock`
- Access to individual panels
- Menu bar rendering

### EditorMode

An enum defining the current editor mode:
- `Edit`: Scene editing mode with paused simulation
- `Play`: Runtime mode with active simulation

### Panels

Modular UI components implementing the `EditorPanel` trait:
- **SceneViewPanel**: 3D scene viewport
- **HierarchyPanel**: Entity hierarchy tree
- **InspectorPanel**: Component inspector
- **ConsolePanel**: Console output and commands
- **AssetsPanel**: Asset browser

## Usage

```rust
use praxis_editor::{EditorState, EditorMode};

// Initialize the editor system
praxis_editor::init()?;

// Create editor state
let mut editor = EditorState::new();

// Switch to play mode
editor.set_mode(EditorMode::Play);

// Render editor UI (called every frame)
editor.ui(&egui_context);

// Access individual panels
editor.console_panel_mut().add_log("Hello from editor!".to_string());
```

## Dependencies

- `egui`: Immediate mode GUI framework
- `egui_dock`: Docking system for egui
- `praxis_gui`: Base GUI utilities
- `praxis_ecs`: Entity-component system
- `praxis_scene`: Scene management
- `praxis_assets`: Asset loading
- `praxis_input`: Input handling

## Default Layout

The editor creates a default layout on initialization:
- Left side: Hierarchy panel (top), Assets panel (bottom)
- Center: Scene view panel
- Right side: Inspector panel (top), Console panel (bottom)

Panels can be freely rearranged, split, and tabbed through drag-and-drop.
