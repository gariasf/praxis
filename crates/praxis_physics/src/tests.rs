//! Unit tests for the physics system.
//!
//! These tests verify:
//! - Component creation and configuration
//! - Rigid body state changes and property updates
//! - Collision detection and event handling
//! - Spatial queries (raycasting, shape casting, point tests)
//! - Fixed timestep integration
//! - Transform synchronization between ECS and physics engine
//! - System execution and behavior

use super::*;
use praxis_ecs::{IntoSystemConfigs, Schedule, Transform, World};
use praxis_math::Vec3;

// ============================================================================
// COMPONENT CREATION TESTS
// ============================================================================

#[test]
fn test_rigid_body_component_creation() {
    // Test 1: Create different rigid body types and verify properties
    let dynamic = RigidBody::Dynamic;
    assert!(dynamic.is_dynamic());
    assert!(!dynamic.is_static());
    assert!(!dynamic.is_kinematic());

    let static_body = RigidBody::Static;
    assert!(!static_body.is_dynamic());
    assert!(static_body.is_static());
    assert!(!static_body.is_kinematic());

    let kinematic = RigidBody::Kinematic;
    assert!(!kinematic.is_dynamic());
    assert!(!kinematic.is_static());
    assert!(kinematic.is_kinematic());

    // Test 2: Verify default is Dynamic
    let default = RigidBody::default();
    assert!(default.is_dynamic());
}

#[test]
fn test_collider_component_creation() {
    // Test 1: Create cuboid collider
    let cuboid = Collider::cuboid(1.0, 2.0, 3.0);
    match cuboid {
        Collider::Cuboid { hx, hy, hz } => {
            assert_eq!(hx, 1.0);
            assert_eq!(hy, 2.0);
            assert_eq!(hz, 3.0);
        }
        _ => panic!("Expected Cuboid variant"),
    }

    // Test 2: Create sphere collider
    let sphere = Collider::sphere(5.0);
    match sphere {
        Collider::Sphere { radius } => {
            assert_eq!(radius, 5.0);
        }
        _ => panic!("Expected Sphere variant"),
    }

    // Test 3: Create capsule colliders
    let capsule_y = Collider::capsule_y(2.0, 0.5);
    match capsule_y {
        Collider::CapsuleY {
            half_height,
            radius,
        } => {
            assert_eq!(half_height, 2.0);
            assert_eq!(radius, 0.5);
        }
        _ => panic!("Expected CapsuleY variant"),
    }

    let capsule_x = Collider::capsule_x(1.5, 0.3);
    match capsule_x {
        Collider::CapsuleX {
            half_height,
            radius,
        } => {
            assert_eq!(half_height, 1.5);
            assert_eq!(radius, 0.3);
        }
        _ => panic!("Expected CapsuleX variant"),
    }

    let capsule_z = Collider::capsule_z(1.0, 0.4);
    match capsule_z {
        Collider::CapsuleZ {
            half_height,
            radius,
        } => {
            assert_eq!(half_height, 1.0);
            assert_eq!(radius, 0.4);
        }
        _ => panic!("Expected CapsuleZ variant"),
    }

    // Test 4: Create cylinder collider
    let cylinder = Collider::cylinder_y(3.0, 1.0);
    match cylinder {
        Collider::CylinderY {
            half_height,
            radius,
        } => {
            assert_eq!(half_height, 3.0);
            assert_eq!(radius, 1.0);
        }
        _ => panic!("Expected CylinderY variant"),
    }
}

