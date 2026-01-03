//! Scene graph traversal utilities.

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

    let entities_to_check: Vec<Entity> = root.map_or_else(
        || world.iter_entities().map(|e| e.id()).collect(),
        |root_entity| SceneGraphIterator::new(world, root_entity, TraversalOrder::DepthFirst).collect(),
    );

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

    #[test]
    fn test_scene_graph_iterator_breadth_first() {
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

        let grandchild1 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child1),
        ));

        world
            .entity_mut(root)
            .insert(Children(vec![child1, child2]));
        world
            .entity_mut(child1)
            .insert(Children(vec![grandchild1]));

        let visited: Vec<Entity> =
            SceneGraphIterator::new(&world, root, TraversalOrder::BreadthFirst).collect();

        assert_eq!(visited.len(), 4);
        assert_eq!(visited[0], root);
        assert_eq!(visited[1], child1);
        assert_eq!(visited[2], child2);
        assert_eq!(visited[3], grandchild1);
    }

    #[test]
    fn test_scene_graph_iterator_empty() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let visited: Vec<Entity> =
            SceneGraphIterator::new(&world, root, TraversalOrder::DepthFirst).collect();

        assert_eq!(visited.len(), 1);
        assert_eq!(visited[0], root);
    }

    #[test]
    fn test_scene_graph_iterator_depth_first_complex() {
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

        let grandchild1 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child1),
        ));

        let grandchild2 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child2),
        ));

        world
            .entity_mut(root)
            .insert(Children(vec![child1, child2]));
        world
            .entity_mut(child1)
            .insert(Children(vec![grandchild1]));
        world
            .entity_mut(child2)
            .insert(Children(vec![grandchild2]));

        let visited: Vec<Entity> =
            SceneGraphIterator::new(&world, root, TraversalOrder::DepthFirst).collect();

        assert_eq!(visited.len(), 5);
        assert_eq!(visited[0], root);
        let child1_idx = visited.iter().position(|&e| e == child1).unwrap();
        let grandchild1_idx = visited.iter().position(|&e| e == grandchild1).unwrap();
        assert!(grandchild1_idx > child1_idx);
    }

    #[test]
    fn test_get_all_children_empty() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let descendants = get_all_children(&world, root);
        assert_eq!(descendants.len(), 0);
    }

    #[test]
    fn test_get_all_children_deep_hierarchy() {
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

        let great_grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(grandchild),
        ));

        world.entity_mut(root).insert(Children(vec![child]));
        world.entity_mut(child).insert(Children(vec![grandchild]));
        world
            .entity_mut(grandchild)
            .insert(Children(vec![great_grandchild]));

        let descendants = get_all_children(&world, root);
        assert_eq!(descendants.len(), 3);
        assert!(descendants.contains(&child));
        assert!(descendants.contains(&grandchild));
        assert!(descendants.contains(&great_grandchild));
    }

    #[test]
    fn test_get_all_children_multiple_branches() {
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

        let grandchild1 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child1),
        ));

        let grandchild2 = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child2),
        ));

        world
            .entity_mut(root)
            .insert(Children(vec![child1, child2]));
        world
            .entity_mut(child1)
            .insert(Children(vec![grandchild1]));
        world
            .entity_mut(child2)
            .insert(Children(vec![grandchild2]));

        let descendants = get_all_children(&world, root);
        assert_eq!(descendants.len(), 4);
        assert!(descendants.contains(&child1));
        assert!(descendants.contains(&child2));
        assert!(descendants.contains(&grandchild1));
        assert!(descendants.contains(&grandchild2));
    }

    #[test]
    fn test_get_parent_chain_empty() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let chain = get_parent_chain(&world, root);
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_get_parent_chain_deep() {
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
        let great_grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(grandchild),
        ));

        let chain = get_parent_chain(&world, great_grandchild);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], grandchild);
        assert_eq!(chain[1], child);
        assert_eq!(chain[2], root);
    }

    #[test]
    fn test_get_root_entity_deep_hierarchy() {
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

        assert_eq!(get_root_entity(&world, grandchild), root);
        assert_eq!(get_root_entity(&world, child), root);
        assert_eq!(get_root_entity(&world, root), root);
    }

    #[test]
    fn test_is_ancestor_of_false() {
        let mut world = World::new();

        let entity1 = world.spawn((Transform::default(), GlobalTransform::default()));
        let entity2 = world.spawn((Transform::default(), GlobalTransform::default()));

        assert!(!is_ancestor_of(&world, entity1, entity2));
        assert!(!is_ancestor_of(&world, entity2, entity1));
    }

    #[test]
    fn test_is_ancestor_of_direct_parent() {
        let mut world = World::new();

        let parent = world.spawn((Transform::default(), GlobalTransform::default()));
        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(parent),
        ));

        assert!(is_ancestor_of(&world, parent, child));
        assert!(!is_ancestor_of(&world, child, parent));
    }

    #[test]
    fn test_is_ancestor_of_self() {
        let mut world = World::new();
        let entity = world.spawn((Transform::default(), GlobalTransform::default()));

        assert!(!is_ancestor_of(&world, entity, entity));
    }

    #[test]
    fn test_get_entity_depth_complex() {
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
        let grandchild = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(child1),
        ));

        assert_eq!(get_entity_depth(&world, root), 0);
        assert_eq!(get_entity_depth(&world, child1), 1);
        assert_eq!(get_entity_depth(&world, child2), 1);
        assert_eq!(get_entity_depth(&world, grandchild), 2);
    }

    #[test]
    fn test_get_root_entities_single() {
        let mut world = World::new();
        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let roots = get_root_entities(&world);
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&root));
    }

    #[test]
    fn test_get_root_entities_multiple() {
        let mut world = World::new();

        let root1 = world.spawn((Transform::default(), GlobalTransform::default()));
        let root2 = world.spawn((Transform::default(), GlobalTransform::default()));

        let child = world.spawn((
            Transform::default(),
            GlobalTransform::default(),
            Parent(root1),
        ));

        world.entity_mut(root1).insert(Children(vec![child]));

        let roots = get_root_entities(&world);
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&root1));
        assert!(roots.contains(&root2));
        assert!(!roots.contains(&child));
    }

    #[test]
    fn test_get_root_entities_no_transform() {
        let mut world = World::new();

        let _entity_without_transform = world.spawn(());
        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let roots = get_root_entities(&world);
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&root));
    }

    #[test]
    fn test_find_entities_by_name_multiple() {
        let mut world = World::new();

        let entity1 = world.spawn((
            Name::new("Player"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let entity2 = world.spawn((
            Name::new("Player"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let _entity3 = world.spawn((
            Name::new("Enemy"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let found = find_entities_by_name(&world, "Player", None);
        assert_eq!(found.len(), 2);
        assert!(found.contains(&entity1));
        assert!(found.contains(&entity2));
    }

    #[test]
    fn test_find_entities_by_name_with_root() {
        let mut world = World::new();

        let root = world.spawn((
            Name::new("Root"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let child1 = world.spawn((
            Name::new("Target"),
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        let child2 = world.spawn((
            Name::new("Other"),
            Transform::default(),
            GlobalTransform::default(),
            Parent(root),
        ));

        let _other_root = world.spawn((
            Name::new("Target"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        world
            .entity_mut(root)
            .insert(Children(vec![child1, child2]));

        let found = find_entities_by_name(&world, "Target", Some(root));
        assert_eq!(found.len(), 1);
        assert!(found.contains(&child1));
    }

    #[test]
    fn test_find_entity_by_name_first() {
        let mut world = World::new();

        let entity1 = world.spawn((
            Name::new("Duplicate"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let _entity2 = world.spawn((
            Name::new("Duplicate"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        let found = find_entity_by_name(&world, "Duplicate", None);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), entity1);
    }

    #[test]
    fn test_traversal_order_enum() {
        assert_eq!(TraversalOrder::DepthFirst, TraversalOrder::DepthFirst);
        assert_ne!(TraversalOrder::DepthFirst, TraversalOrder::BreadthFirst);

        let order = TraversalOrder::BreadthFirst;
        let order_clone = order;
        assert_eq!(order, order_clone);
    }

    #[test]
    fn test_scene_graph_iterator_large_tree() {
        let mut world = World::new();

        let root = world.spawn((Transform::default(), GlobalTransform::default()));

        let mut children = Vec::new();
        for _ in 0..5 {
            let child = world.spawn((
                Transform::default(),
                GlobalTransform::default(),
                Parent(root),
            ));
            children.push(child);
        }

        world.entity_mut(root).insert(Children(children.clone()));

        let visited: Vec<Entity> =
            SceneGraphIterator::new(&world, root, TraversalOrder::BreadthFirst).collect();

        assert_eq!(visited.len(), 6);
        assert_eq!(visited[0], root);
        for child in children {
            assert!(visited.contains(&child));
        }
    }
}
