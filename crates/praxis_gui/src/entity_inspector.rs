//! Entity inspector for viewing and editing ECS component data.
//!
//! This module provides backward compatibility with the old EntityInspector API.
//! The new implementation is in `inspector_panel.rs` which provides expanded functionality.

use crate::InspectorPanel;
use praxis_ecs::{Entity, World};

/// Entity inspector state and configuration.
///
/// This is a compatibility wrapper around `InspectorPanel` with the same API.
/// For new code, consider using `InspectorPanel` directly for access to all features.
#[derive(Default)]
pub struct EntityInspector {
    panel: InspectorPanel,
}

impl EntityInspector {
    /// Creates a new entity inspector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the entity inspector UI.
    /// 
    /// The selected_entity parameter should come from the HierarchyPanel's selection_state.
    pub fn render(&mut self, ctx: &egui::Context, world: &mut World, selected_entity: Option<Entity>) {
        self.panel.render(ctx, world, selected_entity);
    }

    /// Toggles the visibility of the entity inspector.
    pub fn toggle(&mut self) {
        self.panel.toggle();
    }

    /// Sets the visibility of the entity inspector.
    pub fn set_visible(&mut self, visible: bool) {
        self.panel.set_visible(visible);
    }

    /// Gets a reference to the underlying InspectorPanel.
    pub fn panel(&self) -> &InspectorPanel {
        &self.panel
    }

    /// Gets a mutable reference to the underlying InspectorPanel.
    pub fn panel_mut(&mut self) -> &mut InspectorPanel {
        &mut self.panel
    }
}