#[test]
fn test_physics_velocity_component_creation() {
    // Test 1: Create velocity with only linear component
    let linear_only = PhysicsVelocity::linear(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(linear_only.linear, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(linear_only.angular, Vec3::ZERO);

    // Test 2: Create velocity with only angular component
    let angular_only = PhysicsVelocity::angular(Vec3::new(0.5, 1.0, 1.5));
    assert_eq!(angular_only.linear, Vec3::ZERO);
    assert_eq!(angular_only.angular, Vec3::new(0.5, 1.0, 1.5));

    // Test 3: Create velocity with both components
    let both = PhysicsVelocity::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.1, 0.2, 0.3));
    assert_eq!(both.linear, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(both.angular, Vec3::new(0.1, 0.2, 0.3));

    // Test 4: Verify default is zero velocity
    let default = PhysicsVelocity::default();
    assert_eq!(default.linear, Vec3::ZERO);
    assert_eq!(default.angular, Vec3::ZERO);
}

#[test]
fn test_external_forces_component() {
    // Test 1: Create default external forces (zero)
    let mut forces = ExternalForces::default();
    assert_eq!(forces.force, Vec3::ZERO);
    assert_eq!(forces.torque, Vec3::ZERO);

    // Test 2: Apply force
    forces.apply_force(Vec3::new(10.0, 0.0, 0.0));
    assert_eq!(forces.force, Vec3::new(10.0, 0.0, 0.0));

    // Test 3: Apply another force (should accumulate)
    forces.apply_force(Vec3::new(0.0, 5.0, 0.0));
    assert_eq!(forces.force, Vec3::new(10.0, 5.0, 0.0));

    // Test 4: Apply torque
    forces.apply_torque(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(forces.torque, Vec3::new(1.0, 2.0, 3.0));

    // Test 5: Apply force at point (generates both force and torque)
    let mut forces2 = ExternalForces::default();
    forces2.apply_force_at_point(
        Vec3::new(10.0, 0.0, 0.0), // Force
        Vec3::new(0.0, 1.0, 0.0),  // Point (lever arm)
    );
    assert_eq!(forces2.force, Vec3::new(10.0, 0.0, 0.0));
    // Torque = point.cross(force) = (0,1,0) × (10,0,0) = (0,0,-10)
    assert_eq!(forces2.torque, Vec3::new(0.0, 0.0, -10.0));

    // Test 6: Clear forces
    forces.clear();
    assert_eq!(forces.force, Vec3::ZERO);
    assert_eq!(forces.torque, Vec3::ZERO);
}

#[test]
fn test_mass_component() {
    // Test 1: Create mass with default inertia
    let mass1 = Mass::new(10.0);
    assert_eq!(mass1.mass, 10.0);
    assert_eq!(mass1.angular_inertia, 10.0);

    // Test 2: Create mass with custom inertia
    let mass2 = Mass::with_inertia(5.0, 2.5);
    assert_eq!(mass2.mass, 5.0);
    assert_eq!(mass2.angular_inertia, 2.5);

    // Test 3: Verify default mass is 1.0
    let default = Mass::default();
    assert_eq!(default.mass, 1.0);
    assert_eq!(default.angular_inertia, 1.0);
}

#[test]
fn test_friction_component() {
    // Test 1: Create friction with specific coefficient
    let friction = Friction::new(0.7);
    assert_eq!(friction.coefficient, 0.7);

    // Test 2: Verify default friction
    let default = Friction::default();
    assert_eq!(default.coefficient, 0.5);
}

#[test]
fn test_restitution_component() {
    // Test 1: Create restitution (bounciness)
    let bouncy = Restitution::new(0.8);
    assert_eq!(bouncy.coefficient, 0.8);

    // Test 2: No bounce
    let no_bounce = Restitution::new(0.0);
    assert_eq!(no_bounce.coefficient, 0.0);

    // Test 3: Perfect bounce
    let perfect = Restitution::new(1.0);
    assert_eq!(perfect.coefficient, 1.0);

    // Test 4: Verify default is no bounce
    let default = Restitution::default();
    assert_eq!(default.coefficient, 0.0);
}

#[test]
fn test_collision_groups_component() {
    // Test 1: Create collision groups with specific memberships and filters
    let groups = CollisionGroups::new(0b0001, 0b1110);
    assert_eq!(groups.memberships, 0b0001);
    assert_eq!(groups.filter, 0b1110);

    // Test 2: Create groups that collide with everything
    let all = CollisionGroups::all();
    assert_eq!(all.memberships, u32::MAX);
    assert_eq!(all.filter, u32::MAX);

    // Test 3: Create groups for specific group index
    let group5 = CollisionGroups::group(5);
    assert_eq!(group5.memberships, 1 << 5);
    assert_eq!(group5.filter, u32::MAX);

    // Test 4: Verify default is all groups
    let default = CollisionGroups::default();
    assert_eq!(default.memberships, u32::MAX);
    assert_eq!(default.filter, u32::MAX);
}

#[test]
fn test_sleeping_component() {
    // Test 1: Create default sleeping config
    let sleeping = Sleeping::default();
    assert!(sleeping.enabled);
    assert_eq!(sleeping.linear_threshold, 0.01);
    assert_eq!(sleeping.angular_threshold, 0.01);

    // Test 2: Create disabled sleeping
    let no_sleep = Sleeping::disabled();
    assert!(!no_sleep.enabled);

    // Test 3: Create with custom thresholds
    let custom = Sleeping::with_thresholds(0.1, 0.2);
    assert!(custom.enabled);
    assert_eq!(custom.linear_threshold, 0.1);
    assert_eq!(custom.angular_threshold, 0.2);
}

#[test]
fn test_locked_axes_component() {
    // Test 1: Create unlocked axes
    let unlocked = LockedAxes::new();
    assert!(!unlocked.lock_translation_x);
    assert!(!unlocked.lock_translation_y);
    assert!(!unlocked.lock_translation_z);
    assert!(!unlocked.lock_rotation_x);
    assert!(!unlocked.lock_rotation_y);
    assert!(!unlocked.lock_rotation_z);

    // Test 2: Lock all translation
    let trans_locked = LockedAxes::translation();
    assert!(trans_locked.lock_translation_x);
    assert!(trans_locked.lock_translation_y);
    assert!(trans_locked.lock_translation_z);
    assert!(!trans_locked.lock_rotation_x);

    // Test 3: Lock all rotation
    let rot_locked = LockedAxes::rotation();
    assert!(rot_locked.lock_rotation_x);
    assert!(rot_locked.lock_rotation_y);
    assert!(rot_locked.lock_rotation_z);
    assert!(!rot_locked.lock_translation_x);

    // Test 4: Builder pattern - lock specific axes
    let custom = LockedAxes::new()
        .lock_translation_y()
        .lock_rotation_x()
        .lock_rotation_z();
    assert!(!custom.lock_translation_x);
    assert!(custom.lock_translation_y);
    assert!(!custom.lock_translation_z);
    assert!(custom.lock_rotation_x);
    assert!(!custom.lock_rotation_y);
    assert!(custom.lock_rotation_z);
}

#[test]
fn test_collision_event_receiver() {
    // Test 1: Create empty receiver
    let receiver = CollisionEventReceiver::new();
    assert_eq!(receiver.event_count(), 0);
    assert!(!receiver.has_events());

    // Test 2: Add events
    let mut receiver = CollisionEventReceiver::new();
    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);

    receiver.add_event(CollisionEvent::CollisionStarted(entity1, entity2));
    assert_eq!(receiver.event_count(), 1);
    assert!(receiver.has_events());

    receiver.add_event(CollisionEvent::CollisionPersisted(entity1, entity2));
    assert_eq!(receiver.event_count(), 2);

    // Test 3: Clear events
    receiver.clear();
    assert_eq!(receiver.event_count(), 0);
    assert!(!receiver.has_events());
}

// ============================================================================
// RESOURCE TESTS
// ============================================================================

#[test]
fn test_physics_world_creation() {
    // Test 1: Create physics world
    let physics_world = PhysicsWorld::new();

    // Verify internal state is initialized (we can't check private fields,
    // but we can verify methods work)
    let test_entity = bevy_ecs::entity::Entity::from_raw(999);
    assert!(physics_world.get_body_handle(test_entity).is_none());
    assert!(physics_world.get_collider_handle(test_entity).is_none());
}

#[test]
fn test_physics_world_default() {
    let physics_world = PhysicsWorld::default();
    let test_entity = bevy_ecs::entity::Entity::from_raw(123);
    assert!(physics_world.get_body_handle(test_entity).is_none());
}

#[test]
fn test_physics_config() {
    // Test 1: Create default config (Earth gravity)
    let config = PhysicsConfig::default();
    assert_eq!(config.gravity, Vec3::new(0.0, -9.81, 0.0));
    assert_eq!(config.timestep, 1.0 / 60.0);

    // Test 2: Create zero gravity config (space)
    let space_config = PhysicsConfig::zero_gravity();
    assert_eq!(space_config.gravity, Vec3::ZERO);
    assert_eq!(space_config.timestep, 1.0 / 60.0);

    // Test 3: Create config with custom gravity
    let moon_config = PhysicsConfig::with_gravity(Vec3::new(0.0, -1.62, 0.0));
    assert_eq!(moon_config.gravity, Vec3::new(0.0, -1.62, 0.0));

    // Test 4: Create config with custom timestep
    let slow_config = PhysicsConfig::with_timestep(1.0 / 30.0);
    assert_eq!(slow_config.timestep, 1.0 / 30.0);
    assert_eq!(slow_config.gravity, Vec3::new(0.0, -9.81, 0.0));
}

#[test]
fn test_physics_time_accumulator() {
    // Test 1: Create physics time
    let mut time = PhysicsTime::new();
    assert_eq!(time.accumulator, 0.0);

    // Test 2: Add time (exactly one timestep)
    time.add(1.0 / 60.0);
    assert!((time.accumulator - 1.0 / 60.0).abs() < 0.0001);

    // Test 3: Check if should step (timestep = 1/60 = 0.01666...)
    assert!(time.should_step(1.0 / 60.0));

    // Test 4: Perform step
    time.step(1.0 / 60.0);
    assert!(time.accumulator < 0.0001); // Should be ~0 after step

    // Test 5: Add smaller amount (should not step yet)
    time.add(0.5 / 60.0); // Half frame
    assert!(!time.should_step(1.0 / 60.0));

    // Test 6: Add more time (should step now)
    time.add(0.5 / 60.0); // Another half frame
    assert!(time.should_step(1.0 / 60.0));
}

#[test]
fn test_contact_events() {
    // Test 1: Create empty contact events
    let mut events = ContactEvents::new();
    assert!(events.collision_started.is_empty());
    assert!(events.collision_stopped.is_empty());

    // Test 2: Add events
    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);
    events.collision_started.push((entity1, entity2));
    events.collision_stopped.push((entity1, entity2));

    assert_eq!(events.collision_started.len(), 1);
    assert_eq!(events.collision_stopped.len(), 1);

    // Test 3: Clear events
    events.clear();
    assert!(events.collision_started.is_empty());
    assert!(events.collision_stopped.is_empty());
}

