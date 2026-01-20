# Memory Management Approaches

Memory management determines how a game engine allocates, tracks, and frees memory for game objects. This fundamental choice affects performance, safety, complexity, and the types of bugs that can occur.

## The Core Problem

Game engines must manage thousands to millions of objects:
- Entities, components, meshes, textures
- Temporary allocations (per-frame data)
- Long-lived resources (loaded for entire level)

Memory management must handle:
1. **Allocation**: Get memory for new objects
2. **Lifetime tracking**: Know when objects are no longer needed
3. **Deallocation**: Return memory for reuse
4. **Fragmentation**: Keep memory contiguous for performance
5. **Safety**: Prevent use-after-free, double-free, leaks

No single approach is perfect. The choice involves trade-offs between performance, safety, and programmer convenience.

## Pattern Variants

### 1. Manual Memory Management

**Concept**: Programmer explicitly allocates and frees memory. The programmer is responsible for correctness.

```c
// C/C++ example (pseudocode)
void game_loop() {
    // Allocate
    Enemy* enemy = malloc(sizeof(Enemy));
    enemy_init(enemy);
    
    // Use
    update_enemy(enemy);
    render_enemy(enemy);
    
    // Free (programmer must remember!)
    enemy_destroy(enemy);
    free(enemy);
}
```

**Lifetime management**:
```c
// Option 1: Immediate free
Enemy* enemy = create_enemy();
use(enemy);
free(enemy);  // Freed immediately after use

// Option 2: Deferred free
Enemy* enemy = create_enemy();
add_to_world(enemy);
// ... many frames later ...
remove_from_world(enemy);
free(enemy);  // Who calls this? When?

// Option 3: Pooling
Enemy* enemy = pool_alloc(&enemy_pool);
use(enemy);
pool_free(&enemy_pool, enemy);  // Return to pool
```

**Trade-offs**:

✅ **Strengths**:
- Maximum performance (no runtime overhead)
- Predictable memory usage
- Deterministic (no GC pauses)
- Full control over allocation patterns
- Minimal memory overhead (no tracking metadata)
- Works in any language (C, C++, Rust unsafe)

❌ **Weaknesses**:
- **Memory leaks**: Forgetting to free
- **Use-after-free**: Using freed memory (crash/corruption)
- **Double-free**: Freeing same memory twice (crash)
- **Dangling pointers**: Pointer to freed memory
- **Manual burden**: Programmer must track everything
- **Error-prone**: Easy to make mistakes

**When to use**:
- Performance-critical code (tight loops, allocators)
- Systems programming (OS, embedded)
- Custom allocators and memory pools
- When you need absolute control
- C codebases

**Real-world examples**:
- **id Tech engines** (Doom, Quake) - manual C
- **Custom AAA allocators** - pooling, stack allocators
- **Embedded games** (no GC available)
- **Performance-critical subsystems** in any engine

**Common patterns to improve safety**:

```c
// Pattern 1: Ownership conventions
// Naming: create/destroy, alloc/free, new/delete
Enemy* create_enemy();   // Caller owns, must destroy
void destroy_enemy(Enemy* e);

// Pattern 2: Null after free
free(ptr);
ptr = NULL;  // Prevents accidental reuse

// Pattern 3: Arena/pool allocators
Arena arena = create_arena(1MB);
void* data = arena_alloc(&arena, size);
// ... use data ...
arena_reset(&arena);  // Free all at once

// Pattern 4: RAII (C++)
{
    std::unique_ptr<Enemy> enemy = std::make_unique<Enemy>();
    // Automatically freed when scope exits
}
```

### 2. Reference Counting

**Concept**: Track how many references exist to an object. Free when count reaches zero.

```python
# Python-like pseudocode
class RefCounted:
    def __init__(self):
        self.ref_count = 1  # Creator holds first reference
    
    def add_ref(self):
        self.ref_count += 1
    
    def release(self):
        self.ref_count -= 1
        if self.ref_count == 0:
            self.destroy()  # Free memory

# Usage
enemy = Enemy()             # ref_count = 1
player.target = enemy       # ref_count = 2
enemy.add_ref()

player.target = None        # ref_count = 1
enemy.release()

# Later...
enemy.release()             # ref_count = 0 → destroyed
```

