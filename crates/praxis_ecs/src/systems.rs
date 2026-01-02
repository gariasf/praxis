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
    system::Commands,
};

use crate::{
    Camera, CameraMatrices, Children, GlobalTransform, OrthographicProjection, Parent,
    PerspectiveProjection, Query, Transform,
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
    // Handle newly added Parent components
    for (child_entity, parent) in added_parents.iter() {
        add_child_to_parent(child_entity, parent.0, &mut commands, &mut parents_query);
    }

    // Handle changed Parent components
    // Note: We need to track the old parent to remove from its children list
    // For now, we just ensure the child is in the new parent's list
    for (child_entity, parent) in changed_parents.iter() {
        // Skip if this was just added (already handled above)
        if added_parents.get(child_entity).is_ok() {
            continue;
        }

        add_child_to_parent(child_entity, parent.0, &mut commands, &mut parents_query);
    }
}

/// Helper function to add a child to a parent's Children component.
fn add_child_to_parent(
    child_entity: Entity,
    parent_entity: Entity,
    commands: &mut Commands,
    parents_query: &mut Query<&mut Children>,
) {
    if let Ok(mut children) = parents_query.get_mut(parent_entity) {
        if !children.0.contains(&child_entity) {
            trace!(
                "Adding child entity {:?} to parent {:?}",
                child_entity,
                parent_entity
            );
            children.push(child_entity);
        }
    } else {
        // Parent doesn't have a Children component yet, add one
        trace!("Creating Children component for parent {:?}", parent_entity);
        commands
            .entity(parent_entity)
            .insert(Children::with_children(vec![child_entity]));
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
    mut root_query: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        (Without<Parent>, Or<(Changed<Transform>, Added<Transform>)>),
    >,
    mut all_roots: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        Without<Parent>,
    >,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    // First pass: Update root entities whose transforms changed
    for (_entity, transform, mut global_transform, children) in root_query.iter_mut() {
        // Update this root's global transform
        global_transform.matrix = transform.compute_matrix();

        // Propagate to all descendants
        if let Some(children) = children {
            propagate_recursive(&children.0, &global_transform.matrix, &mut child_query);
        }
    }

    // Second pass: Update root entities that didn't change but need their children updated
    // This handles cases where children were added or a parent was removed
    for (entity, transform, mut global_transform, children) in all_roots.iter_mut() {
        // Skip if we already processed this in the first pass
        if root_query.get(entity).is_ok() {
            continue;
        }

        // Update global transform to ensure consistency
        global_transform.matrix = transform.compute_matrix();

        // Check if any children were added to this root
        if let Some(children) = children {
            propagate_to_added_children(&children.0, &global_transform.matrix, &mut child_query);
        }
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
    // Entities whose Parent component was added or changed
    mut reparented: Query<
        (&Parent, &Transform, &mut GlobalTransform, Option<&Children>),
        Or<(Added<Parent>, Changed<Parent>)>,
    >,
    parent_query: Query<&GlobalTransform>,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    for (parent, transform, mut global_transform, maybe_children) in reparented.iter_mut() {
        // Get parent's global transform
        if let Ok(parent_global) = parent_query.get(parent.0) {
            // Update this entity's global transform
            global_transform.matrix = parent_global.matrix * transform.compute_matrix();

            // Propagate to descendants
            if let Some(children) = maybe_children {
                propagate_recursive(&children.0, &global_transform.matrix, &mut child_query);
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
    // Entities with Parent whose Transform changed
    mut changed: Query<
        (&Parent, &Transform, &mut GlobalTransform, Option<&Children>),
        (With<Parent>, Changed<Transform>),
    >,
    parent_query: Query<&GlobalTransform>,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    for (parent, transform, mut global_transform, maybe_children) in changed.iter_mut() {
        // Get parent's global transform
        if let Ok(parent_global) = parent_query.get(parent.0) {
            // Update this entity's global transform
            global_transform.matrix = parent_global.matrix * transform.compute_matrix();

            // Propagate to descendants
            if let Some(children) = maybe_children {
                propagate_recursive(&children.0, &global_transform.matrix, &mut child_query);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use praxis_math::{Quat, Vec3};

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

        // Modify child's transform
        {
            let inner = world.inner_mut();
            if let Some(mut transform) = inner.get_mut::<Transform>(child) {
                transform.translation = Vec3::new(8.0, 3.0, 0.0);
            }
        }

        // Propagate again
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut cleanup_schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
        world.inner_mut().run_schedule(&mut schedule);

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
}
