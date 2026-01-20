# Decision Tree: Scene Graphs vs Flat Entity Lists

```
┌──────────────────────────────────────────────────┐
│ Should I use Scene Graphs or Flat Entity Lists? │
└──────────────────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ Do you need parent-child          │
        │ relationships for transforms?     │
        └───────────────────────────────────┘
                /                   \
               /                     \
             Yes                      No
              │                       │
              ▼                       ▼
    ┌──────────────────┐      ┌──────────────┐
    │ More questions → │      │  Flat List   │
    └──────────────────┘      │  (simpler)   │
              │               └──────────────┘
              ▼
    ┌───────────────────────┐
    │ Using ECS?            │
    └───────────────────────┘
          /           \
         /             \
       Yes              No
        │               │
        ▼               ▼
┌──────────────┐   ┌─────────────┐
│ ECS Hierarchy│   │ Scene Graph │
│ (Praxis way) │   │ (OOP style) │
└──────────────┘   └─────────────┘
        │
        ▼
┌──────────────────────────┐
│ How many objects?        │
└──────────────────────────┘
      /              \
     /                \
< 10,000           > 100,000
    │                  │
    ▼                  ▼
┌─────────┐    ┌────────────────┐
│ Simple  │    │ Consider spatial│
│ Hierarchy│    │ partitioning   │
└─────────┘    └────────────────┘
```

## Quick Decision Matrix

| Factor | Scene Graph | Flat List | ECS Hierarchy |
|--------|-------------|-----------|---------------|
| **Parent-child transforms** | ✅ Natural | ❌ Manual | ✅ Component-based |
| **Spatial queries** | ⚠️ Depends | ❌ O(n) | ⚠️ Need spatial structure |
| **Cache efficiency** | ❌ Pointer chasing | ✅ Excellent | ✅ Excellent |
| **Flexibility** | ⚠️ Tree only | ✅ High | ✅ High |
| **Implementation** | ⚠️ Complex | ✅ Simple | ⚠️ Moderate |
| **ECS compatibility** | ❌ Poor | ✅ Natural | ✅ Natural |
| **Large scale (>100k)** | ❌ Slow | ✅ With spatial structure | ✅ With spatial structure |
| **Memory locality** | ❌ Poor | ✅ Excellent | ✅ Excellent |

## Understanding the Options

### Scene Graph (Traditional OOP)

**Structure:**
```
Root
├── Camera
├── Sun (Light)
└── Level
    ├── Building1
    │   ├── Floor1
    │   │   ├── Wall1
    │   │   └── Door
    │   └── Floor2
    └── Building2
```

**Code representation:**
```cpp
class Node {
    Transform local_transform;
    Matrix4 world_transform;
    Node* parent;
    vector<Node*> children;
    
    void update() {
        world_transform = parent 
            ? parent->world_transform * local_transform
            : local_transform;
        
        for (auto child : children) {
            child->update();
        }
    }
};
```

### Flat Entity List (No Hierarchy)

**Structure:**
```
Entities: [Camera, Sun, Building1, Floor1, Wall1, Door, Floor2, Building2]
(No explicit relationships, all flat)
```

**Code representation:**
```rust
struct World {
    entities: Vec<Entity>,
}

// Each entity independent
// If you need hierarchy, manually track in components
```

### ECS Hierarchy (Praxis Approach)

**Structure:**
```
Components:
- Transform (local)
- GlobalTransform (computed)
- Parent (entity reference)
- Children (list of entity references)

Entities are flat, but components define relationships
```

**Code representation:**
```rust
#[derive(Component)]
struct Transform { /* local transform */ }

#[derive(Component)]
struct GlobalTransform { /* world transform */ }

#[derive(Component)]
struct Parent(Entity);

#[derive(Component)]
struct Children(Vec<Entity>);

// System propagates transforms through hierarchy
fn propagate_transforms(
    query: Query<(Entity, &Transform, Option<&Parent>)>,
    mut global: Query<&mut GlobalTransform>,
) {
    // Compute global transforms from hierarchy
}
```

## Detailed Analysis

### Scene Graph (Traditional Approach)

#### Choose Scene Graph If:

**✅ High Priority:**
- **Not using ECS** (traditional OOP engine)
- Working in **C++/C#** with strong OOP traditions
- Need **intuitive hierarchy** for designers
- **Small-to-medium scenes** (<10,000 nodes)
- Existing codebase uses scene graphs

**Example Use Cases:**
- Unity classic (pre-DOTS)
- Unreal Engine
- Three.js (web 3D)
- Traditional 3D editors

**Pros:**
- **Intuitive**: Matches how designers think (parent-child)
- **Encapsulation**: Each node is self-contained object
- **Traversal**: Easy to walk tree (depth-first, breadth-first)
- **Tools**: Easy to visualize in editor
- **Established pattern**: Lots of resources and examples
- **Transform propagation**: Natural recursive update

**Cons:**
- **Cache misses**: Pointer chasing destroys cache locality
- **Rigid structure**: Must be a tree (no multiple parents)
- **Performance**: O(n) tree traversal every frame
- **Memory fragmentation**: Nodes scattered in memory
- **ECS incompatibility**: Doesn't fit ECS architecture
- **Scalability**: Struggles with >50,000 nodes
- **Parallelization**: Hard to parallelize tree traversal

**Example Implementation:**
```cpp
// Unity-style scene graph
class GameObject {
    Transform transform;
    GameObject* parent;
    vector<GameObject*> children;
    
    // Recursive update (cache-unfriendly)
    void UpdateTransforms() {
        if (parent) {
            worldTransform = parent->worldTransform * transform;
        } else {
            worldTransform = transform;
        }
        
        for (auto child : children) {
            child->UpdateTransforms(); // Poor cache locality
        }
    }
};
```

**Performance:**
```
10,000 nodes scene graph update: ~2-3ms
100,000 nodes scene graph update: ~20-30ms (poor scaling)
```

### Flat Entity List (No Hierarchy)

#### Choose Flat List If:

**✅ High Priority:**
- **No parent-child relationships** needed
- **Performance critical** (cache locality matters)
- **Simple games** (no complex object hierarchies)
- Using **ECS** (natural fit)
- Need **maximum parallelization**

**Example Use Cases:**
- Bullet hell games (thousands of independent bullets)
- Particle systems (no hierarchy)
- RTS games with independent units
- Simple 2D games

**Pros:**
- **Cache efficiency**: Entities stored contiguously
- **Simplicity**: No hierarchy complexity
- **Performance**: O(1) access, easy to parallelize
- **ECS-friendly**: Perfect match for ECS
- **Scalability**: Handles millions of entities
- **Predictable**: No hidden complexity

**Cons:**
- **No hierarchy**: Must manually implement if needed
- **Transform propagation**: Not built-in
- **Logical organization**: Harder to reason about relationships
- **Editor complexity**: Flat lists harder to visualize
- **Duplication**: Can't share transforms easily

**Example Implementation:**
```rust
// Simple flat list
struct World {
    entities: Vec<Entity>,
    transforms: Vec<Transform>,
}

// Each entity independent, no parent-child
// Fast iteration, great cache locality
for transform in transforms.iter_mut() {
    transform.update(); // No hierarchy checks
}
```

**Performance:**
```
1,000,000 entities flat update: ~5ms (excellent cache locality)
```

### ECS Hierarchy (Praxis Approach)

#### Choose ECS Hierarchy If:

**✅ High Priority:**
- Using **ECS architecture**
- Need **parent-child transforms**
- Want **cache-friendly** hierarchy
- Building **modern game engine**
- Need **flexibility** (add/remove hierarchy dynamically)
- **Large-scale scenes** (>10,000 entities)

**Example Use Cases:**
- Praxis engine
- Bevy engine
- Unity DOTS
- Custom ECS engines

**Pros:**
- **ECS-compatible**: Components define relationships
- **Cache-friendly**: Hierarchy data stored in component arrays
- **Flexibility**: Entities can have hierarchy or not
- **Scalability**: Handles large entity counts
- **Parallelizable**: Systems can run in parallel
- **Composition**: Mix hierarchy with other features
- **Performance**: Much faster than traditional scene graph

**Cons:**
- **Complexity**: More complex than flat list
- **Indirection**: Following parent references requires lookups
- **System ordering**: Must update transforms in order
- **Learning curve**: Less intuitive than scene graph
- **Debugging**: Hierarchy scattered across components

