//! Entity Component System for the Praxis engine.
//!
//! This crate provides the ECS architecture that powers the Praxis engine,
//! built on top of bevy_ecs for performance and ergonomics.
//!
//! # Architecture
//!
//! The ECS follows the standard Entity-Component-System pattern:
//! - **Entities**: Unique identifiers for game objects
//! - **Components**: Data attached to entities (position, velocity, etc.)
//! - **Systems**: Logic that operates on entities with specific components
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, Component};
//!
//! #[derive(Component)]
//! struct Position { x: f32, y: f32, z: f32 }
//!
//! #[derive(Component)]
//! struct Velocity { x: f32, y: f32, z: f32 }
//!
//! let mut world = World::new();
//!
//! // Spawn an entity with components
//! world.spawn((
//!     Position { x: 0.0, y: 0.0, z: 0.0 },
//!     Velocity { x: 1.0, y: 0.0, z: 0.0 },
//! ));
//! ```

mod components;
pub mod systems;
mod world;

pub use components::*;
pub use systems::*;
pub use world::*;

// Re-export commonly used bevy_ecs types
pub use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    query::{Added, Changed, Or, QueryState, With, Without},
    schedule::{IntoSystemConfigs, Schedule, ScheduleLabel, Schedules, SystemSet},
    system::{Commands, In, IntoSystem, Local, Res, ResMut, Resource, System, SystemParam},
    world::World as BevyWorld,
};

// Import Query from prelude
pub use bevy_ecs::prelude::Query;

use praxis_utils::{Result, debug, info};

/// Initializes the ECS system.
///
/// This function sets up any necessary global state for the ECS.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Example
///
/// ```rust,no_run
/// praxis_ecs::init().expect("Failed to initialize ECS");
/// ```
pub fn init() -> Result<()> {
    info!("Initializing ECS system");

    // Future initialization logic can go here
    // For now, bevy_ecs doesn't require explicit initialization

    debug!("ECS system initialized successfully");
    Ok(())
}
