# Input System

Keyboard, mouse, and gamepad input handling in Praxis.

## Core Resource

### InputState
Global resource tracking all input state:

```rust
#[derive(Resource)]
pub struct InputState {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
    pub gamepads: GamepadState,
}
```

Access in systems:

```rust
fn player_system(input: Res<InputState>) {
    if input.keyboard.pressed(KeyCode::Space) {
        // Jump
    }
}
```

## Keyboard Input

### Key States

| Method | Description |
|--------|-------------|
| `pressed(key)` | Key is currently held down |
| `just_pressed(key)` | Key was pressed this frame |
| `just_released(key)` | Key was released this frame |

```rust
// Movement
if input.keyboard.pressed(KeyCode::KeyW) {
    velocity.z -= speed;
}
if input.keyboard.pressed(KeyCode::KeyS) {
    velocity.z += speed;
}

// One-shot actions
if input.keyboard.just_pressed(KeyCode::Space) {
    jump();
}
```

### Modifier Keys

```rust
let shift = input.keyboard.pressed(KeyCode::ShiftLeft)
         || input.keyboard.pressed(KeyCode::ShiftRight);
let ctrl = input.keyboard.pressed(KeyCode::ControlLeft)
        || input.keyboard.pressed(KeyCode::ControlRight);

if ctrl && input.keyboard.just_pressed(KeyCode::KeyS) {
    save();
}
```

## Mouse Input

### Position and Delta

```rust
pub struct MouseState {
    pub position: Vec2,      // Screen coordinates
    pub delta: Vec2,         // Movement since last frame
    pub scroll_delta: f32,   // Scroll wheel
    buttons: ButtonState,
}
```

```rust
// Camera look
let sensitivity = 0.005;
camera.yaw -= input.mouse.delta.x * sensitivity;
camera.pitch -= input.mouse.delta.y * sensitivity;

// Zoom
camera.distance -= input.mouse.scroll_delta * zoom_speed;
```

### Mouse Buttons

```rust
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

// Selection
if input.mouse.just_pressed(MouseButton::Left) {
    select_at(input.mouse.position);
}

// Context menu
if input.mouse.just_pressed(MouseButton::Right) {
    show_context_menu();
}
```

## Gamepad Input

Powered by `gilrs` for cross-platform gamepad support.

### Buttons

```rust
if input.gamepads.button_pressed(0, GamepadButton::South) {
    // A button (Xbox) / X button (PlayStation)
    jump();
}

if input.gamepads.button_just_pressed(0, GamepadButton::Start) {
    pause_game();
}
```

### Axes

```rust
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

// Movement from left stick
let move_x = input.gamepads.axis_value(0, GamepadAxis::LeftStickX);
let move_y = input.gamepads.axis_value(0, GamepadAxis::LeftStickY);
velocity = Vec3::new(move_x, 0.0, move_y) * speed;

// Camera from right stick
let look_x = input.gamepads.axis_value(0, GamepadAxis::RightStickX);
let look_y = input.gamepads.axis_value(0, GamepadAxis::RightStickY);
```

### Deadzone

Axes have configurable deadzones to prevent drift:

| Axis | Default Deadzone |
|------|------------------|
| Left Stick | 0.15 |
| Right Stick | 0.15 |
| Triggers | 0.1 |

## Input Actions

Higher-level abstraction mapping inputs to game actions:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    MoveForward,
    MoveBack,
    Jump,
    Attack,
    Interact,
}

pub struct InputMap {
    bindings: HashMap<Action, Vec<InputBinding>>,
}

pub enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
    GamepadAxis { axis: GamepadAxis, positive: bool },
}
```

```rust
// Setup
let mut input_map = InputMap::new();
input_map.bind(Action::Jump, InputBinding::Key(KeyCode::Space));
input_map.bind(Action::Jump, InputBinding::GamepadButton(GamepadButton::South));

// Usage
if input_map.just_pressed(Action::Jump, &input) {
    jump();
}
```

## Winit Integration

Input events from winit are processed into `InputState`:

```rust
// In event handler
match event {
    WindowEvent::KeyboardInput { event, .. } => {
        input_state.keyboard.process_key_event(event);
    }
    WindowEvent::MouseInput { button, state, .. } => {
        input_state.mouse.process_button(button, state);
    }
    WindowEvent::CursorMoved { position, .. } => {
        input_state.mouse.update_position(position);
    }
    // ...
}
```

## Frame Update

Input state should be updated each frame:

```rust
// At frame start
input_state.begin_frame();

// Process events...

// At frame end
input_state.end_frame();  // Clears just_pressed/just_released
```

## Usage Example

```rust
use praxis_input::{InputState, KeyCode, MouseButton};

fn fps_controller(
    input: Res<InputState>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let (mut transform, mut velocity) = query.single_mut();

    // WASD movement
    let mut move_dir = Vec3::ZERO;
    if input.keyboard.pressed(KeyCode::KeyW) { move_dir.z -= 1.0; }
    if input.keyboard.pressed(KeyCode::KeyS) { move_dir.z += 1.0; }
    if input.keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
    if input.keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }

    velocity.0 = move_dir.normalize_or_zero() * MOVE_SPEED;

    // Mouse look
    transform.rotation *= Quat::from_rotation_y(-input.mouse.delta.x * SENSITIVITY);

    // Sprint
    if input.keyboard.pressed(KeyCode::ShiftLeft) {
        velocity.0 *= 2.0;
    }
}
```

## See Also

- [praxis_input crate](../../crates/praxis_input/README.md) - API documentation
- [input_integration example](../../examples/input_integration.rs) - Basic input handling
- [fps_camera_controller example](../../examples/fps_camera_controller.rs) - FPS controls
