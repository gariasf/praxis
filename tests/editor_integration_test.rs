//! Integration tests for editor functionality.
//!
//! These tests verify that different editor panels and systems work together correctly,
//! including hierarchy panel operations, inspector component editing, viewport rendering,
//! drag-and-drop asset instantiation, and editor camera controls.

#![cfg(feature = "editor")]

use praxis_audio::AudioSource;
use praxis_ecs::{
    Children, GlobalTransform, MaterialHandle, MeshHandle, Name, Parent, PerspectiveProjection,
    Transform, World,
};
use praxis_editor::{
    AssetEntry, AssetType, DragDropPayload, DragDropSystem, EditorCamera, EditorCameraController,
    EntityOperations, HierarchyPanel, InspectorPanel, Selectable, SelectionMode, SelectionSystem,
    UndoRedoSystem, ViewportPanel,
};
use praxis_input::InputState;
use praxis_math::{Quat, Vec3};
use praxis_physics::{Collider, Mass, PhysicsVelocity, RigidBody};
use std::collections::HashSet;
use std::path::PathBuf;
use winit::keyboard::KeyCode;

// ============================================================================
// HIERARCHY PANEL TESTS
// ============================================================================

/// Test hierarchy panel entity tree operations with parent-child relationships.
#[test]
fn test_hierarchy_panel_entity_tree_operations() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(UndoRedoSystem::new());

    let mut hierarchy_panel = HierarchyPanel::new();

    // Test entity creation
    let entity_ops = EntityOperations::new();
    let mut undo_system = UndoRedoSystem::new();

    let parent_entity = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Parent", vec![])
        .expect("Failed to create parent entity");

    let child_entity = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Child", vec![])
        .expect("Failed to create child entity");

    // Verify entities exist
    assert!(world.inner().get_entity(parent_entity).is_ok());
    assert!(world.inner().get_entity(child_entity).is_ok());

    // Verify names
    assert_eq!(world.get::<Name>(parent_entity).unwrap().as_str(), "Parent");
    assert_eq!(world.get::<Name>(child_entity).unwrap().as_str(), "Child");

    // Verify expansion state tracking
    hierarchy_panel.expand_to_entity(&world, parent_entity);
    hierarchy_panel.expand_to_entity(&world, child_entity);

    // Test entity deletion
    let result = entity_ops.delete_entities(&mut world, &mut undo_system, vec![child_entity]);
    assert!(result.is_ok());
    assert!(world.inner().get_entity(child_entity).is_err());

    // Parent should still exist
    assert!(world.inner().get_entity(parent_entity).is_ok());
}

/// Test hierarchy panel drag-and-drop reparenting operations.
#[test]
fn test_hierarchy_panel_reparenting() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(UndoRedoSystem::new());

    let entity_ops = EntityOperations::new();
    let mut undo_system = UndoRedoSystem::new();

    // Create parent and child entities
    let parent1 = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Parent1", vec![])
        .expect("Failed to create parent1");

    let parent2 = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Parent2", vec![])
        .expect("Failed to create parent2");

    let child = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Child", vec![])
        .expect("Failed to create child");

    // Set child's parent to parent1
    world.entity_mut(child).insert(Parent(parent1));

    // Add child to parent1's children
    if let Ok(parent_entity) = world.inner_mut().get_entity_mut(parent1) {
        if let Some(mut children) = parent_entity.get_mut::<Children>() {
            children.0.push(child);
        } else {
            world.entity_mut(parent1).insert(Children(vec![child]));
        }
    }

    // Verify parent-child relationship
    assert_eq!(world.get::<Parent>(child).unwrap().0, parent1);
    assert!(world.get::<Children>(parent1).unwrap().0.contains(&child));

    // Reparent to parent2 by updating Parent component
    world.entity_mut(child).insert(Parent(parent2));

    // Update parent1's children list
    if let Ok(parent_entity) = world.inner_mut().get_entity_mut(parent1) {
        if let Some(mut children) = parent_entity.get_mut::<Children>() {
            children.0.retain(|&e| e != child);
        }
    }

    // Add to parent2's children list
    if let Ok(parent_entity) = world.inner_mut().get_entity_mut(parent2) {
        if let Some(mut children) = parent_entity.get_mut::<Children>() {
            children.0.push(child);
        } else {
            world.entity_mut(parent2).insert(Children(vec![child]));
        }
    }

    // Verify new parent-child relationship
    assert_eq!(world.get::<Parent>(child).unwrap().0, parent2);
    assert!(world.get::<Children>(parent2).unwrap().0.contains(&child));
    assert!(!world
        .get::<Children>(parent1)
        .map(|c| c.0.contains(&child))
        .unwrap_or(false));
}

