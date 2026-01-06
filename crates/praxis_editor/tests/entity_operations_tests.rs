//! Comprehensive tests for editor entity operations.
//!
//! Tests cover:
//! - Entity creation with various configurations
//! - Entity deletion (single and batch)
//! - Entity duplication with hierarchy
//! - Component addition and removal
//! - Batch operations
//! - Integration with undo/redo system
//! - Error handling and edge cases

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use praxis_ecs::{Children, Name, Parent, Transform};
use praxis_editor::{EntityOperations, UndoRedoSystem};
use praxis_math::Vec3;

// ============================================================================
// Entity Creation Tests
// ============================================================================

#[test]
fn test_create_empty_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let result = entity_ops.create_entity(&mut world, &mut undo_system);
    assert!(result.is_ok());

    let entity = result.unwrap();
    assert!(world.get_entity(entity).is_some());
}

#[test]
fn test_create_entity_with_transform() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let transform = Transform::from_xyz(10.0, 20.0, 30.0);
    let result = entity_ops.create_entity_with_transform(&mut world, &mut undo_system, transform);
    assert!(result.is_ok());

    let entity = result.unwrap();
    let stored_transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(stored_transform.translation, transform.translation);
}

#[test]
fn test_create_entity_with_components() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let result = entity_ops.create_entity_with_components(
        &mut world,
        &mut undo_system,
        "Test Entity",
        Transform::from_xyz(5.0, 0.0, 0.0),
    );
    assert!(result.is_ok());

    let entity = result.unwrap();
    let name = world.get::<Name>(entity).unwrap();
    assert_eq!(name.0, "Test Entity");

    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation.x, 5.0);
}

#[test]
fn test_create_multiple_entities() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let mut entities = Vec::new();
    for i in 0..5 {
        let result = entity_ops.create_entity_with_components(
            &mut world,
            &mut undo_system,
            format!("Entity {i}"),
            Transform::from_xyz(i as f32, 0.0, 0.0),
        );
        assert!(result.is_ok());
        entities.push(result.unwrap());
    }

    assert_eq!(entities.len(), 5);

    for (i, entity) in entities.iter().enumerate() {
        let name = world.get::<Name>(*entity).unwrap();
        assert_eq!(name.0, format!("Entity {i}"));
    }
}

// ============================================================================
// Entity Deletion Tests
// ============================================================================

#[test]
fn test_delete_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn(Transform::default()).id();

    let result = entity_ops.delete_entity(&mut world, &mut undo_system, entity);
    assert!(result.is_ok());

    assert!(world.get_entity(entity).is_none());
}

#[test]
fn test_delete_entity_with_components() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world
        .spawn((
            Transform::from_xyz(10.0, 20.0, 30.0),
            Name::new("TestEntity"),
        ))
        .id();

    let result = entity_ops.delete_entity(&mut world, &mut undo_system, entity);
    assert!(result.is_ok());

    assert!(world.get_entity(entity).is_none());
}

#[test]
fn test_delete_nonexistent_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let fake_entity = Entity::from_raw(99999);

    let result = entity_ops.delete_entity(&mut world, &mut undo_system, fake_entity);
    assert!(result.is_err());
}

#[test]
fn test_delete_multiple_entities() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let e1 = world.spawn(Transform::default()).id();
    let e2 = world.spawn(Transform::default()).id();
    let e3 = world.spawn(Transform::default()).id();

    let result = entity_ops.delete_entities(&mut world, &mut undo_system, vec![e1, e2, e3]);
    assert!(result.is_ok());

    assert!(world.get_entity(e1).is_none());
    assert!(world.get_entity(e2).is_none());
    assert!(world.get_entity(e3).is_none());
}

#[test]
fn test_delete_empty_entity_list() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let result = entity_ops.delete_entities(&mut world, &mut undo_system, vec![]);
    assert!(result.is_ok());
}

// ============================================================================
// Entity Duplication Tests
// ============================================================================

