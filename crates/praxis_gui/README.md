# Praxis GUI

GUI system for the Praxis engine using egui.

## Features

- **Debug UI**: Performance metrics, frame timing, and system information
- **Entity Inspector**: View and edit entity components
- **Hierarchy Panel**: Scene graph visualization and manipulation
- **Inspector Panel**: Component property editing
- **Transform Gizmos**: Interactive 3D manipulation tools
- **Console Panel**: In-game console with command history and Lua REPL

## Console Panel

The `ConsolePanel` provides a comprehensive in-game console with the following features:

### Features

- **Command History**: Navigate previous commands with Up/Down arrow keys
- **Lua REPL Integration**: Execute Lua code directly in the console
- **Custom Command Registration**: Register debug commands with custom handlers
- **Autocomplete**: Tab completion for registered commands
- **Log Filtering**: Filter messages by log level and search text
- **Auto-scroll**: Automatic scrolling to new messages

### Basic Usage

```rust
use praxis_gui::ConsolePanel;
use praxis_scripting::{ScriptingContext, ScriptingConfig};
use std::sync::Arc;
use parking_lot::RwLock;

// Create the console
let mut console = ConsolePanel::new();

// Optional: Set up Lua REPL integration
let scripting_context = Arc::new(RwLock::new(
    ScriptingContext::new(ScriptingConfig::default())?
));
console.set_lua_context(scripting_context);

// Log messages
console.log_info("Console initialized");
console.log_warning("This is a warning");
console.log_error("This is an error");

// Render the console
console.render(&egui_ctx);
```

### Registering Custom Commands

```rust
use praxis_gui::{ConsolePanel, CommandRegistry};

let console = ConsolePanel::new();
let registry = console.command_registry();
let mut registry = registry.write();

// Register a simple command
registry.register(
    "hello",
    "Prints a greeting",
    "hello [name]",
    |args| {
        let name = args.get(0).unwrap_or(&"World");
        Ok(format!("Hello, {}!", name))
    },
);

// Register a command with error handling
registry.register(
    "divide",
    "Divides two numbers",
    "divide <a> <b>",
    |args| {
        if args.len() != 2 {
            return Err("Usage: divide <a> <b>".to_string());
        }
        
        let a: f32 = args[0].parse()
            .map_err(|_| "Invalid number for a")?;
        let b: f32 = args[1].parse()
            .map_err(|_| "Invalid number for b")?;
        
        if b == 0.0 {
            return Err("Cannot divide by zero".to_string());
        }
        
        Ok(format!("{} / {} = {}", a, b, a / b))
    },
);
```

### Console Controls

- **~ (Tilde) or F1**: Toggle console visibility
- **Up/Down Arrow**: Navigate command history
- **Tab**: Cycle through autocomplete suggestions
- **Enter**: Execute command or Lua code
- **Escape**: Close autocomplete suggestions

### Log Levels

The console supports five log levels:

- `Info`: General informational messages (light gray)
- `Warning`: Warning messages (yellow)
- `Error`: Error messages (red)
- `Success`: Success/completion messages (green)
- `Debug`: Debug messages (light blue)

### Lua REPL

When a Lua scripting context is attached, the console can execute Lua code:

```lua
-- Simple expressions
> 2 + 2
4

-- Function calls
> math.sqrt(16)
4.0

-- Print statements
> print("Hello from Lua")
Hello from Lua

-- Multiple return values
> return 1, 2, 3
1, 2, 3
```

### Example

See `examples/console_demo.rs` for a complete example demonstrating:
- Command registration
- Lua REPL integration
- Console toggling
- Log filtering
- Command history

Run the example:
```bash
cargo run --example console_demo
```

## Other Components

### Debug UI

Displays performance metrics and frame timing information.

### Entity Inspector

Browse and edit entity components in the ECS world.

### Hierarchy Panel

Visualize and manipulate the scene graph with drag-and-drop reparenting.

### Transform Gizmos

Interactive 3D gizmos for translating, rotating, and scaling entities.

## Dependencies

- `egui`: Immediate mode GUI framework
- `praxis_ecs`: Entity component system
- `praxis_scripting`: Lua scripting integration (optional for console)
- `parking_lot`: Efficient synchronization primitives
