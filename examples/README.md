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

## Building Examples

To build all examples:

```bash
cargo build --examples
```

To build a specific example in release mode:

```bash
cargo build --example transform_propagation_demo --release
```
