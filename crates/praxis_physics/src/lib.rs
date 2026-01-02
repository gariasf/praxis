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
//!     PhysicsWorld, PhysicsConfig, PhysicsTime,
//!     RigidBody, Collider, PhysicsVelocity,
//!     physics_step_system, sync_physics_transforms_system,
//! };
//! use praxis_ecs::{World, Schedule, IntoSystemConfigs, Transform};
//!
//! let mut world = World::new();
//! world.insert_resource(PhysicsWorld::new());
//! world.insert_resource(PhysicsConfig::default());
//! world.insert_resource(PhysicsTime::new());
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems((
//!     sync_physics_transforms_system,
//!     physics_step_system,
//!     sync_physics_transforms_system,
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
//!     PhysicsVelocity::default(),
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
//! - **`PhysicsVelocity`**: Linear and angular velocity
//! - **`ExternalForces`**: Accumulated forces and torques
//! - **`Mass`**: Mass properties
//! - **`Friction`**: Surface friction coefficient
//! - **`Restitution`**: Bounciness coefficient
//! - **`CollisionGroups`**: Collision filtering
//! - **`Sleeping`**: Sleep state control
//!
//! # Systems
//!
//! The physics simulation requires two core systems to be scheduled in order:
//!
//! 1. **`sync_physics_transforms_system`**: Bidirectionally syncs Transform components
//!    with Rapier rigid body positions (should be called before and after physics step)
//! 2. **`physics_step_system`**: Advances the physics simulation using fixed timestep integration
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
