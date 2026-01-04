//! GUI system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing GUI elements using egui.

mod debug_ui;
mod egui_integration;
mod entity_inspector;
mod gizmos;
mod gui_state;
mod hierarchy_panel;
mod inspector_panel;

pub use debug_ui::DebugUi;
pub use egui_integration::EguiIntegration;
pub use entity_inspector::EntityInspector;
pub use gizmos::{Gizmo, GizmoMode, TransformGizmos};
pub use gui_state::GuiState;
pub use hierarchy_panel::HierarchyPanel;
pub use inspector_panel::InspectorPanel;

/// Resource that wraps the egui context for ECS access.
///
/// This resource allows systems to access the egui context for rendering GUI elements.
/// It's typically used in conjunction with `EditorState` or other GUI systems.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_gui::EguiContext;
/// use praxis_ecs::{Res, World};
///
/// fn my_gui_system(egui_context: Res<EguiContext>) {
///     let ctx = egui_context.context();
///     // Use ctx to render GUI elements
/// }
/// ```
pub struct EguiContext {
    context: egui::Context,
}

impl praxis_ecs::Resource for EguiContext {}

impl Default for EguiContext {
    fn default() -> Self {
        Self {
            context: egui::Context::default(),
        }
    }
}

impl EguiContext {
    /// Creates a new `EguiContext` with a default egui context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `EguiContext` with the specified egui context.
    #[must_use]
    pub fn with_context(context: egui::Context) -> Self {
        Self { context }
    }

    /// Gets a reference to the egui context.
    #[must_use]
    pub fn context(&self) -> &egui::Context {
        &self.context
    }

    /// Sets the egui context.
    pub fn set_context(&mut self, context: egui::Context) {
        self.context = context;
    }
}

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
