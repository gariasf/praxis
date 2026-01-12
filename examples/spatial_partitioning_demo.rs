//! Comprehensive demonstration of spatial partitioning features.
//!
//! This example shows:
//! - Octree and BVH creation and usage
//! - Dynamic insertion and removal
//! - Ray queries
//! - Automatic rebalancing
//! - ECS integration

use praxis_ecs::{Entity as EcsEntity, IntoSystemConfigs, Schedule, World};
use praxis_math::Vec3;
use praxis_spatial::{
    auto_rebalance_spatial, flush_spatial_updates, insert_spatial_entities,
    remove_spatial_entities, update_spatial_entities, Aabb, Bvh, Octree, SpatialBounds,
    SpatialBundle, SpatialConfig, SpatialEntity, SpatialManager, SpatialResource,
    SpatialStructureType, SpatialSystemSet,
};

fn main() {
    println!("=== Spatial Partitioning Demo ===\n");

    demo_octree_basic();
    demo_bvh_basic();
    demo_ray_queries();
    demo_dynamic_updates();
    demo_spatial_manager();
    demo_ecs_integration();

    println!("\n=== Demo Complete ===");
}

fn demo_octree_basic() {
    println!("--- Octree Basic Usage ---");

    let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);

    // Create entities using sequential IDs
    let entities: Vec<EcsEntity> = (0..10).map(EcsEntity::from_raw).collect();

    for (i, &entity) in entities.iter().enumerate() {
        let x = (i as f32 * 10.0) - 45.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(2.0));
        octree.insert(entity, bounds);
    }

    println!("  Inserted {} entities", octree.entity_count());

    let query_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(20.0));
    let results = octree.query(&query_bounds);
    println!("  Query found {} entities near origin", results.len());

    let radius_results = octree.query_radius(Vec3::ZERO, 15.0);
    println!("  Radius query found {} entities", radius_results.len());

    println!();
}

fn demo_bvh_basic() {
    println!("--- BVH Basic Usage ---");

    let mut bvh = Bvh::new();

    let entities: Vec<(EcsEntity, Aabb)> = (0..8)
        .map(|i| {
            let entity = EcsEntity::from_raw(100 + i);
            let pos = Vec3::new((i as f32 * 5.0) - 17.5, 0.0, 0.0);
            let bounds = Aabb::from_center_half_extents(pos, Vec3::splat(1.5));
            (entity, bounds)
        })
        .collect();

    bvh.build(entities);
    println!("  Built BVH with {} entities", bvh.entity_count());

    let query_bounds = Aabb::from_center_half_extents(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(10.0));
    let results = bvh.query(&query_bounds);
    println!("  Query found {} entities", results.len());

    println!();
}

fn demo_ray_queries() {
    println!("--- Ray Query Demo ---");

    let mut octree = Octree::new(Vec3::ZERO, 200.0, 4);

    for i in 0..5 {
        let entity = EcsEntity::from_raw(200 + i);
        let z = (i as f32 * 10.0) + 5.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(0.0, 0.0, z), Vec3::splat(2.0));
        octree.insert(entity, bounds);
    }

    let origin = Vec3::ZERO;
    let direction = Vec3::Z;
    let max_distance = 100.0;

    let ray_results = octree.query_ray(origin, direction, max_distance);
    println!("  Ray found {} entities", ray_results.len());

    let sorted_results = octree.query_ray_sorted(origin, direction, max_distance);
    println!("  Sorted by distance:");
    for (entity, distance) in sorted_results.iter().take(3) {
        println!("    Entity {entity:?} at distance {distance:.2}");
    }

    let mut bvh = Bvh::new();
    let bvh_entities: Vec<(EcsEntity, Aabb)> = (0..5)
        .map(|i| {
            let entity = EcsEntity::from_raw(300 + i);
            let x = (i as f32 * 8.0) + 5.0;
            let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(2.0));
            (entity, bounds)
        })
        .collect();
    bvh.build(bvh_entities);

    let bvh_ray_results = bvh.query_ray_sorted(Vec3::ZERO, Vec3::X, 100.0);
    println!("\n  BVH ray query found {} entities", bvh_ray_results.len());

    println!();
}

fn demo_dynamic_updates() {
    println!("--- Dynamic Updates Demo ---");

    let mut octree = Octree::new(Vec3::ZERO, 100.0, 4);

    let entity = EcsEntity::from_raw(1000);
    let initial_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
    octree.insert(entity, initial_bounds);

    println!("  Initial position: {:?}", Vec3::ZERO);
    println!("  Entity count: {}", octree.entity_count());

    for i in 1..=5 {
        let new_pos = Vec3::new(i as f32 * 5.0, 0.0, 0.0);
        let new_bounds = Aabb::from_center_half_extents(new_pos, Vec3::splat(1.0));
        octree.update(entity, new_bounds);
        println!("  Updated to position: {new_pos:?}");
    }

    octree.remove(entity);
    println!("  Removed entity, count: {}", octree.entity_count());

    println!();
}

