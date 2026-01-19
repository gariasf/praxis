# Input Guide

Practical guide to handling keyboard, mouse, and gamepad input in Praxis for player controls and UI interactions.

## Quick Start

### Initialize Input System

```rust
use praxis_input::InputState;
use praxis_ecs::{World, Schedule};

let mut world = World::new();

// Initialize input state
let input_state = InputState::new();
world.insert_resource(input_state);
```

### Process Input Events

In your main loop, update input state with window events:

```rust
use winit::event::{Event, WindowEvent};

match event {
    Event::WindowEvent { event, .. } => {
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
            WindowEvent::MouseWheel { delta, .. } => {
                input_state.mouse.process_scroll(delta);
            }
            _ => {}
        }
    }
    _ => {}
}
```

### Frame Management

```rust
// At the start of each frame
input_state.begin_frame();

// Process events...

// At the end of each frame
input_state.end_frame();
```

## Keyboard Input

### Key State Checks

```rust
use praxis_input::KeyCode;

fn player_movement(input: Res<InputState>, mut query: Query<&mut Velocity>) {
    for mut velocity in query.iter_mut() {
        // Held down (every frame while pressed)
        if input.keyboard.pressed(KeyCode::KeyW) {
            velocity.z -= 1.0;
        }
        
        // Just pressed (only first frame)
        if input.keyboard.just_pressed(KeyCode::Space) {
            velocity.y = 10.0;  // Jump
        }
        
        // Just released (only frame it was released)
        if input.keyboard.just_released(KeyCode::ShiftLeft) {
            println!("Stopped sprinting");
        }
    }
}
```

### Common Key Codes

```rust
// Movement
KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD

// Actions
KeyCode::Space      // Jump
KeyCode::KeyE       // Interact
KeyCode::KeyR       // Reload
KeyCode::Escape     // Menu

// Modifiers
KeyCode::ShiftLeft, KeyCode::ShiftRight
KeyCode::ControlLeft, KeyCode::ControlRight
KeyCode::AltLeft, KeyCode::AltRight

// Function keys
KeyCode::F1, KeyCode::F2, ..., KeyCode::F12

// Numbers
KeyCode::Digit1, KeyCode::Digit2, ..., KeyCode::Digit0
```

### Modifier Keys

```rust
fn handle_shortcuts(input: Res<InputState>) {
    let ctrl = input.keyboard.pressed(KeyCode::ControlLeft) 
            || input.keyboard.pressed(KeyCode::ControlRight);
    let shift = input.keyboard.pressed(KeyCode::ShiftLeft) 
             || input.keyboard.pressed(KeyCode::ShiftRight);
    
    // Ctrl+S: Save
    if ctrl && input.keyboard.just_pressed(KeyCode::KeyS) {
        save_game();
    }
    
    // Shift+Click: Special action
    if shift && input.mouse.just_pressed(MouseButton::Left) {
        special_action();
    }
}
```

## Mouse Input

### Mouse Buttons

```rust
use praxis_input::MouseButton;

fn mouse_actions(input: Res<InputState>) {
    // Left click
    if input.mouse.just_pressed(MouseButton::Left) {
        select_object();
    }
    
    // Right click
    if input.mouse.just_pressed(MouseButton::Right) {
        show_context_menu();
    }
    
    // Middle click
    if input.mouse.pressed(MouseButton::Middle) {
        pan_camera();
    }
}
```

### Mouse Position

```rust
fn check_mouse_position(input: Res<InputState>) {
    let pos = input.mouse.position();  // Vec2
    
    println!("Mouse at ({}, {})", pos.x, pos.y);
    
    // Check if in specific region
    if pos.x > 100.0 && pos.x < 200.0 {
        highlight_ui_element();
    }
}
```

### Mouse Movement (Delta)

```rust
fn camera_look(
    input: Res<InputState>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let delta = input.mouse.delta();  // Vec2
        
        let sensitivity = 0.002;
        transform.rotate_y(-delta.x * sensitivity);
        transform.rotate_x(-delta.y * sensitivity);
    }
}
```

### Mouse Scroll

```rust
fn camera_zoom(
    input: Res<InputState>,
    mut query: Query<&mut Camera>,
) {
    for mut camera in query.iter_mut() {
        let scroll = input.mouse.scroll_delta();
        
        camera.distance -= scroll * 0.5;
        camera.distance = camera.distance.clamp(2.0, 50.0);
    }
}
```

