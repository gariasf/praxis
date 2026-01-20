# Decision Tree: Multithreading Strategies for Game Engines

```
┌──────────────────────────────────────────────────┐
│ How should I approach parallelism in my engine? │
└──────────────────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ Using ECS?                        │
        └───────────────────────────────────┘
                /                   \
               /                     \
             Yes                      No
              │                       │
              ▼                       ▼
    ┌──────────────────┐      ┌─────────────────┐
    │ Parallel Systems │      │ More questions →│
    │ (recommended)    │      └─────────────────┘
    └──────────────────┘              │
              │                       ▼
              │          ┌───────────────────────┐
              │          │ What's your target?   │
              │          └───────────────────────┘
              │                /            \
              │               /              \
              │          Desktop          Mobile/Web
              │              │                 │
              │              ▼                 ▼
              │      ┌──────────────┐   ┌────────────┐
              │      │ Task-based   │   │ Minimal    │
              │      │ Parallelism  │   │ Threading  │
              │      └──────────────┘   └────────────┘
              │
              ▼
    ┌────────────────────────┐
    │ How many CPU cores?    │
    └────────────────────────┘
          /              \
         /                \
    2-4 cores          8+ cores
        │                  │
        ▼                  ▼
┌──────────────┐    ┌─────────────┐
│ Basic        │    │ Aggressive  │
│ Parallelism  │    │ Parallelism │
└──────────────┘    └─────────────┘
```

## Quick Decision Matrix

| Strategy | ECS | OOP | Performance | Complexity | Best For |
|----------|-----|-----|-------------|------------|----------|
| **Single-threaded** | ✅ | ✅ | ❌ | ✅ Simple | Web, mobile, prototypes |
| **Parallel Systems (ECS)** | ✅ | ❌ | ✅ Excellent | ⚠️ Moderate | ECS engines, modern CPUs |
| **Task-based** | ⚠️ | ✅ | ✅ Good | ⚠️ Moderate | OOP engines, flexible |
| **Job system** | ✅ | ✅ | ✅ Excellent | ❌ Complex | AAA engines |
| **Pipeline parallelism** | ⚠️ | ⚠️ | ⚠️ Limited | ⚠️ Moderate | Specific workloads |
| **Data parallelism** | ✅ | ⚠️ | ✅ Excellent | ✅ Simple | Bulk operations |

## Detailed Analysis

### Strategy 1: Single-Threaded (Simple)

**How it works:**
```rust
loop {
    handle_input();
    update_game_logic();
    render();
}
```

All work happens on main thread, sequentially.

#### Choose Single-Threaded If:

**✅ High Priority:**
- **Prototyping** (get something working fast)
- **Mobile/Web** target (limited cores, battery concerns)
- **Small games** (performance not critical)
- **Learning project** (avoid threading complexity)
- Team **inexperienced** with threading

**Example Use Cases:**
- Web games (single-threaded JS limitation)
- Simple mobile games
- Prototypes and game jams
- Educational projects

**Pros:**
- **Simplicity**: No threading bugs (race conditions, deadlocks)
- **Debugging**: Easy to trace execution
- **Deterministic**: Predictable behavior
- **Fast development**: No synchronization overhead
- **Web compatible**: Works in WASM/JavaScript

**Cons:**
- **Poor CPU utilization**: 1 core used, others idle
- **Frame drops**: Heavy work blocks rendering
- **Scalability**: Doesn't benefit from more cores
- **Asset loading**: Blocks on I/O

**Example:**
```rust
fn main() {
    let mut world = World::new();
    
    loop {
        // Everything sequential
        input_system(&mut world);
        physics_system(&mut world);
        animation_system(&mut world);
        render_system(&mut world);
    }
}
```

**When to upgrade:**
- Frame time consistently high (>16ms for 60 FPS)
- Asset loading causes stuttering
- Target platform has 4+ cores
- Performance becomes priority

### Strategy 2: Parallel Systems (ECS - Praxis Approach)

**How it works:**
```rust
// Systems declare data dependencies via queries
// Scheduler runs independent systems in parallel

// These can run in parallel (no shared data):
fn physics_system(query: Query<(&mut Position, &Velocity)>);
fn animation_system(query: Query<(&mut AnimationState, &Skeleton)>);

// Scheduler detects independence and parallelizes
```

