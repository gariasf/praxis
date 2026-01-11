//! Resource cleanup and lifecycle management tests.
//!
//! These tests verify that resources are properly managed across
//! different subsystems and cleaned up correctly.

use praxis_ecs::{Component, Resource};
use serial_test::serial;
use std::fs;

/// Test ECS entity lifecycle with multiple component types.
#[test]
fn test_entity_lifecycle_with_multiple_components() {
    use praxis_ecs::{GlobalTransform, Parent, Transform, World};

    let mut world = World::new();

    let parent_entity = world.spawn(Transform::default());

    let entity = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Parent(parent_entity),
    ));

    assert!(world.get::<Transform>(entity).is_some());
    assert!(world.get::<GlobalTransform>(entity).is_some());
    assert!(world.get::<Parent>(entity).is_some());

    world.entity_mut(entity).remove::<Parent>();
    assert!(world.get::<Parent>(entity).is_none());
    assert!(world.get::<Transform>(entity).is_some());

    let _ = world.despawn(entity);
    assert!(world.get::<Transform>(entity).is_none());
    assert!(world.get::<GlobalTransform>(entity).is_none());
}

/// Test physics resource cleanup.
#[test]
#[serial]
fn test_physics_resource_cleanup() {
    use praxis_ecs::World;
    use praxis_physics::{Collider, PhysicsWorld, RigidBody};

    let mut world = World::new();
    let physics_world = PhysicsWorld::new();
    world.insert_resource(physics_world);

    let entities: Vec<_> = (0..10)
        .map(|_| world.spawn((RigidBody::Dynamic, Collider::sphere(1.0))))
        .collect();

    for entity in &entities {
        assert!(world.get::<RigidBody>(*entity).is_some());
        assert!(world.get::<Collider>(*entity).is_some());
    }

    for entity in &entities[0..5] {
        let _ = world.despawn(*entity);
    }

    for entity in &entities[0..5] {
        assert!(world.get::<RigidBody>(*entity).is_none());
    }

    for entity in &entities[5..10] {
        assert!(world.get::<RigidBody>(*entity).is_some());
    }

    world.remove_resource::<PhysicsWorld>();
    assert!(!world.contains_resource::<PhysicsWorld>());
}

/// Test input state cleanup and reset.
#[test]
fn test_input_state_reset() {
    use praxis_input::InputState;
    use winit::keyboard::KeyCode;

    let mut input_state = InputState::new();

    input_state.press_key(KeyCode::KeyW);
    input_state.press_key(KeyCode::KeyA);
    input_state.press_key(KeyCode::KeyS);
    input_state.press_key(KeyCode::KeyD);

    assert!(input_state.is_key_pressed(KeyCode::KeyW));
    assert!(input_state.is_key_pressed(KeyCode::KeyA));
    assert!(input_state.is_key_pressed(KeyCode::KeyS));
    assert!(input_state.is_key_pressed(KeyCode::KeyD));

    input_state.clear();

    assert!(!input_state.is_key_pressed(KeyCode::KeyW));
    assert!(!input_state.is_key_pressed(KeyCode::KeyA));
    assert!(!input_state.is_key_pressed(KeyCode::KeyS));
    assert!(!input_state.is_key_pressed(KeyCode::KeyD));
}

/// Test scene graph cleanup with parent-child relationships.
#[test]
fn test_scene_graph_cleanup() {
    use praxis_ecs::{GlobalTransform, Parent, Transform, World};
    use praxis_math::Vec3;

    let mut world = World::new();

    let root = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    let child1 = world.spawn((
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(root),
    ));

    let child2 = world.spawn((
        Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Parent(root),
    ));

    let grandchild = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        GlobalTransform::default(),
        Parent(child1),
    ));

    assert!(world.get::<Parent>(child1).is_some());
    assert!(world.get::<Parent>(child2).is_some());
    assert!(world.get::<Parent>(grandchild).is_some());

    let _ = world.despawn(child1);
    assert!(world.get::<Transform>(child1).is_none());
    assert!(world.get::<Transform>(root).is_some());
    assert!(world.get::<Transform>(child2).is_some());

    let _ = world.despawn(root);
    assert!(world.get::<Transform>(root).is_none());
}

