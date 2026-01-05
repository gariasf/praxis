//! Integration between spatial optimization and GPU culling systems.
//!
//! This module provides bridge functionality between the CPU-based spatial optimization
//! systems (frustum culling, LOD selection) and the GPU-driven culling system. It allows
//! seamless switching between CPU and GPU culling based on scene complexity.

use crate::{Aabb, LodGroup, LodLevel};
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;
use std::collections::HashMap;

/// Represents an object that can be culled and LOD-selected.
#[derive(Debug, Clone)]
pub struct CullableObject {
    /// Entity ID.
    pub entity: Entity,
    /// Axis-aligned bounding box for frustum culling.
    pub aabb: Aabb,
    /// World position for distance calculations.
    pub position: Vec3,
    /// Mesh ID for rendering.
    pub mesh_id: u32,
    /// Optional LOD group ID for LOD selection.
    pub lod_group_id: Option<u32>,
}

/// Manages hybrid CPU/GPU culling approach.
///
/// This manager automatically switches between CPU and GPU culling based on
/// object count and configuration. For small scenes (< 5000 objects), CPU culling
/// is more efficient. For large scenes (>= 5000 objects), GPU culling is preferred.
pub struct HybridCullingManager {
    /// Threshold for switching to GPU culling.
    gpu_culling_threshold: usize,
    /// Whether GPU culling is available.
    gpu_culling_available: bool,
    /// LOD groups for CPU culling.
    cpu_lod_groups: HashMap<u32, LodGroup>,
}

impl HybridCullingManager {
    /// Creates a new hybrid culling manager.
    pub fn new() -> Self {
        Self {
            gpu_culling_threshold: 5000,
            gpu_culling_available: false,
            cpu_lod_groups: HashMap::new(),
        }
    }

    /// Creates a new hybrid culling manager with custom threshold.
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            gpu_culling_threshold: threshold,
            gpu_culling_available: false,
            cpu_lod_groups: HashMap::new(),
        }
    }

    /// Sets whether GPU culling is available.
    pub fn set_gpu_culling_available(&mut self, available: bool) {
        self.gpu_culling_available = available;
    }

    /// Registers a CPU LOD group.
    pub fn register_lod_group(&mut self, id: u32, group: LodGroup) {
        self.cpu_lod_groups.insert(id, group);
    }

    /// Determines whether to use GPU or CPU culling for the given object count.
    pub fn should_use_gpu_culling(&self, object_count: usize) -> bool {
        self.gpu_culling_available && object_count >= self.gpu_culling_threshold
    }

    /// Gets the GPU culling threshold.
    pub fn gpu_culling_threshold(&self) -> usize {
        self.gpu_culling_threshold
    }

    /// Sets the GPU culling threshold.
    pub fn set_gpu_culling_threshold(&mut self, threshold: usize) {
        self.gpu_culling_threshold = threshold;
    }

    /// Gets a CPU LOD group by ID.
    pub fn get_lod_group(&self, id: u32) -> Option<&LodGroup> {
        self.cpu_lod_groups.get(&id)
    }
}

impl Default for HybridCullingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for converting between CPU and GPU representations.
pub mod conversion {
    use super::{LodGroup, LodLevel};

    /// Converts a spatial LOD group to GPU-compatible format.
    ///
    /// Returns a vector of tuples (`mesh_id`, `min_distance`, `max_distance`).
    pub fn lod_group_to_gpu_format(group: &LodGroup) -> Vec<(u32, f32, f32)> {
        let mut levels = Vec::new();

        for (i, level) in group.levels.iter().enumerate() {
            let mesh_id = hash_mesh_name(&level.mesh_id);
            let min_dist = level.distance;
            let max_dist = if i + 1 < group.levels.len() {
                group.levels[i + 1].distance
            } else {
                f32::MAX
            };

            levels.push((mesh_id, min_dist, max_dist));
        }

        levels
    }

    /// Simple hash function to convert mesh names to IDs.
    ///
    /// In production, this should use a proper string interning system.
    fn hash_mesh_name(name: &str) -> u32 {
        let mut hash: u32 = 0;
        for byte in name.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u32::from(byte));
        }
        hash
    }

    /// Converts spatial LOD level to distance-based format.
    pub fn lod_level_to_distance(level: &LodLevel) -> f32 {
        level.distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_manager_creation() {
        let manager = HybridCullingManager::new();
        assert_eq!(manager.gpu_culling_threshold(), 5000);
        assert!(!manager.should_use_gpu_culling(1000));
    }

    #[test]
    fn test_hybrid_manager_threshold() {
        let mut manager = HybridCullingManager::with_threshold(10000);
        assert_eq!(manager.gpu_culling_threshold(), 10000);

        manager.set_gpu_culling_threshold(8000);
        assert_eq!(manager.gpu_culling_threshold(), 8000);
    }

    #[test]
    fn test_should_use_gpu_culling() {
        let mut manager = HybridCullingManager::with_threshold(5000);

        manager.set_gpu_culling_available(false);
        assert!(!manager.should_use_gpu_culling(10000));

        manager.set_gpu_culling_available(true);
        assert!(!manager.should_use_gpu_culling(4000));
        assert!(manager.should_use_gpu_culling(5000));
        assert!(manager.should_use_gpu_culling(10000));
    }

    #[test]
    fn test_lod_group_registration() {
        let mut manager = HybridCullingManager::new();

        let levels = vec![
            LodLevel::new(10.0, "mesh_high"),
            LodLevel::new(50.0, "mesh_low"),
        ];
        let group = LodGroup::new("test", levels);

        manager.register_lod_group(1, group);
        assert!(manager.get_lod_group(1).is_some());
        assert!(manager.get_lod_group(999).is_none());
    }

    #[test]
    fn test_lod_group_conversion() {
        let levels = vec![
            LodLevel::new(0.0, "mesh_high"),
            LodLevel::new(10.0, "mesh_medium"),
            LodLevel::new(50.0, "mesh_low"),
        ];
        let group = LodGroup::new("test", levels);

        let gpu_format = conversion::lod_group_to_gpu_format(&group);

        assert_eq!(gpu_format.len(), 3);
        assert_eq!(gpu_format[0].1, 0.0);
        assert_eq!(gpu_format[0].2, 10.0);
        assert_eq!(gpu_format[1].1, 10.0);
        assert_eq!(gpu_format[1].2, 50.0);
        assert_eq!(gpu_format[2].1, 50.0);
        assert_eq!(gpu_format[2].2, f32::MAX);
    }

    #[test]
    fn test_hash_mesh_name_deterministic() {
        let hash1 = conversion::hash_mesh_name("test_mesh");
        let hash2 = conversion::hash_mesh_name("test_mesh");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_mesh_name_different() {
        let hash1 = conversion::hash_mesh_name("mesh_a");
        let hash2 = conversion::hash_mesh_name("mesh_b");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_cullable_object_creation() {
        let entity = Entity::from_raw(42);
        let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let position = Vec3::new(5.0, 10.0, 15.0);

        let obj = CullableObject {
            entity,
            aabb,
            position,
            mesh_id: 123,
            lod_group_id: Some(456),
        };

        assert_eq!(obj.entity, entity);
        assert_eq!(obj.mesh_id, 123);
        assert_eq!(obj.lod_group_id, Some(456));
    }
}
