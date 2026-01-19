//! Frustum culling query helpers.
//!
//! This module provides helper functions for querying visible entities
//! after frustum culling has been performed.

use crate::{Entity, Query, Visible, With};

/// Returns the count of visible entities.
///
/// This is useful for performance metrics and debugging.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Query, Entity, With, Visible};
/// use praxis_ecs::culling::count_visible_entities;
///
/// fn debug_system(visible: Query<Entity, With<Visible>>) {
///     let count = count_visible_entities(&visible);
///     println!("Visible entities: {}", count);
/// }
/// ```
pub fn count_visible_entities(visible_entities: &Query<Entity, With<Visible>>) -> usize {
    visible_entities.iter().count()
}

/// Returns true if the given entity is marked as visible.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Entity, Visible};
/// use praxis_ecs::culling::is_entity_visible;
///
/// fn check_visibility(world: &World, entity: Entity) -> bool {
///     is_entity_visible(world.inner(), entity)
/// }
/// ```
pub fn is_entity_visible(world: &bevy_ecs::world::World, entity: Entity) -> bool {
    world.get::<Visible>(entity).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;

    #[test]
    fn test_count_visible_entities_empty() {
        let mut world = World::new();
        let mut query = world.inner_mut().query_filtered::<Entity, With<Visible>>();
        let count = query.iter(world.inner()).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_visible_entities() {
        let mut world = World::new();

        // Spawn some entities with Visible component
        world.spawn(Visible);
        world.spawn(Visible);
        world.spawn(Visible);

        // Spawn some entities without Visible component
        world.spawn(());
        world.spawn(());

        let query = world.inner_mut().query_filtered::<Entity, With<Visible>>();
        let count = count_visible_entities(&query);

        assert_eq!(count, 3);
    }

    #[test]
    fn test_is_entity_visible_true() {
        let mut world = World::new();
        let entity = world.spawn(Visible);

        assert!(is_entity_visible(world.inner(), entity));
    }

    #[test]
    fn test_is_entity_visible_false() {
        let mut world = World::new();
        let entity = world.spawn(());

        assert!(!is_entity_visible(world.inner(), entity));
    }

    #[test]
    fn test_is_entity_visible_nonexistent() {
        let world = World::new();
        let fake_entity = Entity::from_raw(999999);

        assert!(!is_entity_visible(world.inner(), fake_entity));
    }

    #[test]
    fn test_count_visible_after_removal() {
        let mut world = World::new();

        let e1 = world.spawn(Visible);
        let e2 = world.spawn(Visible);
        let e3 = world.spawn(Visible);

        // Remove Visible from one entity
        world.remove_component::<Visible>(e2);

        let query = world.inner_mut().query_filtered::<Entity, With<Visible>>();
        let count = count_visible_entities(&query);

        assert_eq!(count, 2);
    }

    #[test]
    fn test_visible_entities_collection() {
        let mut world = World::new();

        let e1 = world.spawn(Visible);
        let e2 = world.spawn(Visible);
        let _e3 = world.spawn(()); // Not visible

        let mut query = world.inner_mut().query_filtered::<Entity, With<Visible>>();
        let visible: Vec<Entity> = query.iter(world.inner()).collect();

        assert_eq!(visible.len(), 2);
        assert!(visible.contains(&e1));
        assert!(visible.contains(&e2));
    }
}
