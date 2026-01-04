//! Spatial manager for dynamic scene organization and queries.
//!
//! Provides a unified interface for managing spatial partitioning structures
//! with automatic updates and rebalancing.

use crate::{Aabb, Bvh, Octree};
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;
use std::collections::{HashMap, HashSet};

/// Configuration for the spatial manager.
#[derive(Debug, Clone)]
pub struct SpatialConfig {
    /// Center point of the spatial structure.
    pub center: Vec3,
    /// Size of the spatial structure.
    pub size: f32,
    /// Maximum entities per octree node before subdivision.
    pub max_entities_per_node: usize,
    /// Movement threshold before triggering an update.
    pub movement_threshold: f32,
    /// Number of updates before considering rebalancing.
    pub rebalance_interval: usize,
}

impl Default for SpatialConfig {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            size: 1000.0,
            max_entities_per_node: 8,
            movement_threshold: 0.1,
            rebalance_interval: 100,
        }
    }
}

/// Type of spatial partitioning structure to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialStructureType {
    /// Octree partitioning (better for uniform distribution).
    Octree,
    /// BVH partitioning (better for ray tracing and dynamic scenes).
    Bvh,
}

/// Unified spatial manager that handles both octree and BVH structures.
///
/// Automatically tracks entity movement and triggers updates/rebalancing as needed.
pub struct SpatialManager {
    /// Current structure type.
    structure_type: SpatialStructureType,
    /// Octree instance.
    octree: Octree,
    /// BVH instance.
    bvh: Bvh,
    /// Configuration.
    config: SpatialConfig,
    /// Entities that have moved and need updating.
    dirty_entities: HashSet<Entity>,
    /// Update counter for rebalancing.
    update_counter: usize,
    /// Previous positions for movement tracking.
    previous_positions: HashMap<Entity, Vec3>,
}

impl SpatialManager {
    /// Creates a new spatial manager with the given configuration and structure type.
    pub fn new(config: SpatialConfig, structure_type: SpatialStructureType) -> Self {
        let octree = Octree::new(config.center, config.size, config.max_entities_per_node);
        let bvh = Bvh::new();

        Self {
            structure_type,
            octree,
            bvh,
            config,
            dirty_entities: HashSet::new(),
            update_counter: 0,
            previous_positions: HashMap::new(),
        }
    }

    /// Creates a new octree-based spatial manager.
    pub fn new_octree(config: SpatialConfig) -> Self {
        Self::new(config, SpatialStructureType::Octree)
    }

    /// Creates a new BVH-based spatial manager.
    pub fn new_bvh(config: SpatialConfig) -> Self {
        Self::new(config, SpatialStructureType::Bvh)
    }

    /// Creates a spatial manager with default configuration.
    pub fn default_octree() -> Self {
        Self::new_octree(SpatialConfig::default())
    }

    /// Creates a BVH spatial manager with default configuration.
    pub fn default_bvh() -> Self {
        Self::new_bvh(SpatialConfig::default())
    }

    /// Inserts an entity with its bounding box into the spatial structure.
    pub fn insert(&mut self, entity: Entity, bounds: Aabb) -> bool {
        let center = bounds.center();
        self.previous_positions.insert(entity, center);

        match self.structure_type {
            SpatialStructureType::Octree => self.octree.insert(entity, bounds),
            SpatialStructureType::Bvh => {
                self.bvh.insert(entity, bounds);
                true
            }
        }
    }

    /// Removes an entity from the spatial structure.
    pub fn remove(&mut self, entity: Entity) -> bool {
        self.previous_positions.remove(&entity);
        self.dirty_entities.remove(&entity);

        match self.structure_type {
            SpatialStructureType::Octree => self.octree.remove(entity),
            SpatialStructureType::Bvh => self.bvh.remove(entity),
        }
    }

