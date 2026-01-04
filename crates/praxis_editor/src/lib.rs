//! Editor system for the Praxis engine.
//!
//! This crate provides a comprehensive editor interface for creating and managing game content.
//! It includes a dockable panel system, scene editing, asset management, and debugging tools.
//!
//! # Architecture
//!
//! The editor is built around several key components:
//!
//! - **`EditorState`**: The root coordinator that manages all editor panels and modes
//! - **`EditorMode`**: Defines whether the editor is in Edit or Play mode
//! - **Panels**: Modular UI components for different editor functions
//!   - `SceneViewPanel`: 3D viewport for visualizing and interacting with the scene
//!   - `HierarchyPanel`: Tree view of scene entities
//!   - `InspectorPanel`: Component editing for selected entities
//!   - `ConsolePanel`: Log output and command execution
//!   - `AssetsPanel`: Project asset browser
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_editor::{EditorState, EditorMode};
//!
//! // Initialize the editor
//! praxis_editor::init().expect("Failed to initialize editor");
//!
//! // Create editor state
//! let mut editor = EditorState::new();
//!
//! // Toggle between edit and play modes
//! editor.set_mode(EditorMode::Play);
//!
//! // Render editor UI (called every frame)
//! // editor.ui(&egui_context);
//! ```
//!
//! # Editor Modes
//!
//! The editor supports two primary modes:
//!
//! - **Edit Mode**: The default mode where you can modify the scene, entities, and components.
//!   Game simulation is paused in this mode.
//! - **Play Mode**: Game simulation runs while the editor remains visible, allowing for
//!   real-time debugging and testing.
//!
//! # Panel System
//!
//! The editor uses `egui_dock` for a flexible, dockable panel system. Panels can be:
//! - Dragged and rearranged
//! - Split horizontally or vertically
//! - Tabbed together
//! - Closed and reopened
//!
//! Each panel implements the `EditorPanel` trait, providing a consistent interface for
//! rendering and lifecycle management.
//!
//! ## Asset Browser Panel
//!
//! The `AssetsPanel` provides comprehensive asset management:
//! - **Filesystem Browser**: Navigate the assets/ directory with breadcrumb navigation
//! - **Thumbnail Preview**: Automatic thumbnail generation for texture assets
//! - **Search & Filter**: Real-time search across asset names
//! - **Drag-and-Drop**: Drag assets to scene view for instant placement
//! - **Import Dialogs**: Configure import settings per asset type
//! - **Hot-Reload**: Automatic detection of asset changes via file watcher
//!
//! Supported asset types:
//! - Textures: PNG, JPG, JPEG
//! - Models: OBJ, GLTF, GLB
//! - Audio: WAV, OGG, MP3
//! - Scenes: SCENE files
//!
//! # Selection System
//!
//! The editor includes a comprehensive entity selection system with:
//! - **Multi-entity selection**: Select multiple entities with add/remove/toggle modes
//! - **Click-to-select**: Raycast picking in viewport to select entities
//! - **Marquee selection**: Drag to create selection rectangle
//! - **Keyboard shortcuts**: Ctrl+A (select all), Ctrl+D (deselect all)
//! - **Selection events**: Track when selection changes for UI updates
//!
//! See the [`selection`] module and `SELECTION_SYSTEM.md` for detailed documentation.

mod editor_mode;
mod editor_state;
mod gizmo;
mod panels;
pub mod selection;
mod undo;
pub mod drag_drop;

pub use drag_drop::{DragDropPayload, DragDropSystem};
pub use editor_mode::EditorMode;
pub use editor_state::{EditorState, EditorTab};
pub use gizmo::{
    Gizmo, GizmoAxis, GizmoInteraction, GizmoMode, GizmoSpace, GizmoSystem, TransformGizmo,
};
pub use panels::{
    AssetEntry, AssetImportConfig, AssetType, AssetsPanel, ConsolePanel, EditorPanel,
    HierarchyPanel, InspectorPanel, SceneViewPanel, ViewportPanel,
};
pub use selection::{
    handle_selection_input_system, update_selection_system, Selectable, Selected, SelectionEvent,
    SelectionMode, SelectionSystem,
};
pub use undo::{TransformCommand, UndoRedoSystem};

use praxis_utils::{info, Result};

/// Initializes the editor system.
///
/// This function sets up any necessary global state for the editor system.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for editor
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., loading editor preferences,
///   registering custom panels, initializing editor resources)
///
/// # Example
///
/// ```rust,no_run
/// praxis_editor::init().expect("Failed to initialize editor system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    info!("Initializing editor system");
    Ok(())
}
