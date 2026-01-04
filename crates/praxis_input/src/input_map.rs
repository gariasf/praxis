//! Input mapping system for rebindable controls.

use std::collections::{HashMap, HashSet};

use bevy_ecs::system::Resource;
use winit::keyboard::KeyCode;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{Action, ActionId, InputState, MouseButton};

/// A binding that maps a physical input to a logical action.
///
/// Input bindings allow multiple inputs to trigger the same action,
/// and support rebindable controls.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum InputBinding {
    /// A keyboard key binding.
    Key(KeyCode),
    /// A mouse button binding.
    MouseButton(MouseButton),
}

impl From<KeyCode> for InputBinding {
    fn from(key: KeyCode) -> Self {
        Self::Key(key)
    }
}

impl From<MouseButton> for InputBinding {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton(button)
    }
}

/// Resource that maps physical inputs to logical actions.
///
/// The `InputMap` allows you to define rebindable controls by mapping
/// physical inputs (keyboard keys, mouse buttons) to logical actions
/// (e.g., "jump", "fire", "`menu_select`").
///
/// # Example
///
/// ```
/// use praxis_input::{InputMap, Action, InputState};
/// use winit::keyboard::KeyCode;
///
/// let mut input_map = InputMap::default();
///
/// // Bind multiple keys to the same action
/// input_map.bind_key(&Action::new("jump"), KeyCode::Space);
/// input_map.bind_key(&Action::new("jump"), KeyCode::KeyW);
///
/// // Check if action is triggered
/// let mut input_state = InputState::default();
/// input_state.press_key(KeyCode::Space);
///
/// assert!(input_map.is_action_pressed(&Action::new("jump"), &input_state));
/// ```
#[derive(Debug, Clone, Resource)]
pub struct InputMap {
    /// Maps actions to their input bindings.
    action_bindings: HashMap<ActionId, HashSet<InputBinding>>,
    /// Reverse map: input bindings to actions.
    binding_actions: HashMap<InputBinding, HashSet<ActionId>>,
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}