#### Choose Parallel Systems If:

**✅ High Priority (Praxis uses this):**
- Using **ECS architecture**
- Target has **4+ cores** (desktop, console)
- Need **automatic parallelization**
- Want **clean separation** of concerns
- Many **independent systems**

**Example Use Cases:**
- Modern game engines (Bevy, Praxis)
- Large-scale simulations
- Desktop/console games
- Data-oriented engines

**Pros:**
- **Automatic parallelization**: Scheduler handles it
- **Safety**: Compile-time data race prevention (Rust)
- **Scalability**: Utilizes available cores automatically
- **Clean code**: Systems are independent functions
- **Composability**: Easy to add/remove systems
- **Cache-friendly**: Systems process component arrays

**Cons:**
- **ECS required**: Doesn't work with OOP
- **Granularity**: Need many small systems for best parallelism
- **System ordering**: Must handle dependencies
- **Scheduling overhead**: Small cost for scheduling
- **Learning curve**: ECS paradigm unfamiliar to many

**Praxis Implementation (using bevy_ecs):**
```rust
use bevy_ecs::prelude::*;

// Systems with non-overlapping queries run in parallel
fn velocity_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0;
    }
}

fn animation_system(mut query: Query<(&mut AnimationState, &Skeleton)>) {
    for (mut state, skeleton) in query.iter_mut() {
        state.advance(skeleton);
    }
}

// Add to schedule
Schedule::default()
    .add_systems((
        velocity_system,
        animation_system, // Runs in parallel with velocity_system!
    ));
```

**How bevy_ecs parallelizes:**
1. **Analyze queries**: Determine which systems access which components
2. **Build dependency graph**: Find conflicts (mutable access to same data)
3. **Schedule**: Run non-conflicting systems in parallel
4. **Execute**: Use thread pool to run systems

**Performance:**
```
Single-threaded: 100 systems × 0.1ms = 10ms
Parallel (4 cores): 100 systems / 4 = 2.5ms (4x speedup)
Parallel (8 cores): 100 systems / 8 = 1.25ms (8x speedup)
```

**Limitations:**
- Systems accessing same components must run sequentially
- Very small systems (<0.01ms) have scheduling overhead
- Need sufficient parallelizable work

### Strategy 3: Task-Based Parallelism

**How it works:**
```rust
// Manually spawn tasks for specific work
let task1 = thread_pool.spawn(|| heavy_computation_1());
let task2 = thread_pool.spawn(|| heavy_computation_2());

let result1 = task1.await;
let result2 = task2.await;
```

#### Choose Task-Based If:

**✅ High Priority:**
- **OOP architecture** (not ECS)
- Need **fine-grained control** over threading
- **Specific hotspots** to parallelize
- Using **C++** (common pattern)
- Want **gradual parallelization**

**Example Use Cases:**
- Unity classic (not DOTS)
- Unreal Engine
- Traditional OOP engines
- Hybrid approaches

**Pros:**
- **Flexibility**: Parallelize exactly what you want
- **OOP-friendly**: Works with traditional architectures
- **Incremental**: Add parallelism gradually
- **Control**: Explicit task boundaries
- **Proven**: Well-understood pattern

**Cons:**
- **Manual**: Must identify and spawn tasks yourself
- **Synchronization**: Manual barriers and waits
- **Data races**: Easier to introduce bugs
- **Overhead**: Task spawn/join cost
- **Complexity**: More code than sequential

**Example (Rust with rayon):**
```rust
use rayon::prelude::*;

// Parallelize collection processing
let results: Vec<_> = items
    .par_iter() // Parallel iterator
    .map(|item| process(item))
    .collect();

// Or explicit tasks
let (result1, result2) = rayon::join(
    || heavy_work_1(),
    || heavy_work_2(),
);
```

**Example (C++ with threading):**
```cpp
#include <thread>
#include <future>

// Spawn tasks
auto future1 = std::async(std::launch::async, heavy_work_1);
auto future2 = std::async(std::launch::async, heavy_work_2);

// Wait for results
auto result1 = future1.get();
auto result2 = future2.get();
```

