//! Action system for mapping inputs to logical game actions.

use std::fmt;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Unique identifier for an action.
///
/// Actions represent logical game operations (e.g., "jump", "fire", "`menu_up`")
/// that can be bound to multiple physical inputs.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct ActionId(String);

impl ActionId {
    /// Creates a new action ID from a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use praxis_input::ActionId;
    ///
    /// let jump = ActionId::new("jump");
    /// let fire = ActionId::new("fire");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Returns the action name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ActionId(\"{}\")", self.0)
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ActionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ActionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A logical game action that can be bound to multiple inputs.
///
/// Actions provide a layer of abstraction between physical inputs and game logic,
/// enabling rebindable controls and multi-input support.
#[derive(Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Action {
    id: ActionId,
}

impl Action {
    /// Creates a new action with the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// use praxis_input::Action;
    ///
    /// let jump = Action::new("jump");
    /// let fire = Action::new("fire");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ActionId::new(name),
        }
    }

    /// Returns a reference to the action's ID.
    #[must_use]
    pub const fn id(&self) -> &ActionId {
        &self.id
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action(\"{}\")", self.id.0)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id.0)
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Action {}

impl Hash for Action {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
