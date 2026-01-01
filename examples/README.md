# Praxis Examples

This directory contains examples demonstrating how to use the Praxis game engine.

## Available Examples

### Playground

A basic example that demonstrates the Praxis engine initialization and window creation.

To run:

```bash
cargo run --example playground
```

### ECS Demo

Demonstrates the core ECS functionality including:
- Creating a world and spawning entities
- Using built-in components (Transform, Name, etc.)
- Setting up parent-child relationships
- Querying entities and their components

To run:

```bash
cargo run --example ecs_demo
```

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

### Mesh Demo

A documentation example showing the mesh system architecture including:
- Mesh and MeshHandle components in the ECS
- MeshData and MeshAssetManager in the graphics system
- Per-mesh vertex/index buffer management
- Available primitive mesh functions

To run:

```bash
cargo run --example mesh_demo
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

## Building Examples

To build all examples:

```bash
cargo build --examples
```

To build a specific example in release mode:

```bash
cargo build --example transform_propagation_demo --release
```
