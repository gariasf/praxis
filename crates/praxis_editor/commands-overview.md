# Command System Overview

The Praxis Editor command system provides comprehensive undo/redo functionality with full serialization support for editor operations.

## Quick Start

### Basic Usage

```rust
use praxis_editor::{CommandHistory, TransformEditCommand, UndoRedoSystem};
use praxis_ecs::{World, Transform};

// Create world and history
let mut world = World::new();
let mut history = CommandHistory::new();

// Execute a command
let entity = world.spawn(Transform::default()).id();
let command = TransformEditCommand::new(
    entity,
    Transform::default(),
    Transform::from_xyz(10.0, 5.0, 0.0)
);

history.execute(&mut world, Box::new(command)).unwrap();

// Undo/Redo
history.undo(&mut world).unwrap();
history.redo(&mut world).unwrap();
```

### ECS Resource Integration

```rust
use praxis_editor::UndoRedoSystem;
use praxis_ecs::World;

// Add as ECS resource
let mut world = World::new();
world.insert_resource(UndoRedoSystem::new());

// Use in systems
fn my_system(mut undo_system: ResMut<UndoRedoSystem>, world: &mut World) {
    // Execute commands through the resource
}
```

### Keyboard Shortcuts

```rust
use praxis_editor::handle_command_shortcuts;
use praxis_ecs::Schedule;

let mut schedule = Schedule::default();
schedule.add_systems(handle_command_shortcuts);
```

**Shortcuts:**
- `Ctrl+Z`: Undo
- `Ctrl+Y`: Redo
- `Ctrl+Shift+Z`: Redo (alternative)

## Available Commands

### TransformEditCommand

Edits entity transforms with full undo/redo support.

```rust
let command = TransformEditCommand::new(
    entity,
    old_transform,
    new_transform
);
```

### CreateEntityCommand

Creates new entities with specified components.

```rust
let command = CreateEntityCommand::with_transform(
    Transform::from_xyz(5.0, 0.0, 0.0)
);
```

### DeleteEntityCommand

Deletes entities and stores their state for undo.

```rust
let command = DeleteEntityCommand::from_world(entity, &world).unwrap();
```

### AddComponentCommand

Adds components to entities.

```rust
let command = AddComponentCommand::new(
    entity,
    ComponentData::Name("Player".to_string())
);
```

### RemoveComponentCommand

Removes components from entities (stores for undo).

```rust
let command = RemoveComponentCommand::new(
    entity,
    ComponentData::Name(name_value)
);
```

### SetParentCommand

Changes entity parent relationships.

```rust
let command = SetParentCommand::from_world(
    child_entity,
    Some(parent_entity),
    &world
).unwrap();
```

### CompositeCommand

Groups multiple commands into a single undoable operation.

```rust
let mut composite = CompositeCommand::new("Create Scene".to_string());
for i in 0..10 {
    let cmd = CreateEntityCommand::with_transform(
        Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0)
    );
    composite.add_command(SerializableCommand::CreateEntity(cmd));
}
```

## Serialization

### Individual Commands

```rust
// Serialize a command to RON
let ron_string = command.to_ron().unwrap();

// Deserialize from RON
let command = SerializableCommand::from_ron(&ron_string).unwrap();
let trait_object = command.to_trait_object();
```

### Command History

```rust
// Serialize entire history
let history_ron = history.to_ron().unwrap();
std::fs::write("history.ron", history_ron).unwrap();

// Load history
let history_ron = std::fs::read_to_string("history.ron").unwrap();
let mut history = CommandHistory::new();
history.from_ron(&history_ron).unwrap();
```

## Core Types

### EditorCommand Trait

Base trait that all commands implement:

```rust
pub trait EditorCommand: Send + Sync {
    fn execute(&mut self, world: &mut World) -> Result<()>;
    fn undo(&mut self, world: &mut World) -> Result<()>;
    fn redo(&mut self, world: &mut World) -> Result<()>;
    fn description(&self) -> String;
    fn to_ron(&self) -> Result<String>;
    fn type_id(&self) -> &'static str;
}
```

### CommandHistory

Manages undo/redo stacks:

```rust
pub struct CommandHistory {
    undo_stack: VecDeque<Box<dyn EditorCommand>>,
    redo_stack: VecDeque<Box<dyn EditorCommand>>,
    max_history_size: usize,
}
```

**Key Methods:**
- `execute(world, command)`: Execute and add to history
- `undo(world)`: Undo last command
- `redo(world)`: Redo last undone command
- `can_undo()`, `can_redo()`: Check if undo/redo is available
- `undo_description()`, `redo_description()`: Get command descriptions
- `clear()`: Clear all history
- `to_ron()`, `from_ron()`: Serialize/deserialize history

