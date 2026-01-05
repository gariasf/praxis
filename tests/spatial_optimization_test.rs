//! Integration tests for spatial optimization systems.
//!
//! These tests cover:
//! - Frustum culling correctness
//! - Octree and BVH query accuracy
//! - LOD distance selection logic
//! - Occlusion query integration

use praxis_ecs::{BoundingBox, World};
use praxis_math::{Mat4, Vec3};
use praxis_spatial::{
    Aabb, Bvh, CullReason, FrustumCuller, LodGroup, LodLevel, LodManager, Octree, VisibilitySystem,
};

#[test]
fn test_frustum_culling_basic_visibility() {
    // Create a frustum from camera matrices
    let camera_pos = Vec3::new(0.0, 0.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
    let view_proj = proj * view;

    let mut frustum_culler = FrustumCuller::new();
    frustum_culler.update(view_proj);

    // Object at origin should be visible
    let origin_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
    assert!(
        frustum_culler.is_visible(&origin_bounds),
        "Object at origin should be visible"
    );

    // Object in front of camera should be visible
    let front_bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 5.0), Vec3::splat(1.0));
    assert!(
        frustum_culler.is_visible(&front_bounds),
        "Object in front of camera should be visible"
    );

    // Object behind camera should not be visible
    let behind_bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 15.0), Vec3::splat(1.0));
    assert!(
        !frustum_culler.is_visible(&behind_bounds),
        "Object behind camera should not be visible"
    );

    // Object far to the side should not be visible
    let side_bounds = Aabb::from_center_half_extents(Vec3::new(50.0, 0.0, 0.0), Vec3::splat(1.0));
    assert!(
        !frustum_culler.is_visible(&side_bounds),
        "Object far to the side should not be visible"
    );
}

#[test]
fn test_frustum_culling_edge_cases() {
    let camera_pos = Vec3::new(0.0, 5.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), 1.0, 0.1, 100.0);
    let view_proj = proj * view;

    let mut frustum_culler = FrustumCuller::new();
    frustum_culler.update(view_proj);

    // Very large object that intersects frustum partially
    let large_bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, -5.0), Vec3::splat(20.0));
    assert!(
        frustum_culler.is_visible(&large_bounds),
        "Large object partially in frustum should be visible"
    );

    // Object at near plane
    let near_bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 5.0, 9.95), Vec3::splat(0.02));
    assert!(
        frustum_culler.is_visible(&near_bounds),
        "Object at near plane should be visible"
    );

    // Object beyond far plane
    let far_bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 5.0, -95.0), Vec3::splat(1.0));
    assert!(
        !frustum_culler.is_visible(&far_bounds),
        "Object beyond far plane should not be visible"
    );
}

#[test]
fn test_frustum_culling_sphere_visibility() {
    let camera_pos = Vec3::new(0.0, 0.0, 5.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 0.1, 100.0);
    let view_proj = proj * view;

    let mut frustum_culler = FrustumCuller::new();
    frustum_culler.update(view_proj);

    // Sphere at origin should be visible
    assert!(
        frustum_culler.is_sphere_visible(Vec3::ZERO, 1.0),
        "Sphere at origin should be visible"
    );

    // Small sphere far away should not be visible
    assert!(
        !frustum_culler.is_sphere_visible(Vec3::new(100.0, 0.0, 0.0), 0.5),
        "Small sphere far away should not be visible"
    );

    // Large sphere partially in frustum should be visible
    assert!(
        frustum_culler.is_sphere_visible(Vec3::new(10.0, 0.0, 0.0), 15.0),
        "Large sphere partially in frustum should be visible"
    );
}

