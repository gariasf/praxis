# Comprehensive Scene Demo Documentation

This document provides detailed information about the `comprehensive_scene_demo` example, which demonstrates the complete asset loading pipeline from disk to screen in the Praxis engine.

## Overview

The comprehensive scene demo is the most feature-complete example in Praxis, showcasing:

1. **Asset Loading Pipeline** - Complete workflow from disk to GPU
2. **OBJ Mesh Loading** - Using praxis_assets to load 3D models
3. **Procedural Textures** - Runtime texture generation with various patterns
4. **ECS Scene Management** - Entity-based scene organization
5. **Camera System** - FPS-style camera with full navigation
6. **Input Integration** - Action-mapped controls with mouse look

## Architecture

### Component-Based Scene

The demo uses an ECS (Entity Component System) architecture where each object in the scene is an entity with components:

```rust
world.spawn((
    Transform::from_xyz(-3.0, 1.0, 0.0),    // Position
    MeshHandle::new("textured_cube"),        // Mesh reference
    TextureHandle::new("brick"),             // Texture reference
));
```

This approach provides:
- **Flexibility** - Easy to add/remove components
- **Performance** - Cache-friendly data layout
- **Maintainability** - Clear separation of concerns

### Asset Management

#### Mesh Loading

The demo loads OBJ files using the asset system:

```rust
load_obj_mesh(
    render_context.mesh_manager_mut(),
    "cube_obj",
    "assets/models/cube.obj",
)?;
```

If the OBJ file doesn't exist, it falls back to procedural geometry:

```rust
render_context
    .mesh_manager_mut()
    .load_mesh("cube_obj", praxis_graphics::colored_cube_mesh())?;
```

#### Texture Generation

The demo creates procedural textures at runtime using pixel functions:

```rust
Self::create_procedural_texture(
    render_context.texture_manager_mut(),
    "checker",
    64, 64,
    |x, y| {
        let checker_size = 8;
        let is_white = ((x / checker_size) + (y / checker_size)) % 2 == 0;
        if is_white {
            [220, 220, 220, 255]  // RGBA
        } else {
            [80, 80, 80, 255]
        }
    },
)?;
```

Four textures are generated:
- **Checker** - Classic checkerboard pattern
- **Brick** - Brick wall with mortar
- **Metal** - Noisy metallic surface
- **Wood** - Wood grain effect

### Camera System

The demo implements an FPS-style camera using the ECS camera system:

```rust
world.spawn(PerspectiveCameraBundle::new(
    Vec3::new(0.0, 2.0, 8.0),  // Position
    70.0_f32.to_radians(),      // FOV
    WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,  // Aspect ratio
));
```

The camera controller updates camera transform based on input:

```rust
fn fps_camera_movement_system(
    input_state: Res<InputState>,
    input_map: Res<InputMap>,
    mut controller: ResMut<CameraController>,
    mut cameras: Query<(&Camera, &mut Transform), With<PerspectiveProjection>>,
) {
    // Update camera position and rotation based on input
}
```

### Input Handling

The demo uses an action mapping system for flexible controls:

```rust
let mut input_map = InputMap::default();
input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
// ... more bindings
```

This allows for easy remapping and supports multiple input devices.

### Rendering Pipeline

The rendering process follows these steps:

1. **Query Scene Objects** - Get all entities with Transform, MeshHandle, and TextureHandle
2. **Build Draw Commands** - Create a list of objects to render with their matrices and textures
3. **Submit to Renderer** - Pass commands to RenderContext for GPU execution

```rust
let mut draw_commands = Vec::new();

for (transform, mesh_handle, texture_handle) in renderables.iter() {
    draw_commands.push(DrawCommandWithTexture {
        mesh_id: mesh_handle.id.clone(),
        model: transform.compute_matrix(),
        texture_name: Some(texture_handle.id.clone()),
    });
}

render_context.render_textured(&TexturedRenderCommands {
    view: matrices.view,
    proj: matrices.projection,
    draw_commands: &draw_commands,
})?;
```

## Scene Layout

The demo creates a simple scene with 7 objects:

```
Floor (checker texture) at Y=0
Row 1 (Z=0):
  - Textured cube (brick) at X=-3
  - Textured cube (metal) at X=0
  - Textured cube (wood) at X=3

Row 2 (Z=-5):
  - OBJ cube (brick) at X=-3
  - OBJ cube (metal) at X=0
  - OBJ cube (wood) at X=3
```

The camera starts at position (0, 2, 8) looking toward the scene.

## Controls

### Movement
- **W** - Move forward
- **S** - Move backward
- **A** - Strafe left
- **D** - Strafe right
- **Space** - Move up
- **Left Ctrl** - Move down
- **Left Shift** - Sprint (hold for faster movement)

