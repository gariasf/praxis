# Praxis Input System

The Praxis input system provides comprehensive input handling for keyboard, mouse, and gamepad devices, with support for action mapping and rebindable controls.

## Architecture Overview

The input system is organized into several key components:

### Core Components

1. **InputState** (`praxis_input::InputState`)
   - ECS Resource that tracks the current state of all input devices
   - Maintains pressed/just pressed/just released states for keys and mouse buttons
   - Tracks mouse position, delta, and scroll wheel
   - Frame-aware input tracking

2. **InputMap** (`praxis_input::InputMap`)
   - ECS Resource that maps physical inputs to logical actions
   - Enables rebindable controls
   - Supports multiple inputs per action
   - Bidirectional lookup (action → bindings, binding → actions)

3. **Action** (`praxis_input::Action`)
   - Represents a logical game action (e.g., "jump", "fire")
   - Identified by a string-based `ActionId`
   - Provides abstraction layer between input and game logic

4. **InputBinding** (`praxis_input::InputBinding`)
   - Represents a physical input (keyboard key or mouse button)
   - Can be mapped to multiple actions

### Integration Layer

- **winit_integration** module provides helper functions for processing winit events
- `process_window_event()` automatically updates `InputState` from winit events

## Usage Patterns

### Basic Setup

```rust
use bevy_ecs::world::World;
use praxis_input::{InputState, InputMap, Action};
use winit::keyboard::KeyCode;

// Create ECS world and add input resources
let mut world = World::new();
world.insert_resource(InputState::default());

// Configure action mappings
let mut input_map = InputMap::default();
input_map.bind_key(Action::new("jump"), KeyCode::Space);
input_map.bind_key(Action::new("forward"), KeyCode::KeyW);
world.insert_resource(input_map);
```

### Event Processing

In your window event handler:

```rust
use praxis_input::winit_integration;
use winit::event::WindowEvent;

fn handle_window_event(world: &mut World, event: &WindowEvent) {
    let mut input_state = world.get_resource_mut::<InputState>().unwrap();
    winit_integration::process_window_event(&mut input_state, event);
}
```

### Frame Updates

At the beginning of each frame, call `update()` to clear frame-specific state:

```rust
fn begin_frame(world: &mut World) {
    let mut input_state = world.get_resource_mut::<InputState>().unwrap();
    input_state.update();
}
```

### Checking Input in Systems

Direct input queries:

```rust
use bevy_ecs::system::Res;
use praxis_input::InputState;
use winit::keyboard::KeyCode;

fn movement_system(input: Res<InputState>) {
    if input.is_key_pressed(KeyCode::KeyW) {
        // Move forward
    }
    
    if input.is_key_just_pressed(KeyCode::Space) {
        // Jump (triggers once per press)
    }
}
```

Action-based input queries:

```rust
use bevy_ecs::system::Res;
use praxis_input::{InputState, InputMap, Action};

fn player_system(
    input_state: Res<InputState>,
    input_map: Res<InputMap>,
) {
    let forward = Action::new("forward");
    if input_map.is_action_pressed(&forward, &input_state) {
        // Move forward
    }
    
    let jump = Action::new("jump");
    if input_map.is_action_just_pressed(&jump, &input_state) {
        // Jump
    }
}
```

## Input States

The system tracks three states for each input:

1. **Pressed**: Input is currently held down
   - `is_key_pressed()` / `is_mouse_button_pressed()` / `is_action_pressed()`
   
2. **Just Pressed**: Input was pressed this frame
   - `is_key_just_pressed()` / `is_mouse_button_just_pressed()` / `is_action_just_pressed()`
   - Automatically cleared by `update()` at the start of next frame
   
3. **Just Released**: Input was released this frame
   - `is_key_just_released()` / `is_mouse_button_just_released()` / `is_action_just_released()`
   - Automatically cleared by `update()` at the start of next frame

## Action Mapping

### Creating Action Bindings

```rust
use praxis_input::{InputMap, Action, MouseButton};
use winit::keyboard::KeyCode;

let mut input_map = InputMap::default();

// Bind single input to action
input_map.bind_key(Action::new("jump"), KeyCode::Space);

// Bind multiple inputs to same action
input_map.bind_key(Action::new("fire"), KeyCode::ControlLeft);
input_map.bind_mouse_button(Action::new("fire"), MouseButton::Left);

// Movement actions
input_map.bind_key(Action::new("forward"), KeyCode::KeyW);
input_map.bind_key(Action::new("backward"), KeyCode::KeyS);
input_map.bind_key(Action::new("left"), KeyCode::KeyA);
input_map.bind_key(Action::new("right"), KeyCode::KeyD);
```

### Modifying Bindings

```rust
// Remove specific binding
input_map.unbind_key(&Action::new("jump"), KeyCode::Space);

// Remove all bindings for an action
input_map.unbind_all(&Action::new("jump"));

// Clear all bindings
input_map.clear();
```

### Querying Bindings

```rust
// Get all bindings for an action
if let Some(bindings) = input_map.get_bindings(&Action::new("jump")) {
    for binding in bindings {
        println!("Jump bound to: {:?}", binding);
    }
}

// Get all actions for a binding
use praxis_input::InputBinding;
let binding = InputBinding::Key(KeyCode::Space);
if let Some(actions) = input_map.get_actions_for_binding(&binding) {
    for action_id in actions {
        println!("Space triggers: {}", action_id);
    }
}
```

## Mouse Input

### Position and Delta

```rust
let mouse_pos = input_state.mouse_position();  // (x, y) in pixels
let mouse_delta = input_state.mouse_delta();   // (dx, dy) since last frame
```

### Scroll Wheel

```rust
let scroll = input_state.scroll_delta();  // (horizontal, vertical)
if scroll.1 > 0.0 {
    // Scrolled up
} else if scroll.1 < 0.0 {
    // Scrolled down
}
```

## Best Practices

1. **Call `update()` once per frame** at the beginning of your game loop
2. **Use actions for gameplay code** instead of hardcoding keys
3. **Direct input queries** are fine for debug controls or UI
4. **Store `Action` instances** if checking the same action multiple times per frame
5. **Consider input priorities** when multiple systems need input
6. **Clear input when switching game states** using `input_state.clear()`

## Examples

See the following examples for complete demonstrations:

- `examples/input_demo.rs` - Basic input functionality showcase
- `examples/input_integration.rs` - Full integration with winit and ECS

Run with:
```bash
cargo run --example input_demo
cargo run --example input_integration
```

## Future Extensions

The input system is designed to be extended with:

- Gamepad support (Xbox, PlayStation, generic)
- Input recording and playback
- Input buffering for fighting games
- Dead zones and sensitivity curves
- Chord/combo detection
- Context-based input maps (different bindings per game state)