impl InputMap {
    /// Creates a new, empty input map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            action_bindings: HashMap::new(),
            binding_actions: HashMap::new(),
        }
    }

    /// Binds a keyboard key to an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to bind to.
    /// * `key` - The keyboard key to bind.
    ///
    /// # Example
    ///
    /// ```
    /// use praxis_input::{InputMap, Action};
    /// use winit::keyboard::KeyCode;
    ///
    /// let mut input_map = InputMap::new();
    /// input_map.bind_key(&Action::new("jump"), KeyCode::Space);
    /// ```
    pub fn bind_key(&mut self, action: &Action, key: KeyCode) {
        self.bind(action, InputBinding::Key(key));
    }

    /// Binds a mouse button to an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to bind to.
    /// * `button` - The mouse button to bind.
    ///
    /// # Example
    ///
    /// ```
    /// use praxis_input::{InputMap, Action, MouseButton};
    ///
    /// let mut input_map = InputMap::new();
    /// input_map.bind_mouse_button(&Action::new("fire"), MouseButton::Left);
    /// ```
    pub fn bind_mouse_button(&mut self, action: &Action, button: MouseButton) {
        self.bind(action, InputBinding::MouseButton(button));
    }

    /// Binds an input binding to an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to bind to.
    /// * `binding` - The input binding to add.
    pub fn bind(&mut self, action: &Action, binding: InputBinding) {
        let action_id = action.id().clone();

        self.action_bindings
            .entry(action_id.clone())
            .or_default()
            .insert(binding.clone());

        self.binding_actions
            .entry(binding)
            .or_default()
            .insert(action_id);
    }

    /// Unbinds a keyboard key from an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to unbind from.
    /// * `key` - The keyboard key to unbind.
    pub fn unbind_key(&mut self, action: &Action, key: KeyCode) {
        self.unbind(action, &InputBinding::Key(key));
    }

    /// Unbinds a mouse button from an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to unbind from.
    /// * `button` - The mouse button to unbind.
    pub fn unbind_mouse_button(&mut self, action: &Action, button: MouseButton) {
        self.unbind(action, &InputBinding::MouseButton(button));
    }

    /// Unbinds an input binding from an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to unbind from.
    /// * `binding` - The input binding to remove.
    pub fn unbind(&mut self, action: &Action, binding: &InputBinding) {
        let action_id = action.id();

        if let Some(bindings) = self.action_bindings.get_mut(action_id) {
            bindings.remove(binding);
            if bindings.is_empty() {
                self.action_bindings.remove(action_id);
            }
        }

        if let Some(actions) = self.binding_actions.get_mut(binding) {
            actions.remove(action_id);
            if actions.is_empty() {
                self.binding_actions.remove(binding);
            }
        }
    }

    /// Unbinds all inputs from an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to clear bindings for.
    pub fn unbind_all(&mut self, action: &Action) {
        let action_id = action.id();

        if let Some(bindings) = self.action_bindings.remove(action_id) {
            for binding in bindings {
                if let Some(actions) = self.binding_actions.get_mut(&binding) {
                    actions.remove(action_id);
                    if actions.is_empty() {
                        self.binding_actions.remove(&binding);
                    }
                }
            }
        }
    }

    /// Checks if an action is currently pressed.
    ///
    /// Returns `true` if any of the action's bound inputs are currently pressed.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to check.
    /// * `input_state` - The current input state.
    #[must_use]
    pub fn is_action_pressed(&self, action: &Action, input_state: &InputState) -> bool {
        self.action_bindings
            .get(action.id())
            .is_some_and(|bindings| {
                bindings
                    .iter()
                    .any(|binding| Self::is_binding_pressed(binding, input_state))
            })
    }

    /// Checks if an action was just pressed this frame.
    ///
    /// Returns `true` if any of the action's bound inputs were just pressed.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to check.
    /// * `input_state` - The current input state.
    #[must_use]
    pub fn is_action_just_pressed(&self, action: &Action, input_state: &InputState) -> bool {
        self.action_bindings
            .get(action.id())
            .is_some_and(|bindings| {
                bindings
                    .iter()
                    .any(|binding| Self::is_binding_just_pressed(binding, input_state))
            })
    }

    /// Checks if an action was just released this frame.
    ///
    /// Returns `true` if any of the action's bound inputs were just released.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to check.
    /// * `input_state` - The current input state.
    #[must_use]
    pub fn is_action_just_released(&self, action: &Action, input_state: &InputState) -> bool {
        self.action_bindings
            .get(action.id())
            .is_some_and(|bindings| {
                bindings
                    .iter()
                    .any(|binding| Self::is_binding_just_released(binding, input_state))
            })
    }

    /// Gets all bindings for an action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to get bindings for.
    #[must_use]
    pub fn get_bindings(&self, action: &Action) -> Option<&HashSet<InputBinding>> {
        self.action_bindings.get(action.id())
    }

    /// Gets all actions bound to a specific input binding.
    ///
    /// # Arguments
    ///
    /// * `binding` - The input binding to look up.
    #[must_use]
    pub fn get_actions_for_binding(&self, binding: &InputBinding) -> Option<&HashSet<ActionId>> {
        self.binding_actions.get(binding)
    }

    /// Returns an iterator over all action bindings.
    pub fn iter(&self) -> impl Iterator<Item = (&ActionId, &HashSet<InputBinding>)> {
        self.action_bindings.iter()
    }

    /// Clears all bindings from the input map.
    pub fn clear(&mut self) {
        self.action_bindings.clear();
        self.binding_actions.clear();
    }

    /// Checks if a specific binding is currently pressed.
    fn is_binding_pressed(binding: &InputBinding, input_state: &InputState) -> bool {
        match binding {
            InputBinding::Key(key) => input_state.is_key_pressed(*key),
            InputBinding::MouseButton(button) => input_state.is_mouse_button_pressed(*button),
        }
    }

    /// Checks if a specific binding was just pressed this frame.
    fn is_binding_just_pressed(binding: &InputBinding, input_state: &InputState) -> bool {
        match binding {
            InputBinding::Key(key) => input_state.is_key_just_pressed(*key),
            InputBinding::MouseButton(button) => input_state.is_mouse_button_just_pressed(*button),
        }
    }

    /// Checks if a specific binding was just released this frame.
    fn is_binding_just_released(binding: &InputBinding, input_state: &InputState) -> bool {
        match binding {
            InputBinding::Key(key) => input_state.is_key_just_released(*key),
            InputBinding::MouseButton(button) => input_state.is_mouse_button_just_released(*button),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_and_check_action() {
        let mut input_map = InputMap::new();
        let mut input_state = InputState::new();

        let jump = Action::new("jump");
        input_map.bind_key(&jump, KeyCode::Space);

        input_state.press_key(KeyCode::Space);
        assert!(input_map.is_action_pressed(&jump, &input_state));
        assert!(input_map.is_action_just_pressed(&jump, &input_state));
    }

    #[test]
    fn test_multiple_bindings() {
        let mut input_map = InputMap::new();
        let mut input_state = InputState::new();

        let jump = Action::new("jump");
        input_map.bind_key(&jump, KeyCode::Space);
        input_map.bind_key(&jump, KeyCode::KeyW);

        input_state.press_key(KeyCode::KeyW);
        assert!(input_map.is_action_pressed(&jump, &input_state));
    }

    #[test]
    fn test_unbind() {
        let mut input_map = InputMap::new();
        let jump = Action::new("jump");

        input_map.bind_key(&jump, KeyCode::Space);
        assert!(input_map.get_bindings(&jump).is_some());

        input_map.unbind_key(&jump, KeyCode::Space);
        assert!(input_map.get_bindings(&jump).is_none());
    }

    #[test]
    fn test_mouse_button_binding() {
        let mut input_map = InputMap::new();
        let mut input_state = InputState::new();

        let fire = Action::new("fire");
        input_map.bind_mouse_button(&fire, MouseButton::Left);

        input_state.press_mouse_button(MouseButton::Left);
        assert!(input_map.is_action_pressed(&fire, &input_state));
    }
}
