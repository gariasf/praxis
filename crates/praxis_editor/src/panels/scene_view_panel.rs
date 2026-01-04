//! Scene view panel for rendering and interacting with the 3D scene.

use super::EditorPanel;
use egui::Ui;

/// Panel for displaying the 3D scene viewport.
pub struct SceneViewPanel {
    title: String,
}

impl SceneViewPanel {
    /// Creates a new scene view panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Scene".to_string(),
        }
    }
}

impl Default for SceneViewPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for SceneViewPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Scene View");
        ui.separator();
        
        ui.label("3D scene viewport will be rendered here.");
        
        ui.label("Camera controls:");
        ui.label("• Right-click + drag to rotate");
        ui.label("• WASD to move");
        ui.label("• Mouse wheel to zoom");
    }
}
