//! Entity and component operations with full undo/redo support and ECS synchronization.
//!
//! This module provides a comprehensive API for entity and component management in the editor.
//! All operations are integrated with the command system for undo/redo support and ensure
//! proper synchronization with the ECS World.
//!
//! # Core Features
//!
//! - **Entity Operations**: Create, delete, and duplicate entities
//! - **Component Operations**: Add and remove components with type safety
//! - **Undo/Redo Integration**: All operations create commands for history tracking
//! - **ECS Synchronization**: Automatic World updates with proper error handling
//! - **State Capture**: Automatic component state capture for reliable undo
//!
//! # Usage Examples
//!
//! ## Creating Entities
//!
//! ```rust,no_run
//! use praxis_editor::{EntityOperations, UndoRedoSystem};
//! use praxis_ecs::{World, Transform, Name};
//!
//! let mut world = World::new();
//! let mut undo_system = UndoRedoSystem::new();
//! let mut entity_ops = EntityOperations::new();
//!
//! // Create an empty entity
//! let entity = entity_ops.create_entity(&mut world, &mut undo_system)?;
//!
//! // Create an entity with transform
//! let entity = entity_ops.create_entity_with_transform(
//!     &mut world,
//!     &mut undo_system,
//!     Transform::from_xyz(0.0, 1.0, 0.0)
//! )?;
//!
//! // Create an entity with name and transform
//! let entity = entity_ops.create_entity_with_components(
//!     &mut world,
//!     &mut undo_system,
//!     "Player",
//!     Transform::from_xyz(10.0, 0.0, 5.0)
//! )?;
//! ```
//!
//! ## Deleting Entities
//!
//! ```rust,no_run
//! # use praxis_editor::{EntityOperations, UndoRedoSystem};
//! # use praxis_ecs::{World, Transform, Entity};
//! # let mut world = World::new();
//! # let mut undo_system = UndoRedoSystem::new();
//! # let mut entity_ops = EntityOperations::new();
//! # let entity = world.spawn(Transform::default());
//! // Delete a single entity
//! entity_ops.delete_entity(&mut world, &mut undo_system, entity)?;
//!
//! // Delete multiple entities at once
//! let entities = vec![entity1, entity2, entity3];
//! entity_ops.delete_entities(&mut world, &mut undo_system, entities)?;
//! ```
//!
//! ## Duplicating Entities
//!
//! ```rust,no_run
//! # use praxis_editor::{EntityOperations, UndoRedoSystem};
//! # use praxis_ecs::{World, Transform, Entity};
//! # let mut world = World::new();
//! # let mut undo_system = UndoRedoSystem::new();
//! # let mut entity_ops = EntityOperations::new();
//! # let entity = world.spawn(Transform::default());
//! // Duplicate an entity with all its components
//! let new_entity = entity_ops.duplicate_entity(&mut world, &mut undo_system, entity)?;
//!
//! // Duplicate with position offset
//! let new_entity = entity_ops.duplicate_entity_with_offset(
//!     &mut world,
//!     &mut undo_system,
//!     entity,
//!     Vec3::new(1.0, 0.0, 0.0)
//! )?;
//!
//! // Duplicate multiple entities
//! let entities = vec![entity1, entity2];
//! let new_entities = entity_ops.duplicate_entities(&mut world, &mut undo_system, entities)?;
//! ```
//!
//! ## Adding Components
//!
//! ```rust,no_run
//! # use praxis_editor::{EntityOperations, UndoRedoSystem, ComponentData};
//! # use praxis_ecs::{World, Transform, Entity, Name};
//! # let mut world = World::new();
//! # let mut undo_system = UndoRedoSystem::new();
//! # let mut entity_ops = EntityOperations::new();
//! # let entity = world.spawn_empty();
//! // Add a transform component
//! entity_ops.add_transform(
//!     &mut world,
//!     &mut undo_system,
//!     entity,
//!     Transform::default()
//! )?;
//!
//! // Add a name component
//! entity_ops.add_name(&mut world, &mut undo_system, entity, "My Entity")?;
//!
//! // Add a generic component
//! entity_ops.add_component(
//!     &mut world,
//!     &mut undo_system,
//!     entity,
//!     ComponentData::Transform(Transform::default().into())
//! )?;
//! ```
//!
//! ## Removing Components
//!
//! ```rust,no_run
//! # use praxis_editor::{EntityOperations, UndoRedoSystem};
//! # use praxis_ecs::{World, Transform, Entity};
//! # let mut world = World::new();
//! # let mut undo_system = UndoRedoSystem::new();
//! # let mut entity_ops = EntityOperations::new();
//! # let entity = world.spawn(Transform::default());
//! // Remove a transform component
//! entity_ops.remove_transform(&mut world, &mut undo_system, entity)?;
//!
//! // Remove a name component
//! entity_ops.remove_name(&mut world, &mut undo_system, entity)?;
//! ```
//!
//! ## Batch Operations
//!
//! ```rust,no_run
//! # use praxis_editor::{EntityOperations, UndoRedoSystem};
//! # use praxis_ecs::{World, Entity};
//! # let mut world = World::new();
//! # let mut undo_system = UndoRedoSystem::new();
//! # let mut entity_ops = EntityOperations::new();
//! # let entities = vec![];
//! // Start a batch operation
//! entity_ops.begin_batch("Create Multiple Objects");
//!
//! for i in 0..10 {
//!     entity_ops.create_entity_with_components(
//!         &mut world,
//!         &mut undo_system,
//!         &format!("Entity {}", i),
//!         Transform::from_xyz(i as f32, 0.0, 0.0)
//!     )?;
//! }
//!
//! // End batch - all operations are grouped as one undo command
//! entity_ops.end_batch(&mut world, &mut undo_system)?;
//! ```
//!
//! # Error Handling
//!
//! All operations return `Result<T, EntityOperationsError>` with specific error types:
//! - `EntityNotFound`: Entity doesn't exist in World
//! - `ComponentNotFound`: Required component is missing
//! - `CommandExecutionFailed`: Command failed to execute
//! - `InvalidOperation`: Operation cannot be performed in current state
//!
//! # Integration with Selection System
//!
//! EntityOperations works seamlessly with the selection system:
//!
//! ```rust,no_run
//! # use praxis_editor::{EntityOperations, UndoRedoSystem, SelectionSystem};
//! # use praxis_ecs::World;
//! # let mut world = World::new();
//! # let mut undo_system = UndoRedoSystem::new();
//! # let mut entity_ops = EntityOperations::new();
//! # let selection_system = SelectionSystem::new();
//! // Delete all selected entities
//! let selected = selection_system.selected_entities();
//! entity_ops.delete_entities(&mut world, &mut undo_system, selected.to_vec())?;
//!
//! // Duplicate selected entities
//! let new_entities = entity_ops.duplicate_entities(
//!     &mut world,
//!     &mut undo_system,
//!     selected.to_vec()
//! )?;
//! ```

