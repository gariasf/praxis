//! Editor mode definitions for switching between edit and play modes.

/// Represents the current mode of the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Edit mode - editor is active and game simulation is paused.
    Edit,
    /// Play mode - game simulation is running.
    Play,
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Edit
    }
}

impl EditorMode {
    /// Returns `true` if the editor is in edit mode.
    #[must_use]
    pub const fn is_edit(&self) -> bool {
        matches!(self, Self::Edit)
    }

    /// Returns `true` if the editor is in play mode.
    #[must_use]
    pub const fn is_play(&self) -> bool {
        matches!(self, Self::Play)
    }

    /// Toggles between edit and play modes.
    #[must_use]
    pub const fn toggle(self) -> Self {
        match self {
            Self::Edit => Self::Play,
            Self::Play => Self::Edit,
        }
    }
}