    /// Updates an entity's position in the spatial structure.
    ///
    /// Only triggers an update if the entity has moved beyond the movement threshold.
    pub fn update(&mut self, entity: Entity, new_bounds: Aabb) -> bool {
        let new_center = new_bounds.center();

        let should_update = if let Some(&prev_pos) = self.previous_positions.get(&entity) {
            prev_pos.distance(new_center) > self.config.movement_threshold
        } else {
            true
        };

        if should_update {
            self.previous_positions.insert(entity, new_center);
            self.dirty_entities.insert(entity);

            match self.structure_type {
                SpatialStructureType::Octree => self.octree.update(entity, new_bounds),
                SpatialStructureType::Bvh => {
                    self.bvh.update(entity, new_bounds);
                    true
                }
            }
        } else {
            false
        }
    }

    /// Forces an update of an entity regardless of movement threshold.
    pub fn force_update(&mut self, entity: Entity, new_bounds: Aabb) -> bool {
        let new_center = new_bounds.center();
        self.previous_positions.insert(entity, new_center);
        self.dirty_entities.insert(entity);

        match self.structure_type {
            SpatialStructureType::Octree => self.octree.update(entity, new_bounds),
            SpatialStructureType::Bvh => {
                self.bvh.update(entity, new_bounds);
                true
            }
        }
    }

    /// Processes dirty entities and performs rebalancing if needed.
    pub fn flush_updates(&mut self) {
        if !self.dirty_entities.is_empty() {
            self.update_counter += self.dirty_entities.len();
            self.dirty_entities.clear();

            if self.update_counter >= self.config.rebalance_interval {
                self.rebalance_if_needed();
                self.update_counter = 0;
            }
        }
    }

