# praxis_input

Input handling for Praxis engine: keyboard, mouse, gamepad.

## Overview

Provides unified input abstraction for keyboard, mouse, and gamepad input.

## Features

- **Keyboard**: Key state tracking (pressed, just_pressed, just_released)
- **Mouse**: Position, delta, buttons, scroll
- **Gamepad**: Button and axis mapping, multiple controllers
- **Input Mapping**: Map physical inputs to logical actions
- **Frame-based State**: Clear distinction between current and previous frame

## Example

```rust
use praxis_input::{Input, KeyCode, MouseButton};

// Check keyboard
if input.key_pressed(KeyCode::W) {
    // Move forward
}

if input.key_just_pressed(KeyCode::Space) {
    // Jump
}

// Check mouse
let mouse_delta = input.mouse_delta();
if input.mouse_button_pressed(MouseButton::Left) {
    // Fire weapon
}

// Check gamepad
if let Some(gamepad) = input.gamepads().next() {
    let left_stick = gamepad.left_stick();
    // Move character
}
```

## Input Mapping

```rust
use praxis_input::InputMap;

let mut input_map = InputMap::new();
input_map.bind_key("jump", KeyCode::Space);
input_map.bind_key("jump", KeyCode::ButtonSouth); // Gamepad

if input_map.action_pressed(&input, "jump") {
    // Jump
}
```

## Dependencies

- `winit`: Window events
- `gilrs`: Gamepad support
- `rustc-hash`: Fast hash maps

## Usage

```toml
praxis_input = { path = "../praxis_input", version = "0.1.0" }
```
