# Component Storage Strategies

Entity Component System (ECS) architectures organize game data into entities (IDs), components (data), and systems (behavior). How components are stored in memory fundamentally affects performance, flexibility, and implementation complexity.

## The Core Problem

An ECS needs to:
1. **Store component data** for thousands to millions of entities
2. **Query efficiently** (e.g., "all entities with Position + Velocity")
3. **Iterate quickly** over components for systems
4. **Add/remove components** dynamically
5. **Access components randomly** by entity ID

No single storage strategy is perfect for all access patterns. The choice involves trade-offs between iteration speed, memory overhead, query flexibility, and random access.

## Pattern Variants

### 1. Table-Based Storage (Parallel Arrays)

**Concept**: Each component type gets its own array. Entity IDs are indices into these arrays.

```
# Conceptual structure
entities = [0, 1, 2, 3, 4, 5, ...]

positions = [
    0: Position(x=10, y=20),
    1: Position(x=15, y=25),
    2: None,  # Entity 2 doesn't have Position
    3: Position(x=5, y=8),
    ...
]

velocities = [
    0: Velocity(dx=1, dy=0),
    1: None,  # Entity 1 doesn't have Velocity
    2: Velocity(dx=0, dy=2),
    3: Velocity(dx=-1, dy=1),
    ...
]
```

**Access patterns**:

```
# Iterate all positions (skip None entries)
for entity_id, position in enumerate(positions):
    if position is not None:
        update(position)

# Random access: O(1)
position = positions[entity_id]

# Query (Position + Velocity): Must check both arrays
for entity_id in range(max_entities):
    pos = positions[entity_id]
    vel = velocities[entity_id]
    if pos is not None and vel is not None:
        update(pos, vel)
```

**Trade-offs**:

✅ **Strengths**:
- Simple to understand and implement
- O(1) random access by entity ID
- Good cache locality for single-component iteration
- Low memory overhead per component (just the data)
- Easy to implement in any language

❌ **Weaknesses**:
- Wasted memory for sparse components (many None/null entries)
- Multi-component queries require checking multiple arrays
- Iteration wastes time on null checks
- Fragmentation as entities are created/destroyed
- Poor cache locality for multi-component queries

**When to use**:
- Simple games with few component types
- Dense components (most entities have them)
- Single-component systems dominate
- Memory-constrained environments where overhead matters

**Real-world examples**:
- Early ECS implementations
- Component pools in custom engines
- Unity's internal component storage (simplified)

**Memory characteristics**:
- **Best case**: All entities have all components (0% waste)
- **Worst case**: Sparse components (e.g., 1% of entities have component X → 99% waste)
- **Typical**: 30-70% waste depending on component distribution

### 2. Archetype-Based Storage (Table per Archetype)

**Concept**: Group entities by their component "signature" (archetype). Each archetype gets a packed table of its components.

```
# Archetype: (Position, Velocity)
archetype_1 = {
    entities: [0, 3, 7, 9],
    positions: [
        Position(x=10, y=20),
        Position(x=5, y=8),
        Position(x=30, y=15),
        Position(x=12, y=33),
    ],
    velocities: [
        Velocity(dx=1, dy=0),
        Velocity(dx=-1, dy=1),
        Velocity(dx=0, dy=-2),
        Velocity(dx=3, dy=0),
    ]
}

# Archetype: (Position, Health)
archetype_2 = {
    entities: [1, 2, 5],
    positions: [
        Position(x=15, y=25),
        Position(x=8, y=10),
        Position(x=20, y=30),
    ],
    healths: [
        Health(hp=100),
        Health(hp=75),
        Health(hp=50),
    ]
}
```

**Access patterns**:

```
# Iterate Position + Velocity: Just iterate matching archetype
for archetype in archetypes_matching(Position, Velocity):
    for i in range(archetype.entity_count):
        pos = archetype.positions[i]
        vel = archetype.velocities[i]
        update(pos, vel)

# Random access: Need entity → archetype lookup
archetype, index = entity_to_archetype_map[entity_id]
position = archetype.positions[index]

# Add component: May move entity to different archetype
move_entity_to_archetype(entity_id, new_archetype)
```

**Trade-offs**:

✅ **Strengths**:
- Excellent cache locality for queries (data packed contiguously)
- Zero wasted memory (no null components)
- Fast multi-component iteration (data layout matches access pattern)
- Optimal for "mega-systems" (iterate 100k+ entities)
- Scales to millions of entities

❌ **Weaknesses**:
- Complex to implement correctly
- Adding/removing components is expensive (move entity between archetypes)
- Random access requires indirection (entity → archetype lookup)
- Memory fragmentation if many archetypes with few entities
- Not friendly to structural changes (adding/removing components frequently)

