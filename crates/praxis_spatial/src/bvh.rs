//! Bounding Volume Hierarchy (BVH) for efficient ray tracing and spatial queries.
//!
//! A BVH is a tree structure where each node contains a bounding volume that encloses
//! its children. BVHs are typically faster than octrees for ray tracing and nearest neighbor queries.

use crate::aabb::Aabb;
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;

/// Node in the BVH tree.
#[derive(Debug, Clone)]
pub enum BvhNode {
    /// Leaf node containing a single entity.
    Leaf {
        /// Entity contained in this leaf.
        entity: Entity,
        /// Bounding box of the entity.
        bounds: Aabb,
    },
    /// Internal node with two children.
    Internal {
        /// Bounding box that contains both children.
        bounds: Aabb,
        /// Left child node.
        left: Box<BvhNode>,
        /// Right child node.
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    /// Returns the bounding box of this node.
    pub fn bounds(&self) -> &Aabb {
        match self {
            Self::Leaf { bounds, .. } | Self::Internal { bounds, .. } => bounds,
        }
    }

    /// Queries all entities that intersect the given bounds.
    pub fn query(&self, query_bounds: &Aabb, results: &mut Vec<Entity>) {
        if !self.bounds().intersects(query_bounds) {
            return;
        }

        match self {
            Self::Leaf { entity, .. } => {
                results.push(*entity);
            }
            Self::Internal { left, right, .. } => {
                left.query(query_bounds, results);
                right.query(query_bounds, results);
            }
        }
    }

    /// Queries entities within a radius of a point.
    pub fn query_radius(&self, point: Vec3, radius: f32, results: &mut Vec<Entity>) {
        let radius_sq = radius * radius;
        
        if self.bounds().distance_squared(point) > radius_sq {
            return;
        }

        match self {
            Self::Leaf { entity, bounds } => {
                if bounds.distance_squared(point) <= radius_sq {
                    results.push(*entity);
                }
            }
            Self::Internal { left, right, .. } => {
                left.query_radius(point, radius, results);
                right.query_radius(point, radius, results);
            }
        }
    }

    /// Returns the number of entities in this subtree.
    pub fn entity_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Internal { left, right, .. } => left.entity_count() + right.entity_count(),
        }
    }
}

/// Bounding Volume Hierarchy for spatial partitioning.
///
/// BVHs are constructed bottom-up by recursively grouping entities based on spatial proximity.
pub struct Bvh {
    /// Root node of the BVH.
    root: Option<BvhNode>,
}

impl Bvh {
    /// Creates a new empty BVH.
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Builds a BVH from a list of entities and their bounding boxes.
    pub fn build(&mut self, entities: Vec<(Entity, Aabb)>) {
        if entities.is_empty() {
            self.root = None;
            return;
        }

        self.root = Some(Self::build_recursive(entities));
    }

    /// Recursively builds the BVH tree.
    fn build_recursive(mut entities: Vec<(Entity, Aabb)>) -> BvhNode {
        if entities.len() == 1 {
            let (entity, bounds) = entities.pop().unwrap();
            return BvhNode::Leaf { entity, bounds };
        }

        let mut combined_bounds = entities[0].1;
        for (_, bounds) in &entities[1..] {
            combined_bounds = combined_bounds.union(bounds);
        }

        let size = combined_bounds.size();
        
        let split_axis = if size.x >= size.y && size.x >= size.z {
            0
        } else if size.y >= size.z {
            1
        } else {
            2
        };

        entities.sort_by(|(_, a), (_, b)| {
            let a_center = a.center();
            let b_center = b.center();
            
            let a_val = match split_axis {
                0 => a_center.x,
                1 => a_center.y,
                _ => a_center.z,
            };
            
            let b_val = match split_axis {
                0 => b_center.x,
                1 => b_center.y,
                _ => b_center.z,
            };
            
            a_val.partial_cmp(&b_val).unwrap()
        });

        let mid = entities.len() / 2;
        let left_entities = entities[..mid].to_vec();
        let right_entities = entities[mid..].to_vec();

        let left = Box::new(Self::build_recursive(left_entities));
        let right = Box::new(Self::build_recursive(right_entities));

        let bounds = left.bounds().union(right.bounds());

        BvhNode::Internal { bounds, left, right }
    }

    /// Queries all entities that intersect the given bounds.
    pub fn query(&self, bounds: &Aabb) -> Vec<Entity> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.query(bounds, &mut results);
        }
        results
    }

    /// Queries all entities within the given radius of a point.
    pub fn query_radius(&self, point: Vec3, radius: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.query_radius(point, radius, &mut results);
        }
        results
    }

    /// Returns the total number of entities in the BVH.
    pub fn entity_count(&self) -> usize {
        self.root.as_ref().map_or(0, BvhNode::entity_count)
    }

    /// Clears the BVH.
    pub fn clear(&mut self) {
        self.root = None;
    }

    /// Returns true if the BVH is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the root bounding box, if any.
    pub fn bounds(&self) -> Option<&Aabb> {
        self.root.as_ref().map(BvhNode::bounds)
    }
}

impl Default for Bvh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvh_creation() {
        let bvh = Bvh::new();
        assert!(bvh.is_empty());
        assert_eq!(bvh.entity_count(), 0);
    }

    #[test]
    fn test_bvh_build() {
        let mut bvh = Bvh::new();
        let entities = vec![
            (Entity::from_raw(1), Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
            (Entity::from_raw(2), Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0))),
        ];
        
        bvh.build(entities);
        assert_eq!(bvh.entity_count(), 2);
    }

    #[test]
    fn test_bvh_query() {
        let mut bvh = Bvh::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entities = vec![
            (entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
            (entity2, Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0))),
        ];
        
        bvh.build(entities);

        let query_bounds = Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let results = bvh.query(&query_bounds);
        
        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_bvh_query_radius() {
        let mut bvh = Bvh::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entities = vec![
            (entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
            (entity2, Aabb::from_min_max(Vec3::new(20.0, 0.0, 0.0), Vec3::new(21.0, 1.0, 1.0))),
        ];
        
        bvh.build(entities);

        let results = bvh.query_radius(Vec3::ZERO, 10.0);
        
        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_bvh_clear() {
        let mut bvh = Bvh::new();
        let entities = vec![
            (Entity::from_raw(1), Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
        ];
        
        bvh.build(entities);
        assert!(!bvh.is_empty());
        
        bvh.clear();
        assert!(bvh.is_empty());
    }
}