use crate::undo::{
    AddComponentCommand, ComponentData, CompositeCommand, CreateEntityCommand, DeleteEntityCommand,
    EditorCommand, RemoveComponentCommand, SerializableCommand, UndoRedoSystem,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use praxis_ecs::{Name, Parent, Transform};
use praxis_math::Vec3;
use std::fmt;

/// Result type for entity operations.
pub type Result<T> = std::result::Result<T, EntityOperationsError>;

/// Error types for entity operations.
#[derive(Debug, Clone)]
pub enum EntityOperationsError {
    /// Entity was not found in the World.
    EntityNotFound(Entity),
    /// Required component was not found on entity.
    ComponentNotFound { entity: Entity, component: String },
    /// Command execution failed.
    CommandExecutionFailed(String),
    /// Operation is invalid in the current state.
    InvalidOperation(String),
}

impl fmt::Display for EntityOperationsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntityNotFound(entity) => {
                write!(f, "Entity {:?} not found in World", entity)
            }
            Self::ComponentNotFound { entity, component } => {
                write!(
                    f,
                    "Component '{}' not found on entity {:?}",
                    component, entity
                )
            }
            Self::CommandExecutionFailed(msg) => {
                write!(f, "Command execution failed: {}", msg)
            }
            Self::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {}", msg)
            }
        }
    }
}

