# praxis_core

Core engine lifecycle and main loop for Praxis.

## Overview

Manages the engine's initialization, main loop, and shutdown. Coordinates all subsystems.

## Responsibilities

- Engine initialization
- Main game loop
- Frame timing and delta time
- Subsystem coordination
- Resource management
- Shutdown and cleanup

## Architecture

The core provides the foundation that all other subsystems build upon:

```
praxis_core
    ├── Window (praxis_window)
    ├── Graphics (praxis_graphics)
    ├── ECS (praxis_ecs)
    ├── Input (praxis_input)
    └── Audio (praxis_audio)
```

## Example

```rust
use praxis_core::Engine;

fn main() {
    let mut engine = Engine::new().expect("Failed to create engine");
    engine.run();
}
```

## Dependencies

- `praxis_utils`: Logging and error handling
- `praxis_window`: Window management
- `praxis_graphics`: Rendering
- `praxis_ecs`: Entity Component System
- `praxis_input`: Input handling
- `praxis_audio`: Audio system
- `pollster`: Async runtime
- `parking_lot`: Faster mutexes

## Usage

```toml
praxis_core = { path = "../praxis_core", version = "0.1.0" }
```
