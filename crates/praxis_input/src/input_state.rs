//! Input state tracking for keyboard, mouse, and gamepad.

use std::collections::HashSet;

use bevy_ecs::system::Resource;
use winit::event::{ElementState, MouseButton as WinitMouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Mouse button identifier.
///
/// Represents the physical buttons on a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button (scroll wheel click).
    Middle,
    /// Additional mouse button (e.g., side buttons).
    Other(u16),
}

impl From<WinitMouseButton> for MouseButton {
    fn from(button: WinitMouseButton) -> Self {
        match button {
            WinitMouseButton::Left => Self::Left,
            WinitMouseButton::Right => Self::Right,
            WinitMouseButton::Middle => Self::Middle,
            WinitMouseButton::Back => Self::Other(4),
            WinitMouseButton::Forward => Self::Other(5),
            WinitMouseButton::Other(id) => Self::Other(id),
        }
    }
}

/// Resource tracking the current state of all input devices.
///
/// This resource maintains the state of keyboard keys, mouse buttons, and mouse position.
/// It should be updated each frame by processing input events from the windowing system.
///
/// # Example
///
/// ```
/// use praxis_input::InputState;
/// use winit::keyboard::KeyCode;
///
/// let mut input = InputState::default();
///
/// // Simulate key press
/// input.press_key(KeyCode::KeyW);
///
/// // Check key state
/// assert!(input.is_key_pressed(KeyCode::KeyW));
/// assert!(!input.is_key_pressed(KeyCode::KeyS));
/// ```
#[derive(Debug, Clone, Resource)]
pub struct InputState {
    /// Keys currently pressed.
    pressed_keys: HashSet<KeyCode>,
    /// Keys pressed this frame.
    just_pressed_keys: HashSet<KeyCode>,
    /// Keys released this frame.
    just_released_keys: HashSet<KeyCode>,

    /// Mouse buttons currently pressed.
    pressed_mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons pressed this frame.
    just_pressed_mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons released this frame.
    just_released_mouse_buttons: HashSet<MouseButton>,

