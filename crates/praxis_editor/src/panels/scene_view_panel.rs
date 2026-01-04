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
    /// Viewport border color (set externally based on play mode)
    border_color: Color32,
}

impl SceneViewPanel {
    /// Creates a new scene view panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Scene".to_string(),
            last_dropped_asset: None,
            border_color: Color32::from_rgb(76, 76, 89), // Default dark gray
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

    /// Sets the viewport border color (used for play mode indicators)
    pub fn set_border_color(&mut self, color: Color32) {
        self.border_color = color;
    }

    /// Gets the current viewport border color
    pub const fn border_color(&self) -> Color32 {
        self.border_color
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

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&praxis_graphics::RenderContext>,
    ) {
        ui.heading("Scene View");
        ui.separator();

        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // Draw border with color based on play mode
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(3.0, self.border_color),
        );

        if response.hovered() && ui.input(|i| i.pointer.any_released()) {
            info!("Potential drop target activated in scene view");
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
