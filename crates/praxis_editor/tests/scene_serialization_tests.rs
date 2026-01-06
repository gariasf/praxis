//! Comprehensive tests for scene serialization roundtrip functionality.
//!
//! Tests cover:
//! - Scene definition serialization and deserialization
//! - Roundtrip tests (save -> load -> verify)
//! - Entity hierarchy preservation
//! - Component data preservation
//! - Editor data serialization
//! - Migration and validation
//! - Complex scene structures

use praxis_scene::{
    CameraMode, EditorCamera, EditorData, EntityDefinition, GizmoMode, SceneDefinition,
    SceneLoader, TransformDef, ViewportSettings, CURRENT_SCENE_VERSION,
};

// ============================================================================
// Basic Serialization Tests
// ============================================================================

#[test]
fn test_scene_serialization_roundtrip() {
    let mut scene = SceneDefinition::new("Test Scene");
    scene.add_entity(
        EntityDefinition::new()
            .with_name("Entity1")
            .with_transform(TransformDef::from_translation(5.0, 10.0, 15.0))
            .with_mesh("cube"),
    );

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.name, scene.name);
    assert_eq!(loaded_scene.entity_count(), scene.entity_count());
    assert_eq!(loaded_scene.entities[0].name, scene.entities[0].name);
    assert_eq!(loaded_scene.entities[0].mesh.as_deref(), Some("cube"));
}

#[test]
fn test_empty_scene_roundtrip() {
    let scene = SceneDefinition::new("Empty Scene");

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.name, "Empty Scene");
    assert_eq!(loaded_scene.entity_count(), 0);
}

#[test]
fn test_scene_with_multiple_entities_roundtrip() {
    let mut scene = SceneDefinition::new("Multiple Entities");

    for i in 0..5 {
        scene.add_entity(
            EntityDefinition::new()
                .with_name(format!("Entity{i}"))
                .with_transform(TransformDef::from_translation(
                    i as f32,
                    i as f32 * 2.0,
                    i as f32 * 3.0,
                )),
        );
    }

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 5);

    for i in 0..5 {
        assert_eq!(
            loaded_scene.entities[i].name.as_deref(),
            Some(format!("Entity{i}").as_str())
        );
    }
}

// ============================================================================
// Hierarchy Preservation Tests
// ============================================================================

#[test]
fn test_hierarchy_roundtrip() {
    let mut scene = SceneDefinition::new("Hierarchy Test");

    let child = EntityDefinition::new()
        .with_name("Child")
        .with_transform(TransformDef::from_translation(1.0, 0.0, 0.0));

    let parent = EntityDefinition::new()
        .with_name("Parent")
        .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0))
        .with_child(child);

    scene.add_entity(parent);

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 1);
    assert_eq!(
        loaded_scene.total_entity_count(),
        scene.total_entity_count()
    );
    assert_eq!(loaded_scene.entities[0].children.len(), 1);
    assert_eq!(
        loaded_scene.entities[0].children[0].name.as_deref(),
        Some("Child")
    );
}

#[test]
fn test_complex_hierarchy_roundtrip() {
    let mut scene = SceneDefinition::new("Complex Hierarchy");

    let grandchild1 = EntityDefinition::new()
        .with_name("Grandchild1")
        .with_transform(TransformDef::from_translation(2.0, 0.0, 0.0));

    let grandchild2 = EntityDefinition::new()
        .with_name("Grandchild2")
        .with_transform(TransformDef::from_translation(3.0, 0.0, 0.0));

    let child1 = EntityDefinition::new()
        .with_name("Child1")
        .with_transform(TransformDef::from_translation(1.0, 0.0, 0.0))
        .with_child(grandchild1);

    let child2 = EntityDefinition::new()
        .with_name("Child2")
        .with_transform(TransformDef::from_translation(1.0, 1.0, 0.0))
        .with_child(grandchild2);

    let parent = EntityDefinition::new()
        .with_name("Parent")
        .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0))
        .with_child(child1)
        .with_child(child2);

    scene.add_entity(parent);

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 1);
    assert_eq!(loaded_scene.total_entity_count(), 5);
    assert_eq!(loaded_scene.entities[0].children.len(), 2);
    assert_eq!(loaded_scene.entities[0].children[0].children.len(), 1);
    assert_eq!(loaded_scene.entities[0].children[1].children.len(), 1);
}

#[test]
fn test_deep_hierarchy_roundtrip() {
    let mut scene = SceneDefinition::new("Deep Hierarchy");

    // Create a chain of 10 entities
    let mut current = EntityDefinition::new()
        .with_name("Entity9")
        .with_transform(TransformDef::from_translation(9.0, 0.0, 0.0));

    for i in (0..9).rev() {
        current = EntityDefinition::new()
            .with_name(format!("Entity{i}"))
            .with_transform(TransformDef::from_translation(i as f32, 0.0, 0.0))
            .with_child(current);
    }

    scene.add_entity(current);

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 1);
    assert_eq!(loaded_scene.total_entity_count(), 10);
}

