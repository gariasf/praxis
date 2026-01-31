# Praxis ECS Documentation Index

Complete guide to the documentation available for the Praxis ECS crate.

## Quick Navigation

- 🚀 **New to ECS?** → [Getting Started Guide](./GETTING_STARTED.md)
- 📖 **Learning ECS Concepts?** → [README](./README.md)
- 🎯 **Writing Production Code?** → [Best Practices](./ECS_BEST_PRACTICES.md)
- 🔍 **Need Query Examples?** → [Query Patterns](./QUERY_PATTERNS.md)
- ⚙️ **System Order Issues?** → [System Ordering](./SYSTEM_ORDERING.md)
- 🌲 **Working with Hierarchies?** → [Transform Propagation](./transform-propagation.md)
- 💾 **Saving/Loading Worlds?** → [Serialization Guide](./serialization.md)

## Documentation by Skill Level

### Beginner (New to ECS)

Start here if you're new to Entity Component Systems:

1. **[Getting Started Guide](./GETTING_STARTED.md)**
   - 5-minute quick start
   - Your first game (15 minutes)
   - Common patterns
   - Debugging tips

2. **[README.md](./README.md)**
   - Why ECS over OOP?
   - Core concepts explained
   - Built-in components
   - Basic examples

### Intermediate (Building Games)

Once you understand the basics:

1. **[Query Patterns](./QUERY_PATTERNS.md)**
   - Basic queries
   - Filters (With, Without, Changed, Added)
   - Mutable vs immutable access
   - ParamSet for conflicts
   - Performance tips

2. **[System Ordering](./SYSTEM_ORDERING.md)**
   - Why order matters
   - Automatic parallelism
   - Explicit ordering with `.chain()`
   - System sets
   - Debugging order issues

3. **[Transform Propagation](./transform-propagation.md)**
   - Parent-child hierarchies
   - Transform systems
   - GlobalTransform updates
   - Common pitfalls

### Advanced (Production Code)

Writing production-quality game code:

1. **[ECS Best Practices](./ECS_BEST_PRACTICES.md)**
   - Component design patterns
   - System architecture
   - Performance optimization
   - Anti-patterns to avoid
   - Testing strategies

2. **[Serialization Guide](./serialization.md)**
   - Save/load world state
   - Component registration
   - Entity reference resolution
   - Custom serialization

## Documentation by Topic

### Core Concepts

| Topic | Document | Section |
|-------|----------|---------|
| What is ECS? | [README](./README.md) | Overview, Why ECS? |
| Entities | [README](./README.md) | Core Concepts → Entities |
| Components | [README](./README.md) | Core Concepts → Components |
| Systems | [README](./README.md) | Core Concepts → Systems |
| Resources | [README](./README.md) | Core Concepts → Resources |

### Component Design

| Topic | Document | Section |
|-------|----------|---------|
| Component guidelines | [Best Practices](./ECS_BEST_PRACTICES.md) | Component Design |
| Marker components | [Best Practices](./ECS_BEST_PRACTICES.md) | Keep Components Pure Data |
| Component bundles | [Getting Started](./GETTING_STARTED.md) | Pattern 1 |
| Serializable components | [Serialization](./serialization.md) | SerializableComponent Trait |

### Querying

| Topic | Document | Section |
|-------|----------|---------|
| Basic queries | [Query Patterns](./QUERY_PATTERNS.md) | Basic Queries |
| Filters | [Query Patterns](./QUERY_PATTERNS.md) | Query Filters |
| Mutable access | [Query Patterns](./QUERY_PATTERNS.md) | Mutable vs Immutable |
| ParamSet | [Query Patterns](./QUERY_PATTERNS.md) | Advanced Patterns |
| Performance | [Query Patterns](./QUERY_PATTERNS.md) | Performance Considerations |

### System Design

| Topic | Document | Section |
|-------|----------|---------|
| System basics | [Getting Started](./GETTING_STARTED.md) | Step 4: Create Systems |
| Single responsibility | [Best Practices](./ECS_BEST_PRACTICES.md) | System Design |
| Commands | [Best Practices](./ECS_BEST_PRACTICES.md) | Use Commands for Structural Changes |
| System ordering | [System Ordering](./SYSTEM_ORDERING.md) | All sections |
| System sets | [System Ordering](./SYSTEM_ORDERING.md) | System Sets |

### Hierarchies & Transforms

| Topic | Document | Section |
|-------|----------|---------|
| Parent-child setup | [Getting Started](./GETTING_STARTED.md) | Pattern 5 |
| Transform propagation | [Transform Propagation](./transform-propagation.md) | All sections |
| Transform systems | [System Ordering](./SYSTEM_ORDERING.md) | Transform Hierarchy Pattern |

### Performance

| Topic | Document | Section |
|-------|----------|---------|
| Change detection | [Best Practices](./ECS_BEST_PRACTICES.md) | Use Change Detection |
| Batch operations | [Best Practices](./ECS_BEST_PRACTICES.md) | Batch Spawning |
| Query optimization | [Query Patterns](./QUERY_PATTERNS.md) | Performance Considerations |
| Parallelism | [System Ordering](./SYSTEM_ORDERING.md) | Automatic Ordering |

