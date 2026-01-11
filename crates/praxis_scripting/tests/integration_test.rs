//! Integration tests for the scripting system.

use praxis_scripting::{SandboxConfig, SandboxLevel, ScriptingConfig, ScriptingContext};

#[test]
fn test_basic_lua_execution() {
    let config = ScriptingConfig::default();
    let mut context = ScriptingContext::new(config).unwrap();

    context.load_string("test", "x = 42").unwrap();

    let value: i32 = context.get_global("x").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_function_call() {
    let config = ScriptingConfig::default();
    let mut context = ScriptingContext::new(config).unwrap();

    context
        .load_string("test", "function add(a, b) return a + b end")
        .unwrap();

    let result: i32 = context.call_function("test", "add", (5, 3)).unwrap();
    assert_eq!(result, 8);
}

#[test]
fn test_math_api() {
    let config = ScriptingConfig::default();
    let mut context = ScriptingContext::new(config).unwrap();

    context
        .load_string(
            "test",
            r#"
        function test_vec3()
            local v = math.Vec3(1, 2, 3)
            return v.x, v.y, v.z
        end
        
        function test_sqrt()
            return math.sqrt(16)
        end
    "#,
        )
        .unwrap();

    let (x, y, z): (f32, f32, f32) = context.call_function("test", "test_vec3", ()).unwrap();
    assert_eq!(x, 1.0);
    assert_eq!(y, 2.0);
    assert_eq!(z, 3.0);

    let sqrt_val: f32 = context.call_function("test", "test_sqrt", ()).unwrap();
    assert_eq!(sqrt_val, 4.0);
}

#[test]
fn test_sandbox_moderate() {
    let config = ScriptingConfig {
        sandbox: SandboxConfig {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 1_000_000,
            memory_limit: 100 * 1024 * 1024,
        },
        ..Default::default()
    };

    let context = ScriptingContext::new(config).unwrap();

    // Should fail - io is disabled
    let result = context.lua().load("io.open('test.txt')").exec();
    assert!(result.is_err());
}

#[test]
fn test_sandbox_strict() {
    let config = ScriptingConfig {
        sandbox: SandboxConfig {
            level: SandboxLevel::Strict,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 1_000_000,
            memory_limit: 100 * 1024 * 1024,
        },
        ..Default::default()
    };

    let context = ScriptingContext::new(config).unwrap();

    // Should fail - require is disabled
    let result = context.lua().load("require('test')").exec();
    assert!(result.is_err());
}

#[test]
fn test_performance_monitoring() {
    let config = ScriptingConfig {
        enable_performance_monitoring: true,
        max_execution_time_ms: 10,
        ..Default::default()
    };

    let mut context = ScriptingContext::new(config).unwrap();

    context
        .load_string(
            "test",
            r#"
        function slow_function()
            local sum = 0
            for i = 1, 1000 do
                sum = sum + i
            end
            return sum
        end
    "#,
        )
        .unwrap();

    // Call function multiple times
    for _ in 0..5 {
        let _: i32 = context.call_function("test", "slow_function", ()).unwrap();
    }

    // Check performance stats
    let monitor = context.performance_monitor().unwrap();
    let stats = monitor.get_stats("test", "slow_function").unwrap();

    assert_eq!(stats.execution_count, 5);
    assert!(stats.average_time.as_nanos() > 0);
    assert!(stats.min_time <= stats.average_time);
    assert!(stats.max_time >= stats.average_time);
}

#[test]
fn test_engine_logging() {
    let config = ScriptingConfig::default();
    let mut context = ScriptingContext::new(config).unwrap();

    // Should not panic
    context
        .load_string(
            "test",
            r#"
        engine.log_info("Info message")
        engine.log_debug("Debug message")
        engine.log_warn("Warning message")
    "#,
        )
        .unwrap();
}

#[test]
fn test_string_operations() {
    let config = ScriptingConfig::default();
    let mut context = ScriptingContext::new(config).unwrap();

    context
        .load_string(
            "test",
            r#"
        function concat_strings(a, b)
            return a .. " " .. b
        end
        
        function string_length(s)
            return string.len(s)
        end
    "#,
        )
        .unwrap();

    let result: String = context
        .call_function("test", "concat_strings", ("Hello", "World"))
        .unwrap();
    assert_eq!(result, "Hello World");

    let length: i32 = context
        .call_function("test", "string_length", "test")
        .unwrap();
    assert_eq!(length, 4);
}

#[test]
fn test_table_operations() {
    let config = ScriptingConfig::default();
    let mut context = ScriptingContext::new(config).unwrap();

    context
        .load_string(
            "test",
            r#"
        function create_table()
            return {x = 10, y = 20, z = 30}
        end
        
        function sum_table_values(t)
            local sum = 0
            for k, v in pairs(t) do
                sum = sum + v
            end
            return sum
        end
    "#,
        )
        .unwrap();

    let table: mlua::Table = context.call_function("test", "create_table", ()).unwrap();
    let x: i32 = table.get("x").unwrap();
    let y: i32 = table.get("y").unwrap();
    let z: i32 = table.get("z").unwrap();

    assert_eq!(x, 10);
    assert_eq!(y, 20);
    assert_eq!(z, 30);
}
