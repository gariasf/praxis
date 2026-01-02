//! Scene graph traversal utilities.

#![allow(clippy::option_if_let_else)]

use praxis_ecs::{Children, Entity, Parent, World};
use std::collections::VecDeque;

/// Traversal order for scene graph traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalOrder {
    /// Depth-first traversal (visit parent before children).
    DepthFirst,
    /// Breadth-first traversal (visit all siblings before descendants).
    BreadthFirst,
}

/// Iterator for traversing a scene graph from a root entity.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{SceneGraphIterator, TraversalOrder};
/// use praxis_ecs::{World, Entity};
///
/// let world = World::new();
/// let root_entity = Entity::from_raw(0);
///
/// for entity in SceneGraphIterator::new(&world, root_entity, TraversalOrder::DepthFirst) {
///     println!("Visiting entity: {:?}", entity);
/// }
/// ```
pub struct SceneGraphIterator<'w> {
    world: &'w World,
    to_visit: VecDeque<Entity>,
    order: TraversalOrder,
}

impl<'w> SceneGraphIterator<'w> {
    /// Creates a new scene graph iterator starting from the given root entity.
    pub fn new(world: &'w World, root: Entity, order: TraversalOrder) -> Self {
        let mut to_visit = VecDeque::new();
        to_visit.push_back(root);
        Self {
            world,
            to_visit,
            order,
        }
    }
}

impl Iterator for SceneGraphIterator<'_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.to_visit.pop_front()?;

        if let Some(children) = self.world.get::<Children>(entity) {
            match self.order {
                TraversalOrder::DepthFirst => {
                    for child in children.0.iter().rev() {
                        self.to_visit.push_front(*child);
                    }
                }
                TraversalOrder::BreadthFirst => {
                    for child in &children.0 {
                        self.to_visit.push_back(*child);
                    }
                }
            }
        }

        Some(entity)
    }
}

/// Gets all root entities (entities without a parent).
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::get_root_entities;
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// let roots = get_root_entities(&world);
/// println!("Found {} root entities", roots.len());
/// ```
pub fn get_root_entities(world: &World) -> Vec<Entity> {
    let mut all_entities_with_transform = Vec::new();

    for entity in world.iter_entities() {
        if world.get::<praxis_ecs::Transform>(entity.id()).is_some() {
            all_entities_with_transform.push(entity.id());
        }
    }

    all_entities_with_transform
        .into_iter()
        .filter(|entity| world.get::<Parent>(*entity).is_none())
        .collect()
}

/// Gets all children of an entity, recursively.
///
/// Returns all descendants in depth-first order.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::get_all_children;
/// use praxis_ecs::{World, Entity};
///
/// let world = World::new();
/// let parent = Entity::from_raw(0);
/// let descendants = get_all_children(&world, parent);
/// println!("Entity has {} descendants", descendants.len());
/// ```
pub fn get_all_children(world: &World, entity: Entity) -> Vec<Entity> {
    let mut result = Vec::new();
    collect_children_recursive(world, entity, &mut result);
    result
}

fn collect_children_recursive(world: &World, entity: Entity, result: &mut Vec<Entity>) {
    if let Some(children) = world.get::<Children>(entity) {
        for &child in &children.0 {
            result.push(child);
            collect_children_recursive(world, child, result);
        }
    }
}

/// Gets the parent chain for an entity.
///
/// Returns a vector of entities from the immediate parent up to the root,
/// in order from nearest to farthest.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::get_parent_chain;
/// use praxis_ecs::{World, Entity};
///
/// let world = World::new();
/// let entity = Entity::from_raw(0);
/// let parents = get_parent_chain(&world, entity);
/// println!("Entity has {} ancestors", parents.len());
/// ```
pub fn get_parent_chain(world: &World, entity: Entity) -> Vec<Entity> {
    let mut chain = Vec::new();
    let mut current = entity;

    while let Some(parent) = world.get::<Parent>(current) {
        chain.push(parent.0);
        current = parent.0;
    }

    chain
}

/// Gets the root entity for a given entity.
///
/// Traverses up the parent chain until reaching an entity with no parent.
/// Returns the entity itself if it has no parent.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::get_root_entity;
/// use praxis_ecs::{World, Entity};
///
/// let world = World::new();
/// let entity = Entity::from_raw(0);
/// let root = get_root_entity(&world, entity);
/// println!("Root entity: {:?}", root);
/// ```
pub fn get_root_entity(world: &World, entity: Entity) -> Entity {
    let mut current = entity;

    while let Some(parent) = world.get::<Parent>(current) {
        current = parent.0;
    }

    current
}