/// Test hierarchy panel expansion/collapse functionality.
#[test]
fn test_hierarchy_panel_expansion() {
    let mut world = World::new();
    let mut hierarchy_panel = HierarchyPanel::new();

    let entity_ops = EntityOperations::new();
    let mut undo_system = UndoRedoSystem::new();

    // Create entities
    let entity1 = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Entity1", vec![])
        .unwrap();
    let entity2 = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Entity2", vec![])
        .unwrap();

    // Test expand all
    hierarchy_panel.expand_all(&world);

    // Test collapse all
    hierarchy_panel.collapse_all();

    // Test expand to specific entity
    hierarchy_panel.expand_to_entity(&world, entity1);
    hierarchy_panel.expand_to_entity(&world, entity2);
}

// ============================================================================
// INSPECTOR PANEL TESTS
// ============================================================================

/// Test inspector panel component editing functionality.
#[test]
fn test_inspector_panel_component_editing() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(UndoRedoSystem::new());

    let mut inspector_panel = InspectorPanel::new();

    // Create entity with various components
    let entity = world.spawn((
        Name::new("Test Entity"),
        Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        GlobalTransform::default(),
        Selectable,
    ));

    // Select the entity
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity, SelectionMode::Replace);

    // Verify entity is selected
    assert!(world
        .get_resource::<SelectionSystem>()
        .unwrap()
        .is_selected(entity));

    // Test Transform component is present
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));

    // Modify transform
    world.entity_mut(entity).insert(Transform {
        translation: Vec3::new(5.0, 6.0, 7.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    // Verify modification
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation, Vec3::new(5.0, 6.0, 7.0));
}

/// Test inspector panel with multiple component types.
#[test]
fn test_inspector_panel_multiple_components() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    // Create entity with multiple components
    let entity = world.spawn((
        Name::new("Multi-Component Entity"),
        Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        GlobalTransform::default(),
        MeshHandle::new("cube".to_string()),
        MaterialHandle::new("default".to_string()),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        PhysicsVelocity {
            linear: Vec3::new(0.0, -1.0, 0.0),
            angular: Vec3::ZERO,
        },
        Mass {
            mass: 1.0,
            angular_inertia: 1.0,
        },
        Selectable,
    ));

    // Select the entity
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity, SelectionMode::Replace);

    // Verify all components are present
    assert!(world.get::<Name>(entity).is_some());
    assert!(world.get::<Transform>(entity).is_some());
    assert!(world.get::<MeshHandle>(entity).is_some());
    assert!(world.get::<MaterialHandle>(entity).is_some());
    assert!(world.get::<RigidBody>(entity).is_some());
    assert!(world.get::<Collider>(entity).is_some());
    assert!(world.get::<PhysicsVelocity>(entity).is_some());
    assert!(world.get::<Mass>(entity).is_some());

    // Verify component values
    assert_eq!(
        world.get::<Name>(entity).unwrap().as_str(),
        "Multi-Component Entity"
    );
    assert_eq!(
        world.get::<Transform>(entity).unwrap().translation,
        Vec3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(world.get::<MeshHandle>(entity).unwrap().id(), "cube");
    assert!(world.get::<RigidBody>(entity).unwrap().is_dynamic());
    assert_eq!(world.get::<Mass>(entity).unwrap().mass, 1.0);
}

