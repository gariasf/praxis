//! Integration tests for exclusive World access in script systems.
//!
//! These tests verify that Lua scripts can spawn/despawn entities and modify
//! components through the `script_start_system` and `script_update_system`.

use praxis_ecs::{DeltaTime, GlobalTransform, Name, Schedule, Transform, World};
use praxis_scripting::{
    script_initialization_system, script_start_system, script_update_system, ScriptComponent,
    ScriptingConfig, ScriptingContext, ScriptingResource,
};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

/// Helper to create a temporary Lua script file.
fn create_temp_script(dir: &TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(format!("{name}.lua"));
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path.to_str().unwrap().to_string()
}

/// Helper to setup a world with scripting system.
fn setup_world_with_scripting() -> (World, TempDir) {
    let mut world = World::new();
    let temp_dir = TempDir::new().unwrap();

    let config = ScriptingConfig::default();
    let context = ScriptingContext::new(config).unwrap();
    let resource = ScriptingResource::new(context);
    world.inner_mut().insert_resource(resource);
    world.inner_mut().insert_resource(DeltaTime(0.016));

    (world, temp_dir)
}

#[test]
fn test_spawn_entity_in_on_start() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Create a script that spawns an entity in on_start
    let script_content = r#"
function on_start()
    local entity = world.spawn()
    world.add_component_name(entity, "StartSpawned")
    world.add_component_transform(entity, 1.0, 2.0, 3.0)
end
"#;
    let script_path = create_temp_script(&temp_dir, "spawn_test", script_content);

    // Add script component to an entity
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("spawn_test", script_path),
    ));

    // Run initialization system via schedule
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());

    // Run start system
    script_start_system(&mut world);

    // Verify entity was spawned
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query
        .iter(world.inner())
        .filter(|(name, _)| name.as_str() == "StartSpawned")
        .collect();

    assert_eq!(entities.len(), 1);
    let (_, transform) = entities[0];
    assert_eq!(transform.translation.x, 1.0);
    assert_eq!(transform.translation.y, 2.0);
    assert_eq!(transform.translation.z, 3.0);
}

#[test]
fn test_spawn_entity_in_on_update() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Create a script that spawns an entity on first update
    let script_content = r#"
local spawned = false

function on_start()
    spawned = false
end

function on_update(delta_time)
    if not spawned then
        local entity = world.spawn()
        world.add_component_name(entity, "UpdateSpawned")
        world.add_component_transform(entity, 5.0, 6.0, 7.0)
        spawned = true
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "update_spawn_test", script_content);

    // Add script component to an entity
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("update_spawn_test", script_path),
    ));

    // Run initialization and start systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Run update system
    script_update_system(&mut world);

    // Verify entity was spawned
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query
        .iter(world.inner())
        .filter(|(name, _)| name.as_str() == "UpdateSpawned")
        .collect();

    assert_eq!(entities.len(), 1);
    let (_, transform) = entities[0];
    assert_eq!(transform.translation.x, 5.0);
    assert_eq!(transform.translation.y, 6.0);
    assert_eq!(transform.translation.z, 7.0);
}

#[test]
fn test_despawn_entity() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Pre-spawn an entity to be despawned
    let target_entity = world.spawn((
        Name::new("ToBeDeleted"),
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Create a script that despawns the entity
    let script_content = r#"
function on_start()
    local entity = world.get_entity_by_name("ToBeDeleted")
    if entity then
        world.despawn(entity)
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "despawn_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("despawn_test", script_path),
    ));

    // Verify entity exists before script runs
    assert!(world.inner().get_entity(target_entity).is_some());

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Verify entity was despawned
    assert!(world.inner().get_entity(target_entity).is_none());
}

#[test]
fn test_modify_component_transform() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Pre-spawn an entity with a transform
    world.spawn((
        Name::new("MovableEntity"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    // Create a script that modifies the transform
    let script_content = r#"
function on_update(delta_time)
    local entity = world.get_entity_by_name("MovableEntity")
    if entity then
        local transform = world.get_component_transform(entity)
        local translation = transform.translation
        translation.x = translation.x + 1.0
        translation.y = translation.y + 2.0
        translation.z = translation.z + 3.0
        transform.translation = translation
        world.set_component_transform(entity, transform)
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "modify_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("modify_test", script_path),
    ));

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Get initial transform
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let initial_transform = query
        .iter(world.inner())
        .find(|(name, _)| name.as_str() == "MovableEntity")
        .map(|(_, t)| *t)
        .unwrap();

    // Run update system multiple times
    for _ in 0..3 {
        script_update_system(&mut world);
    }

    // Verify transform was modified
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let final_transform = query
        .iter(world.inner())
        .find(|(name, _)| name.as_str() == "MovableEntity")
        .map(|(_, t)| *t)
        .unwrap();

    assert_eq!(
        final_transform.translation.x,
        initial_transform.translation.x + 3.0
    );
    assert_eq!(
        final_transform.translation.y,
        initial_transform.translation.y + 6.0
    );
    assert_eq!(
        final_transform.translation.z,
        initial_transform.translation.z + 9.0
    );
}