**Praxis Implementation:**
```rust
// Components for hierarchy
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Component)]
pub struct GlobalTransform(pub Mat4);

#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);

// System propagates transforms
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        Without<Parent>
    >,
    mut child_query: Query<
        (&Transform, &mut GlobalTransform, Option<&Children>, &Parent)
    >,
) {
    // Update roots (no parent)
    for (entity, transform, mut global, children) in root_query.iter_mut() {
        *global = GlobalTransform(transform.compute_matrix());
        
        // Recursively update children
        if let Some(children) = children {
            propagate_children(children, &global.0, &mut child_query);
        }
    }
}

fn propagate_children(
    children: &Children,
    parent_global: &Mat4,
    child_query: &mut Query</* ... */>,
) {
    for &child_entity in &children.0 {
        if let Ok((transform, mut global, children, _parent)) = 
            child_query.get_mut(child_entity) 
        {
            let local_matrix = transform.compute_matrix();
            global.0 = *parent_global * local_matrix;
            
            if let Some(children) = children {
                propagate_children(children, &global.0, child_query);
            }
        }
    }
}
```

**Performance:**
```
10,000 entities with hierarchy: ~1ms (cache-friendly)
100,000 entities with hierarchy: ~8ms (scales well)
```

## Comparison Scenarios

### Scenario 1: Articulated Robot Arm

**Requirement:** Robot with multiple segments, each rotating relative to parent.

```
Robot
├── Base
├── Shoulder
│   └── Upper Arm
│       └── Elbow
│           └── Forearm
│               └── Wrist
│                   └── Hand
```

**Scene Graph:**
```cpp
class RobotArm {
    Node* base;
    Node* shoulder;
    Node* upper_arm;
    // ...
    
    void Rotate(float angle) {
        shoulder->local_rotation += angle;
        // Recursive update propagates automatically
        base->UpdateTransforms();
    }
};
```
✅ **Natural and intuitive**

**Flat List:**
```rust
// Must manually compute each segment's world position
// No automatic propagation
struct RobotSegment {
    parent_id: Option<EntityId>,
    local_transform: Transform,
    world_transform: Transform,
}

fn update_robot() {
    // Manually walk chain and compute transforms
    for segment in segments {
        if let Some(parent) = segment.parent_id {
            segment.world = parent.world * segment.local;
        }
    }
}
```
❌ **Manual and error-prone**

**ECS Hierarchy (Praxis):**
```rust
// Spawn robot with hierarchy
let base = commands.spawn()
    .insert(Transform::default())
    .insert(GlobalTransform::default())
    .id();

let shoulder = commands.spawn()
    .insert(Transform::from_rotation(Quat::from_rotation_y(angle)))
    .insert(GlobalTransform::default())
    .insert(Parent(base))
    .id();

commands.entity(base).push_children(&[shoulder]);

// propagate_transforms system handles rest automatically
```
✅ **Clean and automatic**

**Winner:** Scene Graph or ECS Hierarchy (both handle well)

### Scenario 2: 100,000 Independent Projectiles

**Requirement:** Bullets flying independently, no parent-child relationships.

**Scene Graph:**
```cpp
// Wasteful - each bullet is a node in tree
for (int i = 0; i < 100000; i++) {
    Node* bullet = new Node();
    bullet->transform = /* ... */;
    root->AddChild(bullet); // Adds to hierarchy unnecessarily
}

// Update traverses entire tree (slow)
root->UpdateTransforms(); // O(n) tree traversal
```
❌ **Slow, wasteful**

**Flat List:**
```rust
// Efficient - just array of bullets
let bullets: Vec<Bullet> = Vec::with_capacity(100000);

// Update is simple loop (fast)
for bullet in bullets.iter_mut() {
    bullet.position += bullet.velocity * dt;
}
```
✅ **Fast and simple**

**ECS Hierarchy (Praxis):**
```rust
// Entities without Parent/Children components
for _ in 0..100000 {
    commands.spawn()
        .insert(Transform::default())
        .insert(Velocity::default());
}

// System updates efficiently (no hierarchy overhead)
fn update_bullets(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * dt;
    }
}
```
✅ **Fast and flexible**

