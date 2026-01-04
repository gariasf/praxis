# UndoRedoSystem Quick Start Guide

This guide provides a quick introduction to using the UndoRedoSystem in your Praxis editor application.

## Installation

The UndoRedoSystem is part of `praxis_editor`. Add it to your dependencies:

```toml
[dependencies]
praxis_editor = { path = "crates/praxis_editor" }
praxis_ecs = { path = "crates/praxis_ecs" }
```

## Basic Usage

### 1. Create the System

```rust
use praxis_editor::UndoRedoSystem;
use praxis_ecs::World;

let mut world = World::new();
let undo_system = UndoRedoSystem::new();

// Insert as an ECS resource
world.insert_resource(undo_system);
```

### 2. Execute Commands

```rust
use praxis_editor::TransformEditCommand;
use praxis_ecs::Transform;

// Get the system from world
let mut undo_system = world.remove_resource::<UndoRedoSystem>().unwrap();

// Create a command
let entity = world.spawn(Transform::default()).id();
let command = Box::new(TransformEditCommand::new(
    entity,
    Transform::default(),
    Transform::from_xyz(10.0, 5.0, 3.0),
));

// Execute it
undo_system.execute_command(&mut world, command).unwrap();

// The system is now dirty (has unsaved changes)
println!("Dirty: {}", undo_system.is_dirty());

// Put it back
world.insert_resource(undo_system);
```

### 3. Undo and Redo

```rust
let mut undo_system = world.remove_resource::<UndoRedoSystem>().unwrap();

// Undo the last command
if undo_system.can_undo() {
    undo_system.undo(&mut world).unwrap();
    println!("Undone!");
}

// Redo it
if undo_system.can_redo() {
    undo_system.redo(&mut world).unwrap();
    println!("Redone!");
}

world.insert_resource(undo_system);
```

### 4. Check Dirty State

```rust
let undo_system = world.resource::<UndoRedoSystem>();

if undo_system.is_dirty() {
    println!("You have unsaved changes!");
}
```

### 5. Mark as Saved

```rust
let mut undo_system = world.resource_mut::<UndoRedoSystem>();

// After saving to disk
save_scene_to_file(&world).unwrap();
undo_system.mark_saved();

// Now it's clean
assert!(!undo_system.is_dirty());
```

## Keyboard Shortcuts

Add the keyboard shortcut handler to your schedule:

```rust
use praxis_editor::handle_command_shortcuts;
use praxis_ecs::Schedule;
use praxis_input::InputState;

let mut schedule = Schedule::default();
schedule.add_systems(handle_command_shortcuts);

// Don't forget to add InputState resource
world.insert_resource(InputState::default());

// In your game loop
schedule.run(&mut world);
```

This automatically handles:
- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Ctrl+Shift+Z**: Redo (alternative)

## Menu Bar Integration

Integrate with the editor UI:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// In your render loop
let mut undo_system = world.remove_resource::<UndoRedoSystem>().unwrap();

editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));

world.insert_resource(undo_system);
```

This adds:
- **Edit > Undo**: Shows command description and Ctrl+Z shortcut
- **Edit > Redo**: Shows command description and Ctrl+Y shortcut
- **Edit > History**: Shows undo/redo stack counts
- **File > Save Scene**: Shows asterisk (*) when dirty
- **Status Bar**: Shows "● Unsaved" indicator when dirty

## Available Commands

### Transform Editing
```rust
use praxis_editor::TransformEditCommand;

let command = TransformEditCommand::new(
    entity,
    old_transform,
    new_transform,
);
```

### Entity Creation
```rust
use praxis_editor::CreateEntityCommand;

let command = CreateEntityCommand::with_transform(
    Transform::from_xyz(5.0, 0.0, 0.0)
);
```

### Entity Deletion
```rust
use praxis_editor::DeleteEntityCommand;

// Captures entity state for undo
let command = DeleteEntityCommand::from_world(entity, &world).unwrap();
```

### Component Management
```rust
use praxis_editor::{AddComponentCommand, RemoveComponentCommand, ComponentData};

// Add component
let command = AddComponentCommand::new(
    entity,
    ComponentData::Name("Player".to_string())
);