**Automatic reference counting** (ARC):
```swift
// Swift example (ARC built into language)
class Enemy {
    var health: Int
}

var enemy1: Enemy? = Enemy()  // ref_count = 1
var enemy2 = enemy1            // ref_count = 2
enemy1 = nil                   // ref_count = 1
enemy2 = nil                   // ref_count = 0 → freed
```

**Smart pointers** (C++):
```cpp
// C++ shared_ptr
std::shared_ptr<Enemy> enemy = std::make_shared<Enemy>();
// ref_count = 1

{
    auto enemy2 = enemy;  // ref_count = 2
    // ...
}  // enemy2 destroyed, ref_count = 1

enemy.reset();  // ref_count = 0 → Enemy destroyed
```

**Trade-offs**:

✅ **Strengths**:
- Automatic memory management (no explicit free)
- Deterministic (freed immediately when count → 0)
- No GC pauses
- Easier than manual (prevents most leaks)
- Clear ownership semantics
- Works with RAII patterns

❌ **Weaknesses**:
- **Cyclic references leak memory** (A → B → A)
- Runtime overhead (increment/decrement on each assign)
- Atomic operations needed (thread-safety cost)
- More memory (ref count per object)
- Not "zero-cost" (every copy/assignment has cost)
- Can't handle all object graphs

**When to use**:
- Shared ownership needed (multiple systems need same data)
- Predictable lifetimes required (no GC pauses)
- Languages supporting it well (C++, Swift, Objective-C)
- Resource management (textures, sounds loaded once)

**Real-world examples**:
- **Unreal Engine** - UObject reference system
- **COM (Component Object Model)** - Windows API
- **Swift** - ARC is the default
- **Objective-C** - ARC/Manual Retain-Release
- **Python** - CPython's primary memory management

**Cycle breaking strategies**:

```cpp
// Problem: Cycle leaks
class Node {
    std::shared_ptr<Node> next;  // Strong reference
    std::shared_ptr<Node> prev;  // Strong reference
};

Node* a = new Node();  // ref_count = 1
Node* b = new Node();  // ref_count = 1
a->next = b;            // ref_count = 2
b->prev = a;            // ref_count = 2
a.reset();              // ref_count = 1 (still alive!)
b.reset();              // ref_count = 1 (still alive!)
// LEAK! a and b reference each other

// Solution 1: Weak pointers
class Node {
    std::shared_ptr<Node> next;      // Strong (ownership)
    std::weak_ptr<Node> prev;        // Weak (no ownership)
};

// Solution 2: Manual cycle breaking
void break_cycles(Node* node) {
    node->next = nullptr;
    node->prev = nullptr;
}

// Solution 3: Ownership hierarchy
// Parent owns children (strong), children don't own parent (weak)
class Parent {
    std::vector<std::shared_ptr<Child>> children;  // Strong
};
class Child {
    std::weak_ptr<Parent> parent;  // Weak (no cycle)
};
```

### 3. Garbage Collection (Tracing GC)

**Concept**: Runtime periodically traces all reachable objects from roots. Unreachable objects are freed.

```csharp
// C# example
class Game {
    void Update() {
        // Allocate freely
        Enemy enemy = new Enemy();
        ProcessEnemy(enemy);
        
        // No explicit free!
        // GC will collect when enemy becomes unreachable
    }
}

// GC algorithm (simplified)
void garbage_collect() {
    // Phase 1: Mark (trace from roots)
    mark_set = {}
    for root in gc_roots:  // Globals, stack, registers
        mark_reachable(root, mark_set)
    
    // Phase 2: Sweep (free unmarked)
    for object in heap:
        if object not in mark_set:
            free(object)
    
    // Phase 3: Compact (optional)
    compact_heap()  // Move objects to remove fragmentation
}

void mark_reachable(obj, mark_set):
    if obj in mark_set:
        return  # Already visited
    
    mark_set.add(obj)
    
    # Recursively mark references
    for ref in obj.references:
        mark_reachable(ref, mark_set)
```

**GC variants**:

