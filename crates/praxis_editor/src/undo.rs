//! Undo/Redo system for editor operations.
//!
//! This module provides a command-based undo/redo system for editor operations,
//! particularly transform manipulations via gizmos.

use praxis_ecs::{Entity, Resource, Transform};
use std::collections::VecDeque;

/// Maximum number of undo/redo entries to keep in history.
const MAX_HISTORY_SIZE: usize = 100;

/// Trait for commands that can be undone and redone.
pub trait Command: Send + Sync {
    /// Executes the command.
    fn execute(&mut self);

    /// Undoes the command.
    fn undo(&mut self);

    /// Returns a description of the command for debugging/UI.
    fn description(&self) -> String;
}

/// Command for transforming entities.
///
/// Stores the old and new transforms and the entities affected.
pub struct TransformCommand {
    /// Entities whose transforms are affected.
    pub entities: Vec<Entity>,
    /// Old transform states (before the operation).
    pub old_transforms: Vec<Transform>,
    /// New transform states (after the operation).
    pub new_transforms: Vec<Transform>,
    /// Whether the command has been executed.
    executed: bool,
}

impl TransformCommand {
    /// Creates a new transform command.
    pub fn new(entities: Vec<Entity>, old_transforms: Vec<Transform>, new_transforms: Vec<Transform>) -> Self {
        assert_eq!(entities.len(), old_transforms.len());
        assert_eq!(entities.len(), new_transforms.len());
        
        Self {
            entities,
            old_transforms,
            new_transforms,
            executed: false,
        }
    }
}

impl Command for TransformCommand {
    fn execute(&mut self) {
        // In practice, this would apply the new transforms to entities
        // Since we're working with ECS, this would need access to the World
        self.executed = true;
    }

    fn undo(&mut self) {
        // In practice, this would apply the old transforms to entities
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

/// Undo/Redo system resource managing command history.
///
/// This system maintains two stacks: one for undo operations and one for redo operations.
/// When a new command is executed, it's pushed onto the undo stack. When undoing, commands
/// are moved from the undo stack to the redo stack. When redoing, they're moved back.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::UndoRedoSystem;
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// world.insert_resource(UndoRedoSystem::new());
///
/// // Later in your editor code:
/// // undo_redo.execute_command(Box::new(my_command));
/// // undo_redo.undo();
/// // undo_redo.redo();
/// ```
#[derive(Resource)]
pub struct UndoRedoSystem {
    /// Stack of commands that can be undone.
    undo_stack: VecDeque<Box<dyn Command>>,
    /// Stack of commands that can be redone.
    redo_stack: VecDeque<Box<dyn Command>>,
}

impl Default for UndoRedoSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoRedoSystem {
    /// Creates a new undo/redo system.
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    /// Executes a command and adds it to the undo stack.
    ///
    /// This clears the redo stack since executing a new command
    /// invalidates any previously undone commands.
    pub fn execute_command(&mut self, mut command: Box<dyn Command>) {
        command.execute();
        
        // Clear redo stack when a new command is executed
        self.redo_stack.clear();
        
        // Add to undo stack
        self.undo_stack.push_back(command);
        
        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.pop_front();
        }
    }

    /// Undoes the last command.
    ///
    /// Returns true if a command was undone, false if the undo stack is empty.
    pub fn undo(&mut self) -> bool {
        if let Some(mut command) = self.undo_stack.pop_back() {
            command.undo();
            self.redo_stack.push_back(command);
            true
        } else {
            false
        }
    }

    /// Redoes the last undone command.
    ///
    /// Returns true if a command was redone, false if the redo stack is empty.
    pub fn redo(&mut self) -> bool {
        if let Some(mut command) = self.redo_stack.pop_back() {
            command.execute();
            self.undo_stack.push_back(command);
            true
        } else {
            false
        }
    }

    /// Returns true if there are commands that can be undone.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns true if there are commands that can be redone.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Gets a description of the next command that would be undone.
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Gets a description of the next command that would be redone.
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Clears all undo/redo history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Returns the number of commands in the undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns the number of commands in the redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        value: i32,
        executed: bool,
    }

    impl TestCommand {
        fn new(value: i32) -> Self {
            Self {
                value,
                executed: false,
            }
        }
    }

    impl Command for TestCommand {
        fn execute(&mut self) {
            self.executed = true;
        }

        fn undo(&mut self) {
            self.executed = false;
        }

        fn description(&self) -> String {
            format!("Test Command {}", self.value)
        }
    }

    #[test]
    fn test_undo_redo_system_creation() {
        let system = UndoRedoSystem::new();
        assert!(!system.can_undo());
        assert!(!system.can_redo());
        assert_eq!(system.undo_count(), 0);
        assert_eq!(system.redo_count(), 0);
    }

    #[test]
    fn test_execute_command() {
        let mut system = UndoRedoSystem::new();
        let command = Box::new(TestCommand::new(1));

        system.execute_command(command);

        assert!(system.can_undo());
        assert!(!system.can_redo());
        assert_eq!(system.undo_count(), 1);
    }

    #[test]
    fn test_undo() {
        let mut system = UndoRedoSystem::new();
        system.execute_command(Box::new(TestCommand::new(1)));

        assert!(system.undo());
        assert!(!system.can_undo());
        assert!(system.can_redo());
        assert_eq!(system.redo_count(), 1);
    }

    #[test]
    fn test_redo() {
        let mut system = UndoRedoSystem::new();
        system.execute_command(Box::new(TestCommand::new(1)));
        system.undo();

        assert!(system.redo());
        assert!(system.can_undo());
        assert!(!system.can_redo());
        assert_eq!(system.undo_count(), 1);
    }

    #[test]
    fn test_execute_clears_redo_stack() {
        let mut system = UndoRedoSystem::new();
        system.execute_command(Box::new(TestCommand::new(1)));
        system.undo();
        assert!(system.can_redo());

        system.execute_command(Box::new(TestCommand::new(2)));
        assert!(!system.can_redo());
    }

    #[test]
    fn test_undo_redo_descriptions() {
        let mut system = UndoRedoSystem::new();
        system.execute_command(Box::new(TestCommand::new(1)));

        assert_eq!(system.undo_description(), Some("Test Command 1".to_string()));
        assert_eq!(system.redo_description(), None);

        system.undo();

        assert_eq!(system.undo_description(), None);
        assert_eq!(system.redo_description(), Some("Test Command 1".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut system = UndoRedoSystem::new();
        system.execute_command(Box::new(TestCommand::new(1)));
        system.execute_command(Box::new(TestCommand::new(2)));
        system.undo();

        system.clear();

        assert!(!system.can_undo());
        assert!(!system.can_redo());
        assert_eq!(system.undo_count(), 0);
        assert_eq!(system.redo_count(), 0);
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut system = UndoRedoSystem::new();
        system.execute_command(Box::new(TestCommand::new(1)));
        system.execute_command(Box::new(TestCommand::new(2)));
        system.execute_command(Box::new(TestCommand::new(3)));

        assert_eq!(system.undo_count(), 3);

        assert!(system.undo());
        assert!(system.undo());
        assert_eq!(system.undo_count(), 1);
        assert_eq!(system.redo_count(), 2);

        assert!(system.redo());
        assert_eq!(system.undo_count(), 2);
        assert_eq!(system.redo_count(), 1);
    }
}
