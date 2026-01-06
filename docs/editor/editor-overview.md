# Praxis Editor System Documentation

Comprehensive documentation for the Praxis game engine editor system, covering architecture, usage, customization, keyboard shortcuts, and troubleshooting.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Getting Started](#getting-started)
3. [Usage Guide](#usage-guide)
4. [Extending the Editor](#extending-the-editor)
5. [Keyboard Shortcuts Reference](#keyboard-shortcuts-reference)
6. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

The Praxis Editor is a comprehensive in-engine editor built on top of the ECS (Entity Component System) architecture. It provides a modular, extensible interface for creating and managing game content.

### Core Components

```
praxis_editor
├── EditorState          # Root coordinator managing all panels and modes
├── EditorMode           # Edit/Play mode state machine
├── Panels               # Modular UI components
│   ├── SceneViewPanel   # 3D viewport
│   ├── HierarchyPanel   # Entity tree
│   ├── InspectorPanel   # Component editing
│   ├── ConsolePanel     # Log output
│   └── AssetsPanel      # Asset browser
├── SelectionSystem      # Multi-entity selection
├── GizmoSystem          # Transform manipulation
├── UndoRedoSystem       # Command pattern undo/redo
├── EditorCameraController # Orbit camera for viewport
├── MenuBar              # File/Edit/Entity/View/Help menus
├── Toolbar              # Quick-access tools
└── PlayModeSystem       # Edit/Play transitions
```

### Key Design Principles

1. **Modularity**: Each panel and system is self-contained and can be used independently
2. **ECS Integration**: Leverages Bevy ECS for entity management and queries
3. **Command Pattern**: All editor operations are undoable via the command system
4. **Serialization**: Full RON (Rusty Object Notation) support for saving/loading
5. **Flexibility**: Dockable panels can be rearranged, split, and tabbed

### Data Flow

```
User Input → InputState
    ↓
EditorState → Menu/Toolbar/Panels
    ↓
SelectionSystem ← User clicks/drags
    ↓
GizmoSystem → Transform manipulation
    ↓
UndoRedoSystem (via Commands)
    ↓
ECS World (entities/components modified)
```

---

## Getting Started

### Basic Setup

```rust
use praxis_editor::{EditorState, init_with_console, LogBuffer, UndoRedoSystem};
use praxis_ecs::World;
use praxis_gui::EguiContext;

fn main() -> Result<()> {
    // Initialize engine systems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_gui::init()?;
    
    // Initialize editor with console logging
    let log_buffer = LogBuffer::new();
    init_with_console(log_buffer.clone())?;
    
    // Create ECS world
    let mut world = World::new();
    world.insert_resource(UndoRedoSystem::new());
    
    // Create editor state
    let mut editor_state = EditorState::with_log_buffer(log_buffer);
    
    // In your render loop:
    // editor_state.ui(&egui_context, Some(&mut undo_system), Some(&mut world));
    
    Ok(())
}
```

### EditorState API

The `EditorState` is the main entry point for the editor:

```rust
// Create editor
let mut editor = EditorState::new();

// Or with console integration
let log_buffer = LogBuffer::new();
let mut editor = EditorState::with_log_buffer(log_buffer);

// Toggle edit/play mode
editor.toggle_mode();

// Access panels
let hierarchy = editor.hierarchy_panel_mut();
let inspector = editor.inspector_panel_mut();

// Render UI each frame
editor.ui(&egui_ctx, Some(&mut undo_system), Some(&mut world));
```

---

## Usage Guide

### Editor Modes

The editor supports two primary modes:

#### Edit Mode (Default)
- **Purpose**: Scene editing and manipulation
- **Features**: Full transform gizmos, selection, hierarchy editing
- **Behavior**: Game simulation is paused
- **Visual**: Dark gray viewport border

#### Play Mode
- **Purpose**: Testing and debugging game functionality
- **Features**: Runtime simulation while editor remains visible
- **Behavior**: Game logic executes, physics simulates
- **Visual**: Green viewport border when playing, orange when paused
- **Controls**: 
  - **Play**: Enter play mode (takes scene snapshot)
  - **Pause**: Pause simulation
  - **Stop**: Exit play mode (restores scene snapshot)

### Panel System

#### Scene View Panel
Main 3D viewport for visualizing and interacting with the scene.

**Features:**
- Real-time 3D rendering
- Editor camera with orbit controls
- Transform gizmos (translate/rotate/scale)
- Raycast entity picking
- Marquee (box) selection
- Visual border indicating play mode state

**Camera Controls:**
- **Alt + LMB**: Orbit rotation around target
- **Alt + MMB**: Pan camera view
- **Scroll Wheel**: Zoom in/out
- **F**: Focus on selected entities

#### Hierarchy Panel
Tree view of scene entities with parent-child relationships.

**Features:**
- Hierarchical entity list
- Drag-and-drop reparenting
- Entity creation/deletion
- Search and filtering
- Selection synchronization

**Usage:**
- **Click**: Select entity
- **Ctrl+Click**: Multi-select
- **Right-click**: Context menu (create child, delete, etc.)
- **Drag**: Reparent entities

#### Inspector Panel
Component editing for selected entities.

**Features:**
- Transform component editing (position, rotation, scale)
- Component list view
- Add/remove components
- Property editing with undo/redo
- Multi-entity editing (coming soon)

**Components Supported:**
- Transform (translation, rotation, scale)
- Name (entity naming)
- Parent/Children (hierarchy)
- Camera, Mesh, Material (when selected)

#### Console Panel
Real-time log output with filtering and search.

**Features:**
- Captures all engine logs via `tracing` integration
- Log level filtering (trace, debug, info, warn, error)
- Real-time search across messages
- Auto-scroll with manual history review
- Color-coded log levels with timestamps
- Clear button
- Thread-safe logging (1000 message buffer)

**Log Levels:**
- **Trace** (gray): Low-level debugging
- **Debug** (cyan): Development info
- **Info** (white): General information
- **Warn** (yellow): Warning messages
- **Error** (red): Error messages

#### Assets Panel
Project asset browser with preview and management.

**Features:**
- Filesystem navigation with breadcrumb trail
- Thumbnail previews for textures
- Drag-and-drop asset placement
- Search and filtering by type
- Import dialogs with settings
- Hot-reload support (file watcher)

**Supported Asset Types:**
- **Textures**: PNG, JPG, JPEG
- **Models**: OBJ, GLTF, GLB
- **Audio**: WAV, OGG, MP3
- **Scenes**: SCENE files

**Usage:**
- **Click**: Select asset
- **Double-click**: Open/import asset
- **Drag to viewport**: Instantiate in scene
- **Right-click**: Context menu (delete, reimport, etc.)

### Selection System

Multi-entity selection with various interaction methods.

**Selection Modes:**
- **Replace** (Click): Clear selection, select clicked entity
- **Add** (Shift+Click): Add to selection
- **Remove** (Ctrl+Click): Remove from selection
- **Toggle** (Alt+Click): Toggle selection state

**Selection Methods:**
1. **Click Selection**: Raycast picking in viewport
2. **Marquee Selection**: Drag box to select multiple entities
3. **Hierarchy Selection**: Click entities in hierarchy panel
4. **Keyboard Shortcuts**:
   - **Ctrl+A**: Select all
   - **Ctrl+D**: Deselect all

**Selection Events:**
The `SelectionSystem` fires events that panels can listen to:
- `SelectionEvent::Selected(entities)`: Entities added to selection
- `SelectionEvent::Deselected(entities)`: Entities removed
- `SelectionEvent::Cleared`: All deselected
- `SelectionEvent::Changed`: Generic change notification

### Transform Gizmos

Visual 3D manipulation tools for selected entities.

**Gizmo Modes:**
- **Translate (W)**: Move entities along axes (red=X, green=Y, blue=Z)
- **Rotate (E)**: Rotate entities around axes
- **Scale (R)**: Scale entities along axes

**Coordinate Spaces:**
- **World Space**: Axes aligned with world coordinates
- **Local Space**: Axes aligned with entity rotation
- **Toggle**: **X** key

**Interaction:**
1. Hover over axis (highlights in lighter color)
2. Click and drag axis to manipulate
3. Release to confirm (creates undo command)
4. **Esc**: Cancel manipulation

**Snap Settings:**
- **Grid Snap** (translate): Snap to grid increments (default: 0.5 units)
- **Angle Snap** (rotate): Snap to angle increments (default: 15°)
- **Toggle snap**: Toolbar buttons

### Undo/Redo System

Comprehensive command-based undo/redo with dirty state tracking.

**Features:**
- 100 command history limit
- All editor operations are undoable
- Composite commands for multi-operation transactions
- Dirty state tracking for unsaved changes
- RON serialization for session recovery

**Available Commands:**
- `TransformEditCommand`: Transform changes
- `CreateEntityCommand`: Entity creation
- `DeleteEntityCommand`: Entity deletion
- `AddComponentCommand`: Component addition
- `RemoveComponentCommand`: Component removal
- `SetParentCommand`: Hierarchy changes
- `CompositeCommand`: Grouped operations

**Usage Example:**
```rust
use praxis_editor::{UndoRedoSystem, TransformEditCommand};

let mut undo_system = UndoRedoSystem::new();

// Create and execute command
let command = Box::new(TransformEditCommand::new(
    entity,
    old_transform,
    new_transform,
));
undo_system.execute_command(&mut world, command)?;

// Undo/redo
undo_system.undo(&mut world)?;
undo_system.redo(&mut world)?;

// Check dirty state
if undo_system.is_dirty() {
    // Prompt to save
}

// Mark as saved
undo_system.mark_saved();
```

### Menu Bar

Standard menu system with keyboard shortcuts.

**File Menu:**
- **New** (Ctrl+N): Create new scene
- **Open** (Ctrl+O): Open existing scene
- **Save** (Ctrl+S): Save current scene
- **Save As** (Ctrl+Shift+S): Save with new name
- **Exit** (Alt+F4): Close editor

**Edit Menu:**
- **Undo** (Ctrl+Z): Undo last command
- **Redo** (Ctrl+Y): Redo last undone command
- **Copy** (Ctrl+C): Copy selected entities
- **Paste** (Ctrl+V): Paste copied entities
- **Duplicate** (Ctrl+D): Duplicate selected entities

**Entity Menu:**
- **Create Empty**: Spawn empty entity
- **Create Cube**: Spawn cube mesh
- **Create Sphere**: Spawn sphere mesh
- **Create Plane**: Spawn plane mesh
- **Create Cylinder**: Spawn cylinder mesh
- **Create Cone**: Spawn cone mesh
- **Delete** (Delete): Delete selected entities

**View Menu:**
- Toggle visibility of panels:
  - Hierarchy
  - Inspector
  - Console
  - Assets
  - Scene View

**Help Menu:**
- **About**: Editor information
- **Documentation** (F1): Open documentation

### Toolbar

Quick-access tools and mode buttons.

**Gizmo Controls:**
- **Translate** (W): Switch to translate mode
- **Rotate** (E): Switch to rotate mode
- **Scale** (R): Switch to scale mode
- **World/Local** (X): Toggle coordinate space

**Snap Settings:**
- **Grid Snap**: Toggle grid snapping (translate)
- **Angle Snap**: Toggle angle snapping (rotate)
- **Settings**: Configure snap increments

**Playback Controls:**
- **Play** (Space): Enter play mode
- **Pause**: Pause simulation
- **Stop**: Exit play mode (restore scene)

**Camera Presets:**
- **Top**: View from above
- **Front**: View from front
- **Right**: View from right
- **Perspective**: Default 3D view

---

## Extending the Editor

### Creating Custom Panels

Implement the `EditorPanel` trait to create new panels:

```rust
use praxis_editor::EditorPanel;
use egui::Ui;

pub struct MyCustomPanel {
    title: String,
    // Your panel state
}

impl MyCustomPanel {
    pub fn new() -> Self {
        Self {
            title: "My Panel".to_string(),
        }
    }
}

impl EditorPanel for MyCustomPanel {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("My Custom Panel");
        ui.separator();
        
        // Your panel UI here
        ui.label("Custom content");
        if ui.button("Do Something").clicked() {
            // Handle button click
        }
    }
    
    fn on_close(&mut self) {
        // Cleanup when panel closes
    }
}
```

#### Adding Panel to EditorState

To integrate your panel into the editor:

1. Add field to `EditorState`:
```rust
// In editor_state.rs
pub struct EditorState {
    // ... existing fields
    my_custom_panel: MyCustomPanel,
}
```

2. Add to dock state initialization:
```rust
// In EditorState::new()
let custom_node = tree.split_below(some_node, 0.5, vec![EditorTab::Custom]);
```

3. Add panel to `EditorTabViewer`:
```rust
impl TabViewer for EditorTabViewer<'_> {
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            // ... existing panels
            EditorTab::Custom => self.my_custom_panel.ui(ui),
        }
    }
}
```

### Creating Custom Components

Add custom components for specialized entities:

```rust
use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MyComponent {
    pub value: f32,
    pub enabled: bool,
}

impl Default for MyComponent {
    fn default() -> Self {
        Self {
            value: 1.0,
            enabled: true,
        }
    }
}
```

#### Integrating Component with Inspector

To make your component editable in the inspector:

1. Add component detection:
```rust
// In inspector_panel.rs
if let Some(my_comp) = entity_ref.get::<MyComponent>() {
    ui.collapsing("My Component", |ui| {
        // Render component UI
    });
}
```

2. Add editing commands:
```rust
// Create edit command for undo/redo
let command = Box::new(EditMyComponentCommand::new(
    entity,
    old_value,
    new_value,
));
undo_system.execute_command(&mut world, command)?;
```

### Creating Custom Commands

Implement `EditorCommand` for undoable operations:

```rust
use praxis_editor::EditorCommand;
use bevy_ecs::world::World;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyCustomCommand {
    data: String,
    executed: bool,
}

impl EditorCommand for MyCustomCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        // Perform operation
        self.executed = true;
        Ok(())
    }
    
    fn undo(&mut self, world: &mut World) -> Result<()> {
        // Reverse operation
        self.executed = false;
        Ok(())
    }
    
    fn redo(&mut self, world: &mut World) -> Result<()> {
        self.execute(world)
    }
    
    fn description(&self) -> String {
        "My Custom Operation".to_string()
    }
    
    fn to_ron(&self) -> Result<String> {
        ron::to_string(&SerializableCommand::Custom(self.clone()))
            .map_err(|e| format!("Serialization failed: {}", e))
    }
    
    fn type_id(&self) -> &'static str {
        "MyCustomCommand"
    }
}
```

#### Registering Custom Commands

Add to `SerializableCommand` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializableCommand {
    // ... existing commands
    Custom(MyCustomCommand),
}

impl SerializableCommand {
    pub fn to_trait_object(self) -> Box<dyn EditorCommand> {
        match self {
            // ... existing matches
            SerializableCommand::Custom(cmd) => Box::new(cmd),
        }
    }
}
```

### Creating Custom ECS Systems

Add custom systems that integrate with the editor:

```rust
use bevy_ecs::system::{Query, Res};
use praxis_editor::{SelectionSystem, Selected};

pub fn my_custom_system(
    query: Query<&Transform, With<Selected>>,
    selection: Res<SelectionSystem>,
) {
    // Operate on selected entities
    for transform in query.iter() {
        // Do something with selected transforms
    }
}
```

#### Integrating with Editor Schedule

```rust
use praxis_ecs::Schedule;

let mut schedule = Schedule::default();
schedule.add_systems((
    my_custom_system,
    // ... other systems
));
```

### Extending Menu Bar

Add custom menu items:

```rust
// In menu_bar.rs or your extension file
pub enum CustomMenuAction {
    DoSomething,
    DoSomethingElse,
}

pub fn render_custom_menu(ui: &mut egui::Ui) -> Vec<CustomMenuAction> {
    let mut actions = vec![];
    
    ui.menu_button("Custom", |ui| {
        if ui.button("Do Something").clicked() {
            actions.push(CustomMenuAction::DoSomething);
            ui.close_menu();
        }
    });
    
    actions
}

pub fn handle_custom_action(action: CustomMenuAction, world: &mut World) {
    match action {
        CustomMenuAction::DoSomething => {
            // Handle action
        }
        CustomMenuAction::DoSomethingElse => {
            // Handle other action
        }
    }
}
```

### Asset Type Extensions

Add support for new asset types:

```rust
use praxis_editor::{AssetType, AssetImportConfig};

pub enum CustomAssetType {
    MyFormat,
}

impl From<&Path> for CustomAssetType {
    fn from(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("myformat") => CustomAssetType::MyFormat,
            _ => panic!("Unsupported format"),
        }
    }
}

pub fn import_custom_asset(path: &Path, config: &AssetImportConfig) -> Result<()> {
    // Load and import asset
    Ok(())
}
```

---

## Keyboard Shortcuts Reference

### Global Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| **Esc** | Cancel current operation | Any |
| **F1** | Open documentation | Any |
| **Space** | Play/Pause game | Edit/Play mode |

### File Operations

| Shortcut | Action | Menu |
|----------|--------|------|
| **Ctrl+N** | New scene | File → New |
| **Ctrl+O** | Open scene | File → Open |
| **Ctrl+S** | Save scene | File → Save |
| **Ctrl+Shift+S** | Save scene as | File → Save As |
| **Alt+F4** | Exit editor | File → Exit |

### Editing Operations

| Shortcut | Action | Menu |
|----------|--------|------|
| **Ctrl+Z** | Undo | Edit → Undo |
| **Ctrl+Y** | Redo | Edit → Redo |
| **Ctrl+Shift+Z** | Redo (alternate) | Edit → Redo |
| **Ctrl+C** | Copy selection | Edit → Copy |
| **Ctrl+V** | Paste | Edit → Paste |
| **Ctrl+D** | Duplicate selection | Edit → Duplicate |

### Entity Operations

| Shortcut | Action | Menu |
|----------|--------|------|
| **Delete** | Delete selected entities | Entity → Delete |
| **Ctrl+Shift+N** | Create empty entity | Entity → Create Empty |

### Selection

| Shortcut | Action | Context |
|----------|--------|---------|
| **Click** | Select entity | Viewport/Hierarchy |
| **Shift+Click** | Add to selection | Viewport/Hierarchy |
| **Ctrl+Click** | Remove from selection | Viewport/Hierarchy |
| **Alt+Click** | Toggle selection | Viewport/Hierarchy |
| **Ctrl+A** | Select all | Viewport/Hierarchy |
| **Ctrl+D** | Deselect all | Viewport/Hierarchy |
| **Drag** | Marquee selection | Viewport |

### Transform Gizmos

| Shortcut | Action | Toolbar |
|----------|--------|---------|
| **W** | Translate mode | Gizmo → Translate |
| **E** | Rotate mode | Gizmo → Rotate |
| **R** | Scale mode | Gizmo → Scale |
| **X** | Toggle World/Local space | Gizmo → Space |

### Camera Controls

| Shortcut | Action | Context |
|----------|--------|---------|
| **Alt+LMB Drag** | Orbit camera | Viewport |
| **Alt+MMB Drag** | Pan camera | Viewport |
| **Scroll Wheel** | Zoom camera | Viewport |
| **F** | Focus on selection | Viewport |

### Panel Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| **Ctrl+1** | Toggle Hierarchy panel | View → Hierarchy |
| **Ctrl+2** | Toggle Inspector panel | View → Inspector |
| **Ctrl+3** | Toggle Console panel | View → Console |
| **Ctrl+4** | Toggle Assets panel | View → Assets |

---

## Troubleshooting

### Common Issues

#### Editor Not Visible

**Symptoms:** Editor UI doesn't appear when running.

**Solutions:**
1. Ensure `editor.ui()` is called in render loop
2. Check that `editor.set_visible(true)` has been called
3. Verify egui context is properly initialized:
```rust
world.insert_resource(EguiContext::default());
```
4. Make sure window has input focus

#### Selection Not Working

**Symptoms:** Clicking entities doesn't select them.

**Solutions:**
1. Add `Selectable` component to entities:
```rust
world.spawn((Transform::default(), Selectable));
```
2. Ensure `SelectionSystem` resource exists:
```rust
world.insert_resource(SelectionSystem::new());
```
3. Add selection systems to schedule:
```rust
schedule.add_systems((
    update_selection_system,
    handle_selection_input_system,
));
```
4. Check that input is being captured correctly

#### Undo/Redo Not Working

**Symptoms:** Ctrl+Z/Ctrl+Y don't undo/redo operations.

**Solutions:**
1. Verify `UndoRedoSystem` resource exists:
```rust
world.insert_resource(UndoRedoSystem::new());
```
2. Ensure commands are executed through the system:
```rust
undo_system.execute_command(&mut world, Box::new(command))?;
```
3. Check that keyboard input is being processed
4. Make sure `editor.ui()` receives the undo system reference:
```rust
editor.ui(&ctx, Some(&mut undo_system), Some(&mut world));
```

#### Gizmos Not Appearing

**Symptoms:** Transform gizmos don't show for selected entities.

**Solutions:**
1. Insert `GizmoSystem` resource:
```rust
world.insert_resource(GizmoSystem::new());
```
2. Ensure entities are selected in `SelectionSystem`
3. Check that gizmos are enabled:
```rust
gizmo_system.set_enabled(true);
```
4. Verify entities have `Transform` component

#### Play Mode Not Working

**Symptoms:** Can't enter/exit play mode or changes aren't restored.

**Solutions:**
1. Ensure world reference is passed to `editor.ui()`:
```rust
editor.ui(&ctx, Some(&mut undo_system), Some(&mut world));
```
2. Check that entities have serializable components
3. Verify `PlayModeSystem` is properly initialized
4. Use the proper methods:
```rust
editor.enter_play_mode(&mut world)?;
editor.exit_play_mode(&mut world)?;
```

#### Console Not Capturing Logs

**Symptoms:** Engine logs don't appear in console panel.

**Solutions:**
1. Initialize editor with console integration:
```rust
let log_buffer = LogBuffer::new();
init_with_console(log_buffer.clone())?;
```
2. Create editor state with log buffer:
```rust
let editor = EditorState::with_log_buffer(log_buffer);
```
3. Ensure `tracing` subscriber is set up correctly
4. Check that console panel is visible in dock

#### Assets Not Loading

**Symptoms:** Assets panel is empty or assets don't import.

**Solutions:**
1. Create `assets/` directory in project root
2. Check file permissions for asset directory
3. Verify asset file formats are supported
4. Check console for import errors
5. Ensure asset paths are correct

#### Transform Edits Not Saving

**Symptoms:** Transform changes revert or aren't undoable.

**Solutions:**
1. Wrap edits in commands:
```rust
let command = Box::new(TransformEditCommand::new(entity, old, new));
undo_system.execute_command(&mut world, command)?;
```
2. Don't modify transforms directly; use commands
3. Ensure entities have both `Transform` and `GlobalTransform` components

### Performance Issues

#### Slow Editor UI

**Symptoms:** Editor UI is laggy or unresponsive.

**Solutions:**
1. Reduce panel update frequency
2. Limit hierarchy panel to visible items only
3. Disable auto-scroll in console for large logs
4. Use asset thumbnails sparingly
5. Profile with `tracy` or `puffin` to identify bottlenecks

#### High Memory Usage

**Symptoms:** Memory usage grows over time.

**Solutions:**
1. Command history has 100 entry limit (check if reasonable)
2. Console panel has 1000 message buffer (clear periodically)
3. Asset thumbnails may accumulate (implement cache eviction)
4. Check for entity leaks in play mode transitions

#### Slow Play Mode Transitions

**Symptoms:** Entering/exiting play mode takes a long time.

**Solutions:**
1. Reduce number of entities in scene
2. Simplify entity component structure
3. Optimize serialization for custom components
4. Consider partial scene snapshots instead of full world

### Debug Logging

Enable detailed editor logging:

```rust
use tracing::Level;

// Set log level to debug
std::env::set_var("RUST_LOG", "praxis_editor=debug");

// Or for specific modules
std::env::set_var("RUST_LOG", "praxis_editor::selection=trace");
```

Check console panel for:
- Command execution logs
- Selection events
- Play mode state transitions
- Asset import logs

### Validation

Run validation checks:

```rust
// Check world state
assert!(world.contains_resource::<SelectionSystem>());
assert!(world.contains_resource::<UndoRedoSystem>());
assert!(world.contains_resource::<GizmoSystem>());

// Verify editor state
assert!(editor.is_visible());
assert_eq!(editor.mode(), EditorMode::Edit);

// Validate command history
let undo_system = world.get_resource::<UndoRedoSystem>().unwrap();
println!("Undo stack: {}", undo_system.undo_count());
println!("Redo stack: {}", undo_system.redo_count());
println!("Dirty: {}", undo_system.is_dirty());
```

### Getting Help

If you continue to experience issues:

1. **Check Examples**: Review `examples/editor_demo.rs` for reference implementation
2. **Read Module Docs**: See `crates/praxis_editor/src/lib.rs` for detailed API docs
3. **Check Specific Docs**: Review specialized documentation:
   - `COMMAND_SYSTEM.md` - Undo/redo system
   - `SELECTION_SYSTEM.md` - Selection system
   - `PLAY_MODE_SYSTEM.md` - Play mode transitions
   - `EDITOR_CAMERA.md` - Camera controls
   - `GIZMOS.md` - Transform gizmos
4. **Enable Debug Logging**: Set `RUST_LOG=praxis_editor=debug`
5. **File Issue**: Report bugs on GitHub with reproduction steps

---

## Additional Resources

### Documentation Files

- **Core Editor**:
  - `crates/praxis_editor/README.md` - Editor overview
  - `crates/praxis_editor/src/lib.rs` - Complete API documentation

- **Systems**:
  - `crates/praxis_editor/COMMAND_SYSTEM.md` - Command pattern details
  - `crates/praxis_editor/UNDO_REDO_SYSTEM.md` - Undo/redo integration
  - `crates/praxis_editor/SELECTION_SYSTEM.md` - Selection architecture
  - `crates/praxis_editor/PLAY_MODE_SYSTEM.md` - Play mode transitions
  - `crates/praxis_editor/EDITOR_CAMERA.md` - Camera controller
  - `crates/praxis_editor/GIZMOS.md` - Transform gizmos
  - `crates/praxis_editor/MENU_BAR.md` - Menu bar system
  - `crates/praxis_editor/TOOLBAR_SYSTEM.md` - Toolbar documentation

- **Panels**:
  - `crates/praxis_editor/VIEWPORT_PANEL.md` - Scene view panel

### Example Programs

- `examples/editor_demo.rs` - Complete editor setup
- `examples/selection_demo.rs` - Selection system demo
- `examples/command_system_demo.rs` - Command pattern demo
- `examples/editor_camera_demo.rs` - Camera controls demo

### External Documentation

- [Bevy ECS Book](https://bevyengine.org/learn/book/getting-started/ecs/) - ECS concepts
- [egui Documentation](https://docs.rs/egui/) - UI framework
- [RON Format](https://github.com/ron-rs/ron) - Serialization format

---

## Version History

- **v0.1.0** (2025-01-XX): Initial editor system implementation
  - Core panel system with docking
  - Selection and gizmo systems
  - Undo/redo with command pattern
  - Edit/Play mode transitions
  - Console integration
  - Asset browser

---

## License

See `LICENSE` file in repository root.