#[test]
fn test_spawn_multiple_entities() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Create a script that spawns multiple entities
    let script_content = r#"
function on_start()
    for i = 1, 5 do
        local entity = world.spawn()
        world.add_component_name(entity, "Entity_" .. i)
        world.add_component_transform(entity, i * 1.0, i * 2.0, i * 3.0)
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "multi_spawn_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("multi_spawn_test", script_path),
    ));

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Verify all entities were spawned
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query
        .iter(world.inner())
        .filter(|(name, _)| name.as_str().starts_with("Entity_"))
        .collect();

    assert_eq!(entities.len(), 5);

    // Verify positions
    for i in 1..=5 {
        let entity_name = format!("Entity_{i}");
        let (_, transform) = entities
            .iter()
            .find(|(name, _)| name.as_str() == entity_name)
            .unwrap();

        let expected_x = i as f32 * 1.0;
        let expected_y = i as f32 * 2.0;
        let expected_z = i as f32 * 3.0;

        assert_eq!(transform.translation.x, expected_x);
        assert_eq!(transform.translation.y, expected_y);
        assert_eq!(transform.translation.z, expected_z);
    }
}

#[test]
fn test_query_and_modify_entities() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Pre-spawn entities with specific names
    for i in 1..=3 {
        world.spawn((
            Name::new(format!("Target_{i}")),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
    }

    // Create a script that queries and modifies entities
    let script_content = r#"
function on_update(delta_time)
    for i = 1, 3 do
        local name = "Target_" .. i
        local entity = world.get_entity_by_name(name)
        if entity then
            local transform = world.get_component_transform(entity)
            local translation = transform.translation
            translation.y = translation.y + delta_time * 10.0
            transform.translation = translation
            world.set_component_transform(entity, transform)
        end
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "query_modify_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("query_modify_test", script_path),
    ));

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Run update system
    script_update_system(&mut world);

    // Verify all entities were modified
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query
        .iter(world.inner())
        .filter(|(name, _)| name.as_str().starts_with("Target_"))
        .collect();

    assert_eq!(entities.len(), 3);

    // Each entity should have moved by delta_time * 10.0 (0.016 * 10.0 = 0.16)
    for (_, transform) in entities {
        assert!(transform.translation.y > 0.0);
        assert!((transform.translation.y - 0.16).abs() < 0.001);
    }
}

#[test]
fn test_add_component_to_existing_entity() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Pre-spawn an entity without components
    world.spawn(Name::new("Bare"));

    // Create a script that adds a transform component
    let script_content = r#"
function on_start()
    local entity = world.get_entity_by_name("Bare")
    if entity then
        world.add_component_transform(entity, 10.0, 20.0, 30.0)
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "add_component_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("add_component_test", script_path),
    ));

    // Verify entity has no Transform initially
    let mut query = world.inner_mut().query::<(&Name, Option<&Transform>)>();
    let (_, initial_transform) = query
        .iter(world.inner())
        .find(|(name, _)| name.as_str() == "Bare")
        .unwrap();
    assert!(initial_transform.is_none());

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Verify transform was added
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let (_, transform) = query
        .iter(world.inner())
        .find(|(name, _)| name.as_str() == "Bare")
        .unwrap();

    assert_eq!(transform.translation.x, 10.0);
    assert_eq!(transform.translation.y, 20.0);
    assert_eq!(transform.translation.z, 30.0);
}

#[test]
fn test_multiple_scripts_with_world_access() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Create first script that spawns entities with prefix "A"
    let script1_content = r#"
function on_start()
    for i = 1, 2 do
        local entity = world.spawn()
        world.add_component_name(entity, "A_" .. i)
        world.add_component_transform(entity, 1.0, 0.0, 0.0)
    end
end
"#;
    let script1_path = create_temp_script(&temp_dir, "script1", script1_content);

    // Create second script that spawns entities with prefix "B"
    let script2_content = r#"
function on_start()
    for i = 1, 2 do
        local entity = world.spawn()
        world.add_component_name(entity, "B_" .. i)
        world.add_component_transform(entity, 2.0, 0.0, 0.0)
    end
end
"#;
    let script2_path = create_temp_script(&temp_dir, "script2", script2_content);

    // Add both script components
    world.spawn((
        Name::new("ScriptEntity1"),
        ScriptComponent::new("script1", script1_path),
    ));
    world.spawn((
        Name::new("ScriptEntity2"),
        ScriptComponent::new("script2", script2_path),
    ));

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // Verify entities from both scripts were spawned
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query.iter(world.inner()).collect();

    let a_entities: Vec<_> = entities
        .iter()
        .filter(|(name, _)| name.as_str().starts_with("A_"))
        .collect();
    let b_entities: Vec<_> = entities
        .iter()
        .filter(|(name, _)| name.as_str().starts_with("B_"))
        .collect();

    assert_eq!(a_entities.len(), 2);
    assert_eq!(b_entities.len(), 2);

    // Verify transforms
    for (_, transform) in a_entities {
        assert_eq!(transform.translation.x, 1.0);
    }
    for (_, transform) in b_entities {
        assert_eq!(transform.translation.x, 2.0);
    }
}