// ============================================================================
// RIGID BODY STATE CHANGE TESTS
// ============================================================================

#[test]
fn test_rigid_body_creation_dynamic() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((Transform::from_xyz(1.0, 2.0, 3.0), RigidBody::Dynamic));

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_physics_transforms_system);
    schedule.run(world.inner_mut());

    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let body_handle = physics_world.get_body_handle(entity);
    assert!(body_handle.is_some(), "Dynamic body should be created");

    let body = physics_world
        .rigid_body_set
        .get(body_handle.unwrap())
        .unwrap();
    assert!(body.is_dynamic());
}

#[test]
fn test_rigid_body_creation_static() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((Transform::from_xyz(0.0, 0.0, 0.0), RigidBody::Static));

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_physics_transforms_system);
    schedule.run(world.inner_mut());

    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let body_handle = physics_world.get_body_handle(entity);
    assert!(body_handle.is_some(), "Static body should be created");

    let body = physics_world
        .rigid_body_set
        .get(body_handle.unwrap())
        .unwrap();
    assert!(body.is_fixed());
}

#[test]
fn test_rigid_body_creation_kinematic() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((Transform::from_xyz(5.0, 5.0, 5.0), RigidBody::Kinematic));

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_physics_transforms_system);
    schedule.run(world.inner_mut());

    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let body_handle = physics_world.get_body_handle(entity);
    assert!(body_handle.is_some(), "Kinematic body should be created");

    let body = physics_world
        .rigid_body_set
        .get(body_handle.unwrap())
        .unwrap();
    assert!(body.is_kinematic());
}