    /// Queries all entities that intersect the given bounds.
    pub fn query(&self, bounds: &Aabb) -> Vec<Entity> {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.query(bounds),
            SpatialStructureType::Bvh => self.bvh.query(bounds),
        }
    }

    /// Queries all entities within the given radius of a point.
    pub fn query_radius(&self, point: Vec3, radius: f32) -> Vec<Entity> {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.query_radius(point, radius),
            SpatialStructureType::Bvh => self.bvh.query_radius(point, radius),
        }
    }

    /// Queries all entities that intersect with a ray.
    pub fn query_ray(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Vec<Entity> {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.query_ray(origin, direction, max_distance),
            SpatialStructureType::Bvh => self.bvh.query_ray(origin, direction, max_distance),
        }
    }

    /// Queries all entities that intersect with a ray and returns them sorted by distance.
    pub fn query_ray_sorted(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Vec<(Entity, f32)> {
        match self.structure_type {
            SpatialStructureType::Octree => {
                self.octree
                    .query_ray_sorted(origin, direction, max_distance)
            }
            SpatialStructureType::Bvh => self.bvh.query_ray_sorted(origin, direction, max_distance),
        }
    }

    /// Returns the total number of entities in the spatial structure.
    pub fn entity_count(&self) -> usize {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.entity_count(),
            SpatialStructureType::Bvh => self.bvh.entity_count(),
        }
    }

    /// Clears all entities from the spatial structure.
    pub fn clear(&mut self) {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.clear(),
            SpatialStructureType::Bvh => self.bvh.clear(),
        }
        self.dirty_entities.clear();
        self.previous_positions.clear();
        self.update_counter = 0;
    }

    /// Checks if the spatial structure needs rebalancing.
    pub fn needs_rebalancing(&self) -> bool {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.needs_rebalancing(),
            SpatialStructureType::Bvh => false,
        }
    }

    /// Performs rebalancing if needed.
    pub fn rebalance_if_needed(&mut self) {
        match self.structure_type {
            SpatialStructureType::Octree => {
                if self.octree.needs_rebalancing() {
                    self.octree.rebuild();
                }
            }
            SpatialStructureType::Bvh => {
                self.bvh.rebuild();
            }
        }
    }

    /// Forces a rebuild of the spatial structure.
    pub fn rebuild(&mut self) {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.rebuild(),
            SpatialStructureType::Bvh => self.bvh.rebuild(),
        }
        self.update_counter = 0;
    }

    /// Returns true if the spatial structure contains the entity.
    pub fn contains(&self, entity: Entity) -> bool {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.contains(entity),
            SpatialStructureType::Bvh => self.bvh.contains(entity),
        }
    }

    /// Gets the bounds of an entity if it exists in the spatial structure.
    pub fn get_bounds(&self, entity: Entity) -> Option<&Aabb> {
        match self.structure_type {
            SpatialStructureType::Octree => self.octree.get_bounds(entity),
            SpatialStructureType::Bvh => self.bvh.get_bounds(entity),
        }
    }

    /// Returns the current structure type.
    pub fn structure_type(&self) -> SpatialStructureType {
        self.structure_type
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &SpatialConfig {
        &self.config
    }

    /// Sets the movement threshold.
    pub fn set_movement_threshold(&mut self, threshold: f32) {
        self.config.movement_threshold = threshold;
    }

    /// Sets the rebalance interval.
    pub fn set_rebalance_interval(&mut self, interval: usize) {
        self.config.rebalance_interval = interval;
    }

    /// Returns the number of dirty entities pending update.
    pub fn dirty_count(&self) -> usize {
        self.dirty_entities.len()
    }

    /// Returns the update counter.
    pub fn update_counter(&self) -> usize {
        self.update_counter
    }

    /// Queries entities using a frustum (view frustum culling).
    pub fn query_frustum(&self, frustum: &crate::Frustum) -> Vec<Entity> {
        let mut results = Vec::new();

        match self.structure_type {
            SpatialStructureType::Octree => {
                for &entity in self.previous_positions.keys() {
                    if let Some(stored_bounds) = self.octree.get_bounds(entity) {
                        if frustum.intersects_aabb(stored_bounds) {
                            results.push(entity);
                        }
                    }
                }
            }
            SpatialStructureType::Bvh => {
                for &entity in self.previous_positions.keys() {
                    if let Some(stored_bounds) = self.bvh.get_bounds(entity) {
                        if frustum.intersects_aabb(stored_bounds) {
                            results.push(entity);
                        }
                    }
                }
            }
        }

        results
    }

    /// Returns statistics about the spatial structure.
    pub fn stats(&self) -> SpatialStats {
        SpatialStats {
            entity_count: self.entity_count(),
            dirty_count: self.dirty_count(),
            update_counter: self.update_counter,
            structure_type: self.structure_type,
        }
    }

    /// Finds the nearest entity to a given point within `max_distance`.
    pub fn query_nearest(&self, point: Vec3, max_distance: f32) -> Option<(Entity, f32)> {
        let candidates = self.query_radius(point, max_distance);

        let mut nearest: Option<(Entity, f32)> = None;

        for entity in candidates {
            if let Some(bounds) = self.get_bounds(entity) {
                let distance = bounds.center().distance(point);
                if distance <= max_distance {
                    if let Some((_, best_dist)) = nearest {
                        if distance < best_dist {
                            nearest = Some((entity, distance));
                        }
                    } else {
                        nearest = Some((entity, distance));
                    }
                }
            }
        }

        nearest
    }

    /// Finds the K nearest entities to a given point within `max_distance`.
    pub fn query_k_nearest(&self, point: Vec3, k: usize, max_distance: f32) -> Vec<(Entity, f32)> {
        let candidates = self.query_radius(point, max_distance);

        let mut distances: Vec<(Entity, f32)> = candidates
            .into_iter()
            .filter_map(|entity| {
                self.get_bounds(entity).map(|bounds| {
                    let distance = bounds.center().distance(point);
                    (entity, distance)
                })
            })
            .filter(|(_, distance)| *distance <= max_distance)
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        distances.truncate(k);
        distances
    }
}

/// Statistics about the spatial structure.
#[derive(Debug, Clone, Copy)]
pub struct SpatialStats {
    /// Total number of entities.
    pub entity_count: usize,
    /// Number of dirty entities.
    pub dirty_count: usize,
    /// Update counter.
    pub update_counter: usize,
    /// Structure type.
    pub structure_type: SpatialStructureType,
}