**Common patterns:**
```rust
// Pattern 1: Parallel collections
entities.par_iter_mut().for_each(|entity| {
    entity.update();
});

// Pattern 2: Fork-join
let (physics, animations) = rayon::join(
    || update_physics(&mut world),
    || update_animations(&mut world), // Must not alias world!
);

// Pattern 3: Task graph
let task_a = pool.spawn(|| work_a());
let task_b = pool.spawn(|| work_b());
let task_c = pool.spawn(move || {
    let a_result = task_a.await;
    let b_result = task_b.await;
    combine(a_result, b_result)
});
```

### Strategy 4: Job System (AAA Engines)

**How it works:**
```
Job Queue: [JobA, JobB, JobC, JobD, ...]
                   ↓
          ┌────────────────┐
          │  Worker Threads │
          │  (one per core) │
          └────────────────┘
               ↓
          Pick jobs, execute, return
```

Jobs are small units of work, threads pull from queue.

#### Choose Job System If:

**✅ High Priority:**
- **AAA production** (need maximum performance)
- **Heterogeneous workloads** (CPU + GPU tasks)
- Need **load balancing** (work stealing)
- Building **professional engine**
- Have **engineering resources**

**Example Use Cases:**
- Unreal Engine (task graph)
- Frostbite Engine
- id Tech engines
- AAA studio engines

**Pros:**
- **Excellent load balancing**: Work stealing prevents idle threads
- **Scalability**: Automatically uses all cores
- **Fine-grained**: Jobs can be very small
- **Composability**: Jobs spawn sub-jobs
- **Flexibility**: Mix different job types

**Cons:**
- **Complexity**: Significant implementation effort
- **Debugging**: Hard to trace job execution
- **Overhead**: Job spawn/management cost
- **Data races**: Manual synchronization needed
- **Engineering time**: Weeks to months to implement

**Conceptual Implementation:**
```rust
struct Job {
    work: Box<dyn FnOnce() + Send>,
    dependencies: Vec<JobHandle>,
}

struct JobSystem {
    queue: Arc<Mutex<VecDeque<Job>>>,
    workers: Vec<JoinHandle<()>>,
}

impl JobSystem {
    fn spawn_job(&self, job: Job) -> JobHandle {
        self.queue.lock().unwrap().push_back(job);
        // Handle returned
    }
    
    fn worker_thread(queue: Arc<Mutex<VecDeque<Job>>>) {
        loop {
            let job = queue.lock().unwrap().pop_front();
            if let Some(job) = job {
                (job.work)(); // Execute
            } else {
                // Work stealing: try to steal from other workers
            }
        }
    }
}
```

**Advanced features:**
- **Work stealing**: Idle workers steal from busy workers
- **Priority queues**: High-priority jobs first
- **Affinity**: Pin jobs to specific cores
- **Job graphs**: Complex dependency management

**Effort estimate:**
- Basic job system: 2-4 weeks
- Production-quality: 3-6 months
- Advanced features: Ongoing

**Praxis decision:** Bevy_ecs provides similar benefits with less complexity.

### Strategy 5: Pipeline Parallelism

**How it works:**
```
Thread 1: Input → Physics → Animation → ...
Thread 2:         Input' → Physics' → Animation' → ...
Thread 3:                  Input'' → Physics'' → ...

Each frame flows through pipeline stages on different threads
```

#### Choose Pipeline Parallelism If:

**✅ High Priority:**
- Have **clear pipeline stages** (input → update → render)
- Stages have **similar duration**
- **Latency acceptable** (1-2 frame delay)
- Desktop/console target

**Example Use Cases:**
- Traditional game engines
- Rendering pipelines
- Video processing

**Pros:**
- **Continuous work**: All threads always busy
- **Predictable**: Fixed pipeline structure
- **Lower synchronization**: Only between stages
- **Throughput**: High when balanced

**Cons:**
- **Latency**: Results delayed by pipeline depth
- **Load balancing**: Slowest stage limits entire pipeline
- **Rigid**: Hard to add stages
- **Underutilization**: If stages have different durations
- **Complexity**: State management between stages