/// Test resource cleanup with multiple resource types.
#[test]
fn test_multiple_resource_cleanup() {
    use praxis_ecs::World;
    use praxis_input::InputState;
    use praxis_physics::PhysicsWorld;

    #[derive(Resource)]
    #[allow(dead_code)]
    struct GameState {
        level: u32,
    }

    #[derive(Resource)]
    #[allow(dead_code)]
    struct AudioManager {
        volume: f32,
    }

    let mut world = World::new();

    world.insert_resource(GameState { level: 1 });
    world.insert_resource(AudioManager { volume: 0.8 });
    world.insert_resource(InputState::new());
    world.insert_resource(PhysicsWorld::new());

    assert!(world.contains_resource::<GameState>());
    assert!(world.contains_resource::<AudioManager>());
    assert!(world.contains_resource::<InputState>());
    assert!(world.contains_resource::<PhysicsWorld>());

    world.remove_resource::<GameState>();
    assert!(!world.contains_resource::<GameState>());
    assert!(world.contains_resource::<AudioManager>());

    world.remove_resource::<AudioManager>();
    world.remove_resource::<InputState>();
    world.remove_resource::<PhysicsWorld>();

    assert!(!world.contains_resource::<AudioManager>());
    assert!(!world.contains_resource::<InputState>());
    assert!(!world.contains_resource::<PhysicsWorld>());
}

/// Test asset loading and unloading pattern.
#[test]
fn test_asset_load_unload_pattern() {
    let temp_dir = std::env::temp_dir();
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;

    let files: Vec<_> = (0..5)
        .map(|i| {
            let file = temp_dir.join(format!("asset_lifecycle_{i}.obj"));
            fs::write(&file, obj_content).expect("Failed to write test file");
            file
        })
        .collect();

    let mut loaded_meshes = Vec::new();
    for file in &files {
        let mesh = praxis_assets::load_obj(file).expect("Failed to load mesh");
        loaded_meshes.push(mesh);
    }

    assert_eq!(loaded_meshes.len(), 5);

    loaded_meshes.clear();
    assert_eq!(loaded_meshes.len(), 0);

    for file in files {
        fs::remove_file(&file).ok();
    }
}

/// Test that world clearing removes all entities and their components.
#[test]
fn test_world_clear_all() {
    use praxis_ecs::World;

    #[derive(Component)]
    #[allow(dead_code)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Component)]
    #[allow(dead_code)]
    struct Name {
        value: String,
    }

    let mut world = World::new();

    for i in 0..100 {
        world.spawn((
            Position {
                x: i as f32,
                y: i as f32 * 2.0,
            },
            Name {
                value: format!("Entity_{i}"),
            },
        ));
    }

    let count_before = world.query::<&Position>().iter(world.inner()).count();
    assert_eq!(count_before, 100);

    world.clear_entities();

    let count_after = world.query::<&Position>().iter(world.inner()).count();
    assert_eq!(count_after, 0);
}

/// Test cleanup of entities with physics components.
#[test]
#[serial]
fn test_physics_entity_cleanup() {
    use praxis_ecs::World;
    use praxis_physics::{Collider, ExternalForces, PhysicsVelocity, RigidBody};

    let mut world = World::new();

    let entity = world.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        PhysicsVelocity::default(),
        ExternalForces::default(),
    ));

    assert!(world.get::<RigidBody>(entity).is_some());
    assert!(world.get::<Collider>(entity).is_some());
    assert!(world.get::<PhysicsVelocity>(entity).is_some());
    assert!(world.get::<ExternalForces>(entity).is_some());

    world.entity_mut(entity).remove::<ExternalForces>();
    assert!(world.get::<ExternalForces>(entity).is_none());
    assert!(world.get::<RigidBody>(entity).is_some());

    let _ = world.despawn(entity);
    assert!(world.get::<RigidBody>(entity).is_none());
    assert!(world.get::<Collider>(entity).is_none());
    assert!(world.get::<PhysicsVelocity>(entity).is_none());
}

/// Test that temporary test files are cleaned up properly.
#[test]
fn test_temporary_file_cleanup() {
    let temp_dir = std::env::temp_dir();
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;

    let test_files: Vec<_> = (0..10)
        .map(|i| temp_dir.join(format!("temp_cleanup_{i}.obj")))
        .collect();

    for file in &test_files {
        fs::write(file, obj_content).expect("Failed to write test file");
        assert!(file.exists(), "File should exist after creation");
    }

    for file in &test_files {
        let _mesh = praxis_assets::load_obj(file).expect("Failed to load mesh");
    }

    for file in &test_files {
        fs::remove_file(file).expect("Failed to remove file");
        assert!(!file.exists(), "File should not exist after removal");
    }
}

