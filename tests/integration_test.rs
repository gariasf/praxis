//! Integration tests for Praxis engine components.
//!
//! These tests verify that different crates work together correctly,
//! focusing on initialization flows, cross-crate interactions, and resource cleanup.

use praxis_utils::init;

/// Test that the tracing system initializes correctly and doesn't interfere
/// with other components.
#[test]
fn test_tracing_initialization() {
    // Note: praxis_utils::init() may succeed or fail depending on whether
    // another test has already initialized the global tracing subscriber.
    // Both outcomes are acceptable - what matters is it doesn't panic unexpectedly.
    let result = init();
    // Either success or an error about already being initialized is OK
    if let Err(e) = &result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("already") || error_str.contains("set"),
            "Unexpected initialization error: {:?}",
            e
        );
    }
}

/// Test initialization order across multiple subsystems.
#[test]
fn test_cross_crate_initialization_order() {
    // Utils init may fail if already initialized by another test, which is OK
    let _ = praxis_utils::init();

    let ecs_result = praxis_ecs::init();
    assert!(
        ecs_result.is_ok(),
        "ECS initialization should succeed after utils"
    );

    let input_result = praxis_input::init();
    assert!(
        input_result.is_ok(),
        "Input initialization should succeed after ECS"
    );

    let physics_result = praxis_physics::init();
    assert!(
        physics_result.is_ok(),
        "Physics initialization should succeed after input"
    );

    let assets_result = praxis_assets::init();
    assert!(
        assets_result.is_ok(),
        "Assets initialization should succeed after physics"
    );
}

/// Test that all subsystems can be initialized independently.
#[test]
fn test_independent_subsystem_initialization() {
    let ecs_result = praxis_ecs::init();
    assert!(ecs_result.is_ok(), "ECS should initialize independently");

    let input_result = praxis_input::init();
    assert!(
        input_result.is_ok(),
        "Input should initialize independently"
    );

    let physics_result = praxis_physics::init();
    assert!(
        physics_result.is_ok(),
        "Physics should initialize independently"
    );

    let assets_result = praxis_assets::init();
    assert!(
        assets_result.is_ok(),
        "Assets should initialize independently"
    );
}

/// Test that repeated initialization calls are safe.
#[test]
fn test_repeated_initialization_calls() {
    for _ in 0..5 {
        let result = praxis_ecs::init();
        assert!(result.is_ok(), "Repeated ECS init should be safe");
    }

    for _ in 0..5 {
        let result = praxis_input::init();
        assert!(result.is_ok(), "Repeated input init should be safe");
    }

    for _ in 0..5 {
        let result = praxis_physics::init();
        assert!(result.is_ok(), "Repeated physics init should be safe");
    }

    for _ in 0..5 {
        let result = praxis_assets::init();
        assert!(result.is_ok(), "Repeated assets init should be safe");
    }
}

/// Test ECS world creation and cleanup.
#[test]
fn test_ecs_world_creation_and_cleanup() {
    use praxis_ecs::{Transform, World};

    let mut world = World::new();

    let entity1 = world.spawn(Transform::default());
    let entity2 = world.spawn(Transform::default());

    assert!(world.get::<Transform>(entity1).is_some());
    assert!(world.get::<Transform>(entity2).is_some());

    let _ = world.despawn(entity1);
    assert!(world.get::<Transform>(entity1).is_none());
    assert!(world.get::<Transform>(entity2).is_some());

    world.clear_entities();
    assert!(world.get::<Transform>(entity2).is_none());
}

/// Test input state creation and cleanup.
#[test]
fn test_input_state_cleanup() {
    use praxis_input::InputState;
    use winit::keyboard::KeyCode;

    let mut input_state = InputState::new();

    input_state.press_key(KeyCode::KeyW);
    input_state.press_key(KeyCode::KeyA);
    assert!(input_state.is_key_pressed(KeyCode::KeyW));
    assert!(input_state.is_key_pressed(KeyCode::KeyA));

    input_state.release_key(KeyCode::KeyW);
    assert!(!input_state.is_key_pressed(KeyCode::KeyW));
    assert!(input_state.is_key_pressed(KeyCode::KeyA));

    input_state.clear();
    assert!(!input_state.is_key_pressed(KeyCode::KeyA));
}

/// Test physics world creation and cleanup.
#[test]
fn test_physics_world_cleanup() {
    use praxis_ecs::World;
    use praxis_physics::{Collider, PhysicsWorld, RigidBody};

    let mut world = World::new();
    let physics_world = PhysicsWorld::new();

    world.insert_resource(physics_world);

    let entity = world.spawn((RigidBody::Dynamic, Collider::sphere(1.0)));

    assert!(world.get::<RigidBody>(entity).is_some());
    assert!(world.get::<Collider>(entity).is_some());

    let _ = world.despawn(entity);
    assert!(world.get::<RigidBody>(entity).is_none());
}