**Winner:** Flat List or ECS (both excellent)

### Scenario 3: Open World with Spatial Queries

**Requirement:** Find all objects within radius of player.

**Scene Graph:**
```cpp
// Must traverse entire tree
vector<Object*> FindNearby(Vec3 pos, float radius) {
    vector<Object*> results;
    root->TraverseDepthFirst([&](Node* node) {
        if (distance(node->position, pos) < radius) {
            results.push_back(node);
        }
    });
    return results; // O(n) - checks every node
}
```
❌ **Slow spatial queries**

**Flat List with Spatial Structure:**
```rust
// Use octree or grid for spatial partitioning
struct World {
    entities: Vec<Entity>,
    octree: Octree<Entity>, // Spatial structure
}

fn find_nearby(pos: Vec3, radius: f32) -> Vec<Entity> {
    octree.query_sphere(pos, radius) // O(log n)
}
```
✅ **Fast spatial queries**

**ECS Hierarchy with Spatial Structure (Praxis):**
```rust
// Best of both worlds
// - Hierarchy for transforms
// - Spatial structure for queries

#[derive(Resource)]
struct SpatialIndex {
    octree: Octree<Entity>,
}

// System maintains spatial index
fn update_spatial_index(
    query: Query<(Entity, &GlobalTransform)>,
    mut index: ResMut<SpatialIndex>,
) {
    index.octree.clear();
    for (entity, transform) in query.iter() {
        index.octree.insert(entity, transform.translation());
    }
}

// Fast queries
fn find_nearby(
    pos: Vec3,
    radius: f32,
    index: &SpatialIndex,
) -> Vec<Entity> {
    index.octree.query_sphere(pos, radius)
}
```
✅ **Fast queries + hierarchy**

**Winner:** Flat List or ECS with separate spatial structure

## Hybrid Approaches

### Scene Graph + Spatial Partitioning

```cpp
class SceneGraph {
    Node* root;
    Octree<Node*> spatial_index;
    
    void Update() {
        root->UpdateTransforms(); // Hierarchy update
        RebuildSpatialIndex();     // Spatial structure
    }
};
```

**When to use:**
- Traditional OOP engine
- Need both hierarchy and spatial queries
- Willing to pay dual-structure cost

### ECS + Optional Hierarchy

```rust
// Some entities have hierarchy, some don't
commands.spawn()
    .insert(Transform::default())
    .insert(Parent(parent_entity)); // Optional

commands.spawn()
    .insert(Transform::default()); // No parent - flat
```

**When to use (Praxis approach):**
- Want flexibility
- Mix hierarchical and independent entities
- ECS architecture

### Flat List + Manual Parent Tracking

```rust
struct Entity {
    transform: Transform,
    parent_id: Option<EntityId>, // Manual tracking
}

fn update() {
    // Manually compute hierarchical transforms when needed
}
```

**When to use:**
- Mostly flat, occasional hierarchy
- Want control over propagation
- Performance-critical code

## Platform/Language Considerations

### Rust
**Strong recommendation: ECS Hierarchy**

Rust's ownership makes traditional scene graphs painful:
```rust
// This doesn't work in Rust
struct Node {
    parent: &mut Node,     // Can't have mutable parent ref
    children: Vec<&mut Node>, // And mutable child refs
}
```

ECS hierarchy fits Rust perfectly:
- Components owned by World
- References through Entity IDs
- Borrow checker enforced at query time

### C++
**Recommendation: Scene Graph or ECS**

C++ supports both well:
- Raw pointers allow traditional scene graphs
- ECS libraries (EnTT) provide hierarchy support

**Choose based on:**
- Team expertise (OOP vs data-oriented)
- Project scale (small = scene graph, large = ECS)
- Performance needs (critical = ECS)

### C#/Unity
**Recommendation: Scene Graph (Unity classic) or ECS (DOTS)**

Unity offers both:
- **Classic:** GameObject hierarchy (scene graph)
- **DOTS:** ECS with TransformSystem

For new Unity projects:
- Small/medium: Classic (easier)
- Large/performance-critical: DOTS (faster)