#[test]
fn test_kinematic_body_position_update() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((Transform::from_xyz(0.0, 0.0, 0.0), RigidBody::Kinematic));

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_physics_transforms_system);
    schedule.run(world.inner_mut());

    // Move kinematic body
    {
        let mut transform = world.inner_mut().get_mut::<Transform>(entity).unwrap();
        transform.translation = Vec3::new(10.0, 5.0, 3.0);
    }

    // Sync again
    schedule.run(world.inner_mut());

    // Verify physics body was updated
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let body_handle = physics_world.get_body_handle(entity).unwrap();
    let body = physics_world.rigid_body_set.get(body_handle).unwrap();
    let pos = body.position().translation;
    assert!((pos.x - 10.0).abs() < 0.01);
    assert!((pos.y - 5.0).abs() < 0.01);
    assert!((pos.z - 3.0).abs() < 0.01);
}

#[test]
fn test_dynamic_body_physics_updates_transform() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let entity = world.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PhysicsVelocity::default(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
            sync_physics_transforms_system,
        )
            .chain(),
    );

    // Run physics for several steps
    for _ in 0..5 {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0);
        drop(physics_time);
        schedule.run(world.inner_mut());
    }

    // Verify object fell due to gravity
    let transform = world.get::<Transform>(entity).unwrap();
    assert!(
        transform.translation.y < 10.0,
        "Dynamic body should fall under gravity"
    );
}

