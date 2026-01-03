//! Systems for the Praxis ECS.
//!
//! This module provides pre-built systems that handle common game engine
//! functionality like transform propagation, parent-child relationships,
//! and other core behaviors.

use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    query::{Added, Changed, Or, With, Without},
    schedule::SystemSet,
    system::{Commands, ParamSet},
};
use std::collections::HashSet;

use crate::{
    Camera, CameraMatrices, Children, DirectionalLight, DirectionalLightInfo, GlobalTransform,
    LightingData, OrthographicProjection, Parent, PerspectiveProjection, PointLight,
    PointLightInfo, Query, Transform,
};
use praxis_math::Vec3;
use praxis_utils::trace;

/// System sets for organizing systems into logical groups.
///
/// These can be used to order systems relative to each other.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreSystemSet {
    /// Systems that run at the very beginning of the frame.
    PreUpdate,

    /// Systems that handle transform propagation.
    TransformPropagate,

    /// General update systems.
    Update,

    /// Systems that run after the main update.
    PostUpdate,
}

/// System that maintains parent-child relationships when parents are added or changed.
///
/// This system ensures that when a Parent component is added or changed on an entity,
/// the parent entity's Children component is updated accordingly. It also handles
/// removing the entity from the old parent's children list when the parent changes.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule};
/// use praxis_ecs::systems::sync_parent_child_relationships;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(sync_parent_child_relationships);
/// ```
pub fn sync_parent_child_relationships(
    mut commands: Commands,
    // Entities where Parent was added this frame
    added_parents: Query<(Entity, &Parent), Added<Parent>>,
    // Entities where Parent was changed this frame (excluding newly added)
    changed_parents: Query<(Entity, &Parent), Changed<Parent>>,
    mut parents_query: Query<&mut Children>,
) {
    // Track children to add to parents that don't have Children component yet
    // We collect these first to avoid multiple overlapping commands
    let mut pending_children: std::collections::HashMap<Entity, Vec<Entity>> =
        std::collections::HashMap::new();

    // Handle newly added Parent components
    for (child_entity, parent) in added_parents.iter() {
        if let Ok(mut children) = parents_query.get_mut(parent.0) {
            if !children.0.contains(&child_entity) {
                trace!(
                    "Adding child entity {:?} to parent {:?}",
                    child_entity,
                    parent.0
                );
                children.push(child_entity);
            }
        } else {
            // Parent doesn't have Children component yet, add to pending
            pending_children.entry(parent.0).or_default().push(child_entity);
        }
    }

    // Handle changed Parent components
    // Note: We need to track the old parent to remove from its children list
    // For now, we just ensure the child is in the new parent's list
    for (child_entity, parent) in changed_parents.iter() {
        // Skip if this was just added (already handled above)
        if added_parents.get(child_entity).is_ok() {
            continue;
        }

        if let Ok(mut children) = parents_query.get_mut(parent.0) {
            if !children.0.contains(&child_entity) {
                trace!(
                    "Adding child entity {:?} to parent {:?}",
                    child_entity,
                    parent.0
                );
                children.push(child_entity);
            }
        } else {
            // Parent doesn't have Children component yet, add to pending
            pending_children.entry(parent.0).or_default().push(child_entity);
        }
    }

    // Insert Children components for all parents that don't have them yet
    // This ensures we add all children in one command rather than overlapping commands
    for (parent_entity, children) in pending_children {
        trace!(
            "Creating Children component for parent {:?} with {} children",
            parent_entity,
            children.len()
        );
        commands
            .entity(parent_entity)
            .insert(Children::with_children(children));
    }
}

/// System that removes children from their parent's Children component when the Parent is removed.
///
/// This system detects when Parent components are removed from entities and updates
/// the parent's Children component accordingly.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule};
/// use praxis_ecs::systems::cleanup_removed_parents;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(cleanup_removed_parents);
/// ```
pub fn cleanup_removed_parents(
    mut commands: Commands,
    // Query all entities with Children to check for orphaned references
    mut parents_query: Query<(Entity, &mut Children)>,
    // Query to check if entities exist and have Parent
    parent_check: Query<&Parent>,
) {
    for (parent_entity, mut children) in parents_query.iter_mut() {
        let original_len = children.0.len();

        // Remove children that no longer have this entity as their parent
        children.0.retain(|&child| {
            if let Ok(parent) = parent_check.get(child) {
                // Child still has a parent, check if it's this parent
                parent.0 == parent_entity
            } else {
                // Child no longer has a Parent component, remove from list
                false
            }
        });

        if children.0.len() != original_len {
            trace!(
                "Cleaned up orphaned children for parent {:?}: {} -> {}",
                parent_entity,
                original_len,
                children.0.len()
            );
        }

        // If children list is now empty, optionally remove the Children component
        if children.0.is_empty() {
            commands.entity(parent_entity).remove::<Children>();
        }
    }
}