// Remove component
let current_name = world.get::<Name>(entity).unwrap().0.clone();
let command = RemoveComponentCommand::new(
    entity,
    ComponentData::Name(current_name)
);
```

### Composite Operations
```rust
use praxis_editor::{CompositeCommand, SerializableCommand};

let mut composite = CompositeCommand::new("Create Scene".to_string());

for i in 0..10 {
    let cmd = CreateEntityCommand::with_transform(
        Transform::from_xyz(i as f32 * 5.0, 0.0, 0.0)
    );
    composite.add_command(SerializableCommand::CreateEntity(cmd));
}

// All 10 entities are created/deleted as one operation
undo_system.execute_command(&mut world, Box::new(composite)).unwrap();
```

## Complete Example

```rust
use praxis_editor::{EditorState, UndoRedoSystem, TransformEditCommand, handle_command_shortcuts};
use praxis_ecs::{World, Schedule, Transform};
use praxis_input::InputState;

fn main() {
    // Setup
    let mut world = World::new();
    let mut schedule = Schedule::default();
    let mut editor = EditorState::new();
    
    // Add systems
    schedule.add_systems(handle_command_shortcuts);
    
    // Add resources
    world.insert_resource(InputState::default());
    world.insert_resource(UndoRedoSystem::new());
    
    // Create an entity
    let entity = world.spawn(Transform::default()).id();
    
    // Game loop
    loop {
        // Handle keyboard shortcuts
        schedule.run(&mut world);
        
        // UI interaction
        let mut undo_system = world.remove_resource::<UndoRedoSystem>().unwrap();
        
        // Render editor (includes undo/redo menu)
        editor.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
        
        // Example: User clicks "Move Entity" button
        if user_wants_to_move_entity() {
            let old_transform = *world.get::<Transform>(entity).unwrap();
            let new_transform = Transform::from_xyz(10.0, 0.0, 0.0);
            
            let command = Box::new(TransformEditCommand::new(
                entity,
                old_transform,
                new_transform,
            ));
            
            undo_system.execute_command(&mut world, command).unwrap();
        }
        
        // Check for unsaved changes
        if undo_system.is_dirty() {
            // Show indicator in window title or status bar
        }
        
        world.insert_resource(undo_system);
    }
}
```

## Key Features

✅ **100 Command History**: Automatically maintains the last 100 commands  
✅ **Dirty State Tracking**: Knows when you have unsaved changes  
✅ **Smart State Management**: Detects when you undo back to saved state  
✅ **Keyboard Shortcuts**: Ctrl+Z and Ctrl+Y work out of the box  
✅ **Menu Integration**: Shows command descriptions and enabled state  
✅ **Serialization**: Save/load command history to RON format  

## Next Steps

- Read [UNDO_REDO_SYSTEM.md](UNDO_REDO_SYSTEM.md) for detailed documentation
- Read [COMMAND_SYSTEM.md](COMMAND_SYSTEM.md) for command implementation details
- Run `cargo run --example undo_redo_system_demo` to see it in action
- Look at `examples/command_system_demo.rs` for more examples

## Common Patterns

### Check Before Exit
```rust
if undo_system.is_dirty() {
    let response = show_dialog("You have unsaved changes. Save before exit?");
    if response == DialogResponse::Save {
        save_scene(&world).unwrap();
        undo_system.mark_saved();
    }
}
```

### Auto-Save on Command
```rust
undo_system.execute_command(&mut world, command).unwrap();

// Auto-save after every command (optional)
if config.auto_save {
    save_scene(&world).unwrap();
    undo_system.mark_saved();
}
```

### Show History in UI
```rust
ui.label(format!("Undo: {}", undo_system.undo_count()));
ui.label(format!("Redo: {}", undo_system.redo_count()));

if let Some(desc) = undo_system.undo_description() {
    ui.label(format!("Can undo: {}", desc));
}
```

### Disable Buttons When Empty
```rust
let undo_enabled = undo_system.can_undo();
if ui.add_enabled(undo_enabled, egui::Button::new("Undo")).clicked() {
    undo_system.undo(&mut world).unwrap();
}

let redo_enabled = undo_system.can_redo();
if ui.add_enabled(redo_enabled, egui::Button::new("Redo")).clicked() {
    undo_system.redo(&mut world).unwrap();
}
```
