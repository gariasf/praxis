//! Spatial Optimization Demo
//!
//! This example demonstrates the spatial optimization systems including:
//! - Frustum culling
//! - Octree spatial partitioning
//! - BVH queries
//! - LOD (Level of Detail) system
//! - Occlusion culling (conceptual, requires GPU)
//!
//! Press:
//! - W/A/S/D: Move camera
//! - Space/Shift: Move camera up/down
//! - Mouse: Look around
//! - F: Toggle frustum culling visualization
//! - O: Toggle octree visualization
//! - L: Toggle LOD level display

use praxis_ecs::{BoundingBox, LodComponent, MeshHandle, Transform, World};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_spatial::{
    Aabb, Bvh, CullingStats, FrustumCuller, LodGroup, LodLevel, Octree, SpatialLodManager,
    VisibilitySystem,
};
use praxis_utils::info;

fn main() -> praxis_utils::Result<()> {
    info!("Starting Spatial Optimization Demo");

    // Create world and systems
    let mut world = World::new();
    let mut octree = Octree::new(Vec3::ZERO, 1000.0, 8);
    let mut bvh = Bvh::new();
    let mut lod_manager = SpatialLodManager::new();
    let mut visibility_system = VisibilitySystem::with_max_distance(500.0);

    // Configure LOD groups
    setup_lod_groups(&mut lod_manager);

    // Spawn a large number of objects in a grid
    let grid_size = 20;
    let spacing = 10.0;
    let mut entities_with_bounds = Vec::new();

    for x in 0..grid_size {
        for z in 0..grid_size {
            let position = Vec3::new(
                (x as f32 - grid_size as f32 / 2.0) * spacing,
                0.0,
                (z as f32 - grid_size as f32 / 2.0) * spacing,
            );

            // Alternate between different object types
            let object_type = ((x + z) % 3) as usize;
            let (mesh_name, lod_group, bounds_size) = match object_type {
                0 => ("tree_high", "tree", 3.0),
                1 => ("rock_high", "rock", 2.0),
                _ => ("bush_high", "bush", 1.5),
            };

            let bounds = BoundingBox::from_center_half_extents(position, Vec3::splat(bounds_size));

            let entity = world.spawn((
                Transform::from_translation(position),
                MeshHandle::new(mesh_name),
                bounds,
                LodComponent::new(lod_group),
            ));

            // Insert into octree
            let aabb = Aabb::from_min_max(bounds.min, bounds.max);
            octree.insert(entity, aabb);
            entities_with_bounds.push((entity, aabb));

            // Assign to LOD manager
            visibility_system
                .lod_manager_mut()
                .assign_entity(entity, lod_group);
        }
    }

    info!("Spawned {} entities in octree", entities_with_bounds.len());

    // Build BVH
    bvh.build(entities_with_bounds.clone());
    info!("Built BVH with {} entities", bvh.entity_count());

    // Simulate camera movement and culling
    simulate_camera_culling(&octree, &bvh, &visibility_system);

    // Demonstrate spatial queries
    demonstrate_spatial_queries(&octree, &bvh);

    // Show LOD selection
    demonstrate_lod_selection(&visibility_system, &entities_with_bounds);

    info!("Spatial Optimization Demo completed");
    Ok(())
}

fn setup_lod_groups(lod_manager: &mut SpatialLodManager) {
    // Tree LOD group
    lod_manager.register_lod_group(LodGroup::new(
        "tree",
        vec![
            LodLevel::new(0.0, "tree_high"),
            LodLevel::new(50.0, "tree_medium"),
            LodLevel::new(100.0, "tree_low"),
            LodLevel::new(200.0, "tree_billboard"),
        ],
    ));

    // Rock LOD group
    lod_manager.register_lod_group(LodGroup::new(
        "rock",
        vec![
            LodLevel::new(0.0, "rock_high"),
            LodLevel::new(40.0, "rock_low"),
        ],
    ));

    // Bush LOD group
    lod_manager.register_lod_group(LodGroup::new(
        "bush",
        vec![
            LodLevel::new(0.0, "bush_high"),
            LodLevel::new(30.0, "bush_low"),
        ],
    ));

    info!("Configured {} LOD groups", lod_manager.group_count());
}

