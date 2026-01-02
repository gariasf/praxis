//! Physics simulation for the Praxis engine.
//!
//! This crate provides physics simulation capabilities using the Rapier3D physics engine,
//! wrapped in an ECS-friendly interface that integrates seamlessly with Praxis's architecture.
//!
//! # Architecture
//!
//! The physics system follows Praxis's ECS-first design:
//! - **Components**: Physics properties are stored as ECS components
//! - **Resources**: Physics pipeline and configuration are ECS resources
//! - **Systems**: Physics simulation runs as scheduled ECS systems
//!
//! # Basic Usage
//!
//! ```rust,no_run
//! use praxis_physics::{
//!     PhysicsWorld, PhysicsConfig,
//!     RigidBody, Collider, Velocity,
//!     step_physics_simulation, sync_transforms_from_physics, sync_transforms_to_physics,
//! };
//! use praxis_ecs::{World, Schedule, IntoSystemConfigs, Transform};
//!
//! let mut world = World::new();
//! world.insert_resource(PhysicsWorld::new());
//! world.insert_resource(PhysicsConfig::default());
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems((
//!     sync_transforms_to_physics,
//!     step_physics_simulation,
//!     sync_transforms_from_physics,
//! ).chain());
//!
//! // Create a static ground plane
//! world.spawn((
//!     Transform::from_xyz(0.0, 0.0, 0.0),
//!     RigidBody::Static,
//!     Collider::cuboid(50.0, 0.5, 50.0),
//! ));
//!
//! // Create a dynamic sphere
//! world.spawn((
//!     Transform::from_xyz(0.0, 10.0, 0.0),
//!     RigidBody::Dynamic,
//!     Collider::sphere(1.0),
//!     Velocity::default(),
//! ));
//!
//! // Run the simulation
//! world.inner_mut().run_schedule(&mut schedule);
//! ```
//!
//! # Components
//!
//! Physics properties are defined through components:
//!
//! - **`RigidBody`**: Marks an entity as a rigid body (Dynamic, Static, or Kinematic)
//! - **`Collider`**: Defines collision geometry
//! - **`Velocity`**: Linear and angular velocity
//! - **`ExternalForces`**: Accumulated forces and torques
//! - **`Mass`**: Mass properties
//! - **`Friction`**: Surface friction coefficient
//! - **`Restitution`**: Bounciness coefficient
//! - **`CollisionGroups`**: Collision filtering
//! - **`Sleeping`**: Sleep state control
//!
//! # Systems
//!
//! The physics simulation requires three systems to be scheduled in order:
//!
//! 1. **`sync_transforms_to_physics`**: Updates physics bodies from ECS transforms
//! 2. **`step_physics_simulation`**: Advances the physics simulation
//! 3. **`sync_transforms_from_physics`**: Updates ECS transforms from physics bodies
//!
//! # Transform Synchronization
//!
//! The physics system maintains bidirectional synchronization between ECS `Transform`
//! components and Rapier rigid body positions. This allows both physics-driven and
//! kinematic movement to work seamlessly.

mod components;
mod resources;
mod systems;

pub use components::*;
pub use resources::*;
pub use systems::*;

use praxis_utils::{info, Result};

/// Initializes the physics system.
///
/// This function sets up any necessary global state for the physics system.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Example
///
/// ```rust,no_run
/// praxis_physics::init().expect("Failed to initialize physics system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    info!("Initializing physics system");
    Ok(())
}