### Persistence

| Topic | Document | Section |
|-------|----------|---------|
| Saving worlds | [Serialization](./serialization.md) | Serialization Example |
| Loading worlds | [Serialization](./serialization.md) | Deserialization |
| Component registry | [Serialization](./serialization.md) | Component Registry Pattern |
| Entity references | [Serialization](./serialization.md) | Entity Reference Resolution |

## Code Examples

### In-Documentation Examples

All documentation includes inline code examples. Key examples:

- **[Getting Started](./GETTING_STARTED.md)**: Complete 15-minute game
- **[Query Patterns](./QUERY_PATTERNS.md)**: Every query type with examples
- **[Best Practices](./ECS_BEST_PRACTICES.md)**: Good vs bad patterns side-by-side
- **[System Ordering](./SYSTEM_ORDERING.md)**: Common ordering patterns

### Runnable Examples

Complete working examples in `examples/`:

- `examples/ecs_integration.rs` - Basic ECS with graphics
- `examples/ecs_patterns_demo.rs` - Demonstrates all patterns
- `examples/transform_propagation_demo.rs` - Hierarchies
- `examples/scene_serialization_demo.rs` - Save/load

Run with: `cargo run --example ecs_patterns_demo`

## Built-in Components Reference

The ECS provides these built-in components:

### Transform Components
- `Transform` - Local position/rotation/scale
- `GlobalTransform` - World-space transform
- `Parent` - Parent entity reference
- `Children` - Child entity list

### Camera Components
- `Camera` - Camera marker
- `PerspectiveProjection` - Perspective camera
- `OrthographicProjection` - Orthographic camera
- `CameraMatrices` - View/projection matrices

### Lighting Components
- `DirectionalLight` - Sun-like lights
- `PointLight` - Omni-directional lights
- `LightingData` - Resource with all lights

### Rendering Components
- `MeshHandle` - Reference to mesh
- `MaterialHandle` - Reference to material
- `Visibility` - Show/hide entities
- `BoundingBox` - Spatial bounds

### Utility Components
- `Name` - Debug name
- `Active` - Active marker
- `NoSave` - Exclude from serialization

See [README Components Section](./README.md#built-in-components) for details.

## Common Questions

| Question | Answer In |
|----------|-----------|
| How do I get started? | [Getting Started Guide](./GETTING_STARTED.md) |
| What are markers? | [Best Practices](./ECS_BEST_PRACTICES.md) - Marker Components |
| How do queries work? | [Query Patterns](./QUERY_PATTERNS.md) |
| Why won't systems run in parallel? | [System Ordering](./SYSTEM_ORDERING.md) - Mutable Access |
| How do I make a hierarchy? | [Getting Started](./GETTING_STARTED.md) - Pattern 5 |
| How do I save/load? | [Serialization](./serialization.md) |
| What's the difference between Transform and GlobalTransform? | [Transform Propagation](./transform-propagation.md) |

## Contributing to Documentation

Found an issue or want to add examples?

1. Documentation source is in `crates/praxis_ecs/`
2. Follow existing patterns and style
3. Include runnable code examples
4. Test all code examples compile
5. Update this index if adding new docs

## External Resources

### Bevy ECS Documentation

Since Praxis uses `bevy_ecs`, the Bevy documentation is useful:

- [Bevy ECS Book](https://bevyengine.org/learn/book/getting-started/ecs/)
- [Bevy Examples](https://github.com/bevyengine/bevy/tree/main/examples#ecs-entity-component-system)

Note: Praxis wraps bevy_ecs with additional features and engine-specific components.

### ECS General Resources

- [ECS FAQ](https://github.com/SanderMertens/ecs-faq) - General ECS concepts
- [Awesome ECS](https://github.com/jslee02/awesome-entity-component-system) - ECS resources

## Quick Reference Card

```rust
// World & Entities
let mut world = World::new();
let entity = world.spawn((ComponentA, ComponentB));

// Components
#[derive(Component)]
struct MyComponent { data: f32 }

// Systems
fn my_system(query: Query<&MyComponent>) { }

// Queries
Query<&Transform>                          // Read
Query<&mut Transform>                      // Write  
Query<&Transform, With<Player>>            // Filter
Query<&Transform, Changed<Transform>>      // Changed only
Query<(Entity, &Transform)>                // With entity ID

// System Ordering
schedule.add_systems((sys_a, sys_b).chain());  // Sequential
schedule.add_systems(sys_a.before(sys_b));     // Explicit order

// Resources
world.insert_resource(MyResource);
fn system(res: Res<MyResource>) { }        // Read
fn system(mut res: ResMut<MyResource>) { } // Write

// Commands
fn system(mut commands: Commands) {
    commands.spawn(MyComponent);           // Deferred spawn
    commands.entity(e).despawn();          // Deferred despawn
}
```

---

**Last Updated:** This index is current as of the latest documentation revision.

For the most up-to-date information, always check the individual documents.
