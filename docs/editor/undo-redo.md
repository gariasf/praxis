# Undo/Redo System

Comprehensive undo/redo functionality for the Praxis editor with command history, dirty state tracking, and keyboard shortcuts.

## Overview

The Undo/Redo System uses the command pattern to provide full undo/redo capabilities for all editor operations. It maintains a history stack, tracks unsaved changes, and integrates with the menu system and keyboard shortcuts.

## Features

- **Command History**: Up to 100 commands in undo/redo stacks
- **Dirty State Tracking**: Automatically tracks unsaved changes
- **Keyboard Shortcuts**: Ctrl+Z (undo), Ctrl+Y (redo), Ctrl+Shift+Z (redo alternate)
- **Menu Integration**: Visual feedback in editor menu with command descriptions
- **Serialization**: Save/load history to RON format
- **Composite Commands**: Group multiple operations into single undoable action

## Architecture

### UndoRedoSystem (Resource)

ECS resource managing command history and dirty state:

```rust
pub struct UndoRedoSystem {
    pub history: CommandHistory,
    dirty: bool,              // Tracks unsaved changes
    saved_undo_count: usize,  // Undo count at last save
}
```

**Key Methods**:
- `execute_command(world, command)` - Execute and add to history
- `undo(world)` - Undo last command
- `redo(world)` - Redo last undone command
- `can_undo()` / `can_redo()` - Check if operation is available
- `undo_description()` / `redo_description()` - Get command descriptions
- `is_dirty()` - Check for unsaved changes
- `mark_saved()` - Mark current state as saved

### EditorCommand Trait

Base interface for undoable operations:

```rust
pub trait EditorCommand {
    fn execute(&mut self, world: &mut World) -> Result<()>;
    fn undo(&mut self, world: &mut World) -> Result<()>;
    fn redo(&mut self, world: &mut World) -> Result<()>;
    fn description(&self) -> String;
}
```

## Usage

### Basic Setup

```rust
use praxis_editor::UndoRedoSystem;

// Create and insert as resource
let undo_system = UndoRedoSystem::new();
world.insert_resource(undo_system);
```

### Executing Commands

```rust
use praxis_editor::TransformEditCommand;

let command = Box::new(TransformEditCommand::new(
    entity,
    old_transform,
    new_transform,
));

// Execute command (adds to undo stack)
undo_system.execute_command(&mut world, command)?;
```

### Undo/Redo Operations

```rust
// Check and perform undo
if undo_system.can_undo() {
    println!("Will undo: {}", undo_system.undo_description().unwrap());
    undo_system.undo(&mut world)?;
}

// Check and perform redo
if undo_system.can_redo() {
    println!("Will redo: {}", undo_system.redo_description().unwrap());
    undo_system.redo(&mut world)?;
}
```

### Dirty State Tracking

```rust
// Check for unsaved changes
if undo_system.is_dirty() {
    show_unsaved_warning();
}

// After saving to disk
save_scene(&world)?;
undo_system.mark_saved();
assert!(!undo_system.is_dirty());

// Becomes dirty after new command
undo_system.execute_command(&mut world, command)?;
assert!(undo_system.is_dirty());

// Undo back to saved state - becomes clean
undo_system.undo(&mut world)?;
assert!(!undo_system.is_dirty());
```

## Available Commands

### Transform Commands
```rust
use praxis_editor::TransformEditCommand;

TransformEditCommand::new(entity, old_transform, new_transform);
```

### Entity Commands
```rust
use praxis_editor::{CreateEntityCommand, DeleteEntityCommand};

CreateEntityCommand::with_transform(Transform::default());
DeleteEntityCommand::from_world(entity, &world)?;
```

### Component Commands
```rust
use praxis_editor::{AddComponentCommand, RemoveComponentCommand, ComponentData};

AddComponentCommand::new(entity, ComponentData::Name("Player".into()));
RemoveComponentCommand::new(entity, ComponentData::Name(old_name));
```

### Hierarchy Commands
```rust
use praxis_editor::SetParentCommand;

SetParentCommand::from_world(child, Some(parent), &world)?;
```

### Composite Commands

Group multiple operations into one undoable action:

```rust
use praxis_editor::{CompositeCommand, SerializableCommand};

let mut composite = CompositeCommand::new("Create Scene".into());
composite.add_command(SerializableCommand::CreateEntity(cmd1));
composite.add_command(SerializableCommand::CreateEntity(cmd2));

// Undo/redo as single operation
undo_system.execute_command(&mut world, Box::new(composite))?;
```