#[test]
fn test_frustum_culling_point_visibility() {
    let camera_pos = Vec3::new(0.0, 10.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(70.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
    let view_proj = proj * view;

    let mut frustum_culler = FrustumCuller::new();
    frustum_culler.update(view_proj);

    // Point at look-at target should be visible
    assert!(
        frustum_culler.is_point_visible(Vec3::ZERO),
        "Point at look-at target should be visible"
    );

    // Point slightly off-center should be visible
    assert!(
        frustum_culler.is_point_visible(Vec3::new(1.0, 1.0, 1.0)),
        "Point slightly off-center should be visible"
    );
}

#[test]
fn test_octree_insertion_and_query() {
    let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);

    // Insert multiple entities
    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);
    let entity3 = bevy_ecs::entity::Entity::from_raw(3);

    let bounds1 = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(1.0));
    let bounds2 = Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0));
    let bounds3 = Aabb::from_center_half_extents(Vec3::new(-10.0, 0.0, 0.0), Vec3::splat(1.0));

    assert!(octree.insert(entity1, bounds1));
    assert!(octree.insert(entity2, bounds2));
    assert!(octree.insert(entity3, bounds3));

    assert_eq!(octree.entity_count(), 3);

    // Query small region containing only entity1
    let query_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(2.0));
    let results = octree.query(&query_bounds);

    assert!(results.contains(&entity1));
    assert_eq!(results.len(), 1);

    // Query larger region containing entity1 and entity2
    let query_bounds2 = Aabb::from_center_half_extents(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(7.0));
    let results2 = octree.query(&query_bounds2);

    assert!(results2.contains(&entity1));
    assert!(results2.contains(&entity2));
    assert!(!results2.contains(&entity3));
}

#[test]
fn test_octree_radius_query() {
    let mut octree = Octree::new(Vec3::ZERO, 200.0, 4);

    // Create a grid of entities
    let mut entities = Vec::new();
    for i in 0..10 {
        let entity = bevy_ecs::entity::Entity::from_raw(i);
        let x = (i as f32 - 5.0) * 5.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(1.0));
        octree.insert(entity, bounds);
        entities.push((entity, x));
    }

    // Query radius around origin
    let results = octree.query_radius(Vec3::ZERO, 10.0);

    // Should find entities within radius
    assert!(!results.is_empty());
    assert!(results.len() <= 10);

    // Verify entities outside radius are not included
    let far_results = octree.query_radius(Vec3::new(100.0, 0.0, 0.0), 5.0);
    assert!(far_results.is_empty());
}

#[test]
fn test_octree_removal_and_update() {
    let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);

    let entity = bevy_ecs::entity::Entity::from_raw(1);
    let bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));

    // Insert and verify
    assert!(octree.insert(entity, bounds));
    assert!(octree.contains(entity));
    assert_eq!(octree.entity_count(), 1);

    // Remove and verify
    assert!(octree.remove(entity));
    assert!(!octree.contains(entity));
    assert_eq!(octree.entity_count(), 0);

    // Re-insert
    assert!(octree.insert(entity, bounds));

    // Update position
    let new_bounds = Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0));
    assert!(octree.update(entity, new_bounds));

    // Query old position - should not find entity
    let old_query = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(2.0));
    let old_results = octree.query(&old_query);
    assert!(!old_results.contains(&entity));

    // Query new position - should find entity
    let new_query = Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(2.0));
    let new_results = octree.query(&new_query);
    assert!(new_results.contains(&entity));
}

#[test]
fn test_octree_ray_query() {
    let mut octree = Octree::new(Vec3::ZERO, 200.0, 4);

    // Place entities along a ray path
    for i in 0..5 {
        let entity = bevy_ecs::entity::Entity::from_raw(i);
        let z = (i as f32 + 1.0) * 10.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, z), Vec3::splat(2.0));
        octree.insert(entity, bounds);
    }

    // Ray along Z axis
    let origin = Vec3::ZERO;
    let direction = Vec3::Z;
    let results = octree.query_ray(origin, direction, 100.0);

    assert!(!results.is_empty());
    assert!(results.len() <= 5);

    // Ray in wrong direction should find nothing
    let wrong_results = octree.query_ray(origin, -direction, 100.0);
    assert!(wrong_results.is_empty());

    // Ray with limited range
    let limited_results = octree.query_ray(origin, direction, 25.0);
    assert!(!limited_results.is_empty());
    assert!(limited_results.len() < results.len());
}

#[test]
fn test_octree_ray_sorted_query() {
    let mut octree = Octree::new(Vec3::ZERO, 200.0, 4);

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);
    let entity3 = bevy_ecs::entity::Entity::from_raw(3);

    // Place entities at different distances
    octree.insert(
        entity1,
        Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 30.0), Vec3::splat(1.0)),
    );
    octree.insert(
        entity2,
        Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 10.0), Vec3::splat(1.0)),
    );
    octree.insert(
        entity3,
        Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 20.0), Vec3::splat(1.0)),
    );

    let results = octree.query_ray_sorted(Vec3::ZERO, Vec3::Z, 100.0);

    assert_eq!(results.len(), 3);

    // Verify sorted by distance
    for i in 0..results.len() - 1 {
        assert!(
            results[i].1 <= results[i + 1].1,
            "Results should be sorted by distance"
        );
    }

    // Closest should be entity2
    assert_eq!(results[0].0, entity2);
}

