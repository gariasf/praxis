# Praxis Editor

The editor system for the Praxis game engine, providing a comprehensive development environment with dockable panels, asset management, and powerful editing tools.

## Features

### Core Editor

- **Dockable Panel System**: Flexible UI layout using `egui_dock`
- **Scene View**: 3D viewport for visualizing and interacting with scenes
- **Hierarchy Panel**: Tree view of scene entities
- **Inspector Panel**: Component editing for selected entities
- **Console Panel**: Log output and command execution
- **Assets Panel**: Project asset browser with drag-and-drop

### Selection System

- Multi-entity selection with add/remove/toggle modes
- Click-to-select with raycast picking
- Marquee (box) selection
- Keyboard shortcuts (Ctrl+A, Ctrl+D)
- Selection change events

See `SELECTION_SYSTEM.md` for detailed documentation.

### Command System

Comprehensive undo/redo system with serialization support:

- **EditorCommand Trait**: Base interface for all commands
- **CommandHistory**: Manages undo/redo stacks
- **Concrete Commands**:
  - `TransformEditCommand`: Edit entity transforms
  - `CreateEntityCommand`: Create new entities
  - `DeleteEntityCommand`: Delete entities
  - `AddComponentCommand`: Add components
  - `RemoveComponentCommand`: Remove components
  - `SetParentCommand`: Change entity hierarchy
  - `CompositeCommand`: Group multiple operations

**Key Features**:
- Full execute/undo/redo support
- RON serialization for save/load
- Command batching with composites
- Type-safe command implementations
- History management with size limits

See `COMMAND_SYSTEM.md` for detailed documentation.

### Gizmo System

Interactive 3D gizmos for transform manipulation:

- Translation, rotation, and scale gizmos
- Local and world space modes
- Visual feedback and snapping
- Undo/redo integration

### Drag-and-Drop

Asset drag-and-drop system for placing assets in the scene:

- Drag models, textures, and other assets
- Visual feedback during drag
- Integration with scene view

## Usage

```rust
use praxis_editor::{EditorState, EditorMode, UndoRedoSystem};
use praxis_ecs::World;

// Initialize the editor
praxis_editor::init().expect("Failed to initialize editor");

// Create editor state
let mut editor = EditorState::new();

// Set up command system
let mut world = World::new();
world.insert_resource(UndoRedoSystem::new());

// Toggle between edit and play modes
editor.set_mode(EditorMode::Play);

// Render editor UI (called every frame)
// editor.ui(&egui_context);
```

## Command System Example

```rust
use praxis_editor::{CommandHistory, TransformEditCommand};
use praxis_ecs::{World, Transform};

let mut world = World::new();
let mut history = CommandHistory::new();

// Create entity and edit transform
let entity = world.spawn(Transform::default()).id();
let command = TransformEditCommand::new(
    entity,
    Transform::default(),
    Transform::from_xyz(10.0, 5.0, 0.0)
);

// Execute command
history.execute(&mut world, Box::new(command)).unwrap();

// Undo
history.undo(&mut world).unwrap();

// Redo
history.redo(&mut world).unwrap();

// Serialize history
let ron = history.to_ron().unwrap();
std::fs::write("history.ron", ron).unwrap();
```

## Examples

Run the command system demo:

```bash
cargo run --example command_system_demo
```

Run the selection system demo:

```bash
cargo run --example selection_demo
```

## Architecture

### Panel System

The editor uses `egui_dock` for flexible panel management:

```rust
pub trait EditorPanel {
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut egui::Ui);
}
```

Panels can be:
- Dragged and rearranged
- Split horizontally or vertically
- Tabbed together
- Closed and reopened

### Selection System

Entity selection is managed through ECS:

- `Selectable` component: Marks entities as selectable
- `Selected` component: Marks currently selected entities
- `SelectionSystem` resource: Manages selection state
- `SelectionEvent`: Events fired on selection changes

### Command System

Commands follow the command pattern:

1. **Execute**: Apply changes to the world
2. **Undo**: Revert changes
3. **Redo**: Reapply changes (defaults to execute)

All commands are serializable via RON format for:
- Session recovery
- Replay functionality
- Collaboration tools

## Integration

### With ECS World

```rust
use praxis_editor::{SelectionSystem, UndoRedoSystem};
use praxis_ecs::World;

let mut world = World::new();

// Add editor systems
world.insert_resource(SelectionSystem::new());
world.insert_resource(UndoRedoSystem::new());
```

### With Input System

```rust
use praxis_editor::handle_selection_input_system;
use praxis_ecs::Schedule;

let mut schedule = Schedule::default();
schedule.add_systems(handle_selection_input_system);
```

### With Rendering

```rust
use praxis_editor::update_selection_system;

// Update selection state each frame
schedule.add_systems(update_selection_system);
```

## Documentation

- [Command System](COMMAND_SYSTEM.md) - Detailed command pattern documentation
- [Selection System](SELECTION_SYSTEM.md) - Selection system documentation

## Dependencies

- `bevy_ecs`: Entity-Component-System
- `egui`: Immediate mode GUI
- `egui_dock`: Dockable panels
- `serde`: Serialization
- `ron`: Rusty Object Notation

## License

MIT
