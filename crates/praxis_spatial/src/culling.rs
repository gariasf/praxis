//! Unified visibility and culling system.
//!
//! This module provides a high-level system that combines frustum culling, occlusion culling,
//! and LOD selection into a single visibility determination pipeline.
//!
//! # Hierarchical Culling
//!
//! The system supports three culling modes:
//!
//! - **None**: O(n) iteration over all entities (simple but slow for large scenes)
//! - **Octree**: Hierarchical spatial partitioning with recursive subdivision
//! - **BVH**: Binary tree with efficient bottom-up construction
//!
//! Both octree and BVH modes provide significant performance improvements by:
//!
//! 1. Testing parent node bounds against the frustum before descending to children
//! 2. Early rejection of entire subtrees that are outside the frustum
//! 3. Reducing the number of entity-level frustum tests from O(n) to O(log n)
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_spatial::{VisibilitySystem, Aabb, HierarchicalCullingMode};
//! use praxis_math::{Vec3, Mat4};
//! use bevy_ecs::entity::Entity;
//!
//! // Create a visibility system with octree-based hierarchical culling
//! let mut system = VisibilitySystem::with_octree(Vec3::ZERO, 1000.0, 8);
//!
//! // Insert entities with their bounds and positions
//! let entity = Entity::from_raw(1);
//! let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
//! system.insert_entity(entity, bounds, Vec3::splat(0.5));
//!
//! // Update frustum from camera
//! let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
//! let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 1000.0);
//! system.update_frustum(proj * view);
//!
//! // Perform hierarchical culling
//! let (results, stats) = system.cull_entities_hierarchical(Vec3::new(0.0, 0.0, 10.0));
//! println!("Visible: {}, Culled: {}", stats.visible_objects, stats.frustum_culled);
//! ```

use crate::{
    aabb::Aabb,
    bvh::Bvh,
    frustum::{Frustum, FrustumCuller},
    lod::{LodSelection, SpatialLodManager},
    octree::Octree,
};
use bevy_ecs::entity::Entity;
use praxis_math::{Mat4, Vec3};
use std::collections::HashMap;

/// Result of the culling process for a single entity.
#[derive(Debug, Clone)]
pub struct CullingResult {
    /// Entity being rendered.
    pub entity: Entity,
    /// Whether the entity is visible.
    pub is_visible: bool,
    /// LOD selection, if applicable.
    pub lod: Option<LodSelection>,
    /// Reason for culling (if culled).
    pub cull_reason: Option<CullReason>,
}

/// Reason why an object was culled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullReason {
    /// Object is outside the view frustum.
    FrustumCull,
    /// Object is occluded by other geometry.
    OcclusionCull,
    /// Object is beyond LOD range.
    DistanceCull,
}

/// Statistics about the culling process.
#[derive(Debug, Clone, Copy, Default)]
pub struct CullingStats {
    /// Total number of objects tested.
    pub total_objects: usize,
    /// Number of objects that passed all culling tests.
    pub visible_objects: usize,
    /// Number of objects culled by frustum.
    pub frustum_culled: usize,
    /// Number of objects culled by occlusion.
    pub occlusion_culled: usize,
    /// Number of objects culled by distance.
    pub distance_culled: usize,
}

impl CullingStats {
    /// Creates a new empty stats object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the culling efficiency as a percentage.
    pub fn cull_rate(&self) -> f32 {
        if self.total_objects == 0 {
            return 0.0;
        }
        let culled = self.total_objects - self.visible_objects;
        (culled as f32 / self.total_objects as f32) * 100.0
    }
}

/// Type of spatial structure to use for hierarchical culling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchicalCullingMode {
    /// Use octree for hierarchical culling.
    Octree,
    /// Use BVH for hierarchical culling.
    Bvh,
    /// Disable hierarchical culling (O(n) iteration).
    None,
}

