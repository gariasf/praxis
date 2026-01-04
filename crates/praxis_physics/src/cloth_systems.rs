//! Systems for cloth simulation.

use bevy_ecs::system::{Query, Res};
use praxis_ecs::Transform;
use praxis_math::Vec3;

use crate::cloth::{Cloth, ClothWind, ClothTearing, ClothCollision};
use crate::resources::{PhysicsConfig, PhysicsWorld};

/// System that integrates cloth particle positions using Verlet integration.
#[allow(clippy::needless_pass_by_value)]
pub fn cloth_integration_system(
    config: Res<PhysicsConfig>,
    mut cloth_query: Query<(&mut Cloth, Option<&ClothWind>)>,
) {
    let dt = config.timestep;
    let gravity = config.gravity;
    
    for (mut cloth, wind) in &mut cloth_query {
        let damping = cloth.damping;
        
        for particle in &mut cloth.particles {
            if particle.inverse_mass == 0.0 {
                continue;
            }
            
            let mut acceleration = gravity;
            
            if let Some(wind) = wind {
                acceleration += wind.direction * (1.0 + wind.turbulence);
            }
            
            acceleration += particle.acceleration;
            
            let velocity = (particle.position - particle.previous_position) * damping;
            
            particle.previous_position = particle.position;
            particle.position += velocity + acceleration * dt * dt;
            particle.velocity = velocity / dt;
            
            particle.acceleration = Vec3::ZERO;
        }
    }
}

/// System that solves cloth distance constraints.
#[allow(clippy::needless_pass_by_value)]
pub fn cloth_constraints_system(
    mut cloth_query: Query<(&mut Cloth, Option<&ClothTearing>)>,
) {
    const ITERATIONS: usize = 4;
    
    for (mut cloth, tearing) in &mut cloth_query {
        for _ in 0..ITERATIONS {
            let mut constraints_to_remove = Vec::new();
            
            let constraints_snapshot: Vec<_> = cloth.constraints.clone();
            
            for (idx, constraint) in constraints_snapshot.iter().enumerate() {
                let particle_a = constraint.particle_a;
                let particle_b = constraint.particle_b;
                
                if particle_a >= cloth.particles.len() || particle_b >= cloth.particles.len() {
                    continue;
                }
                
                let pos_a = cloth.particles[particle_a].position;
                let pos_b = cloth.particles[particle_b].position;
                let inv_mass_a = cloth.particles[particle_a].inverse_mass;
                let inv_mass_b = cloth.particles[particle_b].inverse_mass;
                
                let delta = pos_b - pos_a;
                let distance = delta.length();
                
                if let Some(tearing_settings) = tearing {
                    if tearing_settings.enabled && distance > constraint.rest_length * tearing_settings.tear_distance {
                        constraints_to_remove.push(idx);
                        continue;
                    }
                }
                
                if distance > 0.0001 && (inv_mass_a > 0.0 || inv_mass_b > 0.0) {
                    let diff = (distance - constraint.rest_length) / distance;
                    let correction = delta * diff * constraint.stiffness;
                    
                    let total_inv_mass = inv_mass_a + inv_mass_b;
                    
                    if inv_mass_a > 0.0 {
                        cloth.particles[particle_a].position += correction * (inv_mass_a / total_inv_mass);
                    }
                    
                    if inv_mass_b > 0.0 {
                        cloth.particles[particle_b].position -= correction * (inv_mass_b / total_inv_mass);
                    }
                }
            }
            
            for &idx in constraints_to_remove.iter().rev() {
                cloth.constraints.remove(idx);
            }
        }
    }
}

/// System that handles cloth collision with physics world.
#[allow(clippy::needless_pass_by_value)]
pub fn cloth_collision_system(
    physics_world: Res<PhysicsWorld>,
    mut cloth_query: Query<(&mut Cloth, &ClothCollision, &Transform)>,
) {
    for (mut cloth, collision, transform) in &mut cloth_query {
        for particle in &mut cloth.particles {
            if particle.inverse_mass == 0.0 {
                continue;
            }
            
            let world_pos = transform.transform_point(particle.position);
            
            for collider_handle in physics_world.collider_set.iter() {
                let (_, collider) = collider_handle;
                
                if let Some(shape) = collider.shape().as_ball() {
                    let collider_pos = collider.position().translation;
                    let center = Vec3::new(collider_pos.x, collider_pos.y, collider_pos.z);
                    let radius = shape.radius + collision.particle_radius;
                    
                    let delta = world_pos - center;
                    let distance = delta.length();
                    
                    if distance < radius {
                        let penetration = radius - distance;
                        let normal = if distance > 0.0001 {
                            delta / distance
                        } else {
                            Vec3::Y
                        };
                        
                        let correction = normal * penetration * collision.response_stiffness;
                        particle.position += correction;
                    }
                }
            }
        }
    }
}
