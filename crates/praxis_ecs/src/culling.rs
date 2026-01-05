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

    #[test]
    fn test_count_visible_compiles() {
        // This test ensures the function signature compiles
        // Full testing would require setting up a World with entities
    }
}
