# Undo/Redo System

This document describes the undo/redo system for editor operations in the Praxis editor.

## Overview

The undo/redo system provides a command-based architecture for reversible operations in the editor. It maintains two stacks (undo and redo) and allows users to undo and redo their actions, particularly transform manipulations via gizmos.

## Architecture

### Core Components

#### `UndoRedoSystem` (Resource)

The main system resource that manages command history:

```rust
#[derive(Resource)]
pub struct UndoRedoSystem {
    undo_stack: VecDeque<Box<dyn Command>>,  // Commands that can be undone
    redo_stack: VecDeque<Box<dyn Command>>,  // Commands that can be redone
}
```

**Key Methods:**
- `execute_command(command)` - Execute a command and add to undo stack
- `undo()` - Undo the last command
- `redo()` - Redo the last undone command
- `can_undo()` / `can_redo()` - Check if operations are available
- `undo_description()` / `redo_description()` - Get command descriptions
- `clear()` - Clear all history
- `undo_count()` / `redo_count()` - Get stack sizes

#### `Command` Trait

All undoable operations must implement this trait:

```rust
pub trait Command: Send + Sync {
    fn execute(&mut self);
    fn undo(&mut self);
    fn description(&self) -> String;
}
```

#### `TransformCommand`

Built-in command for transform operations:

```rust
pub struct TransformCommand {
    entities: Vec<Entity>,
    old_transforms: Vec<Transform>,
    new_transforms: Vec<Transform>,
}
```

This command stores both the old and new states of entity transforms, allowing them to be restored during undo/redo operations.

## Usage

### Basic Setup

```rust
use praxis_editor::UndoRedoSystem;
use praxis_ecs::World;

// Initialize in your world
world.insert_resource(UndoRedoSystem::new());
```

### Executing Commands

```rust
use praxis_editor::TransformCommand;

// After a gizmo operation completes
let command = TransformCommand::new(
    entities,           // Vec<Entity>
    old_transforms,     // Vec<Transform> - before manipulation
    new_transforms,     // Vec<Transform> - after manipulation
);

undo_system.execute_command(Box::new(command));
```

### Undo/Redo Operations

```rust
// Undo the last operation
if undo_system.can_undo() {
    undo_system.undo();
}

// Redo a previously undone operation
if undo_system.can_redo() {
    undo_system.redo();
}
```

### Keyboard Shortcuts

Common shortcuts to implement:

```rust
use winit::keyboard::KeyCode;

let ctrl = input.is_key_pressed(KeyCode::ControlLeft) 
    || input.is_key_pressed(KeyCode::ControlRight);

// Ctrl+Z: Undo
if ctrl && input.is_key_just_pressed(KeyCode::KeyZ) {
    if !input.is_key_pressed(KeyCode::ShiftLeft) 
        && !input.is_key_pressed(KeyCode::ShiftRight) 
    {
        undo_system.undo();
    }
}

// Ctrl+Shift+Z or Ctrl+Y: Redo
if ctrl && (
    (input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight))
        && input.is_key_just_pressed(KeyCode::KeyZ)
    || input.is_key_just_pressed(KeyCode::KeyY)
) {
    undo_system.redo();
}
```

### UI Integration

Display undo/redo status in menus:

```rust
// In menu bar
ui.menu_button("Edit", |ui| {
    // Undo button
    let undo_text = if let Some(desc) = undo_system.undo_description() {
        format!("Undo {}", desc)
    } else {
        "Undo".to_string()
    };
    
    if ui.add_enabled(undo_system.can_undo(), egui::Button::new(undo_text)).clicked() {
        undo_system.undo();
        ui.close_menu();
    }
    
    // Redo button
    let redo_text = if let Some(desc) = undo_system.redo_description() {
        format!("Redo {}", desc)
    } else {
        "Redo".to_string()
    };
    
    if ui.add_enabled(undo_system.can_redo(), egui::Button::new(redo_text)).clicked() {
        undo_system.redo();
        ui.close_menu();
    }
});
```

## Command Implementation

### Creating Custom Commands

To create a custom undoable operation:

```rust
use praxis_editor::Command;

struct MyCustomCommand {
    // Store state needed for undo
    entity: Entity,
    old_value: SomeValue,
    new_value: SomeValue,
}

impl Command for MyCustomCommand {
    fn execute(&mut self) {
        // Apply the new value
        // Access to ECS world needed here
    }
    
    fn undo(&mut self) {
        // Restore the old value
        // Access to ECS world needed here
    }
    
    fn description(&self) -> String {
        "My Custom Operation".to_string()
    }
}
```

### Transform Command Details

The `TransformCommand` implementation:

```rust
impl Command for TransformCommand {
    fn execute(&mut self) {
        // In practice, this would apply new_transforms to entities
        // Requires access to the ECS World
        self.executed = true;
    }

    fn undo(&mut self) {
        // In practice, this would apply old_transforms to entities
        // Requires access to the ECS World
        self.executed = false;
    }

    fn description(&self) -> String {
        if self.entities.len() == 1 {
            "Transform Entity".to_string()
        } else {
            format!("Transform {} Entities", self.entities.len())
        }
    }
}
```

