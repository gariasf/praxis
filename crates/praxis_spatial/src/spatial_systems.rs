//! ECS systems for spatial partitioning integration.
//!
//! Provides systems that automatically maintain spatial structures based on entity transforms.

use crate::{Aabb, SpatialConfig, SpatialManager, SpatialStructureType};
use bevy_ecs::prelude::*;
use praxis_math::Vec3;

/// Component marking an entity as part of the spatial partitioning system.
#[derive(Component, Debug, Clone, Copy)]
pub struct SpatialEntity {
    /// Whether this entity should be included in spatial queries.
    pub enabled: bool,
}

impl SpatialEntity {
    /// Creates a new spatial entity marker.
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Creates an enabled spatial entity.
    pub fn enabled() -> Self {
        Self { enabled: true }
    }

    /// Creates a disabled spatial entity.
    pub fn disabled() -> Self {
        Self { enabled: false }
    }
}

impl Default for SpatialEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Component storing cached bounding box for spatial queries.
#[derive(Component, Debug, Clone, Copy)]
pub struct SpatialBounds {
    /// Cached AABB for this entity.
    pub aabb: Aabb,
}

impl SpatialBounds {
    /// Creates new spatial bounds from an AABB.
    pub fn new(aabb: Aabb) -> Self {
        Self { aabb }
    }

    /// Creates spatial bounds from min/max points.
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self {
            aabb: Aabb::from_min_max(min, max),
        }
    }

    /// Creates spatial bounds from center and half-extents.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            aabb: Aabb::from_center_half_extents(center, half_extents),
        }
    }
}

/// Resource holding the spatial manager.
#[derive(Resource)]
pub struct SpatialResource {
    /// The spatial manager instance.
    pub manager: SpatialManager,
}

impl SpatialResource {
    /// Creates a new spatial resource with an octree.
    pub fn new_octree(config: SpatialConfig) -> Self {
        Self {
            manager: SpatialManager::new_octree(config),
        }
    }

    /// Creates a new spatial resource with a BVH.
    pub fn new_bvh(config: SpatialConfig) -> Self {
        Self {
            manager: SpatialManager::new_bvh(config),
        }
    }

    /// Creates a default octree-based spatial resource.
    pub fn default_octree() -> Self {
        Self {
            manager: SpatialManager::default_octree(),
        }
    }

    /// Creates a default BVH-based spatial resource.
    pub fn default_bvh() -> Self {
        Self {
            manager: SpatialManager::default_bvh(),
        }
    }
}

impl Default for SpatialResource {
    fn default() -> Self {
        Self::default_octree()
    }
}

/// System that inserts newly added spatial entities into the spatial structure.
#[allow(clippy::needless_pass_by_value)]
pub fn insert_spatial_entities(
    mut spatial: ResMut<SpatialResource>,
    query: Query<(Entity, &SpatialBounds, &SpatialEntity), Added<SpatialEntity>>,
) {
    for (entity, bounds, spatial_entity) in query.iter() {
        if spatial_entity.enabled {
            spatial.manager.insert(entity, bounds.aabb);
        }
    }
}

/// System that removes spatial entities when they're despawned or the component is removed.
pub fn remove_spatial_entities(
    mut spatial: ResMut<SpatialResource>,
    mut removed: RemovedComponents<SpatialEntity>,
) {
    for entity in removed.read() {
        spatial.manager.remove(entity);
    }
}

/// System that updates entity positions in the spatial structure when bounds change.
#[allow(clippy::needless_pass_by_value)]
pub fn update_spatial_entities(
    mut spatial: ResMut<SpatialResource>,
    query: Query<(Entity, &SpatialBounds, &SpatialEntity), Changed<SpatialBounds>>,
) {
    for (entity, bounds, spatial_entity) in query.iter() {
        if spatial_entity.enabled {
            if spatial.manager.contains(entity) {
                spatial.manager.update(entity, bounds.aabb);
            } else {
                spatial.manager.insert(entity, bounds.aabb);
            }
        }
    }
}

/// System that processes enabled/disabled state changes.
#[allow(clippy::needless_pass_by_value)]
pub fn update_spatial_enabled(
    mut spatial: ResMut<SpatialResource>,
    query: Query<(Entity, &SpatialBounds, &SpatialEntity), Changed<SpatialEntity>>,
) {
    for (entity, bounds, spatial_entity) in query.iter() {
        if spatial_entity.enabled {
            if !spatial.manager.contains(entity) {
                spatial.manager.insert(entity, bounds.aabb);
            }
        } else if spatial.manager.contains(entity) {
            spatial.manager.remove(entity);
        }
    }
}

/// System that flushes pending updates and performs rebalancing.
pub fn flush_spatial_updates(mut spatial: ResMut<SpatialResource>) {
    spatial.manager.flush_updates();
}

/// System that automatically rebalances the spatial structure when needed.
pub fn auto_rebalance_spatial(mut spatial: ResMut<SpatialResource>) {
    spatial.manager.rebalance_if_needed();
}

/// Bundle for spawning an entity with spatial partitioning support.
#[derive(Bundle)]
pub struct SpatialBundle {
    /// Spatial entity marker.
    pub spatial: SpatialEntity,
    /// Cached bounding box.
    pub bounds: SpatialBounds,
}

impl SpatialBundle {
    /// Creates a new spatial bundle with the given bounds.
    pub fn new(aabb: Aabb) -> Self {
        Self {
            spatial: SpatialEntity::enabled(),
            bounds: SpatialBounds::new(aabb),
        }
    }

    /// Creates a spatial bundle from min/max points.
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self {
            spatial: SpatialEntity::enabled(),
            bounds: SpatialBounds::from_min_max(min, max),
        }
    }

    /// Creates a spatial bundle from center and half-extents.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            spatial: SpatialEntity::enabled(),
            bounds: SpatialBounds::from_center_half_extents(center, half_extents),
        }
    }
}

/// Configuration for spatial systems.
#[derive(Resource, Debug, Clone)]
pub struct SpatialSystemConfig {
    /// Whether to automatically rebalance.
    pub auto_rebalance: bool,
    /// Structure type to use.
    pub structure_type: SpatialStructureType,
}

impl Default for SpatialSystemConfig {
    fn default() -> Self {
        Self {
            auto_rebalance: true,
            structure_type: SpatialStructureType::Octree,
        }
    }
}

/// System set for spatial partitioning systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpatialSystemSet {
    /// Insert new entities.
    Insert,
    /// Update existing entities.
    Update,
    /// Flush updates and rebalance.
    Flush,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_entity_creation() {
        let entity = SpatialEntity::enabled();
        assert!(entity.enabled);

        let disabled = SpatialEntity::disabled();
        assert!(!disabled.enabled);
    }

    #[test]
    fn test_spatial_bounds_creation() {
        let bounds = SpatialBounds::from_min_max(Vec3::ZERO, Vec3::ONE);
        assert_eq!(bounds.aabb.min, Vec3::ZERO);
        assert_eq!(bounds.aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_spatial_bundle_creation() {
        let bundle = SpatialBundle::from_min_max(Vec3::ZERO, Vec3::ONE);
        assert!(bundle.spatial.enabled);
        assert_eq!(bundle.bounds.aabb.min, Vec3::ZERO);
    }
}
