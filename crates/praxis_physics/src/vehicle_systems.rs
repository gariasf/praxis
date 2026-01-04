//! Systems for vehicle physics simulation.

use bevy_ecs::system::{Query, Res};
use praxis_ecs::{Entity, Transform};
use praxis_math::Vec3;

use crate::components::ExternalForces;
use crate::resources::{PhysicsConfig, PhysicsWorld};
use crate::vehicle::{WheelCollider, Vehicle, WheelFriction};

/// System that simulates wheel suspension and applies forces.
#[allow(clippy::needless_pass_by_value)]
pub fn vehicle_suspension_system(
    physics_world: Res<PhysicsWorld>,
    mut wheel_query: Query<(&mut WheelCollider, &Transform, &Parent)>,
    vehicle_query: Query<(&Vehicle, &Transform)>,
) {
    for (mut wheel, _wheel_transform, parent) in &mut wheel_query {
        let Ok((_vehicle, vehicle_transform)) = vehicle_query.get(parent.get()) else {
            continue;
        };
        
        let wheel_world_pos = vehicle_transform.transform_point(wheel.local_position);
        let suspension_direction = vehicle_transform.rotation * Vec3::NEG_Y;
        
        let raycast_origin = wheel_world_pos;
        let raycast_distance = wheel.suspension.travel + wheel.radius;
        
        if let Some((_, hit_distance)) = physics_world.raycast(
            raycast_origin,
            suspension_direction,
            raycast_distance,
            true,
        ) {
            wheel.is_grounded = true;
            wheel.ground_point = raycast_origin + suspension_direction * hit_distance;
            wheel.suspension_compression = (hit_distance - wheel.radius) / wheel.suspension.travel;
            wheel.suspension_compression = wheel.suspension_compression.clamp(0.0, 1.0);
        } else {
            wheel.is_grounded = false;
            wheel.suspension_compression = 1.0;
        }
    }
}

/// System that applies wheel forces based on vehicle inputs.
#[allow(clippy::needless_pass_by_value)]
pub fn vehicle_wheel_forces_system(
    _config: Res<PhysicsConfig>,
    mut vehicle_query: Query<(&Vehicle, &mut ExternalForces, &Transform)>,
    wheel_query: Query<(&WheelCollider, &WheelFriction, &Parent)>,
) {
    for (vehicle, mut forces, transform) in &mut vehicle_query {
        let mut total_force = Vec3::ZERO;
        let mut total_torque = Vec3::ZERO;
        
        for (wheel, _friction, _parent) in &wheel_query {
            if !wheel.is_grounded {
                continue;
            }
            
            let wheel_world_pos = transform.transform_point(wheel.local_position);
            let offset = wheel_world_pos - transform.translation;
            
            let suspension_force_magnitude = wheel.suspension.spring_stiffness 
                * (wheel.suspension.rest_position - wheel.suspension_compression) 
                * wheel.suspension.travel;
            
            let suspension_force = transform.rotation * Vec3::Y * suspension_force_magnitude;
            
            total_force += suspension_force;
            total_torque += offset.cross(suspension_force);
            
            if wheel.powered {
                let forward_dir = transform.rotation * Vec3::Z;
                let throttle_force = forward_dir * vehicle.throttle * vehicle.engine_torque;
                total_force += throttle_force;
                total_torque += offset.cross(throttle_force);
            }
            
            if vehicle.brake > 0.0 {
                let forward_dir = transform.rotation * Vec3::Z;
                let brake_force = -forward_dir * vehicle.brake * vehicle.brake_force;
                total_force += brake_force;
                total_torque += offset.cross(brake_force);
            }
        }
        
        forces.apply_force(total_force);
        forces.apply_torque(total_torque);
    }
}

/// Parent component for tracking vehicle-wheel relationships.
#[derive(bevy_ecs::component::Component, Debug, Clone, Copy)]
pub struct Parent(Entity);

impl Parent {
    /// Gets the parent entity.
    #[must_use]
    pub const fn get(&self) -> Entity {
        self.0
    }
}
