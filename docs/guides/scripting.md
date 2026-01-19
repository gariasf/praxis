# Scripting System Guide

This guide covers the Praxis scripting system, which allows you to write runtime game logic in Lua with full access to the engine's ECS and APIs.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [ECS Integration](#ecs-integration)
4. [Script Lifecycle](#script-lifecycle)
5. [Hot-Reload](#hot-reload)
6. [Sandboxing](#sandboxing)
7. [Performance Monitoring](#performance-monitoring)
8. [API Reference](#api-reference)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Overview

The Praxis scripting system provides:

- **Lua 5.4** integration for runtime game logic
- **ECS World access** to manipulate entities and components
- **Hot-reload support** for rapid iteration during development
- **Sandboxing** to restrict dangerous operations
- **Performance monitoring** to detect expensive script operations
- **Type-safe bindings** for math operations (vectors, quaternions)

### Why Lua?

- Battle-tested in game engines (World of Warcraft, Roblox, etc.)
- Easy to learn syntax
- Fast execution via LuaJIT (future optimization)
- Strong embedding support in Rust via `mlua`

## Quick Start

### 1. Setup Scripting Context

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};

let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;
```

### 2. Load and Execute a Script

```rust
// From file
context.load_script("game_logic", "scripts/game.lua")?;

// From string
context.load_string("test", "x = 42")?;
```

### 3. Call Lua Functions

```rust
// Call a function with arguments
let result: String = context.call_function("game_logic", "greet", "Player")?;

// Call with multiple arguments
let sum: i32 = context.call_function("math", "add", (5, 3))?;
```

### 4. Access ECS World

```rust
context.with_world(&mut world, |lua| {
    lua.load(r#"
        local player = world.get_entity_by_name("Player")
        local transform = world.get_component_transform(player)
        transform.translation.x = 10.0
        world.set_component_transform(player, transform)
    "#).exec()?;
    Ok(())
})?;
```

## ECS Integration

### Accessing Entities

```lua
-- Find entity by name
local player = world.get_entity_by_name("Player")

-- Spawn new entity
local enemy = world.spawn()

-- Despawn entity
world.despawn(enemy)
```

### Component Operations

```lua
-- Add components
world.add_component_transform(entity, 0.0, 5.0, 0.0)
world.add_component_name(entity, "MyEntity")

-- Get components
local transform = world.get_component_transform(entity)
local name = world.get_component_name(entity)

-- Modify and set components
transform.translation.x = transform.translation.x + 5.0
world.set_component_transform(entity, transform)
```

### Transform Component

The Transform component has the following structure in Lua:

```lua
transform = {
    translation = { x = 0.0, y = 0.0, z = 0.0 }
    -- rotation and scale coming soon
}
```

## Script Lifecycle

Scripts attached to entities can implement lifecycle methods:

### on_start()

Called once when the entity is first spawned or the script is initialized.

```lua
function on_start()
    engine.log_info("Entity initialized")
    -- Cache references, initialize state
end
```

### on_update(delta_time)

Called every frame with the time elapsed since the last frame (in seconds).

```lua
function on_update(delta_time)
    -- Update logic
    local speed = 5.0
    transform.translation.x = transform.translation.x + speed * delta_time
end
```

### on_destroy()

Called when the entity is destroyed.

```lua
function on_destroy()
    engine.log_info("Entity destroyed")
    -- Cleanup resources
end
```

## Hot-Reload

Enable hot-reload to automatically reload scripts when files change:

```rust
context.enable_hot_reload("scripts")?;

// In your game loop
context.process_hot_reload()?;
```

When a `.lua` file in the watched directory changes:
1. The file is automatically reloaded
2. The script's Lua environment is preserved (global variables persist)
3. Functions are redefined with the new code
4. No need to restart the game!

### Hot-Reload Best Practices

- **Test frequently**: Save scripts often to see changes immediately
- **Use local variables**: Globals persist across reloads
- **Avoid stateful globals**: Use entity components for persistent state
- **Watch for errors**: Check logs for syntax/runtime errors after reload

## Sandboxing

Configure security restrictions to prevent malicious or accidental damage:

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

### Sandbox Levels

**None**
- No restrictions
- Full access to all Lua features
- Use only for trusted scripts

**Moderate** (default)
- Disables `dofile`, `loadfile`, `load`
- Restricts `os` module to safe functions (clock, date, time)
- Removes `io` module if file I/O not allowed
- Good balance for development

**Strict**
- All Moderate restrictions
- Removes `require` and `package` module
- Maximum security for untrusted scripts

### Custom Restrictions

```rust
let config = SandboxConfig {
    level: SandboxLevel::Moderate,
    allow_file_io: false,      // Block file operations
    allow_network: false,      // Block network operations (future)
    allow_os_access: false,    // Restrict OS module
};
```

## Performance Monitoring

Track script execution time to identify performance bottlenecks:

```rust
let config = ScriptingConfig {
    enable_performance_monitoring: true,
    max_execution_time_ms: 16,  // Warning threshold
    ..Default::default()
};
```

### Accessing Statistics

```rust
if let Some(monitor) = context.performance_monitor() {
    // Get stats for specific function
    let stats = monitor.get_stats("player_script", "update").unwrap();
    println!("Average time: {:?}", stats.average_time);
    
    // Get slowest scripts
    for stats in monitor.get_slowest_scripts().iter().take(5) {
        println!("{}: {:?}", stats.script_name, stats.total_time);
    }
    
    // Reset statistics
    monitor.reset();
}
```

### Warning Threshold

Scripts exceeding the threshold generate warnings:

```
WARN Script 'player_script::update' took 18.45ms (threshold: 16ms)
```

### Performance Tips

1. **Cache entity references** in `on_start()` instead of querying every frame
2. **Use local variables** - they're faster than globals
3. **Minimize ECS queries** - batch component access when possible
4. **Profile regularly** - use the performance monitor to find hotspots
5. **Optimize hot paths** - move expensive calculations out of `on_update`

## API Reference

### World API

| Function | Description |
|----------|-------------|
| `world.spawn()` | Create a new entity |
| `world.despawn(entity)` | Destroy an entity |
| `world.get_entity_by_name(name)` | Find entity by name |
| `world.add_component_transform(entity, x, y, z)` | Add Transform component |
| `world.add_component_name(entity, name)` | Add Name component |
| `world.get_component_transform(entity)` | Get Transform component |
| `world.set_component_transform(entity, transform)` | Update Transform |
| `world.get_component_name(entity)` | Get entity name |

### Math API

#### Vectors

```lua
local v = math.Vec3(x, y, z)

-- Fields
v.x, v.y, v.z

-- Methods
v:length()           -- Get magnitude
v:normalize()        -- Get unit vector
v:dot(other)         -- Dot product
v:cross(other)       -- Cross product

-- Operators
v1 + v2              -- Addition
v1 - v2              -- Subtraction
v * scalar           -- Scalar multiplication
```

#### Quaternions

```lua
local q = math.Quat(x, y, z, w)

-- Creation
local q = math.Quat.from_rotation_x(angle)
local q = math.Quat.from_rotation_y(angle)
local q = math.Quat.from_rotation_z(angle)

-- Fields
q.x, q.y, q.z, q.w

-- Methods
q:normalize()        -- Get normalized quaternion

-- Operators
q1 * q2              -- Quaternion multiplication
```

#### Constants

```lua
math.pi              -- 3.14159...
math.tau             -- 2 * pi
```

### Engine API

| Function | Description |
|----------|-------------|
| `engine.log_info(message)` | Log info message |
| `engine.log_debug(message)` | Log debug message |
| `engine.log_warn(message)` | Log warning |
| `engine.log_error(message)` | Log error |

## Best Practices

### 1. Organize Scripts by Responsibility

```
scripts/
  player/
    controller.lua
    inventory.lua
  enemies/
    ai.lua
    spawner.lua
  systems/
    collision.lua
    scoring.lua
```

### 2. Cache Entity References

**Good:**
```lua
local player_entity = nil

function on_start()
    player_entity = world.get_entity_by_name("Player")
end

function on_update(dt)
    if player_entity then
        local transform = world.get_component_transform(player_entity)
        -- Use transform
    end
end
```

**Bad:**
```lua
function on_update(dt)
    -- Don't query every frame!
    local player_entity = world.get_entity_by_name("Player")
    -- ...
end
```

### 3. Use Local Variables

Lua locals are significantly faster than globals:

```lua
-- Good
local speed = 5.0
local function update_position(dt)
    local dx = speed * dt
    -- ...
end

-- Bad
speed = 5.0  -- Global
function update_position(dt)  -- Global function
    dx = speed * dt  -- Global dx
end
```

### 4. Handle Missing Entities Gracefully

```lua
function on_update(dt)
    local entity = world.get_entity_by_name("Enemy")
    if not entity then
        return  -- Entity doesn't exist yet or was destroyed
    end
    
    -- Safe to use entity
end
```

### 5. Use Descriptive Names

```lua
-- Good
local move_speed = 5.0
local rotation_speed = 2.0

function update_player_movement(dt)
    -- Clear purpose
end

-- Bad
local s = 5.0
local r = 2.0

function upd(dt)
    -- What does this do?
end
```

## Examples

### Player Controller

```lua
local move_speed = 5.0
local player_entity = nil

function on_start()
    player_entity = world.get_entity_by_name("Player")
    if not player_entity then
        engine.log_error("Player entity not found!")
    end
end

function on_update(delta_time)
    if not player_entity then return end
    
    local transform = world.get_component_transform(player_entity)
    
    -- Simple forward movement
    transform.translation.x = transform.translation.x + move_speed * delta_time
    
    world.set_component_transform(player_entity, transform)
end
```

### Enemy Patrol AI

```lua
local patrol_points = {
    {x = 0, z = 0},
    {x = 10, z = 0},
    {x = 10, z = 10},
    {x = 0, z = 10}
}

local current_point = 1
local enemy_entity = nil
local patrol_speed = 3.0

function on_start()
    enemy_entity = world.get_entity_by_name("Enemy")
end

function on_update(delta_time)
    if not enemy_entity then return end
    
    local transform = world.get_component_transform(enemy_entity)
    local target = patrol_points[current_point]
    
    local dx = target.x - transform.translation.x
    local dz = target.z - transform.translation.z
    local distance = math.sqrt(dx * dx + dz * dz)
    
    if distance < 0.5 then
        current_point = (current_point % #patrol_points) + 1
    else
        local dir_x = dx / distance
        local dir_z = dz / distance
        
        transform.translation.x = transform.translation.x + dir_x * patrol_speed * delta_time
        transform.translation.z = transform.translation.z + dir_z * patrol_speed * delta_time
        
        world.set_component_transform(enemy_entity, transform)
    end
end
```

### Dynamic Spawner

```lua
local spawn_timer = 0
local spawn_interval = 2.0
local spawn_count = 0
local max_spawns = 10

function on_update(delta_time)
    spawn_timer = spawn_timer + delta_time
    
    if spawn_timer >= spawn_interval and spawn_count < max_spawns then
        spawn_entity()
        spawn_timer = 0
        spawn_count = spawn_count + 1
    end
end

function spawn_entity()
    local entity = world.spawn()
    
    local angle = math.random() * 2 * math.pi
    local radius = math.random() * 10
    local x = math.cos(angle) * radius
    local z = math.sin(angle) * radius
    
    world.add_component_transform(entity, x, 2.0, z)
    world.add_component_name(entity, "Spawned_" .. spawn_count)
    
    engine.log_info("Spawned entity at (" .. x .. ", 2.0, " .. z .. ")")
end
```

## Troubleshooting

### Script Not Loading

**Error:** `Failed to read script file`

**Solution:** Check that the file path is correct and the file exists. Paths are relative to the working directory.

### Function Not Found

**Error:** `Function 'update' not found`

**Solution:** Ensure the function is defined globally in the script, not as a local function.

### Component Not Found

**Error:** `Entity does not have Transform component`

**Solution:** Verify the entity has the component before accessing it:

```lua
if entity then
    local transform = world.get_component_transform(entity)
    if transform then
        -- Safe to use
    end
end
```

### Performance Warnings

**Warning:** `Script 'player::update' took 25ms (threshold: 16ms)`

**Solution:**
1. Profile the script to find expensive operations
2. Move heavy calculations outside the update loop
3. Cache computed values when possible
4. Consider moving logic to Rust systems

## See Also

- [Scripting API Reference](../reference/scripting-api.md) - API documentation
- [Scripting Learning Path](../learning-paths/scripting.md) - Structured learning progression
- [Console Guide](console.md) - In-game console with Lua REPL
- [praxis_scripting Crate](../../crates/praxis_scripting/README.md) - Crate documentation

## Examples

Run the scripting examples to see the system in action:

```bash
cargo run --example scripting_demo           # Basic scripting
cargo run --example scripting_advanced_demo  # Advanced features
cargo run --example scripting_console_demo   # Console with Lua REPL
```
