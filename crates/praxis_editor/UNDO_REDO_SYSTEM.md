# UndoRedoSystem Documentation

The UndoRedoSystem provides comprehensive undo/redo functionality for the Praxis editor with command history management, dirty state tracking, keyboard shortcuts, and menu bar integration.

## Features

- **Command History Stack**: Maintains up to 100 commands in history
- **Undo/Redo Operations**: Full support for undoing and redoing editor operations
- **Dirty State Tracking**: Automatically tracks whether there are unsaved changes
- **Keyboard Shortcuts**: Ctrl+Z for undo, Ctrl+Y for redo
- **Menu Bar Integration**: Visual feedback and controls in the editor menu
- **Serialization**: Save/load command history to RON format

## Architecture

### Core Components

#### `UndoRedoSystem`
ECS resource that wraps `CommandHistory` and adds dirty state tracking:
```rust
pub struct UndoRedoSystem {
    pub history: CommandHistory,
    dirty: bool,              // Tracks unsaved changes
    saved_undo_count: usize,  // Undo count at last save
}
```

#### `CommandHistory`
Manages the undo/redo stacks:
```rust
pub struct CommandHistory {
    undo_stack: VecDeque<Box<dyn EditorCommand>>,
    redo_stack: VecDeque<Box<dyn EditorCommand>>,
    max_history_size: usize,  // Set to 100
}
```

#### `EditorCommand` Trait
Base interface for all undoable operations:
```rust
pub trait EditorCommand {
    fn execute(&mut self, world: &mut World) -> Result<()>;
    fn undo(&mut self, world: &mut World) -> Result<()>;
    fn redo(&mut self, world: &mut World) -> Result<()>;
    fn description(&self) -> String;
    fn to_ron(&self) -> Result<String>;
    fn type_id(&self) -> &'static str;
}
```

## Usage

### Basic Setup

```rust
use praxis_editor::UndoRedoSystem;
use praxis_ecs::World;

// Create the undo/redo system
let mut undo_system = UndoRedoSystem::new();
let mut world = World::new();

// Insert as ECS resource
world.insert_resource(undo_system);
```

### Executing Commands

```rust
use praxis_editor::{UndoRedoSystem, TransformEditCommand};
use praxis_ecs::Transform;

let mut undo_system = UndoRedoSystem::new();
let entity = world.spawn(Transform::default()).id();

// Create and execute a command
let command = Box::new(TransformEditCommand::new(
    entity,
    Transform::default(),
    Transform::from_xyz(10.0, 5.0, 3.0),
));

undo_system.execute_command(&mut world, command).unwrap();
// System is now dirty (has unsaved changes)
```

### Undo/Redo Operations

```rust
// Check if undo is available
if undo_system.can_undo() {
    // Get description of next undo
    if let Some(desc) = undo_system.undo_description() {
        println!("Will undo: {}", desc);
    }
    
    // Perform undo
    undo_system.undo(&mut world).unwrap();
}

// Check if redo is available
if undo_system.can_redo() {
    // Get description of next redo
    if let Some(desc) = undo_system.redo_description() {
        println!("Will redo: {}", desc);
    }
    
    // Perform redo
    undo_system.redo(&mut world).unwrap();
}
```

### Dirty State Tracking

```rust
// Check if there are unsaved changes
if undo_system.is_dirty() {
    println!("There are unsaved changes!");
}

// Mark as saved (e.g., after saving to disk)
undo_system.mark_saved();
assert!(!undo_system.is_dirty());

// Execute another command - becomes dirty again
undo_system.execute_command(&mut world, command).unwrap();
assert!(undo_system.is_dirty());

// Undo back to saved state - becomes clean
undo_system.undo(&mut world).unwrap();
assert!(!undo_system.is_dirty());
```

### Keyboard Shortcuts

The system integrates with keyboard input via the `handle_command_shortcuts` function:

```rust
use praxis_editor::handle_command_shortcuts;
use praxis_ecs::Schedule;

// Add to your ECS schedule
let mut schedule = Schedule::default();
schedule.add_systems(handle_command_shortcuts);

// The system handles:
// - Ctrl+Z: Undo
// - Ctrl+Y: Redo
// - Ctrl+Shift+Z: Redo (alternative)
```

### Menu Bar Integration

The editor state provides full menu bar integration:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// Render with undo/redo integration
editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));

// Menu features:
// - Edit > Undo: Shows "Undo: <description> (Ctrl+Z)", disabled when no undo available
// - Edit > Redo: Shows "Redo: <description> (Ctrl+Y)", disabled when no redo available
// - Edit > History: Shows undo/redo stack counts
// - File > Save Scene: Shows asterisk (*) when dirty
// - Status bar: Shows "● Unsaved" indicator when dirty
```

## Command History Limit

The system maintains a maximum of **100 commands** in the history:

```rust
const MAX_HISTORY_SIZE: usize = 100;

// When executing the 101st command:
// - Command is added to undo stack
// - Oldest command (1st) is removed from the stack
// - Only most recent 100 commands are kept
```

This prevents unbounded memory growth while providing sufficient history for typical editing sessions.

## Dirty State Behavior

The dirty state tracks unsaved changes intelligently:

1. **Initially Clean**: New `UndoRedoSystem` starts with `dirty = false`
2. **Becomes Dirty**: 
   - When executing any command
   - When undoing away from saved state
   - When redoing
3. **Becomes Clean**:
   - When calling `mark_saved()` (typically after saving)
   - When undoing back to the exact saved state
4. **Saved State Tracking**:
   - System records `saved_undo_count` when `mark_saved()` is called
   - Compares current `undo_count` to `saved_undo_count` to determine if state matches saved version

### Example Dirty State Flow

```rust
let mut system = UndoRedoSystem::new();
assert!(!system.is_dirty());  // Clean initially