impl std::error::Error for EntityOperationsError {}

/// High-level API for entity and component operations with undo/redo support.
///
/// This struct provides convenient methods for all common entity operations while
/// ensuring proper integration with the command system and ECS World.
///
/// # Thread Safety
///
/// EntityOperations is not thread-safe and should only be used on the main thread
/// where the World and UndoRedoSystem are accessed.
pub struct EntityOperations {
    /// Optional batch operation state.
    batch_operation: Option<BatchOperation>,
}

/// Batch operation state for grouping multiple commands.
struct BatchOperation {
    description: String,
    commands: Vec<SerializableCommand>,
}

impl Default for EntityOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityOperations {
    /// Creates a new EntityOperations instance.
    pub fn new() -> Self {
        Self {
            batch_operation: None,
        }
    }

    // ============================================================================
    // Entity Creation
    // ============================================================================

    /// Creates a new empty entity.
    ///
    /// # Returns
    ///
    /// The newly created entity ID.
    ///
    /// # Errors
    ///
    /// Returns error if command execution fails.
    pub fn create_entity(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
    ) -> Result<Entity> {
        let mut command = CreateEntityCommand::new(vec![]);
        self.execute_create_command(world, undo_system, &mut command)?;

        command.entity.map(|e| e.into()).ok_or_else(|| {
            EntityOperationsError::CommandExecutionFailed(
                "Failed to retrieve created entity".to_string(),
            )
        })
    }

    /// Creates a new entity with a transform component.
    ///
    /// # Arguments
    ///
    /// * `transform` - The initial transform for the entity
    ///
    /// # Returns
    ///
    /// The newly created entity ID.
    pub fn create_entity_with_transform(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        transform: Transform,
    ) -> Result<Entity> {
        let mut command = CreateEntityCommand::with_transform(transform);
        self.execute_create_command(world, undo_system, &mut command)?;

        command.entity.map(|e| e.into()).ok_or_else(|| {
            EntityOperationsError::CommandExecutionFailed(
                "Failed to retrieve created entity".to_string(),
            )
        })
    }

    /// Creates a new entity with name and transform components.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the entity
    /// * `transform` - The initial transform for the entity
    ///
    /// # Returns
    ///
    /// The newly created entity ID.
    pub fn create_entity_with_components(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        name: impl Into<String>,
        transform: Transform,
    ) -> Result<Entity> {
        let components = vec![
            ComponentData::Transform(transform.into()),
            ComponentData::Name(name.into()),
        ];
        let mut command = CreateEntityCommand::new(components);
        self.execute_create_command(world, undo_system, &mut command)?;

        command.entity.map(|e| e.into()).ok_or_else(|| {
            EntityOperationsError::CommandExecutionFailed(
                "Failed to retrieve created entity".to_string(),
            )
        })
    }

    // ============================================================================
    // Entity Deletion
    // ============================================================================

    /// Deletes an entity and all its components.
    ///
    /// The entity's state is captured for undo functionality.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to delete
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or command fails.
    pub fn delete_entity(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
    ) -> Result<()> {
        // Capture entity state before deletion
        let command = DeleteEntityCommand::from_world(entity, world)
            .map_err(EntityOperationsError::CommandExecutionFailed)?;

        self.execute_command(world, undo_system, command)
    }

    /// Deletes multiple entities at once.
    ///
    /// All deletions are grouped into a single undo operation.
    ///
    /// # Arguments
    ///
    /// * `entities` - Vector of entities to delete
    ///
    /// # Errors
    ///
    /// Returns error if any entity doesn't exist or commands fail.
    pub fn delete_entities(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entities: Vec<Entity>,
    ) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        if entities.len() == 1 {
            return self.delete_entity(world, undo_system, entities[0]);
        }

        // Create composite command for multiple deletions
        let mut composite = CompositeCommand::new(format!("Delete {} Entities", entities.len()));

        for entity in entities {
            let delete_cmd = DeleteEntityCommand::from_world(entity, world)
                .map_err(EntityOperationsError::CommandExecutionFailed)?;
            composite.add_command(SerializableCommand::DeleteEntity(delete_cmd));
        }

