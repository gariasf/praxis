//! Physics systems for the Praxis ECS.
//!
//! This module provides systems that integrate Rapier physics simulation
//! with the Praxis ECS architecture.

use praxis_ecs::{Commands, Entity, Query, Res, ResMut, Transform, With};
use praxis_math::{Quat, Vec3};
use rapier3d::prelude::{
    ColliderBuilder, Isometry, RigidBodyBuilder, RigidBodyType, SharedShape, vector, nalgebra,
};

use crate::components::{
    Collider as PraxisCollider, ExternalForces, Friction, Restitution,
    RigidBody as PraxisRigidBody, Sensor, PhysicsVelocity,
};
use crate::resources::{PhysicsWorld, PhysicsConfig, ContactEvents};

/// Synchronizes ECS transforms to Rapier rigid bodies.
///
/// This system runs before the physics step and updates Rapier body positions
/// based on ECS Transform components. This allows kinematic bodies to be moved
/// via Transform changes and ensures dynamic bodies start at the correct position.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::sync_transforms_to_physics;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(sync_transforms_to_physics);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_transforms_to_physics(
    mut physics_world: ResMut<PhysicsWorld>,
    query: Query<(Entity, &Transform, &PraxisRigidBody), With<PraxisRigidBody>>,
) {
    for (entity, transform, rigid_body) in query.iter() {
        // Get or create rigid body handle
        let body_handle = if let Some(handle) = physics_world.get_body_handle(entity) {
            handle
        } else {
            // Create new rigid body
            let rapier_body_type = match rigid_body {
                PraxisRigidBody::Dynamic => RigidBodyType::Dynamic,
                PraxisRigidBody::Static => RigidBodyType::Fixed,
                PraxisRigidBody::Kinematic => RigidBodyType::KinematicPositionBased,
            };

            let rapier_body = RigidBodyBuilder::new(rapier_body_type)
                .translation(vector![
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z
                ])
                .rotation(vector![
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z
                ] * transform.rotation.w)
                .build();

            let handle = physics_world.rigid_body_set.insert(rapier_body);
            physics_world.entity_to_body.insert(entity, handle);
            physics_world.body_to_entity.insert(handle, entity);
            handle
        };

        // Update position for kinematic bodies
        if rigid_body.is_kinematic() {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                body.set_position(
                    Isometry::new(
                        vector![
                            transform.translation.x,
                            transform.translation.y,
                            transform.translation.z
                        ],
                        vector![
                            transform.rotation.x,
                            transform.rotation.y,
                            transform.rotation.z
                        ] * transform.rotation.w,
                    ),
                    true,
                );
            }
        }
    }
}

/// Steps the physics simulation forward by the configured timestep.
///
/// This system runs the Rapier physics pipeline, advancing the simulation
/// by one timestep. It should run after `sync_transforms_to_physics` and
/// before `sync_transforms_from_physics`.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::{sync_transforms_to_physics, step_physics_simulation, sync_transforms_from_physics};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     sync_transforms_to_physics,
///     step_physics_simulation,
///     sync_transforms_from_physics,
/// ).chain());
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn step_physics_simulation(
    mut physics_world: ResMut<PhysicsWorld>,
    config: Res<PhysicsConfig>,
    mut contact_events: ResMut<ContactEvents>,
) {
    // Clear previous contact events
    contact_events.clear();

    // Update integration parameters from config
    physics_world.integration_parameters.dt = config.timestep;

    // Set gravity
    let gravity = vector![config.gravity.x, config.gravity.y, config.gravity.z];

    // Create event handler for collision events
    let event_handler = ();

    // Step the physics simulation
    // Need to destructure to avoid borrowing issues
    let PhysicsWorld {
        ref mut rigid_body_set,
        ref mut collider_set,
        ref integration_parameters,
        ref mut physics_pipeline,
        ref mut island_manager,
        ref mut broad_phase,
        ref mut narrow_phase,
        ref mut impulse_joint_set,
        ref mut multibody_joint_set,
        ref mut ccd_solver,
        ref mut query_pipeline,
        ..
    } = *physics_world;
    
    physics_pipeline.step(
        &gravity,
        integration_parameters,
        island_manager,
        broad_phase,
        narrow_phase,
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        Some(query_pipeline),
        &(),
        &event_handler,
    );
}

/// Synchronizes Rapier rigid body positions back to ECS transforms.
///
/// This system runs after the physics step and updates ECS Transform components
/// based on Rapier body positions. This allows dynamic bodies to move based on
/// physics simulation.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::{sync_transforms_to_physics, step_physics_simulation, sync_transforms_from_physics};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     sync_transforms_to_physics,
///     step_physics_simulation,
///     sync_transforms_from_physics,
/// ).chain());
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_transforms_from_physics(
    physics_world: Res<PhysicsWorld>,
    mut query: Query<(Entity, &mut Transform, &PraxisRigidBody), With<PraxisRigidBody>>,
) {
    for (entity, mut transform, rigid_body) in &mut query {
        // Only update transforms for dynamic bodies
        if !rigid_body.is_dynamic() {
            continue;
        }

        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                let position = body.position();
                let translation = position.translation;
                let rotation = position.rotation;

                transform.translation = Vec3::new(translation.x, translation.y, translation.z);

                // Convert rotation to quaternion
                let axis = rotation.scaled_axis();
                let angle = axis.norm();
                if angle > 0.0 {
                    let axis_normalized = axis / angle;
                    transform.rotation = Quat::from_axis_angle(
                        Vec3::new(axis_normalized.x, axis_normalized.y, axis_normalized.z),
                        angle,
                    );
                }
            }
        }
    }
}

