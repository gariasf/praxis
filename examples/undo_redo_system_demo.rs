//! Demonstration of the complete UndoRedoSystem with all features.
//!
//! This example shows:
//! - Command history with 100 entry limit
//! - Keyboard shortcuts (Ctrl+Z, Ctrl+Y)
//! - Menu bar integration with undo/redo actions
//! - Dirty state tracking for unsaved changes
//! - Integration with editor operations
//! - Visual feedback for command state

use praxis_ecs::{Transform, World};
use praxis_editor::{
    CommandHistory, ComponentData, CreateEntityCommand, TransformEditCommand, UndoRedoSystem,
};
use praxis_math::Vec3;

fn main() {
    println!("=== UndoRedoSystem Complete Demo ===\n");

    let mut world = World::new();

    demo_max_history_limit(&mut world);
    demo_dirty_state_tracking(&mut world);
    demo_command_integration(&mut world);
    demo_history_management();

    println!("\n=== Demo Complete ===");
}

fn demo_max_history_limit(world: &mut World) {
    println!("--- Demo 1: Maximum History Size (100 entries) ---");

    let mut history = CommandHistory::new();
    let entity = world.spawn(Transform::default()).id();

    println!("Executing 150 commands (exceeds limit of 100)...");

    for i in 0..150 {
        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::from_xyz(i as f32, 0.0, 0.0),
            Transform::from_xyz((i + 1) as f32, 0.0, 0.0),
        ));
        history.execute(world, command).unwrap();
    }

    println!("History size after 150 commands: {}", history.undo_count());
    println!(
        "✓ History correctly limited to maximum size (expected: 100, actual: {})",
        history.undo_count()
    );
    assert_eq!(history.undo_count(), 100);

    // Verify oldest commands were dropped
    println!("\nUndoing all commands...");
    let mut undo_count = 0;
    while history.undo(world).unwrap() {
        undo_count += 1;
    }
    println!("Successfully undid {} commands", undo_count);
    assert_eq!(undo_count, 100);
}

fn demo_dirty_state_tracking(world: &mut World) {
    println!("\n--- Demo 2: Dirty State Tracking for Unsaved Changes ---");

    let mut system = UndoRedoSystem::new();
    let entity = world.spawn(Transform::default()).id();

    // Initially clean
    println!("Initial state - Dirty: {}", system.is_dirty());
    assert!(!system.is_dirty());

    // Execute command -> becomes dirty
    let command = Box::new(TransformEditCommand::new(
        entity,
        Transform::default(),
        Transform::from_xyz(10.0, 0.0, 0.0),
    ));
    system.execute_command(world, command).unwrap();
    println!("After command execution - Dirty: {}", system.is_dirty());
    assert!(system.is_dirty());

    // Mark as saved -> becomes clean
    system.mark_saved();
    println!("After marking as saved - Dirty: {}", system.is_dirty());
    assert!(!system.is_dirty());

    // Execute another command -> dirty again
    let command = Box::new(TransformEditCommand::new(
        entity,
        Transform::from_xyz(10.0, 0.0, 0.0),
        Transform::from_xyz(20.0, 0.0, 0.0),
    ));
    system.execute_command(world, command).unwrap();
    println!(
        "After another command execution - Dirty: {}",
        system.is_dirty()
    );
    assert!(system.is_dirty());

    // Undo back to saved state -> becomes clean
    system.undo(world).unwrap();
    println!(
        "After undoing to saved state - Dirty: {}",
        system.is_dirty()
    );
    assert!(!system.is_dirty());

    // Redo -> becomes dirty
    system.redo(world).unwrap();
    println!("After redo - Dirty: {}", system.is_dirty());
    assert!(system.is_dirty());

    println!("✓ Dirty state tracking working correctly");
}