/// Test inspector panel audio component editing.
#[test]
fn test_inspector_panel_audio_component() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    let entity = world.spawn((
        Name::new("Audio Entity"),
        Transform::default(),
        AudioSource::new("test_audio.wav".to_string()),
        Selectable,
    ));

    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity, SelectionMode::Replace);

    // Verify audio source
    let audio = world.get::<AudioSource>(entity).unwrap();
    assert_eq!(audio.path, "test_audio.wav");
    assert_eq!(audio.volume, 1.0);
    assert!(!audio.spatial);

    // Modify audio properties
    world.entity_mut(entity).insert(AudioSource {
        path: "modified_audio.wav".to_string(),
        volume: 0.5,
        spatial: true,
        looping: true,
        max_distance: 100.0,
        reference_distance: 1.0,
        state: praxis_audio::AudioState::Stopped,
    });

    let audio = world.get::<AudioSource>(entity).unwrap();
    assert_eq!(audio.path, "modified_audio.wav");
    assert_eq!(audio.volume, 0.5);
    assert!(audio.spatial);
    assert!(audio.looping);
}

/// Test inspector panel camera component editing.
#[test]
fn test_inspector_panel_camera_component() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    let entity = world.spawn((
        Name::new("Camera Entity"),
        Transform::default(),
        PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        },
        Selectable,
    ));

    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity, SelectionMode::Replace);

    // Verify camera properties
    let camera = world.get::<PerspectiveProjection>(entity).unwrap();
    assert!((camera.fov.to_degrees() - 60.0).abs() < 0.01);
    assert!((camera.aspect_ratio - (16.0 / 9.0)).abs() < 0.01);
    assert!((camera.near - 0.1).abs() < 0.01);
    assert!((camera.far - 1000.0).abs() < 0.01);

    // Modify camera
    world.entity_mut(entity).insert(PerspectiveProjection {
        fov: 90.0_f32.to_radians(),
        aspect_ratio: 1.0,
        near: 0.01,
        far: 500.0,
    });

    let camera = world.get::<PerspectiveProjection>(entity).unwrap();
    assert!((camera.fov.to_degrees() - 90.0).abs() < 0.01);
    assert!((camera.aspect_ratio - 1.0).abs() < 0.01);
}

// ============================================================================
// VIEWPORT PANEL TESTS
// ============================================================================

/// Test viewport panel initialization and camera setup.
#[test]
fn test_viewport_panel_initialization() {
    let mut viewport_panel = ViewportPanel::new();

    // Verify initial state
    assert_eq!(viewport_panel.camera_distance(), 10.0);
    assert!(viewport_panel.show_grid());
    assert!(viewport_panel.show_gizmos());
    assert_eq!(viewport_panel.camera_target(), Vec3::ZERO);
}

/// Test viewport panel camera controls and transformations.
#[test]
fn test_viewport_panel_camera_controls() {
    let mut viewport_panel = ViewportPanel::new();

    // Test camera distance control
    viewport_panel.set_camera_distance(20.0);
    viewport_panel.update_camera(1.0); // Large delta for immediate update
    assert!((viewport_panel.camera_distance() - 20.0).abs() < 0.01);

    // Test camera target control
    let target = Vec3::new(5.0, 3.0, -2.0);
    viewport_panel.set_camera_target(target);
    viewport_panel.update_camera(1.0);
    assert!((viewport_panel.camera_target() - target).length() < 0.01);

    // Test camera position computation
    let position = viewport_panel.compute_camera_transform();
    assert!(position.translation.length() > 0.0);

    // Test camera reset
    viewport_panel.reset_camera();
    viewport_panel.update_camera(1.0);
    assert!((viewport_panel.camera_distance() - 10.0).abs() < 0.01);
    assert!((viewport_panel.camera_target() - Vec3::ZERO).length() < 0.01);
}

/// Test viewport panel rendering integration with selection.
#[test]
fn test_viewport_panel_selection_integration() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    let mut viewport_panel = ViewportPanel::new();

    // Create selectable entities
    let entity1 = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
        MeshHandle::new("cube".to_string()),
        Selectable,
    ));

    let entity2 = world.spawn((
        Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        GlobalTransform::default(),
        MeshHandle::new("sphere".to_string()),
        Selectable,
    ));

    // Select entity1
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity1, SelectionMode::Replace);

    // Focus camera on selection
    viewport_panel.focus_on_selection(&mut world);
    viewport_panel.update_camera(1.0);

    // Camera should be looking at entity1's position
    let target = viewport_panel.camera_target();
    assert!((target - Vec3::ZERO).length() < 0.1);

    // Add entity2 to selection
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity2, SelectionMode::Add);

    // Focus on multi-selection
    viewport_panel.focus_on_selection(&mut world);
    viewport_panel.update_camera(1.0);

    // Camera should be looking at center of both entities
    let target = viewport_panel.camera_target();
    // Center should be around (2.5, 0.0, 0.0)
    assert!((target.x - 2.5).abs() < 1.0);
}

