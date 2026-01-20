# Exercise 21: Component Registration

**Difficulty**: 🟢 Beginner | **Estimated Time**: 1-2h | **Subsystem**: ECS

## Overview

Build a simple ECS component registry that allows defining and managing component types. Foundation for understanding entity-component systems.

## Learning Objectives

- Understand ECS component patterns
- Learn type registration and reflection
- Implement component storage
- Handle component lifecycle

## Requirements

### Functional Requirements

1. **Component Registration**
   - Register component types with the ECS
   - Assign unique component IDs
   - Store component metadata (name, size, alignment)

2. **Component Storage**
   - Add components to entities
   - Remove components from entities
   - Query if entity has component

3. **Type Safety**
   - Compile-time type checking
   - Runtime type validation

### Non-Functional Requirements

- **Performance**: O(1) component access by entity ID
- **Memory**: Packed component storage (no fragmentation)
- **Scalability**: Support 10,000+ entities

## API Design

```rust
#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

pub struct World {
    entities: Vec<Entity>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl World {
    pub fn new() -> Self;
    pub fn spawn(&mut self) -> Entity;
    pub fn despawn(&mut self, entity: Entity);
    
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T);
    pub fn remove_component<T: Component>(&mut self, entity: Entity);
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T>;
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T>;
}
```

## Validation Criteria

### Correctness
- [ ] Components correctly associated with entities
- [ ] Type-safe access (can't get wrong component type)
- [ ] Components removed when entity despawned
- [ ] No memory leaks

### Performance
- [ ] Add/remove component in O(1)
- [ ] Get component in O(1)
- [ ] Memory overhead < 10% of component data

## Test Cases

```rust
#[test]
fn test_add_and_get_component() {
    let mut world = World::new();
    let entity = world.spawn();
    
    world.add_component(entity, Position { x: 1.0, y: 2.0, z: 3.0 });
    
    let pos = world.get_component::<Position>(entity).unwrap();
    assert_eq!(pos.x, 1.0);
}

#[test]
fn test_remove_component() {
    let mut world = World::new();
    let entity = world.spawn();
    
    world.add_component(entity, Position { x: 1.0, y: 2.0, z: 3.0 });
    assert!(world.get_component::<Position>(entity).is_some());
    
    world.remove_component::<Position>(entity);
    assert!(world.get_component::<Position>(entity).is_none());
}

#[test]
fn test_multiple_components() {
    let mut world = World::new();
    let entity = world.spawn();
    
    world.add_component(entity, Position { x: 1.0, y: 2.0, z: 3.0 });
    world.add_component(entity, Velocity { dx: 0.1, dy: 0.2, dz: 0.3 });
    
    assert!(world.get_component::<Position>(entity).is_some());
    assert!(world.get_component::<Velocity>(entity).is_some());
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type Entity = u32;

pub trait Component: 'static + Send + Sync {}

trait ComponentStorage: Any {
    fn remove(&mut self, entity: Entity);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct TypedComponentStorage<T: Component> {
    components: HashMap<Entity, T>,
}

impl<T: Component> TypedComponentStorage<T> {
    fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
}

impl<T: Component> ComponentStorage for TypedComponentStorage<T> {
    fn remove(&mut self, entity: Entity) {
        self.components.remove(&entity);
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct World {
    next_entity_id: Entity,
    entities: Vec<Entity>,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            components: HashMap::new(),
        }
    }
    
    pub fn spawn(&mut self) -> Entity {
        let entity = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }
    
    pub fn despawn(&mut self, entity: Entity) {
        self.entities.retain(|&e| e != entity);
        
        // Remove all components for this entity
        for storage in self.components.values_mut() {
            storage.remove(entity);
        }
    }
    
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        
        let storage = self
            .components
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedComponentStorage::<T>::new()));
        
        let typed_storage = storage
            .as_any_mut()
            .downcast_mut::<TypedComponentStorage<T>>()
            .unwrap();
        
        typed_storage.components.insert(entity, component);
    }
    
    pub fn remove_component<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        
        if let Some(storage) = self.components.get_mut(&type_id) {
            storage.remove(entity);
        }
    }
    
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        
        self.components
            .get(&type_id)
            .and_then(|storage| {
                storage
                    .as_any()
                    .downcast_ref::<TypedComponentStorage<T>>()
            })
            .and_then(|typed_storage| typed_storage.components.get(&entity))
    }
    
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        
        self.components
            .get_mut(&type_id)
            .and_then(|storage| {
                storage
                    .as_any_mut()
                    .downcast_mut::<TypedComponentStorage<T>>()
            })
            .and_then(|typed_storage| typed_storage.components.get_mut(&entity))
    }
    
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.get_component::<T>(entity).is_some()
    }
}

// Example components
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Component for Position {}

#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl Component for Velocity {}

// Example usage
fn main() {
    let mut world = World::new();
    
    let player = world.spawn();
    world.add_component(player, Position { x: 0.0, y: 0.0, z: 0.0 });
    world.add_component(player, Velocity { dx: 1.0, dy: 0.0, dz: 0.0 });
    
    if let Some(pos) = world.get_component::<Position>(player) {
        println!("Player position: {:?}", pos);
    }
}
```

</details>

## Related Resources

- [Bevy ECS Overview](https://bevyengine.org/learn/book/getting-started/ecs/)
- [ECS FAQ](https://github.com/SanderMertens/ecs-faq)
- [Praxis ECS Documentation](../../reference/crates.md#praxis_ecs)

## Next Steps

- Implement queries (Exercise 23)
- Add system scheduling (Exercise 22)
- Study `bevy_ecs` component implementation