// ============================================================================
// Component Data Preservation Tests
// ============================================================================

#[test]
fn test_transform_data_roundtrip() {
    let mut scene = SceneDefinition::new("Transform Test");
    let transform = TransformDef {
        translation: (1.0, 2.0, 3.0),
        rotation: (0.0, 0.707, 0.0, 0.707),
        scale: (2.0, 2.0, 2.0),
    };

    scene.add_entity(EntityDefinition::new().with_transform(transform));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    let loaded_transform = loaded_scene.entities[0].transform.unwrap();
    assert_eq!(loaded_transform.translation, (1.0, 2.0, 3.0));
    assert_eq!(loaded_transform.rotation, (0.0, 0.707, 0.0, 0.707));
    assert_eq!(loaded_transform.scale, (2.0, 2.0, 2.0));
}

#[test]
fn test_camera_data_roundtrip() {
    let mut scene = SceneDefinition::new("Camera Test");
    scene.add_entity(EntityDefinition::perspective_camera(
        "MainCamera",
        (0.0, 5.0, 10.0),
        1.22,
        1.77,
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 1);
    assert!(loaded_scene.entities[0].camera.is_some());

    let camera = loaded_scene.entities[0].camera.unwrap();
    assert_eq!(camera.fov, Some(1.22));
    assert_eq!(camera.aspect_ratio, Some(1.77));
}

#[test]
fn test_light_data_roundtrip() {
    let mut scene = SceneDefinition::new("Light Test");
    scene.add_entity(EntityDefinition::directional_light(
        "Sun",
        (0.0, -1.0, 0.0),
        (1.0, 1.0, 0.9),
        1.5,
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 1);
    assert!(loaded_scene.entities[0].directional_light.is_some());

    let light = loaded_scene.entities[0].directional_light.unwrap();
    assert_eq!(light.direction, (0.0, -1.0, 0.0));
    assert_eq!(light.color, (1.0, 1.0, 0.9));
    assert_eq!(light.intensity, 1.5);
}

#[test]
fn test_mesh_and_texture_roundtrip() {
    let mut scene = SceneDefinition::new("Mesh Test");
    scene.add_entity(EntityDefinition::textured_mesh_entity(
        "Cube",
        (0.0, 0.0, 0.0),
        "cube_mesh",
        "cube_texture",
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 1);
    assert_eq!(loaded_scene.entities[0].mesh.as_deref(), Some("cube_mesh"));
    assert_eq!(
        loaded_scene.entities[0].texture.as_deref(),
        Some("cube_texture")
    );
}

#[test]
fn test_visibility_and_active_roundtrip() {
    let mut scene = SceneDefinition::new("Visibility Test");
    let mut entity = EntityDefinition::new()
        .with_name("HiddenEntity")
        .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0));
    entity.visible = Some(false);
    entity.active = Some(false);
    scene.add_entity(entity);

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entities[0].visible, Some(false));
    assert_eq!(loaded_scene.entities[0].active, Some(false));
}

// ============================================================================
// Editor Data Tests
// ============================================================================