#[test]
fn test_velocity_component_updates_physics() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());

    let entity = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PhysicsVelocity::linear(Vec3::new(10.0, 0.0, 0.0)),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((sync_physics_transforms_system, sync_physics_properties).chain());
    schedule.run(world.inner_mut());

    // Check physics body has velocity
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let body_handle = physics_world.get_body_handle(entity).unwrap();
    let body = physics_world.rigid_body_set.get(body_handle).unwrap();
    let vel = body.linvel();
    assert!((vel.x - 10.0).abs() < 0.01);
    assert!(vel.y.abs() < 0.01);
    assert!(vel.z.abs() < 0.01);
}

#[test]
fn test_collider_synchronization() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(2.5),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((sync_physics_transforms_system, sync_colliders).chain());
    schedule.run(world.inner_mut());

    // Verify collider was created
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let collider_handle = physics_world.get_collider_handle(entity);
    assert!(collider_handle.is_some(), "Collider should be created");
}

#[test]
fn test_friction_and_restitution_sync() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        Friction::new(0.8),
        Restitution::new(0.9),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            sync_physics_properties,
        )
            .chain(),
    );
    schedule.run(world.inner_mut());

    // Verify properties were applied
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let collider_handle = physics_world.get_collider_handle(entity).unwrap();
    let collider = physics_world.collider_set.get(collider_handle).unwrap();
    assert!((collider.friction() - 0.8).abs() < 0.01);
    assert!((collider.restitution() - 0.9).abs() < 0.01);
}

// ============================================================================
// COLLISION DETECTION TESTS
// ============================================================================

#[test]
fn test_collision_event_receiver_system() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(ContactEvents::new());

    let entity1 = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        CollisionEventReceiver::new(),
    ));

    let entity2 = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        CollisionEventReceiver::new(),
    ));

    // Manually add collision event to ContactEvents resource
    {
        let mut contact_events = world.inner_mut().resource_mut::<ContactEvents>();
        contact_events.collision_started.push((entity1, entity2));
    }

    // Run populate_collision_events system
    let mut schedule = Schedule::default();
    schedule.add_systems(populate_collision_events);
    schedule.run(world.inner_mut());

    // Verify events were distributed to both entities
    let receiver1 = world.get::<CollisionEventReceiver>(entity1).unwrap();
    assert_eq!(receiver1.event_count(), 1);
    assert!(receiver1.has_events());

    let receiver2 = world.get::<CollisionEventReceiver>(entity2).unwrap();
    assert_eq!(receiver2.event_count(), 1);
    assert!(receiver2.has_events());

    // Clear events
    let mut clear_schedule = Schedule::default();
    clear_schedule.add_systems(clear_collision_event_receivers);
    clear_schedule.run(world.inner_mut());

    // Verify events were cleared
    let receiver1 = world.get::<CollisionEventReceiver>(entity1).unwrap();
    assert_eq!(receiver1.event_count(), 0);
    assert!(!receiver1.has_events());
}

#[test]
fn test_collision_event_types() {
    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);

    let started = CollisionEvent::CollisionStarted(entity1, entity2);
    let stopped = CollisionEvent::CollisionStopped(entity1, entity2);
    let persisted = CollisionEvent::CollisionPersisted(entity1, entity2);

    // Test equality
    assert_eq!(started, CollisionEvent::CollisionStarted(entity1, entity2));
    assert_ne!(started, stopped);
    assert_ne!(started, persisted);
}

#[test]
fn test_collision_stopped_event() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(ContactEvents::new());

    let entity1 = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        CollisionEventReceiver::new(),
    ));

    let entity2 = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        CollisionEventReceiver::new(),
    ));

    // Add stopped event
    {
        let mut contact_events = world.inner_mut().resource_mut::<ContactEvents>();
        contact_events.collision_stopped.push((entity1, entity2));
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(populate_collision_events);
    schedule.run(world.inner_mut());

    let receiver1 = world.get::<CollisionEventReceiver>(entity1).unwrap();
    assert_eq!(receiver1.event_count(), 1);
    match receiver1.events[0] {
        CollisionEvent::CollisionStopped(_, _) => {} // Expected
        _ => panic!("Expected CollisionStopped event"),
    }
}