impl Default for SpatialManager {
    fn default() -> Self {
        Self::default_octree()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_manager_creation() {
        let manager = SpatialManager::default_octree();
        assert_eq!(manager.structure_type(), SpatialStructureType::Octree);
        assert_eq!(manager.entity_count(), 0);
    }

    #[test]
    fn test_spatial_manager_insert_remove() {
        let mut manager = SpatialManager::default_octree();
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        assert!(manager.insert(entity, bounds));
        assert_eq!(manager.entity_count(), 1);
        assert!(manager.contains(entity));

        assert!(manager.remove(entity));
        assert!(!manager.contains(entity));
    }

    #[test]
    fn test_spatial_manager_query() {
        let mut manager = SpatialManager::default_octree();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        manager.insert(entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE));
        manager.insert(
            entity2,
            Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0)),
        );

        let query_bounds =
            Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let results = manager.query(&query_bounds);

        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_spatial_manager_ray_query() {
        let mut manager = SpatialManager::default_octree();
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0));

        manager.insert(entity, bounds);

        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let results = manager.query_ray(origin, direction, 100.0);

        assert!(results.contains(&entity));
    }

    #[test]
    fn test_spatial_manager_update_threshold() {
        let mut manager = SpatialManager::default_octree();
        manager.set_movement_threshold(1.0);

        let entity = Entity::from_raw(1);
        let bounds1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        manager.insert(entity, bounds1);

        let bounds2 = Aabb::from_min_max(Vec3::new(0.1, 0.0, 0.0), Vec3::new(1.1, 1.0, 1.0));
        assert!(!manager.update(entity, bounds2));

        let bounds3 = Aabb::from_min_max(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));
        assert!(manager.update(entity, bounds3));
    }

    #[test]
    fn test_bvh_manager() {
        let mut manager = SpatialManager::default_bvh();
        assert_eq!(manager.structure_type(), SpatialStructureType::Bvh);

        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        manager.insert(entity, bounds);
        assert!(manager.contains(entity));

        let results = manager.query(&bounds);
        assert!(results.contains(&entity));
    }

    #[test]
    fn test_nearest_query() {
        let mut manager = SpatialManager::default_octree();

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        manager.insert(
            entity1,
            Aabb::from_center_half_extents(Vec3::new(5.0, 0.0, 0.0), Vec3::ONE),
        );
        manager.insert(
            entity2,
            Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::ONE),
        );
        manager.insert(
            entity3,
            Aabb::from_center_half_extents(Vec3::new(3.0, 0.0, 0.0), Vec3::ONE),
        );

        let nearest = manager.query_nearest(Vec3::ZERO, 100.0);
        assert!(nearest.is_some());
        let (nearest_entity, _) = nearest.unwrap();
        assert_eq!(nearest_entity, entity3);
    }

    #[test]
    fn test_k_nearest_query() {
        let mut manager = SpatialManager::default_octree();

        for i in 0..10 {
            let entity = Entity::from_raw(i);
            let x = (i as f32 * 5.0);
            manager.insert(
                entity,
                Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::ONE),
            );
        }

        let k_nearest = manager.query_k_nearest(Vec3::ZERO, 3, 100.0);
        assert_eq!(k_nearest.len(), 3);

        for i in 0..k_nearest.len() - 1 {
            assert!(k_nearest[i].1 <= k_nearest[i + 1].1);
        }
    }

    #[test]
    fn test_stats() {
        let mut manager = SpatialManager::default_octree();
        let entity = Entity::from_raw(1);
        manager.insert(entity, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE));

        let stats = manager.stats();
        assert_eq!(stats.entity_count, 1);
        assert_eq!(stats.structure_type, SpatialStructureType::Octree);
    }

    #[test]
    fn test_force_update() {
        let mut manager = SpatialManager::default_octree();
        manager.set_movement_threshold(100.0);

        let entity = Entity::from_raw(1);
        let bounds1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        manager.insert(entity, bounds1);

        let bounds2 = Aabb::from_min_max(Vec3::new(0.1, 0.0, 0.0), Vec3::new(1.1, 1.0, 1.0));
        assert!(!manager.update(entity, bounds2));

        assert!(manager.force_update(entity, bounds2));
    }
}