#[test]
fn test_bvh_build_and_query() {
    let mut bvh = Bvh::new();

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);
    let entity3 = bevy_ecs::entity::Entity::from_raw(3);

    let entities = vec![
        (
            entity1,
            Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0)),
        ),
        (
            entity2,
            Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0)),
        ),
        (
            entity3,
            Aabb::from_center_half_extents(Vec3::new(20.0, 0.0, 0.0), Vec3::splat(1.0)),
        ),
    ];

    bvh.build(entities);

    assert_eq!(bvh.entity_count(), 3);
    assert!(!bvh.is_empty());

    // Query near origin
    let query_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(2.0));
    let results = bvh.query(&query_bounds);

    assert!(results.contains(&entity1));
    assert_eq!(results.len(), 1);

    // Query larger region
    let large_query = Aabb::from_center_half_extents(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(7.0));
    let large_results = bvh.query(&large_query);

    assert!(large_results.contains(&entity1));
    assert!(large_results.contains(&entity2));
    assert!(!large_results.contains(&entity3));
}

#[test]
fn test_bvh_radius_query() {
    let mut bvh = Bvh::new();

    // Create entities at various distances
    let mut entities = Vec::new();
    for i in 0..8 {
        let entity = bevy_ecs::entity::Entity::from_raw(i);
        let angle = (i as f32) * std::f32::consts::PI / 4.0;
        let radius = 10.0;
        let pos = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        let bounds = Aabb::from_center_half_extents(pos, Vec3::splat(1.0));
        entities.push((entity, bounds));
    }

    bvh.build(entities);

    // Query with radius that should include all entities
    let results_all = bvh.query_radius(Vec3::ZERO, 15.0);
    assert_eq!(results_all.len(), 8);

    // Query with smaller radius
    let results_some = bvh.query_radius(Vec3::ZERO, 5.0);
    assert!(results_some.len() < 8);

    // Query with very small radius
    let results_none = bvh.query_radius(Vec3::ZERO, 1.0);
    assert!(results_none.is_empty());
}

#[test]
fn test_bvh_ray_query() {
    let mut bvh = Bvh::new();

    // Place entities along X axis
    let mut entities = Vec::new();
    for i in 0..5 {
        let entity = bevy_ecs::entity::Entity::from_raw(i);
        let x = (i as f32 + 1.0) * 10.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(2.0));
        entities.push((entity, bounds));
    }

    bvh.build(entities);

    // Ray along X axis
    let results = bvh.query_ray(Vec3::ZERO, Vec3::X, 100.0);
    assert_eq!(results.len(), 5);

    // Ray in opposite direction
    let neg_results = bvh.query_ray(Vec3::ZERO, -Vec3::X, 100.0);
    assert!(neg_results.is_empty());

    // Ray with limited range
    let limited_results = bvh.query_ray(Vec3::ZERO, Vec3::X, 25.0);
    assert!(!limited_results.is_empty());
    assert!(limited_results.len() < 5);
}

#[test]
fn test_bvh_ray_sorted_query() {
    let mut bvh = Bvh::new();

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);
    let entity3 = bevy_ecs::entity::Entity::from_raw(3);

    let entities = vec![
        (
            entity1,
            Aabb::from_center_half_extents(Vec3::new(30.0, 0.0, 0.0), Vec3::splat(1.0)),
        ),
        (
            entity2,
            Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0)),
        ),
        (
            entity3,
            Aabb::from_center_half_extents(Vec3::new(20.0, 0.0, 0.0), Vec3::splat(1.0)),
        ),
    ];

    bvh.build(entities);

    let results = bvh.query_ray_sorted(Vec3::ZERO, Vec3::X, 100.0);
    assert_eq!(results.len(), 3);

    // Verify sorted by distance
    for i in 0..results.len() - 1 {
        assert!(results[i].1 <= results[i + 1].1);
    }

    // Closest should be entity2
    assert_eq!(results[0].0, entity2);
}

