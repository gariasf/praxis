//! Hierarchy panel for displaying and managing the scene entity hierarchy.

use super::EditorPanel;
use egui::Ui;

/// Panel for displaying and manipulating the scene hierarchy.
pub struct HierarchyPanel {
    title: String,
}

impl HierarchyPanel {
    /// Creates a new hierarchy panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Hierarchy".to_string(),
        }
    }
}

impl Default for HierarchyPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for HierarchyPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Scene Hierarchy");
        ui.separator();

        ui.label("Entity tree will be displayed here.");

        if ui.button("Create Entity").clicked() {
            ui.label("Entity creation not yet implemented");
        }
    }
}
