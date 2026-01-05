//! Systems for managing joint constraints.

use bevy_ecs::system::{Query, ResMut};
use praxis_ecs::{Entity, With};
use rapier3d::na::Unit as NaUnit;
use rapier3d::prelude::*;

use crate::components::RigidBody as PraxisRigidBody;
use crate::joints::{
    BallJoint, FixedJoint as PraxisFixedJoint, HingeJoint, SliderJoint,
    SpringJoint as PraxisSpringJoint,
};
use crate::resources::PhysicsWorld;

/// System that creates and updates joints between rigid bodies.
#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_lines,
    unused_variables
)]
pub fn sync_joints_system(
    mut physics_world: ResMut<PhysicsWorld>,
    hinge_query: Query<(Entity, &HingeJoint), With<PraxisRigidBody>>,
    ball_query: Query<(Entity, &BallJoint), With<PraxisRigidBody>>,
    slider_query: Query<(Entity, &SliderJoint), With<PraxisRigidBody>>,
    fixed_query: Query<(Entity, &PraxisFixedJoint), With<PraxisRigidBody>>,
) {
    for (entity, hinge) in &hinge_query {
        if physics_world.entity_to_joint.contains_key(&entity) {
            continue;
        }

        let Some(body1_handle) = physics_world.get_body_handle(entity) else {
            continue;
        };

        let Some(body2_handle) = physics_world.get_body_handle(hinge.connected_entity) else {
            continue;
        };

        let local_anchor1 = point![
            hinge.local_anchor1.x,
            hinge.local_anchor1.y,
            hinge.local_anchor1.z
        ];
        let local_anchor2 = point![
            hinge.local_anchor2.x,
            hinge.local_anchor2.y,
            hinge.local_anchor2.z
        ];
        let local_axis1 = NaUnit::new_normalize(vector![
            hinge.local_axis1.x,
            hinge.local_axis1.y,
            hinge.local_axis1.z
        ]);

        let mut joint = RevoluteJointBuilder::new(local_axis1)
            .local_anchor1(local_anchor1)
            .local_anchor2(local_anchor2);

        if hinge.limits_enabled {
            joint = joint.limits([hinge.min_angle, hinge.max_angle]);
        }

        if hinge.motor_max_force > 0.0 {
            joint = joint.motor_velocity(hinge.motor_velocity, hinge.motor_max_force);
        }

        let PhysicsWorld {
            ref mut rigid_body_set,
            ref mut impulse_joint_set,
            ref mut entity_to_joint,
            ..
        } = *physics_world;

        let joint_handle =
            impulse_joint_set.insert(body1_handle, body2_handle, joint.build(), true);

        entity_to_joint.insert(entity, joint_handle);
    }

    for (entity, ball) in &ball_query {
        if physics_world.entity_to_joint.contains_key(&entity) {
            continue;
        }

        let Some(body1_handle) = physics_world.get_body_handle(entity) else {
            continue;
        };

        let Some(body2_handle) = physics_world.get_body_handle(ball.connected_entity) else {
            continue;
        };

        let local_anchor1 = point![
            ball.local_anchor1.x,
            ball.local_anchor1.y,
            ball.local_anchor1.z
        ];
        let local_anchor2 = point![
            ball.local_anchor2.x,
            ball.local_anchor2.y,
            ball.local_anchor2.z
        ];

        let joint = SphericalJointBuilder::new()
            .local_anchor1(local_anchor1)
            .local_anchor2(local_anchor2)
            .build();

        let PhysicsWorld {
            ref mut rigid_body_set,
            ref mut impulse_joint_set,
            ref mut entity_to_joint,
            ..
        } = *physics_world;

        let joint_handle = impulse_joint_set.insert(body1_handle, body2_handle, joint, true);

        entity_to_joint.insert(entity, joint_handle);
    }

    for (entity, slider) in &slider_query {
        if physics_world.entity_to_joint.contains_key(&entity) {
            continue;
        }

        let Some(body1_handle) = physics_world.get_body_handle(entity) else {
            continue;
        };

        let Some(body2_handle) = physics_world.get_body_handle(slider.connected_entity) else {
            continue;
        };

        let local_anchor1 = point![
            slider.local_anchor1.x,
            slider.local_anchor1.y,
            slider.local_anchor1.z
        ];
        let local_anchor2 = point![
            slider.local_anchor2.x,
            slider.local_anchor2.y,
            slider.local_anchor2.z
        ];
        let local_axis1 = NaUnit::new_normalize(vector![
            slider.local_axis1.x,
            slider.local_axis1.y,
            slider.local_axis1.z
        ]);
        let local_axis2 = NaUnit::new_normalize(vector![
            slider.local_axis2.x,
            slider.local_axis2.y,
            slider.local_axis2.z
        ]);

        let mut joint = PrismaticJointBuilder::new(local_axis1)
            .local_anchor1(local_anchor1)
            .local_anchor2(local_anchor2)
            .local_axis2(local_axis2);

        if slider.limits_enabled {
            joint = joint.limits([slider.min_distance, slider.max_distance]);
        }

        if slider.motor_max_force > 0.0 {
            joint = joint.motor_velocity(slider.motor_velocity, slider.motor_max_force);
        }

        let PhysicsWorld {
            ref mut rigid_body_set,
            ref mut impulse_joint_set,
            ref mut entity_to_joint,
            ..
        } = *physics_world;

        let joint_handle =
            impulse_joint_set.insert(body1_handle, body2_handle, joint.build(), true);

        entity_to_joint.insert(entity, joint_handle);
    }

    for (entity, fixed) in &fixed_query {
        if physics_world.entity_to_joint.contains_key(&entity) {
            continue;
        }

        let Some(body1_handle) = physics_world.get_body_handle(entity) else {
            continue;
        };

        let Some(body2_handle) = physics_world.get_body_handle(fixed.connected_entity) else {
            continue;
        };

        let local_anchor1 = point![
            fixed.local_anchor1.x,
            fixed.local_anchor1.y,
            fixed.local_anchor1.z
        ];
        let local_anchor2 = point![
            fixed.local_anchor2.x,
            fixed.local_anchor2.y,
            fixed.local_anchor2.z
        ];

        let joint = FixedJointBuilder::new()
            .local_anchor1(local_anchor1)
            .local_anchor2(local_anchor2)
            .build();

        let PhysicsWorld {
            ref mut rigid_body_set,
            ref mut impulse_joint_set,
            ref mut entity_to_joint,
            ..
        } = *physics_world;

        let joint_handle = impulse_joint_set.insert(body1_handle, body2_handle, joint, true);

        entity_to_joint.insert(entity, joint_handle);
    }
}