**When to use**:
- Performance-critical systems iterating many entities
- Stable component sets (not changing frequently)
- Games with clear entity categories (player, enemy, projectile)
- AAA games with millions of entities

**Real-world examples**:
- **Unity DOTS** (Data-Oriented Technology Stack)
- **Bevy ECS** (Rust game engine)
- **Flecs** (high-performance C ECS)
- **Our Entity Component System** (OECS)

**Memory characteristics**:
- **Best case**: Few archetypes, many entities each (optimal packing)
- **Worst case**: Many archetypes, few entities each (fragmentation)
- **Typical**: 10-100 archetypes, excellent packing within each

### 3. Sparse Set Storage

**Concept**: Each component type uses two arrays: a **sparse array** (entity → index) and a **dense array** (packed component data).

```
# Component: Position
position_component = {
    # Sparse array: entity_id → index in dense array (or -1 if doesn't exist)
    sparse: [
        0:  0,   # Entity 0 has Position at dense[0]
        1: -1,   # Entity 1 doesn't have Position
        2:  2,   # Entity 2 has Position at dense[2]
        3:  1,   # Entity 3 has Position at dense[1]
        ...
    ],
    
    # Dense array: packed component data + entity IDs
    dense_entities: [0, 3, 2, 7, ...],
    dense_data: [
        Position(x=10, y=20),  # Entity 0's position
        Position(x=5, y=8),    # Entity 3's position
        Position(x=8, y=10),   # Entity 2's position
        Position(x=30, y=15),  # Entity 7's position
        ...
    ]
}
```

**Access patterns**:

```
# Iterate all positions: Use dense array (packed)
for i in range(position_component.count):
    entity_id = position_component.dense_entities[i]
    position = position_component.dense_data[i]
    update(position)

# Random access: O(1) via sparse array
index = position_component.sparse[entity_id]
if index != -1:
    position = position_component.dense_data[index]

# Add component: Append to dense, update sparse
index = position_component.count
position_component.dense_entities[index] = entity_id
position_component.dense_data[index] = new_position
position_component.sparse[entity_id] = index
position_component.count += 1

# Remove component: Swap with last, update sparse
index = position_component.sparse[entity_id]
last_index = position_component.count - 1

# Swap with last element
position_component.dense_data[index] = position_component.dense_data[last_index]
position_component.dense_entities[index] = position_component.dense_entities[last_index]

# Update sparse mapping for swapped entity
swapped_entity = position_component.dense_entities[index]
position_component.sparse[swapped_entity] = index

position_component.sparse[entity_id] = -1
position_component.count -= 1
```

**Trade-offs**:

✅ **Strengths**:
- Fast iteration (dense array is packed)
- O(1) random access
- O(1) add/remove component (no archetype moves)
- Flexible (structural changes are cheap)
- No wasted memory in dense arrays
- Simple mental model

❌ **Weaknesses**:
- Sparse array can be large (size = max entity ID)
- Memory overhead (sparse + dense arrays)
- Multi-component queries less efficient than archetypes
- Cache misses when accessing multiple components (data not co-located)
- Iteration order unstable (swap-remove changes order)

**When to use**:
- Frequent component add/remove operations
- Sparse components (only some entities have them)
- Need fast random access
- Prototyping (easier to implement than archetypes)

**Real-world examples**:
- **EnTT** (C++ ECS library)
- **Shipyard** (Rust ECS)
- **specs** (older Rust ECS)
- Many custom game engines

**Memory characteristics**:
- **Sparse array**: O(max_entity_id) - can be large
- **Dense array**: O(component_count) - perfectly packed
- **Optimization**: Can use paging/chunking for sparse array

### 4. Hybrid / Tiered Storage

**Concept**: Use different storage strategies for different component types based on their characteristics.

```
# Dense components (90%+ of entities): Archetype storage
archetypes_for_common_components = [
    Archetype(Position, Velocity),
    Archetype(Position, Velocity, Health),
    ...
]

# Sparse components (< 10% of entities): Sparse set storage
sparse_components = {
    Loot: SparseSet(...),
    QuestGiver: SparseSet(...),
    Boss: SparseSet(...),
}

# Singleton components (exactly 1 entity): Direct storage
singletons = {
    GameState: GameState(...),
    Camera: Camera(...),
}

# Tag components (no data): Bitsets
tags = {
    Player: Bitset(entity_ids),
    Enemy: Bitset(entity_ids),
    Dead: Bitset(entity_ids),
}
```

**Trade-offs**:

✅ **Strengths**:
- Optimal storage for each component type
- Minimizes memory waste
- Best performance for each access pattern
- Scales to very large entity counts

❌ **Weaknesses**:
- Most complex to implement
- Need heuristics to choose storage strategy
- Query logic becomes more complex
- Harder to debug and reason about

**When to use**:
- Large-scale games with diverse component types
- When profiling shows different components need different strategies
- AAA engines with performance budgets

**Real-world examples**:
- **Unreal Engine** (various storage strategies per component)
- **Flecs** (configurable storage per component)
- Custom AAA engines

## Comparison Table

| Strategy | Iteration | Random Access | Add/Remove | Memory | Multi-Query | Complexity |
|----------|-----------|---------------|------------|--------|-------------|------------|
| **Table-based** | 🟡 Moderate | ✅ O(1) | ✅ O(1) | ❌ Wasteful | ❌ Slow | ⭐ Simple |
| **Archetype** | ✅ Fastest | 🟡 Indirect | ❌ Expensive | ✅ Packed | ✅ Fastest | ⭐⭐⭐ Complex |
| **Sparse Set** | ✅ Fast | ✅ O(1) | ✅ O(1) | 🟡 Overhead | 🟡 Moderate | ⭐⭐ Moderate |
| **Hybrid** | ✅ Optimal | ✅ Optimal | ✅ Optimal | ✅ Optimal | ✅ Optimal | ⭐⭐⭐⭐ Very Complex |

## Advanced Considerations

### Query Caching

Many ECS implementations cache query results to avoid repeated archetype/component lookups:

```
# First query: Find all archetypes matching (Position, Velocity)
query_result = find_archetypes_matching(Position, Velocity)

# Cache for future frames
query_cache[(Position, Velocity)] = query_result

# Invalidate cache when archetypes change
on_archetype_created():
    clear_query_cache()
```

**Trade-off**: Memory vs. CPU time

### Change Detection

Track which components have been modified to skip unchanged data:

```
# Add version counter to component data
positions = [
    (Position(x=10, y=20), version=42),
    (Position(x=15, y=25), version=43),
    ...
]

# Systems track last-seen version
system.last_version = 40

# Only process changed components
for pos, version in positions:
    if version > system.last_version:
        update(pos)
```

**Use cases**: 
- Rendering (only re-upload changed meshes)
- Networking (only send modified components)
- Physics (only update modified transforms)

### Chunk-Based Allocation

Instead of single large arrays, allocate fixed-size chunks:

```
chunk_size = 16384  # 16KB chunks

archetype = {
    chunks: [
        Chunk(positions=[...], velocities=[...]),  # 0-255 entities
        Chunk(positions=[...], velocities=[...]),  # 256-511 entities
        ...
    ]
}
```

**Benefits**:
- Reduces memory fragmentation
- Better cache locality (chunks fit in L3)
- Easier to parallelize (chunk per thread)
- Limits worst-case allocation size

**Used in**: Unity DOTS, Bevy

### Multi-threading Strategies

Different storage strategies enable different parallelization:

**Archetype-based**:
```
# Parallel iteration: One archetype per thread (no conflicts)
parallel_for archetype in archetypes:
    for entity in archetype:
        update(entity)
```

**Sparse set**:
```
# Need synchronization if multiple components modified
# Option 1: Partition entities
thread_1: process entities [0, 1000)
thread_2: process entities [1000, 2000)

# Option 2: Read-write locks per component
acquire_read(positions)
acquire_write(velocities)
```

**Archetype storage is generally more thread-friendly** (natural data partitioning).

## Component Organization Patterns

### Tags vs. Data Components

**Tags**: Marker components with no data (just presence/absence)

```
# Option 1: Empty struct
struct Player {}  # Just marks entity as player

# Option 2: Bitset (more efficient)
player_tag = Bitset([0, 5, 12, ...])  # Entity IDs with Player tag

# Query
if entity in player_tag:
    ...
```

**Data components**: Actual data

```
struct Health { hp: f32, max_hp: f32 }
```

**Optimization**: Store tags in bitsets separately from component data (saves memory).

### Shared Components

Components with same value across many entities:

```
# Instead of duplicating
entity_0: Material(texture="stone.png", shader="pbr")
entity_1: Material(texture="stone.png", shader="pbr")
entity_2: Material(texture="stone.png", shader="pbr")
...

# Share the data
material_shared = Material(texture="stone.png", shader="pbr")
entity_0: MaterialRef(id=42)
entity_1: MaterialRef(id=42)
entity_2: MaterialRef(id=42)
```

**Used in**: Unity DOTS (Shared Components), Unreal (Instanced Components)