/// Test viewport panel camera presets.
#[test]
fn test_viewport_panel_camera_presets() {
    use praxis_editor::panels::CameraPreset;

    let mut viewport_panel = ViewportPanel::new();

    // Test top view preset
    viewport_panel.apply_camera_preset(CameraPreset::Top);
    viewport_panel.update_camera(1.0);
    assert!(viewport_panel.camera_settings().orthographic);

    // Test front view preset
    viewport_panel.apply_camera_preset(CameraPreset::Front);
    viewport_panel.update_camera(1.0);
    assert!(viewport_panel.camera_settings().orthographic);

    // Test right view preset
    viewport_panel.apply_camera_preset(CameraPreset::Right);
    viewport_panel.update_camera(1.0);
    assert!(viewport_panel.camera_settings().orthographic);

    // Test perspective view preset
    viewport_panel.apply_camera_preset(CameraPreset::Perspective);
    viewport_panel.update_camera(1.0);
    assert!(!viewport_panel.camera_settings().orthographic);
}

/// Test viewport panel grid and gizmo visibility.
#[test]
fn test_viewport_panel_visibility_toggles() {
    let mut viewport_panel = ViewportPanel::new();

    // Test grid visibility
    assert!(viewport_panel.show_grid());
    viewport_panel.set_show_grid(false);
    assert!(!viewport_panel.show_grid());
    viewport_panel.set_show_grid(true);
    assert!(viewport_panel.show_grid());

    // Test gizmo visibility
    assert!(viewport_panel.show_gizmos());
    viewport_panel.set_show_gizmos(false);
    assert!(!viewport_panel.show_gizmos());
    viewport_panel.set_show_gizmos(true);
    assert!(viewport_panel.show_gizmos());
}

/// Test viewport panel camera entity tracking.
#[test]
fn test_viewport_panel_camera_entity() {
    let mut world = World::new();
    let mut viewport_panel = ViewportPanel::new();

    // Create viewport camera entity
    let camera_entity = world.spawn((
        Transform::default(),
        PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        },
        EditorCamera,
    ));

    viewport_panel.set_camera_entity(camera_entity);
    assert_eq!(viewport_panel.camera_entity(), Some(camera_entity));
}

// ============================================================================
// DRAG-AND-DROP TESTS
// ============================================================================

/// Test drag-and-drop asset instantiation workflow.
#[test]
fn test_drag_drop_asset_instantiation() {
    let mut world = World::new();
    world.insert_resource(DragDropSystem::new());

    // Create asset entry
    let asset_entry = AssetEntry {
        name: "test_model.obj".to_string(),
        path: PathBuf::from("assets/models/test_model.obj"),
        is_directory: false,
        asset_type: AssetType::Model,
        modified: None,
        thumbnail: None,
    };

    // Start drag operation
    let payload = DragDropPayload::from_asset(&asset_entry);
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .start_drag(payload.clone());

    // Verify drag is active
    assert!(world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .is_dragging());

    // Complete drop
    let dropped = world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .complete_drop();

    assert!(dropped.is_some());
    assert!(world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .drop_just_completed());

    // Verify dropped asset path
    if let Some(path) = dropped.unwrap().as_asset_path() {
        assert_eq!(path, &PathBuf::from("assets/models/test_model.obj"));
    } else {
        panic!("Expected asset path in dropped payload");
    }
}