/// Test batch entity spawning and cleanup.
#[test]
fn test_batch_entity_operations() {
    use praxis_ecs::World;

    #[derive(Component)]
    #[allow(dead_code)]
    struct BatchId {
        id: usize,
    }

    let mut world = World::new();

    let batch1: Vec<_> = (0..50).map(|i| world.spawn(BatchId { id: i })).collect();

    let batch2: Vec<_> = (50..100).map(|i| world.spawn(BatchId { id: i })).collect();

    let total_count = world.query::<&BatchId>().iter(world.inner()).count();
    assert_eq!(total_count, 100);

    for entity in batch1 {
        let _ = world.despawn(entity);
    }

    let remaining_count = world.query::<&BatchId>().iter(world.inner()).count();
    assert_eq!(remaining_count, 50);

    for entity in batch2 {
        let _ = world.despawn(entity);
    }

    let final_count = world.query::<&BatchId>().iter(world.inner()).count();
    assert_eq!(final_count, 0);
}

/// Test resource replacement pattern.
#[test]
fn test_resource_replacement() {
    use praxis_ecs::World;

    #[derive(Resource)]
    struct Counter {
        value: i32,
    }

    let mut world = World::new();

    world.insert_resource(Counter { value: 1 });
    assert_eq!(world.get_resource::<Counter>().unwrap().value, 1);

    world.insert_resource(Counter { value: 2 });
    assert_eq!(world.get_resource::<Counter>().unwrap().value, 2);

    world.insert_resource(Counter { value: 3 });
    assert_eq!(world.get_resource::<Counter>().unwrap().value, 3);

    world.remove_resource::<Counter>();
    assert!(!world.contains_resource::<Counter>());
}

/// Test that components can be added and removed dynamically.
#[test]
fn test_dynamic_component_management() {
    use praxis_ecs::World;

    #[derive(Component)]
    struct Active;

    #[derive(Component)]
    struct Visible;

    #[derive(Component)]
    struct Selected;

    let mut world = World::new();

    let entity = world.spawn(Active);

    assert!(world.get::<Active>(entity).is_some());
    assert!(world.get::<Visible>(entity).is_none());
    assert!(world.get::<Selected>(entity).is_none());

    world.entity_mut(entity).insert(Visible);
    assert!(world.get::<Visible>(entity).is_some());

    world.entity_mut(entity).insert(Selected);
    assert!(world.get::<Selected>(entity).is_some());

    world.entity_mut(entity).remove::<Active>();
    assert!(world.get::<Active>(entity).is_none());
    assert!(world.get::<Visible>(entity).is_some());
    assert!(world.get::<Selected>(entity).is_some());

    world.entity_mut(entity).remove::<Visible>();
    world.entity_mut(entity).remove::<Selected>();
    assert!(world.get::<Visible>(entity).is_none());
    assert!(world.get::<Selected>(entity).is_none());
}

/// Test cleanup pattern for game loop simulation.
#[test]
fn test_game_loop_cleanup_pattern() {
    use praxis_ecs::World;

    #[derive(Component)]
    #[allow(dead_code)]
    struct Enemy {
        health: f32,
    }

    #[derive(Resource)]
    #[allow(dead_code)]
    struct FrameCount {
        count: u32,
    }

    let mut world = World::new();
    world.insert_resource(FrameCount { count: 0 });

    for frame in 0..10 {
        for _ in 0..5 {
            world.spawn(Enemy { health: 100.0 });
        }

        {
            let frame_count = world.get_resource_mut::<FrameCount>();
            if let Some(fc) = frame_count {
                fc.count = frame;
            }
        }

        let enemy_count = world.query::<&Enemy>().iter(world.inner()).count();
        assert_eq!(enemy_count, (frame + 1) as usize * 5);
    }

    world.clear_entities();
    let final_count = world.query::<&Enemy>().iter(world.inner()).count();
    assert_eq!(final_count, 0);

    assert!(world.contains_resource::<FrameCount>());
}
