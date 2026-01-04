//! Assets panel for browsing and managing project assets.

use super::EditorPanel;
use egui::Ui;

/// Panel for browsing and managing project assets.
pub struct AssetsPanel {
    title: String,
    current_path: String,
}

impl AssetsPanel {
    /// Creates a new assets panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Assets".to_string(),
            current_path: "assets/".to_string(),
        }
    }
}

impl Default for AssetsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for AssetsPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Assets");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut self.current_path);
        });
        
        ui.separator();
        
        ui.label("Asset browser will display project files here.");
        
        ui.label("Supported asset types:");
        ui.label("• Models (.obj, .gltf)");
        ui.label("• Textures (.png, .jpg)");
        ui.label("• Audio (.wav, .ogg, .mp3)");
        ui.label("• Scenes (.scene)");
    }
}