#[test]
fn test_entity_without_receiver_ignored() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(ContactEvents::new());

    let entity1 = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        CollisionEventReceiver::new(),
    ));

    let entity2 = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        // No CollisionEventReceiver
    ));

    // Add collision event
    {
        let mut contact_events = world.inner_mut().resource_mut::<ContactEvents>();
        contact_events.collision_started.push((entity1, entity2));
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(populate_collision_events);
    schedule.run(world.inner_mut());

    // Entity1 should receive event
    let receiver1 = world.get::<CollisionEventReceiver>(entity1).unwrap();
    assert_eq!(receiver1.event_count(), 1);

    // Entity2 has no receiver, so nothing to check
}

#[test]
fn test_sensor_collider() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((
        Transform::default(),
        RigidBody::Static,
        Collider::cuboid(5.0, 5.0, 5.0),
        Sensor,
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((sync_physics_transforms_system, sync_colliders).chain());
    schedule.run(world.inner_mut());

    // Verify sensor collider was created
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let collider_handle = physics_world.get_collider_handle(entity).unwrap();
    let collider = physics_world.collider_set.get(collider_handle).unwrap();
    assert!(collider.is_sensor(), "Collider should be marked as sensor");
}

// ============================================================================
// SPATIAL QUERY TESTS
// ============================================================================

#[test]
fn test_raycast_basic() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Create static box at origin
    let _target = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    // Sync to create physics bodies
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
        )
            .chain(),
    );

    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
    physics_time.add(1.0 / 60.0);
    drop(physics_time);

    schedule.run(world.inner_mut());

    // Perform raycast from above pointing down
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let result = physics_world.raycast(
        Vec3::new(0.0, 10.0, 0.0), // Origin above target
        Vec3::new(0.0, -1.0, 0.0), // Direction down
        100.0,                     // Max distance
        true,                      // Solid
    );

    assert!(result.is_some(), "Raycast should hit the box");
    let (_entity, distance) = result.unwrap();
    assert!(
        distance > 8.0 && distance < 10.0,
        "Hit distance should be ~9.0 (10 - box half height)"
    );
}

#[test]
fn test_raycast_miss() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Create static box at origin
    let _target = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
        )
            .chain(),
    );

    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
    physics_time.add(1.0 / 60.0);
    drop(physics_time);

    schedule.run(world.inner_mut());

    // Raycast away from target
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let result = physics_world.raycast(
        Vec3::new(10.0, 0.0, 0.0), // Origin far away
        Vec3::new(1.0, 0.0, 0.0),  // Direction away from target
        100.0,
        true,
    );

    assert!(result.is_none(), "Raycast should miss the box");
}

#[test]
fn test_raycast_max_distance() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let _target = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::sphere(1.0),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
        )
            .chain(),
    );

    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
    physics_time.add(1.0 / 60.0);
    drop(physics_time);

    schedule.run(world.inner_mut());

    // Raycast with insufficient max distance
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let result = physics_world.raycast(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        5.0, // Too short to reach target
        true,
    );

    assert!(
        result.is_none(),
        "Raycast should not reach target with short max_distance"
    );
}

#[test]
fn test_point_inside_basic() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let target = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(5.0, 5.0, 5.0),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
        )
            .chain(),
    );

    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
    physics_time.add(1.0 / 60.0);
    drop(physics_time);

    schedule.run(world.inner_mut());

    // Test point inside box
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let result = physics_world.point_inside(Vec3::new(2.0, 2.0, 2.0));
    assert!(result.is_some(), "Point should be inside box");
    assert_eq!(result.unwrap(), target, "Should return the correct entity");
}

#[test]
fn test_point_outside() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let _target = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
        )
            .chain(),
    );

    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
    physics_time.add(1.0 / 60.0);
    drop(physics_time);

    schedule.run(world.inner_mut());

    // Test point outside box
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let result = physics_world.point_inside(Vec3::new(10.0, 10.0, 10.0));
    assert!(result.is_none(), "Point should be outside box");
}

