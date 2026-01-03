//! Unit tests for the physics system.
//!
//! These tests verify:
//! - Component creation and configuration
//! - System execution and behavior
//! - Transform synchronization between ECS and physics engine
//! - Fixed timestep simulation
//! - Collision event handling

use super::*;
use praxis_ecs::{IntoSystemConfigs, Schedule, Transform, World};
use praxis_math::{Quat, Vec3};

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

    // Test 2: Add time
    time.add(0.016); // ~60fps frame
    assert!((time.accumulator - 0.016).abs() < 0.0001);

    // Test 3: Check if should step (timestep = 1/60 = 0.01666...)
    assert!(time.should_step(1.0 / 60.0));

    // Test 4: Perform step
    time.step(1.0 / 60.0);
    assert!(time.accumulator < 0.0001); // Should be ~0 after step

    // Test 5: Add smaller amount (should not step yet)
    time.add(0.008); // Half frame
    assert!(!time.should_step(1.0 / 60.0));

    // Test 6: Add more time (should step now)
    time.add(0.008); // Another half frame
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
// SYSTEM EXECUTION TESTS
// ============================================================================

#[test]
fn test_physics_step_system_execution() {
    // Test 1: Set up world with physics resources
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Test 2: Add time to trigger physics step
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(1.0 / 60.0); // Add one timestep worth of time
    }

    // Test 3: Create schedule with physics systems
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            clear_collision_event_receivers,
            sync_physics_transforms_system,
            physics_step_system,
            sync_physics_transforms_system,
            populate_collision_events,
        )
            .chain(),
    );

    // Test 4: Run schedule (should not panic)
    world.inner_mut().run_schedule(&mut schedule);

    // Test 5: Verify time was consumed
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert!(physics_time.accumulator < 0.0001); // Should be near zero after step
}

#[test]
fn test_transform_synchronization_dynamic_body() {
    // Test 1: Set up world
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Test 2: Spawn dynamic body at specific position
    let initial_pos = Vec3::new(0.0, 10.0, 0.0);
    let entity = world.spawn((
        Transform::from_xyz(initial_pos.x, initial_pos.y, initial_pos.z),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
        PhysicsVelocity::default(),
    ));

    // Test 3: Run sync to create physics body
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_physics_transforms_system);
    world.inner_mut().run_schedule(&mut schedule);

    // Test 4: Verify body was created in physics world
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    let body_handle = physics_world.get_body_handle(entity);
    assert!(body_handle.is_some(), "Physics body should be created");
}

#[test]
fn test_transform_synchronization_kinematic_body() {
    // Test 1: Set up world
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());

    // Test 2: Spawn kinematic body
    let initial_pos = Vec3::new(5.0, 0.0, 5.0);
    let entity = world.spawn((
        Transform::from_xyz(initial_pos.x, initial_pos.y, initial_pos.z),
        RigidBody::Kinematic,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    // Test 3: Run sync system
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_physics_transforms_system);
    world.inner_mut().run_schedule(&mut schedule);

    // Test 4: Move kinematic body
    {
        let mut transform = world.get_mut::<Transform>(entity).unwrap();
        transform.translation = Vec3::new(10.0, 0.0, 10.0);
    }

    // Test 5: Sync again - kinematic position should update
    world.inner_mut().run_schedule(&mut schedule);

    // Test 6: Verify position is maintained (kinematic bodies don't fall)
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation, Vec3::new(10.0, 0.0, 10.0));
}

#[test]
fn test_collision_event_receiver_system() {
    // Test 1: Set up world
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(ContactEvents::new());

    // Test 2: Spawn entities with event receivers
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

    // Test 3: Manually add collision event to ContactEvents resource
    {
        let mut contact_events = world.inner_mut().resource_mut::<ContactEvents>();
        contact_events.collision_started.push((entity1, entity2));
    }

    // Test 4: Run populate_collision_events system
    let mut schedule = Schedule::default();
    schedule.add_systems(populate_collision_events);
    world.inner_mut().run_schedule(&mut schedule);

    // Test 5: Verify events were distributed to both entities
    let receiver1 = world.get::<CollisionEventReceiver>(entity1).unwrap();
    assert_eq!(receiver1.event_count(), 1);
    assert!(receiver1.has_events());

    let receiver2 = world.get::<CollisionEventReceiver>(entity2).unwrap();
    assert_eq!(receiver2.event_count(), 1);
    assert!(receiver2.has_events());

    // Test 6: Clear events
    let mut clear_schedule = Schedule::default();
    clear_schedule.add_systems(clear_collision_event_receivers);
    world.inner_mut().run_schedule(&mut clear_schedule);

    // Test 7: Verify events were cleared
    let receiver1 = world.get::<CollisionEventReceiver>(entity1).unwrap();
    assert_eq!(receiver1.event_count(), 0);
    assert!(!receiver1.has_events());
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_physics_pipeline() {
    // Test 1: Set up complete physics scene
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Test 2: Create ground (static body)
    let _ground = world.spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(10.0, 0.5, 10.0),
    ));

    // Test 3: Create falling object (dynamic body)
    let falling_object = world.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0),
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        PhysicsVelocity::default(),
        CollisionEventReceiver::new(),
    ));

    // Test 4: Set up systems
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            clear_collision_event_receivers,
            sync_physics_transforms_system,
            physics_step_system,
            sync_physics_transforms_system,
            populate_collision_events,
        )
            .chain(),
    );

    // Test 5: Run multiple physics steps
    for _ in 0..10 {
        // Add time
        {
            let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
            physics_time.add(1.0 / 60.0);
        }

        // Run systems
        world.inner_mut().run_schedule(&mut schedule);
    }

    // Test 6: Verify object fell (Y position should be lower)
    let transform = world.get::<Transform>(falling_object).unwrap();
    assert!(
        transform.translation.y < 5.0,
        "Object should have fallen under gravity. Y: {}",
        transform.translation.y
    );
}

#[test]
fn test_fixed_timestep_accumulation() {
    // Test that physics runs correct number of steps based on accumulated time

    // Test 1: Set up world
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    // Test 2: Add exactly 2 timesteps worth of time
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(2.0 / 60.0); // Two timesteps at 60Hz
    }

    // Test 3: Run physics_step_system
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);
    world.inner_mut().run_schedule(&mut schedule);

    // Test 4: Verify accumulator is near zero (both steps consumed)
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert!(
        physics_time.accumulator < 0.0001,
        "Accumulator should be near zero after consuming 2 timesteps. Actual: {}",
        physics_time.accumulator
    );

    // Test 5: Add less than one timestep
    {
        let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
        physics_time.add(0.5 / 60.0); // Half a timestep
    }

    // Test 6: Run system again
    world.inner_mut().run_schedule(&mut schedule);

    // Test 7: Verify accumulator still has the half timestep
    let physics_time = world.inner_mut().resource::<PhysicsTime>();
    assert!(
        physics_time.accumulator > 0.008 && physics_time.accumulator < 0.009,
        "Accumulator should retain partial timestep. Actual: {}",
        physics_time.accumulator
    );
}
