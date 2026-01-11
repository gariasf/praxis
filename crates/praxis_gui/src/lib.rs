//! GUI system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing GUI elements using egui.
//!
//! # Architecture Overview
//!
//! ## Immediate-Mode UI (egui)
//!
//! Unlike traditional retained-mode GUI frameworks (Qt, GTK, WPF), egui uses an
//! **immediate-mode** approach where UI code runs every frame:
//!
//! ```text
//! Traditional (Retained):           Immediate-Mode (egui):
//! ┌────────────────────┐           ┌────────────────────┐
//! │ Create widgets     │           │ Each frame:         │
//! │ Set properties     │           │   if button() {     │
//! │ Connect callbacks  │           │     // handle click │
//! │                    │           │   }                 │
//! │ Event loop runs    │           │                     │
//! │ Forever            │           │ No state to manage  │
//! └────────────────────┘           └────────────────────┘
//! ```
//!
//! Benefits:
//! - **Simple mental model**: UI state = application state, no synchronization bugs
//! - **Easy to compose**: UI functions call other UI functions naturally
//! - **Dynamic UI**: Conditionally render elements without widget tree manipulation
//! - **No callbacks**: Logic is inline with UI code
//!
//! ## egui + Vulkan Integration
//!
//! egui is rendering-backend agnostic. This crate integrates it with Vulkan via:
//!
//! 1. **`EguiIntegration`**: Manages the egui→Vulkan bridge
//!    - Handles input events from winit
//!    - Converts egui's `ClippedPrimitive` output into Vulkan draw calls
//!    - Manages font/texture uploads to GPU memory
//!
//! 2. **`EguiContext`**: ECS resource wrapping `egui::Context`
//!    - Allows systems to render UI by accessing `Res<EguiContext>`
//!    - The context tracks layout, input state, and widget IDs between frames
//!
//! 3. **Render flow each frame**:
//!    ```text
//!    ┌──────────────────────────────────────────────────────┐
//!    │ 1. begin_frame()                                      │
//!    │    - Collect input events (mouse, keyboard)          │
//!    │    - Start new egui frame                            │
//!    ├──────────────────────────────────────────────────────┤
//!    │ 2. UI systems run (your game code)                   │
//!    │    - Call egui::Window::show()                       │
//!    │    - Add buttons, sliders, text, etc.                │
//!    │    - egui builds internal mesh/command list          │
//!    ├──────────────────────────────────────────────────────┤
//!    │ 3. end_frame()                                        │
//!    │    - egui finalizes ClippedPrimitives                │
//!    │    - Convert to Vulkan vertex/index buffers          │
//!    ├──────────────────────────────────────────────────────┤
//!    │ 4. Render pass                                        │
//!    │    - Upload buffers to GPU                           │
//!    │    - Draw GUI on top of 3D scene                     │
//!    │    - Use alpha blending for transparency             │
//!    └──────────────────────────────────────────────────────┘
//!    ```
//!
//! 4. **Memory management**:
//!    - egui allocates shapes/vertices on CPU each frame (cheap, ~1ms)
//!    - `egui_vulkano` caches GPU textures for fonts/images (LRU eviction)
//!    - Vertex/index buffers are dynamically sized and reused
//!
//! ## Module Organization
//!
//! - **`console_panel`**: Debug console with command registry & Lua REPL
//! - **`egui_integration`**: Low-level Vulkan rendering integration
//! - **`entity_inspector`**: ECS entity/component editor
//! - **`hierarchy_panel`**: Scene hierarchy tree view
//! - **`inspector_panel`**: Properties panel for selected entities
//! - **`gizmos`**: 3D transform manipulation widgets
//! - **`debug_ui`**: Performance metrics and debug overlays
//! - **`gui_state`**: Shared GUI state (selections, tool modes, etc.)

mod console_panel;
mod debug_ui;
mod egui_integration;
mod entity_inspector;
mod gizmos;
mod gui_state;
mod hierarchy_panel;
mod inspector_panel;

pub use console_panel::{CommandRegistry, ConsolePanel, LogEntry, LogLevel};
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
#[derive(Default)]
pub struct EguiContext {
    context: egui::Context,
}

impl praxis_ecs::Resource for EguiContext {}

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