#[test]
fn test_raycast_all_no_hits() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let results = physics_world.raycast_all(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        100.0,
        false,
    );

    // Should return empty when there are no objects
    assert!(results.is_empty());
}

#[test]
fn test_raycast_all_multiple_hits() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Create three boxes in a line
    let box1 = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    let box2 = world.spawn((
        Transform::from_xyz(5.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    let box3 = world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((
        cleanup_physics_entities,
        sync_physics_transforms_system,
        physics_step_system,
    ));

    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
    physics_time.add(1.0 / 60.0);
    drop(physics_time);

    schedule.run(world.inner_mut());

    // Raycast through all three boxes
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let results = physics_world.raycast_all(
        Vec3::new(-5.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        20.0,
        false,
    );

    // Should hit all three boxes
    assert_eq!(results.len(), 3, "Should hit all three boxes");
    
    // Results should be sorted by distance
    assert!(
        results[0].1 < results[1].1 && results[1].1 < results[2].1,
        "Results should be sorted by distance"
    );
    
    // Verify we hit the correct entities
    let hit_entities: Vec<_> = results.iter().map(|(e, _)| *e).collect();
    assert!(hit_entities.contains(&box1));
    assert!(hit_entities.contains(&box2));
    assert!(hit_entities.contains(&box3));
}

// ============================================================================
// FIXED TIMESTEP INTEGRATION TESTS
// ============================================================================

#[test]
fn test_fixed_timestep_no_step() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Don't add time - accumulator is zero
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);
    schedule.run(world.inner_mut());

    // Accumulator should still be zero (no time added, no step taken)
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert_eq!(physics_time.accumulator, 0.0);
}

#[test]
fn test_fixed_timestep_single_step() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Add exactly one timestep
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0);
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);
    schedule.run(world.inner_mut());

    // Accumulator should be ~0 after one step
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert!(physics_time.accumulator < 0.001);
}

#[test]
fn test_fixed_timestep_multiple_steps() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Add exactly 3 timesteps
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(3.0 / 60.0);
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);
    schedule.run(world.inner_mut());

    // Accumulator should be less than one timestep after consuming all 3 timesteps
    // (floating point precision may leave a small remainder)
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert!(
        physics_time.accumulator < 1.0 / 60.0,
        "Accumulator {} should be less than one timestep after 3 steps",
        physics_time.accumulator
    );
}

#[test]
fn test_fixed_timestep_partial_accumulation() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Add less than one timestep
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(0.5 / 60.0); // Half timestep
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);
    schedule.run(world.inner_mut());

    // Accumulator should retain the partial time
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert!((physics_time.accumulator - 0.5 / 60.0).abs() < 0.0001);
}

#[test]
fn test_fixed_timestep_accumulation_over_frames() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);

    // Frame 1: Add 0.5 timestep
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(0.5 / 60.0);
    }
    schedule.run(world.inner_mut());

    let time1 = world.inner_mut().resource::<PhysicsTime>().accumulator;
    assert!(
        (time1 - 0.5 / 60.0).abs() < 0.0001,
        "Should retain 0.5 timestep"
    );

    // Frame 2: Add another 0.7 timestep (total 1.2)
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(0.7 / 60.0);
    }
    schedule.run(world.inner_mut());

    let time2 = world.inner_mut().resource::<PhysicsTime>().accumulator;
    assert!(
        (time2 - 0.2 / 60.0).abs() < 0.0001,
        "Should retain 0.2 timestep after consuming 1.0"
    );
}

#[test]
fn test_fixed_timestep_with_simulation() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let entity = world.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PhysicsVelocity::default(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
            sync_physics_transforms_system,
        )
            .chain(),
    );

    let initial_y = world.get::<Transform>(entity).unwrap().translation.y;

    // Run multiple timesteps
    for _ in 0..10 {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0);
        drop(physics_time);
        schedule.run(world.inner_mut());
    }

    // Verify gravity affected the body
    let final_y = world.get::<Transform>(entity).unwrap().translation.y;
    assert!(
        final_y < initial_y,
        "Body should fall due to gravity in fixed timestep simulation"
    );
}