### Dynamic vs. Static Components

Some ECS systems separate:

- **Static components**: Never/rarely change (e.g., rendering mesh, archetype)
- **Dynamic components**: Change frequently (e.g., position, velocity)

**Optimization**: Different update frequencies, different memory layouts

## Practical Recommendations

### For Beginners
Start with **table-based** or **sparse set**:
- Simpler to understand
- Easier to debug
- Good enough for small games (<10k entities)

### For Performance-Critical Games
Use **archetype-based** if:
- Iterating 100k+ entities
- Component sets are stable (not changing)
- Willing to invest in complex implementation

Use **sparse set** if:
- Frequent component add/remove
- Many sparse components
- Need simple mental model

### For Large-Scale Engines
Consider **hybrid approach**:
- Profile component usage patterns
- Use archetype for common, dense components
- Use sparse set for rare, sparse components
- Use bitsets for tags

### Implementation Checklist

When implementing ECS component storage:

- [ ] Identify common query patterns (what systems need)
- [ ] Measure component density (% of entities with each component)
- [ ] Profile iteration vs. random access frequency
- [ ] Choose storage strategy based on data
- [ ] Implement query caching if queries are repeated
- [ ] Add change detection if needed (rendering, networking)
- [ ] Consider multi-threading requirements
- [ ] Benchmark with realistic entity counts
- [ ] Test component add/remove performance
- [ ] Profile memory usage at scale

## Common Pitfalls

### Pitfall 1: Premature Optimization

Don't start with archetype storage unless you need it.

```
# Start simple
components = {
    Position: [pos1, pos2, ...],
    Velocity: [vel1, vel2, ...],
}

# Profile, then optimize if needed
```

### Pitfall 2: Not Considering Query Patterns

Design storage around how you **access** data, not how you **think about** entities.

```
# If you mostly do:
for entity with (Position, Velocity):
    update_movement()

# Then optimize for this pattern (archetype or sparse set)
# Not for random access by entity ID
```

### Pitfall 3: Ignoring Memory Fragmentation

```
# BAD: Creates 1000 archetypes with 1 entity each
for i in range(1000):
    create_entity_with_unique_components()

# GOOD: Reuse common archetypes
standardize_component_combinations()
```

### Pitfall 4: Not Measuring Real Performance

Theoretical performance ≠ real performance. Always profile on target hardware:

```
# Measure:
- Time per iteration (not just big-O)
- Cache misses (use profiling tools)
- Memory usage (not just algorithmic complexity)
- Allocation patterns (GC pressure, fragmentation)
```

## Further Reading

### Articles & Blogs
- **"Understanding Data-Oriented Design for Entity Component Systems"** - Sander Mertens
- **"Archetypes and Vectorization"** - Unity DOTS blog
- **"Building an ECS"** series by Austin Morlan
- **"SoA vs AoS"** (Structure of Arrays vs Array of Structures)

### Academic Papers
- **"Data-Oriented Design"** by Richard Fabian (book)
- **"Performance Analysis of Entity Systems"** - various GDC talks
- **"Cache-Oblivious Algorithms"** - MIT lecture notes

### Open Source Implementations
- **bevy_ecs** (Rust) - archetypal, well-documented
- **EnTT** (C++) - sparse set, header-only
- **Flecs** (C) - hybrid approach, very fast
- **Unity DOTS** - commercial archetypal implementation

### GDC Talks
- **"Overwatch Gameplay Architecture and Netcode"** - archetype-based ECS
- **"Data-Oriented Design in Practice"** - Insomniac Games
- **"Ditching OOP for Data-Oriented Design"** - various talks

### Documentation
- **Bevy ECS**: Excellent Rust-based ECS with clear explanations
- **Unity DOTS**: Commercial archetype implementation
- **EnTT**: C++ sparse set implementation
- **Flecs**: C hybrid implementation with great docs

## Summary

Component storage strategy is a fundamental ECS design choice:

- **Table-based**: Simple, good for dense components, wasteful for sparse
- **Archetype**: Fastest iteration, best cache locality, complex to implement
- **Sparse set**: Balanced, flexible, good for most use cases
- **Hybrid**: Optimal but most complex

Choose based on:
1. Entity count (small vs. millions)
2. Query patterns (iteration vs. random access)
3. Component dynamics (stable vs. changing)
4. Implementation complexity tolerance

For most games, **sparse set** is the sweet spot. For massive scale (AAA), **archetype** wins despite complexity. For simple games, **table-based** is fine.

The "best" storage strategy depends entirely on your specific access patterns and scale requirements. Profile and measure with realistic data!
