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
//! # Transform Propagation System
//!
//! The transform propagation system automatically updates `GlobalTransform` components
//! based on local `Transform` and the parent-child hierarchy defined by `Parent` and
//! `Children` components.
//!
//! ## Key Systems
//!
//! - **`sync_parent_child_relationships`**: Maintains the bidirectional parent-child
//!   relationship by updating `Children` components when `Parent` components are added
//!   or changed.
//!
//! - **`cleanup_removed_parents`**: Removes orphaned children from `Children` components
//!   when `Parent` components are removed.
//!
//! - **`propagate_transforms`**: Updates `GlobalTransform` for root entities (without parents)
//!   when their `Transform` changes, and recursively propagates to all descendants.
//!
//! - **`propagate_transforms_for_reparented`**: Immediately updates `GlobalTransform` for
//!   entities whose `Parent` was added or changed.
//!
//! - **`propagate_transforms_for_changed_children`**: Updates `GlobalTransform` for entities
//!   with parents when their local `Transform` changes.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, Schedule, IntoSystemConfigs};
//! use praxis_ecs::{Transform, GlobalTransform, Parent, Children};
//! use praxis_ecs::systems::*;
//!
//! let mut world = World::new();
//! let mut schedule = Schedule::default();
//!
//! // Add transform propagation systems in the correct order
//! schedule.add_systems((
//!     sync_parent_child_relationships,
//!     cleanup_removed_parents,
//!     propagate_transforms,
//!     propagate_transforms_for_reparented,
//!     propagate_transforms_for_changed_children,
//! ).chain());
//!
//! // Create a parent-child hierarchy
//! let parent = world.spawn((
//!     Transform::from_xyz(10.0, 0.0, 0.0),
//!     GlobalTransform::default(),
//! ));
//!
//! let child = world.spawn((
//!     Transform::from_xyz(5.0, 0.0, 0.0),
//!     GlobalTransform::default(),
//!     Parent(parent),
//! ));
//!
//! // Run the schedule to propagate transforms
//! world.inner_mut().run_schedule(&mut schedule);
//!
//! // Child's global position will be (15, 0, 0)
//! ```
//!
//! # Mesh Components
//!
//! The ECS provides two mesh-related components for rendering 3D geometry:
//!
//! - **`Mesh`**: Stores mesh data directly on the entity. Useful for procedural
//!   or dynamic meshes that are unique to an entity.
//!
//! - **`MeshHandle`**: References a mesh by ID from the graphics system's asset
//!   manager. This is the preferred approach for shared static meshes.
//!
//! ## Mesh Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, Mesh, Transform};
//!
//! let mut world = World::new();
//!
//! let vertices = vec![
//!     [0.0, 1.0, 0.0],
//!     [-1.0, -1.0, 0.0],
//!     [1.0, -1.0, 0.0],
//! ];
//! let indices = vec![0, 1, 2];
//!
//! world.spawn((
//!     Transform::default(),
//!     Mesh::new(vertices, indices),
//! ));
//! ```
//!
//! ## MeshHandle Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, MeshHandle, Transform};
//!
//! let mut world = World::new();
//!
//! // Reference a mesh loaded in the graphics system
//! world.spawn((
//!     Transform::from_xyz(0.0, 0.0, 0.0),
//!     MeshHandle::new("cube"),
//! ));
//! ```
//!
//! See the [mesh system documentation](../../docs/mesh_system.md) for complete details.
//!
//! # Camera System
//!
//! The ECS provides a flexible camera system with perspective and orthographic projections:
//!
//! - **`Camera`**: Marks an entity as a camera with active state and priority
//! - **`PerspectiveProjection`**: Defines perspective projection parameters (FOV, aspect ratio)
//! - **`OrthographicProjection`**: Defines orthographic projection parameters (bounds)
//! - **`CameraMatrices`**: Automatically computed view and projection matrices
//!
//! ## Camera Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, PerspectiveCameraBundle};
//! use praxis_math::Vec3;
//!
//! let mut world = World::new();
//!
//! // Create a perspective camera
//! world.spawn(PerspectiveCameraBundle::new(
//!     Vec3::new(0.0, 5.0, 10.0),
//!     70.0_f32.to_radians(),
//!     16.0 / 9.0,
//! ));
//! ```
//!
//! ## Camera Systems
//!
//! Add camera update systems to your schedule:
//!
//! ```rust,no_run
//! use praxis_ecs::{Schedule, IntoSystemConfigs};
//! use praxis_ecs::systems::{update_perspective_cameras, update_orthographic_cameras};
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems((
//!     update_perspective_cameras,
//!     update_orthographic_cameras,
//! ));
//! ```
//!
//! ## Camera Query Helpers
//!
//! Use the camera module for common camera queries:
//!
//! ```rust,no_run
//! use praxis_ecs::{Query, camera};
//!
//! fn render_system(cameras: Query<camera::ActivePerspectiveCameras>) {
//!     // Get the primary camera
//!     if let Some((entity, camera, matrices)) = camera::primary_perspective_camera(&cameras) {
//!         // Render with view_projection matrix from matrices
//!     }
//! }
//! ```
//!
//! # Basic Example
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

/// Camera query helpers for common camera operations.
pub mod camera {
    use super::*;
    use bevy_ecs::query::{QueryData, QueryFilter};

    /// Query for all active perspective cameras.
    ///
    /// Returns cameras with Camera, Transform, PerspectiveProjection, and CameraMatrices
    /// that are active.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{Query, camera};
    ///
    /// fn render_system(cameras: Query<camera::ActivePerspectiveCameras>) {
    ///     for (entity, camera, transform, projection, matrices) in cameras.iter() {
    ///         // Render with this camera
    ///     }
    /// }
    /// ```
    #[derive(QueryData)]
    pub struct ActivePerspectiveCameras {
        pub entity: Entity,
        pub camera: &'static Camera,
        pub transform: &'static Transform,
        pub projection: &'static PerspectiveProjection,
        pub matrices: &'static CameraMatrices,
    }

    /// Query for all active orthographic cameras.
    ///
    /// Returns cameras with Camera, Transform, OrthographicProjection, and CameraMatrices
    /// that are active.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{Query, camera};
    ///
    /// fn render_system(cameras: Query<camera::ActiveOrthographicCameras>) {
    ///     for (entity, camera, transform, projection, matrices) in cameras.iter() {
    ///         // Render with this camera
    ///     }
    /// }
    /// ```
    #[derive(QueryData)]
    pub struct ActiveOrthographicCameras {
        pub entity: Entity,
        pub camera: &'static Camera,
        pub transform: &'static Transform,
        pub projection: &'static OrthographicProjection,
        pub matrices: &'static CameraMatrices,
    }

    /// Query filter for active cameras only.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{Query, Camera, CameraMatrices, camera};
    ///
    /// fn system(cameras: Query<(&Camera, &CameraMatrices), camera::ActiveCameraFilter>) {
    ///     for (camera, matrices) in cameras.iter() {
    ///         // Only active cameras
    ///     }
    /// }
    /// ```
    #[derive(QueryFilter)]
    pub struct ActiveCameraFilter {
        camera: With<Camera>,
    }

    /// Gets the primary camera (highest priority active camera).
    ///
    /// Returns the entity and components of the camera with the highest priority.
    /// If multiple cameras have the same priority, returns one of them.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{Query, camera};
    ///
    /// fn get_main_camera(
    ///     perspective_cameras: Query<camera::ActivePerspectiveCameras>,
    ///     orthographic_cameras: Query<camera::ActiveOrthographicCameras>,
    /// ) {
    ///     if let Some((entity, camera, matrices)) = camera::primary_perspective_camera(&perspective_cameras) {
    ///         // Use primary perspective camera
    ///     }
    /// }
    /// ```
    pub fn primary_perspective_camera<'a>(
        cameras: &'a Query<ActivePerspectiveCameras>,
    ) -> Option<(Entity, &'a Camera, &'a CameraMatrices)> {
        cameras
            .iter()
            .filter(|item| item.camera.is_active)
            .max_by_key(|item| item.camera.priority)
            .map(|item| (item.entity, item.camera, item.matrices))
    }

    /// Gets the primary orthographic camera (highest priority active camera).
    ///
    /// Returns the entity and components of the camera with the highest priority.
    /// If multiple cameras have the same priority, returns one of them.
    pub fn primary_orthographic_camera<'a>(
        cameras: &'a Query<ActiveOrthographicCameras>,
    ) -> Option<(Entity, &'a Camera, &'a CameraMatrices)> {
        cameras
            .iter()
            .filter(|item| item.camera.is_active)
            .max_by_key(|item| item.camera.priority)
            .map(|item| (item.entity, item.camera, item.matrices))
    }

    /// Gets all active cameras sorted by priority (lowest to highest).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{Query, camera};
    ///
    /// fn render_all_cameras(cameras: Query<camera::ActivePerspectiveCameras>) {
    ///     let sorted = camera::sorted_perspective_cameras(&cameras);
    ///     for (entity, camera, matrices) in sorted {
    ///         // Render in priority order
    ///     }
    /// }
    /// ```
    pub fn sorted_perspective_cameras<'a>(
        cameras: &'a Query<ActivePerspectiveCameras>,
    ) -> Vec<(Entity, &'a Camera, &'a CameraMatrices)> {
        let mut result: Vec<_> = cameras
            .iter()
            .filter(|item| item.camera.is_active)
            .map(|item| (item.entity, item.camera, item.matrices))
            .collect();

        result.sort_by_key(|(_, camera, _)| camera.priority);
        result
    }

    /// Gets all active orthographic cameras sorted by priority (lowest to highest).
    pub fn sorted_orthographic_cameras<'a>(
        cameras: &'a Query<ActiveOrthographicCameras>,
    ) -> Vec<(Entity, &'a Camera, &'a CameraMatrices)> {
        let mut result: Vec<_> = cameras
            .iter()
            .filter(|item| item.camera.is_active)
            .map(|item| (item.entity, item.camera, item.matrices))
            .collect();

        result.sort_by_key(|(_, camera, _)| camera.priority);
        result
    }
}

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

use praxis_utils::{debug, info, Result};

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
