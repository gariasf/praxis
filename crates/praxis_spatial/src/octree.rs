//! Octree spatial partitioning structure.
//!
//! An octree recursively subdivides 3D space into eight octants, providing efficient
//! spatial queries for large numbers of objects.

use crate::aabb::Aabb;
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;
use std::collections::HashMap;

/// Maximum depth of the octree to prevent infinite recursion.
const MAX_DEPTH: u32 = 10;

/// Node in the octree structure.
#[derive(Debug)]
pub struct OctreeNode {
    /// Bounding box of this node.
    pub bounds: Aabb,
    /// Current depth in the tree (0 = root).
    depth: u32,
    /// Entities contained in this node.
    entities: Vec<Entity>,
    /// Child nodes (if subdivided).
    children: Option<Box<[OctreeNode; 8]>>,
}

impl OctreeNode {
    /// Creates a new octree node.
    fn new(bounds: Aabb, depth: u32) -> Self {
        Self {
            bounds,
            depth,
            entities: Vec::new(),
            children: None,
        }
    }

    /// Subdivides this node into eight children.
    fn subdivide(&mut self) {
        let center = self.bounds.center();
        let half_size = self.bounds.half_extents();

        let mut children = Vec::with_capacity(8);

        for i in 0..8 {
            let offset = Vec3::new(
                if i & 1 != 0 {
                    half_size.x
                } else {
                    -half_size.x
                },
                if i & 2 != 0 {
                    half_size.y
                } else {
                    -half_size.y
                },
                if i & 4 != 0 {
                    half_size.z
                } else {
                    -half_size.z
                },
            ) * 0.5;

            let child_center = center + offset;
            let child_half_size = half_size * 0.5;
            let child_bounds = Aabb::from_center_half_extents(child_center, child_half_size);

            children.push(Self::new(child_bounds, self.depth + 1));
        }

        self.children = Some(Box::new(children.try_into().unwrap()));
    }

    /// Determines which child octant contains the given bounds.
    fn get_octant_index(&self, bounds: &Aabb) -> Option<usize> {
        let center = self.bounds.center();
        let obj_center = bounds.center();

        if !self.bounds.contains(bounds) {
            return None;
        }

        let mut index = 0;
        if obj_center.x >= center.x {
            index |= 1;
        }
        if obj_center.y >= center.y {
            index |= 2;
        }
        if obj_center.z >= center.z {
            index |= 4;
        }

        Some(index)
    }

    /// Inserts an entity into this node or its children.
    fn insert(&mut self, entity: Entity, bounds: &Aabb, max_entities: usize) -> bool {
        if !self.bounds.intersects(bounds) {
            return false;
        }

        if self.children.is_none() && self.entities.len() < max_entities && self.depth < MAX_DEPTH {
            self.entities.push(entity);
            return true;
        }

        if self.children.is_none() {
            self.subdivide();
        }

        let octant = self.get_octant_index(bounds);
        if let Some(children) = &mut self.children {
            if let Some(octant_idx) = octant {
                if children[octant_idx].insert(entity, bounds, max_entities) {
                    return true;
                }
            }
        }

        self.entities.push(entity);
        true
    }

    /// Queries all entities that intersect the given bounds.
    fn query(&self, bounds: &Aabb, results: &mut Vec<Entity>) {
        if !self.bounds.intersects(bounds) {
            return;
        }

        results.extend(&self.entities);

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query(bounds, results);
            }
        }
    }

    /// Queries all entities within range of a point.
    fn query_radius(&self, point: Vec3, radius: f32, results: &mut Vec<Entity>) {
        let sphere_bounds = Aabb::from_center_half_extents(point, Vec3::splat(radius));

        if !self.bounds.intersects(&sphere_bounds) {
            return;
        }

        for &entity in &self.entities {
            results.push(entity);
        }

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_radius(point, radius, results);
            }
        }
    }

    /// Returns the number of entities in this node and its children.
    fn entity_count(&self) -> usize {
        let mut count = self.entities.len();
        if let Some(children) = &self.children {
            for child in children.iter() {
                count += child.entity_count();
            }
        }
        count
    }

    /// Clears all entities from this node and its children.
    fn clear(&mut self) {
        self.entities.clear();
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                child.clear();
            }
        }
    }

    /// Queries all entities that intersect with a ray.
    fn query_ray(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        results: &mut Vec<Entity>,
    ) {
        if !self.bounds.intersects_ray(origin, direction, max_distance) {
            return;
        }

        results.extend(&self.entities);

        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_ray(origin, direction, max_distance, results);
            }
        }
    }

    /// Removes a specific entity from this node.
    fn remove(&mut self, entity: Entity) -> bool {
        if let Some(pos) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(pos);
            return true;
        }

        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.remove(entity) {
                    return true;
                }
            }
        }

        false
    }
}