### Camera
- **Mouse** - Look around (when cursor is locked)
- **ESC** - Toggle cursor lock / Exit (when unlocked)

## Key Systems Integration

### 1. praxis_assets
- Provides the `AssetLoader` trait for generic asset loading
- `MeshLoader` implementation for OBJ files
- Integration with `MeshAssetManager` for GPU upload

### 2. praxis_graphics
- `RenderContext` manages Vulkan resources
- `MeshAssetManager` stores loaded meshes
- `TextureManager` manages texture cache
- `render_textured()` renders objects with custom textures

### 3. praxis_ecs
- `World` stores all entities and components
- `Schedule` runs systems each frame
- `Transform` component for positioning
- `MeshHandle` and `TextureHandle` for asset references
- Camera components for view/projection matrices

### 4. praxis_input
- `InputState` tracks keyboard/mouse state
- `InputMap` maps physical inputs to logical actions
- Integration with winit event processing

### 5. praxis_math
- `Vec3` for positions and directions
- `Quat` for rotations
- `Mat4` for transformation matrices

## Event Loop Architecture

The example uses winit's `ApplicationHandler` trait:

```rust
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize window, graphics, and scene
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Handle window events (resize, input, redraw)
    }

    fn device_event(&mut self, ..., event: DeviceEvent) {
        // Handle raw device events (mouse motion for camera)
    }
}
```

This provides:
- Clean separation of initialization and runtime logic
- Proper event handling for all window events
- Device event handling for raw mouse input

## Performance Considerations

### Asset Loading
- Assets are loaded once during initialization
- Meshes are uploaded to GPU and cached
- Textures are procedurally generated once and reused

### Rendering
- Uses indexed drawing for efficient GPU usage
- Batches objects by pipeline (all use the same shader)
- Minimizes state changes between draw calls

### ECS Queries
- Queries are compiled once and reused each frame
- Component iteration is cache-friendly
- Systems run in parallel when possible

## Extending the Demo

### Adding New Objects

```rust
world.spawn((
    Transform::from_xyz(x, y, z),
    MeshHandle::new("your_mesh"),
    TextureHandle::new("your_texture"),
));
```

### Creating New Textures

```rust
Self::create_procedural_texture(
    texture_manager,
    "my_texture",
    width, height,
    |x, y| {
        // Return [r, g, b, a] based on x, y
        [r, g, b, 255]
    },
)?;
```

### Loading Custom OBJ Files

Place your OBJ file in `assets/models/` and load it:

```rust
load_obj_mesh(
    render_context.mesh_manager_mut(),
    "my_model",
    "assets/models/my_model.obj",
)?;
```

### Adding New Controls

```rust
input_map.bind_key(&Action::new("my_action"), KeyCode::KeyF);

// In your system:
if input_map.is_action_pressed(&Action::new("my_action"), &input_state) {
    // Do something
}
```

## Troubleshooting

### OBJ File Not Loading
- Ensure the file exists at `assets/models/cube.obj`
- Check that the OBJ file is triangulated
- The demo will fall back to procedural geometry if loading fails

### Window Not Appearing
- Check that Vulkan drivers are installed
- Verify your GPU supports Vulkan 1.0 or higher
- Check console output for initialization errors

### Mouse Look Not Working
- Press ESC to lock the cursor
- Ensure the window has focus
- Check that device events are being processed

### Performance Issues
- Try running in release mode: `cargo run --example comprehensive_scene_demo --release`
- Check GPU driver updates
- Reduce window resolution

## Related Examples

- **obj_loader_demo** - Focused example of OBJ loading
- **fps_camera_controller** - Camera system without scene rendering
- **multi_mesh_demo** - Multiple meshes without textures
- **input_integration** - Input system integration

## Code Structure

```
comprehensive_scene_demo.rs
├── CameraController (Resource)
│   ├── Movement parameters
│   └── Mouse sensitivity
├── App (ApplicationHandler)
│   ├── setup_scene() - Initialize graphics and scene
│   ├── load_assets() - Load meshes and create textures
│   ├── create_procedural_texture() - Generate textures
│   ├── setup_input_bindings() - Configure controls
│   ├── spawn_scene_objects() - Create scene entities
│   ├── render_scene() - Execute rendering
│   └── cursor management
└── fps_camera_movement_system()
    └── Update camera based on input
```

## Conclusion

The comprehensive scene demo demonstrates the complete workflow for creating a 3D scene in Praxis, from loading assets to rendering with a controllable camera. It serves as a reference implementation for integrating all major engine systems and provides a solid foundation for building more complex applications.
