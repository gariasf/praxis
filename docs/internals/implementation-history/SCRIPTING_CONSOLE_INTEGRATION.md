# Scripting Console Integration

This document describes the console integration with `praxis_scripting`, including REPL support and engine introspection commands.

## Overview

The Praxis scripting system now includes a fully-featured REPL (Read-Eval-Print Loop) for interactive debugging and runtime ECS introspection through the console panel.

## Features Implemented

### 1. Interactive REPL (`ScriptingContext::eval_interactive`)

A new method for evaluating Lua code in REPL mode with automatic expression handling:

```rust
// In ScriptingContext
pub fn eval_interactive(&self, code: &str) -> Result<String>
pub fn eval_interactive_with_world(&self, code: &str, world: &mut World) -> Result<String>
```

**Features:**
- Automatic expression evaluation (tries as statement first, then as expression)
- Multi-value return support
- Formatted output with proper type display
- Performance monitoring integration

**Examples:**
```lua
2 + 2                     -- Returns: "4"
math.sqrt(16)             -- Returns: "4"
x = 42                    -- Returns: "" (statement, no output)
return 1, 2, 3            -- Returns: "1, 2, 3"
```

### 2. Console Commands Module (`console_commands.rs`)

A comprehensive set of Lua functions for ECS introspection and runtime modifications:

#### Entity Queries
- `console.list_entities()` - List all entities with IDs and names
- `console.entity_count()` - Get total entity count
- `console.query_with_name()` - Query entities with Name component
- `console.query_with_transform()` - Query entities with Transform component

#### Entity Inspection
- `console.find_entity(name)` - Find entity by name, returns entity ID
- `console.inspect(entity_id)` - Display all recognized components

#### Transform Operations
- `console.get_transform(entity_id)` - Get entity position as table {x, y, z}
- `console.set_transform(entity_id, x, y, z)` - Set entity position

#### Entity Lifecycle
- `console.spawn(name)` - Spawn new entity with Name and Transform
- `console.despawn(entity_id)` - Remove entity from world

### 3. Console Panel Integration

The `ConsolePanel` now integrates seamlessly with scripting:

```rust
// In ConsolePanel
pub fn set_lua_context(&mut self, context: Arc<RwLock<ScriptingContext>>)
pub fn set_world(&mut self, world: &mut World)
```

**Usage Pattern:**
```rust
// Setup
let mut console = ConsolePanel::new();
let scripting_context = Arc::new(RwLock::new(ScriptingContext::new(config)?));
console.set_lua_context(scripting_context);

// In game loop
console.set_world(&mut world);  // Update world reference each frame
console.render(&egui_ctx);
```

The console automatically uses `eval_interactive_with_world` when a world is available, providing full ECS access to all Lua commands.

## Implementation Details

### Thread-Local World Context

The console commands access the ECS World through a thread-local storage pattern:

```rust
thread_local! {
    static WORLD_CONTEXT: RefCell<Option<*mut World>> = const { RefCell::new(None) };
}
```

This is set temporarily during console command execution via `set_world_context` and cleared afterward, ensuring safe access while maintaining the Lua API simplicity.

### Value Formatting

The `format_lua_value` helper provides user-friendly display:
- Numbers: Smart formatting (integers without decimals, floats with precision)
- Strings: Displayed with quotes for clarity
- Tables/Functions: Type name only
- Boolean/Nil: Standard representation

### Automatic Registration

Console commands are automatically registered during `ScriptingContext::new()` via `register_console_commands()`, so they're always available.

## Examples

### Basic Usage

```lua
-- Check entities
console.entity_count()
console.list_entities()

-- Find and inspect
local id = console.find_entity("Player")
console.inspect(id)

-- Modify position
console.set_transform(id, 10, 5, 0)
```

### Advanced Workflow

```lua
-- Query all entities with transforms
console.query_with_transform()

-- Spawn multiple entities
for i = 1, 5 do
    console.spawn("Enemy_" .. i)
end

-- Verify they were created
console.list_entities()

-- Clean up
local id = console.find_entity("Enemy_1")
console.despawn(id)
```

### Integration with Regular Lua

```lua
-- Mix console commands with regular Lua
local count = console.entity_count()
print("Current entity count: " .. count)

-- Use math operations
local id = console.find_entity("Player")
if id then
    local pos = console.get_transform(id)
    local distance = math.sqrt(pos.x * pos.x + pos.z * pos.z)
    print("Player distance from origin: " .. distance)
end
```

## Files Modified

### Core Implementation
- `crates/praxis_scripting/src/context.rs` - Added `eval_interactive` methods
- `crates/praxis_scripting/src/bindings/console_commands.rs` - **New file** with console commands
- `crates/praxis_scripting/src/bindings/mod.rs` - Export console_commands module
- `crates/praxis_scripting/src/bindings/ecs_api.rs` - Exposed `with_world_raw` helper
- `crates/praxis_scripting/src/lib.rs` - Updated docs and exports

### Console Integration
- `crates/praxis_gui/src/console_panel.rs` - Added `set_world` and updated `execute_lua`

### Documentation & Examples
- `crates/praxis_scripting/README.md` - Comprehensive documentation
- `examples/console_demo.rs` - Updated to demonstrate new features
- `examples/scripting_console_demo.rs` - **New example** showcasing full REPL capabilities
- `CLAUDE.md` - Added new example to list

## Testing

New tests added in `context.rs`:
- `test_eval_interactive_expression` - Expression evaluation
- `test_eval_interactive_statement` - Statement execution
- `test_eval_interactive_with_world` - World context integration
- `test_eval_interactive_error` - Error handling
- `test_eval_interactive_multi_value` - Multiple return values

Tests in `console_commands.rs`:
- `test_register_console_commands` - Registration verification
- `test_console_commands_with_world` - World integration

## Future Enhancements

Potential additions for future versions:
1. **Component Registration API** - Allow scripts to query custom components
2. **Entity Filtering** - Advanced query syntax (e.g., `console.query("Name && !Transform")`)
3. **Batch Operations** - Operate on multiple entities at once
4. **Undo/Redo Support** - Track console command history for rollback
5. **Script Templates** - Pre-defined command sequences
6. **Performance Profiling** - Track console command execution time
7. **Autocomplete** - Suggest console.* commands in the UI

## Performance Considerations

- Console commands use the existing ECS query system, so performance scales with entity count
- World pointer is updated each frame but only dereferenced during command execution
- REPL evaluation creates minimal overhead (single parse attempt, or two for expressions)
- Performance monitoring can track console command execution time when enabled

## Security Notes

Console commands respect the sandbox configuration:
- When `SandboxLevel::Strict` is enabled, file I/O and OS access remain blocked
- Console commands only access the ECS World (read/write entity data)
- No access to raw memory or system resources
- Script evaluation is still subject to memory limits and execution time monitoring
