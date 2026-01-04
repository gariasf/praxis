# Praxis Scripting

Lua scripting system integration for the Praxis game engine.

## Features

- **Lua 5.4 Integration**: Full Lua scripting support via `mlua`
- **Hot-Reload**: Automatically reload scripts when files change
- **Sandboxing**: Configurable security levels (None/Moderate/Strict)
- **Performance Monitoring**: Track execution time to detect expensive operations
- **Math API**: Vector and math operations
- **Engine API**: Logging and utility functions

## Quick Start

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};

let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;

// Load and execute a script
context.load_script("game_logic", "scripts/game.lua")?;
context.call_function::<_, ()>("game_logic", "update", 0.016)?;

// Enable hot-reload
context.enable_hot_reload("scripts")?;
```

## API Reference

### Rust API

```rust
// Context creation
let context = ScriptingContext::new(config)?;

// Script loading
context.load_script(name, path)?;
context.load_string(name, source)?;

// Function calling
let result: T = context.call_function(script, function, args)?;

// Hot-reload
context.enable_hot_reload(path)?;
context.process_hot_reload()?;

// Performance
if let Some(monitor) = context.performance_monitor() {
    let stats = monitor.get_stats(script, function)?;
}
```

### Lua API

#### Math Operations

```lua
-- Vector creation
local v = math.Vec3(x, y, z)
local q = math.Quat(x, y, z, w)

-- Math functions
math.sqrt(x)
math.sin(x)
math.cos(x)
math.tan(x)
math.abs(x)

-- Constants
math.pi
math.tau
```

#### Engine Functions

```lua
engine.log_info("Message")
engine.log_debug("Debug")
engine.log_warn("Warning")
engine.log_error("Error")
```

## Sandboxing

Configure security restrictions:

```rust
use praxis_scripting::{SandboxConfig, SandboxLevel};

let config = ScriptingConfig {
    sandbox: SandboxConfig {
        level: SandboxLevel::Strict,
        allow_file_io: false,
        allow_network: false,
        allow_os_access: false,
    },
    ..Default::default()
};
```

**Sandbox Levels:**
- **None**: No restrictions
- **Moderate**: Disables dangerous operations (`dofile`, `loadfile`), restricts `os` and `io`
- **Strict**: Maximum restrictions, removes module loading

## Performance Monitoring

```rust
if let Some(monitor) = context.performance_monitor() {
    let stats = monitor.get_stats("game_logic", "update").unwrap();
    println!("Average execution time: {:?}", stats.average_time);
    
    // Get slowest scripts
    for stats in monitor.get_slowest_scripts().iter().take(5) {
        println!("{}: {:?}", stats.script_name, stats.average_time);
    }
}
```

Scripts exceeding the threshold generate warnings:

```
WARN Script 'game_logic::update' took 18.45ms (threshold: 16ms)
```

## Example Scripts

### Basic Script

```lua
function greet(name)
    return "Hello, " .. name .. "!"
end

function calculate_sum(a, b)
    return a + b
end
```

### Math Operations

```lua
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
```

### Lifecycle Methods

```lua
function on_start()
    engine.log_info("Script initialized!")
end

function on_update(delta_time)
    -- Update logic called every frame
end

function on_destroy()
    engine.log_info("Script destroyed!")
end
```

## Configuration

### ScriptingConfig

```rust
pub struct ScriptingConfig {
    pub sandbox: SandboxConfig,
    pub enable_performance_monitoring: bool,
    pub max_execution_time_ms: u64,
    pub memory_limit: usize,
}
```

### SandboxConfig

```rust
pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allow_file_io: bool,
    pub allow_network: bool,
    pub allow_os_access: bool,
}
```

## Performance Tips

1. **Cache values**: Store frequently accessed data in local variables
2. **Use local variables**: Lua locals are faster than globals
3. **Profile regularly**: Use the performance monitor to find bottlenecks
4. **Keep functions focused**: Smaller functions are easier to optimize

## Examples

Run the demo:

```bash
cargo run --example scripting_demo
```

## Documentation

- See `docs/guides/scripting.md` for comprehensive guide
- Run `cargo doc --open` for API documentation
- Check `examples/` for more examples

## Future Enhancements

- [ ] ECS World access from scripts
- [ ] Input system API
- [ ] Physics system API  
- [ ] Audio system API
- [ ] Visual scripting integration