/// Test drag-and-drop entity hierarchy operations.
#[test]
fn test_drag_drop_entity_hierarchy() {
    let mut world = World::new();
    world.insert_resource(DragDropSystem::new());

    // Create entities
    let entity1 = world.spawn((Name::new("Entity1"), Transform::default()));
    let entity2 = world.spawn((Name::new("Entity2"), Transform::default()));

    // Start drag operation with entity
    let payload = DragDropPayload::Entity(entity1);
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .start_drag(payload);

    // Verify drag state
    assert!(world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .is_dragging());

    // Peek at payload
    let current = world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .current_payload();
    assert!(current.is_some());
    assert_eq!(current.unwrap().as_entity(), Some(entity1));

    // Complete drop
    let dropped = world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .complete_drop();

    assert!(dropped.is_some());
    assert_eq!(dropped.unwrap().as_entity(), Some(entity1));
}

/// Test drag-and-drop cancellation.
#[test]
fn test_drag_drop_cancellation() {
    let mut world = World::new();
    world.insert_resource(DragDropSystem::new());

    let payload = DragDropPayload::FilePath(PathBuf::from("test.txt"));
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .start_drag(payload);

    assert!(world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .is_dragging());

    // Cancel drag
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .cancel_drag();

    assert!(!world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .is_dragging());
    assert!(world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .current_payload()
        .is_none());
}

/// Test drag-and-drop frame reset.
#[test]
fn test_drag_drop_frame_reset() {
    let mut world = World::new();
    world.insert_resource(DragDropSystem::new());

    let payload = DragDropPayload::FilePath(PathBuf::from("test.txt"));
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .start_drag(payload);

    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .complete_drop();

    assert!(world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .drop_just_completed());

    // Reset frame state
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .reset_frame();

    assert!(!world
        .get_resource::<DragDropSystem>()
        .unwrap()
        .drop_just_completed());
}

// ============================================================================
// EDITOR CAMERA CONTROLLER TESTS
// ============================================================================

/// Test editor camera controller initialization and basic controls.
#[test]
fn test_editor_camera_controller_initialization() {
    let controller = EditorCameraController::new();

    assert_eq!(controller.target(), Vec3::ZERO);
    assert_eq!(controller.distance(), 10.0);

    let (yaw, pitch) = controller.angles();
    assert!(yaw.abs() > 0.0);
    assert!(pitch.abs() > 0.0);
}

/// Test editor camera controller target and distance controls.
#[test]
fn test_editor_camera_controller_target_distance() {
    let mut controller = EditorCameraController::new();

    // Set target
    let target = Vec3::new(10.0, 5.0, -3.0);
    controller.set_target(target);
    controller.update(1.0); // Complete interpolation
    assert_eq!(controller.target(), target);

    // Set distance
    controller.set_distance(20.0);
    controller.update(1.0);
    assert_eq!(controller.distance(), 20.0);

    // Test distance clamping
    controller.set_distance(0.1); // Below minimum
    controller.update(1.0);
    assert!(controller.distance() >= 0.5);

    controller.set_distance(2000.0); // Above maximum
    controller.update(1.0);
    assert!(controller.distance() <= 1000.0);
}

/// Test editor camera controller angle controls.
#[test]
fn test_editor_camera_controller_angles() {
    let mut controller = EditorCameraController::new();

    let yaw = std::f32::consts::PI;
    let pitch = std::f32::consts::FRAC_PI_4;

    controller.set_angles(yaw, pitch);
    controller.update(1.0);

    let (actual_yaw, actual_pitch) = controller.angles();
    assert!((actual_yaw - yaw).abs() < 0.001);
    assert!((actual_pitch - pitch).abs() < 0.001);
}

/// Test editor camera controller focus functionality.
#[test]
fn test_editor_camera_controller_focus() {
    let mut controller = EditorCameraController::new();

    let focus_point = Vec3::new(5.0, 10.0, -5.0);
    let distance = 15.0;

    controller.focus_on(focus_point, Some(distance));
    controller.update(1.0);

    assert_eq!(controller.target(), focus_point);
    assert_eq!(controller.distance(), distance);
}