## Gamepad Input

### Gamepad Buttons

```rust
use praxis_input::GamepadButton;

fn gamepad_controls(input: Res<InputState>) {
    let gamepad_id = 0;  // First connected gamepad
    
    // Face buttons
    if input.gamepads.button_just_pressed(gamepad_id, GamepadButton::South) {
        jump();  // A on Xbox, X on PlayStation
    }
    
    if input.gamepads.button_pressed(gamepad_id, GamepadButton::West) {
        sprint();  // X on Xbox, Square on PlayStation
    }
    
    // Shoulder buttons
    if input.gamepads.button_just_pressed(gamepad_id, GamepadButton::LeftBumper) {
        previous_weapon();
    }
    
    // Menu buttons
    if input.gamepads.button_just_pressed(gamepad_id, GamepadButton::Start) {
        pause_game();
    }
}
```

### Analog Sticks

```rust
use praxis_input::GamepadAxis;

fn gamepad_movement(
    input: Res<InputState>,
    mut query: Query<&mut Velocity>,
) {
    let gamepad_id = 0;
    
    for mut velocity in query.iter_mut() {
        // Left stick: movement
        let move_x = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickX);
        let move_y = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickY);
        
        velocity.x = move_x * 5.0;
        velocity.z = move_y * 5.0;
    }
}

fn gamepad_camera(
    input: Res<InputState>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    let gamepad_id = 0;
    
    for mut transform in query.iter_mut() {
        // Right stick: camera
        let look_x = input.gamepads.axis_value(gamepad_id, GamepadAxis::RightStickX);
        let look_y = input.gamepads.axis_value(gamepad_id, GamepadAxis::RightStickY);
        
        let sensitivity = 0.05;
        transform.rotate_y(-look_x * sensitivity);
        transform.rotate_x(-look_y * sensitivity);
    }
}
```

### Triggers

```rust
fn gamepad_triggers(input: Res<InputState>) {
    let gamepad_id = 0;
    
    // Left trigger: aim
    let aim = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftTrigger);
    if aim > 0.5 {
        start_aiming();
    }
    
    // Right trigger: shoot
    let shoot = input.gamepads.axis_value(gamepad_id, GamepadAxis::RightTrigger);
    if shoot > 0.9 {
        fire_weapon();
    }
}
```

## Common Patterns

### FPS Controller

```rust
#[derive(Component)]
struct FpsController {
    move_speed: f32,
    sprint_speed: f32,
    jump_force: f32,
    mouse_sensitivity: f32,
}

fn fps_controller(
    input: Res<InputState>,
    mut query: Query<(&FpsController, &mut Transform, &mut Velocity)>,
) {
    for (controller, mut transform, mut velocity) in query.iter_mut() {
        // Mouse look
        let mouse_delta = input.mouse.delta();
        transform.rotate_y(-mouse_delta.x * controller.mouse_sensitivity);
        transform.rotate_x(-mouse_delta.y * controller.mouse_sensitivity);
        
        // WASD movement
        let mut move_dir = Vec3::ZERO;
        if input.keyboard.pressed(KeyCode::KeyW) { move_dir.z -= 1.0; }
        if input.keyboard.pressed(KeyCode::KeyS) { move_dir.z += 1.0; }
        if input.keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
        if input.keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }
        
        // Apply movement
        if move_dir.length() > 0.0 {
            move_dir = move_dir.normalize();
            
            let speed = if input.keyboard.pressed(KeyCode::ShiftLeft) {
                controller.sprint_speed
            } else {
                controller.move_speed
            };
            
            velocity.linear = transform.rotation * (move_dir * speed);
        }
        
        // Jump
        if input.keyboard.just_pressed(KeyCode::Space) {
            velocity.linear.y = controller.jump_force;
        }
    }
}
```

### Top-Down Controller

```rust
fn top_down_controller(
    input: Res<InputState>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    for mut transform in query.iter_mut() {
        let speed = 5.0;
        let mut direction = Vec3::ZERO;
        
        // Arrow keys or WASD
        if input.keyboard.pressed(KeyCode::ArrowUp) 
        || input.keyboard.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if input.keyboard.pressed(KeyCode::ArrowDown) 
        || input.keyboard.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if input.keyboard.pressed(KeyCode::ArrowLeft) 
        || input.keyboard.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if input.keyboard.pressed(KeyCode::ArrowRight) 
        || input.keyboard.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }
        
        if direction.length() > 0.0 {
            transform.translation += direction.normalize() * speed * time.delta_seconds();
        }
    }
}
```

