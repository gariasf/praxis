# Toolbar System

The Praxis editor includes a comprehensive toolbar system that provides quick access to commonly-used editor operations. The toolbar is displayed as a horizontal panel below the menu bar and contains grouped buttons for different categories of operations.

## Features

### Gizmo Mode Selection

The toolbar provides three buttons for selecting the active gizmo transformation mode:

- **🔷 Move (Translate)**: Transform mode for moving entities along axes
- **🔄 Rotate**: Transform mode for rotating entities around axes
- **📏 Scale**: Transform mode for scaling entities along axes

The currently selected mode is highlighted visually. Each button includes a hover tooltip with keyboard shortcut hints (W/E/R).

### Coordinate Space Toggle

A single button toggles between two coordinate systems:

- **🌍 World**: Gizmo axes align with world coordinates (absolute)
- **📍 Local**: Gizmo axes align with the entity's local rotation (relative)

The button displays the current space and includes a tooltip with the keyboard shortcut (X).

### Snap Settings

The snap settings group provides:

- **Toggle Button**: Enable/disable grid snapping with visual feedback (ON/OFF state)
- **Grid Size Display**: Shows current grid snap increment when snapping is enabled
- **Configurable Settings**: Grid size, angle increment, and scale increment

Snapping can be toggled with Ctrl+\ keyboard shortcut.

### Playback Controls

Three buttons control game simulation:

- **▶ Play**: Start play mode (only enabled in Edit mode)
- **⏸ Pause**: Pause play mode and return to Edit (only enabled in Play mode)
- **⏹ Stop**: Stop play mode and return to Edit (only enabled in Play mode)

Each button includes hover tooltips with keyboard shortcuts (F5/F6/F7).

### Camera Presets

The camera preset group provides:

- **📷 View Menu**: Dropdown menu with all 7 camera presets
- **Quick Access Buttons**: Single-letter buttons for common views
  - **T**: Top view (looking down Y axis)
  - **F**: Front view (looking down Z axis)
  - **R**: Right view (looking left along X axis)
  - **P**: Perspective view (free camera)

The dropdown menu shows all available presets:
- Perspective (free camera)
- Top (looking down Y axis)
- Bottom (looking up Y axis)
- Front (looking down Z axis)
- Back (looking up Z axis)
- Right (looking left along X axis)
- Left (looking right along X axis)

### Status Display

The right side of the toolbar displays current editor state:
- Current gizmo mode
- Current coordinate space

## Architecture

### ToolbarState

The `ToolbarState` struct maintains all toolbar-related state:

```rust
pub struct ToolbarState {
    pub gizmo_mode: GizmoMode,
    pub gizmo_space: GizmoSpace,
    pub snap_settings: SnapSettings,
    pub editor_mode: EditorMode,
    pub camera_preset: CameraPreset,
}
```

### SnapSettings

Configurable snap settings:

```rust
pub struct SnapSettings {
    pub enabled: bool,
    pub grid_size: f32,        // World units (default: 1.0)
    pub angle_increment: f32,   // Degrees (default: 15.0)
    pub scale_increment: f32,   // Scale factor (default: 0.1)
}
```

### ToolbarAction

Actions triggered by toolbar buttons:

```rust
pub enum ToolbarAction {
    SetGizmoTranslate,
    SetGizmoRotate,
    SetGizmoScale,
    ToggleGizmoSpace,
    ToggleSnapEnabled,
    Play,
    Pause,
    Stop,
    SetCameraPreset(CameraPreset),
}
```

### CameraPreset

Available camera view presets:

```rust
pub enum CameraPreset {
    Top,
    Bottom,
    Front,
    Back,
    Right,
    Left,
    Perspective,
}
```

## Usage

### Basic Integration

The toolbar is automatically integrated into `EditorState`:

```rust
use praxis_editor::EditorState;

let mut editor = EditorState::new();

// The toolbar is rendered as part of editor.ui()
editor.ui(&ctx, Some(&mut undo_system), Some(&mut world));
```

### Accessing Toolbar State

```rust
// Read toolbar state
let toolbar = editor.toolbar_state();
println!("Current gizmo mode: {:?}", toolbar.gizmo_mode);
println!("Snap enabled: {}", toolbar.snap_settings.enabled);

// Modify toolbar state
let toolbar = editor.toolbar_state_mut();
toolbar.snap_settings.grid_size = 2.0;
toolbar.gizmo_mode = GizmoMode::Rotate;
```

### Manual Rendering

You can also render the toolbar independently:

```rust
use praxis_editor::{render_toolbar, handle_toolbar_action, ToolbarState};

let mut toolbar_state = ToolbarState::new();

// Render and get actions
let actions = render_toolbar(&ctx, &mut toolbar_state);

// Handle actions
for action in actions {
    handle_toolbar_action(action, &mut toolbar_state);
}
```

### Syncing with GizmoSystem

To sync toolbar state with the gizmo system:

```rust
use praxis_editor::GizmoSystem;

// Get toolbar state
let toolbar = editor.toolbar_state();

// Update gizmo system
if let Some(mut gizmo_system) = world.get_resource_mut::<GizmoSystem>() {
    gizmo_system.set_mode(toolbar.gizmo_mode);
    gizmo_system.set_space(toolbar.gizmo_space);
}
```

### Implementing Camera Presets

Camera preset actions should be handled by the application:

```rust
use praxis_editor::{ToolbarAction, CameraPreset};
use praxis_math::{Vec3, Quat};

match action {
    ToolbarAction::SetCameraPreset(preset) => {
        let (position, rotation) = match preset {
            CameraPreset::Top => (
                Vec3::new(0.0, 10.0, 0.0),
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
            ),
            CameraPreset::Front => (
                Vec3::new(0.0, 0.0, 10.0),
                Quat::IDENTITY
            ),
            CameraPreset::Right => (
                Vec3::new(10.0, 0.0, 0.0),
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            ),
            CameraPreset::Perspective => {
                // Free camera - don't change position
                return;
            },
            // ... other presets
        };
        
        // Update camera transform
        if let Some(mut camera_transform) = query.get_single_mut().ok() {
            camera_transform.translation = position;
            camera_transform.rotation = rotation;
        }
    }
    _ => {}
}
```

## Customization

### Custom Snap Increments

```rust
let toolbar = editor.toolbar_state_mut();
toolbar.snap_settings.grid_size = 0.5;      // Finer grid
toolbar.snap_settings.angle_increment = 5.0; // Finer rotation
toolbar.snap_settings.scale_increment = 0.05; // Finer scaling
```

### Default Gizmo Settings

```rust
let toolbar = editor.toolbar_state_mut();
toolbar.gizmo_mode = GizmoMode::Rotate;
toolbar.gizmo_space = GizmoSpace::Local;
```

### Default Camera View

```rust
let toolbar = editor.toolbar_state_mut();
toolbar.camera_preset = CameraPreset::Top;
```

## Keyboard Shortcuts

While the toolbar provides mouse-based quick access, these keyboard shortcuts are suggested:

- **W**: Switch to Translate mode
- **E**: Switch to Rotate mode
- **R**: Switch to Scale mode
- **X**: Toggle World/Local space
- **Ctrl+\\**: Toggle snap settings
- **F5**: Play
- **F6**: Pause
- **F7**: Stop

Note: Keyboard shortcuts must be implemented separately in your input handling system.

## Visual Design

The toolbar uses egui's grouping system to visually separate different categories of controls:

1. **Gizmo Mode Group**: Three selectable buttons showing current mode
2. **Space Toggle Group**: Single button showing current space
3. **Snap Settings Group**: Toggle button with optional size display
4. **Playback Group**: Three buttons with enable/disable states
5. **Camera Group**: Dropdown menu and quick-access buttons

Each button includes:
- Unicode emoji icons for visual recognition
- Text labels for clarity
- Hover tooltips with descriptions and shortcuts
- Visual feedback for current state (selected/enabled/disabled)

## Integration with Editor Systems

### GizmoSystem

The toolbar provides UI for controlling the `GizmoSystem` resource:
- Mode selection directly maps to `GizmoMode`
- Space toggle directly maps to `GizmoSpace`
- Applications should sync toolbar state with gizmo system

### EditorMode

The toolbar controls play/pause/stop state through `EditorMode`:
- Play/Pause/Stop buttons modify the editor mode
- Button enable states reflect current mode
- Synced automatically in `EditorState::ui()`

### Camera System

Camera presets require application-specific implementation:
- Toolbar tracks selected preset
- Application must respond to `SetCameraPreset` actions
- Typically involves updating camera transform and projection

## Testing

The toolbar module includes comprehensive unit tests:

```bash
cargo test -p praxis_editor toolbar
```

Tests cover:
- Default state initialization
- Action handling and state updates
- Mode and space toggling
- Snap settings toggle
- Playback control state transitions
- Camera preset selection
- Return value semantics for gizmo updates
