# Phase 5: Build the ECS

## Context

Phase 4 is done. Three hardcoded `glam::Mat4` transforms plus a loaded
`DamagedHelmet.glb` get rendered by `State::render()` via a nested loop over
`self.transforms` and `self.primitives`. Camera, lights, model uniforms, and
GPU buffers all live directly as fields on `State`.

Time to replace the ad-hoc scene data with a real ECS. Goal of this phase:
the three helmets are still on screen at the end, but their `Transform` +
`MeshHandle` + `MaterialHandle` now come from an ECS `World` query instead of
hardcoded `Vec`s on `State`.

## Architecture decisions (agreed)

Shape is adapted from the Pikuma-style sparse-set ECS (see
`gariasf/2d-engine`), with four Rust-shaped changes.

| Decision | Choice | Why |
|---|---|---|
| Storage | Sparse set: `Pool<T>` = dense `Vec<T>` + sparse index maps | Simpler than archetype; fine iteration for this scale |
| Entity ID | Generational `{ index: u32, generation: u32 }` | Stale handles fail cleanly instead of silently aliasing reused slots |
| Component ID | `std::any::TypeId` keyed into `HashMap<TypeId, Box<dyn ErasedPool>>` | Rust-native; no fragile `static nextId++` counter |
| Resource pools | Outside `World`, owned separately (on `State` or a `Resources` struct) | ECS holds game data; wgpu resources are engine data, kept decoupled |

## Gotchas

### Borrow checker vs ergonomic API

The Pikuma `entity.AddComponent<T>()` pattern needs a back-pointer from
`Entity` to the registry. In Rust that's a borrow-checker minefield. Don't
port it. Always go through the world: `world.add_component::<Transform>(e, t)`.

### Iteration invalidation

Systems can spawn and kill entities during their own `update()`. Mutating
storage mid-iteration is UB in C++ and a panic in Rust. Buffer spawns and
kills into `pending_spawn` / `pending_kill` queues on the world and flush them
at the top of each frame, before systems run.

### Type-erased storage

`Pool<T>` is a concrete type, but `World` stores them as `Box<dyn ErasedPool>`.
The `ErasedPool` trait must only expose methods that don't mention `T` (e.g.
`fn remove(&mut self, e: EntityId)`, `fn as_any_mut(&mut self) -> &mut dyn Any`).
Typed access goes through `Any::downcast_mut::<Pool<T>>()`. Plan for this in
Step 3 — it shapes the whole storage API.

### `vec3<f32>` uniform trap (reminder)

No new uniforms are expected in this phase, but if one slips in: no
`vec3<f32>` in WGSL uniform structs. Use `vec4<f32>` or an explicit pad field.

---

## Mental models

Cross-cutting concepts. Worth reading once before Step 1 — they recur in
almost every step.

### Acronyms and vocabulary

- **ECS** — Entity Component System. Architectural pattern: entities are
  bare IDs, components are pure data, systems are functions that query for
  component combinations and act on them. Replaces deep OOP inheritance.
- **GPU** — Graphics Processing Unit.
- **UB** — Undefined Behavior. C/C++ term for "code violates language
  rules, so the compiler is allowed to assume it never happens". Crashes,
  silent corruption, or "the program runs fine until you ship" all count.
  Rust's borrow checker exists to make most UB impossible to write.
- **ABA bug** — A slot holds value A, gets freed, gets reused for a new
  value that also looks like A. Old reference still passes equality checks
  but points at the wrong thing. Generational IDs prevent it by bumping a
  counter on free.
- **API** — Application Programming Interface. The shape callers see.
- **WGSL** — WebGPU Shading Language. The shader language wgpu accepts.
- **glTF / `.glb`** — GL Transmission Format. JSON-based 3D asset format.
  `.glb` is the binary-packed variant (single file).
- **`TypeId`** — `std::any::TypeId`. Rust's runtime identifier for a type.
  `TypeId::of::<Transform>()` returns an opaque value that uniquely
  identifies `Transform` and can be compared at runtime.
- **Pikuma** — Online instructor (Gustavo Pezzi). His C++ ECS course is
  the structural model this phase adapts to Rust. Reference repo:
  `gariasf/2d-engine`.

### Storage shape: sparse set vs archetype

Two main ways an ECS lays out component data. Phase 5 picks sparse set;
this section explains why, what the choice costs if Praxis grows into a
real game over the next several years, and what to keep clean now so the
door stays open.

