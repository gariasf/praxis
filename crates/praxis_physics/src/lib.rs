//! Physics simulation for the Praxis engine.
//!
//! This crate provides physics simulation capabilities using the `Rapier3D` physics engine,
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
//!     PhysicsWorld, PhysicsConfig, PhysicsTime, ContactEvents,
//!     RigidBody, Collider, PhysicsVelocity, CollisionEventReceiver,
//!     cleanup_physics_entities, physics_step_system, sync_physics_transforms_system,
//!     clear_collision_event_receivers, populate_collision_events,
//! };
//! use praxis_ecs::{World, Schedule, IntoSystemConfigs, Transform};
//!
//! let mut world = World::new();
//! world.insert_resource(PhysicsWorld::new());
//! world.insert_resource(PhysicsConfig::default());
//! world.insert_resource(PhysicsTime::new());
//! world.insert_resource(ContactEvents::new());
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems((
//!     cleanup_physics_entities,           // Clean up despawned entities
//!     clear_collision_event_receivers,    // Clear old collision events
//!     sync_physics_transforms_system,     // ECS → Physics
//!     physics_step_system,                // Run simulation
//!     sync_physics_transforms_system,     // Physics → ECS
//!     populate_collision_events,          // Distribute collision events
//! ).chain());
//!
//! // Create a static ground plane
//! world.spawn((
//!     Transform::from_xyz(0.0, 0.0, 0.0),
//!     RigidBody::Static,
//!     Collider::cuboid(50.0, 0.5, 50.0),
//! ));
//!
//! // Create a dynamic sphere that receives collision events
//! world.spawn((
//!     Transform::from_xyz(0.0, 10.0, 0.0),
//!     RigidBody::Dynamic,
//!     Collider::sphere(1.0),
//!     PhysicsVelocity::default(),
//!     CollisionEventReceiver::new(), // Enable collision events
//! ));
//!
//! // Run the simulation
//! schedule.run(world.inner_mut());
//! ```
//!
//! # Collision Event Handling Example
//!
//! ```rust,no_run
//! use praxis_physics::{CollisionEventReceiver, CollisionEvent};
//! use praxis_ecs::{Query, Entity};
//!
//! /// System that handles collision events for a player entity
//! fn handle_player_collisions(
//!     query: Query<&CollisionEventReceiver>
//! ) {
//!     for receiver in query.iter() {
//!         for event in &receiver.events {
//!             match event {
//!                 CollisionEvent::CollisionStarted(self_entity, other_entity) => {
//!                     println!("Player {:?} started colliding with {:?}",
//!                              self_entity, other_entity);
//!                     // Play impact sound, trigger damage, etc.
//!                 }
//!                 CollisionEvent::CollisionStopped(self_entity, other_entity) => {
//!                     println!("Player {:?} stopped colliding with {:?}",
//!                              self_entity, other_entity);
//!                     // Stop continuous effects
//!                 }
//!                 CollisionEvent::CollisionPersisted(self_entity, other_entity) => {
//!                     println!("Player {:?} continues colliding with {:?}",
//!                              self_entity, other_entity);
//!                     // Apply damage over time
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # Spatial Query Example
//!
//! ```rust,no_run
//! use praxis_physics::PhysicsWorld;
//! use praxis_math::Vec3;
//! use praxis_ecs::Res;
//!
//! /// System that performs raycasting for weapon firing
//! fn weapon_raycast(physics: Res<PhysicsWorld>) {
//!     let origin = Vec3::new(0.0, 1.5, 0.0);
//!     let direction = Vec3::new(0.0, 0.0, 1.0).normalize();
//!     
//!     // Raycast to find what the weapon hits
//!     if let Some((entity, distance)) = physics.raycast(
//!         origin,
//!         direction,
//!         100.0, // Max range
//!         true,  // Solid (stop at first hit)
//!     ) {
//!         println!("Hit entity {:?} at distance {}", entity, distance);
//!         let hit_point = origin + direction * distance;
//!         // Apply damage, spawn hit effects at hit_point, etc.
//!     }
//!     
//!     // Check if a point is inside a damage zone
//!     let player_pos = Vec3::new(5.0, 0.0, 5.0);
//!     if let Some(zone) = physics.point_inside(player_pos) {
//!         println!("Player is inside damage zone {:?}", zone);
//!     }
//! }
//! ```
//!
//! # Components
//!
//! Physics properties are defined through components:
//!
//! - **`RigidBody`**: Marks an entity as a rigid body (Dynamic, Static, or Kinematic)
//! - **`Collider`**: Defines collision geometry
//! - **`PhysicsVelocity`**: Linear and angular velocity
//! - **`ExternalForces`**: Accumulated forces and torques
//! - **`Mass`**: Mass properties
//! - **`Friction`**: Surface friction coefficient
//! - **`Restitution`**: Bounciness coefficient
//! - **`CollisionGroups`**: Collision filtering
//! - **`Sleeping`**: Sleep state control
//! - **`CollisionEventReceiver`**: Component for receiving collision events
//!
//! # Collision Events
//!
//! The physics system provides collision event handling through:
//!
//! - **`CollisionEvent`**: Enum representing collision event types (Started, Stopped, Persisted)
//! - **`CollisionEventReceiver`**: Component that stores collision events for an entity
//! - **`clear_collision_event_receivers`**: System that clears event buffers each frame
//! - **`populate_collision_events`**: System that distributes events to entities
//!
//! # Spatial Queries
//!
//! The `PhysicsWorld` resource provides query helpers for spatial operations:
//!
//! - **`raycast`**: Cast a ray and return the first hit
//! - **`raycast_all`**: Cast a ray and return all hits
//! - **`shape_cast`**: Sweep a shape and return the first hit
//! - **`point_inside`**: Check if a point is inside any collider
//!
//! These queries use the physics world's spatial acceleration structures for efficient
//! collision detection and are commonly used for weapon systems, character controllers,
//! and spatial awareness.
//!
//! # Systems
//!
//! The physics simulation requires these core systems to be scheduled in order:
//!
//! 1. **`cleanup_physics_entities`**: Removes Rapier bodies/colliders for despawned entities
//!    (should run before physics to avoid processing stale entities)
//! 2. **`sync_physics_transforms_system`**: Bidirectionally syncs Transform components
//!    with Rapier rigid body positions (should be called before and after physics step)
//! 3. **`physics_step_system`**: Advances the physics simulation using fixed timestep integration
//!
//! For collision events, add these systems:
//! 1. **`clear_collision_event_receivers`**: Clears event buffers (before physics step)
//! 2. **`populate_collision_events`**: Distributes events to entities (after physics step)
//!
//! Alternative: Use the separate legacy systems:
//! 1. **`sync_transforms_to_physics`**: Updates physics bodies from ECS transforms
//! 2. **`step_physics_simulation`**: Advances the physics simulation (no fixed timestep)
//! 3. **`sync_transforms_from_physics`**: Updates ECS transforms from physics bodies
//!
//! # Transform Synchronization
//!
//! The physics system maintains bidirectional synchronization between ECS `Transform`
//! components and Rapier rigid body positions. This allows both physics-driven and
//! kinematic movement to work seamlessly.

mod cloth;
mod cloth_systems;
mod components;
mod joint_systems;
mod joints;
mod ragdoll;
mod ragdoll_systems;
mod resources;
mod systems;
mod vehicle;
mod vehicle_systems;

#[cfg(test)]
mod tests;

pub use cloth::*;
pub use cloth_systems::*;
pub use components::*;
pub use joint_systems::*;
pub use joints::*;
pub use ragdoll::*;
pub use ragdoll_systems::*;
pub use resources::*;
pub use systems::*;
pub use vehicle::*;
pub use vehicle_systems::*;

use praxis_utils::{info, Result};

/// Initializes the physics system.
///
/// This function sets up any necessary global state for the physics system.
/// It should be called once during engine initialization, before any physics
/// resources or components are used.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for physics
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., thread pools, GPU backends)
/// - Validates that dependencies are available
///
/// # Integration
///
/// This function follows the Praxis pattern where each subsystem provides an `init()`
/// function that is called during engine startup. The physics system is typically
/// initialized after core systems (utils, ECS) but before window and rendering.
///
/// # Example
///
/// ```rust,no_run
/// // In engine initialization sequence
/// praxis_utils::init().expect("Failed to initialize utilities");
/// praxis_ecs::init().expect("Failed to initialize ECS");
/// praxis_physics::init().expect("Failed to initialize physics system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds,
/// but future versions may perform validation or setup that could fail.
///
/// # Thread Safety
///
/// This function is safe to call from any thread, but should only be called once
/// during application lifetime. Multiple calls are harmless but redundant.
pub fn init() -> Result<()> {
    info!("Initializing physics system");
    // Future initialization work can be added here, such as:
    // - Setting up thread pools for parallel collision detection
    // - Initializing GPU acceleration if available
    // - Validating Rapier version compatibility
    // - Loading physics configuration from files
    Ok(())
}
