# Praxis Examples

This directory contains examples demonstrating how to use the Praxis game engine.

## Available Examples

### ECS Integration

Shows how the ECS integrates with the rest of the Praxis engine systems.

To run:

```bash
cargo run --example ecs_integration
```

### Transform Propagation Demo

Comprehensive demonstration of the transform propagation system, showing:
- Automatic GlobalTransform updates from local Transform
- Parent-child hierarchy management with Parent and Children components
- Transform changes and propagation through hierarchies
- Reparenting entities
- Adding/removing entities from the hierarchy
- Transform changes with rotation and scale

This example is particularly useful for understanding how the transform system works
and how to properly set up entity hierarchies for scene graphs.

To run:

```bash
cargo run --example transform_propagation_demo
```

### Multi-Mesh Demo

A fully functional example that renders multiple different mesh types in a single scene:
- Loading meshes into the mesh asset manager
- Rendering cubes, pyramids, and quads with different transforms
- Using DrawCommands to specify mesh and transform per object
- Multiple rotating objects demonstrating the mesh system

To run:

```bash
cargo run --example multi_mesh_demo
```

### Input Integration

Shows how to integrate the input system with winit event loops and ECS.

To run:

```bash
cargo run --example input_integration
```

### FPS Camera Controller

A fully functional FPS-style camera controller showing:
- WASD movement with camera
- Mouse look with sensitivity control
- Sprint mode
- Mouse cursor locking
- Integration of input and camera systems

To run:

```bash
cargo run --example fps_camera_controller
```

### OBJ Loader Demo

Demonstrates loading OBJ mesh files using the asset system:
- Using the AssetLoader trait
- Loading OBJ files from disk
- Uploading meshes to GPU
- Rendering loaded meshes

To run:

```bash
cargo run --example obj_loader_demo
```

### Scene Demo

Demonstrates the scene management system including:
- Loading scenes from RON files
- Spawning scene entities into the world
- Scene graph traversal
- Finding entities by name
- Unloading scenes
- Creating and saving scenes programmatically

To run:

```bash
cargo run --example scene_demo
```

### Comprehensive Scene Demo

**★ Complete asset pipeline demonstration ★**

This is the most comprehensive example showing the complete asset loading pipeline from disk to screen:
- Loading OBJ mesh files from disk using praxis_assets
- Procedural texture generation with various patterns (checker, brick, metal, wood)
- ECS-based scene management with multiple objects
- FPS camera controller with full navigation
- Multiple textured objects in the scene
- Integration of all major systems (ECS, graphics, input, camera, assets)

Controls:
- WASD - Move camera horizontally
- Space/Left Ctrl - Move camera vertically
- Left Shift - Sprint mode
- Mouse - Look around (when cursor is locked)
- ESC - Toggle cursor lock / Exit

To run:

```bash
cargo run --example comprehensive_scene_demo
```

### GUI Demo

A basic example demonstrating GUI functionality.

To run:

```bash
cargo run --example gui_demo
```

### Skybox Demo

Demonstrates skybox rendering with cubemap textures. Shows how to:
- Load a cubemap texture (from 6 faces or equirectangular image)
- Create a skybox renderer with reversed depth
- Render a skybox that always appears at infinite distance
- First-person camera controls for viewing the skybox

The skybox uses specialized rendering techniques:
- Reversed depth testing to ensure it renders behind all geometry
- Camera-centered transform (no translation, only rotation)
- Cubemap texture sampling for seamless panoramic views

To run:

```bash
cargo run --example skybox_demo
```

Controls:
- WASD - Move camera
- Mouse - Look around
- ESC - Exit

## Building Examples

To build all examples:

```bash
cargo build --examples
```

To build a specific example in release mode:

```bash
cargo build --example transform_propagation_demo --release
```
