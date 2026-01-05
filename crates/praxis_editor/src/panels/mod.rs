//! Editor panel definitions and implementations.

mod assets_panel;
mod console_panel;
mod hierarchy_panel;
mod inspector_panel;
mod scene_view_panel;
#[cfg(feature = "terrain")]
mod terrain_panel;
pub mod viewport_panel;

pub use assets_panel::{AssetEntry, AssetImportConfig, AssetType, AssetsPanel, AssetsPanelExt};
pub use console_panel::{ConsoleLayer, ConsolePanel, LogBuffer, LogLevel, LogMessage};
pub use hierarchy_panel::HierarchyPanel;
pub use inspector_panel::InspectorPanel;
pub use scene_view_panel::{SceneViewPanel, SceneViewPanelExt};
#[cfg(feature = "terrain")]
pub use terrain_panel::{TerrainPanel, TerrainPanelExt};
pub use viewport_panel::ViewportPanel;

use egui::Ui;

/// Trait for editor panels that can be displayed in the dock system.
pub trait EditorPanel {
    /// Returns the title of the panel.
    fn title(&self) -> &str;

    /// Updates and renders the panel UI.
    fn ui(
        &mut self,
        ui: &mut Ui,
        world: Option<&praxis_ecs::World>,
        render_context: Option<&mut praxis_graphics::RenderContext>,
    );

    /// Returns whether the panel is currently open.
    fn is_open(&self) -> bool {
        true
    }

    /// Sets whether the panel is open.
    fn set_open(&mut self, _open: bool) {}

    /// Called when the panel is about to be closed.
    fn on_close(&mut self) {}
}
