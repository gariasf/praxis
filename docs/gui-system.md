# GUI System

The Praxis GUI system provides debug tools, entity inspection, and runtime scene editing capabilities using egui.

## Architecture

The GUI system is built on three layers:

1. **egui Integration** (`EguiIntegration`): Low-level integration with Vulkan rendering
2. **GUI Components**: Individual UI tools (DebugUi, EntityInspector, TransformGizmos)
3. **GUI State** (`GuiState`): High-level coordinator that manages all components

## Components

### Debug UI

The Debug UI provides real-time performance monitoring with:

- **FPS Counter**: Persistent overlay showing current frames per second
- **Performance Window**: Detailed metrics including:
  - Frame time (with color-coded warnings)
  - Delta time
  - Frame count
  - Total runtime

The FPS counter uses color coding:
- Green: < 16.6ms (60+ FPS)
- Yellow: 16.6-33ms (30-60 FPS)
- Red: > 33ms (< 30 FPS)

### Entity Inspector

The Entity Inspector allows runtime inspection and editing of ECS entities:

**Features:**
- Entity list with search/filter
- Component viewing and editing for:
  - Transform (translation, rotation, scale)
  - Global Transform (read-only)
  - Name
  - Mesh references
  - Camera settings
  - Light properties
  - Hierarchy (parent/children)

**Editing:**
- Translation: Drag values with 0.1 unit increments
- Rotation: Euler angles in degrees with 1-degree increments
- Scale: Drag values with 0.01 unit increments
- All changes apply immediately to the ECS world

### Transform Gizmos

Transform Gizmos provide interactive scene editing tools:

**Modes:**
- **Translate**: Move entities along X, Y, Z axes
- **Rotate**: Rotate entities around axes
- **Scale**: Resize entities uniformly or per-axis

**Operations:**
- `add_gizmo(entity)`: Attach a gizmo to an entity
- `remove_gizmo(entity)`: Remove gizmo from entity
- `set_mode(mode)`: Change operation mode
- `cycle_mode()`: Rotate through modes (T → R → S → T)
- `apply_translation/rotation/scale()`: Apply transformations

## Integration

### Basic Setup

```rust
use praxis_gui::GuiState;
use praxis_ecs::World;

// During initialization (in resumed() or similar)
let gui_state = GuiState::new(
    event_loop,
    window.clone(),
    graphics_queue.clone(),
    swapchain_format,
);

// In event handler
if gui_state.handle_event(&window, &event) {
    // Event was consumed by GUI, don't process further
    return;
}

// In render loop
gui_state.render(
    &window,
    &mut world,
    swapchain_image_view.clone(),
    render_pass.clone(),
)?;
```

### Event Handling

The GUI consumes input events when interacting with UI elements. Always check the return value of `handle_event()` to avoid input leaking through to game systems:

```rust
match event {
    WindowEvent::KeyboardInput { .. } | 
    WindowEvent::MouseInput { .. } |
    WindowEvent::CursorMoved { .. } => {
        if gui_state.handle_event(&window, &event) {
            return; // GUI consumed the event
        }
        // Process game input
    }
    _ => {}
}
```

### Render Pass Integration

The GUI renders after the main 3D scene within the same render pass:

```rust
// 1. Begin render pass
// 2. Render 3D scene
// 3. Render GUI (overlays on top)
gui_state.render(...)?;
// 4. End render pass
```

## Keyboard Shortcuts (Recommended)

While not enforced by the library, these shortcuts are commonly implemented:

- `F1`: Toggle debug UI visibility
- `F2`: Toggle entity inspector
- `F3`: Toggle transform gizmos
- `T`: Switch to translate mode
- `R`: Switch to rotate mode
- `S`: Switch to scale mode
- `G`: Add gizmo to selected entity
- `Delete`: Remove gizmo from selected entity

Implementation example:

```rust
WindowEvent::KeyboardInput {
    event: KeyEvent {
        logical_key: Key::Named(NamedKey::F1),
        state: ElementState::Pressed,
        ..
    },
    ..
} => {
    gui_state.debug_ui.toggle();
}
```

## Performance Considerations

### Overhead

The GUI system is designed for development/debugging use:

- **Minimal when hidden**: < 0.1ms when all components are hidden
- **Moderate when visible**: 1-3ms depending on entity count and UI complexity
- **Entity Inspector**: O(n) where n = visible entities in list
- **Transform Gizmos**: O(m) where m = active gizmos

### Optimization Tips

1. **Hide unused components**: Set `visible = false` on unused GUI components
2. **Limit entity list**: Use search filter in entity inspector to reduce displayed entities
3. **Minimize active gizmos**: Only attach gizmos to entities actively being edited
4. **Avoid every-frame queries**: Cache entity lookups when possible

### Memory Usage

- **Base overhead**: ~2MB (egui context, fonts, textures)
- **Per-frame**: ~100KB for typical UI (temporary allocations)
- **Texture cache**: ~500KB (egui internal texture atlas)

## Thread Safety

The GUI system is single-threaded and must be accessed from the main render thread. The `World` reference passed to `render()` allows safe component mutation during UI interaction.

## Future Improvements

Planned enhancements:

- Visual gizmo handles in 3D viewport
- Component adding/removing via inspector
- Scene hierarchy tree view
- Material editor
- Asset browser
- Console/logging window
- Profiler visualization
- Network stats (when multiplayer added)

## Dependencies

- `egui` 0.29: Immediate mode GUI framework
- `egui-winit` 0.29: Winit integration for input
- `egui_vulkano` 0.6: Vulkan rendering backend
- `vulkano` 0.34: Vulkan abstraction layer
- `winit` 0.30: Window management

## Examples

See `examples/gui_demo.rs` for a complete integration example.