/// Checks if an entity is an ancestor of another entity.
///
/// Returns `true` if `potential_ancestor` is in the parent chain of `entity`.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::is_ancestor_of;
/// use praxis_ecs::{World, Entity};
///
/// let world = World::new();
/// let grandparent = Entity::from_raw(0);
/// let grandchild = Entity::from_raw(2);
///
/// if is_ancestor_of(&world, grandparent, grandchild) {
///     println!("Is ancestor!");
/// }
/// ```
pub fn is_ancestor_of(world: &World, potential_ancestor: Entity, entity: Entity) -> bool {
    let mut current = entity;

    while let Some(parent) = world.get::<Parent>(current) {
        if parent.0 == potential_ancestor {
            return true;
        }
        current = parent.0;
    }

    false
}

/// Gets the depth of an entity in the scene hierarchy.
///
/// Returns 0 for root entities, 1 for their immediate children, etc.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::get_entity_depth;
/// use praxis_ecs::{World, Entity};
///
/// let world = World::new();
/// let entity = Entity::from_raw(0);
/// let depth = get_entity_depth(&world, entity);
/// println!("Entity depth: {}", depth);
/// ```
pub fn get_entity_depth(world: &World, entity: Entity) -> usize {
    let mut depth = 0;
    let mut current = entity;

    while let Some(parent) = world.get::<Parent>(current) {
        depth += 1;
        current = parent.0;
    }

    depth
}

/// Finds entities by name in the scene graph.
///
/// Returns all entities with the given name, starting from the root or a specified parent.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::find_entities_by_name;
/// use praxis_ecs::World;
///
/// let world = World::new();
/// let entities = find_entities_by_name(&world, "Player", None);
/// println!("Found {} entities named 'Player'", entities.len());
/// ```
pub fn find_entities_by_name(world: &World, name: &str, root: Option<Entity>) -> Vec<Entity> {
    let mut results = Vec::new();

    let entities_to_check: Vec<Entity> = if let Some(root_entity) = root {
        SceneGraphIterator::new(world, root_entity, TraversalOrder::DepthFirst).collect()
    } else {
        world.iter_entities().map(|e| e.id()).collect()
    };

    for entity in entities_to_check {
        if let Some(entity_name) = world.get::<praxis_ecs::Name>(entity) {
            if entity_name.as_str() == name {
                results.push(entity);
            }
        }
    }

    results
}

/// Finds the first entity by name in the scene graph.
///
/// Returns the first entity with the given name, or `None` if not found.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::find_entity_by_name;
/// use praxis_ecs::World;
///
/// let world = World::new();
/// if let Some(player) = find_entity_by_name(&world, "Player", None) {
///     println!("Found player entity: {:?}", player);
/// }
/// ```
pub fn find_entity_by_name(world: &World, name: &str, root: Option<Entity>) -> Option<Entity> {
    find_entities_by_name(world, name, root).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_ecs::{GlobalTransform, Name, Transform};

    #[test]
    fn test_scene_graph_iterator_depth_first() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let child1 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        let child2 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        world
            .entity_mut(root)
            .insert(Children(vec![child1, child2]));

        let visited: Vec<Entity> =
            SceneGraphIterator::new(&world, root, TraversalOrder::DepthFirst).collect();

        assert_eq!(visited.len(), 3);
        assert_eq!(visited[0], root);
    }

    #[test]
    fn test_get_all_children() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let child1 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        let grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child1),
        ));

        world.entity_mut(root).insert(Children(vec![child1]));
        world.entity_mut(child1).insert(Children(vec![grandchild]));

        let descendants = get_all_children(&world, root);

        assert_eq!(descendants.len(), 2);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&grandchild));
    }

    #[test]
    fn test_get_parent_chain() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));
        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));
        let grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child),
        ));

        let chain = get_parent_chain(&world, grandchild);

        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], child);
        assert_eq!(chain[1], root);
    }

    #[test]
    fn test_get_root_entity() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));
        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        assert_eq!(get_root_entity(&world, child), root);
        assert_eq!(get_root_entity(&world, root), root);
    }

    #[test]
    fn test_is_ancestor_of() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));
        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));
        let grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child),
        ));

        assert!(is_ancestor_of(&world, root, grandchild));
        assert!(is_ancestor_of(&world, child, grandchild));
        assert!(!is_ancestor_of(&world, grandchild, root));
    }

    #[test]
    fn test_get_entity_depth() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));
        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));
        let grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child),
        ));

        assert_eq!(get_entity_depth(&world, root), 0);
        assert_eq!(get_entity_depth(&world, child), 1);
        assert_eq!(get_entity_depth(&world, grandchild), 2);
    }

    #[test]
    fn test_find_entity_by_name() {
        let mut world = World::new();

        let _entity1 = world.spawn((
            Name::new("Player"),
            Transform::default(),
            GlobalTransform::default(),
        ));
        let entity2 = world.spawn((
            Name::new("Enemy"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let found = find_entity_by_name(&world, "Enemy", None);
        assert_eq!(found, Some(entity2));

        let not_found = find_entity_by_name(&world, "Boss", None);
        assert_eq!(not_found, None);
    }
}
