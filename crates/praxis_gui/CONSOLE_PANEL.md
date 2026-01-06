# Console Panel

The Console Panel is a powerful in-game debugging tool that provides command execution, Lua REPL integration, and comprehensive logging.

## Features

### 1. Command History
- Navigate through previous commands using Up/Down arrow keys
- Maintains last 100 commands
- Restores partially typed commands when navigating history

### 2. Lua REPL Integration
Execute Lua code directly in the console:
```lua
> 2 + 2
4

> math.sqrt(16)
4.0

> return 1, 2, 3
1, 2, 3
```

### 3. Custom Command Registration
Register debug commands with custom handlers:
```rust
let registry = console.command_registry();
let mut registry = registry.write();

registry.register(
    "spawn",
    "Spawn an entity",
    "spawn <type> [x] [y] [z]",
    |args| {
        // Command implementation
        Ok("Entity spawned".to_string())
    }
);
```

### 4. Autocomplete
- Type a partial command name and press Tab
- Cycles through matching commands
- Shows all matches in a dropdown

### 5. Log Filtering
- Filter by log level (Info, Warning, Error, Success, Debug)
- Text search across all messages
- Clear history button
- Auto-scroll toggle

## Usage

### Basic Setup

```rust
use praxis_gui::ConsolePanel;

let mut console = ConsolePanel::new();
console.show(); // Make visible
```

### With Lua REPL

```rust
use praxis_gui::ConsolePanel;
use praxis_scripting::{ScriptingContext, ScriptingConfig};
use std::sync::Arc;
use parking_lot::RwLock;

let mut console = ConsolePanel::new();

let scripting_context = Arc::new(RwLock::new(
    ScriptingContext::new(ScriptingConfig::default())?
));
console.set_lua_context(scripting_context);
```

### Registering Commands

```rust
let registry = console.command_registry();
let mut registry = registry.write();

// Simple command
registry.register(
    "hello",
    "Prints a greeting",
    "hello [name]",
    |args| {
        let name = args.get(0).unwrap_or(&"World");
        Ok(format!("Hello, {}!", name))
    }
);

// Command with validation
registry.register(
    "teleport",
    "Teleport to coordinates",
    "teleport <x> <y> <z>",
    |args| {
        if args.len() != 3 {
            return Err("Usage: teleport <x> <y> <z>".to_string());
        }
        
        let x: f32 = args[0].parse()
            .map_err(|_| "Invalid x coordinate")?;
        let y: f32 = args[1].parse()
            .map_err(|_| "Invalid y coordinate")?;
        let z: f32 = args[2].parse()
            .map_err(|_| "Invalid z coordinate")?;
        
        // Perform teleport
        Ok(format!("Teleported to ({}, {}, {})", x, y, z))
    }
);
```

### Logging Messages

```rust
// Different log levels
console.log_info("Application started");
console.log_warning("Low memory");
console.log_error("Failed to load asset");
console.log_success("Save completed");

// Generic logging with custom level
use praxis_gui::LogLevel;
console.log("Debug information", LogLevel::Debug);
```

### Rendering

```rust
// In your render loop
console.render(&egui_ctx);
```

### Toggling Visibility

```rust
// Toggle
console.toggle();

// Explicit show/hide
console.show();
console.hide();

// Common pattern: bind to tilde key
if input.key_pressed(Key::Grave) {
    console.toggle();
}
```

## Built-in Commands

### help
Display help information about commands.
```
> help
Available commands (3):
  clear: Clear the console history
    Usage: clear
  echo: Echo text to the console
    Usage: echo <text>
  help: Display help information about available commands
    Usage: help [command]

You can also execute Lua code directly.

> help echo
Echo text to the console
Usage: echo <text>
```

### clear
Clear all console history.
```
> clear
```

### echo
Echo text back to the console.
```
> echo Hello, World!
Hello, World!
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Execute command/code |
| `Up` | Previous command in history |
| `Down` | Next command in history |
| `Tab` | Cycle through autocomplete suggestions |
| `Escape` | Close autocomplete |

## Log Levels

The console supports five log levels, each with a distinct color:

- **Info** (Light Gray): General information
- **Warning** (Yellow): Warnings and potential issues
- **Error** (Red): Errors and failures
- **Success** (Green): Successful operations
- **Debug** (Light Blue): Debug information

## Advanced Usage

### Capturing Engine Logs

You can connect the engine's logging system to the console:

```rust
use praxis_utils::tracing;
use praxis_gui::{ConsolePanel, LogLevel};

// Custom tracing subscriber that writes to console
struct ConsoleLayer {
    console: Arc<RwLock<ConsolePanel>>,
}

impl ConsoleLayer {
    pub fn new(console: Arc<RwLock<ConsolePanel>>) -> Self {
        Self { console }
    }
}

// Implement tracing::Layer for ConsoleLayer
// Then messages logged via tracing macros appear in console
```

### Accessing ECS World from Commands

```rust
use praxis_ecs::World;
use std::sync::Arc;
use parking_lot::RwLock;

let world = Arc::new(RwLock::new(World::new()));
let world_clone = Arc::clone(&world);

registry.register(
    "entity_count",
    "Count entities in the world",
    "entity_count",
    move |_args| {
        let world = world_clone.read();
        let count = world.inner().entities().len();
        Ok(format!("World contains {} entities", count))
    }
);
```

### Command with Side Effects

```rust
let game_state = Arc::new(RwLock::new(GameState::default()));
let state_clone = Arc::clone(&game_state);

registry.register(
    "pause",
    "Pause the game",
    "pause",
    move |_args| {
        let mut state = state_clone.write();
        state.paused = !state.paused;
        if state.paused {
            Ok("Game paused".to_string())
        } else {
            Ok("Game resumed".to_string())
        }
    }
);
```

## Performance Considerations

- The console maintains a maximum of 1000 log entries to prevent memory bloat
- Command history is limited to 100 entries
- Autocomplete is triggered only on non-whitespace input
- Filtering is performed client-side on render

## Integration Examples

See `examples/console_demo.rs` for a complete working example demonstrating:
- Console creation and setup
- Lua REPL integration
- Custom command registration
- Event handling for console toggle
- Rendering in a game loop

Run the example:
```bash
cargo run --example console_demo
```

## API Reference

See the [module documentation](src/console_panel.rs) for detailed API reference.