**1. Stop-the-World (Mark-Sweep)**:
```
Game runs → GC pause (STW) → Mark → Sweep → Game resumes
            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            Can be 10-100ms+ (frame drop!)
```

**2. Generational GC**:
```
# Observation: Most objects die young
# Split heap into generations:

Generation 0 (Eden): New objects
Generation 1 (Survivor): Survived 1 GC
Generation 2 (Old): Long-lived objects

# Collect Gen 0 frequently (fast, most garbage)
# Collect Gen 1 occasionally
# Collect Gen 2 rarely (slow, but little garbage)
```

**3. Incremental GC**:
```
# Spread GC work across multiple frames
Frame 1: Mark 10% of heap
Frame 2: Mark next 10%
...
Frame 10: Sweep

# Reduces pause time but increases total GC cost
```

**4. Concurrent GC**:
```
# GC runs on separate thread
Game thread: runs normally
GC thread: marks and sweeps concurrently

# Requires synchronization (complexity)
# Can still have pauses (final sweep phase)
```

**Trade-offs**:

✅ **Strengths**:
- No manual memory management
- No use-after-free bugs
- No double-free bugs
- Handles cycles automatically
- Programmer convenience
- Easier to write correct code

❌ **Weaknesses**:
- **Non-deterministic pauses** (GC can happen anytime)
- Performance overhead (tracing, synchronization)
- Higher memory usage (live + garbage until collected)
- Unpredictable frame times (bad for games!)
- No control over when objects are freed
- Finalizers add complexity

