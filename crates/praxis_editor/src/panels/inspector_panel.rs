//! Inspector panel for viewing and editing entity components.

use super::EditorPanel;
use egui::Ui;

/// Panel for inspecting and editing selected entity properties.
pub struct InspectorPanel {
    title: String,
}

impl InspectorPanel {
    /// Creates a new inspector panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Inspector".to_string(),
        }
    }
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for InspectorPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Inspector");
        ui.separator();
        
        ui.label("Select an entity to inspect its components.");
    }
}