/// Octree spatial partitioning structure.
///
/// The octree divides 3D space hierarchically to enable fast spatial queries.
pub struct Octree {
    /// Root node of the octree.
    root: OctreeNode,
    /// Maximum entities per node before subdivision.
    max_entities_per_node: usize,
    /// Map from entity to its bounding box for updates.
    entity_bounds: HashMap<Entity, Aabb>,
}

impl Octree {
    /// Creates a new octree with the given bounds and maximum entities per node.
    pub fn new(center: Vec3, size: f32, max_entities_per_node: usize) -> Self {
        let half_size = size * 0.5;
        let bounds = Aabb::from_center_half_extents(center, Vec3::splat(half_size));

        Self {
            root: OctreeNode::new(bounds, 0),
            max_entities_per_node,
            entity_bounds: HashMap::new(),
        }
    }

    /// Inserts an entity with its bounding box into the octree.
    pub fn insert(&mut self, entity: Entity, bounds: Aabb) -> bool {
        let inserted = self
            .root
            .insert(entity, &bounds, self.max_entities_per_node);
        if inserted {
            self.entity_bounds.insert(entity, bounds);
        }
        inserted
    }

    /// Removes an entity from the octree.
    pub fn remove(&mut self, entity: Entity) -> bool {
        if self.entity_bounds.remove(&entity).is_some() {
            self.root.remove(entity);
            true
        } else {
            false
        }
    }

    /// Updates an entity's position in the octree.
    pub fn update(&mut self, entity: Entity, new_bounds: Aabb) -> bool {
        self.remove(entity);
        self.insert(entity, new_bounds)
    }

    /// Queries all entities that intersect the given bounds.
    pub fn query(&self, bounds: &Aabb) -> Vec<Entity> {
        let mut results = Vec::new();
        self.root.query(bounds, &mut results);
        results
    }

    /// Queries all entities within the given radius of a point.
    pub fn query_radius(&self, point: Vec3, radius: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        self.root.query_radius(point, radius, &mut results);

        let radius_sq = radius * radius;
        results.retain(|&entity| {
            self.entity_bounds
                .get(&entity)
                .is_some_and(|bounds| bounds.center().distance_squared(point) <= radius_sq)
        });

        results
    }

    /// Returns the total number of entities in the octree.
    pub fn entity_count(&self) -> usize {
        self.root.entity_count()
    }

    /// Clears all entities from the octree.
    pub fn clear(&mut self) {
        self.root.clear();
        self.entity_bounds.clear();
    }

    /// Rebuilds the octree from the current entity bounds.
    pub fn rebuild(&mut self) {
        let entities: Vec<_> = self.entity_bounds.clone().into_iter().collect();
        self.clear();
        for (entity, bounds) in entities {
            self.insert(entity, bounds);
        }
    }

    /// Returns the bounds of the octree.
    pub fn bounds(&self) -> &Aabb {
        &self.root.bounds
    }

    /// Queries all entities that intersect with a ray.
    pub fn query_ray(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        self.root
            .query_ray(origin, direction, max_distance, &mut results);
        results
    }

