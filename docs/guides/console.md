# In-Game Console

Praxis provides a powerful in-game console with Lua REPL integration, command history, autocomplete, and full ECS introspection. The `ConsolePanel` is built on egui and integrates seamlessly with the scripting system.

## Overview

The console serves as a debugging and development tool with features designed for rapid prototyping and runtime inspection.

**Features:**
- **Lua REPL**: Execute Lua code interactively
- **Command System**: Register custom debug commands
- **ECS Introspection**: Query and modify entities at runtime
- **Command History**: Navigate previous commands (Up/Down arrows)
- **Autocomplete**: Tab-complete command names
- **Log Filtering**: Filter by level and text search
- **Syntax Support**: Highlight different log levels
- **Auto-scroll**: Automatically scroll to latest messages

**Use Cases:**
- Runtime debugging
- Tweaking values without recompile
- Spawning test entities
- Querying game state
- Prototyping gameplay features
- Performance profiling

## Basic Usage

### Creating a Console

```rust
use praxis_gui::ConsolePanel;

let mut console = ConsolePanel::new();

// Show console
console.show();

// Or toggle visibility
console.toggle();
```

### Logging Messages

```rust
// Different log levels
console.log_info("Game initialized");
console.log_warning("Low memory warning");
console.log_error("Failed to load asset");
console.log_success("Checkpoint saved");
console.log_debug("Frame time: 16ms");

// Generic logging with level
use praxis_gui::LogLevel;
console.log("Custom message", LogLevel::Info);
```

### Rendering

```rust
// In your game loop
fn render(&mut self, egui_ctx: &egui::Context) {
    console.render(egui_ctx);
}
```

## Lua REPL Integration

### Setting Up Lua Context

```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};
use std::sync::Arc;
use parking_lot::RwLock;

// Create scripting context
let scripting_context = Arc::new(RwLock::new(
    ScriptingContext::new(ScriptingConfig::default())?
));

// Attach to console
console.set_lua_context(scripting_context);
```

### Basic Lua Expressions

Once Lua is integrated, the console can evaluate expressions:

```lua
-- Arithmetic
> 2 + 2
4

-- Math functions
> math.sqrt(16)
4.0

-- String operations
> string.upper("hello")
HELLO

-- Tables
> {1, 2, 3, 4}
{1, 2, 3, 4}
```

### ECS Access from Lua

Enable ECS introspection by setting the world:

```rust
// Each frame, update the world reference
console.set_world(&mut world);
```

Now you can use console commands for ECS:

```lua
-- List all entities
> console.list_entities()
Entity 0: "Player"
Entity 1: "MainCamera"
Entity 2: "Sun"
...

-- Count entities
> console.entity_count()
15

-- Find entity by name
> local player_id = console.find_entity("Player")
> console.inspect(player_id)
Entity 5 (Player):
  - Transform: pos=(0.0, 1.0, 0.0)
  - MeshHandle: "character"
  - Active: true

-- Modify transform
> console.set_transform(player_id, 10.0, 5.0, 0.0)
Set transform for Entity 5

-- Get transform
> console.get_transform(player_id)
Position: (10.0, 5.0, 0.0)

-- Query entities
> console.query_with_transform()
Found 12 entities with Transform

> console.query_with_name()
Found 15 entities with Name
```

## Custom Commands

Register custom debug commands:

```rust
// Get command registry
let registry = console.command_registry();
let mut registry = registry.write();

// Register a command
registry.register(
    "spawn_enemy",                      // Command name
    "Spawn an enemy at position",       // Description
    "spawn_enemy <x> <y> <z>",         // Usage
    |args| {                            // Handler
        if args.len() != 3 {
            return Err("Usage: spawn_enemy <x> <y> <z>".to_string());
        }
        
        let x: f32 = args[0].parse()
            .map_err(|_| "Invalid x coordinate")?;
        let y: f32 = args[1].parse()
            .map_err(|_| "Invalid y coordinate")?;
        let z: f32 = args[2].parse()
            .map_err(|_| "Invalid z coordinate")?;
        
        // Spawn enemy (needs access to world - see pattern below)
        Ok(format!("Spawned enemy at ({}, {}, {})", x, y, z))
    }
);
```

Usage in console:

```
> spawn_enemy 10 0 5
Spawned enemy at (10, 0, 5)

> help spawn_enemy
Spawn an enemy at position
Usage: spawn_enemy <x> <y> <z>
```

### Commands with World Access

For commands that need to modify the ECS world:

```rust
use std::sync::Arc;
use parking_lot::Mutex;

// Wrap world in Arc<Mutex<>> for shared access
let world = Arc::new(Mutex::new(World::new()));

// Clone for command
let world_clone = Arc::clone(&world);

registry.register(
    "spawn_cube",
    "Spawn a cube at origin",
    "spawn_cube",
    move |args| {
        let mut world = world_clone.lock();
        
        world.spawn((
            Name("SpawnedCube".to_string()),
            Transform::from_xyz(0.0, 1.0, 0.0),
            MeshHandle::new("cube"),
        ));
        
        Ok("Spawned cube".to_string())
    }
);
```