### JavaScript/Web
**Recommendation: Scene Graph**

Web 3D (Three.js, Babylon.js) uses scene graphs:
- Familiar to web developers
- Performance usually fine for web
- Easy debugging in browser tools

## Performance Benchmarks

### Transform Propagation (10,000 Entities)

**Scene Graph (OOP):**
```
Update time: 2.5ms
Memory: 800KB (fragmented)
Cache misses: High
```

**Flat List (No Hierarchy):**
```
Update time: 0.3ms
Memory: 400KB (contiguous)
Cache misses: Low
```

**ECS Hierarchy (Praxis):**
```
Update time: 0.8ms
Memory: 450KB (mostly contiguous)
Cache misses: Medium
```

### Spatial Query (Find 100 entities within radius, 100,000 total)

**Scene Graph (No Spatial Structure):**
```
Query time: 15ms (traverse all nodes)
```

**Flat List + Octree:**
```
Query time: 0.2ms (octree query)
```

**ECS Hierarchy + Octree:**
```
Query time: 0.3ms (octree query + entity lookup)
```

## Decision Checklist

| Question | Scene Graph | Flat List | ECS Hierarchy |
|----------|-------------|-----------|---------------|
| Using ECS? | | ✓ | ✓ |
| Using OOP (C++/C#)? | ✓ | | |
| Need parent-child? | ✓ | | ✓ |
| No hierarchy needed? | | ✓ | ✓ |
| < 10,000 entities? | ✓ | ✓ | ✓ |
| > 100,000 entities? | | ✓ | ✓ |
| Performance critical? | | ✓ | ✓ |
| Designer-friendly tools? | ✓ | | ⚠️ |
| Need spatial queries? | ⚠️ | ✓ | ✓ |
| Working in Rust? | | ✓ | ✓ |

**Score:**
- **Scene Graph**: Traditional OOP engine, small-medium scale
- **Flat List**: No hierarchy, performance critical
- **ECS Hierarchy**: ECS engine with hierarchy needs

## Common Pitfalls

### Scene Graph Pitfalls

1. **Deep hierarchies**: >10 levels cause performance issues
2. **Updating everything**: Don't update nodes that haven't moved
3. **No spatial structure**: Add octree/grid for spatial queries
4. **Cache misses**: Consider cache-friendly alternatives for large scenes

### Flat List Pitfalls

1. **Reimplementing hierarchy**: If you need it, use proper hierarchy
2. **No spatial structure**: O(n) queries kill performance
3. **Over-flattening**: Some logical grouping is useful

### ECS Hierarchy Pitfalls

1. **Forgetting system order**: Transform propagation must run in order
2. **Circular references**: Parent-child cycles crash system
3. **Over-querying**: Don't look up hierarchy every frame
4. **Mixing concerns**: Keep hierarchy separate from gameplay logic

## Recommended Reading

- **Scene Graphs:**
  - [3D Game Engine Design, 2nd Edition](https://www.geometrictools.com/)
  - Unity documentation - GameObject hierarchy

- **ECS Hierarchy:**
  - [Bevy Transform System](https://bevyengine.org/learn/book/migration-guides/0.5-0.6/#transform-and-globaltransform)
  - Praxis: `docs/concepts/scene-management.md`

- **Spatial Structures:**
  - [Octrees](https://en.wikipedia.org/wiki/Octree)
  - Praxis: `crates/praxis_spatial/README.md`

## Conclusion

**TL;DR:**
- **Traditional OOP engine? → Scene Graph**
- **ECS with hierarchy? → ECS Hierarchy (Praxis approach)**
- **No hierarchy needed? → Flat List**
- **Spatial queries important? → Add octree/grid regardless of choice**

**Praxis Choice: ECS Hierarchy + Spatial Structures**
- `Transform`, `GlobalTransform`, `Parent`, `Children` components
- `propagate_transforms` system
- Separate `Octree` and `BVH` for spatial queries
- Best of both worlds: hierarchy + performance

**Key insight:** Hierarchy and spatial queries are orthogonal concerns:
- **Hierarchy**: For transform propagation (parent-child)
- **Spatial structure**: For proximity queries (nearby objects)

Don't confuse them - you can have one, both, or neither depending on needs.