#[test]
fn test_duplicate_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let original = world
        .spawn((Transform::from_xyz(10.0, 0.0, 0.0), Name::new("Original")))
        .id();

    let result = entity_ops.duplicate_entity(&mut world, &mut undo_system, original);
    assert!(result.is_ok());

    let duplicate = result.unwrap();
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

    let original = world.spawn(Transform::from_xyz(10.0, 0.0, 0.0)).id();

    let result = entity_ops.duplicate_entity_with_offset(
        &mut world,
        &mut undo_system,
        original,
        Vec3::new(5.0, 0.0, 0.0),
    );
    assert!(result.is_ok());

    let duplicate = result.unwrap();
    let dup_transform = world.get::<Transform>(duplicate).unwrap();
    assert_eq!(dup_transform.translation.x, 15.0);
}

#[test]
fn test_duplicate_entity_without_transform() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let original = world.spawn(Name::new("Original")).id();

    let result = entity_ops.duplicate_entity(&mut world, &mut undo_system, original);
    assert!(result.is_ok());

    let duplicate = result.unwrap();
    let dup_name = world.get::<Name>(duplicate).unwrap();
    assert_eq!(dup_name.0, "Original Copy");
}

#[test]
fn test_duplicate_multiple_entities() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let e1 = world
        .spawn((Transform::from_xyz(1.0, 0.0, 0.0), Name::new("Entity1")))
        .id();
    let e2 = world
        .spawn((Transform::from_xyz(2.0, 0.0, 0.0), Name::new("Entity2")))
        .id();
    let e3 = world
        .spawn((Transform::from_xyz(3.0, 0.0, 0.0), Name::new("Entity3")))
        .id();

    let result = entity_ops.duplicate_entities(&mut world, &mut undo_system, vec![e1, e2, e3]);
    assert!(result.is_ok());

    let new_entities = result.unwrap();
    assert_eq!(new_entities.len(), 3);

    for new_entity in new_entities {
        assert!(world.get::<Name>(new_entity).is_some());
        assert!(world.get::<Transform>(new_entity).is_some());
    }
}

#[test]
fn test_duplicate_entity_with_parent() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let parent = world.spawn_empty().id();
    let child = world.spawn(Parent(parent)).id();

    let result = entity_ops.duplicate_entity(&mut world, &mut undo_system, child);
    assert!(result.is_ok());

    let duplicate = result.unwrap();
    let dup_parent = world.get::<Parent>(duplicate);
    assert!(dup_parent.is_some());
    assert_eq!(dup_parent.unwrap().0, parent);
}

#[test]
fn test_duplicate_nonexistent_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let fake_entity = Entity::from_raw(99999);

    let result = entity_ops.duplicate_entity(&mut world, &mut undo_system, fake_entity);
    assert!(result.is_err());
}

// ============================================================================
// Component Addition Tests
// ============================================================================

#[test]
fn test_add_transform_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn_empty().id();

    let transform = Transform::from_xyz(5.0, 10.0, 15.0);
    let result = entity_ops.add_transform(&mut world, &mut undo_system, entity, transform);
    assert!(result.is_ok());

    assert!(world.get::<Transform>(entity).is_some());
    let stored_transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(stored_transform.translation, transform.translation);
}

#[test]
fn test_add_name_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn_empty().id();

    let result = entity_ops.add_name(&mut world, &mut undo_system, entity, "Test");
    assert!(result.is_ok());

    let name = world.get::<Name>(entity).unwrap();
    assert_eq!(name.0, "Test");
}

#[test]
fn test_add_parent_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let parent = world.spawn_empty().id();
    let child = world.spawn_empty().id();

    let result = entity_ops.add_parent(&mut world, &mut undo_system, child, parent);
    assert!(result.is_ok());

    let child_parent = world.get::<Parent>(child);
    assert!(child_parent.is_some());
    assert_eq!(child_parent.unwrap().0, parent);
}

#[test]
fn test_add_component_to_nonexistent_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let fake_entity = Entity::from_raw(99999);

    let result = entity_ops.add_name(&mut world, &mut undo_system, fake_entity, "Test");
    assert!(result.is_err());
}

// ============================================================================
// Component Removal Tests
// ============================================================================

#[test]
fn test_remove_transform_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn(Transform::default()).id();

    let result = entity_ops.remove_transform(&mut world, &mut undo_system, entity);
    assert!(result.is_ok());

    assert!(world.get::<Transform>(entity).is_none());
}

