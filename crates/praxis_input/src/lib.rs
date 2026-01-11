//! Input system for the Praxis engine.
//!
//! This crate provides functionality for handling keyboard, mouse, and gamepad input,
//! including an action mapping system for rebindable controls.
//!
//! # Architecture
//!
//! The input system is organized around several key components:
//!
//! - **`InputState`**: A resource that tracks the current state of all input devices
//!   (keyboard, mouse, gamepad).
//! - **`InputMap`**: A resource that maps physical inputs to logical actions, enabling
//!   rebindable controls.
//! - **Event handling**: Integration with `winit` for processing raw input events.
//!
//! # Basic Usage
//!
//! ```rust,no_run
//! use praxis_input::{InputState, InputMap, Action};
//! use bevy_ecs::world::World;
//! use winit::keyboard::KeyCode;
//!
//! // Create a world and add input resources
//! let mut world = World::new();
//! world.insert_resource(InputState::default());
//!
//! let mut input_map = InputMap::default();
//! input_map.bind_key(&Action::new("jump"), KeyCode::Space);
//! input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
//! world.insert_resource(input_map);
//!
//! // In your game loop, update input state and check actions
//! // let input_state = world.get_resource::<InputState>().unwrap();
//! // let input_map = world.get_resource::<InputMap>().unwrap();
//! // if input_map.is_action_pressed(&Action::new("jump"), input_state) {
//! //     // Perform jump
//! // }
//! ```
//!
//! # Integration with winit
//!
//! The input system integrates with winit events through the `InputState` resource.
//! Call the appropriate handler methods when processing window events:
//!
//! ```rust,no_run
//! use praxis_input::InputState;
//! use winit::event::WindowEvent;
//!
//! fn handle_event(input_state: &mut InputState, event: &WindowEvent) {
//!     match event {
//!         WindowEvent::KeyboardInput { event, .. } => {
//!             input_state.handle_keyboard_input(event.physical_key, event.state);
//!         }
//!         WindowEvent::MouseInput { state, button, .. } => {
//!             input_state.handle_mouse_button((*button).into(), *state);
//!         }
//!         WindowEvent::CursorMoved { position, .. } => {
//!             input_state.handle_cursor_moved((position.x, position.y));
//!         }
//!         WindowEvent::MouseWheel { delta, .. } => {
//!             // Handle mouse wheel based on delta type
//!         }
//!         _ => {}
//!     }
//! }
//! ```

// ============================================================================
// MODULE ORGANIZATION
// ============================================================================
// The input system is split into focused modules, each handling a specific
// aspect of input processing:
//
// - `action`: Defines logical game actions (e.g., "jump", "fire") that
//   decouple game logic from physical inputs
// - `input_map`: Maps physical inputs to actions, enabling rebindable controls
// - `input_state`: Tracks raw input device state (pressed keys, mouse position)
// - `winit_integration`: Helper functions for processing winit window events

mod action;
mod input_map;
mod input_state;
pub mod winit_integration;

// ============================================================================
// PUBLIC API
// ============================================================================
// Re-export key types to provide a clean, flat API surface for users of this
// crate. Users can import everything they need from `praxis_input::*` without
// navigating the internal module structure.

pub use action::{Action, ActionId};
pub use input_map::{InputBinding, InputMap};
pub use input_state::{InputState, MouseButton};

use praxis_utils::{info, Result};

// ============================================================================
// INPUT STATE MANAGEMENT
// ============================================================================
// The input system uses a dual-phase state tracking pattern:
//
// 1. **Current State**: Tracks which inputs are currently held down
//    - Used for continuous actions (e.g., holding W to move forward)
//    - Persists across frames until the input is released
//
// 2. **Frame State**: Tracks inputs that changed *this frame*
//    - "Just pressed": Input transitioned from up to down this frame
//    - "Just released": Input transitioned from down to up this frame
//    - Used for discrete actions (e.g., jump on button press)
//    - Cleared at the start of each frame by calling `InputState::update()`
//
// This pattern is essential for game input because it allows you to:
// - Detect the exact frame an input occurs (for responsive controls)
// - Hold inputs for continuous actions (for movement, aiming, etc.)
// - Avoid processing the same input multiple times
//
// Example:
// ```rust,no_run
// # use praxis_input::InputState;
// # use winit::keyboard::KeyCode;
// # let mut input = InputState::default();
// // Frame 1: Player presses Space
// input.press_key(KeyCode::Space);
// assert!(input.is_key_pressed(KeyCode::Space));        // true - held down
// assert!(input.is_key_just_pressed(KeyCode::Space));   // true - just pressed
//
// // Frame 2: Space is still held, but not just pressed
// input.update(); // Clear frame state
// assert!(input.is_key_pressed(KeyCode::Space));        // true - still held
// assert!(input.is_key_just_pressed(KeyCode::Space));   // false - not new
//
// // Frame 3: Player releases Space
// input.release_key(KeyCode::Space);
// assert!(!input.is_key_pressed(KeyCode::Space));       // false - released
// assert!(input.is_key_just_released(KeyCode::Space));  // true - just released
// ```

