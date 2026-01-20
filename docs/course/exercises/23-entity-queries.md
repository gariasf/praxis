# Exercise 23: Entity Queries

**Difficulty**: 🟢 Beginner | **Estimated Time**: 2-3h | **Subsystem**: ECS

## Overview

Implement flexible entity queries to retrieve entities with specific component combinations. The core of data-oriented ECS iteration.

## Learning Objectives

- Understand ECS query patterns
- Learn efficient component iteration
- Implement query filters (With, Without)
- Handle mutable vs immutable access

## Requirements

### Functional Requirements

1. **Basic Queries**
   - Query single component type
   - Query multiple component types (tuples)
   - Iterate over matching entities

2. **Query Filters**
   - `With<T>`: Entity must have component T
   - `Without<T>`: Entity must NOT have component T
   - Combine multiple filters

3. **Access Modes**
   - Immutable access: `&Component`
   - Mutable access: `&mut Component`
   - Mixed access in same query

4. **Optional Components**
   - `Option<&Component>`: Include entities even if missing component

### Non-Functional Requirements

- **Performance**: Iterate 10,000 entities in < 1ms
- **Safety**: Enforce Rust borrow rules at compile time
- **Ergonomics**: Clean, intuitive API

## API Design

```rust
pub struct Query<'w, Q: QueryData> {
    world: &'w World,
    _marker: PhantomData<Q>,
}

pub trait QueryData {
    type Item<'a>;
    
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>>;
}

// Usage examples:
impl World {
    // Query single component
    pub fn query<T: Component>(&self) -> Query<&T>;
    pub fn query_mut<T: Component>(&mut self) -> Query<&mut T>;
    
    // Query multiple components
    pub fn query<T1, T2>(&self) -> Query<(&T1, &T2)>;
    
    // With filters
    pub fn query<T>(&self) -> Query<&T, With<OtherComponent>>;
    pub fn query<T>(&self) -> Query<&T, Without<OtherComponent>>;
}

// Example usage:
for (position, velocity) in world.query::<(&Position, &mut Velocity)>().iter() {
    position.x += velocity.dx;
}

for position in world.query::<&Position>().with::<Player>().iter() {
    // Only player positions
}
```

## Validation Criteria

### Correctness
- [ ] Returns all matching entities
- [ ] Filters work correctly (With, Without)
- [ ] Mutable queries enforce exclusive access
- [ ] Optional components handled properly

### Performance
- [ ] Iterate 10,000 entities in < 1ms
- [ ] Query setup overhead < 0.1ms
- [ ] Memory efficient iteration

## Test Cases

```rust
#[test]
fn test_single_component_query() {
    let mut world = World::new();
    
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 1.0, y: 2.0 });
    
    let e2 = world.spawn();
    world.add_component(e2, Position { x: 3.0, y: 4.0 });
    
    let mut count = 0;
    for pos in world.query::<&Position>().iter() {
        count += 1;
    }
    
    assert_eq!(count, 2);
}

#[test]
fn test_multi_component_query() {
    let mut world = World::new();
    
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 1.0, y: 2.0 });
    world.add_component(e1, Velocity { dx: 0.1, dy: 0.2 });
    
    let e2 = world.spawn();
    world.add_component(e2, Position { x: 3.0, y: 4.0 });
    // No velocity
    
    let mut count = 0;
    for (pos, vel) in world.query::<(&Position, &Velocity)>().iter() {
        count += 1;
    }
    
    assert_eq!(count, 1); // Only e1 has both
}

#[test]
fn test_with_filter() {
    let mut world = World::new();
    
    let player = world.spawn();
    world.add_component(player, Position { x: 0.0, y: 0.0 });
    world.add_component(player, Player);
    
    let enemy = world.spawn();
    world.add_component(enemy, Position { x: 1.0, y: 1.0 });
    
    let mut count = 0;
    for pos in world.query::<&Position>().with::<Player>().iter() {
        count += 1;
    }
    
    assert_eq!(count, 1); // Only player
}

#[test]
fn test_without_filter() {
    let mut world = World::new();
    
    let player = world.spawn();
    world.add_component(player, Position { x: 0.0, y: 0.0 });
    world.add_component(player, Player);
    
    let enemy = world.spawn();
    world.add_component(enemy, Position { x: 1.0, y: 1.0 });
    
    let mut count = 0;
    for pos in world.query::<&Position>().without::<Player>().iter() {
        count += 1;
    }
    
    assert_eq!(count, 1); // Only enemy
}

#[test]
fn test_mutable_query() {
    let mut world = World::new();
    
    let e = world.spawn();
    world.add_component(e, Position { x: 0.0, y: 0.0 });
    
    for mut pos in world.query::<&mut Position>().iter() {
        pos.x += 1.0;
    }
    
    let pos = world.get_component::<Position>(e).unwrap();
    assert_eq!(pos.x, 1.0);
}

#[test]
fn test_optional_component() {
    let mut world = World::new();
    
    let e1 = world.spawn();
    world.add_component(e1, Position { x: 0.0, y: 0.0 });
    world.add_component(e1, Velocity { dx: 1.0, dy: 0.0 });
    
    let e2 = world.spawn();
    world.add_component(e2, Position { x: 1.0, y: 1.0 });
    // No velocity
    
    let mut count = 0;
    for (pos, vel_opt) in world.query::<(&Position, Option<&Velocity>)>().iter() {
        count += 1;
        if let Some(vel) = vel_opt {
            // Has velocity
        }
    }
    
    assert_eq!(count, 2); // Both entities, even though one lacks velocity
}
```