/// System that updates spring joints.
#[allow(clippy::needless_pass_by_value)]
pub fn update_spring_joints_system(
    mut physics_world: ResMut<PhysicsWorld>,
    spring_query: Query<(Entity, &PraxisSpringJoint), With<PraxisRigidBody>>,
) {
    for (entity, spring) in &spring_query {
        let Some(body1_handle) = physics_world.get_body_handle(entity) else {
            continue;
        };

        let Some(body2_handle) = physics_world.get_body_handle(spring.connected_entity) else {
            continue;
        };

        let Some(body1) = physics_world.rigid_body_set.get(body1_handle) else {
            continue;
        };

        let Some(body2) = physics_world.rigid_body_set.get(body2_handle) else {
            continue;
        };

        let pos1 = body1.position().transform_point(&point![
            spring.local_anchor1.x,
            spring.local_anchor1.y,
            spring.local_anchor1.z
        ]);

        let pos2 = body2.position().transform_point(&point![
            spring.local_anchor2.x,
            spring.local_anchor2.y,
            spring.local_anchor2.z
        ]);

        let delta = pos2 - pos1;
        let distance = delta.norm();
        let extension = distance - spring.rest_length;

        if distance > 0.0001 {
            let direction = delta / distance;
            let spring_force = direction * (spring.stiffness * extension);

            let vel1 = body1.linvel();
            let vel2 = body2.linvel();
            let relative_vel = vel2 - vel1;
            let damping_force = direction * (spring.damping * relative_vel.dot(&direction));

            let total_force = spring_force + damping_force;

            if let Some(body1_mut) = physics_world.rigid_body_set.get_mut(body1_handle) {
                body1_mut.add_force(total_force, true);
            }

            if let Some(body2_mut) = physics_world.rigid_body_set.get_mut(body2_handle) {
                body2_mut.add_force(-total_force, true);
            }
        }
    }
}