#[test]
fn test_editor_data_roundtrip() {
    let mut scene = SceneDefinition::new("Editor Scene");
    let editor_data = EditorData::new()
        .with_camera(EditorCamera::new())
        .with_selected_entities(vec!["Entity1".to_string()])
        .with_viewport(ViewportSettings::new());
    scene.set_editor_data(editor_data);

    scene.add_entity(EntityDefinition::mesh_entity(
        "Entity1",
        (0.0, 0.0, 0.0),
        "cube",
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert!(loaded_scene.has_editor_data());
    let editor = loaded_scene.editor_data().unwrap();
    assert!(editor.camera.is_some());
    assert_eq!(editor.selected_entities.len(), 1);
    assert!(editor.viewport.is_some());
}

#[test]
fn test_editor_camera_roundtrip() {
    let mut scene = SceneDefinition::new("Camera Test");
    let mut camera = EditorCamera::new();
    camera.position = (5.0, 10.0, 15.0);
    camera.target = (0.0, 1.0, 0.0);
    camera.distance = 20.0;
    camera.pitch = -0.5;
    camera.yaw = 1.2;
    camera.fov = 75.0;
    camera.mode = CameraMode::Free;

    scene.set_editor_data(EditorData::new().with_camera(camera));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    let loaded_camera = loaded_scene.editor_data().unwrap().camera.as_ref().unwrap();

    assert_eq!(loaded_camera.position, (5.0, 10.0, 15.0));
    assert_eq!(loaded_camera.target, (0.0, 1.0, 0.0));
    assert_eq!(loaded_camera.distance, 20.0);
    assert_eq!(loaded_camera.pitch, -0.5);
    assert_eq!(loaded_camera.yaw, 1.2);
    assert_eq!(loaded_camera.fov, 75.0);
    assert_eq!(loaded_camera.mode, CameraMode::Free);
}

#[test]
fn test_viewport_settings_roundtrip() {
    let mut scene = SceneDefinition::new("Viewport Test");
    let mut viewport = ViewportSettings::new();
    viewport.show_grid = false;
    viewport.show_wireframe = true;
    viewport.grid_size = 30;
    viewport.grid_spacing = 2.0;
    viewport.background_color = (0.2, 0.3, 0.4);
    viewport.gizmo_mode = GizmoMode::Scale;

    scene.set_editor_data(EditorData::new().with_viewport(viewport));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    let loaded_viewport = loaded_scene
        .editor_data()
        .unwrap()
        .viewport
        .as_ref()
        .unwrap();

    assert!(!loaded_viewport.show_grid);
    assert!(loaded_viewport.show_wireframe);
    assert_eq!(loaded_viewport.grid_size, 30);
    assert_eq!(loaded_viewport.grid_spacing, 2.0);
    assert_eq!(loaded_viewport.background_color, (0.2, 0.3, 0.4));
    assert_eq!(loaded_viewport.gizmo_mode, GizmoMode::Scale);
}

#[test]
fn test_scene_without_editor_data() {
    let mut scene = SceneDefinition::new("Runtime Scene");
    scene.add_entity(EntityDefinition::mesh_entity(
        "Entity1",
        (0.0, 0.0, 0.0),
        "cube",
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();

    // Verify editor_data is not in the serialized form
    assert!(!ron_string.contains("editor_data"));

    let loaded_scene = loader.load_from_string(&ron_string).unwrap();
    assert!(!loaded_scene.has_editor_data());
}

#[test]
fn test_runtime_scene_conversion() {
    let mut scene = SceneDefinition::new("Test Scene");
    scene.set_editor_data(EditorData::new().with_camera(EditorCamera::new()));
    scene.add_entity(EntityDefinition::mesh_entity(
        "Entity1",
        (0.0, 0.0, 0.0),
        "cube",
    ));

    assert!(scene.has_editor_data());

    let runtime_scene = scene.to_runtime_scene();

    assert!(!runtime_scene.has_editor_data());
    assert_eq!(runtime_scene.name, scene.name);
    assert_eq!(runtime_scene.entity_count(), scene.entity_count());
}

// ============================================================================
// Complex Scene Tests
// ============================================================================

#[test]
fn test_complex_scene_roundtrip() {
    let mut scene = SceneDefinition::new("Complex Scene");

    // Add camera
    scene.add_entity(EntityDefinition::perspective_camera(
        "MainCamera",
        (0.0, 5.0, 10.0),
        1.22,
        1.77,
    ));

    // Add directional light
    scene.add_entity(EntityDefinition::directional_light(
        "Sun",
        (0.0, -1.0, 0.0),
        (1.0, 1.0, 0.9),
        1.5,
    ));

    // Add mesh hierarchy
    let child =
        EntityDefinition::textured_mesh_entity("Child", (1.0, 0.0, 0.0), "cube", "texture1");

    let parent =
        EntityDefinition::textured_mesh_entity("Parent", (0.0, 0.0, 0.0), "cube", "texture2")
            .with_child(child);

    scene.add_entity(parent);

    // Add point light
    scene.add_entity(EntityDefinition::point_light(
        "Light",
        (5.0, 5.0, 5.0),
        (1.0, 0.8, 0.6),
        2.0,
        10.0,
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 4);
    assert_eq!(loaded_scene.total_entity_count(), 5);

    // Verify camera
    assert!(loaded_scene.entities[0].camera.is_some());

    // Verify directional light
    assert!(loaded_scene.entities[1].directional_light.is_some());

    // Verify mesh hierarchy
    assert_eq!(loaded_scene.entities[2].children.len(), 1);
    assert!(loaded_scene.entities[2].mesh.is_some());
    assert!(loaded_scene.entities[2].children[0].mesh.is_some());

    // Verify point light
    assert!(loaded_scene.entities[3].point_light.is_some());
}

#[test]
fn test_scene_with_metadata_roundtrip() {
    let mut scene = SceneDefinition::new("Metadata Scene");
    scene.metadata.description = Some("A test scene with metadata".to_string());
    scene.metadata.author = Some("Test Author".to_string());
    scene.metadata.version = Some("1.0.0".to_string());
    scene.metadata.tags = vec!["test".to_string(), "demo".to_string()];

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(
        loaded_scene.metadata.description.as_deref(),
        Some("A test scene with metadata")
    );
    assert_eq!(loaded_scene.metadata.author.as_deref(), Some("Test Author"));
    assert_eq!(loaded_scene.metadata.version.as_deref(), Some("1.0.0"));
    assert_eq!(loaded_scene.metadata.tags.len(), 2);
}

// ============================================================================
// Version and Migration Tests
// ============================================================================

#[test]
fn test_scene_version_preservation() {
    let scene = SceneDefinition::new("Version Test");

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.version, CURRENT_SCENE_VERSION);
}

#[test]
fn test_old_version_scene_migration() {
    // Simulate an old version scene (version 0) without version field
    let old_scene_ron = r#"
        (
            name: "Old Scene",
            entities: [
                (
                    name: Some("OldEntity"),
                    transform: Some((
                        translation: (1.0, 2.0, 3.0),
                        rotation: (0.0, 0.0, 0.0, 1.0),
                        scale: (1.0, 1.0, 1.0),
                    )),
                    children: [],
                ),
            ],
            metadata: (),
        )
        "#;

    let loader = SceneLoader::new();
    let loaded_scene = loader.load_from_string(old_scene_ron).unwrap();

    // Scene should be migrated to current version
    assert_eq!(loaded_scene.version, CURRENT_SCENE_VERSION);
    assert_eq!(loaded_scene.name, "Old Scene");
    assert_eq!(loaded_scene.entity_count(), 1);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_ron_handling() {
    let invalid_ron = "this is not valid RON";

    let loader = SceneLoader::new();
    let result = loader.load_from_string(invalid_ron);

    assert!(result.is_err());
}

#[test]
fn test_validation_invalid_camera() {
    // Scene with invalid camera (near > far)
    let invalid_scene_ron = r#"
        (
            version: 1,
            name: "Invalid Scene",
            entities: [
                (
                    name: Some("BadCamera"),
                    camera: Some((
                        camera_type: Perspective,
                        fov: Some(1.0),
                        aspect_ratio: Some(1.77),
                        near: 100.0,
                        far: 10.0,
                        is_active: true,
                        priority: 0,
                    )),
                    children: [],
                ),
            ],
            metadata: (),
        )
        "#;

    let loader = SceneLoader::new();
    let result = loader.load_from_string(invalid_scene_ron);

    // Should fail validation
    assert!(result.is_err());
}

// ============================================================================
// Large Scene Tests
// ============================================================================

#[test]
fn test_large_scene_roundtrip() {
    let mut scene = SceneDefinition::new("Large Scene");

    // Create 100 entities
    for i in 0..100 {
        scene.add_entity(
            EntityDefinition::new()
                .with_name(format!("Entity{i}"))
                .with_transform(TransformDef::from_translation(
                    i as f32,
                    i as f32 / 2.0,
                    i as f32 / 3.0,
                ))
                .with_mesh(format!("mesh_{}", i % 5)),
        );
    }

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 100);

    for i in 0..100 {
        assert_eq!(
            loaded_scene.entities[i].name.as_deref(),
            Some(format!("Entity{i}").as_str())
        );
    }
}

#[test]
fn test_scene_with_all_component_types() {
    let mut scene = SceneDefinition::new("All Components");

    // Entity with transform
    scene.add_entity(
        EntityDefinition::new()
            .with_name("TransformEntity")
            .with_transform(TransformDef::from_translation(1.0, 2.0, 3.0)),
    );

    // Entity with mesh and texture
    scene.add_entity(EntityDefinition::textured_mesh_entity(
        "MeshEntity",
        (4.0, 5.0, 6.0),
        "mesh",
        "texture",
    ));

    // Entity with perspective camera
    scene.add_entity(EntityDefinition::perspective_camera(
        "Camera",
        (7.0, 8.0, 9.0),
        1.5,
        1.77,
    ));

    // Entity with directional light
    scene.add_entity(EntityDefinition::directional_light(
        "DirLight",
        (0.0, -1.0, 0.0),
        (1.0, 1.0, 1.0),
        1.0,
    ));

    // Entity with point light
    scene.add_entity(EntityDefinition::point_light(
        "PointLight",
        (10.0, 11.0, 12.0),
        (1.0, 0.8, 0.6),
        2.5,
        15.0,
    ));

    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).unwrap();
    let loaded_scene = loader.load_from_string(&ron_string).unwrap();

    assert_eq!(loaded_scene.entity_count(), 5);

    // Verify each entity type
    assert!(loaded_scene.entities[0].transform.is_some());
    assert!(loaded_scene.entities[1].mesh.is_some());
    assert!(loaded_scene.entities[1].texture.is_some());
    assert!(loaded_scene.entities[2].camera.is_some());
    assert!(loaded_scene.entities[3].directional_light.is_some());
    assert!(loaded_scene.entities[4].point_light.is_some());
}