### Complex Command Example

```rust
registry.register(
    "set_time_scale",
    "Set game time scale (0.0-2.0)",
    "set_time_scale <scale>",
    move |args| {
        if args.is_empty() {
            return Err("Usage: set_time_scale <scale>".to_string());
        }
        
        let scale: f32 = args[0].parse()
            .map_err(|_| "Invalid scale value")?;
        
        if !(0.0..=2.0).contains(&scale) {
            return Err("Scale must be between 0.0 and 2.0".to_string());
        }
        
        // Set time scale (would need shared time manager)
        TIME_MANAGER.lock().set_scale(scale);
        
        Ok(format!("Time scale set to {:.2}", scale))
    }
);
```

## Built-in Commands

The console comes with several built-in commands:

### `help`

Display command help:

```
> help
Available commands (15):
  clear: Clear the console history
  echo: Echo text to the console
  help: Display help information
  ...

> help echo
Echo text to the console
Usage: echo <text>
```

### `clear`

Clear console history:

```
> clear
(console cleared)
```

### `echo`

Echo text back to console:

```
> echo Hello, World!
Hello, World!
```

## UI Features

### Command History Navigation

```
> command1
> command2
> command3
> (press Up arrow)
command3
> (press Up arrow)
command2
> (press Down arrow)
command3
```

History is preserved across console close/open cycles.

### Autocomplete

```
> spa<Tab>
spawn_enemy
spawn_cube

> spawn_<Tab>
spawn_enemy
spawn_cube
```

Cycle through suggestions with Tab.

### Log Filtering

Filter logs by level:

```rust
// In console toolbar
// Dropdown: All | Info | Warning | Error | Success | Debug
```

Filter by text:

```rust
// In console toolbar
// Text box: "player" (shows only logs containing "player")
```

### Auto-Scroll

Toggle auto-scroll in toolbar. When enabled, console automatically scrolls to newest messages.

## Integration Patterns

### Toggle Console with Hotkey