/// Test asset loading and proper resource management.
#[test]
fn test_asset_loading_cleanup() {
    use std::fs;

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("integration_test_mesh.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok(), "Asset loading should succeed");

    let mesh = result.unwrap();
    assert_eq!(mesh.positions.len(), 3);
    assert_eq!(mesh.indices.len(), 3);

    fs::remove_file(&test_file).ok();
}

/// Test scene graph with ECS integration and cleanup.
#[test]
fn test_scene_ecs_integration_cleanup() {
    use praxis_ecs::{GlobalTransform, Parent, Transform, World};
    use praxis_math::Vec3;

    let mut world = World::new();

    let parent = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    let child = world.spawn((
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(parent),
    ));

    assert!(world.get::<Transform>(parent).is_some());
    assert!(world.get::<Transform>(child).is_some());
    assert!(world.get::<Parent>(child).is_some());

    let _ = world.despawn(child);
    assert!(world.get::<Transform>(child).is_none());
    assert!(world.get::<Transform>(parent).is_some());

    world.clear_entities();
    assert!(world.get::<Transform>(parent).is_none());
}

/// Test multiple worlds and resource isolation.
#[test]
fn test_multiple_worlds_isolation() {
    use praxis_ecs::{Transform, World};
    use praxis_math::Vec3;

    let mut world1 = World::new();
    let mut world2 = World::new();

    let entity1 = world1.spawn(Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)));
    let entity2 = world2.spawn(Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)));

    let data1 = world1.get::<Transform>(entity1).unwrap();
    let data2 = world2.get::<Transform>(entity2).unwrap();

    assert_eq!(data1.translation.x, 1.0);
    assert_eq!(data2.translation.x, 2.0);

    // Note: We don't test cross-world entity lookups here because entity IDs
    // are generated independently per world and may overlap. The key isolation
    // property is that entities in each world have their own data.
}

/// Test that asset loader can be used multiple times without issues.
#[test]
fn test_asset_loader_reuse() {
    use praxis_assets::{AssetLoader, MeshLoader};
    use std::fs;

    let loader = MeshLoader::new();

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();

    for i in 0..5 {
        let test_file = temp_dir.join(format!("reuse_test_{}.obj", i));
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Loader should work multiple times");

        fs::remove_file(&test_file).ok();
    }
}

/// Test cross-crate type compatibility (Transform with Physics).
#[test]
fn test_transform_physics_compatibility() {
    use praxis_ecs::{GlobalTransform, Transform, World};
    use praxis_math::Vec3;
    use praxis_physics::{Collider, RigidBody};

    let mut world = World::new();

    let entity = world.spawn((
        Transform::from_translation(Vec3::new(5.0, 10.0, 15.0)),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
    ));

    let transform = world.get::<Transform>(entity).unwrap();
    let rigid_body = world.get::<RigidBody>(entity).unwrap();

    assert_eq!(transform.translation, Vec3::new(5.0, 10.0, 15.0));
    assert!(rigid_body.is_dynamic());
}

/// Test error propagation across crate boundaries.
#[test]
fn test_error_handling_across_crates() {
    let result = praxis_assets::load_obj("definitely_does_not_exist_xyz123.obj");
    assert!(result.is_err());

    let error_message = result.unwrap_err().to_string();
    assert!(
        !error_message.is_empty(),
        "Error message should be descriptive"
    );
}

/// Test resource insertion and removal in ECS.
#[test]
fn test_ecs_resource_lifecycle() {
    use praxis_ecs::World;
    use praxis_input::InputState;

    let mut world = World::new();

    assert!(!world.contains_resource::<InputState>());

    world.insert_resource(InputState::new());
    assert!(world.contains_resource::<InputState>());

    {
        let _input_state = world.get_resource::<InputState>().unwrap();
    }

    world.remove_resource::<InputState>();
    assert!(!world.contains_resource::<InputState>());
}

/// Test that assets can be loaded with different path types.
#[test]
fn test_asset_path_flexibility() {
    use std::fs;
    use std::path::PathBuf;

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("path_flex_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result1 = praxis_assets::load_obj(&test_file);
    assert!(result1.is_ok(), "Should load with &Path");

    let path_buf: PathBuf = test_file.clone();
    let result2 = praxis_assets::load_obj(path_buf);
    assert!(result2.is_ok(), "Should load with PathBuf");

    let path_str = test_file.to_str().unwrap();
    let result3 = praxis_assets::load_obj(path_str);
    assert!(result3.is_ok(), "Should load with &str");

    fs::remove_file(&test_file).ok();
}

/// Test concurrent world operations don't interfere with each other.
#[test]
fn test_concurrent_world_operations() {
    use praxis_ecs::{Transform, World};
    use praxis_math::Vec3;

    let mut world1 = World::new();
    let mut world2 = World::new();

    for i in 0..100 {
        world1.spawn(Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)));
        world2.spawn(Transform::from_translation(Vec3::new(
            (i * 2) as f32,
            0.0,
            0.0,
        )));
    }

    let count1: Vec<_> = world1
        .query::<&Transform>()
        .iter(world1.inner())
        .map(|t| t.translation.x)
        .collect();

    let count2: Vec<_> = world2
        .query::<&Transform>()
        .iter(world2.inner())
        .map(|t| t.translation.x)
        .collect();

    assert_eq!(count1.len(), 100);
    assert_eq!(count2.len(), 100);
    assert_ne!(count1, count2);
}
