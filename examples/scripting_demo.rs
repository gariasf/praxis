//! Demonstrates the scripting system with Lua integration.
//!
//! This example shows:
//! - Loading and executing Lua scripts
//! - Hot-reload support for rapid iteration
//! - Performance monitoring
//! - Sandboxing for security

#[cfg(feature = "scripting")]
use praxis_scripting::{SandboxConfig, SandboxLevel, ScriptingConfig, ScriptingContext};

#[cfg(feature = "scripting")]
fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_scripting::init()?;

    println!("=== Praxis Scripting Demo ===\n");

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

    let mut context = ScriptingContext::new(config)?;

    demo_basic_scripting(&mut context)?;
    demo_math_api(&mut context)?;
    demo_performance_monitoring(&mut context)?;

    println!("\n=== Demo Complete ===");

    Ok(())
}

#[cfg(feature = "scripting")]
fn demo_basic_scripting(context: &mut ScriptingContext) -> praxis_utils::Result<()> {
    println!("--- Basic Scripting ---");

    context.load_string(
        "basic",
        r#"
        function greet(name)
            return "Hello, " .. name .. "!"
        end
        
        function add(a, b)
            return a + b
        end
    "#,
    )?;

    let result: String = context.call_function("basic", "greet", "World")?;
    println!("Script returned: {}", result);

    let sum: i32 = context.call_function("basic", "add", (5, 3))?;
    println!("5 + 3 = {}", sum);

    Ok(())
}

#[cfg(feature = "scripting")]
fn demo_math_api(context: &mut ScriptingContext) -> praxis_utils::Result<()> {
    println!("\n--- Math API ---");

    context.load_string(
        "math_test",
        r#"
        function calculate_distance(x1, y1, z1, x2, y2, z2)
            local dx = x2 - x1
            local dy = y2 - y1
            local dz = z2 - z1
            return math.sqrt(dx*dx + dy*dy + dz*dz)
        end
        
        function create_vector()
            local v = math.Vec3(3, 4, 0)
            return v.x, v.y, v.z
        end
    "#,
    )?;

    let distance: f32 = context.call_function(
        "math_test",
        "calculate_distance",
        (0.0, 0.0, 0.0, 3.0, 4.0, 0.0),
    )?;
    println!("Distance between (0,0,0) and (3,4,0): {}", distance);

    let (x, y, z): (f32, f32, f32) = context.call_function("math_test", "create_vector", ())?;
    println!("Vector created: ({}, {}, {})", x, y, z);

    Ok(())
}

#[cfg(feature = "scripting")]
fn demo_performance_monitoring(context: &mut ScriptingContext) -> praxis_utils::Result<()> {
    println!("\n--- Performance Monitoring ---");

    context.load_string(
        "perf_test",
        r#"
        function fast_function()
            local sum = 0
            for i = 1, 100 do
                sum = sum + i
            end
            return sum
        end
        
        function slow_function()
            local sum = 0
            for i = 1, 100000 do
                sum = sum + i
            end
            return sum
        end
    "#,
    )?;

    for _ in 0..5 {
        let _: i32 = context.call_function("perf_test", "fast_function", ())?;
    }

    for _ in 0..3 {
        let _: i64 = context.call_function("perf_test", "slow_function", ())?;
    }

    if let Some(monitor) = context.performance_monitor() {
        println!("\nPerformance Statistics:");
        println!(
            "{:<40} {:<15} {:<15} {:<15}",
            "Function", "Avg Time", "Min Time", "Max Time"
        );
        println!("{}", "-".repeat(85));

        for stats in monitor.get_all_stats() {
            let func_name = format!(
                "{}::{}",
                stats.script_name,
                stats.function_name.unwrap_or_default()
            );
            println!(
                "{:<40} {:<15.3?} {:<15.3?} {:<15.3?}",
                func_name, stats.average_time, stats.min_time, stats.max_time
            );
        }

        if !monitor.get_slowest_scripts().is_empty() {
            println!("\nSlowest Scripts:");
            for stats in monitor.get_slowest_scripts().iter().take(3) {
                println!(
                    "  - {}: {:.3?} total ({} calls)",
                    stats.script_name, stats.total_time, stats.execution_count
                );
            }
        }
    }

    Ok(())
}

#[cfg(not(feature = "scripting"))]
fn main() {
    eprintln!("This example requires the 'scripting' feature to be enabled.");
    eprintln!("Run with: cargo run --example scripting_demo --features scripting");
    std::process::exit(1);
}
