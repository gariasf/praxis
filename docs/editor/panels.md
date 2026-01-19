# Editor Panels

Overview of the dockable panel system using `egui_dock` for flexible editor layouts.

## Panel System

The Praxis editor uses a dockable panel architecture that allows users to:
- Drag and rearrange panels
- Split panels horizontally or vertically
- Tab panels together
- Close and reopen panels via View menu

All panels implement the `EditorPanel` trait which provides a consistent interface for rendering and lifecycle management.

## Built-in Panels

### Hierarchy Panel
**Location**: Left side by default  
**Purpose**: Tree view of all scene entities showing parent-child relationships

**Key Features**:
- Hierarchical entity list with collapsible nodes
- Drag-and-drop reparenting
- Entity creation/deletion with undo support
- Multi-selection integration
- Search and filtering
- Live updates as entities spawn/despawn

**See**: [Hierarchy Panel Guide](hierarchy-panel.md)

---

### Inspector Panel
**Location**: Right side by default  
**Purpose**: Component editing for selected entities

**Key Features**:
- View and edit all components on selected entities
- Type-specific editors (Transform, Camera, Light, Physics, etc.)
- Add/remove components with undo support
- Real-time property updates
- Material editing with color pickers and sliders
- Physics component visualization

**See**: [Inspector Panel Guide](inspector.md)

---

### Scene View Panel
**Location**: Center by default  
**Purpose**: 3D viewport for visualizing and interacting with the scene

**Key Features**:
- Real-time 3D rendering
- Orbit camera controls (Alt+LMB, Alt+MMB, Scroll)
- Transform gizmos (translate/rotate/scale)
- Raycast entity picking
- Marquee (box) selection
- Grid floor with axis indicators
- Focus on selection (F key)
- Visual border indicating play mode state

**Camera Controls**:
- **Alt + LMB Drag**: Orbit rotation around target
- **Alt + MMB Drag**: Pan camera view
- **Scroll Wheel**: Zoom in/out
- **F**: Focus on selected entities

**Technical Details**: See [crates/praxis_editor/VIEWPORT_PANEL.md](../../crates/praxis_editor/VIEWPORT_PANEL.md)

---

### Console Panel
**Location**: Bottom by default  
**Purpose**: Real-time log output with filtering and search

**Key Features**:
- Captures all engine logs via `tracing` integration
- Log level filtering (trace, debug, info, warn, error)
- Real-time search across messages
- Auto-scroll with manual history review
- Color-coded log levels with timestamps
- Clear button
- Thread-safe logging (1000 message buffer)

**Log Levels**:
- **Trace** (gray): Low-level debugging
- **Debug** (cyan): Development info
- **Info** (white): General information
- **Warn** (yellow): Warning messages
- **Error** (red): Error messages

---

### Assets Panel
**Location**: Bottom-left by default  
**Purpose**: Project asset browser with preview and management

**Key Features**:
- Filesystem navigation with breadcrumb trail
- Thumbnail previews for textures (96×96 pixels)
- Drag-and-drop asset placement into scene
- Search and filtering by name/type
- Import dialogs with per-asset-type settings
- Hot-reload support via file watcher
- Context menus for asset operations

**Supported Asset Types**:
- **Textures**: PNG, JPG, JPEG
- **Models**: OBJ, GLTF, GLB
- **Audio**: WAV, OGG, MP3
- **Scenes**: SCENE files

**See**: [Asset Browser Guide](asset-browser.md)

---

### Project Settings Panel (Optional)
**Location**: Configurable  
**Purpose**: Project-wide configuration settings

**Key Features**:
- Physics settings (gravity, timestep)
- Rendering settings (shadow quality, ambient lighting)
- Audio settings (master volume, doppler scale)
- Input mappings
- Build settings

**Note**: This panel is available but not shown by default. Enable via View menu or custom editor configuration.

---

### Terrain Panel (Optional, requires `terrain` feature)
**Location**: Configurable  
**Purpose**: Terrain generation and editing tools

**Key Features**:
- Procedural terrain generation
- Height map editing
- Texture splatting
- LOD configuration
- Terrain sculpting tools

**Note**: Only available when compiled with the `terrain` feature flag.

## Panel Architecture

### EditorPanel Trait

All panels implement the `EditorPanel` trait:

```rust
pub trait EditorPanel {
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut egui::Ui, world: Option<&World>, render_context: Option<&mut RenderContext>);
    fn is_open(&self) -> bool { true }
    fn set_open(&mut self, open: bool) {}
    fn on_close(&mut self) {}
}
```

This provides:
- Consistent panel rendering interface
- Optional world and render context access
- Open/close state management
- Cleanup on panel close

### Docking System

Panels use `egui_dock` for layout management:

```rust
use egui_dock::{DockArea, DockState, NodeIndex, Style};

// Panels are organized in a tree structure
let tree = DockState::new(vec![/* panel tabs */]);

// Split operations
tree.split_left(node, 0.3, vec![HierarchyTab]);
tree.split_right(node, 0.25, vec![InspectorTab]);
tree.split_below(node, 0.3, vec![ConsoleTab, AssetsTab]);
```

**Layout Features**:
- Drag panel tabs to rearrange
- Drag to window edge to dock
- Drag tabs together to create tab groups
- Right-click tab for close option
- Window → View menu to show/hide panels

## Panel Visibility

Control panel visibility via:

1. **View Menu**: Checkboxes for each panel
2. **Panel Close Button**: X on panel tab
3. **Programmatic Control**:
```rust
menu_state.show_hierarchy = true;
menu_state.show_inspector = false;
// etc.
```

## Custom Panels

Create custom panels by implementing `EditorPanel`:

```rust
use praxis_editor::EditorPanel;

pub struct MyCustomPanel {
    title: String,
    // Your state
}

impl EditorPanel for MyCustomPanel {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn ui(&mut self, ui: &mut egui::Ui, world: Option<&World>, render_context: Option<&mut RenderContext>) {
        ui.heading("My Custom Panel");
        // Your UI code
    }
}
```

Then integrate with `EditorState` or render directly in your application.

## Panel State Persistence

Panel layout and visibility can be saved/restored:

```rust
// Save layout
let layout = editor.save_layout();
std::fs::write("layout.ron", ron::to_string(&layout)?)?;

// Restore layout
let layout = ron::from_str(&std::fs::read_to_string("layout.ron")?)?;
editor.restore_layout(layout);
```

This preserves:
- Panel positions and sizes
- Tab groupings
- Split ratios
- Visibility states

## Best Practices

1. **Panel Design**: Keep panels focused on a single concern
2. **Performance**: Update only when visible or state changes
3. **Integration**: Use provided world/render_context rather than global state
4. **Cleanup**: Implement `on_close()` to release resources
5. **Responsiveness**: Design panels to work at various sizes

## See Also

- [Hierarchy Panel](hierarchy-panel.md) - Entity tree management
- [Inspector Panel](inspector.md) - Component editing
- [Asset Browser](asset-browser.md) - Asset management
- [Selection System](selection-system.md) - Entity selection
- [Editor Overview](editor-overview.md) - Overall architecture
