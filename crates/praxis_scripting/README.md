# Praxis Scripting

Lua scripting integration for the Praxis game engine with full ECS access and hot-reload support.

## Features

- **Lua 5.4** integration via `mlua`
- **ECS Access**: Query and modify entities and components from Lua scripts
- **Hot-Reload**: Auto-reload scripts on file changes for rapid iteration
- **Sandboxing**: Configurable security levels (None/Moderate/Strict)
- **Performance Monitoring**: Track execution time and detect expensive operations
- **REPL Support**: Interactive console for runtime debugging and introspection

## Quick Start

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};

let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;

// Load and execute a script
context.load_script("game_logic", "scripts/game.lua")?;
context.call_function::<_, ()>("game_logic", "update", 0.016)?;
```

## Interactive REPL

The scripting system includes an interactive REPL mode designed for console/debugging:

```rust
use praxis_scripting::ScriptingContext;

let context = ScriptingContext::new(ScriptingConfig::default())?;

// Evaluate expressions interactively
let result = context.eval_interactive("2 + 2")?;
println!("{}", result); // "4"

// With ECS World access
let mut world = World::new();
let result = context.eval_interactive_with_world(
    "console.list_entities()",
    &mut world
)?;
```

## Console Commands

When integrated with the console panel, the scripting system provides powerful introspection commands:

### Entity Queries

```lua
-- List all entities
console.list_entities()

-- Get entity count
console.entity_count()

-- Query entities by component
console.query_with_name()
console.query_with_transform()
```

### Entity Inspection

```lua
-- Find entity by name
local id = console.find_entity("Player")

-- Inspect entity components
console.inspect(id)

-- Get transform
local pos = console.get_transform(id)
print(pos.x, pos.y, pos.z)
```

### Runtime Modifications

```lua
-- Set entity position
console.set_transform(id, 10, 5, 0)

-- Spawn new entity
local new_id = console.spawn("DynamicEntity")

-- Remove entity
console.despawn(id)
```

## ECS Integration

The scripting system provides a `world` table for ECS operations:

```lua
-- Spawn an entity
local entity = world.spawn()

-- Add components
world.add_component_name(entity, "Player")
world.add_component_transform(entity, 0, 0, 0)

-- Query entities
local player = world.get_entity_by_name("Player")
if player then
    local transform = world.get_component_transform(player)
    print("Player position:", transform.translation.x, transform.translation.y, transform.translation.z)
end

-- Modify components
world.set_component_transform(player, transform)
```

## Hot-Reload

Enable automatic script reloading on file changes:

```rust
context.enable_hot_reload("scripts")?;

// In your game loop
context.process_hot_reload()?;
```

## Sandboxing

Configure security levels to restrict dangerous operations:

```rust
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

## Performance Monitoring

Track script performance and detect expensive operations:

```rust
let config = ScriptingConfig {
    enable_performance_monitoring: true,
    max_execution_time_ms: 16, // Warn if script takes > 16ms
    ..Default::default()
};

let context = ScriptingContext::new(config)?;

// After running scripts
if let Some(monitor) = context.performance_monitor() {
    let stats = monitor.get_stats("game_logic");
    println!("Avg execution time: {:?}", stats.average_time);
}
```

## Examples

- `examples/scripting_demo.rs` - Basic scripting setup
- `examples/scripting_advanced_demo.rs` - Advanced features
- `examples/console_demo.rs` - Console integration
- `examples/scripting_console_demo.rs` - Full REPL with ECS introspection

## Console Panel Integration

To integrate scripting with the console panel:

```rust
use praxis_gui::ConsolePanel;
use praxis_scripting::{ScriptingContext, ScriptingConfig};

let mut console = ConsolePanel::new();

// Set up scripting context
let scripting_context = Arc::new(RwLock::new(
    ScriptingContext::new(ScriptingConfig::default())?
));
console.set_lua_context(scripting_context);

// In your game loop, provide world access
console.set_world(&mut world);

// Render the console
console.render(&egui_ctx);
```

Now users can execute Lua code and use console commands interactively!

## API Reference

See the [full API documentation](https://docs.rs/praxis_scripting) for detailed information.