#[test]
fn test_bvh_insert_remove_update() {
    let mut bvh = Bvh::new();

    let entity = bevy_ecs::entity::Entity::from_raw(1);
    let bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));

    // Insert
    bvh.insert(entity, bounds);
    assert!(bvh.contains(entity));
    assert_eq!(bvh.entity_count(), 1);

    // Update
    let new_bounds = Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0));
    bvh.update(entity, new_bounds);
    assert!(bvh.contains(entity));

    let stored_bounds = bvh.get_bounds(entity);
    assert!(stored_bounds.is_some());
    assert_eq!(stored_bounds.unwrap().center(), Vec3::new(10.0, 0.0, 0.0));

    // Remove
    assert!(bvh.remove(entity));
    assert!(!bvh.contains(entity));
    assert_eq!(bvh.entity_count(), 0);
}

#[test]
fn test_lod_level_creation() {
    let level = LodLevel::new(50.0, "tree_medium");
    assert_eq!(level.distance, 50.0);
    assert_eq!(level.mesh_id, "tree_medium");
}

#[test]
fn test_lod_group_selection() {
    let levels = vec![
        LodLevel::new(50.0, "tree_high"),
        LodLevel::new(100.0, "tree_medium"),
        LodLevel::new(200.0, "tree_low"),
    ];

    let group = LodGroup::new("tree", levels);
    assert_eq!(group.level_count(), 3);

    // Test distance-based selection
    assert_eq!(group.select_lod(10.0), Some("tree_high"));
    assert_eq!(group.select_lod(75.0), Some("tree_medium"));
    assert_eq!(group.select_lod(150.0), Some("tree_low"));
    assert_eq!(group.select_lod(300.0), Some("tree_low")); // Beyond all thresholds, use last
}

#[test]
fn test_lod_group_boundary_cases() {
    let levels = vec![
        LodLevel::new(50.0, "high"),
        LodLevel::new(100.0, "medium"),
        LodLevel::new(200.0, "low"),
    ];

    let group = LodGroup::new("test", levels);

    // Test exact boundaries
    assert_eq!(group.select_lod(0.0), Some("high"));
    assert_eq!(group.select_lod(50.0), Some("medium"));
    assert_eq!(group.select_lod(100.0), Some("low"));
    assert_eq!(group.select_lod(200.0), Some("low"));
}

#[test]
fn test_lod_manager_registration() {
    let mut manager = LodManager::new();

    let levels = vec![
        LodLevel::new(30.0, "rock_high"),
        LodLevel::new(60.0, "rock_low"),
    ];

    manager.register_lod_levels("rock", levels);
    assert_eq!(manager.group_count(), 1);

    let group = manager.get_group("rock");
    assert!(group.is_some());
    assert_eq!(group.unwrap().level_count(), 2);
}

#[test]
fn test_lod_manager_entity_assignment() {
    let mut manager = LodManager::new();

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);

    manager.register_lod_levels(
        "tree",
        vec![
            LodLevel::new(50.0, "tree_high"),
            LodLevel::new(100.0, "tree_low"),
        ],
    );

    manager.assign_entity(entity1, "tree");
    manager.assign_entity(entity2, "tree");

    assert_eq!(manager.entity_count(), 2);

    // Remove one entity
    manager.remove_entity(entity1);
    assert_eq!(manager.entity_count(), 1);
}

#[test]
fn test_lod_manager_selection() {
    let mut manager = LodManager::new();

    let entity = bevy_ecs::entity::Entity::from_raw(1);

    manager.register_lod_levels(
        "tree",
        vec![
            LodLevel::new(50.0, "tree_high"),
            LodLevel::new(100.0, "tree_medium"),
            LodLevel::new(200.0, "tree_low"),
        ],
    );

    manager.assign_entity(entity, "tree");

    // Test selection at various distances
    let camera_pos = Vec3::ZERO;

    // Close distance - should use high detail
    let close_pos = Vec3::new(20.0, 0.0, 0.0);
    let close_selection = manager.select_lod(entity, camera_pos, close_pos);
    assert!(close_selection.is_some());
    let close = close_selection.unwrap();
    assert_eq!(close.mesh_id, "tree_high");
    assert_eq!(close.level_index, 0);

    // Medium distance - should use medium detail
    let medium_pos = Vec3::new(75.0, 0.0, 0.0);
    let medium_selection = manager.select_lod(entity, camera_pos, medium_pos);
    assert!(medium_selection.is_some());
    let medium = medium_selection.unwrap();
    assert_eq!(medium.mesh_id, "tree_medium");
    assert_eq!(medium.level_index, 1);

    // Far distance - should use low detail
    let far_pos = Vec3::new(150.0, 0.0, 0.0);
    let far_selection = manager.select_lod(entity, camera_pos, far_pos);
    assert!(far_selection.is_some());
    let far = far_selection.unwrap();
    assert_eq!(far.mesh_id, "tree_low");
    assert_eq!(far.level_index, 2);
}