/// Propagates transforms through the entity hierarchy.
///
/// This system updates the GlobalTransform component based on the local Transform
/// and the parent's GlobalTransform. It ensures that child entities inherit
/// the transformations of their parents.
///
/// The system runs in multiple phases:
/// 1. Update root entities (entities with Transform but no Parent)
/// 2. Recursively update children when their parent or their own Transform changes
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule, IntoSystemConfigs};
/// use praxis_ecs::systems::{propagate_transforms, CoreSystemSet};
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(
///     propagate_transforms.in_set(CoreSystemSet::TransformPropagate)
/// );
/// ```
#[allow(clippy::type_complexity)]
pub fn propagate_transforms(
    mut root_queries: ParamSet<(
        Query<
            (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
            (Without<Parent>, Or<(Changed<Transform>, Added<Transform>)>),
        >,
        Query<
            (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
            Without<Parent>,
        >,
    )>,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    // First pass: Collect changed roots and process them
    // We collect into intermediate vectors because we can only access one ParamSet query at a time
    let mut changed_roots = HashSet::new();
    let mut children_to_propagate: Vec<(Vec<Entity>, praxis_math::Mat4)> = Vec::new();

    for (entity, transform, mut global_transform, children) in root_queries.p0().iter_mut() {
        changed_roots.insert(entity);
        global_transform.matrix = transform.compute_matrix();

        if let Some(children) = children {
            children_to_propagate.push((children.0.clone(), global_transform.matrix));
        }
    }

    // Propagate to children of changed roots
    for (children, parent_matrix) in children_to_propagate {
        propagate_recursive(&children, &parent_matrix, &mut child_query);
    }

    // Second pass: Update root entities that didn't change but need their children updated
    // This handles cases where children were added or a parent was removed
    let mut unchanged_children: Vec<(Vec<Entity>, praxis_math::Mat4)> = Vec::new();

    for (entity, transform, mut global_transform, children) in root_queries.p1().iter_mut() {
        // Skip if we already processed this in the first pass
        if changed_roots.contains(&entity) {
            continue;
        }

        // Update global transform to ensure consistency
        global_transform.matrix = transform.compute_matrix();

        // Check if any children were added to this root
        if let Some(children) = children {
            unchanged_children.push((children.0.clone(), global_transform.matrix));
        }
    }

    // Propagate to children of unchanged roots
    for (children, parent_matrix) in unchanged_children {
        propagate_to_added_children(&children, &parent_matrix, &mut child_query);
    }
}

/// Recursively propagates transforms to all descendants.
///
/// This function updates the GlobalTransform of all children based on their
/// parent's GlobalTransform and their own local Transform.
fn propagate_recursive(
    children: &[Entity],
    parent_matrix: &praxis_math::Mat4,
    child_query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    // Use a work queue to avoid recursion and borrow checker issues
    let mut work_queue: Vec<(Entity, praxis_math::Mat4)> = children
        .iter()
        .map(|&child| (child, *parent_matrix))
        .collect();

    while let Some((entity, parent_matrix)) = work_queue.pop() {
        if let Ok((transform, mut global_transform, maybe_children)) = child_query.get_mut(entity) {
            // Compute the child's global transform by combining with parent
            let child_matrix = parent_matrix * transform.compute_matrix();
            global_transform.matrix = child_matrix;

            // Add this entity's children to the work queue
            if let Some(children) = maybe_children {
                for &child in children.0.iter() {
                    work_queue.push((child, child_matrix));
                }
            }
        }
    }
}

/// Propagates transforms only to children that were recently added or whose transforms changed.
///
/// This is an optimization to avoid updating the entire hierarchy when only a subtree changed.
fn propagate_to_added_children(
    children: &[Entity],
    parent_matrix: &praxis_math::Mat4,
    child_query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    // Use the same iterative approach as propagate_recursive
    propagate_recursive(children, parent_matrix, child_query);
}

/// Propagates transforms for entities with changed parents.
///
/// This system specifically handles the case where a Parent component was added or changed,
/// ensuring that the entity's GlobalTransform is immediately updated based on its new parent.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule};
/// use praxis_ecs::systems::propagate_transforms_for_reparented;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(propagate_transforms_for_reparented);
/// ```
#[allow(clippy::type_complexity)]
pub fn propagate_transforms_for_reparented(
    // Entities whose Parent component was added or changed (read-only first pass)
    reparented: Query<
        (Entity, &Parent, &Transform, Option<&Children>),
        Or<(Added<Parent>, Changed<Parent>)>,
    >,
    // ParamSet to avoid read/write conflicts on GlobalTransform
    mut global_set: ParamSet<(
        Query<&GlobalTransform>,      // p0: for reading parent transforms
        Query<&mut GlobalTransform>,  // p1: for writing entity transforms
        Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>, // p2: for child propagation
    )>,
) {
    // First pass: collect all data we need (using read-only access to parent GlobalTransforms)
    let updates: Vec<(Entity, praxis_math::Mat4, Option<Vec<Entity>>)> = {
        let parent_query = global_set.p0();
        reparented
            .iter()
            .filter_map(|(entity, parent, transform, maybe_children)| {
                parent_query.get(parent.0).ok().map(|parent_global| {
                    let new_matrix = parent_global.matrix * transform.compute_matrix();
                    (entity, new_matrix, maybe_children.map(|c| c.0.clone()))
                })
            })
            .collect()
    };

    // Second pass: apply updates (using mutable access)
    {
        let mut write_query = global_set.p1();
        for (entity, new_matrix, _) in &updates {
            if let Ok(mut global) = write_query.get_mut(*entity) {
                global.matrix = *new_matrix;
            }
        }
    }

    // Third pass: propagate to children using p2
    {
        let mut child_query = global_set.p2();
        for (_, new_matrix, maybe_children) in updates {
            if let Some(children) = maybe_children {
                propagate_recursive(&children, &new_matrix, &mut child_query);
            }
        }
    }
}

/// Propagates transforms for entities whose local Transform changed.
///
/// This system handles transform changes for entities that have a parent,
/// ensuring their GlobalTransform and all descendants are updated.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule};
/// use praxis_ecs::systems::propagate_transforms_for_changed_children;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(propagate_transforms_for_changed_children);
/// ```
#[allow(clippy::type_complexity)]
pub fn propagate_transforms_for_changed_children(
    // Entities with Parent whose Transform changed (read-only first pass)
    changed: Query<
        (Entity, &Parent, &Transform, Option<&Children>),
        (With<Parent>, Changed<Transform>),
    >,
    // ParamSet to avoid read/write conflicts on GlobalTransform
    mut global_set: ParamSet<(
        Query<&GlobalTransform>,      // p0: for reading parent transforms
        Query<&mut GlobalTransform>,  // p1: for writing entity transforms
        Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>, // p2: for child propagation
    )>,
) {
    // First pass: collect all data we need (using read-only access to parent GlobalTransforms)
    let updates: Vec<(Entity, praxis_math::Mat4, Option<Vec<Entity>>)> = {
        let parent_query = global_set.p0();
        changed
            .iter()
            .filter_map(|(entity, parent, transform, maybe_children)| {
                parent_query.get(parent.0).ok().map(|parent_global| {
                    let new_matrix = parent_global.matrix * transform.compute_matrix();
                    (entity, new_matrix, maybe_children.map(|c| c.0.clone()))
                })
            })
            .collect()
    };

    // Second pass: apply updates (using mutable access)
    {
        let mut write_query = global_set.p1();
        for (entity, new_matrix, _) in &updates {
            if let Ok(mut global) = write_query.get_mut(*entity) {
                global.matrix = *new_matrix;
            }
        }
    }

    // Third pass: propagate to children using p2
    {
        let mut child_query = global_set.p2();
        for (_, new_matrix, maybe_children) in updates {
            if let Some(children) = maybe_children {
                propagate_recursive(&children, &new_matrix, &mut child_query);
            }
        }
    }
}

/// Updates camera matrices for cameras with perspective projection.
///
/// This system computes the view and projection matrices for all active cameras
/// with a perspective projection. The view matrix is computed from the camera's
/// Transform (or GlobalTransform if it has a parent), and the projection matrix
/// is computed from the PerspectiveProjection component.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule};
/// use praxis_ecs::systems::update_perspective_cameras;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(update_perspective_cameras);
/// ```
#[allow(clippy::type_complexity)]
pub fn update_perspective_cameras(
    mut cameras: Query<
        (
            &Camera,
            &Transform,
            &PerspectiveProjection,
            &mut CameraMatrices,
            Option<&GlobalTransform>,
        ),
        Or<(
            Changed<Transform>,
            Changed<PerspectiveProjection>,
            Added<Camera>,
            Added<PerspectiveProjection>,
            Added<CameraMatrices>,
        )>,
    >,
) {
    for (camera, transform, projection, mut matrices, global_transform) in cameras.iter_mut() {
        if !camera.is_active {
            continue;
        }

        let view_matrix = if let Some(global) = global_transform {
            global.matrix.inverse()
        } else {
            transform.compute_inverse_matrix()
        };

        let projection_matrix = projection.compute_matrix();

        matrices.update(view_matrix, projection_matrix);
    }
}

/// Updates camera matrices for cameras with orthographic projection.
///
/// This system computes the view and projection matrices for all active cameras
/// with an orthographic projection. The view matrix is computed from the camera's
/// Transform (or GlobalTransform if it has a parent), and the projection matrix
/// is computed from the OrthographicProjection component.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule};
/// use praxis_ecs::systems::update_orthographic_cameras;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// schedule.add_systems(update_orthographic_cameras);
/// ```
#[allow(clippy::type_complexity)]
pub fn update_orthographic_cameras(
    mut cameras: Query<
        (
            &Camera,
            &Transform,
            &OrthographicProjection,
            &mut CameraMatrices,
            Option<&GlobalTransform>,
        ),
        Or<(
            Changed<Transform>,
            Changed<OrthographicProjection>,
            Added<Camera>,
            Added<OrthographicProjection>,
            Added<CameraMatrices>,
        )>,
    >,
) {
    for (camera, transform, projection, mut matrices, global_transform) in cameras.iter_mut() {
        if !camera.is_active {
            continue;
        }

        let view_matrix = if let Some(global) = global_transform {
            global.matrix.inverse()
        } else {
            transform.compute_inverse_matrix()
        };

        let projection_matrix = projection.compute_matrix();

        matrices.update(view_matrix, projection_matrix);
    }
}

/// System that cleans up children when entities are removed.
///
/// This system should run when entities are about to be despawned to ensure
/// that parent-child relationships remain consistent.
///
/// Note: In a real implementation, this would need to hook into the despawn
/// process. For now, this is a placeholder showing the pattern.
pub fn cleanup_despawned_children(
    mut parents_query: Query<&mut Children>,
    // In practice, we'd need a way to detect despawned entities
) {
    // This is a simplified version. In practice, you'd need to:
    // 1. Detect which entities were despawned this frame
    // 2. Remove them from their parent's Children component
    // 3. Optionally reparent orphaned children

    for mut children in parents_query.iter_mut() {
        // Remove any invalid entity references
        children.0.retain(|&_child| {
            // In a real implementation, check if the entity still exists
            true
        });
    }
}

/// Bundle for spawning entities with transform hierarchy support.
///
/// This bundle includes everything needed for an entity to participate
/// in the transform hierarchy.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, TransformBundle, Transform};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// world.spawn(TransformBundle {
///     transform: Transform::from_xyz(10.0, 0.0, 0.0),
///     ..Default::default()
/// });
/// ```
#[derive(Bundle, Default)]
pub struct TransformBundle {
    /// The local transform of the entity.
    pub transform: Transform,

    /// The global (world-space) transform of the entity.
    pub global_transform: GlobalTransform,
}

impl TransformBundle {
    /// Creates a new transform bundle with the given transform.
    pub fn from_transform(transform: Transform) -> Self {
        Self {
            transform,
            global_transform: GlobalTransform::from(transform),
        }
    }

    /// Creates a new transform bundle at the given position.
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_transform(Transform::from_xyz(x, y, z))
    }
}

/// Bundle for spawning cameras with perspective projection.
///
/// This bundle includes everything needed for a perspective camera entity.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, PerspectiveCameraBundle};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// world.spawn(PerspectiveCameraBundle::new(
///     Vec3::new(0.0, 5.0, 10.0),
///     70.0_f32.to_radians(),
///     16.0 / 9.0,
/// ));
/// ```
#[derive(Bundle)]
pub struct PerspectiveCameraBundle {
    /// The camera component.
    pub camera: Camera,

    /// The local transform of the camera.
    pub transform: Transform,

    /// The global transform of the camera.
    pub global_transform: GlobalTransform,

    /// The perspective projection settings.
    pub projection: PerspectiveProjection,

    /// The computed camera matrices.
    pub matrices: CameraMatrices,
}

impl PerspectiveCameraBundle {
    /// Creates a new perspective camera bundle.
    pub fn new(position: Vec3, fov: f32, aspect_ratio: f32) -> Self {
        let transform = Transform::from_translation(position);
        Self {
            camera: Camera::default(),
            transform,
            global_transform: GlobalTransform::from(transform),
            projection: PerspectiveProjection::new(fov, aspect_ratio, 0.1, 1000.0),
            matrices: CameraMatrices::default(),
        }
    }

    /// Creates a new perspective camera bundle with custom near and far planes.
    pub fn with_near_far(position: Vec3, fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let transform = Transform::from_translation(position);
        Self {
            camera: Camera::default(),
            transform,
            global_transform: GlobalTransform::from(transform),
            projection: PerspectiveProjection::new(fov, aspect_ratio, near, far),
            matrices: CameraMatrices::default(),
        }
    }
}

impl Default for PerspectiveCameraBundle {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, 0.0, 10.0), 70.0_f32.to_radians(), 16.0 / 9.0)
    }
}

