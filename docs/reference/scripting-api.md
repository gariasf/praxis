# Scripting API Reference

API reference for Lua scripting integration in Praxis.

## Core Types

### ScriptingContext

Main interface for Lua scripting.

```rust
pub struct ScriptingContext { /* ... */ }
```

**Methods:**
- `new(config: ScriptingConfig) -> Result<Self>`
- `load_script(name: &str, path: &str) -> Result<()>` - Load script from file
- `load_script_string(name: &str, code: &str) -> Result<()>` - Load from string
- `call_function<A, R>(script: &str, func: &str, args: A) -> Result<R>`
- `execute(code: &str) -> Result<()>` - Execute Lua code
- `enable_hot_reload(watch_dir: &str) -> Result<()>`
- `disable_hot_reload()`
- `process_hot_reload() -> Result<Vec<String>>` - Returns reloaded scripts
- `get_global<T>(name: &str) -> Result<T>`
- `set_global<T>(name: &str, value: T) -> Result<()>`

### ScriptingConfig

Configuration for scripting engine.

```rust
pub struct ScriptingConfig {
    pub sandbox: SandboxConfig,
    pub hot_reload: bool,
    pub performance_monitoring: bool,
    pub slow_script_threshold_ms: f32,  // Default: 16.0
}
```

**Methods:**
- `default()` - Standard configuration
- `with_sandbox(level: SandboxLevel)` - Set security level
- `with_hot_reload(enabled: bool)`

### SandboxConfig

Security and resource limits.

```rust
pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allow_file_io: bool,
    pub allow_network: bool,
    pub instruction_limit: usize,        // Max instructions per call
    pub memory_limit: usize,             // Max memory in bytes
    pub execution_timeout_ms: u64,       // Max execution time
}
```

### SandboxLevel

Security levels for Lua scripts.

```rust
pub enum SandboxLevel {
    None,      // All Lua standard library available
    Moderate,  // Disable dangerous functions (io, os, debug)
    Strict,    // Only safe subset (math, string, table)
}
```

## ECS Bridge

### World Access (Lua Side)

Functions available to Lua scripts for ECS interaction.

```lua
-- Entity Management
entity = world.spawn()
world.despawn(entity)
entities = world.get_all_entities()

-- Component Queries
entity = world.get_entity_by_name("Player")
has = world.has_component_transform(entity)
has = world.has_component_name(entity)

-- Transform Component
transform = world.get_component_transform(entity)
-- Returns: { translation = {x, y, z}, rotation = {x, y, z, w}, scale = {x, y, z} }
world.set_component_transform(entity, transform)
world.add_component_transform(entity, x, y, z)

-- Name Component
name = world.get_component_name(entity)
world.set_component_name(entity, "NewName")
world.add_component_name(entity, "EntityName")

-- Velocity Component (if physics enabled)
velocity = world.get_component_velocity(entity)
world.set_component_velocity(entity, vx, vy, vz)
```

### Custom Bindings

Register custom Rust functions for Lua.

```rust
use praxis_scripting::ScriptingContext;

context.register_function("damage_entity", |entity: u64, amount: f32| {
    // Custom logic
    Ok(())
})?;
```

From Lua:
```lua
damage_entity(player_entity, 10.0)
```

## Hot Reload

### Setup

```rust
context.enable_hot_reload("scripts")?;
```

### Update Loop

```rust
fn script_hot_reload_system(
    mut context: ResMut<ScriptingContext>,
) {
    if let Ok(reloaded) = context.process_hot_reload() {
        for script in reloaded {
            info!("Reloaded: {}", script);
        }
    }
}
```

## Performance Monitoring

### ScriptMetrics

Performance data for script execution.

```rust
pub struct ScriptMetrics {
    pub execution_count: u64,
    pub total_time_ms: f64,
    pub average_time_ms: f64,
    pub max_time_ms: f64,
    pub slow_execution_count: u64,
}
```

**Access:**
```rust
let metrics = context.get_metrics("game_logic")?;
println!("Average: {:.2}ms", metrics.average_time_ms);
```

## REPL Support

### Interactive Console