**Sparse set** (this phase). One pool per component type. Each pool holds
a dense `Vec<T>` for fast iteration plus sparse maps that answer "where
does entity `e`'s `T` live in this pool?".

- Pros: simple to write; cheap insert and remove; component types stay
  independent.
- Cons: multi-component queries must intersect pools, which means
  per-entity HashMap lookups across N pools. Cache-unfriendly. Adding
  or removing a component is cheap (one pool touched), but iteration
  pays the cost.

**Archetype** (Bevy, EnTT default, flecs). Group entities by their
*exact* component set. All entities with `(Transform, Velocity)` live in
one archetype, all with `(Transform, Velocity, Health)` in another. A
query is one tight loop per matching archetype.

- Pros: blazing fast multi-component queries; great cache locality;
  SIMD-friendly. Iteration dominates the budget in a real game, and
  iteration is what archetype is good at.
- Cons: harder to write correctly (component graph, archetype migrations
  on add/remove, free-listing of empty archetypes); larger code surface.

#### Why sparse set now

The phase goal is **understanding**, not performance. Sparse set is
roughly half the code of archetype, and the trickiest parts (type
erasure, generational IDs, swap-remove) are shared between both.
Writing archetype first would mean fighting two unfamiliar patterns at
once; writing sparse set means fighting one, then graduating.

At the end of Phase 5 the scene has maybe 10–100 entities. At that
scale, either storage strategy completes its iteration in microseconds.
The choice is invisible.

#### When the choice starts to matter

Rough crossover, with the usual "depends on the systems" caveat:

- **Under 1,000 entities:** invisible. Either works.
- **1,000–10,000 entities:** sparse set's per-entity HashMap lookups in
  multi-component queries start showing up in profiles. Workable if
  queries are infrequent or per-entity work is heavy.
- **10,000–100,000 entities:** archetype is meaningfully faster on the
  hot loops. Sparse set is still shippable, but you'll feel it.
- **100,000+ entities:** archetype is the sane default. Sparse set needs
  bespoke optimization to keep up.

Skyrim, as a yardstick, sits in the "hundreds to a few thousand active
entities per cell" zone (NPCs, items, projectiles, lights, particle
systems). That puts a Skyrim-shape Praxis in the "either works" or
"archetype helps a bit" zone — not the "must have archetype" zone.

If gameplay ever turns swarmy (Total-War-style battles, simulation-heavy
ecology, dense projectile patterns), the bracket jumps and archetype
becomes the obvious choice.

#### What it would take to switch

Storage is meant to be swappable. If the `World` API stays clean —
`world.spawn`, `world.add_component::<T>`, `world.query::<(A, B)>()` —
the storage layer can be rewritten without touching system code.
Systems are 95%+ of an ECS-shaped codebase; storage is the small core.

A storage rewrite from sparse set to archetype is on the order of 1–2
weeks of focused work for someone who already knows how archetype
internals work. The public API and the systems on top of it stay
intact.

To preserve this option, the rule is simple: **never reach into pools
or `World` internals from outside the ECS module**. Game code calls
`world.query`, never `world.pools.get_mut(...)`. Iteration order is
*never* assumed to mean anything — sparse set's "insertion-ish, modulo
swap-removes" and archetype's "per-archetype" are both unstable;
depending on either is a latent bug.

#### Realistic exit paths

Five to ten years out, three plausible directions:

1. **Hand-rolled archetype ECS.** Same skill exercise as Phase 5, but
   harder. Worth it if rendering-engine learning is still the point of
   the project at that point.
2. **Drop in `bevy_ecs`** (the storage crate, separable from the full
   Bevy engine) or `hecs`. Production-grade archetype ECS,
   multi-threaded, mature, heavily benchmarked. Trades "I wrote every
   line" for "I can ship". This is what most serious solo Rust gamedevs
   do once their project gets real.
3. **Stick with sparse set.** If the game never crosses ~10k active
   entities — plausible for a Skyrim-shape RPG with chunked streaming
   and aggressive culling — sparse set may simply never need replacing.

The fact that all three remain open is the point. The Phase-5 choice is
reversible.

#### The bigger long-term risks (not storage)

If Praxis ever heads toward production scale, the limits worth worrying
about most are *not* sparse-vs-archetype. They are:

- **Single-threaded ECS.** `World` exposes one `&mut` at a time and
  systems run sequentially. Modern engines run dozens of systems in
  parallel across cores via a *scheduler* that builds a dependency
  graph from each system's declared component reads and writes.
  Retrofitting parallelism onto an ECS that assumed single-threaded
  mutation is invasive — easily a multi-month rewrite. Bevy and flecs
  both have this; hand-rolling it is a project of its own.
- **No system scheduler.** Phase 5 calls systems in a hardcoded order
  inside `State::update`. At 10 systems, fine. At 100 systems with
  cross-system dependencies, humans get the order wrong. Real engines
  let systems declare their reads/writes and let a scheduler compute
  the order (and which systems can run in parallel). Adding this later
  is a bigger refactor than swapping storage.
- **Materialized query results.** The Phase-5 sidestep for the
  joint-borrow trap is "collect matching `EntityId`s into a `Vec`".
  Real ECS queries return lazy iterators. If system code comes to
  depend on the Vec shape (`.len()`, indexing, random access), changing
  the query return type later breaks every callsite. Treat the Vec as
  iteration-only from day one.

These three are where production lock-in actually hides. Sparse-set is
recoverable. An ECS that bakes single-threaded sequential mutation into
every system is not.

### Generational entity IDs

An `EntityId` is a struct of two `u32`s: an `index` into the allocator's
storage, and a `generation` counter that says how many times that slot has
been reused.

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EntityId {
    pub index: u32,
    pub generation: u32,
}
```

The allocator owns:

```rust
pub struct Entities {
    generations: Vec<u32>,   // generations[i] = current generation of slot i
    free: Vec<u32>,          // slot indices ready to reuse
}
```

Behavior:

- **Spawn:** if `free` is non-empty, pop an index off it; otherwise push a
  fresh slot to `generations` (starting at 0). Return
  `EntityId { index, generation: generations[index] }`.
- **Despawn:** `generations[id.index] += 1; free.push(id.index);`. The
  bump invalidates every old handle to that slot.
- **`is_alive(id)`:** `generations[id.index as usize] == id.generation`.
  One array read, one compare.

Concrete trace:

- Spawn → `{ index: 7, generation: 0 }`. `generations[7] = 0`.
- Despawn that → `generations[7] = 1`, `free = [..., 7]`.
- Spawn again → pops 7, returns `{ index: 7, generation: 1 }`.
- Old handle `{ 7, 0 }` now fails `is_alive` because `generations[7] != 0`.

Without the generation field, the new entity at slot 7 silently aliases
the old one — that is the ABA bug. With it, mismatches cost one `u32`
compare to detect.

### Sparse set storage

For each component type `T`, the pool stores three things:

```rust
pub struct Pool<T> {
    data: Vec<T>,                                // dense, contiguous
    entity_to_index: HashMap<EntityId, usize>,   // forward map
    index_to_entity: HashMap<usize, EntityId>,   // reverse map (for swap-remove)
}
```

`data` iterates at array speed. The two maps are the "sparse" side — they
let you go from an entity to its component slot and back.

**Insert (`set(e, value)`):**

```
push value onto data
i = data.len() - 1
entity_to_index[e] = i
index_to_entity[i] = e
```

**Swap-and-pop remove (`remove(e)`):** the trick that keeps `data`
contiguous in O(1).

```
i    = entity_to_index[e]
last = data.len() - 1
if i != last:
    data[i] = data[last]                  // overwrite removed slot with last element
    moved   = index_to_entity[last]
    entity_to_index[moved] = i            // moved entity now lives at i
    index_to_entity[i]     = moved
