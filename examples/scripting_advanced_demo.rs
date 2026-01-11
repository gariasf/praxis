//! Advanced scripting demo with performance monitoring and complex scripts.
//!
//! This example demonstrates:
//! - Loading and executing multiple scripts
//! - Performance profiling of script execution
//! - Complex Lua logic with state management
//! - Script interaction with ECS World and engine APIs

#[cfg(feature = "scripting")]
use praxis_ecs::{Entity, Name, Transform, World};
#[cfg(feature = "scripting")]
use praxis_scripting::{mlua, SandboxConfig, SandboxLevel, ScriptingConfig, ScriptingContext};

#[cfg(feature = "scripting")]
fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_scripting::init()?;

    println!("=== Praxis Advanced Scripting Demo ===\n");

    let mut world = World::new();
    let mut context = setup_scripting_context()?;

    setup_test_entities(&mut world);
    load_movement_script(&mut context)?;

    println!("Running simulation for 10 frames...\n");

    for frame in 0..10 {
        let delta_time = 0.016_f32;

        // Call the update script with world access
        let result: Result<(), praxis_utils::eyre::Report> =
            context.with_world(&mut world, |lua| {
                // Get the on_update function and call it
                lua.globals()
                    .get::<_, mlua::Function>("on_update")
                    .and_then(|f| f.call::<_, ()>(delta_time))
                    .map_err(|e| praxis_utils::eyre::eyre!("Script error: {}", e))?;
                Ok(())
            });

        if let Err(e) = result {
            eprintln!("Error running script: {}", e);
        }

        if frame % 3 == 0 {
            print_entity_positions(&mut world);
        }
    }

    print_performance_stats(&context);

    println!("\n=== Demo Complete ===");

    Ok(())
}

#[cfg(feature = "scripting")]
fn setup_scripting_context() -> praxis_utils::Result<ScriptingContext> {
    let config = ScriptingConfig {
        sandbox: SandboxConfig {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 1_000_000,    // 1 million instructions
            memory_limit: 100 * 1024 * 1024, // 100 MB
        },
        enable_performance_monitoring: true,
        max_execution_time_ms: 16,
    };

    let context = ScriptingContext::new(config)?;

    println!("Scripting system configured:");
    println!("  - Sandbox: Moderate");
    println!("  - Performance monitoring: Enabled");
    println!("  - Max execution time: 16ms");
    println!("  - Instruction limit: 1,000,000");
    println!("  - Memory limit: 100 MB");
    println!();

    Ok(context)
}

#[cfg(feature = "scripting")]
fn setup_test_entities(world: &mut World) {
    println!("Creating test entities...\n");

    let player = world.spawn((Name::new("Player"), Transform::from_xyz(0.0, 0.0, 0.0)));
    println!("Created Player entity: {:?}", player);

    let enemy = world.spawn((Name::new("Enemy"), Transform::from_xyz(5.0, 0.0, 5.0)));
    println!("Created Enemy entity: {:?}", enemy);

    println!();
}

#[cfg(feature = "scripting")]
fn load_movement_script(context: &mut ScriptingContext) -> praxis_utils::Result<()> {
    context.load_string(
        "movement_script",
        r#"
        local entities = {}
        local initialized = false
        
        function on_update(delta_time)
            -- Initialize entity references on first update
            if not initialized then
                engine.log_info("Movement script started")
                entities.player = world.get_entity_by_name("Player")
                entities.enemy = world.get_entity_by_name("Enemy")
                initialized = true
            end
            
            -- Update player position
            if entities.player then
                local transform = world.get_component_transform(entities.player)
                transform.translation.x = transform.translation.x + 2.0 * delta_time
                world.set_component_transform(entities.player, transform)
            end
            
            -- Update enemy position (circular motion)
            if entities.enemy then
                local transform = world.get_component_transform(entities.enemy)
                local time = transform.translation.x * 0.5
                transform.translation.x = 5.0 + math.cos(time) * 3.0
                transform.translation.z = 5.0 + math.sin(time) * 3.0
                world.set_component_transform(entities.enemy, transform)
            end
        end
    "#,
    )?;

    println!("Loaded movement script\n");
    Ok(())
}

#[cfg(feature = "scripting")]
fn print_entity_positions(world: &mut World) {
    let mut query = world.inner_mut().query::<(Entity, &Name, &Transform)>();

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

#[cfg(feature = "scripting")]
fn print_performance_stats(context: &ScriptingContext) {
    if let Some(monitor) = context.performance_monitor() {
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

#[cfg(not(feature = "scripting"))]
fn main() {
    eprintln!("This example requires the 'scripting' feature to be enabled.");
    eprintln!("Run with: cargo run --example scripting_advanced_demo --features scripting");
    std::process::exit(1);
}