```rust
use praxis_scripting::repl::LuaRepl;

let mut repl = LuaRepl::new(context)?;
repl.run()?;  // Starts interactive session
```

**Commands:**
- `.help` - Show help
- `.clear` - Clear screen
- `.exit` - Exit REPL
- Any Lua code - Execute and print result

## Common Patterns

### Basic Script Loading

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};

let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;
context.load_script("game_logic", "scripts/game.lua")?;

world.insert_resource(context);
```

### Calling Lua Functions

```rust
fn update_scripts(
    mut context: ResMut<ScriptingContext>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    
    // Call update() function in script
    if let Err(e) = context.call_function::<_, ()>("game_logic", "update", delta) {
        error!("Script error: {}", e);
    }
}
```

### Sandboxed Script

```rust
use praxis_scripting::{SandboxConfig, SandboxLevel};

let config = ScriptingConfig {
    sandbox: SandboxConfig {
        level: SandboxLevel::Strict,
        allow_file_io: false,
        allow_network: false,
        instruction_limit: 1_000_000,
        memory_limit: 100 * 1024 * 1024,  // 100 MB
        execution_timeout_ms: 100,
    },
    ..Default::default()
};
```

### Hot Reload with Monitoring

```rust
fn script_system(
    mut context: ResMut<ScriptingContext>,
    time: Res<Time>,
) {
    // Check for reloaded scripts
    if let Ok(reloaded) = context.process_hot_reload() {
        for script in reloaded {
            info!("Reloaded: {}", script);
            
            // Re-initialize reloaded script
            let _ = context.call_function::<_, ()>(&script, "init", ());
        }
    }
    
    // Update scripts
    let _ = context.call_function::<_, ()>("game_logic", "update", time.delta_seconds());
    
    // Check performance
    if let Ok(metrics) = context.get_metrics("game_logic") {
        if metrics.slow_execution_count > 0 {
            warn!("Script 'game_logic' had {} slow frames (>{:.1}ms)", 
                metrics.slow_execution_count, context.config().slow_script_threshold_ms);
        }
    }
}
```

## Lua Script Examples

### Player Controller

```lua
-- scripts/player.lua

function init()
    player = world.get_entity_by_name("Player")
    speed = 5.0
end

function update(delta_time)
    -- Get current transform
    local transform = world.get_component_transform(player)
    
    -- Calculate movement
    local dx = 0
    local dz = 0
    
    if input.is_key_pressed("W") then
        dz = -speed * delta_time
    end
    if input.is_key_pressed("S") then
        dz = speed * delta_time
    end
    if input.is_key_pressed("A") then
        dx = -speed * delta_time
    end
    if input.is_key_pressed("D") then
        dx = speed * delta_time
    end
    
    -- Update position
    transform.translation.x = transform.translation.x + dx
    transform.translation.z = transform.translation.z + dz
    
    -- Write back
    world.set_component_transform(player, transform)
end
```

### Enemy AI

```lua
-- scripts/enemy_ai.lua

function update_enemy(enemy_entity, delta_time)
    local enemy_transform = world.get_component_transform(enemy_entity)
    local player = world.get_entity_by_name("Player")
    local player_transform = world.get_component_transform(player)
    
    -- Calculate distance
    local dx = player_transform.translation.x - enemy_transform.translation.x
    local dz = player_transform.translation.z - enemy_transform.translation.z
    local distance = math.sqrt(dx * dx + dz * dz)
    
    -- Chase player if close enough
    if distance < 10.0 then
        local speed = 2.0
        enemy_transform.translation.x = enemy_transform.translation.x + (dx / distance) * speed * delta_time
        enemy_transform.translation.z = enemy_transform.translation.z + (dz / distance) * speed * delta_time
        
        world.set_component_transform(enemy_entity, enemy_transform)
    end
end
```

## See Also

- [Scripting Guide](../guides/scripting.md) - Comprehensive scripting guide
- [Scripting Learning Path](../learning-paths/scripting.md) - Step-by-step tutorials
- [praxis_scripting crate](../../crates/praxis_scripting/README.md) - Crate documentation