// ============================================================================
// KEYBOARD AND MOUSE HANDLING
// ============================================================================
// The `InputState` resource tracks keyboard and mouse input using HashSets
// for efficient O(1) lookups:
//
// **Keyboard:**
// - Uses winit's `KeyCode` enum for physical key identification
// - Tracks key state independently of text input (physical keys, not characters)
// - Supports checking if any key is pressed via `pressed_keys()` iterator
//
// **Mouse:**
// - Tracks button state (left, right, middle, and extra buttons)
// - Tracks absolute cursor position in window coordinates
// - Tracks cursor delta (movement since last frame) for FPS camera controls
// - Tracks scroll wheel delta (horizontal and vertical) for zooming/scrolling
//
// All mouse data is reset each frame via `update()`, except the absolute
// position which persists and is used to compute the next frame's delta.
//
// Example usage patterns:
// ```rust,no_run
// # use praxis_input::{InputState, MouseButton};
// # use winit::keyboard::KeyCode;
// # let input = InputState::default();
// // Movement with WASD
// if input.is_key_pressed(KeyCode::KeyW) { /* move forward */ }
// if input.is_key_pressed(KeyCode::KeyS) { /* move backward */ }
//
// // Jump only on button press (not every frame it's held)
// if input.is_key_just_pressed(KeyCode::Space) { /* jump */ }
//
// // FPS camera control with mouse delta
// let (dx, dy) = input.mouse_delta();
// // camera.rotate(dx, dy);
//
// // Zoom with scroll wheel
// let (_horizontal, vertical) = input.scroll_delta();
// // camera.zoom(vertical);
//
// // Fire weapon on mouse click
// if input.is_mouse_button_just_pressed(MouseButton::Left) { /* fire */ }
// ```

// ============================================================================
// GAMEPAD HANDLING
// ============================================================================
// **Note:** Gamepad support is currently planned but not yet implemented.
// The architecture is designed to accommodate gamepads in the future by:
// - Adding a `GamepadButton` enum (similar to `MouseButton`)
// - Adding gamepad state tracking to `InputState`
// - Adding `InputBinding::GamepadButton` variant
// - Supporting analog stick and trigger inputs with axis values
//
// The action mapping system (`InputMap`) will seamlessly support gamepads
// once implemented, allowing the same actions to be triggered by keyboard,
// mouse, or gamepad inputs.

// ============================================================================
// INPUT MAPPING ABSTRACTION
// ============================================================================
// The input mapping system provides a layer of indirection between physical
// inputs and game logic through the `Action` and `InputMap` types:
//
// **Why Use Input Mapping?**
// 1. **Rebindable Controls**: Players can customize their key bindings
// 2. **Multi-Input Support**: Multiple keys can trigger the same action
//    (e.g., Space and W both for jump)
// 3. **Platform Independence**: Different platforms can have different default
//    bindings while using the same game logic
// 4. **Code Clarity**: Game code uses semantic action names ("jump") rather
//    than physical keys (KeyCode::Space)
//
// **Action-Based Architecture:**
// - `Action`: A logical game operation (e.g., "jump", "fire", "menu_select")
// - `ActionId`: The unique identifier for an action (string-based)
// - `InputBinding`: A physical input (keyboard key or mouse button)
// - `InputMap`: The bidirectional mapping between actions and bindings
//
// The `InputMap` maintains two hash maps for efficient lookups:
// 1. `action_bindings`: Action -> Set of inputs that trigger it
// 2. `binding_actions`: Input -> Set of actions it triggers
//
// Example:
// ```rust,no_run
// # use praxis_input::{InputMap, Action, InputState};
// # use winit::keyboard::KeyCode;
// let mut input_map = InputMap::default();
// let jump = Action::new("jump");
//
// // Bind multiple inputs to the same action
// input_map.bind_key(&jump, KeyCode::Space);
// input_map.bind_key(&jump, KeyCode::KeyW);
// input_map.bind_key(&jump, KeyCode::GamepadSouth); // Future gamepad support
//
// // In game logic, check the action instead of specific keys
// # let input_state = InputState::default();
// if input_map.is_action_just_pressed(&jump, &input_state) {
//     // This triggers if ANY bound input is pressed
//     // player.jump();
// }
//
// // Support rebinding at runtime
// input_map.unbind_key(&jump, KeyCode::Space);
// input_map.bind_key(&jump, KeyCode::KeyE); // Player prefers E for jump
// ```

