# praxis_scripting

Lua scripting integration for Praxis engine.

## Overview

Embeds Lua 5.4 for game logic scripting with ECS integration, hot-reload, and sandboxing.

## Features

### Lua Integration

- Lua 5.4 via `mlua`
- Safe Rust-Lua bridge
- Type conversions
- Error handling

### ECS Access

- Query entities and components
- Modify component values
- Spawn and despawn entities
- Access resources

### Hot-Reload

- File watcher for script changes
- Automatic reload on save
- Preserve state (optional)

### Sandboxing

- Configurable security levels:
  - **None**: Full Lua standard library
  - **Moderate**: Limited I/O and OS access
  - **Strict**: Minimal safe subset
- Prevent malicious scripts

### Performance Monitoring

- Execution time tracking
- Warning on slow scripts
- Configurable time limits

## Example

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};

// Initialize
let config = ScriptingConfig {
    hot_reload: true,
    sandbox_level: SandboxLevel::Moderate,
    max_execution_time_ms: 16,
};

let mut context = ScriptingContext::new(config)?;

// Load script
context.load_script("game_logic", "scripts/game.lua")?;

// Enable hot-reload
context.enable_hot_reload("scripts")?;

// Execute
context.execute_script("game_logic", world)?;
```

## Lua Script Example

```lua
-- scripts/game.lua

function on_update(entities, dt)
    -- Query entities with Health component
    for _, entity in ipairs(entities:with("Health")) do
        local health = entity:get("Health")
        
        -- Regenerate health
        health.value = math.min(health.value + 10 * dt, 100)
        entity:set("Health", health)
        
        -- Check death
        if health.value <= 0 then
            entity:despawn()
        end
    end
end
```

## ECS Bridge

```rust
// Register component types
context.register_component::<Health>()?;
context.register_component::<Transform>()?;

// Expose systems
context.add_lua_function("spawn_enemy", |lua, position: Vec3| {
    // Spawn enemy logic
    Ok(entity_id)
})?;
```

## Security

```rust
let config = ScriptingConfig {
    sandbox_level: SandboxLevel::Strict,
    allow_file_access: false,
    allow_network: false,
    max_memory_mb: 64,
};
```

## Performance

```rust
// Set execution time limit
config.max_execution_time_ms = 16;

// Get performance stats
let stats = context.performance_stats();
println!("Avg execution: {:.2}ms", stats.avg_execution_time_ms);
```

## Dependencies

- `mlua`: Lua bindings
- `notify`: File watching
- `serde`: Serialization
- `rustc-hash`: Fast hash maps
- `parking_lot`: Fast mutexes

## Usage

```toml
# In root Cargo.toml
[features]
scripting = ["praxis_scripting"]

# In your crate
praxis_scripting = { path = "../praxis_scripting", version = "0.1.0" }
```
