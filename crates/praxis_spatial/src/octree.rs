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
                if i & 1 != 0 { half_size.x } else { -half_size.x },
                if i & 2 != 0 { half_size.y } else { -half_size.y },
                if i & 4 != 0 { half_size.z } else { -half_size.z },
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

        if self.children.is_none() && self.entities.len() < max_entities && self.depth < MAX_DEPTH
        {
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

        let radius_sq = radius * radius;
        for &entity in &self.entities {
            if self.bounds.center().distance_squared(point) <= radius_sq {
                results.push(entity);
            }
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
        let inserted = self.root.insert(entity, &bounds, self.max_entities_per_node);
        if inserted {
            self.entity_bounds.insert(entity, bounds);
        }
        inserted
    }

    /// Removes an entity from the octree.
    pub fn remove(&mut self, entity: Entity) {
        self.entity_bounds.remove(&entity);
    }

    /// Updates an entity's position in the octree.
    pub fn update(&mut self, entity: Entity, new_bounds: Aabb) {
        self.remove(entity);
        self.insert(entity, new_bounds);
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
        octree.insert(entity2, Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0)));

        let query_bounds = Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
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
        octree.insert(entity2, Aabb::from_min_max(Vec3::new(20.0, 0.0, 0.0), Vec3::new(21.0, 1.0, 1.0)));

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
        
        octree.remove(entity);
        let results = octree.query(&bounds);
        assert!(results.is_empty() || !results.contains(&entity));
    }
}