    /// Queries all entities that intersect with a ray and returns them sorted by distance.
    pub fn query_ray_sorted(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Vec<(Entity, f32)> {
        let entities = self.query_ray(origin, direction, max_distance);
        let mut results = Vec::new();

        for entity in entities {
            if let Some(bounds) = self.entity_bounds.get(&entity) {
                if let Some(distance) =
                    bounds.ray_intersection_distance(origin, direction, max_distance)
                {
                    results.push((entity, distance));
                }
            }
        }

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Returns true if the octree contains the entity.
    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_bounds.contains_key(&entity)
    }

    /// Gets the bounds of an entity if it exists in the octree.
    pub fn get_bounds(&self, entity: Entity) -> Option<&Aabb> {
        self.entity_bounds.get(&entity)
    }

    /// Checks if the octree needs rebalancing based on entity distribution.
    pub fn needs_rebalancing(&self) -> bool {
        let total = self.entity_count();
        if total == 0 {
            return false;
        }

        let imbalance_ratio = self.calculate_imbalance_ratio();
        imbalance_ratio > 2.0
    }

    /// Calculates the imbalance ratio of the octree.
    fn calculate_imbalance_ratio(&self) -> f32 {
        let total = self.entity_count();
        if total == 0 {
            return 0.0;
        }

        let ideal_per_node = self.max_entities_per_node as f32;
        let actual_nodes = (total as f32 / ideal_per_node).max(1.0);

        actual_nodes / total.max(1) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octree_creation() {
        let octree = Octree::new(Vec3::ZERO, 100.0, 4);
        assert_eq!(octree.entity_count(), 0);
    }

    #[test]
    fn test_octree_insert() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        assert!(octree.insert(entity, bounds));
        assert_eq!(octree.entity_count(), 1);
    }

    #[test]
    fn test_octree_query() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        octree.insert(entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE));
        octree.insert(
            entity2,
            Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0)),
        );

        let query_bounds =
            Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let results = octree.query(&query_bounds);

        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_octree_query_radius() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        octree.insert(entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE));
        octree.insert(
            entity2,
            Aabb::from_min_max(Vec3::new(20.0, 0.0, 0.0), Vec3::new(21.0, 1.0, 1.0)),
        );

        let results = octree.query_radius(Vec3::ZERO, 10.0);

        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_octree_remove() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        octree.insert(entity, bounds);
        assert_eq!(octree.entity_count(), 1);

        assert!(octree.remove(entity));
        assert!(!octree.contains(entity));
    }

    #[test]
    fn test_octree_ray_query() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);

        for i in 0..5 {
            let entity = Entity::from_raw(i);
            let z = (i as f32 * 10.0) + 5.0;
            let bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, z), Vec3::splat(2.0));
            octree.insert(entity, bounds);
        }

        let origin = Vec3::ZERO;
        let direction = Vec3::Z;
        let results = octree.query_ray(origin, direction, 100.0);

        assert!(!results.is_empty());
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_octree_ray_sorted() {
        let mut octree = Octree::new(Vec3::ZERO, 200.0, 4);

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        octree.insert(
            entity1,
            Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 10.0), Vec3::splat(1.0)),
        );
        octree.insert(
            entity2,
            Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 30.0), Vec3::splat(1.0)),
        );
        octree.insert(
            entity3,
            Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 20.0), Vec3::splat(1.0)),
        );

        let results = octree.query_ray_sorted(Vec3::ZERO, Vec3::Z, 100.0);

        assert_eq!(results.len(), 3);

        for i in 0..results.len() - 1 {
            assert!(results[i].1 <= results[i + 1].1);
        }
    }

    #[test]
    fn test_octree_get_bounds() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        octree.insert(entity, bounds);

        let retrieved = octree.get_bounds(entity);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().min, Vec3::ZERO);
        assert_eq!(retrieved.unwrap().max, Vec3::ONE);
    }

    #[test]
    fn test_octree_needs_rebalancing() {
        let mut octree = Octree::new(Vec3::ZERO, 100.0, 2);

        for i in 0..20 {
            let entity = Entity::from_raw(i);
            let x = (i as f32 * 3.0) - 30.0;
            let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(1.0));
            octree.insert(entity, bounds);
        }

        let _ = octree.needs_rebalancing();
    }
}
