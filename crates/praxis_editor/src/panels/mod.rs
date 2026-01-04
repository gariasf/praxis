//! Editor panel definitions and implementations.

mod hierarchy_panel;
mod inspector_panel;
mod scene_view_panel;
mod console_panel;
mod assets_panel;
pub mod viewport_panel;

pub use hierarchy_panel::HierarchyPanel;
pub use inspector_panel::InspectorPanel;
pub use scene_view_panel::SceneViewPanel;
pub use console_panel::ConsolePanel;
pub use assets_panel::AssetsPanel;
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