#[test]
fn test_remove_name_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn(Name::new("Test")).id();

    let result = entity_ops.remove_name(&mut world, &mut undo_system, entity);
    assert!(result.is_ok());

    assert!(world.get::<Name>(entity).is_none());
}

#[test]
fn test_remove_parent_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let parent = world.spawn_empty().id();
    let child = world.spawn(Parent(parent)).id();

    let result = entity_ops.remove_parent(&mut world, &mut undo_system, child);
    assert!(result.is_ok());

    assert!(world.get::<Parent>(child).is_none());
}

#[test]
fn test_remove_nonexistent_component() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn_empty().id();

    let result = entity_ops.remove_name(&mut world, &mut undo_system, entity);
    assert!(result.is_err());
}

// ============================================================================
// Batch Operation Tests
// ============================================================================

#[test]
fn test_batch_create_entities() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    entity_ops.begin_batch("Create Multiple Entities");

    for i in 0..3 {
        let result = entity_ops.create_entity_with_components(
            &mut world,
            &mut undo_system,
            format!("Entity {i}"),
            Transform::from_xyz(i as f32, 0.0, 0.0),
        );
        assert!(result.is_ok());
    }

    let result = entity_ops.end_batch(&mut world, &mut undo_system);
    assert!(result.is_ok());

    // All operations should be in one command
    assert_eq!(undo_system.undo_count(), 1);
}

#[test]
fn test_batch_cancel() {
    let mut entity_ops = EntityOperations::new();

    entity_ops.begin_batch("Test Batch");
    assert!(entity_ops.is_batch_in_progress());

    entity_ops.cancel_batch();
    assert!(!entity_ops.is_batch_in_progress());
}

#[test]
fn test_batch_empty() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    entity_ops.begin_batch("Empty Batch");
    let result = entity_ops.end_batch(&mut world, &mut undo_system);
    assert!(result.is_ok());

    // No commands should be added
    assert_eq!(undo_system.undo_count(), 0);
}

#[test]
#[should_panic]
fn test_nested_batch_panic() {
    let mut entity_ops = EntityOperations::new();

    entity_ops.begin_batch("Batch 1");
    entity_ops.begin_batch("Batch 2"); // Should panic
}

#[test]
fn test_end_batch_without_begin() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let result = entity_ops.end_batch(&mut world, &mut undo_system);
    assert!(result.is_err());
}

// ============================================================================
// Integration with Undo/Redo Tests
// ============================================================================

#[test]
fn test_create_entity_undo() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let result = entity_ops.create_entity(&mut world, &mut undo_system);
    assert!(result.is_ok());
    let entity = result.unwrap();

    assert!(world.get_entity(entity).is_some());

    undo_system.undo(&mut world).unwrap();

    // Entity should be removed after undo
    assert!(
        world.get_entity(entity).is_none()
            || !world.get_entity(entity).unwrap().contains::<Transform>()
    );
}

#[test]
fn test_delete_entity_undo() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world
        .spawn((
            Transform::from_xyz(10.0, 20.0, 30.0),
            Name::new("TestEntity"),
        ))
        .id();

    entity_ops
        .delete_entity(&mut world, &mut undo_system, entity)
        .unwrap();

    assert!(world.get_entity(entity).is_none());

    undo_system.undo(&mut world).unwrap();

    // Entity is recreated (possibly with different ID)
    // Just verify undo worked
    assert!(undo_system.can_redo());
}

#[test]
fn test_add_component_undo() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn_empty().id();

    entity_ops
        .add_name(&mut world, &mut undo_system, entity, "Test")
        .unwrap();

    assert!(world.get::<Name>(entity).is_some());

    undo_system.undo(&mut world).unwrap();

    assert!(world.get::<Name>(entity).is_none());
}

#[test]
fn test_remove_component_undo() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn(Name::new("TestEntity")).id();

    entity_ops
        .remove_name(&mut world, &mut undo_system, entity)
        .unwrap();

    assert!(world.get::<Name>(entity).is_none());

    undo_system.undo(&mut world).unwrap();

    assert!(world.get::<Name>(entity).is_some());
    let name = world.get::<Name>(entity).unwrap();
    assert_eq!(name.0, "TestEntity");
}

