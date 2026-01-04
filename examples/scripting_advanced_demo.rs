//! Advanced scripting demo with ECS systems and hot-reload.
//!
//! This example demonstrates:
//! - Using scripting systems with the ECS scheduler
//! - Hot-reload support for script files
//! - Script components attached to entities
//! - Performance profiling of scripts

use praxis_ecs::{
    Commands, DeltaTime, Entity, IntoSystemConfigs, Name, Query, Schedule, Transform, World,
};
use praxis_math::Vec3;
use praxis_scripting::{
    script_hot_reload_system, script_initialization_system, script_start_system,
    script_update_system, SandboxConfig, SandboxLevel, ScriptComponent, ScriptingConfig,
    ScriptingContext, ScriptingResource,
};
use std::time::Duration;

fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_scripting::init()?;

    println!("=== Praxis Advanced Scripting Demo ===\n");

    let mut world = World::new();
    let mut schedule = Schedule::default();

    setup_scripting_system(&mut world);
    setup_systems(&mut schedule);
    setup_test_entities(&mut world);

    println!("Running simulation for 10 frames...\n");

    for frame in 0..10 {
        world.resource_mut::<DeltaTime>().0 = 0.016;
        schedule.run(world.inner_mut());

        if frame % 3 == 0 {
            print_entity_positions(&world);
        }
    }

    print_performance_stats(&world);

    println!("\n=== Demo Complete ===");

    Ok(())
}

fn setup_scripting_system(world: &mut World) {
    let config = ScriptingConfig {
        sandbox: SandboxConfig {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
        },
        enable_performance_monitoring: true,
        max_execution_time_ms: 16,
        memory_limit: 100 * 1024 * 1024,
    };

    let context = ScriptingContext::new(config).unwrap();
    world.insert_resource(ScriptingResource::new(context));
    world.insert_resource(DeltaTime(0.016));

    println!("Scripting system configured:");
    println!("  - Sandbox: Moderate");
    println!("  - Performance monitoring: Enabled");
    println!("  - Max execution time: 16ms");
    println!();
}

fn setup_systems(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            script_hot_reload_system,
            script_initialization_system,
            script_start_system,
            script_update_system,
        )
            .chain(),
    );
}

fn setup_test_entities(world: &mut World) {
    println!("Creating test entities with script components...\n");

    let player = world.spawn((
        Name::new("Player"),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    println!("Created Player entity: {:?}", player);

    let enemy = world.spawn((
        Name::new("Enemy"),
        Transform::from_xyz(5.0, 0.0, 5.0),
    ));
    println!("Created Enemy entity: {:?}", enemy);

    let scripting = world.resource::<ScriptingResource>();

    scripting
        .context()
        .load_string(
            "movement_script",
            r#"
        local entities = {}
        
        function on_start()
            engine.log_info("Movement script started")
            
            -- Cache entity references
            entities.player = world.get_entity_by_name("Player")
            entities.enemy = world.get_entity_by_name("Enemy")
        end
        
        function on_update(delta_time)
            if entities.player then
                local transform = world.get_component_transform(entities.player)
                transform.translation.x = transform.translation.x + 2.0 * delta_time
                world.set_component_transform(entities.player, transform)
            end
            
            if entities.enemy then
                local transform = world.get_component_transform(entities.enemy)
                -- Enemy moves in a circle
                local time = transform.translation.x * 0.5
                transform.translation.x = 5.0 + math.cos(time) * 3.0
                transform.translation.z = 5.0 + math.sin(time) * 3.0
                world.set_component_transform(entities.enemy, transform)
            end
        end
    "#,
        )
        .unwrap();

    println!("Loaded movement script\n");
}

fn print_entity_positions(world: &World) {
    let mut query = world
        .inner_mut()
        .query::<(Entity, &Name, &Transform)>();

    println!("Entity positions:");
    for (entity, name, transform) in query.iter(world.inner()) {
        println!(
            "  {:?} ({}): ({:.2}, {:.2}, {:.2})",
            entity,
            name.as_str(),
            transform.translation.x,
            transform.translation.y,
            transform.translation.z
        );
    }
    println!();
}

fn print_performance_stats(world: &World) {
    let scripting = world.resource::<ScriptingResource>();

    if let Some(monitor) = scripting.context().performance_monitor() {
        println!("\n=== Performance Statistics ===");
        println!(
            "{:<30} {:<15} {:<15} {:<10}",
            "Script", "Avg Time", "Max Time", "Calls"
        );
        println!("{}", "-".repeat(70));

        for stats in monitor.get_all_stats() {
            let name = format!(
                "{}::{}",
                stats.script_name,
                stats.function_name.unwrap_or_default()
            );
            println!(
                "{:<30} {:<15.3?} {:<15.3?} {:<10}",
                name, stats.average_time, stats.max_time, stats.execution_count
            );
        }

        if monitor.get_all_stats().iter().any(|s| s.warning_count > 0) {
            println!("\nWarnings:");
            for stats in monitor.get_all_stats() {
                if stats.warning_count > 0 {
                    println!(
                        "  - {}: {} warnings (exceeded threshold {} times)",
                        stats.script_name, stats.warning_count, stats.warning_count
                    );
                }
            }
        }
    }
}