**Note**: The actual implementation requires access to the ECS `World` to apply transforms. This typically happens in a system that has access to both the `UndoRedoSystem` and entity queries.

## Stack Behavior

### Undo Stack

When a command is executed:
1. `command.execute()` is called
2. Command is pushed onto undo stack
3. Redo stack is cleared (executing new operations invalidates redo history)
4. If stack exceeds max size (100), oldest command is removed

### Redo Stack

When undo is called:
1. Command is popped from undo stack
2. `command.undo()` is called
3. Command is pushed onto redo stack

When redo is called:
1. Command is popped from redo stack
2. `command.execute()` is called
3. Command is pushed back onto undo stack

### History Limit

The system maintains a maximum of 100 commands in history to prevent unbounded memory growth. This can be adjusted by modifying `MAX_HISTORY_SIZE`.

## Integration with Gizmo System

The gizmo system integrates with undo/redo through the interaction flow:

```rust
// When gizmo interaction ends
if let Some(interaction) = gizmo_system.end_interaction() {
    // Extract transform data
    let entities: Vec<Entity> = interaction.initial_transforms
        .iter()
        .map(|(e, _)| *e)
        .collect();
    
    let old_transforms: Vec<Transform> = interaction.initial_transforms
        .iter()
        .map(|(_, t)| *t)
        .collect();
    
    // Query current transforms
    let new_transforms: Vec<Transform> = entities
        .iter()
        .map(|&e| *world.get::<Transform>(e).unwrap())
        .collect();
    
    // Create and execute command
    let command = TransformCommand::new(entities, old_transforms, new_transforms);
    undo_system.execute_command(Box::new(command));
}
```

## System Integration

To properly integrate undo/redo with the ECS:

```rust
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use praxis_editor::UndoRedoSystem;
use praxis_ecs::Transform;
use praxis_input::InputState;
use winit::keyboard::KeyCode;

fn undo_redo_input_system(
    mut undo_system: ResMut<UndoRedoSystem>,
    input: Res<InputState>,
) {
    let ctrl = input.is_key_pressed(KeyCode::ControlLeft) 
        || input.is_key_pressed(KeyCode::ControlRight);
    
    let shift = input.is_key_pressed(KeyCode::ShiftLeft) 
        || input.is_key_pressed(KeyCode::ShiftRight);
    
    // Ctrl+Z: Undo
    if ctrl && !shift && input.is_key_just_pressed(KeyCode::KeyZ) {
        undo_system.undo();
    }
    
    // Ctrl+Shift+Z: Redo
    if ctrl && shift && input.is_key_just_pressed(KeyCode::KeyZ) {
        undo_system.redo();
    }
    
    // Ctrl+Y: Redo (alternative)
    if ctrl && input.is_key_just_pressed(KeyCode::KeyY) {
        undo_system.redo();
    }
}

fn apply_undo_redo_system(
    mut commands: Commands,
    undo_system: Res<UndoRedoSystem>,
    mut transforms: Query<&mut Transform>,
) {
    // This would need additional state tracking to know which
    // commands need to be applied. In practice, commands would
    // store a "needs_apply" flag or similar mechanism.
}
```

## Best Practices

### Command Granularity

- **Do**: Create one command per user action (e.g., one gizmo drag operation)
- **Don't**: Create a command for every frame of a drag operation

### State Capture

- **Do**: Capture complete state needed to undo/redo
- **Do**: Store entity IDs rather than direct references
- **Don't**: Store pointers or references to ECS components

### Command Descriptions

- **Do**: Provide clear, user-friendly descriptions
- **Do**: Include context (e.g., "Transform 3 Entities")
- **Don't**: Use technical jargon or internal names

### Memory Management

- **Do**: Be mindful of command size (they're stored in memory)
- **Do**: Clear undo/redo history when loading a new scene
- **Don't**: Store large data structures in commands unnecessarily

## Future Enhancements

Potential improvements to the undo/redo system:

1. **Persistent History**: Save/load undo history with scene files
2. **History Browser**: UI panel showing full command history
3. **Selective Undo**: Undo specific commands in history
4. **Command Merging**: Combine similar consecutive commands
5. **Branching History**: Support for non-linear undo (undo tree)
6. **Command Groups**: Batch multiple operations into one undo step
7. **Memory Limits**: Configurable memory budget for history
8. **Command Compression**: Reduce memory usage for transform commands

## Testing

The undo/redo module includes comprehensive tests covering:

- System creation and initialization
- Command execution and undo/redo
- Stack behavior and limits
- Command descriptions
- History clearing
- Multiple undo/redo operations

Run tests with:
```bash
cargo test -p praxis_editor undo
```

## See Also

- `GIZMOS.md` - Transform gizmo integration
- `SELECTION_SYSTEM.md` - Entity selection system
- `ecs/components.rs` - Transform component definition
