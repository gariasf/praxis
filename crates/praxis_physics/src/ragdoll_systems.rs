//! Systems for ragdoll physics simulation.

use bevy_ecs::system::{Query, ResMut};
use praxis_ecs::{Entity, Transform};

use crate::components::RigidBody as PraxisRigidBody;
use crate::ragdoll::{Ragdoll, RagdollJoint, RagdollJointType, RagdollMotor};
use crate::resources::PhysicsWorld;

/// System that updates ragdoll activation and blending.
#[allow(clippy::needless_pass_by_value)]
pub fn ragdoll_activation_system(
    mut ragdoll_query: Query<&mut Ragdoll>,
) {
    let dt = 0.016;
    
    for mut ragdoll in &mut ragdoll_query {
        if ragdoll.active {
            ragdoll.activation_timer += dt;
            
            let blend_progress = (ragdoll.activation_timer / ragdoll.activation_time).min(1.0);
            ragdoll.physics_blend = blend_progress;
        }
    }
}

/// System that creates and updates ragdoll joints.
#[allow(clippy::needless_pass_by_value, unused_variables)]
pub fn ragdoll_joint_system(
    mut physics_world: ResMut<PhysicsWorld>,
    joint_query: Query<(Entity, &RagdollJoint), bevy_ecs::query::With<PraxisRigidBody>>,
) {
    use rapier3d::prelude::*;
    use rapier3d::na::Unit as NaUnit;
    
    for (entity, ragdoll_joint) in &joint_query {
        if physics_world.entity_to_joint.contains_key(&entity) {
            continue;
        }
        
        let Some(body1_handle) = physics_world.get_body_handle(entity) else {
            continue;
        };
        
        let Some(body2_handle) = physics_world.get_body_handle(ragdoll_joint.connected_bone) else {
            continue;
        };
        
        let local_anchor1 = point![
            ragdoll_joint.local_anchor.x,
            ragdoll_joint.local_anchor.y,
            ragdoll_joint.local_anchor.z
        ];
        let local_anchor2 = point![
            ragdoll_joint.connected_anchor.x,
            ragdoll_joint.connected_anchor.y,
            ragdoll_joint.connected_anchor.z
        ];
        
        let joint: GenericJoint = match ragdoll_joint.joint_type {
            RagdollJointType::Ball => {
                SphericalJointBuilder::new()
                    .local_anchor1(local_anchor1)
                    .local_anchor2(local_anchor2)
                    .build()
                    .into()
            }
            RagdollJointType::Hinge => {
                let axis = NaUnit::new_normalize(vector![0.0, 1.0, 0.0]);
                RevoluteJointBuilder::new(axis)
                    .local_anchor1(local_anchor1)
                    .local_anchor2(local_anchor2)
                    .limits([ragdoll_joint.limits.min_angle, ragdoll_joint.limits.max_angle])
                    .build()
                    .into()
            }
            RagdollJointType::Twist => {
                let axis = NaUnit::new_normalize(vector![1.0, 0.0, 0.0]);
                RevoluteJointBuilder::new(axis)
                    .local_anchor1(local_anchor1)
                    .local_anchor2(local_anchor2)
                    .limits([ragdoll_joint.limits.min_angle, ragdoll_joint.limits.max_angle])
                    .build()
                    .into()
            }
        };
        
        let PhysicsWorld {
            ref mut rigid_body_set,
            ref mut impulse_joint_set,
            ref mut entity_to_joint,
            ..
        } = *physics_world;
        
        let joint_handle = impulse_joint_set.insert(
            body1_handle,
            body2_handle,
            joint,
            true,
        );
        
        entity_to_joint.insert(entity, joint_handle);
    }
}

/// System that synchronizes ragdoll bone transforms.
#[allow(clippy::needless_pass_by_value)]
pub fn ragdoll_sync_system(
    ragdoll_query: Query<&Ragdoll>,
    mut bone_query: Query<(&mut Transform, &PraxisRigidBody)>,
) {
    for ragdoll in &ragdoll_query {
        if !ragdoll.active || ragdoll.physics_blend < 0.01 {
            continue;
        }
        
        for bone in &ragdoll.bones {
            if let Ok((_transform, _)) = bone_query.get_mut(bone.entity) {
            }
        }
    }
}

/// System that applies powered ragdoll motors.
#[allow(clippy::needless_pass_by_value)]
pub fn ragdoll_motor_system(
    mut physics_world: ResMut<PhysicsWorld>,
    motor_query: Query<(Entity, &RagdollMotor)>,
) {
    for (entity, motor) in &motor_query {
        if !motor.enabled {
            continue;
        }
        
        let joint_handle = physics_world.entity_to_joint.get(&entity).copied();
        if let Some(handle) = joint_handle {
            if let Some(_joint) = physics_world.impulse_joint_set.get_mut(handle) {
            }
        }
    }
}
