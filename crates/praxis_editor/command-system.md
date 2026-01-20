# Command Pattern System

The Praxis Editor implements a comprehensive command pattern system for undo/redo functionality with full serialization support.

## Overview

The command system provides:

- **Execute/Undo/Redo**: Full undo/redo support for editor operations
- **Serialization**: RON format serialization for save/load and replay
- **Type Safety**: Strongly typed command implementations
- **Composability**: Group multiple commands into composite operations
- **Flexibility**: Easy to extend with new command types

## Architecture

### Core Components

#### `EditorCommand` Trait

The base trait that all commands must implement:

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

#### `CommandHistory`

Manages the undo/redo stacks and command execution:

```rust
pub struct CommandHistory {
    undo_stack: VecDeque<Box<dyn EditorCommand>>,
    redo_stack: VecDeque<Box<dyn EditorCommand>>,
    max_history_size: usize,
}
```

#### `UndoRedoSystem`

ECS Resource wrapper for `CommandHistory` with dirty state tracking:

```rust
#[derive(Resource)]
pub struct UndoRedoSystem {
    pub history: CommandHistory,
    dirty: bool,              // Tracks unsaved changes
    saved_undo_count: usize,  // Undo count at last save
}
```

**Features:**
- Command history management (max 100 entries)
- Dirty state tracking for unsaved changes
- Keyboard shortcuts (Ctrl+Z, Ctrl+Y)
- Menu bar integration
- Serialization support

See `UNDO_REDO_SYSTEM.md` for complete documentation of the undo/redo system, including:
- Dirty state tracking
- Menu bar integration
- Keyboard shortcuts
- Usage examples
- Best practices

## Concrete Commands

### TransformEditCommand

Edits entity transforms with undo/redo support.

**Fields:**
- `entity: Entity` - Target entity
- `old_transform: SerializableTransform` - State before edit
- `new_transform: SerializableTransform` - State after edit

**Example:**
```rust
let mut history = CommandHistory::new();
let entity = world.spawn(Transform::default()).id();

let command = TransformEditCommand::new(
    entity,
    Transform::default(),
    Transform::from_xyz(10.0, 5.0, 0.0)
);

history.execute(&mut world, Box::new(command)).unwrap();
```

### CreateEntityCommand

Creates new entities with specified components.

**Fields:**
- `entity: Option<Entity>` - Created entity (set during execute)
- `components: Vec<ComponentData>` - Components to add

**Example:**
```rust
let command = CreateEntityCommand::with_transform(
    Transform::from_xyz(5.0, 0.0, 0.0)
);

history.execute(&mut world, Box::new(command)).unwrap();
```

### DeleteEntityCommand

Deletes entities and stores their state for undo.

**Fields:**
- `entity: Entity` - Entity to delete
- `stored_components: Vec<ComponentData>` - Captured components
- `parent: Option<Entity>` - Parent relationship
- `children: Vec<Entity>` - Child relationships

**Example:**
```rust
let entity = world.spawn(Transform::default()).id();
let command = DeleteEntityCommand::from_world(entity, &world).unwrap();

history.execute(&mut world, Box::new(command)).unwrap();
```

### AddComponentCommand

Adds a component to an entity.

**Fields:**
- `entity: Entity` - Target entity
- `component: ComponentData` - Component to add

**Example:**
```rust
let entity = world.spawn_empty().id();
let command = AddComponentCommand::new(
    entity,
    ComponentData::Name("Player".to_string())
);

history.execute(&mut world, Box::new(command)).unwrap();
```

### RemoveComponentCommand

Removes a component from an entity (stores for undo).

**Fields:**
- `entity: Entity` - Target entity
- `component: ComponentData` - Component that was removed

**Example:**
```rust
let entity = world.spawn(Name::new("Test")).id();
let name = world.get::<Name>(entity).unwrap();
let component = ComponentData::Name(name.0.clone());

let command = RemoveComponentCommand::new(entity, component);
history.execute(&mut world, Box::new(command)).unwrap();
```

### SetParentCommand

Changes entity parent relationships.

**Fields:**
- `entity: Entity` - Entity whose parent is changing
- `old_parent: Option<Entity>` - Previous parent
- `new_parent: Option<Entity>` - New parent

**Example:**
```rust
let parent = world.spawn_empty().id();
let child = world.spawn_empty().id();

let command = SetParentCommand::from_world(child, Some(parent), &world).unwrap();
history.execute(&mut world, Box::new(command)).unwrap();
```

### CompositeCommand

Groups multiple commands into a single undoable operation.

**Fields:**
- `commands: Vec<SerializableCommand>` - Child commands
- `description: String` - Operation description

**Example:**
```rust
let mut composite = CompositeCommand::new("Create Scene".to_string());

for i in 0..10 {
    let cmd = CreateEntityCommand::with_transform(
        Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0)
    );
    composite.add_command(SerializableCommand::CreateEntity(cmd));
}

history.execute(&mut world, Box::new(composite)).unwrap();
```

## Component Data

The `ComponentData` enum represents serializable component types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentData {
    Transform(SerializableTransform),
    Name(String),
    Parent(Entity),
}
```

## Serialization

### RON Format

All commands serialize to/from RON (Rusty Object Notation):

```rust
// Serialize a command
let command = TransformEditCommand::new(entity, old_t, new_t);
let ron_string = command.to_ron().unwrap();

