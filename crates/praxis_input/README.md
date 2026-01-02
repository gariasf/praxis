# praxis_input

Input handling system for the Praxis game engine, providing keyboard, mouse, and gamepad input tracking with action mapping support for rebindable controls.

## Features

- **Input State Tracking**: Track keyboard keys, mouse buttons, and mouse movement/scroll
- **Action Mapping**: Map physical inputs to logical actions for rebindable controls
- **Frame-aware Input**: Distinguish between "pressed", "just pressed", and "just released" states
- **winit Integration**: Seamless integration with winit window events
- **ECS Resource**: Designed to work as bevy_ecs resources

## Usage

### Basic Input State

```rust
use praxis_input::InputState;
use winit::keyboard::KeyCode;

let mut input = InputState::default();

// Simulate key press
input.press_key(KeyCode::KeyW);

// Check input state
if input.is_key_pressed(KeyCode::KeyW) {
    // Move forward
}

if input.is_key_just_pressed(KeyCode::Space) {
    // Jump (only triggers once per press)
}
```

### Action Mapping

```rust
use praxis_input::{InputMap, Action, InputState};
use winit::keyboard::KeyCode;

let mut input_map = InputMap::default();

// Bind multiple keys to the same action
input_map.bind_key(Action::new("jump"), KeyCode::Space);
input_map.bind_key(Action::new("jump"), KeyCode::KeyW);

// Check if action is active
let input_state = InputState::default();
if input_map.is_action_pressed(&Action::new("jump"), &input_state) {
    // Perform jump
}
```

### Integration with ECS

```rust
use bevy_ecs::world::World;
use praxis_input::{InputState, InputMap, Action};
use winit::keyboard::KeyCode;

let mut world = World::new();

// Add input resources
world.insert_resource(InputState::default());

let mut input_map = InputMap::default();
input_map.bind_key(Action::new("fire"), KeyCode::Space);
world.insert_resource(input_map);

// In your systems
fn player_input_system(
    input_state: Res<InputState>,
    input_map: Res<InputMap>,
) {
    if input_map.is_action_just_pressed(&Action::new("fire"), &input_state) {
        // Fire weapon
    }
}
```

### winit Event Handling

```rust
use praxis_input::{InputState, winit_integration};
use winit::event::WindowEvent;

let mut input_state = InputState::default();

// In your event loop
match event {
    WindowEvent::KeyboardInput { .. }
    | WindowEvent::MouseInput { .. }
    | WindowEvent::CursorMoved { .. }
    | WindowEvent::MouseWheel { .. } => {
        winit_integration::process_window_event(&mut input_state, &event);
    }
    _ => {}
}
```

### Frame Updates

Call `update()` at the beginning of each frame to clear "just pressed" and "just released" states:

```rust
use praxis_input::InputState;

let mut input_state = InputState::default();

// Game loop
loop {
    input_state.update();  // Clear frame-specific state
    
    // Process events and check input
    // ...
}
```

## Examples

Run the examples to see the input system in action:

```bash
# Basic demonstration
cargo run --example input_demo

# Full integration with winit and ECS
cargo run --example input_integration
```

## Architecture

### InputState

The `InputState` resource tracks the current state of all input devices:

- **Keyboard**: Set of pressed keys, just pressed keys, just released keys
- **Mouse Buttons**: Set of pressed buttons, just pressed buttons, just released buttons
- **Mouse Position**: Current cursor position and delta since last frame
- **Mouse Scroll**: Scroll wheel delta for the current frame

### InputMap

The `InputMap` resource provides action mapping capabilities:

- Maps `Action` identifiers to sets of `InputBinding`s
- Supports multiple inputs per action
- Bidirectional lookup (action → bindings, binding → actions)
- Enables rebindable controls

### Action

Logical game actions that abstract physical inputs:

- Identified by string-based `ActionId`
- Can be bound to multiple physical inputs
- Enables control remapping without changing game logic

## License

GPL-3.0-or-later
