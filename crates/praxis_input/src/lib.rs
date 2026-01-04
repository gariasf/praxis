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

mod action;
mod input_map;
mod input_state;
pub mod winit_integration;

pub use action::{Action, ActionId};
pub use input_map::{InputBinding, InputMap};
pub use input_state::{InputState, MouseButton};

use praxis_utils::{info, Result};

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
