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
//!   - `ConsolePanel`: Log output with filtering, search, and tracing integration
//!   - `AssetsPanel`: Project asset browser
//!   - `OptimizationPanel`: Rendering optimization configuration with performance comparison
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
//! ## Console Panel
//!
//! The `ConsolePanel` provides comprehensive logging and debugging:
//! - **Real-time Log Capture**: Integrates with `praxis_utils::tracing` to capture all engine logs
//! - **Log Filtering**: Filter by level (trace/debug/info/warn/error) with toggle buttons
//! - **Search Functionality**: Real-time search to filter messages by content or module
//! - **Auto-scroll**: Automatically scroll to new messages or manually review history
//! - **Clear Button**: Remove all messages with one click
//! - **Color Coding**: Visual distinction between log levels with timestamps
//! - **Thread-safe**: Uses `Arc<Mutex<VecDeque>>` for safe concurrent access
//! - **Buffer Limit**: Maintains maximum of 1000 messages to prevent memory overflow
//!
//! **Usage:**
//! ```rust,no_run
//! use praxis_editor::{init_with_console, LogBuffer, EditorState};
//!
//! // Create shared log buffer
//! let log_buffer = LogBuffer::new();
//!
//! // Initialize with console integration
//! init_with_console(log_buffer.clone()).unwrap();
//!
//! // Create editor with log buffer
//! let editor = EditorState::with_log_buffer(log_buffer);
//!
//! // All engine logs now appear in the console panel
//! ```
//!
//! See `CONSOLE_PANEL_IMPLEMENTATION.md` and `examples/console_demo.rs` for details.
//!
//! ## Optimization Panel
//!
//! The `OptimizationPanel` provides comprehensive control over rendering optimizations with
//! real-time performance comparison:
//! - **Preset Management**: Low, Medium, High, and Ultra optimization profiles
//! - **Individual Toggles**: Fine-grained control over each optimization
//! - **Performance Comparison**: Before/after snapshots with color-coded deltas
//! - **Live Statistics**: Real-time monitoring of draw calls, culling, and more
//! - **Performance Graphs**: Visual trends of rendering metrics over time
//!
//! **Features:**
//! - Multi-draw indirect batching
//! - GPU culling (frustum, occlusion, backface, distance, small objects)
//! - GPU-driven LOD selection
//! - Descriptor caching
//! - Hi-Z occlusion culling
//! - Mesh streaming
//!
//! **Usage:**
//! ```rust,no_run
//! use praxis_editor::{OptimizationPanel, OptimizationPreset};
//! use praxis_graphics::RenderStats;
//!
//! let mut panel = OptimizationPanel::new();
//!
//! // Update with render stats each frame
//! panel.update_stats(stats);
//!
//! // Apply a preset
//! if let Some(config) = panel.config_mut() {
//!     OptimizationPreset::Ultra.apply_to(config);
//! }
//! ```
//!
//! See `OPTIMIZATION_PANEL.md` and `examples/optimization_panel_demo.rs` for details.
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
//!
//! # MenuBar System
//!
//! The editor provides a comprehensive menu bar system with standard menus and keyboard shortcuts:
//! - **File Menu**: New, Open, Save, Save As, Exit with shortcuts (Ctrl+N/O/S, Ctrl+Shift+S, Alt+F4)
//! - **Edit Menu**: Undo, Redo, Copy, Paste, Duplicate with shortcuts (Ctrl+Z/Y/C/V/D)
//! - **Entity Menu**: Create Empty, Create Primitives (Cube, Sphere, Plane, Cylinder, Cone), Delete (Delete key)
//! - **View Menu**: Toggle visibility of Hierarchy, Inspector, Console, Assets, and Scene View panels
//! - **Help Menu**: About, Documentation (F1)
//!
//! The menu bar automatically integrates with the undo/redo system, showing command descriptions
//! and dirty state indicators when there are unsaved changes.
//!
//! See the [`menu_bar`] module for detailed API documentation.
//!
//! # Toolbar System
//!
//! The editor includes a toolbar with quick-access buttons for common operations:
//! - **Gizmo Mode Buttons**: Translate (Move), Rotate, Scale modes with visual feedback
//! - **Space Toggle**: Switch between World and Local coordinate spaces
//! - **Snap Settings**: Toggle grid/angle snapping with configurable increments
//! - **Playback Controls**: Play, Pause, Stop buttons for game simulation
//! - **Camera Presets**: Quick access to Top, Front, Right, and Perspective views
//!
//! The toolbar automatically syncs with the editor state and provides visual feedback
//! for the current mode and settings.
//!
//! See the [`toolbar`] module for detailed API documentation.
//!
//! # Play Mode System
//!
//! The editor provides a comprehensive play mode system for testing game functionality:
//! - **Edit/Play State Machine**: Clean transitions between Edit and Play modes
//! - **Scene Snapshot/Restore**: Automatic capture and restoration of scene state
//! - **Runtime ECS Isolation**: Changes made in play mode don't affect the original scene
//! - **Input Routing Toggle**: Configurable input handling for play mode
//! - **Visual Indicators**: Viewport border color changes and button states reflect current mode
//!
//! **Usage:**
//! ```rust,no_run
//! use praxis_editor::EditorState;
//! use praxis_ecs::World;
//!
//! let mut world = World::new();
//! let mut editor = EditorState::new();
//!
//! // Enter play mode (takes snapshot)
//! editor.enter_play_mode(&mut world).unwrap();
//!
//! // Exit play mode (restores snapshot)
//! editor.exit_play_mode(&mut world).unwrap();
//! ```
//!
//! **Visual Feedback:**
//! - Edit Mode: Dark gray viewport border
//! - Play Mode: Green viewport border, green Play button
//! - Paused Mode: Orange viewport border
//!
//! See `PLAY_MODE_SYSTEM.md` for comprehensive documentation.
//!
//! # Command System and Undo/Redo
//!
//! The editor provides a comprehensive undo/redo system based on the **Command Pattern**.
//! This design pattern encapsulates each editor operation as a command object that knows
//! how to execute, undo, and redo itself. This provides several benefits:
//!
//! ## Command Pattern Implementation
//!
//! The command pattern in Praxis follows this structure:
//! 1. **Command Interface** (`EditorCommand` trait): Defines execute(), undo(), and redo() methods
//! 2. **Concrete Commands**: Specific implementations for different operations
//! 3. **Invoker** (`CommandHistory`): Manages command execution and history stacks
//! 4. **Receiver**: The ECS World that commands operate on
//!
//! ### Key Components
//!
//! - **`EditorCommand` trait**: Base interface for all undoable operations. Each command
//!   encapsulates both the action and the information needed to reverse it.
//! - **`CommandHistory`**: The invoker that manages undo/redo stacks with 100 entry maximum.
//!   Uses two VecDeques: one for undo (commands executed), one for redo (commands undone).
//! - **`UndoRedoSystem`**: ECS resource wrapper providing dirty state tracking and event system.
//!   Tracks whether there are unsaved changes by monitoring the clean state marker.
//! - **Concrete commands**: Individual command implementations:
//!   - `TransformEditCommand`: Stores old and new transform states
//!   - `CreateEntityCommand`: Stores entity ID after creation for undo
//!   - `DeleteEntityCommand`: Captures all components before deletion for restoration
//!   - Component commands: Store previous values for reliable undo
//!   - Hierarchy commands: Track parent-child relationships
//!
//! ### Composite Commands
//!
//! **Composite commands** allow grouping multiple operations into a single undoable action:
//! - Uses the Composite pattern to treat groups of commands as single commands
//! - Execute: Runs all child commands in sequence
//! - Undo: Reverses all child commands in *reverse* order (LIFO)
//! - Use case: Multi-entity deletion, batch transforms, entity duplication with components
//!
//! ### Command Execution Tracking
//!
//! Each command tracks its execution state to support proper undo/redo:
//! - `executed: bool` flag indicates whether command has been applied
//! - State changes: unexecuted -> executed -> undone -> executed (redo)
//! - Entity IDs captured during execute() for operations that create entities
//! - Component data captured before execute() for operations that delete/modify
//!
//! ### Serialization
//!
//! - **RON serialization**: Commands implement Serialize/Deserialize for persistence
//! - Save/load command history for session recovery or replay functionality
//! - `SerializableCommand` enum wraps all concrete command types
//! - Entity IDs stored as (index, generation) pairs, may need remapping on load
//!
//! ### Dirty State Tracking
//!
//! - Automatically tracks unsaved changes via clean state marker
//! - Clean state set when: file saved, new scene created
//! - Becomes dirty when: commands executed, scene modified
//! - Used for "unsaved changes" warnings and save indicators
//!
//! ### Menu Bar Integration
//!
//! - Edit > Undo/Redo: Shows command descriptions and keyboard shortcuts
//! - File > Save: Shows asterisk (*) when there are unsaved changes
//! - Status bar: Displays unsaved indicator when dirty
//! - History info: Shows undo/redo stack counts
//!
//! **Keyboard shortcuts:**
//! - Ctrl+Z: Undo last command (pops from undo stack, pushes to redo stack)
//! - Ctrl+Y or Ctrl+Shift+Z: Redo last undone command (pops from redo, pushes to undo)
//!
//! **Implementation Pattern:**
//! ```rust,ignore
//! // 1. Create command with necessary state
//! let command = TransformEditCommand::new(entity, old_transform, new_transform);
//!
//! // 2. Execute through CommandHistory (not directly)
//! command_history.execute(world, Box::new(command))?;
//! // This calls command.execute(world) AND adds to undo stack
//!
//! // 3. Undo reverses the operation
//! command_history.undo(world)?;
//! // Calls command.undo(world), moves command from undo to redo stack
//!
//! // 4. Redo reapplies the operation
//! command_history.redo(world)?;
//! // Calls command.redo(world), moves command back to undo stack
//! ```
//!
//! See `COMMAND_SYSTEM.md` for detailed documentation and examples.
//!
//! # Editor Camera Controller
//!
//! The editor provides a dedicated camera controller with orbit controls, separate from game cameras:
//! - **Orbit rotation**: Alt+LMB drag to rotate around target point
//! - **Pan movement**: Alt+MMB drag to pan camera view
//! - **Zoom**: Mouse scroll wheel to move closer/farther from target
//! - **Focus on selection**: F key to frame selected entities in view
//! - **Smooth interpolation**: Smooth camera movement with configurable speed
//!
//! **Setup:**
//! ```rust,no_run
//! use praxis_editor::{EditorCameraController, EditorCamera, update_editor_camera_system};
//! use praxis_ecs::{World, Schedule, PerspectiveCameraBundle};
//! use praxis_math::Vec3;
//!
//! let mut world = World::new();
//! world.insert_resource(EditorCameraController::new());
//!
//! // Create editor camera entity
//! world.spawn((
//!     PerspectiveCameraBundle::new(Vec3::new(0.0, 5.0, 10.0), 70.0_f32.to_radians(), 16.0/9.0),
//!     EditorCamera, // Marker component for editor camera
//! ));
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems(update_editor_camera_system);
//! ```
//!
//! **Features:**
//! - Independent from game cameras (use EditorCamera marker component)
//! - Orbits around a target point with configurable distance
//! - Smooth interpolation for all movements
//! - Automatic framing of selected entities
//! - Customizable sensitivity and constraints
//!
//! See the [`camera_controller`] module for detailed API documentation.

