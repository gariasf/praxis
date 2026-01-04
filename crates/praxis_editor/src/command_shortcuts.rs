//! Keyboard shortcuts for command system operations.
//!
//! This module provides helper functions for integrating keyboard shortcuts
//! with the command history system.

use crate::UndoRedoSystem;
use bevy_ecs::system::{Res, ResMut};
use bevy_ecs::world::World;
use praxis_input::InputState;
use winit::keyboard::KeyCode;

/// System that handles undo/redo keyboard shortcuts.
///
/// Keyboard shortcuts:
/// - Ctrl+Z: Undo last command
/// - Ctrl+Y: Redo last undone command
/// - Ctrl+Shift+Z: Redo last undone command (alternative)
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::Schedule;
/// use praxis_editor::handle_command_shortcuts;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(handle_command_shortcuts);
/// ```
pub fn handle_command_shortcuts(
    input: Res<InputState>,
    mut undo_system: ResMut<UndoRedoSystem>,
    world: &mut World,
) {
    let ctrl = input.is_key_pressed(KeyCode::ControlLeft)
        || input.is_key_pressed(KeyCode::ControlRight);
    let shift = input.is_key_pressed(KeyCode::ShiftLeft)
        || input.is_key_pressed(KeyCode::ShiftRight);

    // Ctrl+Z: Undo
    if ctrl && !shift && input.is_key_just_pressed(KeyCode::KeyZ) {
        if let Err(e) = undo_system.undo(world) {
            praxis_utils::error!("Undo failed: {}", e);
        }
    }

    // Ctrl+Y or Ctrl+Shift+Z: Redo
    if (ctrl && input.is_key_just_pressed(KeyCode::KeyY))
        || (ctrl && shift && input.is_key_just_pressed(KeyCode::KeyZ))
    {
        if let Err(e) = undo_system.redo(world) {
            praxis_utils::error!("Redo failed: {}", e);
        }
    }
}

/// Helper to check if undo shortcut was pressed.
pub fn is_undo_pressed(input: &InputState) -> bool {
    let ctrl = input.is_key_pressed(KeyCode::ControlLeft)
        || input.is_key_pressed(KeyCode::ControlRight);
    let shift = input.is_key_pressed(KeyCode::ShiftLeft)
        || input.is_key_pressed(KeyCode::ShiftRight);

    ctrl && !shift && input.is_key_just_pressed(KeyCode::KeyZ)
}

/// Helper to check if redo shortcut was pressed.
pub fn is_redo_pressed(input: &InputState) -> bool {
    let ctrl = input.is_key_pressed(KeyCode::ControlLeft)
        || input.is_key_pressed(KeyCode::ControlRight);
    let shift = input.is_key_pressed(KeyCode::ShiftLeft)
        || input.is_key_pressed(KeyCode::ShiftRight);

    (ctrl && input.is_key_just_pressed(KeyCode::KeyY))
        || (ctrl && shift && input.is_key_just_pressed(KeyCode::KeyZ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_helpers_exist() {
        // Test that the helper functions exist and have correct signatures
        let input = InputState::default();
        let _ = is_undo_pressed(&input);
        let _ = is_redo_pressed(&input);
    }
}
