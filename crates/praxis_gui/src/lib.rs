//! GUI system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing GUI elements using egui.

mod debug_ui;
mod egui_integration;
mod entity_inspector;
mod gizmos;
mod gui_state;

pub use debug_ui::DebugUi;
pub use egui_integration::EguiIntegration;
pub use entity_inspector::EntityInspector;
pub use gizmos::{Gizmo, GizmoMode, TransformGizmos};
pub use gui_state::GuiState;

use praxis_utils::info;

/// Initializes the GUI system.
pub fn init() {
    info!("GUI system initialized");
}
