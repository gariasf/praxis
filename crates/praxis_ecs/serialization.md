# ECS Serialization System

This document describes the ECS component serialization system implemented in `praxis_ecs`.

## Overview

The serialization system provides a flexible, type-safe way to save and load ECS world state to and from RON (Rusty Object Notation) format. It uses a registration-based approach that allows both built-in and custom components to be serialized.

## Architecture

### Core Components

1. **`SerializableComponent` Trait**
   - Defines the interface for serializable components
   - Methods: `serialize_component()`, `deserialize_component()`, `type_name()`
   - Uses RON format for individual component serialization

2. **`ComponentRegistry`**
   - Central registry mapping component types to serialization functions
   - Type-safe component registration via `register<T>()`
   - World-level serialization/deserialization

3. **`WorldSnapshot`**
   - Intermediate serializable representation of world state
   - Contains vector of `EntityData` 
   - Serializes to/from RON format

4. **`EntityData`**
   - Represents a single entity and its components
   - Maps component type names to serialized data strings
   - Includes entity ID for reference resolution

5. **`DeserializeContext`**
   - Maintains mapping from serialized entity IDs to runtime entities
   - Enables entity reference resolution (Parent/Children components)

## Features

### Entity Reference Resolution

The system handles components that reference other entities (like `Parent` and `Children`):

1. During serialization, entity IDs are stored as `u64` values
2. During deserialization:
   - All entities are spawned first
   - A mapping is built from serialized IDs to runtime entities
   - Component deserializers use this mapping to resolve references

### NoSave Marker

Entities with the `NoSave` component are automatically excluded from serialization. This is useful for:
- Temporary entities (particles, debug visualizations)
- Runtime-only state (cursor, gizmos)
- Engine-managed entities

### Type-Safe Registration

The registry uses Rust's type system to ensure:
- Component types match between serialization and deserialization
- No runtime type errors
- Compile-time verification of serializable components

## Built-in Implementations

The following components implement `SerializableComponent`:

- `Name` - Entity names
- `Transform` - Local transforms (position, rotation, scale)
- `GlobalTransform` - World-space transforms
- `Parent` - Parent entity reference
- `Children` - Child entity list
- `MeshHandle` - Mesh asset references
- `MaterialHandle` - Material asset references
- `Visibility` - Visibility state

## Usage Patterns

### Basic World Serialization

```rust
let mut world = World::new();
world.spawn((Name::new("Player"), Transform::default()));

let mut registry = ComponentRegistry::new();
registry.register_common_types();

let ron_string = world.serialize(&registry)?;
```

### Custom Component

```rust
#[derive(Component, Serialize, Deserialize, Clone)]
struct Health(f32);

impl SerializableComponent for Health {
    fn serialize_component(&self) -> Result<String> {
        Ok(ron::to_string(self)?)
    }

    fn deserialize_component(
        data: &str,
        _entity: Entity,
        _context: &DeserializeContext,
    ) -> Result<Box<dyn FnOnce(&mut EntityWorldMut)>>
    where
        Self: Sized + 'static,
    {
        let component: Health = ron::from_str(data)?;
        Ok(Box::new(move |entity_mut| {
            entity_mut.insert(component);
        }))
    }

    fn type_name() -> &'static str {
        "Health"
    }
}
```

### World Methods

Convenience methods are provided on `World`:

```rust
// Serialize
let ron_string = world.serialize(&registry)?;

// Deserialize
world.deserialize(&ron_string, &registry)?;
```

## Design Decisions

### Why RON?

RON (Rusty Object Notation) was chosen for:
- Human-readable format (easy debugging, version control)
- Native Rust data types
- Good serde integration
- Suitable for configuration and save files

### Closure-Based Deserialization

Component deserialization returns a closure instead of directly inserting. This allows:
- Deferred insertion after all entities are spawned
- Custom deserialization logic per component
- Entity reference resolution before insertion

### Type Name Registration

Each component provides a static type name string:
- Enables version control and migrations
- Allows renaming Rust types without breaking saves
- Clear debugging and error messages

## Performance Considerations

- Serialization is O(n) where n is the number of entities
- Entity reference resolution uses a HashMap (O(1) lookups)
- RON parsing is relatively fast but not zero-cost
- Consider binary formats (bincode, postcard) for performance-critical cases

## Future Enhancements

Potential improvements:
- Schema versioning and migration support
- Partial world serialization (entity subsets)
- Binary format option for faster serialization
- Component delta serialization for networking
- Prefab system built on serialization
- Scene format with metadata

## Testing

Comprehensive tests cover:
- Basic serialization/deserialization
- Parent-child relationships
- NoSave exclusion
- Custom component types
- Registry operations
- Error handling

Run tests with:
```bash
cargo test --package praxis_ecs serialization
```

## Examples

See `README.md` for usage examples and the `serialization` module documentation for API details.
