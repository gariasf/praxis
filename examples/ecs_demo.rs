//! Simple ECS demonstration example for the Praxis engine.
//!
//! This example demonstrates the core ECS functionality:
//! - Creating a world and spawning entities
//! - Using built-in components
//! - Setting up parent-child relationships
//! - Querying entities and their components

use praxis_ecs::{
    Active, Children, GlobalTransform, Name, Parent, Transform, TransformBundle, Visibility, World,
};
use praxis_math::{Quat, Vec3};

fn main() -> praxis_utils::Result<()> {
    // Initialize utilities (logging, error handling)
    praxis_utils::init()?;
    praxis_ecs::init()?;

    println!("=== Praxis ECS Demo ===\n");

    // Create a new world
    let mut world = World::new();
    println!("Created new ECS world");

    // Spawn a scene root
    let scene_root = world.spawn((
        Name::from("Scene Root"),
        TransformBundle::from_xyz(0.0, 0.0, 0.0),
        Children::new(),
    ));
    println!("Spawned scene root entity: {:?}", scene_root);

    // Spawn a player entity
    let player = world.spawn((
        Name::from("Player"),
        TransformBundle::from_xyz(0.0, 1.0, 0.0),
        Active,
        Visibility::Visible,
    ));
    println!("Spawned player entity at (0, 1, 0)");

    // Spawn a rotating platform
    let platform = world.spawn((
        Name::from("Rotating Platform"),
        TransformBundle::from_xyz(10.0, 0.0, 0.0),
        Parent(scene_root),
        Children::new(),
        Visibility::Visible,
    ));
    println!("Spawned platform at (10, 0, 0) as child of scene root");

    // Update scene root's children
    world
        .insert_component(scene_root, Children::with_children(vec![platform]))
        .expect("Failed to add children to scene root");

    // Spawn objects on the platform
    let mut platform_objects = Vec::new();
    for i in 0..3 {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / 3.0;
        let radius = 3.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let obj = world.spawn((
            Name::from(format!("Platform Object {}", i + 1)),
            TransformBundle::from_transform(Transform {
                translation: Vec3::new(x, 1.0, z),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            }),
            Parent(platform),
            Visibility::Visible,
            Active,
        ));
        platform_objects.push(obj);
        println!(
            "  Spawned object {} at local position ({:.2}, 1.0, {:.2})",
            i + 1,
            x,
            z
        );
    }

    // Update platform's children
    world
        .insert_component(platform, Children::with_children(platform_objects.clone()))
        .expect("Failed to add children to platform");

    // Print world statistics
    println!("\n=== World Statistics ===");
    println!("Total entities: {}", world.entity_count());
    println!("Entities spawned: {}", world.stats().entities_spawned);

    // Simulate a frame update
    println!("\n=== Simulating Transform Updates ===");

    // Rotate the platform
    {
        let inner_world = world.inner_mut();
        if let Some(mut transform) = inner_world.get_mut::<Transform>(platform) {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
            println!("Rotated platform by 45 degrees");
        }
    }

    // Manually propagate transforms for this demo
    // In a real game, this would be handled by the transform propagation system

    // First, update scene root's global transform
    {
        let inner_world = world.inner_mut();
        if let Some(transform) = inner_world.get::<Transform>(scene_root) {
            let transform_copy = *transform;
            if let Some(mut global) = inner_world.get_mut::<GlobalTransform>(scene_root) {
                *global = GlobalTransform::from(transform_copy);
            }
        }
    }

    // Then update platform's global transform based on parent
    {
        let inner_world = world.inner_mut();
        let parent_matrix = inner_world
            .get::<GlobalTransform>(scene_root)
            .map(|g| g.matrix)
            .unwrap_or(praxis_math::Mat4::IDENTITY);

        if let Some(transform) = inner_world.get::<Transform>(platform) {
            let child_matrix = parent_matrix * transform.compute_matrix();
            if let Some(mut global) = inner_world.get_mut::<GlobalTransform>(platform) {
                global.matrix = child_matrix;
            }
        }
    }

    // Finally update children's global transforms
    {
        let inner_world = world.inner_mut();
        let platform_matrix = inner_world
            .get::<GlobalTransform>(platform)
            .map(|g| g.matrix)
            .unwrap_or(praxis_math::Mat4::IDENTITY);

        for &child in &platform_objects {
            if let Some(transform) = inner_world.get::<Transform>(child) {
                let child_matrix = platform_matrix * transform.compute_matrix();
                if let Some(mut global) = inner_world.get_mut::<GlobalTransform>(child) {
                    global.matrix = child_matrix;
                }
            }
        }
    }

    // Query and display entity positions
    println!("\n=== Entity World Positions ===");
    {
        let inner_world = world.inner_mut();
        let mut query = inner_world.query::<(&Name, &GlobalTransform)>();

        for (name, global_transform) in query.iter(inner_world) {
            let pos = global_transform.translation();
            println!(
                "{}: World position = ({:.2}, {:.2}, {:.2})",
                name.as_str(),
                pos.x,
                pos.y,
                pos.z
            );
        }
    }

    // Query active entities
    println!("\n=== Active Entities ===");
    {
        let inner_world = world.inner_mut();
        let mut active_query = inner_world.query::<(&Name, &Active)>();

        let active_count = active_query.iter(inner_world).count();
        println!("Found {} active entities:", active_count);

        for (name, _) in active_query.iter(inner_world) {
            println!("  - {}", name.as_str());
        }
    }

    // Query parent-child relationships
    println!("\n=== Parent-Child Relationships ===");
    {
        let inner_world = world.inner_mut();
        let mut parent_query = inner_world.query::<(&Name, &Children)>();

        for (name, children) in parent_query.iter(inner_world) {
            println!("{} has {} children:", name.as_str(), children.len());

            for &child_entity in children.iter() {
                if let Some(child_name) = inner_world.get::<Name>(child_entity) {
                    println!("  - {}", child_name.as_str());
                }
            }
        }
    }

    // Demonstrate visibility queries
    println!("\n=== Visible Entities ===");
    {
        let inner_world = world.inner_mut();
        let mut visibility_query = inner_world.query::<(&Name, &Visibility)>();

        let visible_count = visibility_query
            .iter(inner_world)
            .filter(|(_, vis)| vis.is_visible())
            .count();

        println!("Found {} visible entities", visible_count);
    }

    // Demonstrate entity removal
    println!("\n=== Entity Management ===");
    let first_object = platform_objects[0];
    world.despawn(first_object)?;
    println!("Despawned first platform object");
    println!("Remaining entities: {}", world.entity_count());

    // Final statistics
    println!("\n=== Final Statistics ===");
    println!("Total entities spawned: {}", world.stats().entities_spawned);
    println!(
        "Total entities despawned: {}",
        world.stats().entities_despawned
    );
    println!("Current active entities: {}", world.stats().active_entities);

    println!("\n=== Demo Complete ===");
    Ok(())
}