// Execute command 1
system.execute_command(&mut world, cmd1).unwrap();
assert!(system.is_dirty());  // Dirty after command

// Mark as saved
system.mark_saved();
assert!(!system.is_dirty());  // Clean after save

// Execute command 2
system.execute_command(&mut world, cmd2).unwrap();
assert!(system.is_dirty());  // Dirty after new command

// Undo command 2 (back to saved state)
system.undo(&mut world).unwrap();
assert!(!system.is_dirty());  // Clean - back at saved state

// Redo command 2
system.redo(&mut world).unwrap();
assert!(system.is_dirty());  // Dirty again
```

## Available Commands

### Transform Commands
```rust
use praxis_editor::TransformEditCommand;

let cmd = TransformEditCommand::new(entity, old_transform, new_transform);
```

### Entity Commands
```rust
use praxis_editor::{CreateEntityCommand, DeleteEntityCommand};

// Create entity
let cmd = CreateEntityCommand::with_transform(Transform::default());

// Delete entity (captures state for undo)
let cmd = DeleteEntityCommand::from_world(entity, &world).unwrap();
```

### Component Commands
```rust
use praxis_editor::{AddComponentCommand, RemoveComponentCommand, ComponentData};

// Add component
let cmd = AddComponentCommand::new(entity, ComponentData::Name("Player".to_string()));

// Remove component (stores data for undo)
let cmd = RemoveComponentCommand::new(entity, ComponentData::Name(current_name));
```

### Hierarchy Commands
```rust
use praxis_editor::SetParentCommand;

// Set parent relationship
let cmd = SetParentCommand::from_world(child, Some(parent), &world).unwrap();
```

### Composite Commands
```rust
use praxis_editor::{CompositeCommand, SerializableCommand};

let mut composite = CompositeCommand::new("Create Scene".to_string());
composite.add_command(SerializableCommand::CreateEntity(cmd1));
composite.add_command(SerializableCommand::CreateEntity(cmd2));

// Execute as single undoable operation
undo_system.execute_command(&mut world, Box::new(composite)).unwrap();
```

## Serialization

Save and load command history:

```rust
// Serialize history to RON
let ron = undo_system.to_ron().unwrap();
std::fs::write("history.ron", ron).unwrap();

// Load history from RON
let ron = std::fs::read_to_string("history.ron").unwrap();
undo_system.from_ron(&ron).unwrap();
// Note: Loading marks state as dirty
```

## Best Practices

1. **Always use `execute_command`**: Don't execute commands directly; use `UndoRedoSystem::execute_command` to ensure proper history tracking and dirty state management.

2. **Capture state before deletion**: When deleting entities or removing components, capture their state first:
   ```rust
   let cmd = DeleteEntityCommand::from_world(entity, &world)?;
   ```

3. **Group related operations**: Use `CompositeCommand` for multi-step operations that should undo/redo as a unit:
   ```rust
   let mut composite = CompositeCommand::new("Create Player".to_string());
   composite.add_command(SerializableCommand::CreateEntity(create_cmd));
   composite.add_command(SerializableCommand::AddComponent(name_cmd));
   ```

4. **Mark saved at appropriate times**: Call `mark_saved()` after successful save operations:
   ```rust
   save_scene_to_file(&world)?;
   undo_system.mark_saved();
   ```

5. **Check dirty state before exit**: Warn users about unsaved changes:
   ```rust
   if undo_system.is_dirty() {
       show_unsaved_changes_dialog();
   }
   ```

## Integration Example

Complete integration with an editor:

```rust
use praxis_editor::{EditorState, UndoRedoSystem, handle_command_shortcuts};
use praxis_ecs::{World, Schedule};
use praxis_input::InputState;

// Setup
let mut world = World::new();
let mut editor = EditorState::new();
let mut undo_system = UndoRedoSystem::new();
let mut schedule = Schedule::default();

// Add keyboard shortcut handler
schedule.add_systems(handle_command_shortcuts);

// Insert resources
world.insert_resource(InputState::default());
world.insert_resource(undo_system);

// Game loop
loop {
    // Update systems (handles keyboard shortcuts)
    schedule.run(&mut world);
    
    // Get mutable reference for UI
    let mut undo_system = world.remove_resource::<UndoRedoSystem>().unwrap();
    
    // Render editor UI with undo/redo integration
    editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
    
    // Restore resource
    world.insert_resource(undo_system);
}
```

## Testing

The system includes comprehensive tests:

```bash
cargo test -p praxis_editor --lib undo
```

Test coverage includes:
- Command execution and undo/redo
- History size limits
- Dirty state tracking
- Command serialization
- Composite commands

## Performance Considerations

- **Memory**: Each command stores enough data to undo/redo. With 100 command limit, memory usage is bounded.
- **Command Execution**: O(1) for single commands, O(n) for composite commands with n operations
- **Undo/Redo**: O(1) operation to pop from stack and execute
- **History Limit**: When limit reached, oldest command is dropped in O(1) time

## Limitations

1. **Entity ID Stability**: Serialized commands reference entities by ID. These IDs are not stable across sessions, so loaded commands may not work if entity IDs have changed.

2. **Component Types**: Only supports `Transform`, `Name`, and `Parent` components out of the box. Custom components need custom command implementations.

3. **History Size**: Limited to 100 entries. Very long edit sessions may lose oldest commands.

4. **No Branching**: Executing a new command clears the redo stack. No support for undo tree or branching history.

## Future Enhancements

Potential improvements:
- Entity ID remapping for cross-session serialization
- Configurable history size
- Command compression/deduplication
- Undo tree with branching support
- More component command types
- Batch command optimization