**Example:**
```rust
// Frame N on thread 1
// Frame N-1 on thread 2
// Frame N-2 on thread 3

struct Pipeline {
    input_thread: JoinHandle<()>,
    update_thread: JoinHandle<()>,
    render_thread: JoinHandle<()>,
}

// Data flows through channels
let (input_tx, input_rx) = channel();
let (update_tx, update_rx) = channel();

// Input thread
thread::spawn(move || loop {
    let input = gather_input();
    input_tx.send(input);
});

// Update thread
thread::spawn(move || loop {
    let input = input_rx.recv();
    let state = update_game(input);
    update_tx.send(state);
});

// Render thread
thread::spawn(move || loop {
    let state = update_rx.recv();
    render(state);
});
```

**Problem: Load imbalance:**
```
Input:  5ms  ████▌
Update: 10ms █████████
Render: 3ms  ███

Pipeline limited by slowest stage (10ms)
Other stages waste time waiting
```

### Strategy 6: Data Parallelism (SIMD + Multi-threading)

**How it works:**
```rust
// Process arrays of data in parallel
positions.par_iter_mut()
    .zip(velocities.par_iter())
    .for_each(|(pos, vel)| {
        *pos += *vel; // SIMD within thread, multiple threads
    });
```

#### Choose Data Parallelism If:

**✅ High Priority:**
- Processing **large arrays** (thousands+ items)
- **Uniform operations** (same work per item)
- **ECS architecture** (component arrays)
- Performance critical (physics, particles)

**Example Use Cases:**
- Particle systems (millions of particles)
- Physics simulation (thousands of rigid bodies)
- Animation (many skeletal meshes)
- AI (pathfinding for many agents)

**Pros:**
- **Scalability**: Scales with data and cores
- **Predictable**: Regular access patterns
- **Cache-friendly**: Sequential memory access
- **Simple**: Conceptually straightforward
- **Effective**: Often highest speedup

**Cons:**
- **Requires arrays**: Doesn't fit all problems
- **Load balancing**: If items have different work amounts
- **Dependencies**: Can't parallelize dependent operations

**Praxis Example:**
```rust
// ECS systems naturally data-parallel
fn particle_system(mut query: Query<(&mut Position, &Velocity)>) {
    // bevy_ecs parallelizes this automatically
    query.par_iter_mut().for_each(|(mut pos, vel)| {
        pos.0 += vel.0;
    });
}
```

**Performance:**
```
Single-threaded: 100,000 particles × 10 ops = 10ms
Data parallel (4 cores): 100,000 / 4 = 2.5ms
Data parallel + SIMD: 100,000 / 16 = 0.625ms (16x speedup!)
```

## Hybrid Approach: Combining Strategies

**Most engines use multiple strategies:**

### Praxis Approach

```rust
// 1. Parallel systems (ECS)
Schedule::default()
    .add_systems((
        input_system,
        physics_system,    // Parallel with animation_system
        animation_system,  // Parallel with physics_system
    ))
    .add_systems(render_system.after(physics_system));

// 2. Data parallelism within systems
fn particle_system(mut query: Query<(&mut Position, &Velocity)>) {
    query.par_iter_mut().for_each(|(mut pos, vel)| {
        pos.0 += vel.0; // Also SIMD optimized
    });
}

// 3. Async asset loading (separate threads)
async fn load_model(path: &str) -> Model {
    tokio::fs::read(path).await // I/O on thread pool
}
```

**Three levels of parallelism:**
1. **System level**: Independent systems run in parallel
2. **Data level**: Within systems, process entities in parallel
3. **I/O level**: Async tasks for asset loading

### Unity DOTS

```csharp
// Parallel jobs for ECS
[BurstCompile]
struct VelocityJob : IJobForEach<Position, Velocity>
{
    public void Execute(ref Position pos, ref Velocity vel)
    {
        pos.Value += vel.Value; // Burst-compiled, SIMD, multi-threaded
    }
}
```

### Unreal Engine

```cpp
// Task graph for complex dependencies
FGraphEventRef TaskA = FFunctionGraphTask::CreateAndDispatchWhenReady(
    []() { /* work A */ }
);

FGraphEventRef TaskB = FFunctionGraphTask::CreateAndDispatchWhenReady(
    []() { /* work B */ },
    nullptr,
    TaskA  // Depends on A
);
```