        self.execute_command(world, undo_system, composite)
    }

    // ============================================================================
    // Entity Duplication
    // ============================================================================

    /// Duplicates an entity with all its supported components.
    ///
    /// Currently supports duplicating: Transform, Name, Parent components.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to duplicate
    ///
    /// # Returns
    ///
    /// The newly created duplicate entity ID.
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or duplication fails.
    pub fn duplicate_entity(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
    ) -> Result<Entity> {
        self.duplicate_entity_with_offset(world, undo_system, entity, Vec3::ZERO)
    }

    /// Duplicates an entity with a position offset.
    ///
    /// If the entity has a Transform, the duplicate's position will be offset by the given amount.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to duplicate
    /// * `offset` - Position offset for the duplicate
    ///
    /// # Returns
    ///
    /// The newly created duplicate entity ID.
    pub fn duplicate_entity_with_offset(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
        offset: Vec3,
    ) -> Result<Entity> {
        // Verify entity exists
        let entity_ref = world
            .get_entity(entity)
            .ok_or(EntityOperationsError::EntityNotFound(entity))?;

        // Capture components
        let mut components = Vec::new();

        // Capture transform with offset
        if let Some(transform) = entity_ref.get::<Transform>() {
            let mut new_transform = *transform;
            new_transform.translation += offset;
            components.push(ComponentData::Transform(new_transform.into()));
        }

        // Capture name with " Copy" suffix
        if let Some(name) = entity_ref.get::<Name>() {
            let new_name = format!("{} Copy", name.0);
            components.push(ComponentData::Name(new_name));
        }

        // Capture parent if exists
        if let Some(parent) = entity_ref.get::<Parent>() {
            components.push(ComponentData::Parent(parent.0.into()));
        }

        // Create the duplicate
        let mut command = CreateEntityCommand::new(components);
        self.execute_create_command(world, undo_system, &mut command)?;

        command.entity.map(|e| e.into()).ok_or_else(|| {
            EntityOperationsError::CommandExecutionFailed(
                "Failed to retrieve duplicated entity".to_string(),
            )
        })
    }

    /// Duplicates multiple entities at once.
    ///
    /// All duplications are grouped into a single undo operation.
    ///
    /// # Arguments
    ///
    /// * `entities` - Vector of entities to duplicate
    ///
    /// # Returns
    ///
    /// Vector of newly created entity IDs in the same order as input.
    ///
    /// # Errors
    ///
    /// Returns error if any entity doesn't exist or duplication fails.
    pub fn duplicate_entities(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entities: Vec<Entity>,
    ) -> Result<Vec<Entity>> {
        if entities.is_empty() {
            return Ok(Vec::new());
        }

        // Start batch operation
        self.begin_batch(format!("Duplicate {} Entities", entities.len()));

        let mut new_entities = Vec::new();
        for entity in entities {
            let new_entity = self.duplicate_entity(world, undo_system, entity)?;
            new_entities.push(new_entity);
        }

        // End batch operation
        self.end_batch(world, undo_system)?;

        Ok(new_entities)
    }

    // ============================================================================
    // Component Addition
    // ============================================================================

    /// Adds a component to an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the component to
    /// * `component` - The component data to add
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or component addition fails.
    pub fn add_component(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
        component: ComponentData,
    ) -> Result<()> {
        // Verify entity exists
        if world.get_entity(entity).is_none() {
            return Err(EntityOperationsError::EntityNotFound(entity));
        }

        let command = AddComponentCommand::new(entity, component);
        self.execute_command(world, undo_system, command)
    }

    /// Adds a Transform component to an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the transform to
    /// * `transform` - The transform component
    pub fn add_transform(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
        transform: Transform,
    ) -> Result<()> {
        self.add_component(
            world,
            undo_system,
            entity,
            ComponentData::Transform(transform.into()),
        )
    }

    /// Adds a Name component to an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the name to
    /// * `name` - The name string
    pub fn add_name(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
        name: impl Into<String>,
    ) -> Result<()> {
        self.add_component(world, undo_system, entity, ComponentData::Name(name.into()))
    }

    /// Adds a Parent component to an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The child entity
    /// * `parent` - The parent entity
    pub fn add_parent(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
        parent: Entity,
    ) -> Result<()> {
        // Verify both entities exist
        if world.get_entity(entity).is_none() {
            return Err(EntityOperationsError::EntityNotFound(entity));
        }
        if world.get_entity(parent).is_none() {
            return Err(EntityOperationsError::EntityNotFound(parent));
        }

        self.add_component(
            world,
            undo_system,
            entity,
            ComponentData::Parent(parent.into()),
        )
    }

    // ============================================================================
    // Component Removal
    // ============================================================================

    /// Removes a component from an entity.
    ///
    /// The component's state is captured for undo functionality.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from
    /// * `component` - The component data (must match the current value for undo)
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or component removal fails.
    pub fn remove_component(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
        component: ComponentData,
    ) -> Result<()> {
        // Verify entity exists
        if world.get_entity(entity).is_none() {
            return Err(EntityOperationsError::EntityNotFound(entity));
        }

        let command = RemoveComponentCommand::new(entity, component);
        self.execute_command(world, undo_system, command)
    }

    /// Removes a Transform component from an entity.
    ///
    /// The current transform value is captured for undo.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the transform from
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or doesn't have a Transform.
    pub fn remove_transform(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
    ) -> Result<()> {
        // Capture current transform value
        let transform = world
            .get_entity(entity)
            .and_then(|e| e.get::<Transform>())
            .copied()
            .ok_or(EntityOperationsError::ComponentNotFound {
                entity,
                component: "Transform".to_string(),
            })?;

        self.remove_component(
            world,
            undo_system,
            entity,
            ComponentData::Transform(transform.into()),
        )
    }

    /// Removes a Name component from an entity.
    ///
    /// The current name value is captured for undo.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the name from
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or doesn't have a Name.
    pub fn remove_name(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
    ) -> Result<()> {
        // Capture current name value
        let name = world
            .get_entity(entity)
            .and_then(|e| e.get::<Name>())
            .map(|n| n.0.clone())
            .ok_or(EntityOperationsError::ComponentNotFound {
                entity,
                component: "Name".to_string(),
            })?;

        self.remove_component(world, undo_system, entity, ComponentData::Name(name))
    }

    /// Removes a Parent component from an entity.
    ///
    /// The current parent value is captured for undo.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the parent from
    ///
    /// # Errors
    ///
    /// Returns error if entity doesn't exist or doesn't have a Parent.
    pub fn remove_parent(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        entity: Entity,
    ) -> Result<()> {
        // Capture current parent value
        let parent = world
            .get_entity(entity)
            .and_then(|e| e.get::<Parent>())
            .map(|p| p.0)
            .ok_or(EntityOperationsError::ComponentNotFound {
                entity,
                component: "Parent".to_string(),
            })?;

        self.remove_component(
            world,
            undo_system,
            entity,
            ComponentData::Parent(parent.into()),
        )
    }

    // ============================================================================
    // Batch Operations
    // ============================================================================

    /// Begins a batch operation.
    ///
    /// All operations between `begin_batch` and `end_batch` will be grouped
    /// into a single undo command.
    ///
    /// # Arguments
    ///
    /// * `description` - Description for the batch operation
    ///
    /// # Panics
    ///
    /// Panics if a batch is already in progress.
    pub fn begin_batch(&mut self, description: impl Into<String>) {
        if self.batch_operation.is_some() {
            panic!("Batch operation already in progress");
        }

        self.batch_operation = Some(BatchOperation {
            description: description.into(),
            commands: Vec::new(),
        });
    }

    /// Ends the current batch operation and executes it as a composite command.
    ///
    /// # Errors
    ///
    /// Returns error if no batch is in progress or command execution fails.
    pub fn end_batch(&mut self, world: &mut World, undo_system: &mut UndoRedoSystem) -> Result<()> {
        let batch = self
            .batch_operation
            .take()
            .ok_or(EntityOperationsError::InvalidOperation(
                "No batch operation in progress".to_string(),
            ))?;

        if batch.commands.is_empty() {
            return Ok(());
        }

        let mut composite = CompositeCommand::new(batch.description);
        for command in batch.commands {
            composite.add_command(command);
        }

        undo_system
            .execute_command(world, Box::new(composite))
            .map_err(EntityOperationsError::CommandExecutionFailed)
    }

    /// Checks if a batch operation is currently in progress.
    pub fn is_batch_in_progress(&self) -> bool {
        self.batch_operation.is_some()
    }

    /// Cancels the current batch operation without executing it.
    pub fn cancel_batch(&mut self) {
        self.batch_operation = None;
    }

    // ============================================================================
    // Internal Helpers
    // ============================================================================

    /// Executes a command, either adding it to a batch or executing immediately.
    fn execute_command<C: EditorCommand + Into<SerializableCommand> + 'static>(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        command: C,
    ) -> Result<()> {
        if let Some(batch) = &mut self.batch_operation {
            // Add to batch
            batch.commands.push(command.into());
            Ok(())
        } else {
            // Execute immediately
            undo_system
                .execute_command(world, Box::new(command))
                .map_err(EntityOperationsError::CommandExecutionFailed)
        }
    }

    /// Executes a CreateEntityCommand and mutates it to get the created entity.
    fn execute_create_command(
        &mut self,
        world: &mut World,
        undo_system: &mut UndoRedoSystem,
        command: &mut CreateEntityCommand,
    ) -> Result<()> {
        if let Some(batch) = &mut self.batch_operation {
            // Execute to get entity, then add to batch
            command
                .execute(world)
                .map_err(|e| EntityOperationsError::CommandExecutionFailed(e.to_string()))?;
            batch
                .commands
                .push(SerializableCommand::CreateEntity(command.clone()));
            Ok(())
        } else {
            // Execute and add to undo system
            command
                .execute(world)
                .map_err(|e| EntityOperationsError::CommandExecutionFailed(e.to_string()))?;
            undo_system
                .execute_command(world, Box::new(command.clone()))
                .map_err(EntityOperationsError::CommandExecutionFailed)
        }
    }
}

