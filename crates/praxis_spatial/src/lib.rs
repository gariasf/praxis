//! Spatial optimization systems for the Praxis engine.
//!
//! This crate provides comprehensive spatial optimization systems including:
//! - **Frustum Culling**: Eliminates objects outside camera view
//! - **Octree/BVH**: Hierarchical spatial partitioning for efficient queries
//! - **LOD System**: Level-of-detail mesh switching based on distance
//! - **Occlusion Culling**: Hardware-based occlusion queries to skip hidden objects
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::cast_precision_loss)]
//!
//! # Architecture
//!
//! The spatial optimization pipeline works as follows:
//!
//! 1. **Scene Organization**: Objects are inserted into spatial structures (octree/BVH)
//! 2. **Frustum Culling**: Quick rejection of objects outside camera frustum
//! 3. **LOD Selection**: Choose appropriate mesh detail level based on distance
//! 4. **Occlusion Culling**: Test object visibility using GPU occlusion queries
//! 5. **Rendering**: Only visible objects at appropriate LOD levels are rendered
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use praxis_spatial::{
//!     FrustumCuller, Octree, LodManager, OcclusionCuller,
//!     Aabb, BoundingVolume, LodLevel
//! };
//! use praxis_ecs::World;
//! use praxis_math::{Vec3, Mat4};
//!
//! # fn example() -> praxis_utils::Result<()> {
//! let mut world = World::new();
//!
//! // Create spatial optimization systems
//! let mut octree = Octree::new(Vec3::ZERO, 1000.0, 4);
//! let mut lod_manager = LodManager::new();
//! // let mut occlusion_culler = OcclusionCuller::new(&device, &allocator)?;
//!
//! // Add objects to octree
//! let bounds = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
//! // octree.insert(entity, bounds);
//!
//! // Configure LOD levels for a mesh
//! // lod_manager.register_lod_group("tree", vec![
//! //     LodLevel { distance: 0.0, mesh_id: "tree_high".to_string() },
//! //     LodLevel { distance: 50.0, mesh_id: "tree_medium".to_string() },
//! //     LodLevel { distance: 100.0, mesh_id: "tree_low".to_string() },
//! //     LodLevel { distance: 200.0, mesh_id: "tree_billboard".to_string() },
//! // ]);
//!
//! // During rendering, use systems to cull and optimize
//! // let visible = frustum_culler.cull(&camera_frustum, &octree);
//! # Ok(())
//! # }
//! ```

pub mod aabb;
pub mod bvh;
pub mod culling;
pub mod frustum;
pub mod lod;
pub mod occlusion;
pub mod octree;
pub mod spatial_manager;
pub mod spatial_systems;

pub use aabb::{Aabb, BoundingVolume};
pub use bvh::{Bvh, BvhNode};
pub use culling::{CullReason, CullingResult, CullingStats, VisibilitySystem};
pub use frustum::{Frustum, FrustumCuller, Plane};
pub use lod::{LodGroup, LodLevel, LodManager, LodSelection};
pub use occlusion::{OcclusionCuller, OcclusionCullerStats, OcclusionQuery, OcclusionQueryPool, OcclusionQueryResult};
pub use octree::{Octree, OctreeNode};
pub use spatial_manager::{SpatialConfig, SpatialManager, SpatialStats, SpatialStructureType};
pub use spatial_systems::{
    SpatialBundle, SpatialBounds, SpatialEntity, SpatialResource, SpatialSystemConfig,
    SpatialSystemSet, auto_rebalance_spatial, flush_spatial_updates, insert_spatial_entities,
    remove_spatial_entities, update_spatial_enabled, update_spatial_entities,
};

use praxis_utils::{info, Result};

/// Initializes the spatial optimization system.
///
/// # Errors
///
/// Returns an error if initialization fails.
pub fn init() -> Result<()> {
    info!("Initializing spatial optimization system");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