// ============================================================================
// EVENT PROCESSING PATTERNS
// ============================================================================
// The input system follows a specific event processing pattern to ensure
// correct behavior across frame boundaries:
//
// **1. Start of Frame: Update Input State**
// Call `InputState::update()` to clear frame-specific state (just pressed/released)
// while preserving continuous state (currently pressed).
//
// **2. Event Processing: Handle All Events**
// Process all winit events that occurred since last frame:
// - `WindowEvent::KeyboardInput` -> `handle_keyboard_input()`
// - `WindowEvent::MouseInput` -> `handle_mouse_button()`
// - `WindowEvent::CursorMoved` -> `handle_cursor_moved()`
// - `WindowEvent::MouseWheel` -> `handle_mouse_wheel()`
//
// **3. Game Logic: Query Input State**
// Systems query the `InputState` and `InputMap` to make gameplay decisions.
//
// **4. End of Frame: Render and Repeat**
//
// Example event loop integration:
// ```rust,no_run
// # use praxis_input::{InputState, winit_integration};
// # use bevy_ecs::world::World;
// # use winit::event::{Event, WindowEvent};
// # use winit::event_loop::{EventLoop, ControlFlow};
// # let event_loop = EventLoop::new().unwrap();
// # let mut world = World::new();
// # world.insert_resource(InputState::default());
// // In your main loop
// event_loop.run(move |event, elwt| {
//     match event {
//         Event::NewEvents(_) => {
//             // 1. Clear frame state
//             let mut input_state = world.resource_mut::<InputState>();
//             input_state.update();
//         }
//         Event::WindowEvent { event, .. } => {
//             // 2. Process input events
//             let mut input_state = world.resource_mut::<InputState>();
//             winit_integration::process_window_event(&mut input_state, &event);
//         }
//         Event::AboutToWait => {
//             // 3. Run game systems that query input
//             // run_game_systems(&mut world);
//             // 4. Render
//             // render(&world);
//         }
//         _ => {}
//     }
// });
// ```
//
// **Important:** Always call `update()` before processing new events to avoid
// stale "just pressed" states bleeding across frames.

// ============================================================================
// INTEGRATION WITH WINIT EVENTS
// ============================================================================
// The `winit_integration` module provides helper functions to bridge winit's
// event system with our input state tracking:
//
// **Event Type Mapping:**
// - `WindowEvent::KeyboardInput` -> Physical key presses/releases
//   - Uses `PhysicalKey::Code(KeyCode)` to identify keys by location
//   - Ignores `ElementState::Pressed` repeats (OS key repeat)
// - `WindowEvent::MouseInput` -> Mouse button presses/releases
//   - Converts `winit::MouseButton` to our `MouseButton` enum
// - `WindowEvent::CursorMoved` -> Absolute mouse position in window
//   - Position is in physical pixels (multiply by DPI scale for logical coords)
// - `WindowEvent::MouseWheel` -> Scroll wheel movement
//   - Handles both `LineDelta` (discrete clicks) and `PixelDelta` (precise)
//   - Normalizes pixel delta by dividing by 120.0 (standard scroll unit)
//
// **Two Usage Patterns:**
//
// 1. **Automatic (Recommended):** Use the provided helper function
// ```rust,no_run
// # use praxis_input::{InputState, winit_integration};
// # use winit::event::WindowEvent;
// # let mut input_state = InputState::default();
// # let event = unsafe { std::mem::zeroed::<WindowEvent>() };
// winit_integration::process_window_event(&mut input_state, &event);
// ```
//
// 2. **Manual (More Control):** Call specific handlers
// ```rust,no_run
// # use praxis_input::InputState;
// # use winit::event::WindowEvent;
// # let mut input_state = InputState::default();
// # let event = unsafe { std::mem::zeroed::<WindowEvent>() };
// match event {
//     WindowEvent::KeyboardInput { event, .. } => {
//         input_state.handle_keyboard_input(event.physical_key, event.state);
//         // Custom handling here
//     }
//     // ... other events
//     _ => {}
// }
// ```
//
// **Key Repeat Handling:**
// Winit sends repeated `KeyboardInput` events while a key is held (OS key
// repeat). The `InputState` automatically deduplicates these by checking if
// the key is already pressed before setting `just_pressed`. This ensures
// `is_key_just_pressed()` only returns true on the initial press.

/// Initializes the input system.
///
/// This function sets up any necessary global state for the input system.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Example
///
/// ```rust,no_run
/// praxis_input::init().expect("Failed to initialize input system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    info!("Initializing input system");
    Ok(())
}