#[test]
fn test_physics_time_should_step() {
    let mut time = PhysicsTime::new();
    let timestep = 1.0 / 60.0;

    // No time accumulated
    assert!(!time.should_step(timestep));

    // Add exactly one timestep
    time.add(timestep);
    assert!(time.should_step(timestep));

    // Take the step
    time.step(timestep);
    assert!(!time.should_step(timestep));

    // Add two timesteps
    time.add(2.0 * timestep);
    assert!(time.should_step(timestep));
}

// ============================================================================
// CLEANUP AND ENTITY REMOVAL TESTS
// ============================================================================

#[test]
fn test_cleanup_physics_entities() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
    ));

    // Create physics body
    let mut schedule = Schedule::default();
    schedule.add_systems((sync_physics_transforms_system, sync_colliders).chain());
    schedule.run(world.inner_mut());

    // Verify body exists
    {
        let physics_world = world.inner_mut().resource::<PhysicsWorld>();
        assert!(physics_world.get_body_handle(entity).is_some());
    }

    // Despawn entity
    world.despawn(entity).expect("Failed to despawn entity");

    // Run cleanup system
    let mut cleanup_schedule = Schedule::default();
    cleanup_schedule.add_systems(cleanup_physics_entities);
    cleanup_schedule.run(world.inner_mut());

    // Verify physics body was removed
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    assert!(physics_world.get_body_handle(entity).is_none());
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_physics_pipeline() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Create ground (static body)
    let _ground = world.spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(10.0, 0.5, 10.0),
    ));

    // Create falling object (dynamic body)
    let falling_object = world.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        PhysicsVelocity::default(),
        CollisionEventReceiver::new(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            clear_collision_event_receivers,
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
            sync_physics_transforms_system,
            populate_collision_events,
        )
            .chain(),
    );

    // Run multiple physics steps
    for _ in 0..10 {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0);
        drop(physics_time);
        schedule.run(world.inner_mut());
    }

    // Verify object fell (Y position should be lower)
    let transform = world.get::<Transform>(falling_object).unwrap();
    assert!(
        transform.translation.y < 5.0,
        "Object should have fallen under gravity. Y: {}",
        transform.translation.y
    );
}

#[test]
fn test_multiple_dynamic_bodies() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Create multiple dynamic bodies
    let body1 = world.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PhysicsVelocity::default(),
    ));

    let body2 = world.spawn((
        Transform::from_xyz(5.0, 15.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PhysicsVelocity::default(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
            sync_physics_transforms_system,
        )
            .chain(),
    );

    let initial_y1 = world.get::<Transform>(body1).unwrap().translation.y;
    let initial_y2 = world.get::<Transform>(body2).unwrap().translation.y;

    // Simulate
    for _ in 0..5 {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0);
        drop(physics_time);
        schedule.run(world.inner_mut());
    }

    // Both bodies should fall
    let final_y1 = world.get::<Transform>(body1).unwrap().translation.y;
    let final_y2 = world.get::<Transform>(body2).unwrap().translation.y;

    assert!(final_y1 < initial_y1, "Body 1 should fall");
    assert!(final_y2 < initial_y2, "Body 2 should fall");
}

#[test]
fn test_external_forces_application() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    let entity = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        ExternalForces::default(),
    ));

    // Apply force
    {
        let mut forces = world.inner_mut().get_mut::<ExternalForces>(entity).unwrap();
        forces.apply_force(Vec3::new(100.0, 0.0, 0.0));
    }

    let mut schedule = Schedule::default();
    schedule.add_systems((sync_physics_transforms_system, apply_external_forces).chain());
    schedule.run(world.inner_mut());

    // Verify forces were cleared after application
    let forces = world.get::<ExternalForces>(entity).unwrap();
    assert_eq!(forces.force, Vec3::ZERO);
    assert_eq!(forces.torque, Vec3::ZERO);
}

#[test]
fn test_static_body_does_not_move() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    let static_entity = world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(5.0, 0.5, 5.0),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_physics_transforms_system,
            sync_colliders,
            physics_step_system,
            sync_physics_transforms_system,
        )
            .chain(),
    );

    let initial_pos = world.get::<Transform>(static_entity).unwrap().translation;

    // Run simulation
    for _ in 0..10 {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0);
        drop(physics_time);
        schedule.run(world.inner_mut());
    }

    // Static body should not move
    let final_pos = world.get::<Transform>(static_entity).unwrap().translation;
    assert_eq!(initial_pos, final_pos, "Static body should not move");
}
