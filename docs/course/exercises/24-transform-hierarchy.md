# Exercise 24: Transform Hierarchy

**Difficulty**: 🔴 Advanced | **Estimated Time**: 5-6h | **Subsystem**: ECS

## Overview

Implement parent-child transform relationships where child transforms are relative to their parent. Essential for scene graphs, skeletal animation, and hierarchical object systems.

## Learning Objectives

- Understand local vs world space transformations
- Implement transform propagation algorithms
- Handle hierarchy mutations efficiently
- Learn dirty flagging optimization

## Requirements

### Functional Requirements

1. **Transform Components**
   - `Transform`: Local position, rotation, scale
   - `GlobalTransform`: Computed world-space transform
   - `Parent` and `Children` components for hierarchy

2. **Transform Propagation**
   - Compute world transforms from parent chain
   - Update only dirty transforms (changed locally or parent changed)
   - Handle arbitrary depth hierarchies

3. **Hierarchy Management**
   - Add child to parent
   - Remove child from parent
   - Reparent entity
   - Maintain consistency

### Non-Functional Requirements

- **Performance**: Update 10,000 transforms in < 5ms
- **Correctness**: Matrix math must be precise
- **Memory**: O(n) memory for n entities

## API Design

```rust
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Component)]
pub struct GlobalTransform {
    matrix: Mat4,
}

#[derive(Component)]
pub struct Parent(Entity);

#[derive(Component)]
pub struct Children(Vec<Entity>);

pub fn transform_propagation_system(
    mut query: Query<(&Transform, &mut GlobalTransform, Option<&Parent>)>,
    parent_query: Query<&GlobalTransform>,
) {
    // Update global transforms based on local transform and parent
}
```

## Validation Criteria

### Correctness
- [ ] Child moves with parent
- [ ] Multiple hierarchy levels work correctly
- [ ] Rotation inheritance correct (quaternion multiplication)
- [ ] Scale inheritance correct (component-wise multiplication)
- [ ] Handles cycles gracefully (error or prevention)

### Performance
- [ ] 10,000 entities updated in < 5ms
- [ ] Only dirty transforms recomputed
- [ ] No unnecessary allocations

## Test Cases

```rust
#[test]
fn test_basic_hierarchy() {
    let mut world = World::new();
    
    let parent = world.spawn();
    world.add_component(parent, Transform {
        translation: Vec3::new(10.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    
    let child = world.spawn();
    world.add_component(child, Transform {
        translation: Vec3::new(5.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child, Parent(parent));
    
    transform_propagation_system(&mut world);
    
    let child_global = world.get_component::<GlobalTransform>(child).unwrap();
    let pos = child_global.translation();
    
    assert!((pos.x - 15.0).abs() < 0.001); // 10 + 5 = 15
}

#[test]
fn test_rotation_inheritance() {
    let mut world = World::new();
    
    let parent = world.spawn();
    world.add_component(parent, Transform {
        translation: Vec3::ZERO,
        rotation: Quat::from_rotation_y(PI / 2.0), // 90 degrees
        scale: Vec3::ONE,
    });
    
    let child = world.spawn();
    world.add_component(child, Transform {
        translation: Vec3::new(1.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child, Parent(parent));
    
    transform_propagation_system(&mut world);
    
    let child_global = world.get_component::<GlobalTransform>(child).unwrap();
    let pos = child_global.translation();
    
    // After 90° rotation, (1,0,0) becomes (0,0,-1)
    assert!(pos.x.abs() < 0.001);
    assert!((pos.z + 1.0).abs() < 0.001);
}

#[test]
fn test_multi_level_hierarchy() {
    let mut world = World::new();
    
    // Grandparent -> Parent -> Child
    let grandparent = world.spawn();
    world.add_component(grandparent, Transform {
        translation: Vec3::new(10.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    
    let parent = world.spawn();
    world.add_component(parent, Transform {
        translation: Vec3::new(5.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(parent, Parent(grandparent));
    
    let child = world.spawn();
    world.add_component(child, Transform {
        translation: Vec3::new(3.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });
    world.add_component(child, Parent(parent));
    
    transform_propagation_system(&mut world);
    
    let child_global = world.get_component::<GlobalTransform>(child).unwrap();
    assert!((child_global.translation().x - 18.0).abs() < 0.001);
}
```

## Performance Targets