/// Test editor camera controller focus on selection.
#[test]
fn test_editor_camera_controller_focus_on_selection() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    let mut controller = EditorCameraController::new();

    // Create entities at different positions
    let entity1 = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Selectable,
    ));

    let entity2 = world.spawn((
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        Selectable,
    ));

    // Select both entities
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity1, SelectionMode::Replace);
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity2, SelectionMode::Add);

    // Focus on selection
    let selection = world.get_resource::<SelectionSystem>().unwrap();
    let transform_query = world.query::<&GlobalTransform>();
    controller.focus_on_selection(&selection, &transform_query);
    controller.update(1.0);

    // Camera should focus on center of bounding box
    let target = controller.target();
    assert!((target.x - 5.0).abs() < 0.1); // Center between 0 and 10
}

/// Test editor camera controller input processing.
#[test]
fn test_editor_camera_controller_input_processing() {
    let mut controller = EditorCameraController::new();
    let mut input = InputState::new();

    let initial_distance = controller.distance();

    // Simulate scroll wheel zoom
    input.set_scroll_delta((0.0, 1.0));
    controller.process_input(&input, 0.016);
    controller.update(1.0);

    // Distance should have changed
    assert_ne!(controller.distance(), initial_distance);
}

/// Test editor camera controller position computation.
#[test]
fn test_editor_camera_controller_position_computation() {
    let mut controller = EditorCameraController::new();
    controller.set_target(Vec3::ZERO);
    controller.set_distance(10.0);
    controller.set_angles(0.0, 0.0);
    controller.update(1.0);

    let position = controller.compute_position();
    let transform = controller.compute_transform();

    // Position should be at distance from target
    let distance_from_target = (position - controller.target()).length();
    assert!((distance_from_target - 10.0).abs() < 0.01);

    // Transform position should match computed position
    assert!((transform.translation - position).length() < 0.01);
}

/// Test editor camera controller smooth interpolation.
#[test]
fn test_editor_camera_controller_smooth_interpolation() {
    let mut controller = EditorCameraController::new();
    controller.set_distance(10.0);
    controller.update(1.0);

    let initial_distance = controller.distance();

    // Request new distance
    controller.set_distance(20.0);

    // After small update, should be between initial and target
    controller.update(0.01);
    let distance_after_small = controller.distance();
    assert!(distance_after_small > initial_distance);
    assert!(distance_after_small < 20.0);

    // After large update, should reach target
    controller.update(1.0);
    assert!((controller.distance() - 20.0).abs() < 0.001);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

/// Test complete editor workflow: create entity, select, edit, delete.
#[test]
fn test_complete_editor_workflow() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(UndoRedoSystem::new());

    let entity_ops = EntityOperations::new();
    let mut undo_system = UndoRedoSystem::new();

    // Create entity
    let entity = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Test Entity", vec![])
        .expect("Failed to create entity");

    // Verify entity exists
    assert!(world.inner().get_entity(entity).is_ok());

    // Select entity
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity, SelectionMode::Replace);
    assert!(world
        .get_resource::<SelectionSystem>()
        .unwrap()
        .is_selected(entity));

    // Edit entity components
    world.entity_mut(entity).insert(Transform {
        translation: Vec3::new(5.0, 10.0, 15.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    // Verify edit
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation, Vec3::new(5.0, 10.0, 15.0));

    // Delete entity
    entity_ops
        .delete_entities(&mut world, &mut undo_system, vec![entity])
        .expect("Failed to delete entity");
    assert!(world.inner().get_entity(entity).is_err());
}

/// Test multi-panel interaction: hierarchy selection affects inspector.
#[test]
fn test_multi_panel_interaction() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(UndoRedoSystem::new());

    let entity_ops = EntityOperations::new();
    let mut undo_system = UndoRedoSystem::new();

    // Create multiple entities
    let entity1 = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Entity1", vec![])
        .unwrap();
    let entity2 = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Entity2", vec![])
        .unwrap();

    // Add transforms
    world
        .entity_mut(entity1)
        .insert(Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)));
    world
        .entity_mut(entity2)
        .insert(Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)));

    // Select entity1 in hierarchy (simulated)
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity1, SelectionMode::Replace);

    // Inspector should see entity1's transform
    let selected: Vec<_> = world
        .get_resource::<SelectionSystem>()
        .unwrap()
        .selected_entities()
        .collect();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0], entity1);

    let transform = world.get::<Transform>(entity1).unwrap();
    assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));

    // Switch selection to entity2
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity2, SelectionMode::Replace);

    // Inspector should now see entity2's transform
    let selected: Vec<_> = world
        .get_resource::<SelectionSystem>()
        .unwrap()
        .selected_entities()
        .collect();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0], entity2);

    let transform = world.get::<Transform>(entity2).unwrap();
    assert_eq!(transform.translation, Vec3::new(4.0, 5.0, 6.0));
}

