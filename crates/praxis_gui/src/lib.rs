//! GUI system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing GUI elements using egui.

mod debug_ui;
mod egui_integration;
mod entity_inspector;
mod gizmos;
mod gui_state;
mod hierarchy_panel;

pub use debug_ui::DebugUi;
pub use egui_integration::EguiIntegration;
pub use entity_inspector::EntityInspector;
pub use gizmos::{Gizmo, GizmoMode, TransformGizmos};
pub use gui_state::GuiState;
pub use hierarchy_panel::HierarchyPanel;

use praxis_utils::{info, Result};

/// Initializes the GUI system.
///
/// This function sets up any necessary global state for the GUI system.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for GUI
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., font loading,
///   style configuration, custom widget registration)
///
/// # Example
///
/// ```rust,no_run
/// praxis_gui::init().expect("Failed to initialize GUI system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    info!("Initializing GUI system");
    Ok(())
}