### SerializableCommand

Enum wrapper for serialization:

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializableCommand {
    TransformEdit(TransformEditCommand),
    CreateEntity(CreateEntityCommand),
    DeleteEntity(DeleteEntityCommand),
    AddComponent(AddComponentCommand),
    RemoveComponent(RemoveComponentCommand),
    SetParent(SetParentCommand),
    Composite(CompositeCommand),
}
```

### ComponentData

Serializable component data:

```rust
#[derive(Serialize, Deserialize)]
pub enum ComponentData {
    Transform(SerializableTransform),
    Name(String),
    Parent(Entity),
}
```

## Examples

### command_system_demo.rs

Comprehensive demonstration of all command types:
- Transform editing
- Entity creation/deletion
- Component management
- Hierarchy operations
- Composite commands
- Command serialization
- History management

Run: `cargo run --example command_system_demo`

### command_serialization_demo.rs

Focused demonstration of serialization:
- Single command serialization
- Composite command serialization
- History serialization
- Round-trip testing

Run: `cargo run --example command_serialization_demo`

## Integration Patterns

### With Editor UI

```rust
fn editor_menu_bar(
    ui: &mut egui::Ui,
    undo_system: &mut UndoRedoSystem,
    world: &mut World,
) {
    if ui.button("Undo").clicked() {
        if let Err(e) = undo_system.undo(world) {
            eprintln!("Undo failed: {}", e);
        }
    }
    
    if ui.button("Redo").clicked() {
        if let Err(e) = undo_system.redo(world) {
            eprintln!("Redo failed: {}", e);
        }
    }
    
    // Show command descriptions
    if let Some(desc) = undo_system.undo_description() {
        ui.label(format!("Undo: {}", desc));
    }
}
```

### With Transform Gizmos

```rust
fn gizmo_manipulation_complete(
    entity: Entity,
    old_transform: Transform,
    new_transform: Transform,
    undo_system: &mut UndoRedoSystem,
    world: &mut World,
) {
    let command = TransformEditCommand::new(entity, old_transform, new_transform);
    undo_system.execute_command(world, Box::new(command)).unwrap();
}
```

### Batch Operations

```rust
fn create_scene(
    undo_system: &mut UndoRedoSystem,
    world: &mut World,
) {
    let mut composite = CompositeCommand::new("Create Scene".to_string());
    
    // Add multiple entity creations
    for i in 0..10 {
        let cmd = CreateEntityCommand::with_transform(
            Transform::from_xyz(i as f32, 0.0, 0.0)
        );
        composite.add_command(SerializableCommand::CreateEntity(cmd));
    }
    
    // Execute as single undoable operation
    undo_system.execute_command(world, Box::new(composite)).unwrap();
}
```

## Best Practices

### Command Design

1. **Capture complete state**: Store enough information to fully undo/redo
2. **Keep commands focused**: One logical operation per command
3. **Use composites**: Group related operations
4. **Handle errors**: Return meaningful error messages
5. **Test round-trips**: Verify serialization/deserialization works

### Performance

1. **Set appropriate history size**: Default is 1000, adjust based on needs
2. **Batch operations**: Use composites to reduce undo/redo overhead
3. **Avoid large data**: Don't store excessive state in commands
4. **Clear when appropriate**: Clear history when loading new scenes

### Error Handling

```rust
match undo_system.undo(world) {
    Ok(true) => println!("Undo successful"),
    Ok(false) => println!("Nothing to undo"),
    Err(e) => eprintln!("Undo failed: {}", e),
}
```

## Extending

### Adding New Commands

1. Create command struct with `Serialize` and `Deserialize`
2. Implement `EditorCommand` trait
3. Add variant to `SerializableCommand` enum
4. Update `to_trait_object()` method

See `COMMAND_SYSTEM.md` for detailed instructions.

### Custom Component Types

To support new component types:

1. Add variant to `ComponentData` enum
2. Update command implementations to handle new type
3. Test serialization round-trips

## Limitations

Current limitations:
- Only Transform, Name, and Parent components supported
- Entity IDs are not remapped on history load
- No support for asset references
- No undo/redo branching (linear history only)

See `COMMAND_SYSTEM.md` for future enhancement plans.

## Documentation

- [COMMAND_SYSTEM.md](COMMAND_SYSTEM.md) - Detailed system documentation
- [README.md](README.md) - Editor overview
- API docs: Run `cargo doc --open`

## Testing

Run tests:
```bash
cargo test -p praxis_editor
```

Run examples:
```bash
cargo run --example command_system_demo
cargo run --example command_serialization_demo
```