/// Applies external forces to rigid bodies.
///
/// This system takes forces and torques from `ExternalForces` components
/// and applies them to the corresponding Rapier rigid bodies, then clears
/// the accumulators.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::apply_external_forces;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(apply_external_forces);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn apply_external_forces(
    mut physics_world: ResMut<PhysicsWorld>,
    mut query: Query<(Entity, &mut ExternalForces), With<PraxisRigidBody>>,
) {
    for (entity, mut forces) in &mut query {
        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                // Apply force
                if forces.force.length_squared() > 0.0 {
                    body.add_force(
                        vector![forces.force.x, forces.force.y, forces.force.z],
                        true,
                    );
                }

                // Apply torque
                if forces.torque.length_squared() > 0.0 {
                    body.add_torque(
                        vector![forces.torque.x, forces.torque.y, forces.torque.z],
                        true,
                    );
                }

                // Clear forces after applying
                forces.clear();
            }
        }
    }
}

/// Creates or updates colliders for entities with Collider components.
///
/// This system checks for entities with Collider components and ensures they
/// have corresponding Rapier colliders attached to their rigid bodies.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::sync_colliders;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(sync_colliders);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_colliders(
    mut physics_world: ResMut<PhysicsWorld>,
    query: Query<(Entity, &PraxisCollider, Option<&Sensor>), With<PraxisRigidBody>>,
) {
    for (entity, collider, sensor) in &query {
        // Skip if collider already exists
        if physics_world.get_collider_handle(entity).is_some() {
            continue;
        }

        // Get rigid body handle
        let Some(body_handle) = physics_world.get_body_handle(entity) else { continue };

        // Create Rapier collider shape
        let shape: SharedShape = match collider {
            PraxisCollider::Cuboid { hx, hy, hz } => SharedShape::cuboid(*hx, *hy, *hz),
            PraxisCollider::Sphere { radius } => SharedShape::ball(*radius),
            PraxisCollider::CapsuleY {
                half_height,
                radius,
            } => SharedShape::capsule_y(*half_height, *radius),
            PraxisCollider::CapsuleX {
                half_height,
                radius,
            } => SharedShape::capsule_x(*half_height, *radius),
            PraxisCollider::CapsuleZ {
                half_height,
                radius,
            } => SharedShape::capsule_z(*half_height, *radius),
            PraxisCollider::CylinderY {
                half_height,
                radius,
            } => SharedShape::cylinder(*half_height, *radius),
        };

        // Build collider
        let mut collider_builder = ColliderBuilder::new(shape);

        // Set as sensor if component present
        if sensor.is_some() {
            collider_builder = collider_builder.sensor(true);
        }

        let rapier_collider = collider_builder.build();

        // Insert collider - need separate borrows to avoid conflict
        let PhysicsWorld {
            ref mut collider_set,
            ref mut rigid_body_set,
            ref mut entity_to_collider,
            ..
        } = *physics_world;
        
        let collider_handle = collider_set.insert_with_parent(
            rapier_collider,
            body_handle,
            rigid_body_set,
        );

        entity_to_collider.insert(entity, collider_handle);
    }
}

/// Updates rigid body properties from components.
///
/// This system synchronizes physics properties (mass, velocity, friction, etc.)
/// from ECS components to Rapier rigid bodies.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
/// use praxis_physics::sync_physics_properties;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(sync_physics_properties);
/// ```
#[allow(clippy::needless_pass_by_value)]
pub fn sync_physics_properties(
    mut physics_world: ResMut<PhysicsWorld>,
    velocity_query: Query<(Entity, &PhysicsVelocity), With<PraxisRigidBody>>,
    friction_query: Query<(Entity, &Friction), With<PraxisCollider>>,
    restitution_query: Query<(Entity, &Restitution), With<PraxisCollider>>,
) {
    // Sync velocities
    for (entity, velocity) in &velocity_query {
        if let Some(body_handle) = physics_world.get_body_handle(entity) {
            if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                body.set_linvel(
                    vector![velocity.linear.x, velocity.linear.y, velocity.linear.z],
                    true,
                );
                body.set_angvel(
                    vector![velocity.angular.x, velocity.angular.y, velocity.angular.z],
                    true,
                );
            }
        }
    }

    // Sync friction
    for (entity, friction) in &friction_query {
        if let Some(collider_handle) = physics_world.get_collider_handle(entity) {
            if let Some(collider) = physics_world.collider_set.get_mut(collider_handle) {
                collider.set_friction(friction.coefficient);
            }
        }
    }

    // Sync restitution
    for (entity, restitution) in &restitution_query {
        if let Some(collider_handle) = physics_world.get_collider_handle(entity) {
            if let Some(collider) = physics_world.collider_set.get_mut(collider_handle) {
                collider.set_restitution(restitution.coefficient);
            }
        }
    }
}

/// Removes physics bodies and colliders for despawned entities.
///
/// This system should be run to clean up Rapier resources when ECS entities
/// with physics components are despawned.
///
/// Note: This is a placeholder. Proper cleanup requires tracking entity
/// despawn events, which may need integration with `bevy_ecs` removal detection.
pub const fn cleanup_physics_entities(_commands: Commands, _physics_world: ResMut<PhysicsWorld>) {
    // Placeholder for cleanup logic
    // In a full implementation, this would detect removed RigidBody components
    // and clean up the corresponding Rapier handles
}