#[test]
fn test_lod_manager_batch_selection() {
    let mut manager = LodManager::new();

    manager.register_lod_levels(
        "tree",
        vec![
            LodLevel::new(50.0, "tree_high"),
            LodLevel::new(100.0, "tree_low"),
        ],
    );

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);
    let entity3 = bevy_ecs::entity::Entity::from_raw(3);

    manager.assign_entity(entity1, "tree");
    manager.assign_entity(entity2, "tree");
    manager.assign_entity(entity3, "tree");

    let entities = vec![
        (entity1, Vec3::new(20.0, 0.0, 0.0)),  // Close - high detail
        (entity2, Vec3::new(75.0, 0.0, 0.0)),  // Far - low detail
        (entity3, Vec3::new(30.0, 0.0, 0.0)),  // Close - high detail
    ];

    let selections = manager.select_lods(&entities, Vec3::ZERO);

    assert_eq!(selections.len(), 3);
    assert_eq!(selections[0].mesh_id, "tree_high");
    assert_eq!(selections[1].mesh_id, "tree_low");
    assert_eq!(selections[2].mesh_id, "tree_high");
}

#[test]
fn test_lod_distance_calculation_accuracy() {
    let mut manager = LodManager::new();

    manager.register_lod_levels(
        "test",
        vec![
            LodLevel::new(10.0, "lod0"),
            LodLevel::new(20.0, "lod1"),
        ],
    );

    let entity = bevy_ecs::entity::Entity::from_raw(1);
    manager.assign_entity(entity, "test");

    let camera_pos = Vec3::ZERO;
    let entity_pos = Vec3::new(3.0, 4.0, 0.0); // Distance = 5.0

    let selection = manager.select_lod(entity, camera_pos, entity_pos);
    assert!(selection.is_some());

    let sel = selection.unwrap();
    assert!((sel.distance - 5.0).abs() < 0.001);
    assert_eq!(sel.mesh_id, "lod0");
}

#[test]
fn test_visibility_system_basic() {
    let system = VisibilitySystem::new();
    assert_eq!(system.max_distance(), 1000.0);

    let system_custom = VisibilitySystem::with_max_distance(500.0);
    assert_eq!(system_custom.max_distance(), 500.0);
}

#[test]
fn test_visibility_system_frustum_update() {
    let mut system = VisibilitySystem::new();

    let camera_pos = Vec3::new(0.0, 0.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
    let view_proj = proj * view;

    system.update_frustum(view_proj);

    // Test that frustum was updated by checking visibility
    let entities = vec![(
        bevy_ecs::entity::Entity::from_raw(1),
        Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0)),
    )];

    let visible = system.frustum_cull_only(&entities);
    assert_eq!(visible.len(), 1);
}

#[test]
fn test_visibility_system_distance_culling() {
    let mut system = VisibilitySystem::with_max_distance(50.0);

    let camera_pos = Vec3::ZERO;
    let view = Mat4::look_at_rh(camera_pos, Vec3::Z, Vec3::Y);
    let proj = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 0.1, 200.0);
    let view_proj = proj * view;

    system.update_frustum(view_proj);

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);

    let entities = vec![
        (
            entity1,
            Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 20.0), Vec3::splat(1.0)),
            Vec3::new(0.0, 0.0, 20.0),
        ),
        (
            entity2,
            Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 100.0), Vec3::splat(1.0)),
            Vec3::new(0.0, 0.0, 100.0),
        ),
    ];

    let (results, stats) = system.cull_entities(&entities, camera_pos);

    assert_eq!(stats.total_objects, 2);
    assert_eq!(stats.distance_culled, 1);
    assert!(results
        .iter()
        .any(|r| r.cull_reason == Some(CullReason::DistanceCull)));
}