/// Unified visibility determination system.
///
/// Combines multiple culling techniques and LOD selection into a single pipeline.
pub struct VisibilitySystem {
    /// Frustum culler.
    frustum_culler: FrustumCuller,
    /// LOD manager.
    lod_manager: SpatialLodManager,
    /// Maximum rendering distance.
    max_distance: f32,
    /// Hierarchical culling mode.
    hierarchical_mode: HierarchicalCullingMode,
    /// Octree for spatial queries (if enabled).
    octree: Option<Octree>,
    /// BVH for spatial queries (if enabled).
    bvh: Option<Bvh>,
    /// Map from entity to position for quick lookups.
    entity_positions: HashMap<Entity, Vec3>,
}

impl VisibilitySystem {
    /// Creates a new visibility system with no hierarchical culling.
    pub fn new() -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            lod_manager: SpatialLodManager::new(),
            max_distance: 1000.0,
            hierarchical_mode: HierarchicalCullingMode::None,
            octree: None,
            bvh: None,
            entity_positions: HashMap::new(),
        }
    }

    /// Creates a new visibility system with octree-based hierarchical culling.
    pub fn with_octree(center: Vec3, size: f32, max_entities_per_node: usize) -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            lod_manager: SpatialLodManager::new(),
            max_distance: 1000.0,
            hierarchical_mode: HierarchicalCullingMode::Octree,
            octree: Some(Octree::new(center, size, max_entities_per_node)),
            bvh: None,
            entity_positions: HashMap::new(),
        }
    }

    /// Creates a new visibility system with BVH-based hierarchical culling.
    pub fn with_bvh() -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            lod_manager: SpatialLodManager::new(),
            max_distance: 1000.0,
            hierarchical_mode: HierarchicalCullingMode::Bvh,
            octree: None,
            bvh: Some(Bvh::new()),
            entity_positions: HashMap::new(),
        }
    }

    /// Creates a new visibility system with a custom maximum distance.
    pub fn with_max_distance(max_distance: f32) -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            lod_manager: SpatialLodManager::new(),
            max_distance,
            hierarchical_mode: HierarchicalCullingMode::None,
            octree: None,
            bvh: None,
            entity_positions: HashMap::new(),
        }
    }

    /// Updates the frustum from camera matrices.
    pub fn update_frustum(&mut self, view_projection: Mat4) {
        self.frustum_culler.update(view_projection);
    }

    /// Returns a reference to the LOD manager.
    pub fn lod_manager(&self) -> &SpatialLodManager {
        &self.lod_manager
    }

    /// Returns a mutable reference to the LOD manager.
    pub fn lod_manager_mut(&mut self) -> &mut SpatialLodManager {
        &mut self.lod_manager
    }

    /// Sets the maximum rendering distance.
    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_distance = distance;
    }

    /// Gets the maximum rendering distance.
    pub fn max_distance(&self) -> f32 {
        self.max_distance
    }

    /// Performs visibility culling on a list of entities.
    pub fn cull_entities(
        &self,
        entities: &[(Entity, Aabb, Vec3)],
        camera_position: Vec3,
    ) -> (Vec<CullingResult>, CullingStats) {
        let mut results = Vec::with_capacity(entities.len());
        let mut stats = CullingStats {
            total_objects: entities.len(),
            ..Default::default()
        };

        for &(entity, bounds, position) in entities {
            let distance = camera_position.distance(position);

            if distance > self.max_distance {
                results.push(CullingResult {
                    entity,
                    is_visible: false,
                    lod: None,
                    cull_reason: Some(CullReason::DistanceCull),
                });
                stats.distance_culled += 1;
                continue;
            }

            if !self.frustum_culler.is_visible(&bounds) {
                results.push(CullingResult {
                    entity,
                    is_visible: false,
                    lod: None,
                    cull_reason: Some(CullReason::FrustumCull),
                });
                stats.frustum_culled += 1;
                continue;
            }

            let lod = self
                .lod_manager
                .select_lod(entity, camera_position, position);

            results.push(CullingResult {
                entity,
                is_visible: true,
                lod,
                cull_reason: None,
            });
            stats.visible_objects += 1;
        }

        (results, stats)
    }

    /// Performs frustum culling only on a list of entities.
    pub fn frustum_cull_only(&self, entities: &[(Entity, Aabb)]) -> Vec<Entity> {
        entities
            .iter()
            .filter(|(_, bounds)| self.frustum_culler.is_visible(bounds))
            .map(|(entity, _)| *entity)
            .collect()
    }

    /// Returns a reference to the frustum culler.
    pub fn frustum_culler(&self) -> &FrustumCuller {
        &self.frustum_culler
    }

    /// Sets the hierarchical culling mode.
    pub fn set_hierarchical_mode(&mut self, mode: HierarchicalCullingMode) {
        self.hierarchical_mode = mode;
        match mode {
            HierarchicalCullingMode::Octree => {
                if self.octree.is_none() {
                    self.octree = Some(Octree::new(Vec3::ZERO, 2000.0, 8));
                }
            }
            HierarchicalCullingMode::Bvh => {
                if self.bvh.is_none() {
                    self.bvh = Some(Bvh::new());
                }
            }
            HierarchicalCullingMode::None => {}
        }
    }

    /// Returns the current hierarchical culling mode.
    pub fn hierarchical_mode(&self) -> HierarchicalCullingMode {
        self.hierarchical_mode
    }

    /// Inserts or updates an entity in the spatial structure.
    pub fn insert_entity(&mut self, entity: Entity, bounds: Aabb, position: Vec3) {
        self.entity_positions.insert(entity, position);

        match self.hierarchical_mode {
            HierarchicalCullingMode::Octree => {
                if let Some(octree) = &mut self.octree {
                    octree.update(entity, bounds);
                }
            }
            HierarchicalCullingMode::Bvh => {
                if let Some(bvh) = &mut self.bvh {
                    bvh.update(entity, bounds);
                }
            }
            HierarchicalCullingMode::None => {}
        }
    }

    /// Removes an entity from the spatial structure.
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        self.entity_positions.remove(&entity);

        match self.hierarchical_mode {
            HierarchicalCullingMode::Octree => {
                if let Some(octree) = &mut self.octree {
                    return octree.remove(entity);
                }
            }
            HierarchicalCullingMode::Bvh => {
                if let Some(bvh) = &mut self.bvh {
                    return bvh.remove(entity);
                }
            }
            HierarchicalCullingMode::None => {}
        }
        false
    }

    /// Clears all entities from the spatial structure.
    pub fn clear_entities(&mut self) {
        self.entity_positions.clear();

        match self.hierarchical_mode {
            HierarchicalCullingMode::Octree => {
                if let Some(octree) = &mut self.octree {
                    octree.clear();
                }
            }
            HierarchicalCullingMode::Bvh => {
                if let Some(bvh) = &mut self.bvh {
                    bvh.clear();
                }
            }
            HierarchicalCullingMode::None => {}
        }
    }

    /// Rebuilds the spatial structure from current entity data.
    pub fn rebuild_spatial_structure(&mut self) {
        match self.hierarchical_mode {
            HierarchicalCullingMode::Octree => {
                if let Some(octree) = &mut self.octree {
                    octree.rebuild();
                }
            }
            HierarchicalCullingMode::Bvh => {
                if let Some(bvh) = &mut self.bvh {
                    bvh.rebuild();
                }
            }
            HierarchicalCullingMode::None => {}
        }
    }

    /// Performs hierarchical frustum culling using the spatial structure.
    ///
    /// This method uses the octree or BVH to efficiently cull entities hierarchically,
    /// avoiding the need to test every entity individually.
    pub fn cull_entities_hierarchical(
        &self,
        camera_position: Vec3,
    ) -> (Vec<CullingResult>, CullingStats) {
        let frustum = self.frustum_culler.frustum();
        let mut results = Vec::new();
        let mut stats = CullingStats::default();

        match self.hierarchical_mode {
            HierarchicalCullingMode::Octree => {
                if let Some(octree) = &self.octree {
                    self.cull_with_octree(
                        octree,
                        frustum,
                        camera_position,
                        &mut results,
                        &mut stats,
                    );
                }
            }
            HierarchicalCullingMode::Bvh => {
                if let Some(bvh) = &self.bvh {
                    self.cull_with_bvh(bvh, frustum, camera_position, &mut results, &mut stats);
                }
            }
            HierarchicalCullingMode::None => {
                // Fallback to O(n) iteration
                for (&entity, &position) in &self.entity_positions {
                    let bounds = match self.get_entity_bounds(entity) {
                        Some(b) => *b,
                        None => continue,
                    };

                    self.test_entity_visibility(
                        entity,
                        bounds,
                        position,
                        camera_position,
                        &mut results,
                        &mut stats,
                    );
                }
            }
        }

        stats.total_objects = self.entity_positions.len();
        (results, stats)
    }

    /// Performs frustum culling using the octree.
    fn cull_with_octree(
        &self,
        octree: &Octree,
        frustum: &Frustum,
        camera_position: Vec3,
        results: &mut Vec<CullingResult>,
        stats: &mut CullingStats,
    ) {
        // First test if the octree's root bounds intersect the frustum
        if !frustum.intersects_aabb(octree.bounds()) {
            // Early exit: entire octree is outside frustum
            return;
        }

        // Use predicate-based hierarchical query
        // This tests each octree node's bounds against the frustum before descending,
        // enabling efficient early rejection of entire subtrees
        let frustum_clone = frustum.clone();
        let candidates =
            octree.query_with_predicate(&|bounds: &Aabb| frustum_clone.intersects_aabb(bounds));

        for entity in candidates {
            if let Some(&bounds) = octree.get_bounds(entity) {
                if let Some(&position) = self.entity_positions.get(&entity) {
                    self.test_entity_visibility(
                        entity,
                        bounds,
                        position,
                        camera_position,
                        results,
                        stats,
                    );
                }
            }
        }
    }

    /// Performs frustum culling using the BVH.
    fn cull_with_bvh(
        &self,
        bvh: &Bvh,
        frustum: &Frustum,
        camera_position: Vec3,
        results: &mut Vec<CullingResult>,
        stats: &mut CullingStats,
    ) {
        // Early exit if BVH is empty
        if bvh.is_empty() {
            return;
        }

        // Test root bounds first
        if let Some(root_bounds) = bvh.bounds() {
            if !frustum.intersects_aabb(root_bounds) {
                // Early exit: entire BVH is outside frustum
                return;
            }
        }

        // Use predicate-based hierarchical query
        // This tests each BVH node's bounds against the frustum before descending,
        // enabling efficient early rejection of entire subtrees
        let frustum_clone = frustum.clone();
        let candidates =
            bvh.query_with_predicate(&|bounds: &Aabb| frustum_clone.intersects_aabb(bounds));

        for entity in candidates {
            if let Some(&bounds) = bvh.get_bounds(entity) {
                if let Some(&position) = self.entity_positions.get(&entity) {
                    self.test_entity_visibility(
                        entity,
                        bounds,
                        position,
                        camera_position,
                        results,
                        stats,
                    );
                }
            }
        }
    }

    /// Tests visibility for a single entity and updates results/stats.
    fn test_entity_visibility(
        &self,
        entity: Entity,
        bounds: Aabb,
        position: Vec3,
        camera_position: Vec3,
        results: &mut Vec<CullingResult>,
        stats: &mut CullingStats,
    ) {
        let distance = camera_position.distance(position);

        if distance > self.max_distance {
            results.push(CullingResult {
                entity,
                is_visible: false,
                lod: None,
                cull_reason: Some(CullReason::DistanceCull),
            });
            stats.distance_culled += 1;
            return;
        }

        if !self.frustum_culler.is_visible(&bounds) {
            results.push(CullingResult {
                entity,
                is_visible: false,
                lod: None,
                cull_reason: Some(CullReason::FrustumCull),
            });
            stats.frustum_culled += 1;
            return;
        }

        let lod = self
            .lod_manager
            .select_lod(entity, camera_position, position);

        results.push(CullingResult {
            entity,
            is_visible: true,
            lod,
            cull_reason: None,
        });
        stats.visible_objects += 1;
    }

    /// Computes an approximate AABB for the frustum.
    fn _compute_frustum_aabb(&self, _frustum: &Frustum, camera_position: Vec3) -> Aabb {
        // Compute frustum corners at max distance
        let max_dist = self.max_distance;

        // Extract frustum planes to compute approximate bounds
        // For simplicity, we create a large AABB that encompasses the visible area
        let half_size = Vec3::splat(max_dist);
        Aabb::from_center_half_extents(camera_position, half_size)
    }

    /// Gets the bounds of an entity from the appropriate spatial structure.
    fn get_entity_bounds(&self, entity: Entity) -> Option<&Aabb> {
        match self.hierarchical_mode {
            HierarchicalCullingMode::Octree => {
                self.octree.as_ref().and_then(|o| o.get_bounds(entity))
            }
            HierarchicalCullingMode::Bvh => self.bvh.as_ref().and_then(|b| b.get_bounds(entity)),
            HierarchicalCullingMode::None => None,
        }
    }

    /// Returns the number of entities tracked by this system.
    pub fn entity_count(&self) -> usize {
        self.entity_positions.len()
    }

    /// Returns a reference to the octree if available.
    pub fn octree(&self) -> Option<&Octree> {
        self.octree.as_ref()
    }

    /// Returns a mutable reference to the octree if available.
    pub fn octree_mut(&mut self) -> Option<&mut Octree> {
        self.octree.as_mut()
    }

    /// Returns a reference to the BVH if available.
    pub fn bvh(&self) -> Option<&Bvh> {
        self.bvh.as_ref()
    }

    /// Returns a mutable reference to the BVH if available.
    pub fn bvh_mut(&mut self) -> Option<&mut Bvh> {
        self.bvh.as_mut()
    }

    /// Performs advanced hierarchical frustum culling with BVH node traversal.
    ///
    /// This method directly traverses the BVH hierarchy, testing each node's bounds
    /// against the frustum before descending to children, enabling efficient early rejection
    /// of entire subtrees.
    pub fn cull_entities_with_bvh_traversal(
        &self,
        camera_position: Vec3,
    ) -> (Vec<CullingResult>, CullingStats) {
        let mut results = Vec::new();
        let mut stats = CullingStats::default();

        if let Some(bvh) = &self.bvh {
            let frustum = self.frustum_culler.frustum();
            self.traverse_bvh_hierarchy(bvh, frustum, camera_position, &mut results, &mut stats);
        }

        stats.total_objects = self.entity_positions.len();
        (results, stats)
    }

    /// Traverses the BVH hierarchy recursively for frustum culling.
    fn traverse_bvh_hierarchy(
        &self,
        bvh: &Bvh,
        frustum: &Frustum,
        camera_position: Vec3,
        results: &mut Vec<CullingResult>,
        stats: &mut CullingStats,
    ) {
        // Use predicate-based query for true hierarchical culling
        // This tests each BVH node's bounds against the frustum before descending
        let frustum_clone = frustum.clone();
        let candidates =
            bvh.query_with_predicate(&|bounds: &Aabb| frustum_clone.intersects_aabb(bounds));

        for entity in candidates {
            if let Some(&bounds) = bvh.get_bounds(entity) {
                if let Some(&position) = self.entity_positions.get(&entity) {
                    self.test_entity_visibility(
                        entity,
                        bounds,
                        position,
                        camera_position,
                        results,
                        stats,
                    );
                }
            }
        }
    }

    /// Performs advanced hierarchical frustum culling with octree node traversal.
    ///
    /// This method directly traverses the octree hierarchy, testing each node's bounds
    /// against the frustum before descending to children, enabling efficient early rejection
    /// of entire subtrees.
    pub fn cull_entities_with_octree_traversal(
        &self,
        camera_position: Vec3,
    ) -> (Vec<CullingResult>, CullingStats) {
        let mut results = Vec::new();
        let mut stats = CullingStats::default();

        if let Some(octree) = &self.octree {
            let frustum = self.frustum_culler.frustum();
            self.traverse_octree_hierarchy(
                octree,
                frustum,
                camera_position,
                &mut results,
                &mut stats,
            );
        }

        stats.total_objects = self.entity_positions.len();
        (results, stats)
    }

    /// Traverses the octree hierarchy recursively for frustum culling.
    fn traverse_octree_hierarchy(
        &self,
        octree: &Octree,
        frustum: &Frustum,
        camera_position: Vec3,
        results: &mut Vec<CullingResult>,
        stats: &mut CullingStats,
    ) {
        // Check if octree root bounds intersect frustum
        if !frustum.intersects_aabb(octree.bounds()) {
            return;
        }

        // Use predicate-based query for true hierarchical culling
        // This tests each octree node's bounds against the frustum before descending
        let frustum_clone = frustum.clone();
        let candidates =
            octree.query_with_predicate(&|bounds: &Aabb| frustum_clone.intersects_aabb(bounds));

        for entity in candidates {
            if let Some(&bounds) = octree.get_bounds(entity) {
                if let Some(&position) = self.entity_positions.get(&entity) {
                    self.test_entity_visibility(
                        entity,
                        bounds,
                        position,
                        camera_position,
                        results,
                        stats,
                    );
                }
            }
        }
    }
}