## Platform Considerations

### Desktop (8+ cores)

**Recommendation: Aggressive parallelism**

Modern desktop CPUs have many cores:
- Intel i9: 16-24 cores
- AMD Ryzen: 12-16 cores
- Apple M series: 8-12 cores

**Strategy:**
- Parallel systems (ECS)
- Data parallelism within systems
- Async I/O
- Utilize all cores

### Console (PS5, Xbox Series X)

**Recommendation: Parallel systems + job system**

Fixed hardware allows optimization:
- 8 Zen 2 cores (16 threads)
- Known performance characteristics
- Can tune precisely

**Strategy:**
- Parallel systems for gameplay
- Job system for rendering
- Reserve cores for specific tasks (physics, audio)

### Mobile (4-8 cores)

**Recommendation: Conservative parallelism**

Mobile constraints:
- Battery life (power consumption)
- Thermal throttling (heat)
- Weaker cores (big.LITTLE)

**Strategy:**
- Minimal threading (2-4 threads)
- Async I/O only
- Prefer single-threaded when possible
- Monitor battery/thermal

### Web (WASM)

**Recommendation: Single-threaded + Web Workers (limited)**

Web constraints:
- SharedArrayBuffer limited availability
- Web Workers have overhead
- JavaScript single-threaded

**Strategy:**
- Single-threaded main game loop
- Offload heavy tasks to Web Workers (pathfinding, asset loading)
- Prepare for single-core performance

## Common Pitfalls

### Pitfall 1: Over-threading

```rust
// Bad: Too many threads, too little work
for item in items {
    thread::spawn(move || process(item)); // Overhead > benefit
}

// Good: Batch work
items.par_iter().for_each(|item| process(item)); // Thread pool
```

**Rule of thumb:** Work per thread should be > 1ms to overcome overhead.

### Pitfall 2: False Sharing

```rust
// Bad: Adjacent data written by different threads
struct GameState {
    player1_score: AtomicU32, // Cache line 1
    player2_score: AtomicU32, // Cache line 1 (false sharing!)
}

// Good: Separate cache lines
#[repr(align(64))] // Cache line size
struct Aligned<T>(T);

struct GameState {
    player1_score: Aligned<AtomicU32>, // Cache line 1
    player2_score: Aligned<AtomicU32>, // Cache line 2
}
```

**Impact:** 10-100x slowdown from cache coherency traffic.

### Pitfall 3: Unbalanced Work

```rust
// Bad: Some threads finish early
thread1: ████████████████████ 20ms
thread2: ████ 4ms (idle 16ms)
thread3: ████████ 8ms (idle 12ms)

// Frame time still 20ms!

// Good: Work stealing or dynamic scheduling
```

### Pitfall 4: Excessive Synchronization

```rust
// Bad: Lock for every operation
for item in items {
    lock.lock();
    process(item);
    lock.unlock(); // Lock overhead kills performance
}

// Good: Batch operations
let batch = items.collect();
lock.lock();
for item in batch {
    process(item);
}
lock.unlock();
```

### Pitfall 5: Race Conditions

```rust
// Bad: Unsynchronized shared access
static mut COUNTER: u32 = 0;

// Thread 1
unsafe { COUNTER += 1; }

// Thread 2
unsafe { COUNTER += 1; } // Race! Might both read 0, write 1

// Good: Atomic or lock
static COUNTER: AtomicU32 = AtomicU32::new(0);
COUNTER.fetch_add(1, Ordering::Relaxed); // Safe
```

## Language-Specific Guidance

### Rust
**Recommendation: Parallel systems (ECS)**

Rust's strengths:
- **Compile-time safety**: Data races impossible
- **Zero-cost**: No runtime overhead
- **rayon**: Easy data parallelism
- **async**: Great I/O parallelism

```rust
// Rust prevents data races at compile time
fn system1(mut query: Query<&mut ComponentA>) {} // Mutable access
fn system2(query: Query<&ComponentA>) {}         // Immutable access

// This won't compile - can't run in parallel
// Scheduler detects conflict automatically
```

