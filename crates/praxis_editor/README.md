# Praxis Editor

The editor system for the Praxis game engine, providing a comprehensive suite of tools for creating and managing game content.

## Features

### Viewport Panel

The `ViewportPanel` provides a 3D scene viewport with full camera controls:

- **Offscreen Rendering**: Renders to a framebuffer that can be displayed in egui
- **Orbit Camera**: Right-click + drag to orbit around a target point
- **Pan Camera**: Middle-click + drag to pan the camera
- **Zoom**: Mouse wheel to zoom in/out
- **Keyboard Movement**: WASD/QE to move the camera target
- **Grid Floor**: Visual reference grid with axis indicators (X=red, Z=blue)
- **Camera Information**: Real-time display of camera parameters

See [VIEWPORT_PANEL.md](VIEWPORT_PANEL.md) for detailed documentation.

### Editor Panels

The editor uses a dockable panel system powered by `egui_dock`:

- **ViewportPanel**: 3D scene viewport with camera controls
- **SceneViewPanel**: Scene visualization
- **HierarchyPanel**: Entity hierarchy tree view
- **InspectorPanel**: Component editing for selected entities
- **ConsolePanel**: Log output and command execution
- **AssetsPanel**: Project asset browser

### Selection System

Comprehensive entity selection with:
- Multi-entity selection (add/remove/toggle modes)
- Click-to-select in viewport
- Marquee selection
- Keyboard shortcuts (Ctrl+A, Ctrl+D)
- Selection events for UI updates

See `SELECTION_SYSTEM.md` for detailed documentation.

## Usage

### Basic Setup

```rust
use praxis_editor::{EditorState, ViewportPanel};
use praxis_graphics::RenderContext;

// Initialize the editor system
praxis_editor::init()?;

// Create editor state
let mut editor = EditorState::new();

// Create and initialize viewport
let mut viewport = ViewportPanel::new();
viewport.initialize(&mut render_context)?;
```

### Camera Controls

The viewport uses an orbit camera system:

```rust
// Programmatic camera control
viewport.set_camera_distance(15.0);
viewport.set_camera_target(Vec3::new(0.0, 0.0, 0.0));
viewport.reset_camera();

// Get camera transform for rendering
let camera_transform = viewport.compute_camera_transform();
let view_matrix = camera_transform.compute_inverse_matrix();
```

### Grid Rendering

The viewport includes a grid floor for spatial reference:

```rust
// Toggle grid visibility
viewport.set_show_grid(true);

// Grid features:
// - 50x50 unit grid by default
// - Center lines highlighted
// - X-axis in red, Z-axis in blue
```

## Architecture

### Panel System

All editor panels implement the `EditorPanel` trait:

```rust
pub trait EditorPanel {
    fn title(&self) -> &str;
    fn ui(&mut self, ui: &mut Ui);
    fn on_close(&mut self) {}
}
```

### Viewport Rendering Pipeline

```text
┌─────────────────────┐
│  ViewportPanel      │
│  - Camera state     │
│  - Grid renderer    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Offscreen Target   │
│  - Framebuffer      │
│  - Render pass      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Vulkan Image       │
│  (RGBA texture)     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  egui::Image        │
│  (Display in UI)    │
└─────────────────────┘
```

### Camera System

The viewport uses spherical coordinates for orbit control:

- **Distance**: Radius from target point (1.0 to 1000.0)
- **Pitch**: Vertical angle (±89 degrees to avoid gimbal lock)
- **Yaw**: Horizontal angle (unlimited rotation)
- **Target**: Center point of orbit

Camera position computation:
```rust
position = target + Vec3(
    distance * cos(pitch) * sin(yaw),
    distance * sin(pitch),
    distance * cos(pitch) * cos(yaw)
)
```

## Dependencies

- `praxis_ecs`: Entity-component system
- `praxis_graphics`: Rendering and graphics APIs
- `praxis_math`: Math utilities (Vec3, Mat4, etc.)
- `praxis_input`: Input handling
- `praxis_utils`: Utilities and error handling
- `egui`: Immediate mode GUI
- `egui_dock`: Dockable panel system
- `vulkano`: Vulkan bindings

## Testing

Run tests for the editor:

```bash
cargo test -p praxis_editor
```

Tests cover:
- Viewport panel creation and configuration
- Camera transform calculations
- Grid mesh generation
- Panel interface compliance

## Examples

### Creating a Viewport

```rust
use praxis_editor::ViewportPanel;
use praxis_graphics::RenderContext;

let mut viewport = ViewportPanel::new();
viewport.initialize(&mut render_context)?;

// Use in your editor loop
// viewport.ui(&mut egui_ui);
```

### Custom Camera Position

```rust
use praxis_editor::ViewportPanel;
use praxis_math::Vec3;

let mut viewport = ViewportPanel::new();
viewport.initialize(&mut render_context)?;

// Position camera to look at a specific object
viewport.set_camera_target(Vec3::new(5.0, 2.0, 3.0));
viewport.set_camera_distance(15.0);
```

### Integrating with ECS

```rust
use praxis_editor::ViewportPanel;
use praxis_ecs::{World, Camera, Transform, PerspectiveProjection};

let mut world = World::new();
let mut viewport = ViewportPanel::new();

// Create a camera entity for the viewport
let camera_entity = world.spawn((
    Camera::new(),
    PerspectiveProjection::default(),
    Transform::default(),
));

viewport.set_camera_entity(camera_entity);

// Update camera transform each frame
let camera_transform = viewport.compute_camera_transform();
// Apply to camera entity...
```

## Performance Considerations

### Offscreen Rendering

Each viewport requires GPU memory for its render target:
- 800x600 RGBA: ~1.8 MB
- 1920x1080 RGBA: ~8.3 MB

### Grid Rendering

The default 50x50 grid generates:
- ~200 lines (2 vertices each)
- ~400 vertices total
- Minimal GPU cost

### Input Handling

Input events are filtered to viewport bounds, preventing unnecessary processing when the mouse is outside the viewport.

## Known Limitations

1. **Texture Display**: Rendered texture not yet displayed in egui (requires additional integration)
2. **Scene Rendering**: Scene entities not yet queried and rendered (requires ECS query system)
3. **Lighting**: Scene lighting not yet integrated with viewport rendering
4. **Multiple Viewports**: Architecturally supported but not fully tested

## Future Enhancements

- [ ] Display rendered Vulkan texture in egui
- [ ] Query and render scene entities in viewport
- [ ] Integrate scene lighting in viewport
- [ ] Add gizmos for object manipulation
- [ ] Add viewport-specific rendering settings
- [ ] Add camera preset positions
- [ ] Add viewport statistics (FPS, triangle count)
- [ ] Add screenshot functionality
- [ ] Add grid customization UI

## Documentation

- [VIEWPORT_PANEL.md](VIEWPORT_PANEL.md) - Detailed viewport panel documentation
- [SELECTION_SYSTEM.md](SELECTION_SYSTEM.md) - Selection system documentation
- [lib.rs](src/lib.rs) - API documentation

## License

MIT
