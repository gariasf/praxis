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

let mut context = ScriptingContext::new(ScriptingConfig::default())?;

// Load and execute
context.load_script("game_logic", "scripts/game.lua")?;
context.call_function::<_, ()>("game_logic", "update", 0.016)?;

// Hot-reload
context.enable_hot_reload("scripts")?;
context.process_hot_reload()?;
```

## Lua ECS Access

```lua
-- Spawn entity
local entity = world.spawn()
world.add_component_name(entity, "Player")
world.add_component_transform(entity, 0, 0, 0)

-- Query entities
local player = world.get_entity_by_name("Player")
local transform = world.get_component_transform(player)
print("Player at", transform.translation.x, transform.translation.y)

-- Modify components
world.set_component_transform(player, transform)
```

## Sandboxing

```rust
let config = ScriptingConfig {
    sandbox: SandboxConfig {
        level: SandboxLevel::Strict,
        allow_file_io: false,
        instruction_limit: 1_000_000,
        memory_limit: 100 * 1024 * 1024, // 100 MB
    },
    ..Default::default()
};
```

## Documentation

**Comprehensive Guide:**
- [Scripting Guide](../../docs/guides/scripting.md) - Complete usage, patterns, security

**Learning Path:**
- [Scripting Learning Path](../../docs/learning-paths/scripting.md)

## Examples

```bash
cargo run --example scripting_demo
cargo run --example scripting_advanced_demo
cargo run --example scripting_console_demo
```

## Dependencies

- `mlua` 0.10: Lua 5.4 bindings
- `notify` 7.0: File watching for hot-reload