### C++
**Recommendation: Task-based or job system**

C++ flexibility:
- **std::thread**: Manual threading
- **std::async**: Task-based
- **TBB**: Intel threading library
- **Unreal/Unity**: Engine-specific

```cpp
// C++ requires manual care
std::mutex mtx;
void thread_safe_update() {
    std::lock_guard<std::mutex> lock(mtx); // Manual
    // Update shared state
}
```

**Warning:** Easy to introduce data races in C++.

### C#
**Recommendation: Task Parallel Library (TPL) or Unity Jobs**

C# options:
- **TPL**: Task.Run(), Parallel.For()
- **Unity Jobs**: Burst-compiled jobs
- **async/await**: I/O parallelism

```csharp
// TPL for general parallelism
Parallel.For(0, count, i => {
    Process(items[i]);
});

// Unity Jobs for performance
[BurstCompile]
struct MyJob : IJobParallelFor {
    public void Execute(int index) { /* work */ }
}
```

## Performance Expectations

### Single-threaded Baseline
```
Frame time: 16ms (60 FPS target)
Input:      1ms
Update:     10ms
Render:     5ms
Total:      16ms (just hitting target)
```

### With Parallel Systems (4 cores)
```
Frame time: 6ms (166 FPS!)
Input:      1ms (sequential)
Update:     3ms (parallel, 10/4 + overhead)
Render:     2ms (parallel where possible)
Total:      6ms (2.7x speedup)
```

### With Parallel Systems (8 cores)
```
Frame time: 4ms (250 FPS!)
Update:     2ms (parallel, 10/8 + overhead)
Total:      4ms (4x speedup)
```

**Realistic speedup:**
- 4 cores: 2-3x (not perfect 4x due to dependencies, overhead)
- 8 cores: 3-5x
- 16 cores: 4-8x (diminishing returns)

## Decision Checklist

| Question | Single | Parallel Systems | Task-Based | Job System |
|----------|--------|------------------|------------|------------|
| Using ECS? | ✓ | ✓ | | ✓ |
| Using OOP? | ✓ | | ✓ | ✓ |
| 2-4 cores? | ✓ | ⚠️ | ⚠️ | |
| 8+ cores? | | ✓ | ✓ | ✓ |
| Mobile/Web? | ✓ | | | |
| Desktop/Console? | | ✓ | ✓ | ✓ |
| Simple project? | ✓ | | | |
| AAA production? | | ⚠️ | ✓ | ✓ |
| Learning project? | ✓ | ⚠️ | | |

## Recommended Reading

- **General:**
  - *C++ Concurrency in Action* by Anthony Williams
  - *The Art of Multiprocessor Programming* by Herlihy & Shavit

- **Game-Specific:**
  - [Parallelizing the Naughty Dog Engine](https://www.gdcvault.com/play/1022186/Parallelizing-the-Naughty-Dog-Engine)
  - [Destiny's Multithreaded Rendering](https://www.gdcvault.com/play/1021926/Destiny-s-Multithreaded-Rendering)

- **ECS Parallelism:**
  - [Bevy ECS parallelism](https://bevyengine.org/learn/book/getting-started/ecs/)
  - Praxis: `docs/concepts/ecs.md`

## Conclusion

**TL;DR:**
- **Web/mobile? → Single-threaded (+ async I/O)**
- **ECS on desktop? → Parallel systems (Praxis approach)**
- **OOP on desktop? → Task-based parallelism**
- **AAA production? → Job system**
- **Learning? → Start single-threaded, add parallelism when needed**

**Praxis Choice: Parallel Systems via bevy_ecs**

Reasons:
1. **ECS architecture**: Natural fit for parallel systems
2. **Safety**: Rust prevents data races at compile time
3. **Automatic**: Scheduler handles parallelization
4. **Scalable**: Utilizes available cores automatically
5. **Educational**: Demonstrates modern engine architecture

**How to start:**
1. **Begin single-threaded**: Get game working
2. **Profile**: Find bottlenecks
3. **Add parallelism**: Where it matters most
4. **Measure**: Verify improvements

**Don't optimize prematurely!** Single-threaded is fine until you prove you need more performance.