fn demo_spatial_manager() {
    println!("--- Spatial Manager Demo ---");

    let config = SpatialConfig {
        center: Vec3::ZERO,
        size: 200.0,
        max_entities_per_node: 8,
        movement_threshold: 0.5,
        rebalance_interval: 50,
    };

    let mut manager = SpatialManager::new(config, SpatialStructureType::Octree);
    println!("  Created spatial manager (Octree)");

    for i in 0..20 {
        let entity = EcsEntity::from_raw(2000 + i);
        let x = (i as f32 * 8.0) - 76.0;
        let y = (i as f32 * 3.0).sin() * 10.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(x, y, 0.0), Vec3::splat(2.0));
        manager.insert(entity, bounds);
    }

    println!("  Inserted {} entities", manager.entity_count());

    let entity = EcsEntity::from_raw(2005);
    for step in 0..10 {
        let new_pos = Vec3::new(0.0, step as f32 * 0.3, 0.0);
        let new_bounds = Aabb::from_center_half_extents(new_pos, Vec3::splat(2.0));
        let updated = manager.update(entity, new_bounds);
        if updated {
            println!("  Updated entity at step {step}");
        }
    }

    manager.flush_updates();
    println!("  Flushed updates, dirty count: {}", manager.dirty_count());

    if manager.needs_rebalancing() {
        println!("  Rebalancing needed");
        manager.rebalance_if_needed();
        println!("  Rebalanced");
    }

    let query_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(30.0));
    let results = manager.query(&query_bounds);
    println!("  Query found {} entities near origin", results.len());

    let ray_results = manager.query_ray_sorted(Vec3::new(-100.0, 0.0, 0.0), Vec3::X, 200.0);
    println!("  Ray query hit {} entities", ray_results.len());

    println!();
}

fn demo_ecs_integration() {
    println!("--- ECS Integration Demo ---");

    let mut world = World::new();

    let spatial_config = SpatialConfig {
        center: Vec3::ZERO,
        size: 500.0,
        max_entities_per_node: 10,
        movement_threshold: 1.0,
        rebalance_interval: 100,
    };

    world.insert_resource(SpatialResource::new_octree(spatial_config));

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            insert_spatial_entities,
            update_spatial_entities,
            remove_spatial_entities,
            flush_spatial_updates,
            auto_rebalance_spatial,
        )
            .chain()
            .in_set(SpatialSystemSet::Update),
    );

    for i in 0..15 {
        let x = (i as f32 * 12.0) - 84.0;
        let z = (i as f32 * 0.5).sin() * 20.0;
        let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, z), Vec3::splat(3.0));

        world.spawn(SpatialBundle::new(bounds));
    }

    schedule.run(world.inner_mut());

    let spatial = world.inner().resource::<SpatialResource>();
    println!("  Spawned entities in ECS");
    println!(
        "  Spatial manager contains {} entities",
        spatial.manager.entity_count()
    );

    let test_entity = world.spawn((
        SpatialEntity::enabled(),
        SpatialBounds::from_center_half_extents(Vec3::new(50.0, 0.0, 0.0), Vec3::splat(2.0)),
    ));

    schedule.run(world.inner_mut());

    let spatial = world.inner().resource::<SpatialResource>();
    println!("  After spawn: {} entities", spatial.manager.entity_count());

    {
        let mut bounds = world
            .inner_mut()
            .get_mut::<SpatialBounds>(test_entity)
            .unwrap();
        bounds.aabb = Aabb::from_center_half_extents(Vec3::new(100.0, 0.0, 0.0), Vec3::splat(2.0));
    }

    schedule.run(world.inner_mut());

    let spatial = world.inner().resource::<SpatialResource>();
    if let Some(stored_bounds) = spatial.manager.get_bounds(test_entity) {
        println!(
            "  Updated bounds in spatial manager: center = {:?}",
            stored_bounds.center()
        );
    }

    let _ = world.despawn(test_entity);
    schedule.run(world.inner_mut());

    let spatial = world.inner().resource::<SpatialResource>();
    println!(
        "  After despawn: {} entities",
        spatial.manager.entity_count()
    );

    let spatial = world.inner().resource::<SpatialResource>();
    let query_result = spatial.manager.query_radius(Vec3::ZERO, 100.0);
    println!(
        "  Radius query from origin found {} entities",
        query_result.len()
    );

    println!();
}
