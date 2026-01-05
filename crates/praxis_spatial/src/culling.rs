//! Unified visibility and culling system.
//!
//! This module provides a high-level system that combines frustum culling, occlusion culling,
//! and LOD selection into a single visibility determination pipeline.

use crate::{
    aabb::Aabb,
    frustum::FrustumCuller,
    lod::{SpatialLodManager, LodSelection},
};
use bevy_ecs::entity::Entity;
use praxis_math::{Mat4, Vec3};

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
}

impl VisibilitySystem {
    /// Creates a new visibility system.
    pub fn new() -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            lod_manager: SpatialLodManager::new(),
            max_distance: 1000.0,
        }
    }

    /// Creates a new visibility system with a custom maximum distance.
    pub fn with_max_distance(max_distance: f32) -> Self {
        Self {
            frustum_culler: FrustumCuller::new(),
            lod_manager: SpatialLodManager::new(),
            max_distance,
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
}