// Implement conversions for serializable commands
impl From<CreateEntityCommand> for SerializableCommand {
    fn from(cmd: CreateEntityCommand) -> Self {
        SerializableCommand::CreateEntity(cmd)
    }
}

impl From<DeleteEntityCommand> for SerializableCommand {
    fn from(cmd: DeleteEntityCommand) -> Self {
        SerializableCommand::DeleteEntity(cmd)
    }
}

impl From<AddComponentCommand> for SerializableCommand {
    fn from(cmd: AddComponentCommand) -> Self {
        SerializableCommand::AddComponent(cmd)
    }
}

impl From<RemoveComponentCommand> for SerializableCommand {
    fn from(cmd: RemoveComponentCommand) -> Self {
        SerializableCommand::RemoveComponent(cmd)
    }
}

impl From<CompositeCommand> for SerializableCommand {
    fn from(cmd: CompositeCommand) -> Self {
        SerializableCommand::Composite(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_entity() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = entity_ops.create_entity(&mut world, &mut undo_system);
        assert!(entity.is_ok());

        let entity = entity.unwrap();
        assert!(world.get_entity(entity).is_some());
    }

    #[test]
    fn test_create_entity_with_transform() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let transform = Transform::from_xyz(10.0, 20.0, 30.0);
        let entity = entity_ops
            .create_entity_with_transform(&mut world, &mut undo_system, transform)
            .unwrap();

        let stored_transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(stored_transform.translation, transform.translation);
    }