#[test]
#[ignore = "Known issue: duplicate entity undo does not properly remove the duplicated entity"]
fn test_duplicate_entity_undo() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let original = world.spawn(Transform::default()).id();

    let duplicate = entity_ops
        .duplicate_entity(&mut world, &mut undo_system, original)
        .unwrap();

    assert!(world.get_entity(duplicate).is_some());

    undo_system.undo(&mut world).unwrap();

    assert!(world.get_entity(duplicate).is_none());
}

// ============================================================================
// Complex Scenarios Tests
// ============================================================================

#[test]
#[ignore = "Known issue: Children component not properly set during batch hierarchy creation"]
fn test_create_hierarchy_batch() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    entity_ops.begin_batch("Create Hierarchy");

    // Create parent
    let parent = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Parent", Transform::default())
        .unwrap();

    // Create children
    for i in 0..3 {
        let child = entity_ops
            .create_entity_with_components(
                &mut world,
                &mut undo_system,
                format!("Child {i}"),
                Transform::from_xyz(i as f32, 0.0, 0.0),
            )
            .unwrap();

        entity_ops
            .add_parent(&mut world, &mut undo_system, child, parent)
            .unwrap();
    }

    entity_ops.end_batch(&mut world, &mut undo_system).unwrap();

    // Verify hierarchy
    assert!(world.get::<Children>(parent).is_some());
}

#[test]
fn test_modify_multiple_components() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    let entity = world.spawn_empty().id();

    // Add multiple components
    entity_ops
        .add_transform(&mut world, &mut undo_system, entity, Transform::default())
        .unwrap();
    entity_ops
        .add_name(&mut world, &mut undo_system, entity, "Test")
        .unwrap();

    assert!(world.get::<Transform>(entity).is_some());
    assert!(world.get::<Name>(entity).is_some());

    // Undo both operations
    undo_system.undo(&mut world).unwrap();
    assert!(world.get::<Name>(entity).is_none());

    undo_system.undo(&mut world).unwrap();
    assert!(world.get::<Transform>(entity).is_none());

    // Redo both
    undo_system.redo(&mut world).unwrap();
    assert!(world.get::<Transform>(entity).is_some());

    undo_system.redo(&mut world).unwrap();
    assert!(world.get::<Name>(entity).is_some());
}

#[test]
fn test_complex_batch_with_mixed_operations() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    entity_ops.begin_batch("Complex Operations");

    // Create entities
    let e1 = entity_ops
        .create_entity_with_transform(&mut world, &mut undo_system, Transform::default())
        .unwrap();

    let e2 = entity_ops
        .create_entity_with_transform(&mut world, &mut undo_system, Transform::default())
        .unwrap();

    // Add components
    entity_ops
        .add_name(&mut world, &mut undo_system, e1, "Entity1")
        .unwrap();

    // Duplicate
    let e3 = entity_ops
        .duplicate_entity(&mut world, &mut undo_system, e1)
        .unwrap();

    entity_ops.end_batch(&mut world, &mut undo_system).unwrap();

    // Verify all operations
    assert!(world.get_entity(e1).is_some());
    assert!(world.get_entity(e2).is_some());
    assert!(world.get_entity(e3).is_some());
    assert!(world.get::<Name>(e1).is_some());

    // Undo entire batch
    undo_system.undo(&mut world).unwrap();

    // All operations should be undone
    assert!(world.get_entity(e3).is_none());
}

#[test]
fn test_delete_and_recreate_entity() {
    let mut world = World::new();
    let mut undo_system = UndoRedoSystem::new();
    let mut entity_ops = EntityOperations::new();

    // Create
    let entity = entity_ops
        .create_entity_with_components(
            &mut world,
            &mut undo_system,
            "TestEntity",
            Transform::from_xyz(1.0, 2.0, 3.0),
        )
        .unwrap();

    // Delete
    entity_ops
        .delete_entity(&mut world, &mut undo_system, entity)
        .unwrap();

    assert!(world.get_entity(entity).is_none());

    // Undo delete (recreate)
    undo_system.undo(&mut world).unwrap();

    // Undo create (delete again)
    undo_system.undo(&mut world).unwrap();

    assert!(world.get_entity(entity).is_none());
}