#[test]
fn test_despawn_and_respawn_cycle() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Create a script that spawns, despawns, and respawns entities over updates
    let script_content = r#"
local state = "spawn"
local entity = nil

function on_start()
    state = "spawn"
end

function on_update(delta_time)
    if state == "spawn" then
        entity = world.spawn()
        world.add_component_name(entity, "Cyclic")
        world.add_component_transform(entity, 1.0, 1.0, 1.0)
        state = "despawn"
    elseif state == "despawn" then
        if entity then
            world.despawn(entity)
            entity = nil
        end
        state = "respawn"
    elseif state == "respawn" then
        entity = world.spawn()
        world.add_component_name(entity, "Cyclic")
        world.add_component_transform(entity, 2.0, 2.0, 2.0)
        state = "done"
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "cycle_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("cycle_test", script_path),
    ));

    // Run systems
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());
    script_start_system(&mut world);

    // First update: spawn
    script_update_system(&mut world);
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query
        .iter(world.inner())
        .filter(|(name, _)| name.as_str() == "Cyclic")
        .collect();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].1.translation.x, 1.0);

    // Second update: despawn
    script_update_system(&mut world);
    let mut query = world.inner_mut().query::<&Name>();
    let count = query
        .iter(world.inner())
        .filter(|name| name.as_str() == "Cyclic")
        .count();
    assert_eq!(count, 0);

    // Third update: respawn
    script_update_system(&mut world);
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let entities: Vec<_> = query
        .iter(world.inner())
        .filter(|(name, _)| name.as_str() == "Cyclic")
        .collect();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].1.translation.x, 2.0);
}

#[test]
fn test_entity_name_modification() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Pre-spawn an entity
    world.spawn((
        Name::new("OldName"),
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Create a script that spawns a new entity and renames the old one via despawn/respawn
    let script_content = r#"
function on_start()
    -- Get the old entity
    local old_entity = world.get_entity_by_name("OldName")
    
    -- Get its transform
    local transform = nil
    if old_entity then
        transform = world.get_component_transform(old_entity)
        -- Despawn it
        world.despawn(old_entity)
    end
    
    -- Create a new entity with new name and same transform
    local new_entity = world.spawn()
    world.add_component_name(new_entity, "NewName")
    if transform then
        world.set_component_transform(new_entity, transform)
    else
        world.add_component_transform(new_entity, 0.0, 0.0, 0.0)
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "rename_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("rename_test", script_path),
    ));

    // Verify old name exists
    let mut query = world.inner_mut().query::<&Name>();
    let old_count = query
        .iter(world.inner())
        .filter(|name| name.as_str() == "OldName")
        .count();
    assert_eq!(old_count, 1);

    // Run initialization system via schedule
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());

    // Run start system
    script_start_system(&mut world);

    // Verify old name is gone and new name exists
    let mut query = world.inner_mut().query::<&Name>();
    let old_count = query
        .iter(world.inner())
        .filter(|name| name.as_str() == "OldName")
        .count();
    let new_count = query
        .iter(world.inner())
        .filter(|name| name.as_str() == "NewName")
        .count();
    assert_eq!(old_count, 0);
    assert_eq!(new_count, 1);
}

#[test]
fn test_continuous_entity_modification() {
    let (mut world, temp_dir) = setup_world_with_scripting();

    // Pre-spawn an entity
    world.spawn((
        Name::new("Mover"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    // Create a script that continuously modifies the entity's position
    let script_content = r#"
function on_update(delta_time)
    local entity = world.get_entity_by_name("Mover")
    if entity then
        local transform = world.get_component_transform(entity)
        local translation = transform.translation
        translation.x = translation.x + delta_time
        transform.translation = translation
        world.set_component_transform(entity, transform)
    end
end
"#;
    let script_path = create_temp_script(&temp_dir, "continuous_test", script_content);

    // Add script component
    world.spawn((
        Name::new("ScriptEntity"),
        ScriptComponent::new("continuous_test", script_path),
    ));

    // Run initialization system via schedule
    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(script_initialization_system);
    init_schedule.run(world.inner_mut());

    // Run start system
    script_start_system(&mut world);

    // Run update system 10 times
    for _ in 0..10 {
        script_update_system(&mut world);
    }

    // Verify position accumulated correctly (0.016 * 10 = 0.16)
    let mut query = world.inner_mut().query::<(&Name, &Transform)>();
    let (_, transform) = query
        .iter(world.inner())
        .find(|(name, _)| name.as_str() == "Mover")
        .unwrap();

    let expected = 0.016 * 10.0;
    assert!((transform.translation.x - expected).abs() < 0.001);
}