data.pop()                                 // drop the now-duplicate last slot
remove e from both maps
```

Cost per remove: one copy plus a few map updates. Iteration order is
**not** insertion order after any remove — fine for ECS, where systems do
not care about order.

`HashMap` is fine at this scale. If profiling ever shows this hot, swap
the sparse side for `Vec<Option<usize>>` indexed by `entity.index`. Don't
pre-optimize.

### Signature bitsets

Each entity carries a `u64` (or `u128` once component count grows). Bit
`i` is set iff the entity has a component whose type maps to bit `i`. Each
query has the same shape.

```rust
fn matches(entity_sig: u64, query_sig: u64) -> bool {
    (entity_sig & query_sig) == query_sig
}
```

One AND, one compare per entity. Fast enough that the bottleneck is the
surrounding work, not the test.

The mapping `TypeId → bit position` is built lazily: keep
`HashMap<TypeId, u32>` starting empty. First time `Transform` is
registered it claims bit 0; first time `Velocity` is registered, bit 1;
and so on. Same effect as Pikuma's `static int nextId++`, just explicit
and Rust-shaped.

Cap: once you exceed 64 component types, switch storage to `u128` or
`[u64; N]`. Phase 5 won't get close.

### Type-erased component storage

The trickiest Rust pattern in the phase. Worth reading slowly.

**The problem.** `World` wants one map holding pools for many different
component types. The natural attempt:

```rust
HashMap<TypeId, Pool<T>>   // does NOT compile
```

doesn't compile because `T` would have to be one specific type for the
whole map. Rust's generics are *monomorphized* at compile time —
`Pool<Transform>` and `Pool<Velocity>` are separate compiled types with
different sizes and method addresses. A homogeneous container can't hold
both directly.

**The fix: trait objects.** A `dyn Trait` is a runtime-polymorphic
reference: a fat pointer (data pointer + vtable pointer) that can stand in
for any type implementing `Trait`. It hides the concrete type behind a
uniform interface.

For type-erased pools, the trait must only expose methods whose signatures
don't mention `T`:

```rust
use std::any::Any;

