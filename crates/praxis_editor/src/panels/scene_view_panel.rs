//! Scene view panel for rendering and interacting with the 3D scene.

use super::EditorPanel;
use crate::panels::AssetEntry;
use egui::{Color32, Ui};
use praxis_utils::info;

/// Panel for displaying the 3D scene viewport.
pub struct SceneViewPanel {
    title: String,
    /// Last dropped asset (if any)
    last_dropped_asset: Option<AssetEntry>,
}

impl SceneViewPanel {
    /// Creates a new scene view panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Scene".to_string(),
            last_dropped_asset: None,
        }
    }

    /// Gets the last dropped asset and clears it
    pub fn take_dropped_asset(&mut self) -> Option<AssetEntry> {
        self.last_dropped_asset.take()
    }

    /// Checks if the scene view can accept a drop
    pub fn can_accept_drop(&self, asset: &AssetEntry) -> bool {
        !asset.is_directory
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

        let response = ui.allocate_response(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );

        if response.hovered() {
            ui.painter().rect_stroke(
                response.rect,
                0.0,
                egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 200)),
            );

            if ui.input(|i| i.pointer.any_released()) {
                info!("Potential drop target activated in scene view");
            }
        }

        ui.painter().text(
            response.rect.center(),
            egui::Align2::CENTER_CENTER,
            "3D scene viewport will be rendered here.\n\nDrag assets from the Asset Browser to add them to the scene.",
            egui::FontId::proportional(14.0),
            Color32::GRAY,
        );
        
        ui.label("Camera controls:");
        ui.label("• Right-click + drag to rotate");
        ui.label("• WASD to move");
        ui.label("• Mouse wheel to zoom");
    }
}