## Performance Targets

| Operation | Target |
|-----------|--------|
| Query setup | < 0.1ms |
| Iterate 1,000 entities | < 0.1ms |
| Iterate 10,000 entities | < 1ms |
| Filter overhead | < 10% |

## Implementation Patterns

### Archetype Storage (Optimal)
Group entities by component combination:
```
Archetype [Position, Velocity]: [e1, e2, e5]
Archetype [Position]: [e3, e4]
Archetype [Position, Health]: [e6, e7]
```

Query can iterate archetypes directly, skipping incompatible ones.

### Sparse Set Storage (Simple)
Each component type has a sparse set:
```
Position: {e1, e2, e3, e4, e5, e6, e7}
Velocity: {e1, e2, e5}
```

Query intersects sets to find matching entities.

## Hints & Guidance

### Simple Implementation
```rust
impl World {
    pub fn query<T: Component>(&self) -> Vec<&T> {
        let mut results = Vec::new();
        
        for entity in &self.entities {
            if let Some(component) = self.get_component::<T>(*entity) {
                results.push(component);
            }
        }
        
        results
    }
}
```

### Optimized with Archetypes
```rust
impl World {
    pub fn query<T: Component>(&self) -> impl Iterator<Item = &T> {
        self.archetypes
            .iter()
            .filter(|arch| arch.has_component::<T>())
            .flat_map(|arch| arch.get_components::<T>())
    }
}
```

### Compile-time Safety
Use Rust's type system to enforce:
- Can't have two mutable references
- Can't mix immutable and mutable references to same component
- Query lifetime tied to World borrow

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::marker::PhantomData;
use std::collections::HashMap;
use std::any::TypeId;

pub struct World {
    entities: Vec<Entity>,
    components: HashMap<TypeId, Box<dyn std::any::Any>>,
}

pub type Entity = u32;

pub trait Component: 'static {}

// Simple query implementation
pub struct Query<'w, T> {
    world: &'w World,
    _marker: PhantomData<T>,
}

impl World {
    pub fn query<T: Component>(&self) -> Query<&T> {
        Query {
            world: self,
            _marker: PhantomData,
        }
    }
}

impl<'w, T: Component> Query<'w, T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.world.entities.iter().filter_map(|entity| {
            self.world.get_component::<T>(*entity)
        })
    }
}

// For production, see bevy_ecs Query implementation
```

</details>

## Related Resources

- [Bevy Query Documentation](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/struct.Query.html)
- [ECS Back and Forth - Query](https://skypjack.github.io/2019-03-07-ecs-baf-part-2/)
- [Praxis ECS Guide](../../reference/crates.md#praxis_ecs)

## Next Steps

- Implement system scheduling (Exercise 22)
- Add change detection (Exercise 29)
- Study `bevy_ecs` query implementation for advanced patterns