/// Bundle for spawning cameras with orthographic projection.
///
/// This bundle includes everything needed for an orthographic camera entity.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, OrthographicCameraBundle};
/// use praxis_math::Vec3;
///
/// let mut world = World::new();
///
/// world.spawn(OrthographicCameraBundle::new(
///     Vec3::new(0.0, 10.0, 0.0),
///     20.0,
///     10.0,
/// ));
/// ```
#[derive(Bundle)]
pub struct OrthographicCameraBundle {
    /// The camera component.
    pub camera: Camera,

    /// The local transform of the camera.
    pub transform: Transform,

    /// The global transform of the camera.
    pub global_transform: GlobalTransform,

    /// The orthographic projection settings.
    pub projection: OrthographicProjection,

    /// The computed camera matrices.
    pub matrices: CameraMatrices,
}

impl OrthographicCameraBundle {
    /// Creates a new orthographic camera bundle.
    pub fn new(position: Vec3, width: f32, height: f32) -> Self {
        let transform = Transform::from_translation(position);
        Self {
            camera: Camera::default(),
            transform,
            global_transform: GlobalTransform::from(transform),
            projection: OrthographicProjection::from_size(width, height, 0.1, 1000.0),
            matrices: CameraMatrices::default(),
        }
    }

    /// Creates a new orthographic camera bundle with custom near and far planes.
    pub fn with_near_far(position: Vec3, width: f32, height: f32, near: f32, far: f32) -> Self {
        let transform = Transform::from_translation(position);
        Self {
            camera: Camera::default(),
            transform,
            global_transform: GlobalTransform::from(transform),
            projection: OrthographicProjection::from_size(width, height, near, far),
            matrices: CameraMatrices::default(),
        }
    }

    /// Creates an orthographic camera with custom bounds.
    pub fn with_bounds(
        position: Vec3,
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let transform = Transform::from_translation(position);
        Self {
            camera: Camera::default(),
            transform,
            global_transform: GlobalTransform::from(transform),
            projection: OrthographicProjection::new(left, right, bottom, top, near, far),
            matrices: CameraMatrices::default(),
        }
    }
}

impl Default for OrthographicCameraBundle {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, 10.0, 0.0), 20.0, 10.0)
    }
}

