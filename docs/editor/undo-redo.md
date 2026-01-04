# Undo/Redo System

Comprehensive undo/redo functionality for the Praxis editor with command history, dirty state tracking, and keyboard shortcuts.

## Features

- **Command History**: Up to 100 commands in history
- **Dirty State Tracking**: Tracks unsaved changes automatically
- **Keyboard Shortcuts**: Ctrl+Z (undo), Ctrl+Y (redo)
- **Menu Integration**: Visual feedback in editor menu
- **Serialization**: Save/load history to RON format

## Architecture

### UndoRedoSystem

ECS resource wrapping command history with dirty state:

```rust
pub struct UndoRedoSystem {
    pub history: CommandHistory,
    dirty: bool,              // Tracks unsaved changes
    saved_undo_count: usize,  // Undo count at last save
}
```

### CommandHistory

Manages undo/redo stacks:

```rust
pub struct CommandHistory {
    undo_stack: VecDeque<Box<dyn EditorCommand>>,
    redo_stack: VecDeque<Box<dyn EditorCommand>>,
    max_history_size: usize,  // 100
}
```

### EditorCommand Trait

Interface for undoable operations:

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
use praxis_ecs::World;

let mut undo_system = UndoRedoSystem::new();
let mut world = World::new();
world.insert_resource(undo_system);
```

### Executing Commands

```rust
use praxis_editor::{UndoRedoSystem, TransformEditCommand};
use praxis_ecs::Transform;

let command = Box::new(TransformEditCommand::new(
    entity,
    Transform::default(),
    Transform::from_xyz(10.0, 5.0, 3.0),
));

undo_system.execute_command(&mut world, command).unwrap();
```

### Undo/Redo Operations

```rust
// Check availability
if undo_system.can_undo() {
    println!("Will undo: {}", undo_system.undo_description().unwrap());
    undo_system.undo(&mut world).unwrap();
}

if undo_system.can_redo() {
    undo_system.redo(&mut world).unwrap();
}
```

### Dirty State Tracking

```rust
// Check for unsaved changes
if undo_system.is_dirty() {
    show_unsaved_warning();
}

// Mark as saved (after saving to disk)
undo_system.mark_saved();

// Becomes dirty again after new command
undo_system.execute_command(&mut world, command).unwrap();
assert!(undo_system.is_dirty());

// Undo back to saved state → becomes clean
undo_system.undo(&mut world).unwrap();
assert!(!undo_system.is_dirty());
```

### Keyboard Shortcuts

```rust
use praxis_editor::handle_command_shortcuts;
use praxis_ecs::Schedule;

let mut schedule = Schedule::default();
schedule.add_systems(handle_command_shortcuts);

// Handles: Ctrl+Z (undo), Ctrl+Y (redo), Ctrl+Shift+Z (redo alt)
```

## Available Commands

### Transform

```rust
use praxis_editor::TransformEditCommand;

let cmd = TransformEditCommand::new(entity, old_transform, new_transform);
```

### Entity Creation/Deletion

```rust
use praxis_editor::{CreateEntityCommand, DeleteEntityCommand};

let create = CreateEntityCommand::with_transform(Transform::default());
let delete = DeleteEntityCommand::from_world(entity, &world)?;
```

### Components

```rust
use praxis_editor::{AddComponentCommand, RemoveComponentCommand, ComponentData};

let add = AddComponentCommand::new(entity, ComponentData::Name("Player".into()));
let remove = RemoveComponentCommand::new(entity, ComponentData::Name(current_name));
```

### Hierarchy

```rust
use praxis_editor::SetParentCommand;

let cmd = SetParentCommand::from_world(child, Some(parent), &world)?;
```

### Composite Commands

Group multiple operations into one undoable action:

```rust
use praxis_editor::{CompositeCommand, SerializableCommand};

let mut composite = CompositeCommand::new("Create Scene".into());
composite.add_command(SerializableCommand::CreateEntity(cmd1));
composite.add_command(SerializableCommand::CreateEntity(cmd2));

// Undo/redo as a single operation
undo_system.execute_command(&mut world, Box::new(composite)).unwrap();
```

## Best Practices

1. **Always use `execute_command`**: Don't execute commands directly

2. **Capture state before deletion**:
   ```rust
   let cmd = DeleteEntityCommand::from_world(entity, &world)?;
   ```

3. **Group related operations**:
   ```rust
   let mut composite = CompositeCommand::new("Create Player".into());
   composite.add_command(create_cmd);
   composite.add_command(name_cmd);
   ```

4. **Mark saved appropriately**:
   ```rust
   save_scene_to_file(&world)?;
   undo_system.mark_saved();
   ```

5. **Check dirty on exit**:
   ```rust
   if undo_system.is_dirty() {
       show_unsaved_changes_dialog();
   }
   ```

## Serialization

```rust
// Save history
let ron = undo_system.to_ron()?;
std::fs::write("history.ron", ron)?;

// Load history
let ron = std::fs::read_to_string("history.ron")?;
undo_system.from_ron(&ron)?;
```

## History Limit

The system maintains a maximum of **100 commands**. When executing the 101st command, the oldest is automatically removed.

## Performance

- **Memory**: Bounded by 100 command limit
- **Execution**: O(1) for single commands, O(n) for composite
- **Undo/Redo**: O(1) stack operations

## Limitations

- Entity IDs not stable across sessions (serialized commands may not work)
- Only Transform, Name, Parent components supported out-of-box
- No branching/undo tree support

## Example

```bash
cargo run --example undo_redo_system_demo
```

## See Also

- [Command System](commands.md) - Detailed command pattern documentation
- [Selection System](selection.md) - Selection works with undo/redo