#[test]
fn test_visibility_system_frustum_culling() {
    let mut system = VisibilitySystem::with_max_distance(1000.0);

    let camera_pos = Vec3::new(0.0, 0.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), 1.0, 0.1, 100.0);
    let view_proj = proj * view;

    system.update_frustum(view_proj);

    let entity1 = bevy_ecs::entity::Entity::from_raw(1);
    let entity2 = bevy_ecs::entity::Entity::from_raw(2);

    let entities = vec![
        (
            entity1,
            Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0)),
            Vec3::ZERO,
        ),
        (
            entity2,
            Aabb::from_center_half_extents(Vec3::new(50.0, 0.0, 0.0), Vec3::splat(1.0)),
            Vec3::new(50.0, 0.0, 0.0),
        ),
    ];

    let (results, stats) = system.cull_entities(&entities, camera_pos);

    assert_eq!(stats.total_objects, 2);
    assert!(stats.frustum_culled > 0);
    assert!(results
        .iter()
        .any(|r| r.cull_reason == Some(CullReason::FrustumCull)));
}

#[test]
fn test_visibility_system_with_lod() {
    let mut system = VisibilitySystem::new();

    // Register LOD group
    system.lod_manager_mut().register_lod_levels(
        "tree",
        vec![
            LodLevel::new(50.0, "tree_high"),
            LodLevel::new(100.0, "tree_low"),
        ],
    );

    let entity = bevy_ecs::entity::Entity::from_raw(1);
    system.lod_manager_mut().assign_entity(entity, "tree");

    let camera_pos = Vec3::ZERO;
    let view = Mat4::look_at_rh(camera_pos, Vec3::Z, Vec3::Y);
    let proj = Mat4::perspective_rh(90.0_f32.to_radians(), 1.0, 0.1, 200.0);
    let view_proj = proj * view;

    system.update_frustum(view_proj);

    let entities = vec![(
        entity,
        Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, 30.0), Vec3::splat(1.0)),
        Vec3::new(0.0, 0.0, 30.0),
    )];

    let (results, stats) = system.cull_entities(&entities, camera_pos);

    assert_eq!(stats.visible_objects, 1);
    assert!(results[0].is_visible);
    assert!(results[0].lod.is_some());

    let lod = results[0].lod.as_ref().unwrap();
    assert_eq!(lod.mesh_id, "tree_high");
}

#[test]
fn test_visibility_system_culling_stats() {
    let mut system = VisibilitySystem::with_max_distance(100.0);

    let camera_pos = Vec3::new(0.0, 0.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
    let view_proj = proj * view;

    system.update_frustum(view_proj);

    let mut entities = Vec::new();
    for i in 0..20 {
        let entity = bevy_ecs::entity::Entity::from_raw(i);
        let x = ((i % 5) as f32 - 2.0) * 30.0;
        let z = ((i / 5) as f32 - 1.0) * 30.0;
        let pos = Vec3::new(x, 0.0, z);
        let bounds = Aabb::from_center_half_extents(pos, Vec3::splat(2.0));
        entities.push((entity, bounds, pos));
    }

    let (_results, stats) = system.cull_entities(&entities, camera_pos);

    assert_eq!(stats.total_objects, 20);
    assert!(stats.visible_objects < stats.total_objects);
    assert!(stats.cull_rate() > 0.0);
    assert!(stats.cull_rate() <= 100.0);
}

#[test]
fn test_octree_bvh_query_consistency() {
    // Test that octree and BVH return similar results for the same queries
    let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);
    let mut bvh = Bvh::new();

    let mut bvh_entities = Vec::new();
    for i in 0..10 {
        let entity = bevy_ecs::entity::Entity::from_raw(i);
        let x = (i as f32 - 5.0) * 5.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(1.0));

        octree.insert(entity, bounds);
        bvh_entities.push((entity, bounds));
    }

    bvh.build(bvh_entities);

    // Query both structures
    let query_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(15.0));

    let octree_results = octree.query(&query_bounds);
    let bvh_results = bvh.query(&query_bounds);

    // Should find similar entities (exact match depends on implementation details)
    assert!(!octree_results.is_empty());
    assert!(!bvh_results.is_empty());

    // Both should find entities near origin
    let entity_near_origin = bevy_ecs::entity::Entity::from_raw(5); // Entity at x=0
    assert!(octree_results.contains(&entity_near_origin));
    assert!(bvh_results.contains(&entity_near_origin));
}

