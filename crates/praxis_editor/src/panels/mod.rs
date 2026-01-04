//! Editor panel definitions and implementations.

mod assets_panel;
mod console_panel;
mod hierarchy_panel;
mod inspector_panel;
mod scene_view_panel;
pub mod viewport_panel;

pub use assets_panel::{AssetEntry, AssetImportConfig, AssetType, AssetsPanel};
pub use console_panel::ConsolePanel;
pub use hierarchy_panel::HierarchyPanel;
pub use inspector_panel::InspectorPanel;
pub use scene_view_panel::SceneViewPanel;
pub use viewport_panel::ViewportPanel;

use egui::Ui;

/// Trait for editor panels that can be displayed in the dock system.
pub trait EditorPanel {
    /// Returns the title of the panel.
    fn title(&self) -> &str;

    /// Updates and renders the panel UI.
    fn ui(&mut self, ui: &mut Ui);

    /// Called when the panel is about to be closed.
    fn on_close(&mut self) {}
}