pub trait ErasedPool: Any {
    fn remove(&mut self, entity: EntityId);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

Why those three?

- `remove` — `World::despawn(e)` needs to call `remove` on every pool
  without knowing what each pool holds. The signature has no `T`, so it
  survives erasure.
- `as_any` / `as_any_mut` — the escape hatch back to typed access. They
  hand out `&dyn Any`, which can be downcast to recover the concrete
  `Pool<T>` when you do know `T`.

Then `World` stores:

```rust
pools: HashMap<TypeId, Box<dyn ErasedPool>>
```

`Box<dyn ErasedPool>` reads as "a heap-allocated thing that implements
`ErasedPool`; I do not care which concrete type". The compiler doesn't
need to know.

**Recovering the type (downcasting):**

```rust
fn get_pool_mut<T: 'static>(&mut self) -> Option<&mut Pool<T>> {
    let erased = self.pools.get_mut(&TypeId::of::<T>())?;
    erased.as_any_mut().downcast_mut::<Pool<T>>()
}
```

`downcast_mut::<Pool<T>>()` is one runtime check: "is the `TypeId` stored
inside this `Any` equal to `TypeId::of::<Pool<T>>()`?". If yes, hand back
`&mut Pool<T>`; if no, return `None`. O(1).

**Why the `'static` bound?** `TypeId::of::<T>()` requires `T: 'static`.
The reason: a type that borrows something (`&'a Foo`) is technically a
different type for each lifetime, so there'd be no single global ID.
Components own their data outright, so they're `'static` already; adding
the bound documents and enforces this.

**The joint-borrow trap.** This compiles:

```rust
let pool_a = world.get_pool_mut::<A>()?;
// done with pool_a before the next line
let pool_b = world.get_pool_mut::<B>()?;
```

This does **not** compile when both borrows must live at once:

```rust
let pool_a = world.get_pool_mut::<A>()?;
let pool_b = world.get_pool_mut::<B>()?;   // error: cannot borrow `world.pools` mutably twice
// ... use both at the same time
```

The borrow checker can't prove the two lookups hit different `HashMap`
entries — both calls borrow `&mut self.pools`.

Phase-5 sidestep: queries collect matching `EntityId`s first, then the
per-entity loop looks pools up one at a time. Slow, but borrow-clean and
ships the phase. A real joint iterator (returning `(&T1, &T2)` tuples
without the intermediate `Vec`) is a future cleanup.

### Handles as lightweight references

A `MeshHandle(u32)` is a plain integer index into a pool that owns the
real `Mesh`. Many entities can share one mesh by copying the handle. No
`Arc`, no lifetimes, no borrow-checker pressure on game code.

```rust
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MeshHandle(pub u32);

pub struct MeshPool {
    meshes: Vec<Mesh>,
}

impl MeshPool {
    pub fn insert(&mut self, mesh: Mesh) -> MeshHandle {
        let h = MeshHandle(self.meshes.len() as u32);
        self.meshes.push(mesh);
        h
    }
    pub fn get(&self, h: MeshHandle) -> &Mesh {
        &self.meshes[h.0 as usize]
    }
}
```

Handles are not `EntityId`s. Entities are game data and live in `World`.
Resources are engine data and live in pools owned by `State`. Keeping
that line clean is Design Principle #1 in `ROADMAP.md`.

Generational handles for resources are optional here — plain `u32` is
fine while assets never unload. Revisit if/when streaming lands
(Phase 13 in the current ROADMAP.md).

### Deferred spawn and kill

Systems iterate pools; spawning or killing during iteration mutates them.
Buffer both:

```rust
pending_spawn: Vec<PendingSpawn>,   // component bundles to install
pending_kill:  Vec<EntityId>,
```

Systems call `world.spawn_deferred(...)` / `world.kill_deferred(id)`. The
main loop drains both at the top of each frame, before any system runs:

```
loop {
    world.flush_kills();    // bumps generations, removes from every pool
    world.flush_spawns();   // installs queued component bundles
    for system in systems { system.run(&mut world, dt, &input); }
    renderer.render(&world, &resources);
}
```

This also gives a single well-defined point where generation counters
bump — no surprises mid-frame.

### Instance buffers vs per-entity uniform buffers (Step 7 background)

Step 7 uses a **single instance buffer** rather than per-entity uniform
buffers. Earlier drafts of this plan suggested
`HashMap<EntityId, wgpu::Buffer>` keyed per-entity — that was the wrong
shape and has been removed. Quick tour of the patterns and why
instancing won here:

- **Per-entity uniform buffers** — each entity gets its own small
  buffer; one `queue.write_buffer` call per entity per frame. Driver
  overhead scales linearly with entity count. Easy to write but
  architecturally a dead end — GPU-driven rendering, RT TLAS rebuilds,
  and instancing all want their data consolidated. Phase 5 skips this
  trap.
- **Instance buffer** — one structured buffer holds every renderable's
  transform end-to-end. Draws use `base_instance` +
  `@builtin(instance_index)` to index into it. One `queue.write_buffer`
  per frame total. **This is what Step 7 builds.** It is also the shape
  that GPU-driven rendering, RT, and bindless all extend cleanly.
- **Single uniform buffer + dynamic offset** — older variant of the
  same idea using uniform buffers and per-draw dynamic offsets. Works
  fine but has stricter alignment rules than storage buffers. The
  instance-buffer pattern with `@builtin(instance_index)` is the modern
  default.
- **GPU-driven rendering** — Phase-11 territory. CPU writes a
  draw-command buffer; GPU consumes it via indirect draws. The
  per-instance data is the same instance buffer Step 7 builds, plus a
  draw-command buffer alongside it. Layering this on top of Step 7's
  architecture is additive, not a rewrite.

Phase 5 ships the instance-buffer pattern with a simple "rewrite the
whole buffer every frame" upload. Incremental updates (dirty flags) are
Phase-7 scene-graph work; indirect draws are Phase 11.

### GPU-driven rendering (forward look, post-Phase-5)

"Everything on GPU" is the modern AAA pattern Spartan and UE5 advertise.
It is a *renderer* concern, separate from the ECS choice in this phase.
Praxis can adopt it later without revisiting Phase 5 work.

Concretely, GPU-driven rendering replaces these CPU-side patterns:

- **Per-entity uniform buffer** (Step 7's plan) → **one persistent scene
  buffer** holding every transform / material entry, updated
  incrementally via dirty flags.
- **Bind-group-per-material** → **bindless**: one giant descriptor array
  of textures; shaders index into it by material ID. No per-material
  bind-group churn.
- **Submit-draw-per-mesh** → **indirect drawing**: CPU writes a command
  buffer once; GPU reads it and dispatches every draw without further
  CPU involvement.
- **CPU-side frustum / occlusion culling** → **compute-shader culling**:
  GPU tests visibility against camera + depth pyramid, builds the
  visible-instance list the indirect draw consumes.
- **One wgpu buffer per mesh** → **one mega-buffer**: meshes are
  sub-allocations referenced by offset; helps streaming and reduces
  bind churn.

Where the threshold sits, roughly: CPU draw-call submission becomes the
bottleneck around 3,000–5,000 visible draws per frame on modern
hardware. Below that, traditional submission is fine. A Skyrim-shape
RPG with chunked streaming and aggressive culling typically lives in
the hundreds to low thousands — well below the threshold.

UE5 Nanite is the extreme other end (virtualized geometry, millions of
triangles, fully GPU-resident). It is a tech-demo flagship more than a
necessity; it also leans hard on TAA / TSR to hide its micropolygon
stochastic-shading artifacts, which is part of why modern UE titles
look soft and smeary in motion.

Phase-5 lock-in for going GPU-driven later: low. The lock-in lives in
the renderer — per-entity uniform buffers, bind-group-per-material,
immediate-mode draw submission. All of those are already on the rewrite
list whenever a rendering-focused phase tackles bindless or indirect.
The ECS sitting underneath stays unchanged or gets swapped for
`bevy_ecs` independently.

### Pull-renderer and uber-shader (forward look)

Two terms from `ROADMAP.md`'s Design Principles that surface in phase
planning but aren't built yet.

- **Pull-renderer.** Each frame, the renderer queries the ECS for what to
  draw (`world.query::<(Transform, MeshHandle, MaterialHandle)>()`) and
  pulls the data it needs. The opposite is a push model: ECS code calls
  into the renderer to register draws. Pull keeps sim ignorant of wgpu —
  Design Principle #1.
- **Uber-shader.** One large shader with branches or feature flags for
  every material variant, instead of N specialized shaders. Trades
  runtime branching for build-time and binding simplicity. Phase 5 does
  not write one; the seed is just "do not proliferate per-material
  shaders without thinking".

### Rust syntax cheatsheet

The bits of Rust most likely to feel hazy this phase, in one place.

- **`Box<T>`** — owned heap allocation. `Box<Pool<Transform>>` puts the
  pool on the heap and gives back an owning pointer. Drops the pool when
  the `Box` drops.
- **`dyn Trait`** — runtime-polymorphic trait object. `&dyn Trait` is a
  fat pointer (data + vtable). `Box<dyn Trait>` is the owned version. Use
  when you need a heterogeneous container; pay one indirect call per
  method.
- **`impl Trait`** — static-dispatch alternative. `fn foo() -> impl
  Iterator` means "some concrete iterator type, fixed at compile time".
  No vtable, but every call site sees one specific type.
- **Generics `<T: Trait>`** — monomorphized. `Pool<Transform>` and
  `Pool<Velocity>` are *different* compiled types. Fast, but explodes
  binary size if abused.
- **`'static` bound** — "this type doesn't borrow from anything with a
  shorter lifetime". Required for `TypeId::of::<T>()` and for storing in
  long-lived containers. Owned data (`String`, `Vec<u8>`, plain structs)
  is `'static` by default.
- **`Any` trait** — built-in trait every `'static` type implements.
  Provides `type_id()` and the `downcast_*` methods. The mechanism behind
  type-erased storage.
- **`TypeId::of::<T>()`** — opaque, comparable runtime ID for `T`. Two
  `TypeId`s are equal iff the types are the same.
- **`HashMap::entry(k).or_insert_with(...)`** — get the value at `k`, or
  insert a default if missing. Common when lazily creating pools.
- **`Option<&mut T>` and `?`** — Rust's null-safe pointer plus its
  short-circuit operator. Pattern: `let pool = self.pools.get_mut(&id)?;`
  returns `None` from the enclosing function if the lookup fails.
- **`Vec<T>::swap_remove(i)`** — built-in helper that does the
  swap-and-pop dance for plain `Vec`s. The maps in `Pool<T>` make us do
  it manually since we have to update sparse indices alongside.
- **`#[derive(Copy, Clone)]`** — auto-implements bit-copy semantics. Use
  for tiny ID structs (`EntityId`, `MeshHandle`). Don't `Copy` anything
  that owns a heap allocation.
- **`#[repr(C)]`** — forces C-style struct layout (predictable field
  order, no Rust-internal reordering). Required for any struct sent to
  the GPU, since WGSL expects a fixed layout. Pairs with
  `bytemuck::Pod + Zeroable` to safely cast to `&[u8]`.
- **`bytemuck::Pod` / `Zeroable`** — `Pod` = "all bit patterns valid, no
  padding holes" (safe to reinterpret). `Zeroable` = "all-zero bytes is a
  valid value". Together they let `bytemuck::cast_slice(&[my_struct])`
  produce `&[u8]` for `queue.write_buffer`.

---

## Steps

Each step ends with a visible or verifiable checkpoint. The rule: no step
advances until its checkpoint holds.

### Step 1: Entity allocator

_Introduce `EntityId` (index + generation) and an `Entities` allocator with
a free-list. No components yet._

**Notes.** See *Generational entity IDs* above. Suggested module: `src/ecs/entity.rs`.
`Entities` owns `generations: Vec<u32>` and `free: Vec<u32>`. Resist reaching
for the `slotmap` crate — writing this by hand is the point.

- [ ] <!-- TODO -->

Checkpoint: spawn and despawn a handful of entities in a throwaway test in
`main.rs`. Reusing a despawned slot produces an `EntityId` whose generation
has bumped. Stale ids fail an `is_alive()` check.

### Step 2: One-component storage (sparse set)

_Implement `Pool<Transform>` with dense `Vec<Transform>`, forward map
`HashMap<EntityId, usize>`, reverse map `HashMap<usize, EntityId>`, and
swap-and-pop remove._

**Notes.** See *Sparse set storage* above. Build it concretely for
`Transform` first — do not try to write the generic `Pool<T>` and the erased
trait in one go. Prove the data structure with one type, then generalize in
Step 3. This is how Pikuma's ECS is also written: `Pool<T>` exists, but the
author can reason about one `T` at a time.

- [ ] <!-- TODO -->

Checkpoint: insert 3 transforms, iterate, remove the middle one, iterate
again. Data stays contiguous; indices stay consistent.

### Step 3: Generalize to any component type

_Introduce `trait Component: 'static` (or just require `Any + 'static`), a
`trait ErasedPool` for type-erased storage, and a `World` that owns
`HashMap<TypeId, Box<dyn ErasedPool>>`._

**Notes.** See *Type-erased component storage* above. The minimum viable
`ErasedPool` has three methods: `remove`, `as_any`, `as_any_mut`. Add more
only when a concrete need appears. `World::add_component::<T>(e, value)`
either grabs or creates the `Pool<T>` for that `TypeId`, then inserts.

- [ ] <!-- TODO -->

Checkpoint: `world.add_component::<Transform>(e, t)` and
`world.add_component::<Velocity>(e, v)` both compile, survive downcasting,
and round-trip through `get_component`.

### Step 4: Query API

_Single-component query `world.query::<Transform>()`, then tuple
`world.query::<(Transform, Velocity)>()`. Uses a per-entity signature
bitset to skip non-matching entities fast._

**Notes.** See *Signature bitsets*. Simplest first cut for tuple queries:
compute the query signature, walk `entity_signatures`, collect matching
`EntityId`s into a `Vec`, then for each match look up each component pool
separately. This avoids the double-mut-borrow problem flagged in the mental
model. A "real" joint iterator (returning `(&T1, &T2)` tuples without the
intermediate `Vec`) can wait until the pattern is proven.

- [ ] <!-- TODO -->

Checkpoint: a two-component query over 5 entities (3 with both components,
2 with only one) yields exactly the 3 matching ones. Iteration order is
deterministic.

### Step 5: Resource pools and handles

_Introduce `MeshHandle(u32)`, `TextureHandle(u32)`, `MaterialHandle(u32)` and
central pools that own the `wgpu` objects. Pools live outside `World`._

**Notes.** See *Handles as lightweight references*. Simplest pool: a newtype
around `Vec<Mesh>`; `MeshHandle(u32)` is the index. No free-list — Phase 5
never unloads. The existing `GpuPrimitive` / `model` in `state.rs` should
migrate into a `MeshPool` + `MaterialPool`; decide whether a "mesh" is one
primitive or a group of them (one helmet has several primitives — this is a
real design decision, think before coding).

- [ ] <!-- TODO -->

Checkpoint: the helmet mesh is loaded once into `MeshPool`; its handle is
logged on startup. The three helmet entities will share that single handle
in Step 6.

### Step 6: Migrate the 3 helmets into ECS

_Replace `State.transforms` and `State.primitives` with ECS entities carrying
`Transform` + `MeshHandle` + `MaterialHandle` components. Keep the old render
path temporarily ugly — no renderer refactor yet._

**Notes.** Do not rewrite `render()` yet. Goal: "scene data lives in ECS,
render path still nested." The render loop can iterate
`world.query::<(Transform, MeshHandle, MaterialHandle)>()` and then, for each
renderable, look the mesh up in `MeshPool` and draw. It will look
transitional — that is fine. The milestone is **the picture on screen is
identical**, proving the data flow works before you touch rendering.

- [ ] <!-- TODO -->

Checkpoint: **three helmets still on screen, unchanged**. This is the
milestone commit for this phase.

### Step 7: Renderer queries ECS, single instance buffer

_Delete the hardcoded loops and per-index `State.model_buffers`. `render()`
(or a `prepare_renderables` pass feeding it) drives off the query alone.
Per-entity GPU buffers are out — instead, one persistent **instance
buffer** holds every renderable's transform, indexed by draw position._

**Notes.** Skip the `HashMap<EntityId, Buffer>` per-entity pattern that
earlier drafts of this plan suggested. It is a dead end: GPU-driven
rendering, RT TLAS rebuilds, and instancing all want one structured
buffer of instance data, not many small uniform buffers. Bake the right
shape now — see **Load-bearing decisions** in `ROADMAP.md` for why.

Concrete shape:

- `State` owns one `wgpu::Buffer` of `InstanceData` (model matrix for
  now; normal matrix + material ID get added in later phases), sized
  for an initial capacity and resized on overflow.
- A `prepare_renderables` pass each frame walks
  `world.query::<(Transform, MeshHandle, MaterialHandle)>()`, builds a
  `Vec<InstanceData>`, and uploads it once via `queue.write_buffer`.
  One write per frame, regardless of entity count.
- Each draw call uses `base_instance` + instance count to span its
  slice of the instance buffer. Vertex shader reads its
  `@builtin(instance_index)` in WGSL and uses it to look up the model
  matrix in the buffer.
- `prepare_renderables` also produces the `(mesh, material, instance_idx)`
  draw list the renderer iterates.

What this is **not** doing yet: bindless materials (Phase 10), indirect
draws (Phase 11), persistent buffer with incremental dirty-flag updates
(Phase 7 scene-graph work). The instance buffer is fully rewritten each
frame here. Fine at Phase-5 entity counts; the architecture is right
even if the implementation is the simple version.

- [ ] <!-- TODO -->

Checkpoint: same 3 helmets, but `State` no longer owns `transforms` or
any per-entity buffers. One instance buffer, one `queue.write_buffer`
per frame, draws indexed into it. Render data flows from ECS.

### Step 8: Camera and input become systems

_Extract the camera fly logic out of `State::update()` into
`fn camera_system(world: &mut World, dt: f32, input: &Input)`. Same for input
handling. `State::update()` shrinks to a dispatcher._

**Notes.** Free functions, not traits. No system scheduling yet — just an
ordered list of calls in `State::update()`. The camera itself can stay as a
single entity with a `Camera` component + `Transform`, or live outside the
ECS as engine state. Pick one and note the reasoning; both are defensible.

- [ ] <!-- TODO -->

Checkpoint: camera controls identical to Phase 4. `State::update()` is
mostly a list of system calls.

### Step 9: Spawn / despawn at runtime

_Keybind (e.g. `G`) spawns a new helmet at the camera's current position;
another (e.g. `H`) despawns the most recently spawned. Proves the deferred
spawn/kill buffer and the generational id invariant hold under real use._

**Notes.** See *Deferred spawn and kill*. Test the nasty cases: spawn during
iteration, spawn 100 entities in one frame, rapid spawn+kill to exercise
generation bumping, kill then try to use the old handle. Check that
`State.model_buffers` doesn't leak entries for dead entities.

- [ ] <!-- TODO -->

Checkpoint: can add and remove helmets live without crashes, without leaked
GPU resources, and without stale-handle aliasing. Closing the app drops all
pools cleanly.

---

## Final shape (target)

| Thing | Suggested path | Owned by |
|---|---|---|
| `EntityId`, `Entities` allocator | `src/ecs/entity.rs` | `World` |
| `Pool<T>`, `ErasedPool`, query machinery | `src/ecs/storage.rs` | `World` |
| `World` | `src/ecs/world.rs` | `State` |
| `Transform`, `MeshHandle`, `MaterialHandle` components | `src/components/` | entities in `World` |
| `MeshPool`, `TexturePool`, `MaterialPool` | `src/render/resources.rs` | `State` (outside `World`) |
| Camera + light uniform buffers | `State` (unchanged from Phase 4) | `State` |
| Single instance buffer (`Vec<InstanceData>` uploaded each frame) | `State` (new this phase, replaces per-entity `model_buffers`) | `State` |
| Camera and input systems | `src/systems/` | free functions over `&mut World` |

<!-- TODO: adjust module layout if you settle on a different one — this is a starting sketch, not a contract -->