### Vehicle Controls

```rust
fn vehicle_controls(
    input: Res<InputState>,
    mut query: Query<&mut Vehicle>,
) {
    for mut vehicle in query.iter_mut() {
        // Keyboard controls
        if input.keyboard.pressed(KeyCode::KeyW) {
            vehicle.throttle = 1.0;
        } else if input.keyboard.pressed(KeyCode::KeyS) {
            vehicle.throttle = -1.0;
        } else {
            vehicle.throttle = 0.0;
        }
        
        if input.keyboard.pressed(KeyCode::KeyA) {
            vehicle.steering = -1.0;
        } else if input.keyboard.pressed(KeyCode::KeyD) {
            vehicle.steering = 1.0;
        } else {
            vehicle.steering = 0.0;
        }
        
        // Gamepad controls (analog)
        let gamepad_id = 0;
        if input.gamepads.is_connected(gamepad_id) {
            vehicle.throttle = input.gamepads.axis_value(
                gamepad_id, 
                GamepadAxis::RightTrigger
            ) - input.gamepads.axis_value(
                gamepad_id, 
                GamepadAxis::LeftTrigger
            );
            
            vehicle.steering = input.gamepads.axis_value(
                gamepad_id, 
                GamepadAxis::LeftStickX
            );
        }
    }
}
```

### Click to Move

```rust
fn click_to_move(
    input: Res<InputState>,
    camera: Query<(&Camera, &Transform)>,
    mut player: Query<&mut MovementTarget, With<Player>>,
) {
    if input.mouse.just_pressed(MouseButton::Left) {
        let mouse_pos = input.mouse.position();
        let (camera, camera_transform) = camera.single();
        
        // Raycast from mouse to world
        if let Some(world_pos) = screen_to_world(mouse_pos, camera, camera_transform) {
            for mut target in player.iter_mut() {
                target.position = world_pos;
                target.active = true;
            }
        }
    }
}
```

### Hotbar Slots

```rust
fn hotbar_input(
    input: Res<InputState>,
    mut inventory: ResMut<Inventory>,
) {
    // Number keys for quick slots
    for i in 0..9 {
        let key = match i {
            0 => KeyCode::Digit1,
            1 => KeyCode::Digit2,
            2 => KeyCode::Digit3,
            3 => KeyCode::Digit4,
            4 => KeyCode::Digit5,
            5 => KeyCode::Digit6,
            6 => KeyCode::Digit7,
            7 => KeyCode::Digit8,
            8 => KeyCode::Digit9,
            _ => continue,
        };
        
        if input.keyboard.just_pressed(key) {
            inventory.select_slot(i);
        }
    }
}
```

## Input Action Mapping

Create higher-level actions from raw input:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    Jump,
    Attack,
    Interact,
    Pause,
}

struct InputMap {
    bindings: HashMap<Action, Vec<InputBinding>>,
}

enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
}

impl InputMap {
    fn is_pressed(&self, action: Action, input: &InputState) -> bool {
        if let Some(bindings) = self.bindings.get(&action) {
            bindings.iter().any(|binding| match binding {
                InputBinding::Key(key) => input.keyboard.pressed(*key),
                InputBinding::MouseButton(btn) => input.mouse.pressed(*btn),
                InputBinding::GamepadButton(btn) => {
                    input.gamepads.button_pressed(0, *btn)
                }
            })
        } else {
            false
        }
    }
}