#[test]
fn test_integrated_culling_pipeline() {
    // Test complete culling pipeline with multiple systems
    let mut world = World::new();
    let mut visibility_system = VisibilitySystem::with_max_distance(200.0);

    // Set up LOD groups
    visibility_system.lod_manager_mut().register_lod_levels(
        "object",
        vec![
            LodLevel::new(50.0, "high"),
            LodLevel::new(100.0, "medium"),
            LodLevel::new(150.0, "low"),
        ],
    );

    // Create test entities
    let mut test_entities = Vec::new();
    for i in 0..30 {
        let entity = world.spawn(());

        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 30.0;
        let radius = ((i / 10) as f32 + 1.0) * 40.0;
        let pos = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        let bounds = Aabb::from_center_half_extents(pos, Vec3::splat(2.0));

        visibility_system
            .lod_manager_mut()
            .assign_entity(entity, "object");

        test_entities.push((entity, bounds, pos));
    }

    // Set up camera
    let camera_pos = Vec3::new(0.0, 10.0, 0.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Z);
    let proj = Mat4::perspective_rh(75.0_f32.to_radians(), 16.0 / 9.0, 1.0, 300.0);
    let view_proj = proj * view;

    visibility_system.update_frustum(view_proj);

    // Run culling
    let (results, stats) = visibility_system.cull_entities(&test_entities, camera_pos);

    // Verify results
    assert_eq!(stats.total_objects, 30);
    assert!(stats.visible_objects > 0);
    assert!(stats.visible_objects < stats.total_objects);

    // Verify LOD selection for visible objects
    for result in &results {
        if result.is_visible {
            assert!(result.lod.is_some());
            let lod = result.lod.as_ref().unwrap();
            assert!(lod.distance >= 0.0);
            assert!(!lod.mesh_id.is_empty());
        }
    }

    // Verify culling reasons for culled objects
    for result in &results {
        if !result.is_visible {
            assert!(result.cull_reason.is_some());
        }
    }
}

#[test]
fn test_spatial_structures_with_moving_objects() {
    // Test dynamic updates as objects move
    let mut octree = Octree::new(Vec3::ZERO, 200.0, 8);

    let entity = bevy_ecs::entity::Entity::from_raw(1);
    let start_pos = Vec3::new(-50.0, 0.0, 0.0);
    let start_bounds = Aabb::from_center_half_extents(start_pos, Vec3::splat(2.0));

    octree.insert(entity, start_bounds);

    // Simulate object moving across space
    for i in 1..10 {
        let new_pos = Vec3::new(-50.0 + (i as f32 * 10.0), 0.0, 0.0);
        let new_bounds = Aabb::from_center_half_extents(new_pos, Vec3::splat(2.0));
        octree.update(entity, new_bounds);

        // Query should find entity at new position
        let query = Aabb::from_center_half_extents(new_pos, Vec3::splat(5.0));
        let results = octree.query(&query);
        assert!(
            results.contains(&entity),
            "Entity should be found at updated position"
        );

        // Query at old position should not find entity
        if i > 2 {
            let old_query = Aabb::from_center_half_extents(start_pos, Vec3::splat(5.0));
            let old_results = octree.query(&old_query);
            assert!(
                !old_results.contains(&entity),
                "Entity should not be at old position"
            );
        }
    }
}

#[test]
fn test_lod_selection_with_camera_movement() {
    let mut manager = LodManager::new();

    manager.register_lod_levels(
        "building",
        vec![
            LodLevel::new(100.0, "detailed"),
            LodLevel::new(200.0, "simplified"),
            LodLevel::new(400.0, "billboard"),
        ],
    );

    let entity = bevy_ecs::entity::Entity::from_raw(1);
    manager.assign_entity(entity, "building");

    let entity_pos = Vec3::ZERO;

    // Simulate camera moving away
    let distances = [50.0, 150.0, 300.0, 500.0];
    let expected_lods = ["detailed", "simplified", "billboard", "billboard"];

    for (distance, expected) in distances.iter().zip(expected_lods.iter()) {
        let camera_pos = Vec3::new(*distance, 0.0, 0.0);
        let selection = manager.select_lod(entity, camera_pos, entity_pos);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.mesh_id, *expected);
    }
}