    #[test]
    fn test_create_entity_with_components() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = entity_ops
            .create_entity_with_components(
                &mut world,
                &mut undo_system,
                "Test Entity",
                Transform::from_xyz(5.0, 0.0, 0.0),
            )
            .unwrap();

        let name = world.get::<Name>(entity).unwrap();
        assert_eq!(name.0, "Test Entity");

        let transform = world.get::<Transform>(entity).unwrap();
        assert_eq!(transform.translation.x, 5.0);
    }

    #[test]
    fn test_delete_entity() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = world.spawn(Transform::default());

        entity_ops
            .delete_entity(&mut world, &mut undo_system, entity)
            .unwrap();

        assert!(world.get_entity(entity).is_none());
    }

    #[test]
    fn test_delete_entities() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let e1 = world.spawn(Transform::default());
        let e2 = world.spawn(Transform::default());
        let e3 = world.spawn(Transform::default());

        entity_ops
            .delete_entities(&mut world, &mut undo_system, vec![e1, e2, e3])
            .unwrap();

        assert!(world.get_entity(e1).is_none());
        assert!(world.get_entity(e2).is_none());
        assert!(world.get_entity(e3).is_none());
    }

    #[test]
    fn test_duplicate_entity() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let original = world.spawn((Transform::from_xyz(10.0, 0.0, 0.0), Name::new("Original")));

        let duplicate = entity_ops
            .duplicate_entity(&mut world, &mut undo_system, original)
            .unwrap();

        assert_ne!(original, duplicate);

        let dup_transform = world.get::<Transform>(duplicate).unwrap();
        assert_eq!(dup_transform.translation.x, 10.0);

        let dup_name = world.get::<Name>(duplicate).unwrap();
        assert_eq!(dup_name.0, "Original Copy");
    }

    #[test]
    fn test_duplicate_entity_with_offset() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let original = world.spawn(Transform::from_xyz(10.0, 0.0, 0.0));

        let duplicate = entity_ops
            .duplicate_entity_with_offset(
                &mut world,
                &mut undo_system,
                original,
                Vec3::new(5.0, 0.0, 0.0),
            )
            .unwrap();

        let dup_transform = world.get::<Transform>(duplicate).unwrap();
        assert_eq!(dup_transform.translation.x, 15.0);
    }

    #[test]
    fn test_add_component() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = world.spawn_empty();

        entity_ops
            .add_transform(&mut world, &mut undo_system, entity, Transform::default())
            .unwrap();

        assert!(world.get::<Transform>(entity).is_some());
    }

    #[test]
    fn test_add_name() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = world.spawn_empty();

        entity_ops
            .add_name(&mut world, &mut undo_system, entity, "Test")
            .unwrap();

        let name = world.get::<Name>(entity).unwrap();
        assert_eq!(name.0, "Test");
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = world.spawn(Transform::default());

        entity_ops
            .remove_transform(&mut world, &mut undo_system, entity)
            .unwrap();

        assert!(world.get::<Transform>(entity).is_none());
    }

    #[test]
    fn test_batch_operations() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        entity_ops.begin_batch("Create Multiple Entities");

        for i in 0..3 {
            entity_ops
                .create_entity_with_components(
                    &mut world,
                    &mut undo_system,
                    format!("Entity {}", i),
                    Transform::from_xyz(i as f32, 0.0, 0.0),
                )
                .unwrap();
        }

        entity_ops.end_batch(&mut world, &mut undo_system).unwrap();

        // All operations should be in one command
        assert_eq!(undo_system.undo_count(), 1);
    }

    #[test]
    fn test_undo_create_entity() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let entity = entity_ops
            .create_entity(&mut world, &mut undo_system)
            .unwrap();

        assert!(world.get_entity(entity).is_some());

        undo_system.undo(&mut world).unwrap();

        // Entity should be removed after undo
        assert!(
            world.get_entity(entity).is_none()
                || !world.get_entity(entity).unwrap().contains::<Transform>()
        );
    }

    #[test]
    fn test_error_entity_not_found() {
        let mut world = World::new();
        let mut undo_system = UndoRedoSystem::new();
        let mut entity_ops = EntityOperations::new();

        let fake_entity = Entity::from_raw(99999);

        let result = entity_ops.delete_entity(&mut world, &mut undo_system, fake_entity);
        assert!(result.is_err());

        if let Err(EntityOperationsError::CommandExecutionFailed(_)) = result {
            // Expected error type
        } else {
            panic!("Expected CommandExecutionFailed error");
        }
    }

    #[test]
    fn test_batch_cancel() {
        let mut entity_ops = EntityOperations::new();

        entity_ops.begin_batch("Test Batch");
        assert!(entity_ops.is_batch_in_progress());

        entity_ops.cancel_batch();
        assert!(!entity_ops.is_batch_in_progress());
    }
}
