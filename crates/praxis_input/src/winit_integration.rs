//! Integration helpers for processing winit events.

use winit::event::{MouseScrollDelta, WindowEvent};

use crate::InputState;

/// Processes a winit window event and updates the input state accordingly.
///
/// This is a convenience function that handles all input-related window events
/// and updates the `InputState` resource.
///
/// # Arguments
///
/// * `input_state` - Mutable reference to the input state to update.
/// * `event` - The window event to process.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_input::{InputState, winit_integration};
/// use winit::event::WindowEvent;
///
/// let mut input_state = InputState::default();
///
/// // In your event loop
/// fn handle_window_event(input_state: &mut InputState, event: &WindowEvent) {
///     winit_integration::process_window_event(input_state, event);
/// }
/// ```
pub fn process_window_event(input_state: &mut InputState, event: &WindowEvent) {
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            input_state.handle_keyboard_input(event.physical_key, event.state);
        }
        WindowEvent::MouseInput { state, button, .. } => {
            input_state.handle_mouse_button((*button).into(), *state);
        }
        WindowEvent::CursorMoved { position, .. } => {
            input_state.handle_cursor_moved((position.x, position.y));
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let scroll = match delta {
                MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                MouseScrollDelta::PixelDelta(pos) => {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        (pos.x as f32 / 120.0, pos.y as f32 / 120.0)
                    }
                }
            };
            input_state.handle_mouse_wheel(scroll);
        }
        _ => {}
    }
}