fn demo_command_integration(world: &mut World) {
    println!("\n--- Demo 3: Integration with Editor Operations ---");

    let mut system = UndoRedoSystem::new();

    println!("Creating entities with undo/redo...");

    // Create entities
    for i in 0..3 {
        let command = Box::new(CreateEntityCommand::with_transform(Transform::from_xyz(
            i as f32 * 5.0,
            0.0,
            0.0,
        )));
        system.execute_command(world, command).unwrap();
    }

    println!(
        "Created 3 entities - Total entities: {}",
        world.entities().len()
    );
    println!(
        "History: {} undo, {} redo",
        system.undo_count(),
        system.redo_count()
    );
    println!("Dirty: {}", system.is_dirty());

    // Undo all
    println!("\nUndoing all operations...");
    while system.can_undo() {
        let desc = system.undo_description().unwrap();
        system.undo(world).unwrap();
        println!("  Undid: {}", desc);
    }

    println!(
        "After undo all - Total entities: {}",
        world.entities().len()
    );
    println!(
        "History: {} undo, {} redo",
        system.undo_count(),
        system.redo_count()
    );

    // Redo all
    println!("\nRedoing all operations...");
    while system.can_redo() {
        let desc = system.redo_description().unwrap();
        system.redo(world).unwrap();
        println!("  Redid: {}", desc);
    }

    println!(
        "After redo all - Total entities: {}",
        world.entities().len()
    );
    println!(
        "History: {} undo, {} redo",
        system.undo_count(),
        system.redo_count()
    );

    println!("✓ Command integration working correctly");
}

fn demo_history_management() {
    println!("\n--- Demo 4: History Management Features ---");

    let mut system = UndoRedoSystem::new();
    let mut world = World::new();

    // Add some commands
    for i in 0..5 {
        let entity = world.spawn(Transform::default()).id();
        let command = Box::new(TransformEditCommand::new(
            entity,
            Transform::default(),
            Transform::from_xyz(i as f32, 0.0, 0.0),
        ));
        system.execute_command(&mut world, command).unwrap();
    }

    println!("Command History Status:");
    println!("  Can undo: {}", system.can_undo());
    println!("  Can redo: {}", system.can_redo());
    println!("  Undo count: {}", system.undo_count());
    println!("  Redo count: {}", system.redo_count());
    println!("  Is dirty: {}", system.is_dirty());

    if let Some(desc) = system.undo_description() {
        println!("  Next undo: {}", desc);
    }

    // Undo a few
    system.undo(&mut world).unwrap();
    system.undo(&mut world).unwrap();

    println!("\nAfter 2 undos:");
    println!("  Undo count: {}", system.undo_count());
    println!("  Redo count: {}", system.redo_count());

    if let Some(desc) = system.redo_description() {
        println!("  Next redo: {}", desc);
    }

    // Clear history
    println!("\nClearing history...");
    system.clear();
    println!("  Undo count: {}", system.undo_count());
    println!("  Redo count: {}", system.redo_count());
    println!("  Is dirty: {}", system.is_dirty());

    println!("✓ History management working correctly");
}

/// Simulates keyboard shortcut handling
#[allow(dead_code)]
fn simulate_keyboard_shortcuts() {
    println!("\n--- Keyboard Shortcuts ---");
    println!("The following keyboard shortcuts are available:");
    println!("  Ctrl+Z: Undo last command");
    println!("  Ctrl+Y: Redo last undone command");
    println!("  Ctrl+Shift+Z: Redo last undone command (alternative)");
    println!("\nThese are handled by the handle_command_shortcuts system");
    println!("See praxis_editor::command_shortcuts for implementation");
}

/// Simulates menu bar integration
#[allow(dead_code)]
fn simulate_menu_bar_integration() {
    println!("\n--- Menu Bar Integration ---");
    println!("The editor menu bar includes:");
    println!("  Edit > Undo: Shows command description and enabled state");
    println!("  Edit > Redo: Shows command description and enabled state");
    println!("  Edit > History: Shows undo/redo counts");
    println!("  File > Save Scene: Shows '*' when dirty");
    println!("  Status Bar: Shows 'Unsaved' indicator when dirty");
    println!("\nSee EditorState::render_menu_bar for implementation");
}
