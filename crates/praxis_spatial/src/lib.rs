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
//! # Overview: Why Spatial Optimization Matters
//!
//! In a 3D game engine, rendering every object in a scene is prohibitively expensive. Consider
//! a city with 100,000 buildings - rendering all of them when the camera sees only 100 wastes
//! 99.9% of GPU resources. Spatial optimization solves this by:
//!
//! - **Reducing Draw Calls**: Only submit visible objects to the GPU (major performance win)
//! - **Lowering Vertex Processing**: Skip vertex shading for culled objects
//! - **Enabling Large Worlds**: Support scenes with millions of objects efficiently
//! - **Balancing LOD**: Show high detail nearby, low detail far away
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
//! # Spatial Partitioning Trade-offs
//!
//! This crate provides two primary spatial structures, each with different characteristics:
//!
//! ## Octree
//!
//! **Structure**: Recursively subdivides 3D space into 8 equal octants (2×2×2 grid).
//! Each node splits into 8 children when entity count exceeds threshold.
//!
//! **Best For**:
//! - Static scenes with uniform object distribution
//! - Broad-phase collision detection
//! - Scenes where objects naturally cluster in space (e.g., voxel worlds)
//!
//! **Advantages**:
//! - Simple, intuitive spatial subdivision
//! - Good for evenly distributed objects
//! - Natural fit for volumetric data (voxels, particles)
//! - Predictable memory layout
//!
//! **Disadvantages**:
//! - Poor performance with non-uniform object distribution (empty space wastes nodes)
//! - Fixed subdivision can miss tight groupings
//! - Rebuilding entire tree for dynamic objects is expensive
//! - Objects spanning octant boundaries stored in parent (loose octree problem)
//!
//! **Insertion Cost**: O(log n) per object on average
//! **Query Cost**: O(log n) for point queries, O(k + log n) for range queries (k = results)
//!
//! ## BVH (Bounding Volume Hierarchy)
//!
//! **Structure**: Binary tree where each node's bounds tightly enclose its children.
//! Built bottom-up by recursively partitioning objects along longest axis.
//!
//! **Best For**:
//! - Ray tracing and ray casting (near-optimal for ray queries)
//! - Scenes with clustered or non-uniform object distribution
//! - Mesh rendering with frustum culling
//!
//! **Advantages**:
//! - Tight-fitting bounds (no wasted space testing)
//! - Excellent ray tracing performance (O(log n) average case)
//! - Adapts naturally to object clustering
//! - Binary branching = better cache performance than octree's 8-way
//!
//! **Disadvantages**:
//! - More complex construction algorithm
//! - Requires full rebuild for dynamic scenes (though faster to build than octree)
//! - Slightly higher memory overhead per node
//! - Less intuitive spatial partitioning than octree
//!
//! **Insertion Cost**: O(n log n) to rebuild entire tree (optimized with SAH heuristics)
//! **Query Cost**: O(log n) for ray queries, O(log n + k) for range queries
//!
//! ## Choosing Between Them
//!
//! | Scenario | Recommended Structure |
//! |----------|----------------------|
//! | Static mesh rendering | BVH (tighter bounds = better culling) |
//! | Ray casting (e.g., picking) | BVH (near-optimal for rays) |
//! | Voxel/volumetric data | Octree (natural spatial mapping) |
//! | Uniform object distribution | Octree (simpler, equally effective) |
//! | Clustered objects (cities, forests) | BVH (adapts to clustering) |
//! | Frequent insertions/removals | Neither - use spatial hashing instead |
//!
//! **Performance Rule of Thumb**: For typical game scenes with meshes and frustum culling,
//! BVH typically outperforms octree by 20-40% due to tighter bounds and better cache behavior.
//!
//! # Spatial Structure APIs
//!
//! ## SpatialManager (Recommended)
//!
//! `SpatialManager` provides a unified interface over both Octree and BVH structures:
//! - **Automatic structure selection**: Choose Octree or BVH at construction
//! - **Movement tracking**: Only updates entities that move beyond threshold
//! - **Automatic rebalancing**: Triggers rebuilds when needed
//! - **Consistent API**: Same methods work with either underlying structure
//!
//! Use `SpatialManager` when:
//! - You want a simple, high-level API
//! - You need automatic update management
//! - Your choice of Octree vs BVH might change
//! - You're building general-purpose systems
//!
//! ## Direct Octree/BVH (Advanced)
//!
//! Use `Octree` or `BVH` directly when:
//! - You need fine-grained control over rebuilds
//! - You want to avoid wrapper overhead
//! - You're implementing specialized spatial algorithms
//! - You know exactly which structure you need
//!
//! ## LodManager (Separate System)
//!
//! `LodManager` handles distance-based mesh detail selection:
//! - **Independent of spatial structures**: Works with any culling system
//! - **Entity-to-group mapping**: Assign entities to named LOD groups
//! - **Distance-based selection**: Choose mesh detail based on camera distance
//!
//! Use `LodManager` in combination with spatial structures:
//! 1. Spatial structure (Octree/BVH) culls invisible objects
//! 2. `LodManager` selects mesh detail for visible objects
//! 3. Renderer draws only visible objects at appropriate detail
//!
//! # Naming Convention Compliance
//!
//! This crate follows the Praxis naming conventions:
//! - **`SpatialManager`**: Manages spatial structures (Octree/BVH), handles updates/queries
//! - **`LodManager`**: Manages LOD groups and entity assignments
//! - **`Octree`/`Bvh`**: Data structures (no suffix needed for pure data structures)
//! - **`FrustumCuller`**: Performs culling operations (verb-based name acceptable)
//!
//! Note: `SpatialLodManager` has been renamed to `LodManager` to avoid redundant "Spatial" prefix.
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
pub mod gpu_culling;
pub mod gpu_integration;
pub mod lod;
pub mod occlusion;
pub mod octree;
pub mod spatial_manager;
pub mod spatial_systems;

pub use aabb::{Aabb, BoundingVolume};
pub use bvh::{Bvh, BvhNode};
pub use culling::{
    CullReason, CullingResult, CullingStats, HierarchicalCullingMode, VisibilitySystem,
};
pub use frustum::{Frustum, FrustumCuller, Plane};
pub use gpu_culling::{
    GpuCullingConfig, GpuCullingManager, GpuCullingResult, GpuCullingStats, GpuLodGroup,
    GpuLodLevel, GpuObjectData, MAX_LOD_GROUPS, MAX_LOD_LEVELS_PER_GROUP,
};
pub use gpu_integration::{CullableObject, HybridCullingManager};
pub use lod::{LodGroup, LodLevel, LodManager, LodSelection, SpatialLodManager};
pub use occlusion::{
    OcclusionCuller, OcclusionCullerStats, OcclusionQuery, OcclusionQueryPool, OcclusionQueryResult,
};
pub use octree::{Octree, OctreeNode};
pub use spatial_manager::{SpatialConfig, SpatialManager, SpatialStats, SpatialStructureType};
pub use spatial_systems::{
    auto_rebalance_spatial, flush_spatial_updates, insert_spatial_entities,
    remove_spatial_entities, update_spatial_enabled, update_spatial_entities, SpatialBounds,
    SpatialBundle, SpatialEntity, SpatialResource, SpatialSystemConfig, SpatialSystemSet,
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
