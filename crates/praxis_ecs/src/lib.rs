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
//! # Mesh and Material Components
//!
//! The ECS provides components for rendering 3D geometry with materials:
//!
//! - **`Mesh`**: Stores mesh data directly on the entity. Useful for procedural
//!   or dynamic meshes that are unique to an entity.
//!
//! - **`MeshHandle`**: References a mesh by ID from the graphics system's asset
//!   manager. This is the preferred approach for shared static meshes.
//!
//! - **`MaterialHandle`**: References a material by ID from the graphics system's
//!   material manager. Materials define surface appearance including textures and
//!   physical properties.
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
//! ## MaterialHandle Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, MeshHandle, MaterialHandle, Transform};
//!
//! let mut world = World::new();
//!
//! // Spawn an entity with both mesh and material
//! world.spawn((
//!     Transform::from_xyz(0.0, 0.0, 0.0),
//!     MeshHandle::new("cube"),
//!     MaterialHandle::new("brick"),
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
//! # Lighting System
//!
//! The ECS provides a lighting system that collects light data from DirectionalLight and
//! PointLight components and stores them in a LightingData resource for rendering.
//!
//! ## Components
//!
//! - **`DirectionalLight`**: Sun-like lights with parallel rays (direction, color, intensity)
//! - **`PointLight`**: Omnidirectional lights with position-based attenuation (color, intensity, range)
//!
//! ## Resource
//!
//! - **`LightingData`**: Contains collected lighting information from all light entities
//!   - `directional_lights`: Vec of DirectionalLightInfo with world-space directions
//!   - `point_lights`: Vec of PointLightInfo with world-space positions
//!   - `ambient_color`: Global ambient lighting color
//!
//! ## System
//!
//! - **`gather_lighting_system`**: Queries all light components and populates LightingData
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, Schedule, LightingData};
//! use praxis_ecs::{DirectionalLight, PointLight, Transform};
//! use praxis_ecs::systems::gather_lighting_system;
//! use praxis_math::Vec3;
//!
//! let mut world = World::new();
//! let mut schedule = Schedule::default();
//!
//! // Initialize the lighting data resource
//! world.insert_resource(LightingData::default());
//!
//! // Add the lighting system to your schedule
//! schedule.add_systems(gather_lighting_system);
//!
//! // Spawn a directional light (sun)
//! world.spawn(DirectionalLight::new(
//!     Vec3::new(0.5, -1.0, 0.3).normalize(),
//!     Vec3::new(1.0, 0.95, 0.8),
//!     1.0,
//! ));
//!
//! // Spawn a point light with transform
//! world.spawn((
//!     Transform::from_xyz(0.0, 5.0, 0.0),
//!     PointLight::new(Vec3::new(1.0, 0.8, 0.6), 10.0, 20.0),
//! ));
//!
//! // Run the schedule to gather lighting data
//! world.inner_mut().run_schedule(&mut schedule);
//! ```
//!
//! The gathered lighting data can then be accessed in render systems:
//!
//! ```rust,no_run
//! use praxis_ecs::{Res, LightingData};
//!
//! fn render_system(lighting_data: Res<LightingData>) {
//!     // Use lighting_data.directional_lights
//!     // Use lighting_data.point_lights
//!     // Use lighting_data.ambient_color
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let result = init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_camera_query_helpers() {
        let mut world = World::new();

        let camera1 = world.spawn((
            Camera::with_priority(5),
            Transform::from_xyz(0.0, 0.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let camera2 = world.spawn((
            Camera::with_priority(1),
            Transform::from_xyz(5.0, 5.0, 5.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let camera3 = world.spawn((
            Camera::with_priority(10),
            Transform::from_xyz(-5.0, 0.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(systems::update_perspective_cameras);
        world.inner_mut().run_schedule(&mut schedule);

        let mut query = world.inner_mut().query::<camera::ActivePerspectiveCameras>();

        let primary = camera::primary_perspective_camera(&query);
        assert!(primary.is_some());
        let (entity, _, _) = primary.unwrap();
        assert_eq!(entity, camera3);

        let sorted = camera::sorted_perspective_cameras(&query);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].0, camera2);
        assert_eq!(sorted[1].0, camera1);
        assert_eq!(sorted[2].0, camera3);
    }

    #[test]
    fn test_orthographic_camera_helpers() {
        let mut world = World::new();

        let camera1 = world.spawn((
            Camera::with_priority(3),
            Transform::from_xyz(0.0, 10.0, 0.0),
            OrthographicProjection::default(),
            CameraMatrices::default(),
        ));

        let camera2 = world.spawn((
            Camera::with_priority(7),
            Transform::from_xyz(0.0, 15.0, 0.0),
            OrthographicProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(systems::update_orthographic_cameras);
        world.inner_mut().run_schedule(&mut schedule);

        let mut query = world.inner_mut().query::<camera::ActiveOrthographicCameras>();

        let primary = camera::primary_orthographic_camera(&query);
        assert!(primary.is_some());
        let (entity, camera_comp, _) = primary.unwrap();
        assert_eq!(entity, camera2);
        assert_eq!(camera_comp.priority, 7);

        let sorted = camera::sorted_orthographic_cameras(&query);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].0, camera1);
        assert_eq!(sorted[1].0, camera2);
    }

    #[test]
    fn test_inactive_camera_filtered_out() {
        let mut world = World::new();

        let mut inactive_camera = Camera::default();
        inactive_camera.deactivate();

        world.spawn((
            inactive_camera,
            Transform::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let active_camera = world.spawn((
            Camera::default(),
            Transform::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(systems::update_perspective_cameras);
        world.inner_mut().run_schedule(&mut schedule);

        let mut query = world.inner_mut().query::<camera::ActivePerspectiveCameras>();

        let primary = camera::primary_perspective_camera(&query);
        assert!(primary.is_some());
        let (entity, _, _) = primary.unwrap();
        assert_eq!(entity, active_camera);

        let sorted = camera::sorted_perspective_cameras(&query);
        assert_eq!(sorted.len(), 1);
    }

    #[test]
    fn test_system_scheduling() {
        use systems::*;

        let mut world = World::new();
        let mut schedule = Schedule::default();

        schedule.add_systems((
            sync_parent_child_relationships,
            cleanup_removed_parents,
            propagate_transforms,
            propagate_transforms_for_reparented,
            propagate_transforms_for_changed_children,
        ).chain());

        let parent = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let child = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        world.inner_mut().run_schedule(&mut schedule);

        let children = world.inner().get::<Children>(parent);
        assert!(children.is_some());

        let global_transform = world.inner().get::<GlobalTransform>(child);
        assert!(global_transform.is_some());
        let pos = global_transform.unwrap().translation();
        assert!((pos.x - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_lighting_data_resource() {
        let mut world = World::new();
        
        world.insert_resource(LightingData::default());
        
        world.spawn(DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            1.0,
        ));
        
        world.spawn((
            Transform::from_xyz(10.0, 5.0, 0.0),
            PointLight::new(Vec3::ONE, 10.0, 20.0),
        ));
        
        let mut schedule = Schedule::default();
        schedule.add_systems(systems::gather_lighting_system);
        world.inner_mut().run_schedule(&mut schedule);
        
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.directional_light_count(), 1);
        assert_eq!(lighting_data.point_light_count(), 1);
        assert_eq!(lighting_data.ambient_color, Vec3::new(0.1, 0.1, 0.1));
    }

    #[test]
    fn test_component_derivation() {
        #[derive(Component, Debug, Clone, Copy, PartialEq)]
        struct CustomComponent {
            value: i32,
        }

        let mut world = World::new();
        let entity = world.spawn(CustomComponent { value: 42 });

        let component = world.inner().get::<CustomComponent>(entity);
        assert!(component.is_some());
        assert_eq!(component.unwrap().value, 42);
    }

    #[test]
    fn test_multiple_schedules() {
        let mut world = World::new();

        #[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
        struct UpdateSchedule;

        #[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
        struct RenderSchedule;

        let mut update_schedule = Schedule::new(UpdateSchedule);
        update_schedule.add_systems(systems::propagate_transforms);

        let mut render_schedule = Schedule::new(RenderSchedule);
        render_schedule.add_systems(systems::gather_lighting_system);

        world.insert_resource(LightingData::default());

        world.add_schedule(update_schedule);
        world.add_schedule(render_schedule);

        let root = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        world.run_schedule(UpdateSchedule);

        let global = world.inner().get::<GlobalTransform>(root);
        assert!(global.is_some());
    }

    #[test]
    fn test_query_with_added_changed_filters() {
        use bevy_ecs::query::Changed;

        #[derive(Component, Debug, Clone, Copy)]
        struct Position {
            x: f32,
            y: f32,
        }

        let mut world = World::new();

        let entity1 = world.spawn(Position { x: 0.0, y: 0.0 });
        let entity2 = world.spawn(Position { x: 5.0, y: 5.0 });
        let _entity3 = world.spawn(Position { x: 10.0, y: 10.0 });

        world.inner_mut().clear_trackers();

        {
            let mut pos = world.inner_mut().get_mut::<Position>(entity1).unwrap();
            pos.x = 100.0;
        }

        {
            let mut pos = world.inner_mut().get_mut::<Position>(entity2).unwrap();
            pos.y = 200.0;
        }

        let mut query = world.inner_mut().query_filtered::<Entity, Changed<Position>>();
        let changed_entities: Vec<Entity> = query.iter(&world.inner()).collect();

        assert!(changed_entities.contains(&entity1));
        assert!(changed_entities.contains(&entity2));
    }

    #[test]
    fn test_bundle_usage() {
        use systems::TransformBundle;

        let mut world = World::new();

        let entity = world.spawn(TransformBundle::from_xyz(10.0, 20.0, 30.0));

        let transform = world.inner().get::<Transform>(entity);
        let global_transform = world.inner().get::<GlobalTransform>(entity);

        assert!(transform.is_some());
        assert!(global_transform.is_some());
        assert_eq!(transform.unwrap().translation, Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_perspective_camera_bundle_usage() {
        use systems::PerspectiveCameraBundle;

        let mut world = World::new();

        let entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 10.0),
            70.0_f32.to_radians(),
            16.0 / 9.0,
        ));

        let camera = world.inner().get::<Camera>(entity);
        let transform = world.inner().get::<Transform>(entity);
        let projection = world.inner().get::<PerspectiveProjection>(entity);
        let matrices = world.inner().get::<CameraMatrices>(entity);

        assert!(camera.is_some());
        assert!(transform.is_some());
        assert!(projection.is_some());
        assert!(matrices.is_some());

        assert_eq!(transform.unwrap().translation, Vec3::new(0.0, 5.0, 10.0));
        assert_eq!(projection.unwrap().fov, 70.0_f32.to_radians());
    }

    #[test]
    fn test_orthographic_camera_bundle_usage() {
        use systems::OrthographicCameraBundle;

        let mut world = World::new();

        let entity = world.spawn(OrthographicCameraBundle::new(
            Vec3::new(0.0, 10.0, 0.0),
            20.0,
            10.0,
        ));

        let camera = world.inner().get::<Camera>(entity);
        let transform = world.inner().get::<Transform>(entity);
        let projection = world.inner().get::<OrthographicProjection>(entity);

        assert!(camera.is_some());
        assert!(transform.is_some());
        assert!(projection.is_some());

        assert_eq!(transform.unwrap().translation, Vec3::new(0.0, 10.0, 0.0));
    }
}
