//! Input system demonstration example.
//!
//! This example shows how to use the input system with action mappings
//! and direct input state queries.

use praxis_input::{Action, InputMap, InputState, MouseButton};
use winit::keyboard::KeyCode;

fn main() {
    println!("=== Praxis Input System Demo ===\n");

    let mut input_state = InputState::default();
    let mut input_map = InputMap::default();

    setup_input_bindings(&mut input_map);

    println!("Input bindings configured:");
    for (action_id, bindings) in input_map.iter() {
        println!("  {}: {:?}", action_id, bindings);
    }
    println!();

    demo_keyboard_input(&mut input_state, &input_map);
    demo_mouse_input(&mut input_state, &input_map);
    demo_frame_update(&mut input_state);

    println!("\n=== Demo Complete ===");
}

fn setup_input_bindings(input_map: &mut InputMap) {
    let jump = Action::new("jump");
    input_map.bind_key(&jump, KeyCode::Space);
    input_map.bind_key(&jump, KeyCode::KeyW);

    let fire = Action::new("fire");
    input_map.bind_mouse_button(&fire, MouseButton::Left);
    input_map.bind_key(&fire, KeyCode::ControlLeft);

    input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
    input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
    input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
    input_map.bind_key(&Action::new("right"), KeyCode::KeyD);

    input_map.bind_key(&Action::new("menu"), KeyCode::Escape);
}

fn demo_keyboard_input(input_state: &mut InputState, input_map: &InputMap) {
    println!("--- Keyboard Input Demo ---");

    input_state.press_key(KeyCode::Space);
    println!("Pressed Space key");

    if input_state.is_key_pressed(KeyCode::Space) {
        println!("  ✓ Space is pressed");
    }

    if input_state.is_key_just_pressed(KeyCode::Space) {
        println!("  ✓ Space was just pressed this frame");
    }

    let jump = Action::new("jump");
    if input_map.is_action_pressed(&jump, input_state) {
        println!("  ✓ 'jump' action is active");
    }

    if input_map.is_action_just_pressed(&jump, input_state) {
        println!("  ✓ 'jump' action was just activated");
    }

    println!("Releasing Space key");
    input_state.release_key(KeyCode::Space);

    if !input_state.is_key_pressed(KeyCode::Space) {
        println!("  ✓ Space is no longer pressed");
    }

    if input_state.is_key_just_released(KeyCode::Space) {
        println!("  ✓ Space was just released this frame");
    }

    println!();
}

fn demo_mouse_input(input_state: &mut InputState, input_map: &InputMap) {
    println!("--- Mouse Input Demo ---");

    input_state.press_mouse_button(MouseButton::Left);
    println!("Pressed left mouse button");

    if input_state.is_mouse_button_pressed(MouseButton::Left) {
        println!("  ✓ Left mouse button is pressed");
    }

    let fire = Action::new("fire");
    if input_map.is_action_pressed(&fire, input_state) {
        println!("  ✓ 'fire' action is active via mouse");
    }

    input_state.handle_cursor_moved((100.0, 200.0));
    let pos = input_state.mouse_position();
    println!("Mouse moved to ({}, {})", pos.0, pos.1);

    let delta = input_state.mouse_delta();
    println!("  Mouse delta: ({}, {})", delta.0, delta.1);

    input_state.handle_mouse_wheel((0.0, 1.5));
    let scroll = input_state.scroll_delta();
    println!("Mouse wheel scrolled: ({}, {})", scroll.0, scroll.1);

    println!();
}

fn demo_frame_update(input_state: &mut InputState) {
    println!("--- Frame Update Demo ---");

    input_state.press_key(KeyCode::KeyW);
    println!("Frame 1: Pressed W key");

    if input_state.is_key_just_pressed(KeyCode::KeyW) {
        println!("  ✓ W just pressed in frame 1");
    }

    input_state.update();
    println!("\nFrame 2: Called update()");

    if !input_state.is_key_just_pressed(KeyCode::KeyW) {
        println!("  ✓ W is no longer 'just pressed' after update");
    }

    if input_state.is_key_pressed(KeyCode::KeyW) {
        println!("  ✓ But W is still pressed");
    }

    println!();
}