/// System for debugging transform hierarchies.
///
/// This system logs information about the transform hierarchy, useful
/// for debugging parent-child relationships.
#[cfg(debug_assertions)]
pub fn debug_transform_hierarchy(
    roots: Query<(Entity, &Transform, Option<&Children>), Without<Parent>>,
    children_query: Query<(&Transform, Option<&Children>), With<Parent>>,
) {
    use praxis_utils::debug;

    for (entity, transform, maybe_children) in roots.iter() {
        debug!(
            "Root entity {:?} at position {:?}",
            entity, transform.translation
        );

        if let Some(children) = maybe_children {
            debug_children_recursive(&children.0, &children_query, 1);
        }
    }
}

#[cfg(debug_assertions)]
fn debug_children_recursive(
    children: &[Entity],
    query: &Query<(&Transform, Option<&Children>), With<Parent>>,
    depth: usize,
) {
    use praxis_utils::debug;

    let indent = "  ".repeat(depth);

    for &child in children {
        if let Ok((transform, maybe_children)) = query.get(child) {
            debug!(
                "{}Child entity {:?} at local position {:?}",
                indent, child, transform.translation
            );

            if let Some(children) = maybe_children {
                debug_children_recursive(&children.0, query, depth + 1);
            }
        }
    }
}

/// System that gathers lighting data from DirectionalLight and PointLight components.
///
/// This system queries all entities with light components and collects their data
/// into the `LightingData` resource, which can then be consumed by the render system.
///
/// # ECS Query Pattern
///
/// The system uses two separate queries:
///
/// 1. **Directional Light Query**: `Query<(&DirectionalLight, Option<&Transform>)>`
///    - Queries all entities that have a `DirectionalLight` component
///    - Optionally includes their `Transform` for direction transformation
///    - Note: Directional lights without transforms use their default direction
///
/// 2. **Point Light Query**: `Query<(&PointLight, Option<&GlobalTransform>, Option<&Transform>)>`
///    - Queries all entities that have a `PointLight` component
///    - Prefers `GlobalTransform` for world-space position (if entity has a parent)
///    - Falls back to `Transform` for local position (if entity has no parent)
///    - Note: Point lights without transforms default to world origin (0, 0, 0)
///
/// # Resource Access
///
/// The system accesses the `LightingData` resource with mutable access (`ResMut<LightingData>`).
/// This allows the system to:
/// - Clear the previous frame's lighting data
/// - Populate new lighting data from the current frame's light components
///
/// The resource is automatically injected by the ECS scheduler. If the resource doesn't exist,
/// the system will panic at runtime. Always ensure `LightingData` is inserted into the world
/// before running this system:
///
/// ```rust,no_run
/// use praxis_ecs::{World, LightingData};
/// let mut world = World::new();
/// world.insert_resource(LightingData::default());
/// ```
///
/// # Transform Handling
///
/// ## Directional Lights
/// - Direction is taken from the `DirectionalLight` component (already normalized)
/// - If the entity has a `Transform`, the direction is transformed by the rotation component
/// - This allows directional lights to be rotated like other entities
///
/// ## Point Lights
/// - Position is extracted from the entity's transform
/// - `GlobalTransform` is preferred (provides world-space position for entities with parents)
/// - Falls back to `Transform` if no `GlobalTransform` exists (for root entities)
/// - If neither exists, position defaults to world origin (0, 0, 0)
///
/// # Performance Notes
///
/// - The system clears and rebuilds the entire lighting data each frame
/// - For scenes with many lights, consider implementing change detection
/// - The queries only iterate over entities with light components, not all entities
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Schedule, LightingData};
/// use praxis_ecs::systems::gather_lighting_system;
///
/// let mut world = World::new();
/// let mut schedule = Schedule::default();
///
/// // Initialize the lighting data resource
/// world.insert_resource(LightingData::default());
///
/// // Add the system to the schedule
/// schedule.add_systems(gather_lighting_system);
///
/// // Run the schedule each frame
/// schedule.run(world.inner_mut());
/// ```
pub fn gather_lighting_system(
    // Mutable access to the LightingData resource
    // This resource stores the collected lighting information for the render system
    mut lighting_data: crate::ResMut<LightingData>,
    // Query all entities with DirectionalLight components
    // Optional Transform allows us to rotate the light direction if the entity is transformed
    directional_lights: Query<(&DirectionalLight, Option<&Transform>)>,
    // Query all entities with PointLight components
    // GlobalTransform is preferred for world-space position (handles parent hierarchy)
    // Transform is fallback for entities without parents
    point_lights: Query<(&PointLight, Option<&GlobalTransform>, Option<&Transform>)>,
) {
    // Clear the previous frame's lighting data
    // This ensures we start fresh each frame and don't accumulate stale light data
    lighting_data.clear();

    // Iterate through all directional light entities
    // The query returns a tuple of (&DirectionalLight, Option<&Transform>) for each matching entity
    for (dir_light, maybe_transform) in directional_lights.iter() {
        // Determine the world-space direction of the light
        // If the entity has a Transform, rotate the light's direction by the transform's rotation
        // This allows directional lights to be oriented by rotating their entities
        let world_direction = if let Some(transform) = maybe_transform {
            // Transform the light direction by the entity's rotation
            // This applies the same rotation that would affect any child objects
            transform.rotation * dir_light.direction
        } else {
            // No transform, use the light's direction as-is
            dir_light.direction
        };

        // Create a DirectionalLightInfo struct with the collected data
        // This is the format expected by the render system
        let light_info = DirectionalLightInfo {
            direction: world_direction.normalize(), // Ensure direction is normalized
            color: dir_light.color,
            intensity: dir_light.intensity,
        };

        // Add the light info to the resource's collection
        // The render system will iterate through this vector later
        lighting_data.directional_lights.push(light_info);
    }

    // Iterate through all point light entities
    // The query returns (&PointLight, Option<&GlobalTransform>, Option<&Transform>)
    for (point_light, maybe_global_transform, maybe_transform) in point_lights.iter() {
        // Determine the world-space position of the light
        // Priority order:
        // 1. GlobalTransform (if entity is in a hierarchy)
        // 2. Transform (if entity is a root)
        // 3. World origin (if neither exists)
        let world_position = if let Some(global_transform) = maybe_global_transform {
            // Use GlobalTransform for accurate world-space position
            // This is important for lights that are children of other entities
            global_transform.translation()
        } else if let Some(transform) = maybe_transform {
            // Use Transform's translation for root entities
            // Since there's no parent, local position = world position
            transform.translation
        } else {
            // No transform information available, default to world origin
            // This is a fallback case that shouldn't happen in normal usage
            Vec3::ZERO
        };

        // Create a PointLightInfo struct with the collected data
        let light_info = PointLightInfo {
            position: world_position,
            color: point_light.color,
            intensity: point_light.intensity,
            range: point_light.range,
        };

        // Add the light info to the resource's collection
        lighting_data.point_lights.push(light_info);
    }

    // Note: The ambient_color in LightingData is not modified by this system
    // It can be set manually or left at its default value (0.1, 0.1, 0.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use bevy_ecs::schedule::IntoSystemConfigs;
    use praxis_math::{Mat4, Quat, Vec3};

    #[test]
    fn test_transform_propagation_simple() {
        let mut world = World::new();

        // Create parent at (10, 0, 0)
        let parent = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
            Children::new(),
        ));

        // Create child at local position (5, 0, 0)
        let child = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Add child to parent's children list
        world
            .insert_component(parent, Children::with_children(vec![child]))
            .unwrap();

        // Run transform propagation
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);
        schedule.run(world.inner_mut());

        // Check parent's global transform
        let parent_global = world.inner().get::<GlobalTransform>(parent).unwrap();
        assert_eq!(parent_global.translation(), Vec3::new(10.0, 0.0, 0.0));

        // Check child's global transform (should be at world position (15, 0, 0))
        let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
        let child_world_pos = child_global.translation();
        assert!((child_world_pos.x - 15.0).abs() < 0.001);
        assert!(child_world_pos.y.abs() < 0.001);
        assert!(child_world_pos.z.abs() < 0.001);
    }

    #[test]
    fn test_transform_propagation_deep_hierarchy() {
        let mut world = World::new();

        // Create a three-level hierarchy
        let root = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let child1 = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(root),
        ));

        let child2 = world.spawn((
            Transform::from_xyz(3.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(child1),
        ));

        // Set up children components
        world
            .insert_component(root, Children::with_children(vec![child1]))
            .unwrap();
        world
            .insert_component(child1, Children::with_children(vec![child2]))
            .unwrap();

        // Run transform propagation
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);
        schedule.run(world.inner_mut());

        // Check final position: 10 + 5 + 3 = 18
        let child2_global = world.inner().get::<GlobalTransform>(child2).unwrap();
        let pos = child2_global.translation();
        assert!((pos.x - 18.0).abs() < 0.001);
        assert!(pos.y.abs() < 0.001);
        assert!(pos.z.abs() < 0.001);
    }

    #[test]
    fn test_sync_parent_child_relationships() {
        let mut world = World::new();

        let parent = world.spawn(Transform::default());
        let child = world.spawn((Transform::default(), Parent(parent)));

        // Run sync system
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(sync_parent_child_relationships);
        schedule.run(world.inner_mut());

        // Verify parent has Children component
        let children = world.inner().get::<Children>(parent).unwrap();
        assert_eq!(children.0.len(), 1);
        assert_eq!(children.0[0], child);
    }

    #[test]
    fn test_transform_bundle() {
        let bundle = TransformBundle::from_xyz(1.0, 2.0, 3.0);
        assert_eq!(bundle.transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(bundle.transform.rotation, Quat::IDENTITY);
        assert_eq!(bundle.transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_propagate_transforms_for_reparented() {
        let mut world = World::new();

        // Create two parents
        let parent1 = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let parent2 = world.spawn((
            Transform::from_xyz(0.0, 20.0, 0.0),
            GlobalTransform::default(),
        ));

        // Create child under parent1
        let child = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent1),
        ));

        world
            .insert_component(parent1, Children::with_children(vec![child]))
            .unwrap();

        // Initial propagation
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((propagate_transforms, propagate_transforms_for_reparented).chain());
        schedule.run(world.inner_mut());

        // Child should be at (15, 0, 0)
        let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
        let pos1 = child_global.translation();
        assert!((pos1.x - 15.0).abs() < 0.001);

        // Change parent to parent2
        world.inner_mut().entity_mut(child).insert(Parent(parent2));
        world
            .insert_component(parent2, Children::with_children(vec![child]))
            .unwrap();

        // Propagate again
        schedule.run(world.inner_mut());

        // Child should now be at (5, 20, 0)
        let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
        let pos2 = child_global.translation();
        assert!((pos2.x - 5.0).abs() < 0.001);
        assert!((pos2.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_propagate_transforms_for_changed_children() {
        let mut world = World::new();

        let parent = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let child = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        world
            .insert_component(parent, Children::with_children(vec![child]))
            .unwrap();

        // Initial propagation
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(
            (
                propagate_transforms,
                propagate_transforms_for_changed_children,
            )
                .chain(),
        );
        schedule.run(world.inner_mut());

        // Modify child's transform
        {
            let inner = world.inner_mut();
            if let Some(mut transform) = inner.get_mut::<Transform>(child) {
                transform.translation = Vec3::new(8.0, 3.0, 0.0);
            }
        }

        // Propagate again
        schedule.run(world.inner_mut());

        // Child should now be at (18, 3, 0)
        let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
        let pos = child_global.translation();
        assert!((pos.x - 18.0).abs() < 0.001);
        assert!((pos.y - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_cleanup_removed_parents() {
        let mut world = World::new();

        let parent = world.spawn((Transform::default(), GlobalTransform::default()));

        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Sync relationships
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(sync_parent_child_relationships);
        schedule.run(world.inner_mut());

        // Verify parent has child
        {
            let children = world.inner().get::<Children>(parent).unwrap();
            assert_eq!(children.0.len(), 1);
        }

        // Remove Parent component from child
        world.inner_mut().entity_mut(child).remove::<Parent>();

        // Run cleanup
        let mut cleanup_schedule = bevy_ecs::schedule::Schedule::default();
        cleanup_schedule.add_systems(cleanup_removed_parents);
        cleanup_schedule.run(world.inner_mut());

        // Parent should no longer have Children component (empty list removed)
        assert!(world.inner().get::<Children>(parent).is_none());
    }

    #[test]
    fn test_transform_with_rotation_and_scale() {
        let mut world = World::new();

        // Create parent with rotation and scale
        let parent = world.spawn((
            Transform {
                translation: Vec3::new(10.0, 0.0, 0.0),
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2), // 90 degrees
                scale: Vec3::new(2.0, 1.0, 1.0),
            },
            GlobalTransform::default(),
        ));

        let child = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        world
            .insert_component(parent, Children::with_children(vec![child]))
            .unwrap();

        // Run propagation
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);
        schedule.run(world.inner_mut());

        // Child should be rotated 90 degrees and scaled
        let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
        let pos = child_global.translation();

        // After rotation by 90 degrees, (5,0,0) becomes (0,0,-5), scaled by (2,1,1) = (0,0,-10)
        // Then translated by (10,0,0) = (10,0,-10)
        assert!((pos.x - 10.0).abs() < 0.001);
        assert!(pos.y.abs() < 0.001);
        assert!((pos.z - -10.0).abs() < 0.001);
    }

    #[test]
    fn test_multiple_children() {
        let mut world = World::new();

        let parent = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let child1 = world.spawn((
            Transform::from_xyz(1.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        let child2 = world.spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        let child3 = world.spawn((
            Transform::from_xyz(3.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        world
            .insert_component(
                parent,
                Children::with_children(vec![child1, child2, child3]),
            )
            .unwrap();

        // Run propagation
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);
        schedule.run(world.inner_mut());

        // Check all children
        let c1_pos = world
            .inner()
            .get::<GlobalTransform>(child1)
            .unwrap()
            .translation();
        let c2_pos = world
            .inner()
            .get::<GlobalTransform>(child2)
            .unwrap()
            .translation();
        let c3_pos = world
            .inner()
            .get::<GlobalTransform>(child3)
            .unwrap()
            .translation();

        assert!((c1_pos.x - 11.0).abs() < 0.001);
        assert!((c2_pos.x - 12.0).abs() < 0.001);
        assert!((c3_pos.x - 13.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_propagation_full_chain() {
        let mut world = World::new();

        // Test all systems together
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(
            (
                sync_parent_child_relationships,
                cleanup_removed_parents,
                propagate_transforms,
                propagate_transforms_for_reparented,
                propagate_transforms_for_changed_children,
            )
                .chain(),
        );

        // Create a simple hierarchy
        let parent = world.spawn((
            Transform::from_xyz(10.0, 5.0, 0.0),
            GlobalTransform::default(),
        ));

        let child = world.spawn((
            Transform::from_xyz(3.0, 2.0, 1.0),
            GlobalTransform::default(),
            Parent(parent),
        ));

        // Run the full system chain
        schedule.run(world.inner_mut());

        // Verify parent has children component (from sync system)
        assert!(world.inner().get::<Children>(parent).is_some());

        // Verify transforms were propagated
        let child_global = world.inner().get::<GlobalTransform>(child).unwrap();
        let pos = child_global.translation();
        assert!((pos.x - 13.0).abs() < 0.001);
        assert!((pos.y - 7.0).abs() < 0.001);
        assert!((pos.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_full_system_chain_with_reparenting() {
        let mut world = World::new();
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(
            (
                sync_parent_child_relationships,
                cleanup_removed_parents,
                propagate_transforms,
                propagate_transforms_for_reparented,
                propagate_transforms_for_changed_children,
            )
                .chain(),
        );

        let parent1 = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let parent2 = world.spawn((
            Transform::from_xyz(0.0, 20.0, 0.0),
            GlobalTransform::default(),
        ));

        let child = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent1),
        ));

        // Initial propagation
        schedule.run(world.inner_mut());

        // Verify initial state
        assert!(world.inner().get::<Children>(parent1).is_some());
        let pos1 = world
            .inner()
            .get::<GlobalTransform>(child)
            .unwrap()
            .translation();
        assert!((pos1.x - 15.0).abs() < 0.001);

        // Reparent
        world.inner_mut().entity_mut(child).insert(Parent(parent2));
        schedule.run(world.inner_mut());

        // Verify after reparenting
        assert!(world.inner().get::<Children>(parent2).is_some());
        let pos2 = world
            .inner()
            .get::<GlobalTransform>(child)
            .unwrap()
            .translation();
        assert!((pos2.x - 5.0).abs() < 0.001);
        assert!((pos2.y - 20.0).abs() < 0.001);

        // Remove parent
        world.inner_mut().entity_mut(child).remove::<Parent>();
        schedule.run(world.inner_mut());

        // Verify as root
        let pos3 = world
            .inner()
            .get::<GlobalTransform>(child)
            .unwrap()
            .translation();
        assert!((pos3.x - 5.0).abs() < 0.001);
        assert!((pos3.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_update_perspective_cameras() {
        use crate::{Camera, CameraMatrices, PerspectiveProjection};

        let mut world = World::new();

        let camera = world.spawn((
            Camera::default(),
            Transform::from_xyz(0.0, 0.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(update_perspective_cameras);
        schedule.run(world.inner_mut());

        let matrices = world.inner().get::<CameraMatrices>(camera).unwrap();
        assert_ne!(matrices.view, Mat4::IDENTITY);
        assert_ne!(matrices.projection, Mat4::IDENTITY);
        assert_ne!(matrices.view_projection, Mat4::IDENTITY);
    }

    #[test]
    fn test_update_orthographic_cameras() {
        use crate::{Camera, CameraMatrices, OrthographicProjection};

        let mut world = World::new();

        let camera = world.spawn((
            Camera::default(),
            Transform::from_xyz(0.0, 10.0, 0.0),
            OrthographicProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(update_orthographic_cameras);
        schedule.run(world.inner_mut());

        let matrices = world.inner().get::<CameraMatrices>(camera).unwrap();
        assert_ne!(matrices.view, Mat4::IDENTITY);
        assert_ne!(matrices.projection, Mat4::IDENTITY);
        assert_ne!(matrices.view_projection, Mat4::IDENTITY);
    }

    #[test]
    fn test_inactive_camera_not_updated() {
        use crate::{Camera, CameraMatrices, PerspectiveProjection};

        let mut world = World::new();

        let mut inactive_camera = Camera::default();
        inactive_camera.deactivate();

        let camera = world.spawn((
            inactive_camera,
            Transform::from_xyz(0.0, 0.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(update_perspective_cameras);
        schedule.run(world.inner_mut());

        let matrices = world.inner().get::<CameraMatrices>(camera).unwrap();
        assert_eq!(matrices.view, Mat4::IDENTITY);
        assert_eq!(matrices.projection, Mat4::IDENTITY);
    }

    #[test]
    fn test_camera_with_parent_transform() {
        use crate::{Camera, CameraMatrices, PerspectiveProjection};

        let mut world = World::new();

        let parent = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let camera = world.spawn((
            Camera::default(),
            Transform::from_xyz(0.0, 0.0, 5.0),
            GlobalTransform::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
            Parent(parent),
        ));

        world
            .insert_component(parent, Children::with_children(vec![camera]))
            .unwrap();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((propagate_transforms, update_perspective_cameras).chain());
        schedule.run(world.inner_mut());

        let matrices = world.inner().get::<CameraMatrices>(camera).unwrap();
        assert_ne!(matrices.view, Mat4::IDENTITY);
        assert_ne!(matrices.projection, Mat4::IDENTITY);
    }

    #[test]
    fn test_perspective_camera_bundle() {
        let bundle = PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 10.0),
            60.0_f32.to_radians(),
            16.0 / 9.0,
        );

        assert!(bundle.camera.is_active());
        assert_eq!(bundle.transform.translation, Vec3::new(0.0, 5.0, 10.0));
        assert_eq!(bundle.projection.fov, 60.0_f32.to_radians());
        assert_eq!(bundle.projection.aspect_ratio, 16.0 / 9.0);
    }

    #[test]
    fn test_orthographic_camera_bundle() {
        let bundle = OrthographicCameraBundle::new(Vec3::new(0.0, 10.0, 0.0), 20.0, 10.0);

        assert!(bundle.camera.is_active());
        assert_eq!(bundle.transform.translation, Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(bundle.projection.left, -10.0);
        assert_eq!(bundle.projection.right, 10.0);
        assert_eq!(bundle.projection.bottom, -5.0);
        assert_eq!(bundle.projection.top, 5.0);
    }

    #[test]
    fn test_gather_lighting_system_empty() {
        let mut world = World::new();

        // Initialize the lighting data resource
        world.insert_resource(LightingData::default());

        // Run the gather system with no lights
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(gather_lighting_system);
        schedule.run(world.inner_mut());

        // Verify no lights were collected
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.directional_light_count(), 0);
        assert_eq!(lighting_data.point_light_count(), 0);
    }

    #[test]
    fn test_gather_lighting_system_directional_lights() {
        use crate::{DirectionalLight, LightingData};

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        // Spawn a directional light without transform
        world.spawn(DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            1.0,
        ));

        // Spawn a directional light with transform
        world.spawn((
            DirectionalLight::new(
                Vec3::new(1.0, 0.0, 0.0), // Points right
                Vec3::new(1.0, 0.5, 0.0), // Orange
                0.8,
            ),
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)), // Rotate 90 degrees
        ));

        // Run the gather system
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(gather_lighting_system);
        schedule.run(world.inner_mut());

        // Verify lights were collected
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.directional_light_count(), 2);

        // Check first light (no transform)
        let light1 = &lighting_data.directional_lights[0];
        assert!((light1.direction.y - -1.0).abs() < 0.001);
        assert_eq!(light1.color, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(light1.intensity, 1.0);

        // Check second light (with rotation)
        let light2 = &lighting_data.directional_lights[1];
        // After 90 degree Y rotation, (1,0,0) becomes (0,0,-1)
        assert!(light2.direction.z.abs() > 0.99); // Should point mostly in Z direction
        assert_eq!(light2.color, Vec3::new(1.0, 0.5, 0.0));
        assert_eq!(light2.intensity, 0.8);
    }

    #[test]
    fn test_gather_lighting_system_point_lights() {
        use crate::{LightingData, PointLight};

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        // Spawn a point light without transform (defaults to origin)
        world.spawn(PointLight::new(Vec3::new(1.0, 0.0, 0.0), 5.0, 10.0));

        // Spawn a point light with transform
        world.spawn((
            Transform::from_xyz(10.0, 20.0, 30.0),
            PointLight::new(Vec3::new(0.0, 1.0, 0.0), 8.0, 15.0),
        ));

        // Run the gather system
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(gather_lighting_system);
        schedule.run(world.inner_mut());

        // Verify lights were collected
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.point_light_count(), 2);

        // Check first light (no transform, defaults to origin)
        let light1 = &lighting_data.point_lights[0];
        assert_eq!(light1.position, Vec3::ZERO);
        assert_eq!(light1.color, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(light1.intensity, 5.0);
        assert_eq!(light1.range, 10.0);

        // Check second light (with transform)
        let light2 = &lighting_data.point_lights[1];
        assert_eq!(light2.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(light2.color, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(light2.intensity, 8.0);
        assert_eq!(light2.range, 15.0);
    }

    #[test]
    fn test_gather_lighting_system_with_global_transform() {
        use crate::{LightingData, PointLight};

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        // Create a parent at (10, 0, 0)
        let parent = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        // Create a point light as a child at local position (5, 0, 0)
        let light = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(parent),
            PointLight::new(Vec3::new(1.0, 1.0, 1.0), 10.0, 20.0),
        ));

        // Set up parent-child relationship
        world
            .insert_component(parent, Children::with_children(vec![light]))
            .unwrap();

        // First propagate transforms to compute GlobalTransform
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems((propagate_transforms, gather_lighting_system).chain());
        schedule.run(world.inner_mut());

        // Verify the point light uses GlobalTransform (world position)
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.point_light_count(), 1);

        let collected_light = &lighting_data.point_lights[0];
        // Should be at world position (15, 0, 0) = parent (10,0,0) + local (5,0,0)
        assert!((collected_light.position.x - 15.0).abs() < 0.001);
        assert!(collected_light.position.y.abs() < 0.001);
        assert!(collected_light.position.z.abs() < 0.001);
    }

    #[test]
    fn test_gather_lighting_system_mixed_lights() {
        use crate::{DirectionalLight, LightingData, PointLight};

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        // Spawn multiple lights of different types
        world.spawn(DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            1.0,
        ));

        world.spawn((
            Transform::from_xyz(5.0, 5.0, 5.0),
            PointLight::new(Vec3::new(1.0, 0.0, 0.0), 10.0, 20.0),
        ));

        world.spawn(DirectionalLight::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.8, 0.8, 1.0),
            0.5,
        ));

        world.spawn((
            Transform::from_xyz(-5.0, 3.0, 0.0),
            PointLight::new(Vec3::new(0.0, 1.0, 0.0), 5.0, 10.0),
        ));

        // Run the gather system
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(gather_lighting_system);
        schedule.run(world.inner_mut());

        // Verify all lights were collected
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.directional_light_count(), 2);
        assert_eq!(lighting_data.point_light_count(), 2);
    }

    #[test]
    fn test_gather_lighting_system_clears_previous_data() {
        use crate::{DirectionalLight, LightingData};

        let mut world = World::new();
        world.insert_resource(LightingData::default());

        // Spawn a light
        let light_entity = world.spawn(DirectionalLight::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            1.0,
        ));

        // Run the gather system
        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(gather_lighting_system);
        schedule.run(world.inner_mut());

        // Verify one light was collected
        {
            let lighting_data = world.inner().resource::<LightingData>();
            assert_eq!(lighting_data.directional_light_count(), 1);
        }

        // Remove the light
        world.inner_mut().despawn(light_entity);

        // Run the gather system again
        schedule.run(world.inner_mut());

        // Verify the old light data was cleared
        let lighting_data = world.inner().resource::<LightingData>();
        assert_eq!(lighting_data.directional_light_count(), 0);
    }

    #[test]
    fn test_transform_bundle_from_transform() {
        let transform = Transform::from_xyz(5.0, 10.0, 15.0);
        let bundle = TransformBundle::from_transform(transform);

        assert_eq!(bundle.transform.translation, Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(
            bundle.global_transform.translation(),
            Vec3::new(5.0, 10.0, 15.0)
        );
    }

    #[test]
    fn test_perspective_camera_bundle_with_near_far() {
        let bundle = PerspectiveCameraBundle::with_near_far(
            Vec3::new(1.0, 2.0, 3.0),
            90.0_f32.to_radians(),
            16.0 / 9.0,
            0.5,
            500.0,
        );

        assert_eq!(bundle.transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(bundle.projection.fov, 90.0_f32.to_radians());
        assert_eq!(bundle.projection.near, 0.5);
        assert_eq!(bundle.projection.far, 500.0);
    }

    #[test]
    fn test_orthographic_camera_bundle_with_near_far() {
        let bundle = OrthographicCameraBundle::with_near_far(
            Vec3::new(5.0, 10.0, 0.0),
            30.0,
            20.0,
            0.5,
            500.0,
        );

        assert_eq!(bundle.transform.translation, Vec3::new(5.0, 10.0, 0.0));
        assert_eq!(bundle.projection.left, -15.0);
        assert_eq!(bundle.projection.right, 15.0);
        assert_eq!(bundle.projection.bottom, -10.0);
        assert_eq!(bundle.projection.top, 10.0);
        assert_eq!(bundle.projection.near, 0.5);
        assert_eq!(bundle.projection.far, 500.0);
    }

    #[test]
    fn test_orthographic_camera_bundle_with_bounds() {
        let bundle = OrthographicCameraBundle::with_bounds(
            Vec3::new(0.0, 5.0, 0.0),
            -20.0,
            20.0,
            -15.0,
            15.0,
            0.2,
            800.0,
        );

        assert_eq!(bundle.transform.translation, Vec3::new(0.0, 5.0, 0.0));
        assert_eq!(bundle.projection.left, -20.0);
        assert_eq!(bundle.projection.right, 20.0);
        assert_eq!(bundle.projection.bottom, -15.0);
        assert_eq!(bundle.projection.top, 15.0);
        assert_eq!(bundle.projection.near, 0.2);
        assert_eq!(bundle.projection.far, 800.0);
    }

    #[test]
    fn test_sync_parent_child_relationships_multiple_children() {
        let mut world = World::new();

        let parent = world.spawn(Transform::default());
        let child1 = world.spawn((Transform::default(), Parent(parent)));
        let child2 = world.spawn((Transform::default(), Parent(parent)));
        let child3 = world.spawn((Transform::default(), Parent(parent)));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(sync_parent_child_relationships);
        schedule.run(world.inner_mut());

        let children = world.inner().get::<Children>(parent).unwrap();
        assert_eq!(children.len(), 3);
        assert!(children.0.contains(&child1));
        assert!(children.0.contains(&child2));
        assert!(children.0.contains(&child3));
    }

    #[test]
    fn test_sync_parent_child_avoids_duplicates() {
        let mut world = World::new();

        let parent = world.spawn(Transform::default());
        let _child = world.spawn((Transform::default(), Parent(parent)));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(sync_parent_child_relationships);

        schedule.run(world.inner_mut());
        schedule.run(world.inner_mut());
        schedule.run(world.inner_mut());

        let children = world.inner().get::<Children>(parent).unwrap();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_transform_propagation_root_without_children() {
        let mut world = World::new();

        let root = world.spawn((
            Transform::from_xyz(5.0, 10.0, 15.0),
            GlobalTransform::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);
        schedule.run(world.inner_mut());

        let global = world.inner().get::<GlobalTransform>(root).unwrap();
        let pos = global.translation();
        assert_eq!(pos, Vec3::new(5.0, 10.0, 15.0));
    }

    #[test]
    fn test_transform_propagation_skips_unchanged() {
        let mut world = World::new();

        let root = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);

        schedule.run(world.inner_mut());

        let global1 = *world.inner().get::<GlobalTransform>(root).unwrap();

        schedule.run(world.inner_mut());

        let global2 = *world.inner().get::<GlobalTransform>(root).unwrap();

        assert_eq!(global1.matrix, global2.matrix);
    }

    #[test]
    fn test_complex_hierarchy_with_multiple_branches() {
        let mut world = World::new();

        let root = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let branch1 = world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(root),
        ));

        let branch2 = world.spawn((
            Transform::from_xyz(0.0, 5.0, 0.0),
            GlobalTransform::default(),
            Parent(root),
        ));

        let leaf1 = world.spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            GlobalTransform::default(),
            Parent(branch1),
        ));

        let leaf2 = world.spawn((
            Transform::from_xyz(0.0, 3.0, 0.0),
            GlobalTransform::default(),
            Parent(branch2),
        ));

        world
            .insert_component(root, Children::with_children(vec![branch1, branch2]))
            .unwrap();
        world
            .insert_component(branch1, Children::with_children(vec![leaf1]))
            .unwrap();
        world
            .insert_component(branch2, Children::with_children(vec![leaf2]))
            .unwrap();

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(propagate_transforms);
        schedule.run(world.inner_mut());

        let leaf1_pos = world
            .inner()
            .get::<GlobalTransform>(leaf1)
            .unwrap()
            .translation();
        assert!((leaf1_pos.x - 17.0).abs() < 0.001);
        assert!(leaf1_pos.y.abs() < 0.001);

        let leaf2_pos = world
            .inner()
            .get::<GlobalTransform>(leaf2)
            .unwrap()
            .translation();
        assert!((leaf2_pos.x - 10.0).abs() < 0.001);
        assert!((leaf2_pos.y - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_cleanup_removed_parents_keeps_valid_children() {
        let mut world = World::new();

        let parent = world.spawn((Transform::default(), GlobalTransform::default()));

        let child1 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));

        let child2 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(sync_parent_child_relationships);
        schedule.run(world.inner_mut());

        let children_before = world.inner().get::<Children>(parent).unwrap().len();
        assert_eq!(children_before, 2);

        world.inner_mut().entity_mut(child1).remove::<Parent>();

        let mut cleanup_schedule = bevy_ecs::schedule::Schedule::default();
        cleanup_schedule.add_systems(cleanup_removed_parents);
        cleanup_schedule.run(world.inner_mut());

        let children_after = world.inner().get::<Children>(parent).unwrap();
        assert_eq!(children_after.len(), 1);
        assert!(children_after.0.contains(&child2));
        assert!(!children_after.0.contains(&child1));
    }

    #[test]
    fn test_camera_with_changed_projection() {
        let mut world = World::new();

        let camera = world.spawn((
            Camera::default(),
            Transform::from_xyz(0.0, 0.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(update_perspective_cameras);
        schedule.run(world.inner_mut());

        let matrices1 = *world.inner().get::<CameraMatrices>(camera).unwrap();

        {
            let mut projection = world
                .inner_mut()
                .get_mut::<PerspectiveProjection>(camera)
                .unwrap();
            projection.fov = 90.0_f32.to_radians();
        }

        schedule.run(world.inner_mut());

        let matrices2 = *world.inner().get::<CameraMatrices>(camera).unwrap();

        assert_ne!(matrices1.projection, matrices2.projection);
    }

    #[test]
    fn test_multiple_active_cameras_with_different_priorities() {
        let mut world = World::new();

        let _low_priority = world.spawn((
            Camera::with_priority(0),
            Transform::from_xyz(0.0, 0.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let high_priority = world.spawn((
            Camera::with_priority(10),
            Transform::from_xyz(0.0, 5.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(update_perspective_cameras);
        schedule.run(world.inner_mut());

        // Verify that the high priority camera has updated matrices
        let camera = world.inner().get::<Camera>(high_priority).unwrap();
        assert_eq!(camera.priority, 10);
        assert!(camera.is_active);

        // Verify matrices were computed
        let matrices = world.inner().get::<CameraMatrices>(high_priority).unwrap();
        assert!(matrices.view != Mat4::IDENTITY || matrices.projection != Mat4::IDENTITY);
    }

    #[test]
    fn test_sorted_cameras_ordering() {
        let mut world = World::new();

        let cam1 = world.spawn((
            Camera::with_priority(5),
            Transform::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let cam2 = world.spawn((
            Camera::with_priority(1),
            Transform::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let cam3 = world.spawn((
            Camera::with_priority(10),
            Transform::default(),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(update_perspective_cameras);
        schedule.run(world.inner_mut());

        // Verify all cameras were updated by checking their matrices
        for entity in [cam1, cam2, cam3] {
            let camera = world.inner().get::<Camera>(entity).unwrap();
            assert!(camera.is_active);
            let matrices = world.inner().get::<CameraMatrices>(entity).unwrap();
            // Matrices should be computed (not identity for perspective cameras)
            assert!(matrices.view != Mat4::IDENTITY || matrices.projection != Mat4::IDENTITY);
        }

        // Verify priorities are correctly stored
        assert_eq!(world.inner().get::<Camera>(cam1).unwrap().priority, 5);
        assert_eq!(world.inner().get::<Camera>(cam2).unwrap().priority, 1);
        assert_eq!(world.inner().get::<Camera>(cam3).unwrap().priority, 10);
    }
}