impl Default for VisibilitySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_culling_stats() {
        let stats = CullingStats {
            total_objects: 100,
            visible_objects: 60,
            frustum_culled: 30,
            occlusion_culled: 5,
            distance_culled: 5,
        };

        assert_eq!(stats.cull_rate(), 40.0);
    }

    #[test]
    fn test_visibility_system_creation() {
        let system = VisibilitySystem::new();
        assert_eq!(system.max_distance(), 1000.0);
    }

    #[test]
    fn test_visibility_system_max_distance() {
        let mut system = VisibilitySystem::with_max_distance(500.0);
        assert_eq!(system.max_distance(), 500.0);

        system.set_max_distance(750.0);
        assert_eq!(system.max_distance(), 750.0);
    }

    #[test]
    fn test_culling_result() {
        let entity = Entity::from_raw(1);
        let result = CullingResult {
            entity,
            is_visible: false,
            lod: None,
            cull_reason: Some(CullReason::FrustumCull),
        };

        assert_eq!(result.entity, entity);
        assert!(!result.is_visible);
        assert_eq!(result.cull_reason, Some(CullReason::FrustumCull));
    }

    #[test]
    fn test_cull_reason() {
        assert_eq!(CullReason::FrustumCull, CullReason::FrustumCull);
        assert_ne!(CullReason::FrustumCull, CullReason::OcclusionCull);
    }

    #[test]
    fn test_visibility_system_distance_culling() {
        let system = VisibilitySystem::with_max_distance(50.0);

        let entities = vec![
            (
                Entity::from_raw(1),
                Aabb::from_min_max(Vec3::new(0.0, 0.0, 0.0), Vec3::ONE),
                Vec3::new(20.0, 0.0, 0.0),
            ),
            (
                Entity::from_raw(2),
                Aabb::from_min_max(Vec3::new(100.0, 0.0, 0.0), Vec3::new(101.0, 1.0, 1.0)),
                Vec3::new(100.0, 0.0, 0.0),
            ),
        ];

        let (results, stats) = system.cull_entities(&entities, Vec3::ZERO);

        assert_eq!(stats.distance_culled, 1);
        assert!(results
            .iter()
            .any(|r| r.cull_reason == Some(CullReason::DistanceCull)));
    }

    #[test]
    fn test_visibility_system_with_octree() {
        let mut system = VisibilitySystem::with_octree(Vec3::ZERO, 1000.0, 8);

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let bounds1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let bounds2 = Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0));

        system.insert_entity(entity1, bounds1, Vec3::splat(0.5));
        system.insert_entity(entity2, bounds2, Vec3::new(10.5, 0.5, 0.5));

        assert_eq!(system.entity_count(), 2);
        assert_eq!(system.hierarchical_mode(), HierarchicalCullingMode::Octree);
    }

    #[test]
    fn test_visibility_system_with_bvh() {
        let mut system = VisibilitySystem::with_bvh();

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let bounds1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let bounds2 = Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0));

        system.insert_entity(entity1, bounds1, Vec3::splat(0.5));
        system.insert_entity(entity2, bounds2, Vec3::new(10.5, 0.5, 0.5));

        assert_eq!(system.entity_count(), 2);
        assert_eq!(system.hierarchical_mode(), HierarchicalCullingMode::Bvh);
    }

    #[test]
    fn test_hierarchical_culling_octree() {
        let mut system = VisibilitySystem::with_octree(Vec3::ZERO, 1000.0, 8);
        system.set_max_distance(100.0);

        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 1000.0);
        system.update_frustum(proj * view);

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        let bounds1 = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let bounds2 = Aabb::from_min_max(Vec3::new(50.0, 0.0, 0.0), Vec3::new(51.0, 1.0, 1.0));
        let bounds3 = Aabb::from_min_max(Vec3::new(200.0, 0.0, 0.0), Vec3::new(201.0, 1.0, 1.0));

        system.insert_entity(entity1, bounds1, Vec3::ZERO);
        system.insert_entity(entity2, bounds2, Vec3::new(50.5, 0.5, 0.5));
        system.insert_entity(entity3, bounds3, Vec3::new(200.5, 0.5, 0.5));

        let (results, stats) = system.cull_entities_hierarchical(Vec3::new(0.0, 0.0, 10.0));

        assert_eq!(stats.total_objects, 3);
        assert!(stats.visible_objects > 0 || stats.frustum_culled > 0 || stats.distance_culled > 0);
    }

    #[test]
    fn test_hierarchical_culling_bvh() {
        let mut system = VisibilitySystem::with_bvh();
        system.set_max_distance(100.0);

        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 1000.0);
        system.update_frustum(proj * view);

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        let bounds1 = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        let bounds2 = Aabb::from_min_max(Vec3::new(200.0, 0.0, 0.0), Vec3::new(201.0, 1.0, 1.0));

        system.insert_entity(entity1, bounds1, Vec3::ZERO);
        system.insert_entity(entity2, bounds2, Vec3::new(200.5, 0.5, 0.5));

        let (results, stats) = system.cull_entities_hierarchical(Vec3::new(0.0, 0.0, 10.0));

        assert_eq!(stats.total_objects, 2);
        assert!(results
            .iter()
            .any(|r| r.entity == entity1 || r.entity == entity2));
    }

    #[test]
    fn test_remove_entity_from_spatial_structure() {
        let mut system = VisibilitySystem::with_octree(Vec3::ZERO, 1000.0, 8);

        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        system.insert_entity(entity, bounds, Vec3::splat(0.5));
        assert_eq!(system.entity_count(), 1);

        assert!(system.remove_entity(entity));
        assert_eq!(system.entity_count(), 0);
    }

    #[test]
    fn test_clear_spatial_structure() {
        let mut system = VisibilitySystem::with_bvh();

        for i in 0..10 {
            let entity = Entity::from_raw(i);
            let bounds = Aabb::from_min_max(
                Vec3::new(i as f32, 0.0, 0.0),
                Vec3::new(i as f32 + 1.0, 1.0, 1.0),
            );
            system.insert_entity(entity, bounds, Vec3::new(i as f32 + 0.5, 0.5, 0.5));
        }

        assert_eq!(system.entity_count(), 10);

        system.clear_entities();
        assert_eq!(system.entity_count(), 0);
    }

    #[test]
    fn test_set_hierarchical_mode() {
        let mut system = VisibilitySystem::new();
        assert_eq!(system.hierarchical_mode(), HierarchicalCullingMode::None);

        system.set_hierarchical_mode(HierarchicalCullingMode::Octree);
        assert_eq!(system.hierarchical_mode(), HierarchicalCullingMode::Octree);
        assert!(system.octree().is_some());

        system.set_hierarchical_mode(HierarchicalCullingMode::Bvh);
        assert_eq!(system.hierarchical_mode(), HierarchicalCullingMode::Bvh);
        assert!(system.bvh().is_some());
    }

    #[test]
    fn test_rebuild_spatial_structure() {
        let mut system = VisibilitySystem::with_octree(Vec3::ZERO, 1000.0, 8);

        for i in 0..5 {
            let entity = Entity::from_raw(i);
            let bounds = Aabb::from_min_max(
                Vec3::new(i as f32, 0.0, 0.0),
                Vec3::new(i as f32 + 1.0, 1.0, 1.0),
            );
            system.insert_entity(entity, bounds, Vec3::new(i as f32 + 0.5, 0.5, 0.5));
        }

        assert_eq!(system.entity_count(), 5);

        system.rebuild_spatial_structure();
        assert_eq!(system.entity_count(), 5);
    }
}
