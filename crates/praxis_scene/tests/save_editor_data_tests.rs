//! Integration tests for SaveManager editor data preservation.
//!
//! These tests verify that editor-specific data (camera state, selections,
//! viewport settings, preferences) is correctly saved and loaded when configured.

use praxis_ecs::{Active, Name, Transform, World};
use praxis_scene::{
    CameraMode, EditorCamera, EditorData, EditorPreferences, EntityDefinition, GizmoMode,
    SaveConfig, SaveFile, SaveManager, SaveMetadata, SceneDefinition, ViewportSettings,
    CURRENT_SAVE_VERSION, CURRENT_SCENE_VERSION,
};
use std::fs;
use std::path::PathBuf;

/// Helper function to create a temporary test directory.
fn temp_test_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("praxis_editor_tests_{}", rand::random::<u32>()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper function to cleanup test directory.
fn cleanup_test_dir(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_save_without_editor_data() {
    let mut world = World::new();
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("no_editor_data.ron");

    world.spawn((Name("TestEntity".to_string()), Active));

    // Save with editor data explicitly disabled
    let config = SaveConfig {
        compress: false,
        include_editor_data: false,
        validate_after_save: true,
        pretty_print: true,
    };
    let mut manager = SaveManager::with_config(config);
    let metadata = SaveMetadata::new("No Editor Data");
    manager
        .save_to_file(&mut world, &save_path, metadata)
        .unwrap();

    // Read file and verify no editor data section
    let contents = fs::read_to_string(&save_path).unwrap();
    let save_file: SaveFile = ron::from_str(&contents).unwrap();
    assert!(save_file.scene.editor_data.is_none());

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_editor_camera() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("editor_camera.ron");

    // Create scene with editor data
    let mut scene = SceneDefinition::new("Test Scene");
    let editor_camera = EditorCamera::orbit((0.0, 0.0, 0.0), 15.0, -0.5, 0.8);
    scene.set_editor_data(EditorData::new().with_camera(editor_camera));

    let save_file = SaveFile::new(scene, SaveMetadata::new("Editor Camera Test"));
    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify
    let contents = fs::read_to_string(&save_path).unwrap();
    let loaded_file: SaveFile = ron::from_str(&contents).unwrap();

    let editor_data = loaded_file.scene.editor_data.as_ref().unwrap();
    let camera = editor_data.camera.as_ref().unwrap();

    assert_eq!(camera.target, (0.0, 0.0, 0.0));
    assert_eq!(camera.distance, 15.0);
    assert_eq!(camera.mode, CameraMode::Orbit);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_editor_viewport_settings() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("viewport_settings.ron");

    // Create scene with viewport settings
    let mut scene = SceneDefinition::new("Test Scene");
    let mut viewport = ViewportSettings::new();
    viewport.show_grid = true;
    viewport.show_gizmos = true;
    viewport.show_wireframe = true;
    viewport.grid_size = 30;
    viewport.grid_spacing = 2.0;
    viewport.gizmo_mode = GizmoMode::Rotate;

    scene.set_editor_data(EditorData::new().with_viewport(viewport));

    let save_file = SaveFile::new(scene, SaveMetadata::new("Viewport Settings Test"));
    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify
    let contents = fs::read_to_string(&save_path).unwrap();
    let loaded_file: SaveFile = ron::from_str(&contents).unwrap();

    let editor_data = loaded_file.scene.editor_data.as_ref().unwrap();
    let loaded_viewport = editor_data.viewport.as_ref().unwrap();

    assert!(loaded_viewport.show_grid);
    assert!(loaded_viewport.show_gizmos);
    assert!(loaded_viewport.show_wireframe);
    assert_eq!(loaded_viewport.grid_size, 30);
    assert_eq!(loaded_viewport.grid_spacing, 2.0);
    assert_eq!(loaded_viewport.gizmo_mode, GizmoMode::Rotate);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_selected_entities() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("selected_entities.ron");

    // Create scene with selection
    let mut scene = SceneDefinition::new("Test Scene");
    scene.add_entity(EntityDefinition::new().with_name("Entity1"));
    scene.add_entity(EntityDefinition::new().with_name("Entity2"));

    let selected = vec!["Entity1".to_string(), "Entity2".to_string()];
    scene.set_editor_data(EditorData::new().with_selected_entities(selected));

    let save_file = SaveFile::new(scene, SaveMetadata::new("Selection Test"));
    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify
    let contents = fs::read_to_string(&save_path).unwrap();
    let loaded_file: SaveFile = ron::from_str(&contents).unwrap();

    let editor_data = loaded_file.scene.editor_data.as_ref().unwrap();
    assert_eq!(editor_data.selected_entities.len(), 2);
    assert!(editor_data
        .selected_entities
        .contains(&"Entity1".to_string()));
    assert!(editor_data
        .selected_entities
        .contains(&"Entity2".to_string()));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_editor_preferences() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("editor_preferences.ron");

    // Create scene with preferences
    let mut scene = SceneDefinition::new("Test Scene");
    let mut prefs = EditorPreferences::new();
    prefs.auto_save_enabled = true;
    prefs.auto_save_interval = 600.0;
    prefs.snap_to_grid = true;
    prefs.snap_size = 0.5;
    prefs.rotation_snap = 5.0;

    scene.set_editor_data(EditorData::new().with_preferences(prefs));

    let save_file = SaveFile::new(scene, SaveMetadata::new("Preferences Test"));
    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify
    let contents = fs::read_to_string(&save_path).unwrap();
    let loaded_file: SaveFile = ron::from_str(&contents).unwrap();

    let editor_data = loaded_file.scene.editor_data.as_ref().unwrap();
    let loaded_prefs = editor_data.preferences.as_ref().unwrap();

    assert!(loaded_prefs.auto_save_enabled);
    assert_eq!(loaded_prefs.auto_save_interval, 600.0);
    assert!(loaded_prefs.snap_to_grid);
    assert_eq!(loaded_prefs.snap_size, 0.5);
    assert_eq!(loaded_prefs.rotation_snap, 5.0);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_save_with_complete_editor_data() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("complete_editor_data.ron");

    // Create scene with all editor data
    let mut scene = SceneDefinition::new("Test Scene");
    scene.add_entity(EntityDefinition::new().with_name("SelectedEntity"));

    let camera = EditorCamera::free((10.0, 20.0, 30.0), -0.3, 1.5);
    let viewport = ViewportSettings::new();
    let prefs = EditorPreferences::new();
    let selected = vec!["SelectedEntity".to_string()];

    let editor_data = EditorData::new()
        .with_camera(camera)
        .with_viewport(viewport)
        .with_preferences(prefs)
        .with_selected_entities(selected);

    scene.set_editor_data(editor_data);

    let save_file = SaveFile::new(scene, SaveMetadata::new("Complete Editor Data Test"));
    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load and verify all parts
    let contents = fs::read_to_string(&save_path).unwrap();
    let loaded_file: SaveFile = ron::from_str(&contents).unwrap();

    let editor_data = loaded_file.scene.editor_data.as_ref().unwrap();

    // Verify camera
    assert!(editor_data.camera.is_some());
    let loaded_camera = editor_data.camera.as_ref().unwrap();
    assert_eq!(loaded_camera.position, (10.0, 20.0, 30.0));
    assert_eq!(loaded_camera.mode, CameraMode::Free);

    // Verify viewport
    assert!(editor_data.viewport.is_some());

    // Verify preferences
    assert!(editor_data.preferences.is_some());

    // Verify selection
    assert_eq!(editor_data.selected_entities.len(), 1);
    assert_eq!(editor_data.selected_entities[0], "SelectedEntity");

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_editor_camera_modes() {
    let orbit_camera = EditorCamera::orbit((5.0, 0.0, 0.0), 20.0, -0.4, 0.9);
    assert_eq!(orbit_camera.mode, CameraMode::Orbit);
    assert_eq!(orbit_camera.target, (5.0, 0.0, 0.0));
    assert_eq!(orbit_camera.distance, 20.0);

    let free_camera = EditorCamera::free((1.0, 2.0, 3.0), 0.5, 1.0);
    assert_eq!(free_camera.mode, CameraMode::Free);
    assert_eq!(free_camera.position, (1.0, 2.0, 3.0));
}

#[test]
fn test_gizmo_modes() {
    let mut viewport = ViewportSettings::new();

    viewport.gizmo_mode = GizmoMode::Translate;
    assert_eq!(viewport.gizmo_mode, GizmoMode::Translate);

    viewport.gizmo_mode = GizmoMode::Rotate;
    assert_eq!(viewport.gizmo_mode, GizmoMode::Rotate);

    viewport.gizmo_mode = GizmoMode::Scale;
    assert_eq!(viewport.gizmo_mode, GizmoMode::Scale);
}

#[test]
fn test_viewport_default_settings() {
    let viewport = ViewportSettings::new();

    assert!(viewport.show_grid);
    assert!(viewport.show_gizmos);
    assert!(!viewport.show_wireframe);
    assert!(!viewport.show_bounds);
    assert_eq!(viewport.grid_size, 20);
    assert_eq!(viewport.grid_spacing, 1.0);
    assert_eq!(viewport.background_color, (0.118, 0.118, 0.137));
}

#[test]
fn test_editor_preferences_defaults() {
    let prefs = EditorPreferences::new();

    assert!(prefs.auto_save_enabled);
    assert_eq!(prefs.auto_save_interval, 300.0);
    assert!(!prefs.snap_to_grid);
    assert_eq!(prefs.snap_size, 1.0);
    assert_eq!(prefs.rotation_snap, 15.0);
    assert!(prefs.last_asset_path.is_none());
    assert!(prefs.collapsed_hierarchy_nodes.is_empty());
}

#[test]
fn test_scene_to_runtime_scene() {
    let mut scene = SceneDefinition::new("Test Scene");
    scene.add_entity(EntityDefinition::new().with_name("TestEntity"));

    // Add editor data
    let camera = EditorCamera::new();
    scene.set_editor_data(EditorData::new().with_camera(camera));

    assert!(scene.has_editor_data());

    // Convert to runtime scene
    let runtime_scene = scene.to_runtime_scene();

    assert!(!runtime_scene.has_editor_data());
    assert_eq!(runtime_scene.entity_count(), 1);
    assert_eq!(runtime_scene.name, scene.name);
}

#[test]
fn test_clear_editor_data() {
    let mut scene = SceneDefinition::new("Test Scene");
    let camera = EditorCamera::new();
    scene.set_editor_data(EditorData::new().with_camera(camera));

    assert!(scene.has_editor_data());

    scene.clear_editor_data();

    assert!(!scene.has_editor_data());
    assert!(scene.editor_data.is_none());
}

#[test]
fn test_editor_data_mutability() {
    let mut scene = SceneDefinition::new("Test Scene");
    let camera = EditorCamera::new();
    scene.set_editor_data(EditorData::new().with_camera(camera));

    // Get mutable reference
    let editor_data = scene.editor_data_mut().unwrap();
    editor_data.selected_entities.push("NewEntity".to_string());

    // Verify modification persisted
    let editor_data = scene.editor_data().unwrap();
    assert_eq!(editor_data.selected_entities.len(), 1);
    assert_eq!(editor_data.selected_entities[0], "NewEntity");
}

#[test]
fn test_editor_camera_orbit_parameters() {
    let camera = EditorCamera::orbit((1.0, 2.0, 3.0), 25.0, -0.6, 1.2);

    assert_eq!(camera.target, (1.0, 2.0, 3.0));
    assert_eq!(camera.distance, 25.0);
    assert!((camera.pitch - -0.6).abs() < 0.001);
    assert!((camera.yaw - 1.2).abs() < 0.001);
    assert_eq!(camera.mode, CameraMode::Orbit);
}

#[test]
fn test_editor_camera_free_parameters() {
    let camera = EditorCamera::free((5.0, 10.0, 15.0), 0.8, -0.4);

    assert_eq!(camera.position, (5.0, 10.0, 15.0));
    assert!((camera.pitch - 0.8).abs() < 0.001);
    assert!((camera.yaw - -0.4).abs() < 0.001);
    assert_eq!(camera.mode, CameraMode::Free);
}

#[test]
fn test_editor_camera_default_values() {
    let camera = EditorCamera::new();

    assert_eq!(camera.fov, 60.0);
    assert_eq!(camera.near_clip, 0.1);
    assert_eq!(camera.far_clip, 1000.0);
    assert_eq!(camera.distance, 10.0);
    assert_eq!(camera.mode, CameraMode::Orbit);
}

#[test]
fn test_viewport_settings_modification() {
    let mut viewport = ViewportSettings::new();

    viewport.show_grid = false;
    viewport.show_wireframe = true;
    viewport.grid_size = 50;
    viewport.grid_spacing = 0.5;
    viewport.background_color = (0.5, 0.5, 0.5);

    assert!(!viewport.show_grid);
    assert!(viewport.show_wireframe);
    assert_eq!(viewport.grid_size, 50);
    assert_eq!(viewport.grid_spacing, 0.5);
    assert_eq!(viewport.background_color, (0.5, 0.5, 0.5));
}

#[test]
fn test_editor_preferences_modification() {
    let mut prefs = EditorPreferences::new();

    prefs.auto_save_enabled = false;
    prefs.auto_save_interval = 120.0;
    prefs.snap_to_grid = true;
    prefs.snap_size = 2.0;
    prefs.rotation_snap = 30.0;
    prefs.last_asset_path = Some("assets/models".to_string());
    prefs
        .collapsed_hierarchy_nodes
        .push("ParentNode".to_string());

    assert!(!prefs.auto_save_enabled);
    assert_eq!(prefs.auto_save_interval, 120.0);
    assert!(prefs.snap_to_grid);
    assert_eq!(prefs.snap_size, 2.0);
    assert_eq!(prefs.rotation_snap, 30.0);
    assert_eq!(prefs.last_asset_path, Some("assets/models".to_string()));
    assert_eq!(prefs.collapsed_hierarchy_nodes.len(), 1);
}

#[test]
fn test_editor_data_serialization_roundtrip() {
    let test_dir = temp_test_dir();
    let save_path = test_dir.join("editor_roundtrip.ron");

    // Create complex editor data
    let mut scene = SceneDefinition::new("Complex Editor Scene");
    scene.add_entity(EntityDefinition::new().with_name("Entity1"));
    scene.add_entity(EntityDefinition::new().with_name("Entity2"));

    let camera = EditorCamera::orbit((5.0, 5.0, 5.0), 20.0, -0.5, 0.8);

    let mut viewport = ViewportSettings::new();
    viewport.show_wireframe = true;
    viewport.grid_size = 40;
    viewport.gizmo_mode = GizmoMode::Scale;

    let mut prefs = EditorPreferences::new();
    prefs.snap_to_grid = true;
    prefs.snap_size = 0.25;
    prefs.last_asset_path = Some("assets/test".to_string());

    let selected = vec!["Entity1".to_string()];

    let editor_data = EditorData::new()
        .with_camera(camera)
        .with_viewport(viewport)
        .with_preferences(prefs)
        .with_selected_entities(selected);

    scene.set_editor_data(editor_data);

    // Save
    let save_file = SaveFile::new(scene, SaveMetadata::new("Roundtrip Test"));
    let ron_string =
        ron::ser::to_string_pretty(&save_file, ron::ser::PrettyConfig::default()).unwrap();
    fs::write(&save_path, ron_string).unwrap();

    // Load
    let contents = fs::read_to_string(&save_path).unwrap();
    let loaded_file: SaveFile = ron::from_str(&contents).unwrap();

    // Verify everything
    let loaded_editor = loaded_file.scene.editor_data.as_ref().unwrap();

    // Camera
    let loaded_camera = loaded_editor.camera.as_ref().unwrap();
    assert_eq!(loaded_camera.target, (5.0, 5.0, 5.0));
    assert_eq!(loaded_camera.distance, 20.0);
    assert_eq!(loaded_camera.mode, CameraMode::Orbit);

    // Viewport
    let loaded_viewport = loaded_editor.viewport.as_ref().unwrap();
    assert!(loaded_viewport.show_wireframe);
    assert_eq!(loaded_viewport.grid_size, 40);
    assert_eq!(loaded_viewport.gizmo_mode, GizmoMode::Scale);

    // Preferences
    let loaded_prefs = loaded_editor.preferences.as_ref().unwrap();
    assert!(loaded_prefs.snap_to_grid);
    assert_eq!(loaded_prefs.snap_size, 0.25);
    assert_eq!(
        loaded_prefs.last_asset_path,
        Some("assets/test".to_string())
    );

    // Selection
    assert_eq!(loaded_editor.selected_entities.len(), 1);
    assert_eq!(loaded_editor.selected_entities[0], "Entity1");

    cleanup_test_dir(&test_dir);
}
