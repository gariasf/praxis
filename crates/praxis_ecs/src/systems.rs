//! Systems for the Praxis ECS.
//!
//! This module provides pre-built systems that handle common game engine
//! functionality like transform propagation, parent-child relationships,
//! and other core behaviors.

use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    query::{Changed, With, Without},
    schedule::SystemSet,
    system::Commands,
};

use crate::{Children, GlobalTransform, Parent, Query, Transform};
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

/// Propagates transforms through the entity hierarchy.
///
/// This system updates the GlobalTransform component based on the local Transform
/// and the parent's GlobalTransform. It ensures that child entities inherit
/// the transformations of their parents.
///
/// # Algorithm
///
/// 1. First, update all root entities (entities with Transform but no Parent)
/// 2. Then, recursively update children based on their parent's GlobalTransform
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
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        Without<Parent>,
    >,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
    children_query: Query<&Children>,
    _parent_query: Query<&GlobalTransform>,
) {
    // Update root entities (no parent)
    for (entity, transform, mut global_transform, children) in root_query.iter_mut() {
        *global_transform = GlobalTransform::from(*transform);

        // If this root has children, propagate to them
        if let Some(children) = children {
            propagate_to_children(
                &children.0,
                &global_transform.matrix,
                &mut child_query,
                &children_query,
            );
        }
    }
}

/// Propagates transforms to children non-recursively to avoid borrow checker issues.
fn propagate_to_children(
    children: &[Entity],
    parent_matrix: &praxis_math::Mat4,
    child_query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
    _children_query: &Query<&Children>,
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

/// System that maintains parent-child relationships.
///
/// This system ensures that when a Parent component is added to an entity,
/// the parent entity's Children component is updated accordingly.
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
    new_parents: Query<(Entity, &Parent), Changed<Parent>>,
    mut parents_query: Query<&mut Children>,
) {
    for (child_entity, parent) in new_parents.iter() {
        // Add the child to the parent's children list
        if let Ok(mut children) = parents_query.get_mut(parent.0) {
            if !children.0.contains(&child_entity) {
                trace!(
                    "Adding child entity {:?} to parent {:?}",
                    child_entity, parent.0
                );
                children.push(child_entity);
            }
        } else {
            // Parent doesn't have a Children component yet, add one
            trace!("Creating Children component for parent {:?}", parent.0);
            commands
                .entity(parent.0)
                .insert(Children::with_children(vec![child_entity]));
        }
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
/// use praxis_ecs::{World, TransformBundle};
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
    fn test_transform_propagation() {
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
    fn test_transform_bundle() {
        let bundle = TransformBundle::from_xyz(1.0, 2.0, 3.0);
        assert_eq!(bundle.transform.translation, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(bundle.transform.rotation, Quat::IDENTITY);
        assert_eq!(bundle.transform.scale, Vec3::ONE);
    }
}
