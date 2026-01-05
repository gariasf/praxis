# Praxis Core

Core engine systems and main loop for the Praxis game engine.

## Features

- **Engine Lifecycle**: Initialization, main loop, and shutdown management
- **Frame Timing**: Delta time calculation and frame rate control
- **System Orchestration**: Coordinates all engine subsystems (graphics, audio, physics, input, ECS)
- **Resource Management**: Central resource initialization and cleanup
- **Event Loop Integration**: Seamless integration with winit event loop

## Architecture

The core crate ties together all engine subsystems:

- **Graphics**: Vulkan rendering via `praxis_graphics`
- **Window**: Window management via `praxis_window`
- **ECS**: Entity-Component-System via `praxis_ecs`
- **Input**: Input handling via `praxis_input`
- **Audio**: Sound playback via `praxis_audio`
- **Utils**: Logging and error handling via `praxis_utils`

## Usage

```rust
use praxis_core::Engine;

fn main() -> praxis_utils::Result<()> {
    // Initialize engine
    let mut engine = Engine::new()?;
    
    // Run main loop
    engine.run()?;
    
    Ok(())
}
```

## Main Loop

The engine's main loop executes the following phases each frame:

1. **Input**: Process window events and update input state
2. **Update**: Run ECS systems (physics, audio, game logic)
3. **Render**: Draw the current frame
4. **Audio**: Update spatial audio and process sound events
5. **Timing**: Calculate delta time and enforce frame rate

## Integration

The core crate depends on and initializes:

```rust
// Graphics and window
let window = praxis_window::Window::new()?;
let graphics = praxis_graphics::RenderContext::new(&window)?;

// ECS world
let mut world = praxis_ecs::World::new();

// Input system
let input = praxis_input::InputState::default();
world.insert_resource(input);

// Audio system
let audio = praxis_audio::AudioManager::new()?;
world.insert_resource(audio);
```

## Examples

The engine core is used by all examples in the workspace:

```bash
cargo run --example comprehensive_scene_demo
cargo run --example scene_demo
cargo run --example gui_demo
```

## Dependencies

- `praxis_utils`: Logging, error handling
- `praxis_graphics`: Vulkan rendering
- `praxis_window`: Window management
- `praxis_ecs`: Entity-Component-System
- `praxis_input`: Input handling
- `praxis_audio`: Audio playback

## See Also

- [Engine Architecture](../../docs/architecture.md)
- [Getting Started Guide](../../docs/getting-started/README.md)
- [Main Documentation](../../docs/README.md)