/// Test viewport camera focus follows hierarchy selection.
#[test]
fn test_viewport_follows_hierarchy_selection() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    let mut viewport = ViewportPanel::new();

    // Create entities at different positions
    let entity1 = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        MeshHandle::new("cube".to_string()),
        Selectable,
    ));

    let entity2 = world.spawn((
        Transform::from_translation(Vec3::new(20.0, 10.0, 5.0)),
        GlobalTransform::from_translation(Vec3::new(20.0, 10.0, 5.0)),
        MeshHandle::new("sphere".to_string()),
        Selectable,
    ));

    // Select entity1
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity1, SelectionMode::Replace);

    // Focus camera
    viewport.focus_on_selection(&mut world);
    viewport.update_camera(1.0);
    let target1 = viewport.camera_target();

    // Select entity2
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity2, SelectionMode::Replace);

    // Focus camera on new selection
    viewport.focus_on_selection(&mut world);
    viewport.update_camera(1.0);
    let target2 = viewport.camera_target();

    // Camera targets should be different
    assert!((target1 - target2).length() > 1.0);

    // target2 should be near entity2's position
    assert!((target2 - Vec3::new(20.0, 10.0, 5.0)).length() < 1.0);
}

/// Test asset drag from assets panel to viewport instantiation.
#[test]
fn test_asset_drag_to_viewport_instantiation() {
    let mut world = World::new();
    world.insert_resource(DragDropSystem::new());
    world.insert_resource(SelectionSystem::new());

    // Create asset entry
    let asset = AssetEntry {
        name: "test_cube.obj".to_string(),
        path: PathBuf::from("assets/models/test_cube.obj"),
        is_directory: false,
        asset_type: AssetType::Model,
        modified: None,
        thumbnail: None,
    };

    // Start drag from assets panel
    let payload = DragDropPayload::from_asset(&asset);
    world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .start_drag(payload);

    // Simulate drop in viewport
    let dropped = world
        .get_resource_mut::<DragDropSystem>()
        .unwrap()
        .complete_drop();

    assert!(dropped.is_some());

    // Create entity from dropped asset
    if let Some(path) = dropped.unwrap().as_asset_path() {
        let entity = world.spawn((
            Name::new("Instantiated Cube"),
            Transform::from_translation(Vec3::ZERO),
            GlobalTransform::default(),
            MeshHandle::new(path.to_string_lossy().to_string()),
            Selectable,
        ));

        // Verify entity was created
        assert!(world.inner().get_entity(entity).is_ok());
        assert!(world.get::<MeshHandle>(entity).is_some());

        // Select newly created entity
        world
            .get_resource_mut::<SelectionSystem>()
            .unwrap()
            .select_entity(entity, SelectionMode::Replace);

        assert!(world
            .get_resource::<SelectionSystem>()
            .unwrap()
            .is_selected(entity));
    }
}

/// Test undo/redo integration across editor panels.
#[test]
fn test_undo_redo_integration() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(UndoRedoSystem::new());

    let entity_ops = EntityOperations::new();
    let mut undo_system = UndoRedoSystem::new();

    // Create entity
    let entity = entity_ops
        .create_entity_with_components(&mut world, &mut undo_system, "Test Entity", vec![])
        .expect("Failed to create entity");

    // Verify entity exists
    assert!(world.inner().get_entity(entity).is_ok());

    // Undo creation
    undo_system.undo(&mut world).expect("Failed to undo");
    assert!(world.inner().get_entity(entity).is_err());

    // Redo creation
    undo_system.redo(&mut world).expect("Failed to redo");
    // Note: After redo, entity might have different ID, so we check by name
    let mut query = world.query::<&Name>();
    let names: Vec<_> = query.iter(world.inner()).map(|n| n.as_str()).collect();
    assert!(names.contains(&"Test Entity"));
}