```rust
use winit::event::{Event, WindowEvent, KeyEvent, ElementState};
use winit::keyboard::{Key, NamedKey};

fn handle_events(&mut self, event: &Event<()>) {
    match event {
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key: key,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            },
            ..
        } => {
            match key {
                Key::Character(c) if c == "`" || c == "~" => {
                    self.console.toggle();
                }
                Key::Named(NamedKey::F1) => {
                    self.console.toggle();
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

### Custom Log Sink

Forward game logs to console:

```rust
use tracing_subscriber::layer::SubscriberExt;

struct ConsoleLogSink {
    console: Arc<Mutex<ConsolePanel>>,
}

impl tracing_subscriber::Layer for ConsoleLogSink {
    fn on_event(&self, event: &tracing::Event, _ctx: tracing_subscriber::layer::Context) {
        let mut console = self.console.lock();
        
        let level = match *event.metadata().level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warning,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Debug,
        };
        
        console.log(format!("{:?}", event), level);
    }
}

// Set up
let console = Arc::new(Mutex::new(ConsolePanel::new()));
let sink = ConsoleLogSink { console: Arc::clone(&console) };

let subscriber = tracing_subscriber::registry().with(sink);
tracing::subscriber::set_global_default(subscriber)?;
```

### Debug Menu Integration

```rust
fn render_debug_menu(&mut self, ui: &mut egui::Ui) {
    ui.menu_button("Debug", |ui| {
        if ui.button("Toggle Console (F1)").clicked() {
            self.console.toggle();
        }
        
        if ui.button("Clear Console").clicked() {
            self.console.clear();
        }
        
        ui.separator();
        
        if ui.button("Spawn Test Entity").clicked() {
            self.console.log_info("Spawning test entity...");
            // Spawn entity
        }
    });
}
```

### Performance Monitoring

```rust
// Register performance command
registry.register(
    "fps",
    "Show FPS and frame time",
    "fps",
    move |_| {
        let fps = FPS_COUNTER.lock().fps();
        let frame_time = FPS_COUNTER.lock().frame_time_ms();
        
        Ok(format!("FPS: {:.1}, Frame: {:.2}ms", fps, frame_time))
    }
);

// Use in console
> fps
FPS: 60.0, Frame: 16.67ms
```

### State Inspection

```rust
registry.register(
    "game_state",
    "Display current game state",
    "game_state",
    move |_| {
        let state = GAME_STATE.lock();
        
        Ok(format!(
            "State: {:?}\nPlayers: {}\nScore: {}",
            state.current_state,
            state.player_count,
            state.score
        ))
    }
);
```

## Advanced Usage

### Multi-line Lua Scripts

For complex Lua code, use semicolons or load scripts:

```lua
-- Multi-statement execution
> local x = 10; local y = 20; return x + y
30

-- Or load and execute script file
> dofile("scripts/debug_commands.lua")
```

### Variable Persistence

Lua variables persist across commands:

```lua
> player_health = 100
100

> player_health = player_health - 25
75

> print(player_health)
75
```

### Table Introspection

```lua
> t = {name = "Player", health = 100, items = {"sword", "shield"}}
> print(t.name)
Player

> for k, v in pairs(t) do print(k, v) end
name    Player
health  100
items   table: 0x...
```

### Function Definitions

```lua
> function damage_player(amount)
>   player_health = player_health - amount
>   print("Player health:", player_health)
> end

> damage_player(10)
Player health: 90
```

### ECS Query Helpers

Register helper commands for common queries:

```rust
registry.register(
    "find",
    "Find entities by name pattern",
    "find <pattern>",
    move |args| {
        if args.is_empty() {
            return Err("Usage: find <pattern>".to_string());
        }
        
        let pattern = args[0].to_lowercase();
        let world = WORLD.lock();
        
        let mut matches = Vec::new();
        for (entity, name) in world.query::<(Entity, &Name)>().iter(&world) {
            if name.0.to_lowercase().contains(&pattern) {
                matches.push(format!("Entity {:?}: {}", entity, name.0));
            }
        }
        
        if matches.is_empty() {
            Ok(format!("No entities matching '{}'", pattern))
        } else {
            Ok(matches.join("\n"))
        }
    }
);
```

Usage:

```
> find player
Entity 5: "Player"
Entity 12: "PlayerCamera"

> find enemy
Entity 20: "Enemy_Orc_01"
Entity 21: "Enemy_Orc_02"
Entity 22: "Enemy_Spider_01"
```

## Console Styling

### Custom Colors

```rust
// Modify log level colors by creating custom LogEntry
impl LogEntry {
    fn custom_color(&self) -> egui::Color32 {
        match self.level {
            LogLevel::Info => egui::Color32::from_rgb(200, 200, 255),
            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 0),
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
            LogLevel::Success => egui::Color32::from_rgb(100, 255, 100),
            LogLevel::Debug => egui::Color32::from_rgb(150, 150, 200),
        }
    }
}
```

### Window Customization

```rust
// Modify ConsolePanel::render() for custom window style
egui::Window::new("Console")
    .default_pos(egui::pos2(10.0, 400.0))
    .default_size(egui::vec2(1000.0, 500.0))
    .resizable(true)
    .collapsible(false)
    .title_bar(true)
    .frame(egui::Frame {
        fill: egui::Color32::from_rgba_premultiplied(20, 20, 20, 240),
        ..Default::default()
    })
    .show(ctx, |ui| {
        // Console content
    });
```

## Performance Considerations

### History Limits

The console automatically limits history:

```rust
const MAX_HISTORY_SIZE: usize = 1000;      // Log entries
const MAX_COMMAND_HISTORY: usize = 100;    // Command history
```

### Memory Usage

Typical memory usage:
- Empty console: ~10 KB
- 1,000 log entries: ~100-500 KB (depends on message length)
- Command history: ~10-50 KB

### Rendering Performance

Console rendering is negligible when hidden:

```rust
if !self.console.visible {
    return; // No rendering cost
}
```

When visible: ~0.1-0.3ms per frame (depends on number of visible entries).

## Best Practices

### 1. Command Naming

Use consistent naming:
```rust
// Good
"spawn_enemy"
"set_time_scale"
"get_player_pos"

// Avoid
"spawnEnemy"  // Inconsistent case
"SetTimeScale" // PascalCase
"getplayerpos" // No separator
```

### 2. Error Messages

Provide helpful error messages:

```rust
registry.register("teleport", "...", "...", |args| {
    if args.len() != 3 {
        return Err("Usage: teleport <x> <y> <z>\nExample: teleport 10 0 5".to_string());
    }
    
    let x = args[0].parse::<f32>()
        .map_err(|_| format!("Invalid x coordinate: '{}'", args[0]))?;
    
    // ...
});
```

### 3. Command Categories

Organize commands by prefix:

```rust
// Player commands
"player.health"
"player.position"
"player.inventory"

// World commands
"world.spawn"
"world.time"
"world.weather"

// Debug commands
"debug.wireframe"
"debug.colliders"
"debug.fps"
```

### 4. Safe Command Execution

Validate inputs and handle errors gracefully:

```rust
registry.register("god_mode", "...", "...", |args| {
    let enabled = if args.is_empty() {
        // Toggle
        !GOD_MODE.load(Ordering::Relaxed)
    } else {
        // Parse argument
        match args[0] {
            "on" | "true" | "1" => true,
            "off" | "false" | "0" => false,
            _ => return Err("Use: god_mode [on|off]".to_string()),
        }
    };
    
    GOD_MODE.store(enabled, Ordering::Relaxed);
    Ok(format!("God mode: {}", if enabled { "ON" } else { "OFF" }))
});
```

### 5. Development vs. Release

Disable console in release builds if desired:

```rust
#[cfg(debug_assertions)]
console.show();

#[cfg(not(debug_assertions))]
console.hide();
```

Or use a cvar:

```rust
if config.enable_debug_console {
    console.show();
}
```

## Security Considerations

### Sandboxing

The Lua environment is sandboxed by default:

```rust
let config = ScriptingConfig {
    security_level: SecurityLevel::Moderate, // Disables file I/O, os functions
    ..Default::default()
};
```

Levels:
- `None`: Full Lua access (development only)
- `Moderate`: Disables file I/O, os, debug functions
- `Strict`: Only allows safe computation

### Command Validation

Always validate command inputs:

```rust
registry.register("set_level", "...", "...", |args| {
    let level_id = args[0].parse::<u32>()?;
    
    // Validate range
    if level_id > MAX_LEVEL_ID {
        return Err("Invalid level ID".to_string());
    }
    
    // Validate exists
    if !LEVEL_MANAGER.level_exists(level_id) {
        return Err("Level not found".to_string());
    }
    
    LEVEL_MANAGER.load_level(level_id)?;
    Ok(format!("Loaded level {}", level_id))
});
```

### Release Builds

Consider disabling or restricting console in production:

```rust
#[cfg(not(debug_assertions))]
fn create_console() -> ConsolePanel {
    let mut console = ConsolePanel::new();
    console.hide();
    
    // Only register safe commands
    let registry = console.command_registry();
    let mut registry = registry.write();
    registry.register("help", "...", "...", |_| Ok("...".to_string()));
    registry.register("version", "...", "...", |_| Ok(VERSION.to_string()));
    
    console
}
```

## Troubleshooting

### Console Not Showing

Check that:
1. `console.visible` is `true`
2. `console.render()` is called each frame
3. egui integration is set up correctly
4. Window is not collapsed

### Lua Code Not Executing

Verify:
1. Lua context is set: `console.set_lua_context(ctx)`
2. Context is not locked by another thread
3. Code is valid Lua syntax
4. Security level allows the operation

### ECS Commands Not Working

Ensure:
1. World is set each frame: `console.set_world(&mut world)`
2. World pointer remains valid during command execution
3. Commands are called from the main thread

### Performance Issues

If console causes lag:
1. Reduce `MAX_HISTORY_SIZE`
2. Implement log level filtering
3. Clear old entries periodically
4. Disable when not visible

## Example Integration

Complete console integration example:

```rust
use praxis_gui::ConsolePanel;
use praxis_scripting::{ScriptingContext, ScriptingConfig};
use std::sync::Arc;
use parking_lot::RwLock;

struct Game {
    console: ConsolePanel,
    scripting: Arc<RwLock<ScriptingContext>>,
    world: World,
}

impl Game {
    fn new() -> Result<Self> {
        let mut console = ConsolePanel::new();
        
        // Set up Lua
        let scripting = Arc::new(RwLock::new(
            ScriptingContext::new(ScriptingConfig::default())?
        ));
        console.set_lua_context(Arc::clone(&scripting));
        
        // Register commands
        let registry = console.command_registry();
        Self::register_commands(&mut registry.write());
        
        // Initial logs
        console.log_success("Game initialized");
        console.log_info("Press F1 to toggle console");
        
        Ok(Self {
            console,
            scripting,
            world: World::new(),
        })
    }
    
    fn register_commands(registry: &mut CommandRegistry) {
        registry.register("quit", "Quit game", "quit", |_| {
            std::process::exit(0);
        });
        
        registry.register("fps", "Show FPS", "fps", |_| {
            Ok(format!("FPS: {:.1}", 60.0)) // Get actual FPS
        });
    }
    
    fn update(&mut self) {
        // Update world reference for console commands
        self.console.set_world(&mut self.world);
        
        // Game logic...
    }
    
    fn render(&mut self, ctx: &egui::Context) {
        // Render console
        self.console.render(ctx);
        
        // Other UI...
    }
    
    fn handle_input(&mut self, event: &WindowEvent) {
        // Toggle console
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if event.logical_key == Key::Named(NamedKey::F1) {
                self.console.toggle();
            }
        }
    }
}
```

## See Also

- [Scripting](scripting.md) - Lua scripting system
- [ECS](../concepts/ecs.md) - Entity-Component-System
- [GUI](../reference/gui.md) - egui integration
- Example: `examples/scripting_console_demo.rs`
- Example: `examples/console_demo.rs`
