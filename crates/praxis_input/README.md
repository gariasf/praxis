# Praxis Input

Input handling with action mapping for the Praxis game engine.

## Overview

Keyboard, mouse, and gamepad tracking with frame-aware state and rebindable controls.

**Key Features:**
- Input state tracking (pressed, just pressed, just released)
- Action mapping for rebindable controls
- Mouse position/delta and scroll tracking
- Seamless winit integration
- ECS resource integration

## Quick Start

```rust
use praxis_input::{InputState, InputMap, Action};
use winit::keyboard::KeyCode;

let mut input = InputState::default();

// Check state
if input.is_key_pressed(KeyCode::KeyW) {
    // Move forward
}

if input.is_key_just_pressed(KeyCode::Space) {
    // Jump (once per press)
}
```

## Action Mapping

```rust
let mut input_map = InputMap::default();

// Bind keys to actions
input_map.bind_key(Action::new("jump"), KeyCode::Space);
input_map.bind_key(Action::new("fire"), KeyCode::KeyE);

// Check actions
if input_map.is_action_pressed(&Action::new("jump"), &input_state) {
    // Perform jump
}
```

## ECS Integration

```rust
use praxis_ecs::World;

world.insert_resource(InputState::default());
world.insert_resource(input_map);

// In systems
fn player_system(input: Res<InputState>, map: Res<InputMap>) {
    if map.is_action_just_pressed(&Action::new("fire"), &input) {
        // Fire weapon
    }
}
```

## Documentation

**Comprehensive Guide:**
- [Input Guide](../../docs/guides/input.md) - Complete input system guide

**Concepts:**
- [Input Concepts](../../docs/concepts/input.md)

**Reference:**
- [Input API Reference](../../docs/reference/input-api.md)

## Examples

```bash
cargo run --example input_integration
```

## Dependencies

- `winit` 0.30.11: Input events
