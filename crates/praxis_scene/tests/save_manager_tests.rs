//! Integration tests for SaveManager.
//!
//! These tests cover full scene save/load cycles including:
//! - Entity hierarchies with parent-child relationships
//! - All component types (transforms, meshes, cameras, lights, etc.)
//! - Version migration
//! - Save metadata
//! - NoSave marker behavior
//! - Editor data preservation
//! - Statistics tracking

use praxis_ecs::{
    Active, Camera, Children, DirectionalLight, GlobalTransform, MaterialHandle, MeshHandle, Name,
    NoSave, OrthographicProjection, Parent, PerspectiveProjection, PointLight, TextureHandle,
    Transform, Visibility, World,
};
use praxis_scene::{
    migrate_scene, SaveConfig, SaveFile, SaveManager, SaveMetadata, SceneDefinition,
    CURRENT_SAVE_VERSION, CURRENT_SCENE_VERSION,
};
use std::fs;
use std::path::PathBuf;

/// Helper function to create a temporary test directory.
fn temp_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("praxis_scene_tests_{}", rand::random::<u32>()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper function to cleanup test directory.
fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_save_and_load_empty_world() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("empty_world.ron");

    let metadata = SaveMetadata::new("Empty World");
    manager
        .save_to_file(&mut world, &save_path, metadata.clone())
        .unwrap();

    assert!(save_path.exists());

    // Verify statistics
    let stats = manager.last_stats().unwrap();
    assert_eq!(stats.entity_count, 0);
    assert_eq!(stats.component_count, 0);
    assert!(stats.file_size_bytes.is_some());

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify empty world
    let count = new_world.query::<&Name>().iter(&new_world).count();
    assert_eq!(count, 0);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_single_entity_with_transform() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("single_entity.ron");

    // Create entity with transform
    world.spawn((
        Name("TestEntity".to_string()),
        Transform::from_xyz(10.0, 20.0, 30.0),
        GlobalTransform::default(),
        Active,
    ));

    let metadata = SaveMetadata::new("Single Entity Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify entity exists with correct components
    let mut query = new_world.query::<(&Name, &Transform, &Active)>();
    let mut count = 0;
    for (name, transform, _active) in query.iter(&new_world) {
        assert_eq!(name.0, "TestEntity");
        assert_eq!(transform.translation.x, 10.0);
        assert_eq!(transform.translation.y, 20.0);
        assert_eq!(transform.translation.z, 30.0);
        count += 1;
    }
    assert_eq!(count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_entity_with_all_components() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("full_entity.ron");

    // Create entity with many components
    world.spawn((
        Name("FullEntity".to_string()),
        Transform::from_xyz(1.0, 2.0, 3.0),
        GlobalTransform::default(),
        MeshHandle::new("test_mesh"),
        TextureHandle::new("test_texture"),
        MaterialHandle::new("test_material"),
        Visibility::Visible,
        Active,
    ));

    let metadata = SaveMetadata::new("Full Entity Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify all components
    let mut query = new_world.query::<(
        &Name,
        &Transform,
        &MeshHandle,
        &TextureHandle,
        &MaterialHandle,
    )>();
    let mut count = 0;
    for (name, transform, mesh, texture, material) in query.iter(&new_world) {
        assert_eq!(name.0, "FullEntity");
        assert_eq!(transform.translation.x, 1.0);
        assert_eq!(mesh.id, "test_mesh");
        assert_eq!(texture.id, "test_texture");
        assert_eq!(material.id, "test_material");
        count += 1;
    }
    assert_eq!(count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_parent_child_hierarchy() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("hierarchy.ron");

    // Create parent-child hierarchy
    let parent = world
        .spawn((
            Name("Parent".to_string()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            Active,
        ))
        .id();

    let child = world
        .spawn((
            Name("Child".to_string()),
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ))
        .id();

    // Add Children component to parent
    world.entity_mut(parent).insert(Children(vec![child]));

    let metadata = SaveMetadata::new("Hierarchy Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Verify stats
    let stats = manager.last_stats().unwrap();
    assert_eq!(stats.entity_count, 2);

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify parent exists
    let mut parent_query = new_world.query::<(&Name, &Children)>();
    let mut parent_count = 0;
    for (name, children) in parent_query.iter(&new_world) {
        if name.0 == "Parent" {
            assert_eq!(children.0.len(), 1);
            parent_count += 1;
        }
    }
    assert_eq!(parent_count, 1);

    // Verify child exists
    let mut child_query = new_world.query::<(&Name, &Parent)>();
    let mut child_count = 0;
    for (name, _parent) in child_query.iter(&new_world) {
        if name.0 == "Child" {
            child_count += 1;
        }
    }
    assert_eq!(child_count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_deep_hierarchy() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("deep_hierarchy.ron");

    // Create a 3-level hierarchy: Root -> Parent -> Child -> Grandchild
    let root = world
        .spawn((
            Name("Root".to_string()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    let parent = world
        .spawn((
            Name("Parent".to_string()),
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(root),
        ))
        .id();

    let child = world
        .spawn((
            Name("Child".to_string()),
            Transform::from_xyz(2.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ))
        .id();

    let grandchild = world
        .spawn((
            Name("Grandchild".to_string()),
            Transform::from_xyz(3.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(child),
        ))
        .id();

    // Setup Children components
    world.entity_mut(root).insert(Children(vec![parent]));
    world.entity_mut(parent).insert(Children(vec![child]));
    world.entity_mut(child).insert(Children(vec![grandchild]));

    let metadata = SaveMetadata::new("Deep Hierarchy Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Verify stats
    let stats = manager.last_stats().unwrap();
    assert_eq!(stats.entity_count, 4);

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify all entities exist
    let names: Vec<String> = new_world
        .query::<&Name>()
        .iter(&new_world)
        .map(|n| n.0.clone())
        .collect();

    assert!(names.contains(&"Root".to_string()));
    assert!(names.contains(&"Parent".to_string()));
    assert!(names.contains(&"Child".to_string()));
    assert!(names.contains(&"Grandchild".to_string()));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_multiple_children() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("multiple_children.ron");

    // Create parent with multiple children
    let parent = world
        .spawn((
            Name("Parent".to_string()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    let child1 = world
        .spawn((
            Name("Child1".to_string()),
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ))
        .id();

    let child2 = world
        .spawn((
            Name("Child2".to_string()),
            Transform::from_xyz(2.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ))
        .id();

    let child3 = world
        .spawn((
            Name("Child3".to_string()),
            Transform::from_xyz(3.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ))
        .id();

    world
        .entity_mut(parent)
        .insert(Children(vec![child1, child2, child3]));

    let metadata = SaveMetadata::new("Multiple Children Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify parent has 3 children
    let mut query = new_world.query::<(&Name, &Children)>();
    for (name, children) in query.iter(&new_world) {
        if name.0 == "Parent" {
            assert_eq!(children.0.len(), 3);
        }
    }

    // Verify all children exist
    let child_count = new_world
        .query::<(&Name, &Parent)>()
        .iter(&new_world)
        .count();
    assert_eq!(child_count, 3);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_skips_no_save_entities() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("no_save.ron");

    // Create regular entity
    world.spawn((Name("SavedEntity".to_string()), Active));

    // Create entity with NoSave marker
    world.spawn((Name("TemporaryEntity".to_string()), NoSave));

    let metadata = SaveMetadata::new("NoSave Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Verify only 1 entity was saved
    let stats = manager.last_stats().unwrap();
    assert_eq!(stats.entity_count, 1);

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify only saved entity exists
    let names: Vec<String> = new_world
        .query::<&Name>()
        .iter(&new_world)
        .map(|n| n.0.clone())
        .collect();

    assert_eq!(names.len(), 1);
    assert!(names.contains(&"SavedEntity".to_string()));
    assert!(!names.contains(&"TemporaryEntity".to_string()));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_perspective_camera() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("camera.ron");

    // Create camera entity
    world.spawn((
        Name("MainCamera".to_string()),
        Transform::from_xyz(0.0, 5.0, 10.0),
        GlobalTransform::default(),
        Camera {
            is_active: true,
            priority: 0,
        },
        PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        },
    ));

    let metadata = SaveMetadata::new("Camera Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify camera exists
    let mut query = new_world.query::<(&Name, &Camera, &PerspectiveProjection)>();
    let mut count = 0;
    for (name, camera, projection) in query.iter(&new_world) {
        assert_eq!(name.0, "MainCamera");
        assert!(camera.is_active);
        assert_eq!(camera.priority, 0);
        assert!((projection.fov - 70.0_f32.to_radians()).abs() < 0.001);
        assert!((projection.aspect_ratio - 16.0 / 9.0).abs() < 0.001);
        count += 1;
    }
    assert_eq!(count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_orthographic_camera() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("ortho_camera.ron");

    // Create orthographic camera
    world.spawn((
        Name("OrthoCamera".to_string()),
        Transform::from_xyz(0.0, 10.0, 0.0),
        GlobalTransform::default(),
        Camera {
            is_active: false,
            priority: 1,
        },
        OrthographicProjection {
            left: -10.0,
            right: 10.0,
            bottom: -10.0,
            top: 10.0,
            near: 0.1,
            far: 100.0,
        },
    ));

    let metadata = SaveMetadata::new("Ortho Camera Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify camera exists
    let mut query = new_world.query::<(&Name, &Camera, &OrthographicProjection)>();
    let mut count = 0;
    for (name, camera, projection) in query.iter(&new_world) {
        assert_eq!(name.0, "OrthoCamera");
        assert!(!camera.is_active);
        assert_eq!(camera.priority, 1);
        assert_eq!(projection.left, -10.0);
        assert_eq!(projection.right, 10.0);
        count += 1;
    }
    assert_eq!(count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_directional_light() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("dir_light.ron");

    // Create directional light
    world.spawn((
        Name("Sun".to_string()),
        DirectionalLight {
            direction: praxis_math::Vec3::new(0.0, -1.0, 0.0),
            color: praxis_math::Vec3::new(1.0, 0.95, 0.9),
            intensity: 1.5,
        },
    ));

    let metadata = SaveMetadata::new("Directional Light Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify light exists
    let mut query = new_world.query::<(&Name, &DirectionalLight)>();
    let mut count = 0;
    for (name, light) in query.iter(&new_world) {
        assert_eq!(name.0, "Sun");
        assert_eq!(light.direction.y, -1.0);
        assert!((light.intensity - 1.5).abs() < 0.001);
        count += 1;
    }
    assert_eq!(count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_point_light() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("point_light.ron");

    // Create point light
    world.spawn((
        Name("Lamp".to_string()),
        Transform::from_xyz(5.0, 3.0, 2.0),
        GlobalTransform::default(),
        PointLight {
            color: praxis_math::Vec3::new(1.0, 0.8, 0.6),
            intensity: 2.0,
            range: 15.0,
        },
    ));

    let metadata = SaveMetadata::new("Point Light Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify light exists
    let mut query = new_world.query::<(&Name, &PointLight, &Transform)>();
    let mut count = 0;
    for (name, light, transform) in query.iter(&new_world) {
        assert_eq!(name.0, "Lamp");
        assert!((light.intensity - 2.0).abs() < 0.001);
        assert!((light.range - 15.0).abs() < 0.001);
        assert_eq!(transform.translation.x, 5.0);
        count += 1;
    }
    assert_eq!(count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_and_load_visibility() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("visibility.ron");

    // Create visible entity
    world.spawn((
        Name("VisibleEntity".to_string()),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::Visible,
    ));

    // Create hidden entity
    world.spawn((
        Name("HiddenEntity".to_string()),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::Hidden,
    ));

    let metadata = SaveMetadata::new("Visibility Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify visibility states
    let mut query = new_world.query::<(&Name, &Visibility)>();
    for (name, visibility) in query.iter(&new_world) {
        if name.0 == "VisibleEntity" {
            assert!(matches!(visibility, Visibility::Visible));
        } else if name.0 == "HiddenEntity" {
            assert!(matches!(visibility, Visibility::Hidden));
        }
    }

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_metadata_preservation() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("metadata.ron");

    world.spawn((Name("TestEntity".to_string()), Active));

    let metadata = SaveMetadata::new("Test Save")
        .with_description("A test save file")
        .with_playtime(3600)
        .with_game_version("1.0.0")
        .with_screenshot("screenshot.png")
        .with_tag("autosave")
        .with_tag("checkpoint")
        .with_custom_data("level", "forest")
        .with_custom_data("chapter", "1");

    manager
        .save_to_file(&mut world, &save_path, metadata.clone())
        .unwrap();

    // Read metadata back
    let loaded_metadata = manager.read_metadata(&save_path).unwrap();

    assert_eq!(loaded_metadata.name, "Test Save");
    assert_eq!(
        loaded_metadata.description,
        Some("A test save file".to_string())
    );
    assert_eq!(loaded_metadata.playtime_seconds, 3600);
    assert_eq!(loaded_metadata.game_version, Some("1.0.0".to_string()));
    assert_eq!(
        loaded_metadata.screenshot_path,
        Some("screenshot.png".to_string())
    );
    assert!(loaded_metadata.tags.contains(&"autosave".to_string()));
    assert!(loaded_metadata.tags.contains(&"checkpoint".to_string()));
    assert_eq!(
        loaded_metadata.custom_data.get("level"),
        Some(&"forest".to_string())
    );
    assert_eq!(
        loaded_metadata.custom_data.get("chapter"),
        Some(&"1".to_string())
    );

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_config_pretty_print() {
    let mut world = World::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("pretty.ron");

    world.spawn((Name("TestEntity".to_string()), Active));

    // Save with pretty print enabled
    let config = SaveConfig {
        compress: false,
        include_editor_data: false,
        validate_after_save: true,
        pretty_print: true,
    };
    let mut manager = SaveManager::with_config(config);
    let metadata = SaveMetadata::new("Pretty Print Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Read file and check for newlines (pretty printing)
    let contents = fs::read_to_string(&save_path).unwrap();
    assert!(contents.contains('\n'));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_config_validation() {
    let mut world = World::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("validation.ron");

    world.spawn((Name("TestEntity".to_string()), Active));

    // Save with validation enabled
    let config = SaveConfig {
        compress: false,
        include_editor_data: false,
        validate_after_save: true,
        pretty_print: true,
    };
    let mut manager = SaveManager::with_config(config);
    let metadata = SaveMetadata::new("Validation Test");
    let result = manager.save_to_file(&mut world, &save_path, metadata);

    assert!(result.is_ok());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_statistics() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("statistics.ron");

    // Create entities with various components
    world.spawn((
        Name("Entity1".to_string()),
        Transform::default(),
        GlobalTransform::default(),
        MeshHandle::new("mesh1"),
        Active,
    ));

    world.spawn((
        Name("Entity2".to_string()),
        Transform::default(),
        GlobalTransform::default(),
        Active,
    ));

    let metadata = SaveMetadata::new("Statistics Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Check statistics
    let stats = manager.last_stats().unwrap();
    assert_eq!(stats.entity_count, 2);
    assert!(stats.component_count > 0);
    assert!(stats.duration_ms >= 0.0);
    assert!(stats.file_size_bytes.is_some());
    assert!(stats.file_size_bytes.unwrap() > 0);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_load_clears_world() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("clear_world.ron");

    // Create and save one entity
    world.spawn((Name("SavedEntity".to_string()), Active));
    let metadata = SaveMetadata::new("Clear Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Add different entity to world
    world.spawn((Name("ExtraEntity".to_string()), Active));

    // Verify world has 2 entities
    assert_eq!(world.query::<&Name>().iter(&world).count(), 2);

    // Load save (should clear world first)
    manager.load_from_file(&mut world, &save_path).unwrap();

    // Verify world only has the saved entity
    let names: Vec<String> = world
        .query::<&Name>()
        .iter(&world)
        .map(|n| n.0.clone())
        .collect();
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"SavedEntity".to_string()));
    assert!(!names.contains(&"ExtraEntity".to_string()));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_version_migration_from_v0() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("version_0.ron");

    // Manually create a version 0 save file
    let mut scene = SceneDefinition::new("Test Scene");
    scene.version = 0;

    let save_file = SaveFile {
        version: 1,
        metadata: SaveMetadata::new("Version 0 Test"),
        scene,
    };

    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify migration
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let result = manager.load_from_file(&mut world, &save_path);

    assert!(result.is_ok());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_version_migration_from_v1() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("version_1.ron");

    // Create a version 1 save file
    let mut scene = SceneDefinition::new("Test Scene");
    scene.version = 1;

    let save_file = SaveFile {
        version: CURRENT_SAVE_VERSION,
        metadata: SaveMetadata::new("Version 1 Test"),
        scene,
    };

    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify migration
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let result = manager.load_from_file(&mut world, &save_path);

    assert!(result.is_ok());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_scene_version_migration() {
    let mut scene = SceneDefinition::new("Migration Test");
    scene.version = 0;

    let result = migrate_scene(&mut scene);
    assert!(result.is_ok());
    assert!(result.unwrap());
    assert_eq!(scene.version, CURRENT_SCENE_VERSION);
}

#[test]
fn test_complex_scene_round_trip() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("complex_scene.ron");

    // Create a complex scene with multiple entity types

    // Camera
    world.spawn((
        Name("MainCamera".to_string()),
        Transform::from_xyz(0.0, 5.0, 10.0),
        GlobalTransform::default(),
        Camera {
            is_active: true,
            priority: 0,
        },
        PerspectiveProjection {
            fov: 70.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        },
        Active,
    ));

    // Directional light
    world.spawn((
        Name("Sun".to_string()),
        DirectionalLight {
            direction: praxis_math::Vec3::new(0.0, -1.0, 0.0),
            color: praxis_math::Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
        },
        Active,
    ));

    // Point light
    world.spawn((
        Name("Lamp".to_string()),
        Transform::from_xyz(2.0, 3.0, 1.0),
        GlobalTransform::default(),
        PointLight {
            color: praxis_math::Vec3::new(1.0, 0.8, 0.6),
            intensity: 2.0,
            range: 10.0,
        },
        Active,
    ));

    // Mesh entity with hierarchy
    let parent = world
        .spawn((
            Name("MeshParent".to_string()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            MeshHandle::new("cube"),
            TextureHandle::new("wood"),
            MaterialHandle::new("pbr"),
            Visibility::Visible,
            Active,
        ))
        .id();

    let child = world
        .spawn((
            Name("MeshChild".to_string()),
            Transform::from_xyz(2.0, 0.0, 0.0),
            GlobalTransform::default(),
            MeshHandle::new("sphere"),
            Parent(parent),
            Visibility::Visible,
        ))
        .id();

    world.entity_mut(parent).insert(Children(vec![child]));

    let metadata = SaveMetadata::new("Complex Scene")
        .with_description("A complex test scene")
        .with_playtime(1800);

    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Verify stats
    let stats = manager.last_stats().unwrap();
    assert_eq!(stats.entity_count, 5);
    assert!(stats.component_count > 10);

    // Load into new world
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    // Verify all entity types exist
    let names: Vec<String> = new_world
        .query::<&Name>()
        .iter(&new_world)
        .map(|n| n.0.clone())
        .collect();

    assert_eq!(names.len(), 5);
    assert!(names.contains(&"MainCamera".to_string()));
    assert!(names.contains(&"Sun".to_string()));
    assert!(names.contains(&"Lamp".to_string()));
    assert!(names.contains(&"MeshParent".to_string()));
    assert!(names.contains(&"MeshChild".to_string()));

    // Verify hierarchy
    let parent_count = new_world
        .query::<(&Name, &Children)>()
        .iter(&new_world)
        .filter(|(name, _)| name.0 == "MeshParent")
        .count();
    assert_eq!(parent_count, 1);

    let child_count = new_world
        .query::<(&Name, &Parent)>()
        .iter(&new_world)
        .filter(|(name, _)| name.0 == "MeshChild")
        .count();
    assert_eq!(child_count, 1);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_file_validation() {
    let scene = SceneDefinition::new("Test");
    let metadata = SaveMetadata::new("Test");
    let save_file = SaveFile::new(scene, metadata);

    let result = save_file.validate();
    assert!(result.is_ok());
}

#[test]
fn test_save_file_invalid_version() {
    let scene = SceneDefinition::new("Test");
    let metadata = SaveMetadata::new("Test");
    let mut save_file = SaveFile::new(scene, metadata);
    save_file.version = 0;

    let result = save_file.validate();
    assert!(result.is_err());
}

#[test]
fn test_read_metadata_without_loading() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("metadata_read.ron");

    // Create a large world
    for i in 0..100 {
        world.spawn((
            Name(format!("Entity{}", i)),
            Transform::default(),
            GlobalTransform::default(),
        ));
    }

    let metadata = SaveMetadata::new("Metadata Read Test")
        .with_description("Testing metadata reading")
        .with_playtime(7200);

    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Read just metadata (should be fast)
    let loaded_metadata = manager.read_metadata(&save_path).unwrap();

    assert_eq!(loaded_metadata.name, "Metadata Read Test");
    assert_eq!(
        loaded_metadata.description,
        Some("Testing metadata reading".to_string())
    );
    assert_eq!(loaded_metadata.playtime_seconds, 7200);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_multiple_save_load_cycles() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("cycles.ron");

    // Initial save
    world.spawn((Name("Entity1".to_string()), Active));
    let metadata = SaveMetadata::new("Cycle 1");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load and modify
    let mut world2 = World::new();
    manager.load_from_file(&mut world2, &save_path).unwrap();
    world2.spawn((Name("Entity2".to_string()), Active));

    // Save again
    let metadata2 = SaveMetadata::new("Cycle 2");
    manager
        .save_to_file(&mut world2, &save_path, metadata2)
        .unwrap();

    // Load final state
    let mut world3 = World::new();
    manager.load_from_file(&mut world3, &save_path).unwrap();

    let names: Vec<String> = world3
        .query::<&Name>()
        .iter(&world3)
        .map(|n| n.0.clone())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Entity1".to_string()));
    assert!(names.contains(&"Entity2".to_string()));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_rotation_and_scale() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("transform_full.ron");

    // Create entity with rotation and scale
    let rotation = praxis_math::Quat::from_axis_angle(praxis_math::Vec3::Y, 45.0_f32.to_radians());
    world.spawn((
        Name("Rotated".to_string()),
        Transform {
            translation: praxis_math::Vec3::new(1.0, 2.0, 3.0),
            rotation,
            scale: praxis_math::Vec3::new(2.0, 3.0, 4.0),
        },
        GlobalTransform::default(),
    ));

    let metadata = SaveMetadata::new("Transform Test");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Load and verify
    let mut new_world = World::new();
    manager.load_from_file(&mut new_world, &save_path).unwrap();

    let mut query = new_world.query::<(&Name, &Transform)>();
    for (name, transform) in query.iter(&new_world) {
        assert_eq!(name.0, "Rotated");
        assert_eq!(transform.translation.x, 1.0);
        assert_eq!(transform.scale.x, 2.0);
        // Rotation should be preserved (approximate check due to floating point)
        assert!((transform.rotation.w - rotation.w).abs() < 0.001);
    }

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_concurrent_saves_different_paths() {
    let mut world1 = World::new();
    let mut world2 = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let save_path1 = test_dir.join("save1.ron");
    let save_path2 = test_dir.join("save2.ron");

    world1.spawn((Name("World1Entity".to_string()), Active));
    world2.spawn((Name("World2Entity".to_string()), Active));

    let metadata1 = SaveMetadata::new("Save 1");
    let metadata2 = SaveMetadata::new("Save 2");

    manager
        .save_to_file(&mut world1, &save_path1, metadata1)
        .unwrap();
    manager
        .save_to_file(&mut world2, &save_path2, metadata2)
        .unwrap();

    assert!(save_path1.exists());
    assert!(save_path2.exists());

    // Verify they're different
    let contents1 = fs::read_to_string(&save_path1).unwrap();
    let contents2 = fs::read_to_string(&save_path2).unwrap();
    assert_ne!(contents1, contents2);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_to_nested_directory() {
    let mut world = World::new();
    let mut manager = SaveManager::new();
    let test_dir = temp_test_dir();
    let nested_path = test_dir.join("saves/slot1/autosave.ron");

    world.spawn((Name("TestEntity".to_string()), Active));

    let metadata = SaveMetadata::new("Nested Save");
    let result = manager.save_to_file(&mut world, &nested_path, metadata);

    assert!(result.is_ok());
    assert!(nested_path.exists());

    cleanup_test_dir(&test_dir);
}
