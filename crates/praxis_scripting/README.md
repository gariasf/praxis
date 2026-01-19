# Praxis Scripting

Lua 5.4 scripting integration for the Praxis game engine.

## Overview

Full ECS access from Lua with hot-reload, sandboxing, and performance monitoring.

**Key Features:**
- Lua 5.4 via mlua
- ECS entity/component queries and modifications
- Hot-reload with file watching
- Configurable sandboxing (None/Moderate/Strict)
- Performance monitoring and execution limits
- REPL support for debugging

## Quick Start

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};
use color_eyre::Result;

fn main() -> Result<()> {
    // Initialize scripting context with default configuration
    let mut context = ScriptingContext::new(ScriptingConfig::default())?;
    
    // Load and execute a script
    context.load_script("game_logic", "scripts/game.lua")?;
    
    // Call a function in the loaded script
    context.call_function::<_, ()>("game_logic", "update", 0.016)?;
    
    // Enable hot-reload for script directory
    context.enable_hot_reload("scripts")?;
    
    // Process hot-reload events
    context.process_hot_reload()?;
    
    Ok(())
}
```

## Lua ECS Access

```lua
-- Spawn a new entity
local entity = world.spawn()

-- Add components by name
world.add_component_name(entity, "Player")

-- Add Transform component with position
world.add_component_transform(entity, 0, 0, 0)

-- Query entities by name
local player = world.get_entity_by_name("Player")

-- Retrieve component data
local transform = world.get_component_transform(player)
print("Player at", transform.translation.x, transform.translation.y)

-- Modify and update components
transform.translation.x = transform.translation.x + 1.0
world.set_component_transform(player, transform)
```

## Sandboxing

```rust
use praxis_scripting::{ScriptingConfig, SandboxConfig, SandboxLevel};
use color_eyre::Result;

fn setup_secure_scripting() -> Result<ScriptingContext> {
    let config = ScriptingConfig {
        sandbox: SandboxConfig {
            level: SandboxLevel::Strict,
            allow_file_io: false,
            instruction_limit: 1_000_000,  // Prevent infinite loops
            memory_limit: 100 * 1024 * 1024, // 100 MB
        },
        ..Default::default()
    };
    
    ScriptingContext::new(config)
}
```

## Documentation

**Comprehensive Guide:**
- [Scripting Guide](../../docs/guides/scripting.md) - Complete usage, patterns, security

**Learning Path:**
- [Scripting Learning Path](../../docs/learning-paths/scripting.md)

## Examples

```bash
# Basic scripting demo
cargo run --example scripting_demo

# Advanced scripting features
cargo run --example scripting_advanced_demo

# Interactive console with Lua REPL
cargo run --example scripting_console_demo
```

## Dependencies

- `mlua` 0.10: Lua 5.4 bindings
- `notify` 7.0: File watching for hot-reload
