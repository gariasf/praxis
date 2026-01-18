# Praxis Core

Core engine lifecycle and subsystem orchestration for the Praxis game engine.

## Overview

Main entry point coordinating initialization, event loop, and subsystem integration.

**Key Features:**
- Unified initialization (`praxis_core::run()`)
- Subsystem orchestration (utils, ECS, input, audio, window)
- Event loop management
- Resource initialization patterns

## Quick Start

### Simple Initialization

```rust
use praxis_core;

fn main() -> praxis_utils::Result<()> {
    praxis_core::run()
}
```

### Custom Initialization

```rust
use praxis_utils::Result;
use praxis_ecs::World;

fn main() -> Result<()> {
    // Initialize subsystems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;
    
    // Create world and resources
    let mut world = World::new();
    
    // Setup custom application
    // ...
    
    Ok(())
}
```

## Lifecycle Phases

1. **Startup:** Initialize utils → ECS → input → audio
2. **Event Loop:** Create winit EventLoop
3. **Window Creation:** ApplicationHandler::resumed()
4. **Runtime Loop:** Input → Update → Render → Audio
5. **Shutdown:** Cleanup resources on exit

## Documentation

**Comprehensive Guides:**
- [Architecture Guide](../../docs/architecture.md) - Engine design
- [Getting Started](../../docs/getting-started/README.md) - Installation and setup
- [Beginner's Guide](../../docs/beginners-guide.md) - Learning resource

**Architecture Details:**
- [Engine Lifecycle](../../docs/architecture/engine-lifecycle.md)
- [ECS Patterns](../../docs/architecture/ecs-patterns.md)

## Dependencies

- `praxis_utils`: Logging, error handling
- `praxis_ecs`: Entity-Component-System
- `praxis_input`: Input system
- `praxis_audio`: Audio system
- `praxis_window`: Window management
