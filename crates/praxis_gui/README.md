# Praxis GUI

GUI system using egui for the Praxis engine.

## Overview

Immediate mode GUI with debug panels, console, and editor integration.

**Key Features:**
- Debug UI with performance metrics
- Entity inspector and hierarchy panel
- Console panel with Lua REPL
- Transform gizmos
- Command history and autocomplete

## Quick Start

### Console Panel

```rust
use praxis_gui::ConsolePanel;

let mut console = ConsolePanel::new();

// Log messages
console.log_info("Console initialized");
console.log_warning("This is a warning");

// Render
console.render(&egui_ctx);
```

### Custom Commands

```rust
use praxis_gui::CommandRegistry;

let registry = console.command_registry();
let mut registry = registry.write();

registry.register(
    "hello",
    "Prints a greeting",
    "hello [name]",
    |args| {
        let name = args.get(0).unwrap_or(&"World");
        Ok(format!("Hello, {}!", name))
    },
);
```

## Console Controls

- **~ or F1:** Toggle console
- **Up/Down:** Command history
- **Tab:** Autocomplete
- **Enter:** Execute command/Lua

## Documentation

**Reference:**
- [GUI API Reference](../../docs/reference/gui-api.md)

## Examples

```bash
cargo run --example console_demo
cargo run --example gui_demo
```

## Dependencies

- `egui` 0.29: Immediate mode GUI
- `praxis_scripting`: Lua REPL (optional)

## API Stability

**Status:** Evolving

Console panel and command registry are stable. Inspector and hierarchy panels may see API improvements as editor features expand. Breaking changes will be documented in the changelog.