// Usage
fn setup_input_map() -> InputMap {
    let mut map = InputMap { bindings: HashMap::new() };
    
    map.bindings.insert(Action::Jump, vec![
        InputBinding::Key(KeyCode::Space),
        InputBinding::GamepadButton(GamepadButton::South),
    ]);
    
    map.bindings.insert(Action::Attack, vec![
        InputBinding::MouseButton(MouseButton::Left),
        InputBinding::GamepadButton(GamepadButton::West),
    ]);
    
    map
}
```

## Multi-Player Input

Handle multiple gamepads for local multiplayer:

```rust
fn multiplayer_input(
    input: Res<InputState>,
    mut players: Query<(&PlayerId, &mut Transform)>,
) {
    for (player_id, mut transform) in players.iter_mut() {
        let gamepad_id = player_id.0;
        
        if !input.gamepads.is_connected(gamepad_id) {
            continue;
        }
        
        let move_x = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickX);
        let move_y = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickY);
        
        transform.translation.x += move_x * 0.1;
        transform.translation.z += move_y * 0.1;
    }
}
```

## Debugging

### Show Input State

```rust
fn debug_input(input: Res<InputState>) {
    // Log pressed keys
    for key in input.keyboard.pressed_keys() {
        tracing::debug!("Key pressed: {:?}", key);
    }
    
    // Log mouse info
    let mouse_pos = input.mouse.position();
    let mouse_delta = input.mouse.delta();
    tracing::debug!("Mouse: pos={:?}, delta={:?}", mouse_pos, mouse_delta);
    
    // Log gamepad axes
    for gamepad_id in 0..4 {
        if input.gamepads.is_connected(gamepad_id) {
            tracing::debug!("Gamepad {}: LStick=({:.2}, {:.2})", 
                gamepad_id,
                input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickX),
                input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickY)
            );
        }
    }
}
```

### On-Screen Input Display

```rust
fn display_input_state(
    input: Res<InputState>,
    mut debug_text: ResMut<DebugText>,
) {
    debug_text.clear();
    
    debug_text.add_line(format!("Mouse: {:?}", input.mouse.position()));
    debug_text.add_line(format!("Scroll: {:.2}", input.mouse.scroll_delta()));
    
    if input.keyboard.pressed(KeyCode::KeyW) {
        debug_text.add_line("W: PRESSED");
    }
}
```

## Performance Tips

### Input Buffering

Buffer inputs for tight timing windows:

```rust
#[derive(Resource)]
struct InputBuffer {
    jump_buffer: f32,
}

fn buffer_input(
    input: Res<InputState>,
    time: Res<Time>,
    mut buffer: ResMut<InputBuffer>,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        buffer.jump_buffer = 0.2;  // 200ms buffer
    }
    
    buffer.jump_buffer = (buffer.jump_buffer - time.delta_seconds()).max(0.0);
}

fn consume_buffered_jump(mut buffer: ResMut<InputBuffer>) -> bool {
    if buffer.jump_buffer > 0.0 {
        buffer.jump_buffer = 0.0;
        true
    } else {
        false
    }
}
```

### Deadzone Configuration

```rust
fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone {
        0.0
    } else {
        (value - deadzone * value.signum()) / (1.0 - deadzone)
    }
}

fn gamepad_with_deadzone(input: Res<InputState>, gamepad_id: usize) -> Vec2 {
    let x = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickX);
    let y = input.gamepads.axis_value(gamepad_id, GamepadAxis::LeftStickY);
    
    let deadzone = 0.15;
    Vec2::new(
        apply_deadzone(x, deadzone),
        apply_deadzone(y, deadzone)
    )
}
```

## Troubleshooting

### Input Not Working

**Problem**: Key presses don't register

**Solutions**:
- Verify `InputState` resource exists
- Check event processing loop is running
- Ensure `begin_frame()` and `end_frame()` are called
- Confirm correct `KeyCode` enum values

### Just Pressed Fires Multiple Times

**Problem**: `just_pressed` triggers repeatedly

**Solutions**:
- Ensure `end_frame()` is called each frame
- Check frame timing isn't skipping
- Verify only one input system is running

### Gamepad Not Detected

**Problem**: Gamepad input doesn't work

**Solutions**:
- Check gamepad is connected before use
- Use `is_connected()` before reading axes/buttons
- Verify gamepad ID (0-3 typically)
- Test with different gamepad

### Mouse Delta Always Zero

**Problem**: Mouse movement not detected

**Solutions**:
- Ensure cursor is captured/locked
- Check `CursorMoved` events are being processed
- Verify `update_position()` is being called
- Reset delta in `begin_frame()`

## Examples

See working examples:
- `examples/input_integration.rs` - Basic input handling
- `examples/fps_camera_controller.rs` - FPS controls

Run with:
```bash
cargo run --example input_integration
```

## See Also

- [Input Concepts](../concepts/input.md) - Theory and architecture
- [Input API Reference](../reference/input-api.md) - API documentation
- [praxis_input Crate](../../crates/praxis_input/README.md) - Crate documentation
- [winit Documentation](https://docs.rs/winit) - Window and event handling