**When to use**:
- Productivity more important than control
- Managed languages (C#, Java, Go)
- Tools and editors (not runtime-critical)
- Games where GC pauses are acceptable (turn-based, casual)

**Real-world examples**:
- **Unity (C# scripts)** - GC with optimizations
- **Java games** - Minecraft (heavily optimized)
- **Godot (GDScript)** - Reference counted (like GC)
- **Unreal Engine editor** - Some C# tools

**GC optimization strategies for games**:

```csharp
// Problem: Frequent allocation causes GC pressure
void Update() {
    Vector3 direction = new Vector3(x, y, z);  // Allocation!
    // ... many times per frame = many allocations
}

// Solution 1: Reuse objects (pooling)
ObjectPool<Vector3> pool = new ObjectPool<Vector3>();

void Update() {
    Vector3 direction = pool.Get();  // Reuse
    // ... use ...
    pool.Return(direction);          // Return to pool
}

// Solution 2: Use structs (stack allocation, no GC)
struct Vector3 {  // Struct, not class
    float x, y, z;
}

void Update() {
    Vector3 direction = new Vector3(x, y, z);  // Stack, no GC!
}

// Solution 3: Pre-allocate arrays
List<Enemy> enemies = new List<Enemy>(1000);  // Pre-size

// Solution 4: Avoid closures/lambdas (allocate delegates)
// Bad: Creates garbage
button.onClick += () => DoSomething();

// Good: Reuse delegate
Action callback = DoSomething;
button.onClick += callback;
```

### 4. Ownership-Based (Rust-style)

**Concept**: The type system tracks ownership at compile time. Compiler enforces single owner and lifetime rules.

```rust
// Rust example
fn main() {
    // enemy1 owns the Enemy
    let enemy1 = Enemy::new();
    
    // Ownership moved to enemy2
    let enemy2 = enemy1;
    
    // Compile error! enemy1 no longer owns it
    // use(enemy1);  ❌ Error: value moved
    
    use(enemy2);  // ✅ OK
    
}  // enemy2 dropped here (freed automatically)

// Borrowing (references)
fn process(enemy: &Enemy) {  // Borrow, don't take ownership
    // Can read enemy, but can't free it
}

fn main() {
    let enemy = Enemy::new();
    process(&enemy);  // Borrow
    use(enemy);       // Still valid!
}  // enemy freed here
```

**Lifetime rules**:
```rust
// Rule 1: One owner at a time
let a = String::from("hello");
let b = a;  // Ownership moved
// a is now invalid

// Rule 2: Multiple immutable borrows OR one mutable borrow
let s = String::from("hello");

let r1 = &s;    // ✅ Immutable borrow
let r2 = &s;    // ✅ Immutable borrow
// let r3 = &mut s;  ❌ Error: can't borrow mutably while immutably borrowed

let mut s = String::from("hello");
let r1 = &mut s;  // ✅ Mutable borrow
// let r2 = &s;     ❌ Error: can't borrow immutably while mutably borrowed

// Rule 3: References cannot outlive owner
fn bad() -> &String {
    let s = String::from("hello");
    &s  // ❌ Error: s is freed at end of function
}
```

**Trade-offs**:

✅ **Strengths**:
- **Zero runtime overhead** (compile-time checks)
- Memory safe (no use-after-free at compile time)
- No GC pauses (deterministic)
- Thread-safe by default (data races prevented)
- No reference counting overhead
- Explicit ownership (clear semantics)

❌ **Weaknesses**:
- **Learning curve** (borrow checker is strict)
- Fighting the compiler (valid patterns rejected)
- Can't express all patterns (some need unsafe)
- Limited language support (mainly Rust)
- Requires rethinking data structures
- Verbose at times (lifetime annotations)

**When to use**:
- New projects in Rust
- Safety-critical systems
- When GC pauses are unacceptable
- High-performance systems programming
- Multithreaded systems (safety guarantees)

**Real-world examples**:
- **Bevy** (Rust game engine)
- **Amethyst** (Rust game engine)
- **Rust game engines** in general
- **Systems in larger engines** (experimental)

**Common patterns**:

```rust
// Pattern 1: Entity-Component with interior mutability
use std::cell::RefCell;
use std::rc::Rc;

struct Entity {
    components: Vec<Rc<RefCell<dyn Component>>>,
}

// Pattern 2: Arena allocation
struct Arena<'a> {
    data: Vec<&'a mut Object>,
}

let mut arena = Arena::new();
let obj = arena.alloc(Object::new());
// obj lifetime tied to arena

// Pattern 3: Indices instead of pointers
struct World {
    entities: Vec<Entity>,
}

struct EntityRef {
    index: usize,  // Index, not pointer
}

// Pattern 4: ECS with IDs
#[derive(Component)]
struct Position(f32, f32);

fn system(query: Query<&mut Position>) {
    for mut pos in query.iter_mut() {
        // Borrow checker ensures exclusive access
    }
}
```

## Comparison Table

| Approach | Safety | Performance | Determinism | Programmer Burden | Language Support |
|----------|--------|-------------|-------------|-------------------|------------------|
| **Manual** | ❌ Unsafe | ✅ Fastest | ✅ Yes | ❌ High | Universal |
| **Reference Counting** | 🟡 Safe* | 🟡 Moderate | ✅ Yes | 🟡 Moderate | C++, Swift, Python |
| **Garbage Collection** | ✅ Safe | ❌ Variable | ❌ No | ✅ Low | C#, Java, Go |
| **Ownership** | ✅ Safe | ✅ Fast | ✅ Yes | 🟡 Moderate | Rust (mainly) |

*Safe except for cycles

## Hybrid Approaches

Most real engines combine multiple approaches:

### Unity Example
```csharp
// C# objects: Garbage collected
class GameManager {
    List<Enemy> enemies;  // GC'd
}

// Native objects: Manual (C++ engine side)
class Texture2D {
    IntPtr nativePtr;  // C++ texture (manual)
    
    ~Texture2D() {
        DestroyNative(nativePtr);  // Explicit cleanup
    }
}
```

### Unreal Example
```cpp
// UObject: Reference counted + garbage collection
UCLASS()
class AEnemy : public AActor {
    // Automatic GC, tracked by Unreal
};

// Non-UObject: Manual or smart pointers
class FMeshData {
    TSharedPtr<FVertexBuffer> VertexBuffer;  // Ref counted
};

// Temporary allocations: Stack/arena
void Update() {
    FMemMark Mark(FMemStack::Get());  // Arena allocator
    
    float* TempData = new(FMemStack::Get()) float[1000];
    // ... use ...
    
    // Auto-freed when Mark goes out of scope
}
```

### Custom Engine Example
```cpp
// Long-lived: Reference counted
SharedPtr<Texture> texture = AssetManager::Load("texture.png");

// Short-lived: Arena
FrameAllocator frame_alloc;
void* temp = frame_alloc.Alloc(size);
// ... use during frame ...
frame_alloc.Reset();  // Free all at end of frame

// Pooled: Object pools
Enemy* enemy = enemy_pool.Allocate();
// ... use ...
enemy_pool.Free(enemy);

// Critical: Manual
void* gpu_buffer = malloc(size);
upload_to_gpu(gpu_buffer);
free(gpu_buffer);
```

## Memory Allocation Patterns

Beyond lifetime management, allocation patterns matter:

### 1. Linear/Arena Allocator
```c
struct Arena {
    void* base;
    size_t size;
    size_t offset;
};

void* arena_alloc(Arena* arena, size_t size) {
    if (arena->offset + size > arena->size) {
        return NULL;  // Out of memory
    }
    
    void* ptr = (char*)arena->base + arena->offset;
    arena->offset += size;
    return ptr;
}

void arena_reset(Arena* arena) {
    arena->offset = 0;  // Free everything at once
}
```

**Use**: Per-frame allocations, temporary data

### 2. Pool Allocator
```c
struct Pool {
    void* blocks;
    size_t block_size;
    size_t block_count;
    void* free_list;
};

void* pool_alloc(Pool* pool) {
    if (!pool->free_list) {
        return NULL;  // Pool exhausted
    }
    
    void* ptr = pool->free_list;
    pool->free_list = *(void**)ptr;  // Next in list
    return ptr;
}

void pool_free(Pool* pool, void* ptr) {
    *(void**)ptr = pool->free_list;  // Insert into list
    pool->free_list = ptr;
}
```

**Use**: Fixed-size objects (particles, entities)

### 3. Stack Allocator
```c
struct StackAllocator {
    void* base;
    size_t size;
    size_t offset;
};

typedef struct {
    size_t previous_offset;
} Marker;

Marker stack_mark(StackAllocator* stack) {
    return (Marker){ stack->offset };
}

void stack_free_to_marker(StackAllocator* stack, Marker marker) {
    stack->offset = marker.previous_offset;
}

// Usage
Marker mark = stack_mark(&allocator);
void* data = stack_alloc(&allocator, size);
// ... use data ...
stack_free_to_marker(&allocator, mark);  // Unwind
```

**Use**: Nested scope allocations, function call stacks

### 4. Double-Buffered Allocator
```c
struct DoubleBuffer {
    Arena buffers[2];
    int current;
};

void double_buffer_flip(DoubleBuffer* db) {
    arena_reset(&db->buffers[db->current]);
    db->current = 1 - db->current;
}

// Usage
void game_loop() {
    while (running) {
        void* data = arena_alloc(&double_buffer.buffers[double_buffer.current], size);
        // ... use during frame ...
        
        double_buffer_flip(&double_buffer);  // Clear previous frame
    }
}
```

**Use**: Per-frame data that needs to persist for one extra frame (GPU upload)

## Performance Considerations

### Allocation Speed

Approximate allocation costs (nanoseconds):

| Allocator | Allocation Time | Deallocation Time |
|-----------|-----------------|-------------------|
| Arena | ~5 ns | ~0 ns (batch) |
| Pool | ~10 ns | ~10 ns |
| Stack | ~5 ns | ~5 ns |
| malloc/free | ~50-200 ns | ~50-200 ns |
| new/delete (C++) | ~100-300 ns | ~100-300 ns |

**Note**: Times vary by platform, allocator implementation, fragmentation

### Cache Locality

```
# Bad: Scattered allocations (cache misses)
Enemy* enemies[1000];
for (int i = 0; i < 1000; i++) {
    enemies[i] = malloc(sizeof(Enemy));  # Random locations
}

for (int i = 0; i < 1000; i++) {
    update(enemies[i]);  # Cache miss per access!
}

# Good: Contiguous allocation (cache hits)
Enemy* enemies = malloc(sizeof(Enemy) * 1000);  # One allocation

for (int i = 0; i < 1000; i++) {
    update(&enemies[i]);  # Sequential access, cache-friendly
}
```

### Fragmentation

```
# Problem: External fragmentation
allocate(100);  # [100 bytes used]
allocate(50);   # [100 used][50 used]
free(first);    # [100 free][50 used]
allocate(75);   # Can't fit! 100-byte hole too small for 75 + alignment

# Solution 1: Pooling (fixed sizes)
# Solution 2: Arena (reset all)
# Solution 3: Compacting GC (move objects)
```

## Practical Recommendations

### For Beginners
- **Use GC language** (C#, Java) for prototyping
- Learn manual management later (C, C++)
- Don't fight the defaults

### For Performance-Critical Games
- **Manual + Pools** for hot paths
- **Reference counting** for assets
- **Arena/frame allocators** for temporary data
- Profile allocation patterns

### For Safety-Critical Systems
- **Ownership-based** (Rust) if available
- **Smart pointers** (C++) with careful design
- Static analysis tools (Valgrind, ASan)

### For AAA Engines
- **Hybrid approach** (manual + ref counting + custom allocators)
- Profile and optimize per subsystem
- Custom allocators for specific patterns

## Common Pitfalls

### Pitfall 1: Premature Optimization

**Problem**: Implementing complex memory pools before profiling

**Solution**: Start simple, measure, optimize bottlenecks

### Pitfall 2: Ignoring Allocator Choice

**Problem**: Using malloc for frame-temporary data

```c
// Bad: Allocating per frame
void update() {
    float* temp = malloc(1000 * sizeof(float));
    // ... use ...
    free(temp);  // Every frame!
}

// Good: Arena allocator
void update() {
    float* temp = arena_alloc(&frame_arena, 1000 * sizeof(float));
    // ... use ...
    // Freed in batch at end of frame
}
```

### Pitfall 3: Mixing Allocation Strategies

**Problem**: Allocating with malloc, freeing with custom allocator

**Solution**: Tag allocations with allocator type, assert on mismatch

### Pitfall 4: Not Testing Memory Safety

**Problem**: Relying on "it works on my machine"

**Solution**: 
- Run with AddressSanitizer (ASan)
- Use Valgrind
- Test on debug builds
- Enable allocator debugging (guard pages, canaries)

## Further Reading

### Books
- **The Garbage Collection Handbook** by Jones et al. - Comprehensive GC theory
- **C++ High Performance** by Andrist & Sehr - Modern C++ memory management
- **The Rust Programming Language** - Ownership system explained
- **Game Engine Architecture** by Gregory - Memory systems chapter

### Papers
- **Beltway: Getting Around Garbage Collection Gridlock** - GC for games
- **Reconsidering Custom Memory Allocation** - When custom allocators help
- **Ownership Types for Safe Programming** - Type theory foundations

### Articles
- **"Memory Management Reference"** (memorymanagement.org) - Encyclopedic
- **"Understanding Memory Management"** - EA/DICE presentations
- **"Custom Memory Allocators"** - Various GDC talks
- **"The Rust Book"** - Ownership chapters

### GDC Talks
- **"Allocator Designs"** by Eskil Steenberg
- **"Memory Management in AAA Titles"** - various studios
- **"Garbage Collection for Games"** - Unity talks

### Tools
- **AddressSanitizer (ASan)** - Detect memory errors
- **Valgrind** - Memory profiling and leak detection
- **MTuner** - Visual memory profiler
- **Instruments** - Apple's profiling tools

## Summary

Memory management is a fundamental trade-off:

- **Manual**: Maximum performance, maximum danger
- **Reference Counting**: Deterministic, can't handle cycles
- **Garbage Collection**: Convenient, unpredictable pauses
- **Ownership**: Safe and fast, but steep learning curve

Most production engines use **hybrid approaches**:
- Manual for critical systems
- Reference counting for assets
- Custom allocators (pools, arenas) for specific patterns
- Sometimes GC for scripting

**Choose based on**:
1. Language constraints
2. Performance requirements
3. Team expertise
4. Safety requirements
5. Development speed vs. runtime speed

The trend is toward safer systems (smart pointers, ownership), but manual management still dominates performance-critical code.

**Always**:
- Profile before optimizing
- Use custom allocators for specific patterns
- Test with memory debugging tools
- Design clear ownership semantics
- Document allocation/deallocation responsibilities