mod camera_controller;
mod command_shortcuts;
pub mod drag_drop;
mod editor_mode;
mod editor_state;
pub mod entity_operations;
mod gizmo;
mod menu_bar;
mod panels;
mod play_mode;
mod scene_operations;
pub mod selection;
mod toolbar;
mod undo;

pub use camera_controller::{update_editor_camera_system, EditorCamera, EditorCameraController};
pub use command_shortcuts::{handle_command_shortcuts, is_redo_pressed, is_undo_pressed};
pub use drag_drop::{DragDropPayload, DragDropSystem};
pub use editor_mode::EditorMode;
pub use editor_state::{EditorState, EditorTab};
pub use entity_operations::{EntityOperations, EntityOperationsError};
pub use gizmo::{
    Gizmo, GizmoAxis, GizmoInteraction, GizmoMode, GizmoSpace, GizmoSystem, TransformGizmo,
};
pub use menu_bar::{
    check_keyboard_shortcuts, handle_menu_action, render_menu_bar, MenuBarAction, MenuBarState,
};
pub use panels::{
    AssetEntry, AssetImportConfig, AssetType, AssetsPanel, AssetsPanelExt, ConsoleLayer,
    ConsolePanel, EditorPanel, HierarchyPanel, InspectorPanel, LogBuffer, LogLevel, LogMessage,
    OptimizationPanel, OptimizationPreset, SceneViewPanel, SceneViewPanelExt, ViewportPanel,
};
#[cfg(feature = "terrain")]
pub use panels::{TerrainPanel, TerrainPanelExt};
pub use play_mode::{PlayModeState, PlayModeSystem, SceneSnapshot, SnapshotMetadata};
pub use scene_operations::{
    capture_scene_from_world, load_scene_into_world, show_unsaved_changes_dialog,
};
pub use selection::{
    handle_selection_input_system, update_selection_system, Selectable, Selected, SelectionEvent,
    SelectionMode, SelectionSystem,
};
pub use toolbar::{
    handle_toolbar_action, render_toolbar, CameraPreset, SnapSettings, ToolbarAction, ToolbarState,
};
pub use undo::{
    AddComponentCommand, CommandHistory, ComponentData, CompositeCommand, CopyEntityCommand,
    CreateEntityCommand, DeleteEntityCommand, EditorCommand, PasteEntityCommand,
    RemoveComponentCommand, SerializableAudioSource, SerializableCollider, SerializableCommand,
    SerializableEntity, SerializableMass, SerializableMaterialProperties,
    SerializablePerspectiveProjection, SerializablePhysicsVelocity, SerializableRigidBody,
    SerializableTransform, SetParentCommand, TransformEditCommand, UndoRedoSystem,
};

use praxis_utils::{info, init_tracing_with_layer, Result};

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

/// Initializes the editor system with console log capturing.
///
/// This function sets up the tracing system with a custom layer that captures
/// logs and sends them to the provided log buffer, which can then be displayed
/// in the console panel.
///
/// # Arguments
///
/// * `log_buffer` - The log buffer to capture logs into
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::{init_with_console, LogBuffer, ConsolePanel};
///
/// let log_buffer = LogBuffer::new();
/// init_with_console(log_buffer.clone()).expect("Failed to initialize");
///
/// let console_panel = ConsolePanel::with_buffer(log_buffer);
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails.
pub fn init_with_console(log_buffer: LogBuffer) -> Result<()> {
    let console_layer = ConsoleLayer::new(log_buffer);
    init_tracing_with_layer(Some(console_layer))?;
    info!("Editor system initialized with console log capture");
    Ok(())
}