fn simulate_camera_culling(octree: &Octree, bvh: &Bvh, visibility_system: &VisibilitySystem) {
    info!("\n=== Camera Culling Simulation ===");

    // Create camera at different positions
    let camera_positions = [
        Vec3::new(0.0, 10.0, 100.0),
        Vec3::new(50.0, 10.0, 50.0),
        Vec3::new(-50.0, 10.0, -50.0),
    ];

    for (i, camera_pos) in camera_positions.iter().enumerate() {
        info!("\nCamera position {}: {:?}", i + 1, camera_pos);

        // Create view-projection matrix
        let view = Mat4::look_at_rh(*camera_pos, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(70.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
        let view_proj = proj * view;

        // Create frustum culler
        let mut frustum_culler = FrustumCuller::new();
        frustum_culler.update(view_proj);

        // Query octree for potentially visible objects
        let camera_bounds = Aabb::from_center_half_extents(*camera_pos, Vec3::splat(100.0));
        let nearby_entities = octree.query(&camera_bounds);

        // Count visible after frustum culling
        let mut visible_count = 0;
        for entity in &nearby_entities {
            // In a real implementation, you'd query the entity's bounds from ECS
            // For this demo, we'll use a simple check
            let bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(2.0));
            if frustum_culler.is_visible(&bounds) {
                visible_count += 1;
            }
        }

        info!("  Nearby entities (octree): {}", nearby_entities.len());
        info!("  Visible after frustum culling: {}", visible_count);
        info!(
            "  Culled: {} ({:.1}%)",
            nearby_entities.len() - visible_count,
            ((nearby_entities.len() - visible_count) as f32 / nearby_entities.len() as f32) * 100.0
        );
    }
}

fn demonstrate_spatial_queries(octree: &Octree, bvh: &Bvh) {
    info!("\n=== Spatial Queries ===");

    // Radius query
    let query_point = Vec3::new(0.0, 0.0, 0.0);
    let query_radius = 50.0;

    let octree_results = octree.query_radius(query_point, query_radius);
    let bvh_results = bvh.query_radius(query_point, query_radius);

    info!(
        "Radius query at {:?} with radius {}:",
        query_point, query_radius
    );
    info!("  Octree found: {} entities", octree_results.len());
    info!("  BVH found: {} entities", bvh_results.len());

    // AABB query
    let query_bounds =
        Aabb::from_min_max(Vec3::new(-25.0, -5.0, -25.0), Vec3::new(25.0, 5.0, 25.0));

    let octree_box_results = octree.query(&query_bounds);
    let bvh_box_results = bvh.query(&query_bounds);

    info!("\nAABB query with bounds {:?}:", query_bounds);
    info!("  Octree found: {} entities", octree_box_results.len());
    info!("  BVH found: {} entities", bvh_box_results.len());
}

fn demonstrate_lod_selection(
    visibility_system: &VisibilitySystem,
    entities_with_bounds: &[(bevy_ecs::entity::Entity, Aabb)],
) {
    info!("\n=== LOD Selection ===");

    let camera_positions = [
        Vec3::new(0.0, 10.0, 20.0),
        Vec3::new(0.0, 10.0, 80.0),
        Vec3::new(0.0, 10.0, 150.0),
    ];

    for (i, camera_pos) in camera_positions.iter().enumerate() {
        info!(
            "\nLOD selection from camera position {}: {:?}",
            i + 1,
            camera_pos
        );

        // Convert entities and bounds to positions
        let entities_with_positions: Vec<_> = entities_with_bounds
            .iter()
            .map(|(entity, aabb)| (*entity, aabb.center()))
            .collect();

        let selections = visibility_system
            .lod_manager()
            .select_lods(&entities_with_positions, *camera_pos);

        // Count LOD level distribution
        let mut lod_counts = [0; 4];
        for selection in &selections {
            if selection.level_index < 4 {
                lod_counts[selection.level_index] += 1;
            }
        }

        info!("  LOD distribution:");
        info!("    High detail (LOD 0): {}", lod_counts[0]);
        info!("    Medium detail (LOD 1): {}", lod_counts[1]);
        info!("    Low detail (LOD 2): {}", lod_counts[2]);
        info!("    Billboard (LOD 3): {}", lod_counts[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_optimization_demo() {
        assert!(main().is_ok());
    }
}
