# Praxis ECS

Entity Component System for the Praxis game engine, built on top of `bevy_ecs`.

## Features

- **Transform Hierarchy**: Automatic transform propagation with parent-child relationships
- **Camera System**: Perspective and orthographic cameras with automatic matrix computation
- **Lighting System**: Directional and point lights with shadow support
- **Serialization**: Save and load world state to RON format
- **Component Registry**: Flexible registration system for custom serialization
- **Common Components**: Transform, Name, MeshHandle, MaterialHandle, and more

## Serialization

The ECS provides a comprehensive serialization system for saving and loading game state:

### Basic Usage

```rust
use praxis_ecs::{World, ComponentRegistry, Transform, Name};

// Create and populate a world
let mut world = World::new();
world.spawn((
    Name::new("Player"),
    Transform::from_xyz(10.0, 0.0, 5.0),
));

// Create a registry and register components
let mut registry = ComponentRegistry::new();
registry.register_common_types(); // Registers built-in types

// Serialize to RON
let ron_string = world.serialize(&registry).unwrap();
println!("Saved world: {}", ron_string);

// Deserialize into a new world
let mut new_world = World::new();
new_world.deserialize(&ron_string, &registry).unwrap();
```

### Custom Component Serialization

Implement `SerializableComponent` for your own types:

```rust
use praxis_ecs::{Component, SerializableComponent, DeserializeContext};
use serde::{Serialize, Deserialize};
use bevy_ecs::entity::Entity;
use praxis_utils::Result;

#[derive(Component, Serialize, Deserialize, Clone)]
struct Health {
    current: f32,
    max: f32,
}

impl SerializableComponent for Health {
    fn serialize_component(&self) -> Result<String> {
        Ok(ron::to_string(self)?)
    }

    fn deserialize_component(
        data: &str,
        _entity: Entity,
        _context: &DeserializeContext,
    ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
    where
        Self: Sized + 'static,
    {
        let component: Health = ron::from_str(data)?;
        Ok(Box::new(move |entity_mut| {
            entity_mut.insert(component);
        }))
    }

    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "Health"
    }
}

// Register your custom type
let mut registry = ComponentRegistry::new();
registry.register::<Health>();
```

### Built-in Serializable Components

The following components implement `SerializableComponent` out of the box:

- `Name` - Entity names for debugging
- `Transform` - Local position, rotation, and scale
- `GlobalTransform` - Computed world-space transform
- `Parent` - Parent entity reference (maintains relationships)
- `Children` - Child entity list (maintains relationships)
- `MeshHandle` - Reference to mesh assets
- `MaterialHandle` - Reference to material assets
- `Visibility` - Visibility state

### NoSave Marker

Use the `NoSave` component to exclude entities from serialization:

```rust
use praxis_ecs::{World, NoSave, Transform};

let mut world = World::new();

// This entity will be saved
world.spawn(Transform::default());

// This entity will NOT be saved
world.spawn((
    Transform::default(),
    NoSave,
));
```

### Parent-Child Relationships

The serialization system correctly handles entity references, maintaining parent-child relationships:

```rust
use praxis_ecs::{World, ComponentRegistry, Transform, Parent, Children};

let mut world = World::new();

let parent = world.spawn(Transform::default());
let child = world.spawn((
    Transform::from_xyz(1.0, 0.0, 0.0),
    Parent(parent),
));

let mut registry = ComponentRegistry::new();
registry.register::<Transform>();
registry.register::<Parent>();
registry.register::<Children>();

// Serialize and deserialize - relationships are preserved
let ron_string = world.serialize(&registry).unwrap();
let mut new_world = World::new();
new_world.deserialize(&ron_string, &registry).unwrap();
```

## Transform System

See `TRANSFORM_PROPAGATION.md` for details on the transform hierarchy system.

## License

MIT