## Keyboard Shortcuts

Built-in keyboard shortcuts (via `handle_command_shortcuts` system):

| Shortcut | Action |
|----------|--------|
| **Ctrl+Z** | Undo last command |
| **Ctrl+Y** | Redo last undone command |
| **Ctrl+Shift+Z** | Redo (alternative) |

Add to your schedule:
```rust
use praxis_editor::handle_command_shortcuts;

schedule.add_systems(handle_command_shortcuts);
```

## Menu Integration

The menu bar automatically integrates with undo/redo:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// Render editor with undo/redo integration
editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
```

**Menu Features**:
- Command descriptions in menu items (e.g., "Undo: Move Entity")
- Enabled/disabled state based on availability
- Dirty state indicator (asterisk on "Save Scene *")
- Status bar shows "● Unsaved" when dirty

## Dirty State Behavior

The dirty state intelligently tracks unsaved changes:

1. **Initially Clean**: New system starts with `dirty = false`
2. **Becomes Dirty**: When executing commands, or redoing
3. **Becomes Clean**: When calling `mark_saved()`, or undoing back to saved state
4. **Saved State Tracking**: Records undo count at save, compares current count

### Example Flow

```rust
let mut system = UndoRedoSystem::new();
assert!(!system.is_dirty());  // Clean

system.execute_command(&mut world, cmd1)?;
assert!(system.is_dirty());  // Dirty

system.mark_saved();
assert!(!system.is_dirty());  // Clean after save

system.execute_command(&mut world, cmd2)?;
assert!(system.is_dirty());  // Dirty

system.undo(&mut world)?;
assert!(!system.is_dirty());  // Clean - back at saved state
```

## History Limit

The system maintains a maximum of **100 commands**:
- When executing the 101st command, the oldest is automatically removed
- Prevents unbounded memory growth
- Sufficient for typical editing sessions

## Serialization

Save and load command history:

```rust
// Save history to RON
let ron = undo_system.to_ron()?;
std::fs::write("history.ron", ron)?;

// Load history from RON
let ron = std::fs::read_to_string("history.ron")?;
undo_system.from_ron(&ron)?;
```

**Note**: Loading marks the state as dirty.

## Best Practices

1. **Always use `execute_command`**: Don't execute commands directly
   ```rust
   // Wrong: command.execute(&mut world)
   // Right: undo_system.execute_command(&mut world, command)
   ```

2. **Capture state before deletion**: For proper undo
   ```rust
   let cmd = DeleteEntityCommand::from_world(entity, &world)?;
   ```

3. **Group related operations**: Use `CompositeCommand`
   ```rust
   let mut composite = CompositeCommand::new("Create Player".into());
   composite.add_command(create_cmd);
   composite.add_command(name_cmd);
   ```

4. **Mark saved appropriately**: After successful save operations
   ```rust
   save_scene_to_file(&world)?;
   undo_system.mark_saved();
   ```

5. **Check dirty on exit**: Warn users about unsaved changes
   ```rust
   if undo_system.is_dirty() {
       show_unsaved_changes_dialog();
   }
   ```

## Performance

- **Memory**: Bounded by 100 command limit
- **Execution**: O(1) for single commands, O(n) for composite with n operations
- **Undo/Redo**: O(1) stack operations

## Limitations

1. **Entity ID Stability**: Entity IDs not stable across sessions (serialized commands may not work after restart)
2. **Component Support**: Only Transform, Name, Parent components supported out-of-box
3. **History Size**: Limited to 100 entries
4. **No Branching**: Executing new command clears redo stack (no undo tree)

## Examples

See these examples for demonstrations:
- `examples/undo_redo_system_demo.rs` - Full undo/redo demonstration
- `examples/command_system_demo.rs` - Command pattern examples
- `examples/editor_demo.rs` - Integration with editor

## Technical Details

For implementation details, see:
- [crates/praxis_editor/UNDO_REDO_SYSTEM.md](../../crates/praxis_editor/UNDO_REDO_SYSTEM.md) - Complete implementation documentation
- Command serialization format
- History management algorithms
- Dirty state tracking logic

## See Also

- [Menu Bar](menu-bar.md) - Menu integration with undo/redo
- [Selection System](selection-system.md) - Selection operations are undoable
- [Gizmos](gizmos.md) - Transform gizmo operations are undoable
- [Hierarchy Panel](hierarchy-panel.md) - Entity operations are undoable