    /// Current mouse position in pixels (relative to window).
    mouse_position: (f64, f64),
    /// Mouse position delta since last frame.
    mouse_delta: (f64, f64),
    /// Mouse scroll delta (horizontal, vertical).
    scroll_delta: (f32, f32),
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    /// Creates a new, empty input state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            just_pressed_mouse_buttons: HashSet::new(),
            just_released_mouse_buttons: HashSet::new(),
            mouse_position: (0.0, 0.0),
            mouse_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
        }
    }

    /// Updates the input state for a new frame.
    ///
    /// This should be called at the beginning of each frame to clear
    /// the "just pressed" and "just released" states from the previous frame.
    pub fn update(&mut self) {
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    /// Handles a keyboard input event.
    ///
    /// # Arguments
    ///
    /// * `key` - The physical key that was pressed or released.
    /// * `state` - Whether the key was pressed or released.
    pub fn handle_keyboard_input(&mut self, key: PhysicalKey, state: ElementState) {
        if let PhysicalKey::Code(keycode) = key {
            match state {
                ElementState::Pressed => {
                    if self.pressed_keys.insert(keycode) {
                        self.just_pressed_keys.insert(keycode);
                    }
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&keycode);
                    self.just_released_keys.insert(keycode);
                }
            }
        }
    }

    /// Handles a mouse button input event.
    ///
    /// # Arguments
    ///
    /// * `button` - The mouse button that was pressed or released.
    /// * `state` - Whether the button was pressed or released.
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.pressed_mouse_buttons.insert(button) {
                    self.just_pressed_mouse_buttons.insert(button);
                }
            }
            ElementState::Released => {
                self.pressed_mouse_buttons.remove(&button);
                self.just_released_mouse_buttons.insert(button);
            }
        }
    }

    /// Updates the mouse cursor position.
    ///
    /// # Arguments
    ///
    /// * `position` - The new mouse position in pixels (x, y).
    pub fn handle_cursor_moved(&mut self, position: (f64, f64)) {
        let old_position = self.mouse_position;
        self.mouse_position = position;
        self.mouse_delta = (position.0 - old_position.0, position.1 - old_position.1);
    }

    /// Updates the mouse scroll wheel delta.
    ///
    /// # Arguments
    ///
    /// * `delta` - The scroll delta (horizontal, vertical).
    pub const fn handle_mouse_wheel(&mut self, delta: (f32, f32)) {
        self.scroll_delta = delta;
    }

    /// Presses a key programmatically (for testing or simulation).
    pub fn press_key(&mut self, keycode: KeyCode) {
        if self.pressed_keys.insert(keycode) {
            self.just_pressed_keys.insert(keycode);
        }
    }

    /// Releases a key programmatically (for testing or simulation).
    pub fn release_key(&mut self, keycode: KeyCode) {
        self.pressed_keys.remove(&keycode);
        self.just_released_keys.insert(keycode);
    }

    /// Presses a mouse button programmatically (for testing or simulation).
    pub fn press_mouse_button(&mut self, button: MouseButton) {
        if self.pressed_mouse_buttons.insert(button) {
            self.just_pressed_mouse_buttons.insert(button);
        }
    }

    /// Releases a mouse button programmatically (for testing or simulation).
    pub fn release_mouse_button(&mut self, button: MouseButton) {
        self.pressed_mouse_buttons.remove(&button);
        self.just_released_mouse_buttons.insert(button);
    }

    /// Checks if a key is currently pressed.
    #[must_use]
    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.pressed_keys.contains(&keycode)
    }

    /// Checks if a key was just pressed this frame.
    #[must_use]
    pub fn is_key_just_pressed(&self, keycode: KeyCode) -> bool {
        self.just_pressed_keys.contains(&keycode)
    }

    /// Checks if a key was just released this frame.
    #[must_use]
    pub fn is_key_just_released(&self, keycode: KeyCode) -> bool {
        self.just_released_keys.contains(&keycode)
    }

    /// Checks if a mouse button is currently pressed.
    #[must_use]
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    /// Checks if a mouse button was just pressed this frame.
    #[must_use]
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_mouse_buttons.contains(&button)
    }

    /// Checks if a mouse button was just released this frame.
    #[must_use]
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released_mouse_buttons.contains(&button)
    }

    /// Returns the current mouse position in pixels.
    #[must_use]
    pub const fn mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    /// Returns the mouse movement delta since the last frame.
    #[must_use]
    pub const fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    /// Returns the mouse scroll wheel delta for this frame.
    #[must_use]
    pub const fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }

    /// Returns an iterator over all currently pressed keys.
    pub fn pressed_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.pressed_keys.iter()
    }

    /// Returns an iterator over all keys pressed this frame.
    pub fn just_pressed_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.just_pressed_keys.iter()
    }

    /// Returns an iterator over all keys released this frame.
    pub fn just_released_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.just_released_keys.iter()
    }

    /// Returns an iterator over all currently pressed mouse buttons.
    pub fn pressed_mouse_buttons(&self) -> impl Iterator<Item = &MouseButton> {
        self.pressed_mouse_buttons.iter()
    }

    /// Returns an iterator over all mouse buttons pressed this frame.
    pub fn just_pressed_mouse_buttons(&self) -> impl Iterator<Item = &MouseButton> {
        self.just_pressed_mouse_buttons.iter()
    }

    /// Returns an iterator over all mouse buttons released this frame.
    pub fn just_released_mouse_buttons(&self) -> impl Iterator<Item = &MouseButton> {
        self.just_released_mouse_buttons.iter()
    }

    /// Clears all input state.
    ///
    /// This is useful when you need to reset input tracking,
    /// such as when switching between game states.
    pub fn clear(&mut self) {
        self.pressed_keys.clear();
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
        self.mouse_position = (0.0, 0.0);
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_press_and_release() {
        let mut input = InputState::new();

        input.press_key(KeyCode::KeyW);
        assert!(input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_just_pressed(KeyCode::KeyW));
        assert!(!input.is_key_just_released(KeyCode::KeyW));

        input.update();
        assert!(input.is_key_pressed(KeyCode::KeyW));
        assert!(!input.is_key_just_pressed(KeyCode::KeyW));

        input.release_key(KeyCode::KeyW);
        assert!(!input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_just_released(KeyCode::KeyW));

        input.update();
        assert!(!input.is_key_just_released(KeyCode::KeyW));
    }

    #[test]
    fn test_mouse_button() {
        let mut input = InputState::new();

        input.press_mouse_button(MouseButton::Left);
        assert!(input.is_mouse_button_pressed(MouseButton::Left));
        assert!(input.is_mouse_button_just_pressed(MouseButton::Left));

        input.update();
        assert!(input.is_mouse_button_pressed(MouseButton::Left));
        assert!(!input.is_mouse_button_just_pressed(MouseButton::Left));
    }

    #[test]
    fn test_mouse_position() {
        let mut input = InputState::new();

        input.handle_cursor_moved((100.0, 200.0));
        assert_eq!(input.mouse_position(), (100.0, 200.0));
        assert_eq!(input.mouse_delta(), (100.0, 200.0));

        input.update();
        assert_eq!(input.mouse_delta(), (0.0, 0.0));

        input.handle_cursor_moved((150.0, 250.0));
        assert_eq!(input.mouse_position(), (150.0, 250.0));
        assert_eq!(input.mouse_delta(), (50.0, 50.0));
    }

    #[test]
    fn test_mouse_scroll() {
        let mut input = InputState::new();

        input.handle_mouse_wheel((1.0, -2.0));
        assert_eq!(input.scroll_delta(), (1.0, -2.0));

        input.update();
        assert_eq!(input.scroll_delta(), (0.0, 0.0));
    }

    #[test]
    fn test_clear() {
        let mut input = InputState::new();

        input.press_key(KeyCode::Space);
        input.press_mouse_button(MouseButton::Left);
        input.handle_cursor_moved((100.0, 100.0));

        input.clear();

        assert!(!input.is_key_pressed(KeyCode::Space));
        assert!(!input.is_mouse_button_pressed(MouseButton::Left));
        assert_eq!(input.mouse_position(), (0.0, 0.0));
    }
}
