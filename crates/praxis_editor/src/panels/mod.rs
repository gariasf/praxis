//! Editor panel definitions and implementations.

mod assets_panel;
mod console_panel;
mod hierarchy_panel;
mod inspector_panel;
mod scene_view_panel;
pub mod viewport_panel;

pub use assets_panel::{AssetEntry, AssetImportConfig, AssetType, AssetsPanel, AssetsPanelExt};
pub use console_panel::{ConsoleLayer, ConsolePanel, LogBuffer, LogLevel, LogMessage};
pub use hierarchy_panel::HierarchyPanel;
pub use inspector_panel::InspectorPanel;
pub use scene_view_panel::{SceneViewPanel, SceneViewPanelExt};
pub use viewport_panel::ViewportPanel;

use egui::Ui;

/// Trait for editor panels that can be displayed in the dock system.
pub trait EditorPanel {
    /// Returns the title of the panel.
    fn title(&self) -> &str;

    /// Updates and renders the panel UI.
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI context
    /// * `world` - Optional shared reference to the ECS world
    /// * `render_context` - Optional mutable reference to the rendering context
    fn ui(
        &mut self,
        ui: &mut Ui,
        world: Option<&praxis_ecs::World>,
        render_context: Option<&mut praxis_graphics::RenderContext>,
    );

    /// Called when the panel is about to be closed.
    fn on_close(&mut self) {}
}