// Deserialize a command
let command = SerializableCommand::from_ron(&ron_string).unwrap();
let trait_object = command.to_trait_object();
```

### History Serialization

The entire command history can be saved:

```rust
// Save history
let history_ron = history.to_ron().unwrap();
std::fs::write("history.ron", history_ron).unwrap();

// Load history
let history_ron = std::fs::read_to_string("history.ron").unwrap();
let mut history = CommandHistory::new();
history.from_ron(&history_ron).unwrap();
```

## Usage Patterns

### Basic Undo/Redo

```rust
// Execute a command
history.execute(&mut world, Box::new(command)).unwrap();

// Undo
if history.can_undo() {
    history.undo(&mut world).unwrap();
}

// Redo
if history.can_redo() {
    history.redo(&mut world).unwrap();
}
```

### Integration with Editor

```rust
// In editor state
world.insert_resource(UndoRedoSystem::new());

// In editor UI
fn editor_menu(world: &mut World) {
    let mut undo_system = world.resource_mut::<UndoRedoSystem>();
    
    if ui.button("Undo").clicked() {
        if let Err(e) = undo_system.undo(world) {
            error!("Undo failed: {}", e);
        }
    }
    
    if ui.button("Redo").clicked() {
        if let Err(e) = undo_system.redo(world) {
            error!("Redo failed: {}", e);
        }
    }
}
```

### Keyboard Shortcuts

The editor provides a built-in system for undo/redo keyboard shortcuts:

```rust
use praxis_editor::handle_command_shortcuts;
use praxis_ecs::Schedule;

// Add to your schedule
let mut schedule = Schedule::default();
schedule.add_systems(handle_command_shortcuts);
```

**Keyboard shortcuts:**
- `Ctrl+Z`: Undo last command
- `Ctrl+Y`: Redo last undone command
- `Ctrl+Shift+Z`: Redo (alternative shortcut)

**Helper functions:**

```rust
use praxis_editor::{is_undo_pressed, is_redo_pressed};
use praxis_input::InputState;

fn custom_input_handler(input: Res<InputState>) {
    if is_undo_pressed(&input) {
        // Handle undo
    }
    
    if is_redo_pressed(&input) {
        // Handle redo
    }
}
```

## Extending the System

### Creating Custom Commands

To add a new command type:

1. **Define the command struct:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyCustomCommand {
    pub entity: Entity,
    pub data: MyData,
    #[serde(skip)]
    executed: bool,
}
```

2. **Implement EditorCommand:**
```rust
impl EditorCommand for MyCustomCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        // Apply changes
        self.executed = true;
        Ok(())
    }
    
    fn undo(&mut self, world: &mut World) -> Result<()> {
        // Revert changes
        self.executed = false;
        Ok(())
    }
    
    fn description(&self) -> String {
        "My Custom Operation".to_string()
    }
    
    fn to_ron(&self) -> Result<String> {
        let serializable = SerializableCommand::MyCustom(self.clone());
        ron::to_string(&serializable)
            .map_err(|e| format!("Serialization failed: {}", e).into())
    }
    
    fn type_id(&self) -> &'static str {
        "MyCustom"
    }
}
```

3. **Add to SerializableCommand enum:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializableCommand {
    // ... existing variants ...
    MyCustom(MyCustomCommand),
}
```

4. **Update to_trait_object():**
```rust
impl SerializableCommand {
    pub fn to_trait_object(self) -> Box<dyn EditorCommand> {
        match self {
            // ... existing cases ...
            SerializableCommand::MyCustom(cmd) => Box::new(cmd),
        }
    }
}
```

## Best Practices

### Command Design

1. **Store enough state for undo**: Always capture the previous state before modification
2. **Keep commands focused**: Each command should represent a single logical operation
3. **Use composite commands**: Group related operations that should be undone together
4. **Handle errors gracefully**: Return `Result` and provide meaningful error messages

### Performance Considerations

1. **Limit history size**: The default is 100 commands to prevent unbounded memory growth
2. **Be mindful of clone costs**: Commands are cloned during serialization
3. **Lazy evaluation**: Don't capture expensive data until needed
4. **Consider compression**: For large histories, compress the RON string

### Testing Commands

```rust
#[test]
fn test_my_command() {
    let mut world = World::new();
    let entity = world.spawn(MyComponent::default()).id();
    
    let mut command = MyCustomCommand::new(entity);
    
    // Test execute
    assert!(command.execute(&mut world).is_ok());
    // Verify changes
    
    // Test undo
    assert!(command.undo(&mut world).is_ok());
    // Verify restoration
    
    // Test redo
    assert!(command.redo(&mut world).is_ok());
    // Verify changes again
    
    // Test serialization
    let ron = command.to_ron().unwrap();
    let deserialized = SerializableCommand::from_ron(&ron).unwrap();
    // Verify deserialized command works
}
```

## Implementation Notes

### Entity References

Entity references in commands must be handled carefully:
- Entities may be destroyed outside the command system
- Check entity validity before operations
- Consider using entity generations for safety

### World Access

Commands receive mutable world access:
- Be careful not to invalidate iterators
- Avoid holding entity references across operations
- Consider using `EntityRef`/`EntityMut` for safety

### Serialization Limitations

Current limitations:
- Only Transform, Name, and Parent components supported
- Entity IDs are serialized as-is (may need remapping on load)
- No support for asset references yet

### Future Enhancements

Potential improvements:
- Entity ID remapping on history load
- Support for more component types
- Command batching for performance
- Network synchronization support
- Command validation before execution
- Undo/redo branching (tree structure instead of linear)
