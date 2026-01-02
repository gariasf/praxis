//! Transform propagation demonstration example.
//!
//! This example demonstrates the automatic transform propagation system,
//! showing how GlobalTransform is automatically computed from local Transform
//! and parent-child hierarchies.

use praxis_ecs::systems::{
    cleanup_removed_parents, propagate_transforms, propagate_transforms_for_changed_children,
    propagate_transforms_for_reparented, sync_parent_child_relationships,
};
use praxis_ecs::{
    Children, GlobalTransform, IntoSystemConfigs, Name, Parent, Schedule, Transform,
    TransformBundle, World,
};
use praxis_math::{Quat, Vec3};

fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;

    println!("=== Transform Propagation Demo ===\n");

    let mut world = World::new();

    // Create a schedule with all transform propagation systems
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            sync_parent_child_relationships,
            cleanup_removed_parents,
            propagate_transforms,
            propagate_transforms_for_reparented,
            propagate_transforms_for_changed_children,
        )
            .chain(),
    );

    println!("Creating a hierarchical scene:\n");
    println!("  Root (0, 0, 0)");
    println!("  └─ Platform (10, 0, 0)");
    println!("     ├─ Cube1 (5, 0, 0)");
    println!("     ├─ Cube2 (-5, 0, 0)");
    println!("     └─ Arm (0, 2, 0)");
    println!("        └─ Hand (0, 3, 0)\n");

    // Create root entity
    let root = world.spawn((Name::from("Root"), TransformBundle::from_xyz(0.0, 0.0, 0.0)));

    // Create platform (child of root)
    let platform = world.spawn((
        Name::from("Platform"),
        TransformBundle::from_xyz(10.0, 0.0, 0.0),
        Parent(root),
    ));

    // Create cubes on platform
    let cube1 = world.spawn((
        Name::from("Cube1"),
        TransformBundle::from_xyz(5.0, 0.0, 0.0),
        Parent(platform),
    ));

    let cube2 = world.spawn((
        Name::from("Cube2"),
        TransformBundle::from_xyz(-5.0, 0.0, 0.0),
        Parent(platform),
    ));

    // Create arm on platform
    let arm = world.spawn((
        Name::from("Arm"),
        TransformBundle::from_xyz(0.0, 2.0, 0.0),
        Parent(platform),
    ));

    // Create hand on arm (grandchild of platform)
    let hand = world.spawn((
        Name::from("Hand"),
        TransformBundle::from_xyz(0.0, 3.0, 0.0),
        Parent(arm),
    ));

    println!("=== Initial State (Before Transform Propagation) ===\n");
    print_entity_transforms(&world);

    // Run transform propagation systems
    world.inner_mut().run_schedule(&mut schedule);

    println!("\n=== After Initial Transform Propagation ===\n");
    print_entity_transforms(&world);

    // Test 1: Rotate the platform
    println!("\n=== Test 1: Rotating Platform by 90 degrees around Y ===\n");
    {
        let inner = world.inner_mut();
        if let Some(mut transform) = inner.get_mut::<Transform>(platform) {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        }
    }

    world.inner_mut().run_schedule(&mut schedule);
    print_entity_transforms(&world);

    // Test 2: Move a child entity
    println!("\n=== Test 2: Moving Cube1 to (8, 1, 0) ===\n");
    {
        let inner = world.inner_mut();
        if let Some(mut transform) = inner.get_mut::<Transform>(cube1) {
            transform.translation = Vec3::new(8.0, 1.0, 0.0);
        }
    }

    world.inner_mut().run_schedule(&mut schedule);
    print_entity_transforms(&world);

    // Test 3: Scale the arm
    println!("\n=== Test 3: Scaling Arm by 2x ===\n");
    {
        let inner = world.inner_mut();
        if let Some(mut transform) = inner.get_mut::<Transform>(arm) {
            transform.scale = Vec3::new(2.0, 2.0, 2.0);
        }
    }

    world.inner_mut().run_schedule(&mut schedule);
    print_entity_transforms(&world);

    // Test 4: Reparent hand to root
    println!("\n=== Test 4: Reparenting Hand to Root ===\n");
    {
        let inner = world.inner_mut();
        if let Some(mut parent) = inner.get_mut::<Parent>(hand) {
            parent.0 = root;
        }
    }

    world.inner_mut().run_schedule(&mut schedule);
    print_entity_transforms(&world);

    // Test 5: Create a new entity and parent it
    println!("\n=== Test 5: Adding New Cube3 to Platform ===\n");
    let cube3 = world.spawn((
        Name::from("Cube3"),
        TransformBundle::from_xyz(0.0, 5.0, 0.0),
        Parent(platform),
    ));

    world.inner_mut().run_schedule(&mut schedule);
    print_entity_transforms(&world);

    // Test 6: Remove parent from an entity
    println!("\n=== Test 6: Removing Parent from Cube2 ===\n");
    {
        let inner = world.inner_mut();
        inner.entity_mut(cube2).remove::<Parent>();
    }

    world.inner_mut().run_schedule(&mut schedule);
    print_entity_transforms(&world);

    // Verify parent-child relationships
    println!("\n=== Final Parent-Child Relationships ===\n");
    print_hierarchy(&world);

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Helper function to print entity transforms
fn print_entity_transforms(world: &World) {
    let inner = world.inner();
    let mut query = inner.query::<(&Name, &Transform, &GlobalTransform)>();

    for (name, transform, global_transform) in query.iter(inner) {
        let local_pos = transform.translation;
        let global_pos = global_transform.translation();

        println!(
            "{:12} | Local: ({:6.2}, {:6.2}, {:6.2}) | Global: ({:6.2}, {:6.2}, {:6.2})",
            name.as_str(),
            local_pos.x,
            local_pos.y,
            local_pos.z,
            global_pos.x,
            global_pos.y,
            global_pos.z
        );
    }
}

/// Helper function to print the entity hierarchy
fn print_hierarchy(world: &World) {
    let inner = world.inner();

    // Find root entities (no parent)
    let mut root_query = inner.query::<(&Name, Option<&Children>)>();
    let mut parent_query = inner.query::<&Parent>();

    for (entity, (name, maybe_children)) in root_query.iter_with_entity(inner) {
        // Check if this entity has a parent
        if parent_query.get(inner, entity).is_ok() {
            continue;
        }

        println!("{}", name.as_str());
        if let Some(children) = maybe_children {
            print_children(inner, &children.0, 1);
        }
    }
}

/// Recursively print children with indentation
fn print_children(
    world: &bevy_ecs::world::World,
    children: &[bevy_ecs::entity::Entity],
    depth: usize,
) {
    let indent = "  ".repeat(depth);

    for &child in children {
        if let Some(name) = world.get::<Name>(child) {
            println!("{}└─ {}", indent, name.as_str());

            if let Some(children) = world.get::<Children>(child) {
                print_children(world, &children.0, depth + 1);
            }
        }
    }
}