| Scenario | Target |
|----------|--------|
| 1,000 flat entities | < 0.5ms |
| 10,000 flat entities | < 5ms |
| 1,000 entities, depth 10 | < 2ms |
| Dirty subset (10%) | < 1ms |

## Hints & Guidance

### Getting Started
1. Start with simple parent-child (depth 1)
2. Use matrix multiplication: `child_world = parent_world * child_local`
3. Add dirty flagging after basic version works

### Common Pitfalls
- **Cycle Detection**: Prevent entity from being its own ancestor
- **Update Order**: Must process parents before children
- **Matrix Multiplication Order**: Matrix ops are not commutative!
- **Quaternion Normalization**: Accumulation errors require periodic normalization

### Key Concepts

**Transform Spaces**
- **Local Space**: Relative to parent
- **World Space**: Relative to origin
- **Model Space**: Object's own coordinate system

**Matrix Composition**
```
world_matrix = parent_world * TRS(local_translation, local_rotation, local_scale)
```

**Dirty Propagation**
- When parent changes, mark all descendants dirty
- Only recompute matrices for dirty transforms
- Clear dirty flag after update

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
    
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.translation,
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GlobalTransform {
    matrix: Mat4,
}

impl GlobalTransform {
    pub fn from_matrix(matrix: Mat4) -> Self {
        Self { matrix }
    }
    
    pub fn translation(&self) -> Vec3 {
        self.matrix.to_scale_rotation_translation().2
    }
    
    pub fn rotation(&self) -> Quat {
        self.matrix.to_scale_rotation_translation().1
    }
    
    pub fn scale(&self) -> Vec3 {
        self.matrix.to_scale_rotation_translation().0
    }
    
    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }
}

pub struct Parent(pub Entity);
pub struct Children(pub Vec<Entity>);

// System to propagate transforms
pub fn transform_propagation_system(world: &mut World) {
    // First pass: update root entities (no parent)
    let roots: Vec<Entity> = world
        .query::<(Entity, &Transform), Without<Parent>>()
        .collect();
    
    for entity in roots {
        if let Some(transform) = world.get_component::<Transform>(entity) {
            let matrix = transform.to_matrix();
            
            if let Some(global) = world.get_component_mut::<GlobalTransform>(entity) {
                global.matrix = matrix;
            } else {
                world.add_component(entity, GlobalTransform::from_matrix(matrix));
            }
            
            // Propagate to children
            propagate_to_children(world, entity, matrix);
        }
    }
}

fn propagate_to_children(world: &mut World, parent_entity: Entity, parent_matrix: Mat4) {
    if let Some(children) = world.get_component::<Children>(parent_entity) {
        let child_entities: Vec<Entity> = children.0.clone();
        
        for child_entity in child_entities {
            if let Some(local_transform) = world.get_component::<Transform>(child_entity) {
                let local_matrix = local_transform.to_matrix();
                let global_matrix = parent_matrix * local_matrix;
                
                if let Some(global) = world.get_component_mut::<GlobalTransform>(child_entity) {
                    global.matrix = global_matrix;
                } else {
                    world.add_component(
                        child_entity,
                        GlobalTransform::from_matrix(global_matrix),
                    );
                }
                
                // Recursively propagate to grandchildren
                propagate_to_children(world, child_entity, global_matrix);
            }
        }
    }
}

// Helper to add child to parent
pub fn add_child(world: &mut World, parent: Entity, child: Entity) {
    // Add Parent component to child
    world.add_component(child, Parent(parent));
    
    // Add child to parent's Children list
    if let Some(children) = world.get_component_mut::<Children>(parent) {
        if !children.0.contains(&child) {
            children.0.push(child);
        }
    } else {
        world.add_component(parent, Children(vec![child]));
    }
}

pub fn remove_child(world: &mut World, parent: Entity, child: Entity) {
    // Remove Parent component from child
    world.remove_component::<Parent>(child);
    
    // Remove child from parent's Children list
    if let Some(children) = world.get_component_mut::<Children>(parent) {
        children.0.retain(|&e| e != child);
    }
}
```

</details>

## Related Resources

- [Praxis Scene Documentation](../../reference/crates.md#praxis_scene)
- [Transform Propagation Benchmark](../../benchmarking.md#transform-propagation)
- [Understanding Transform Hierarchies](../../concepts/transforms.md)

## Next Steps

- Add dirty flagging optimization
- Implement transform interpolation (Exercise 01)
- Study skeletal animation (Exercise 48)
