# Architecture Decision Records: Praxis Engine Rebuild

**Status**: Decision Review Document  
**Date**: 2024  
**Context**: Evaluating core architectural decisions for Praxis engine rebuild/refactor

---

## Table of Contents

1. [ADR-001: Graphics Backend - wgpu vs vulkano](#adr-001-graphics-backend---wgpu-vs-vulkano)
2. [ADR-002: Crate Structure - Granularity vs Consolidation](#adr-002-crate-structure---granularity-vs-consolidation)
3. [ADR-003: ECS Framework Selection](#adr-003-ecs-framework-selection)
4. [ADR-004: Core Dependencies Architecture](#adr-004-core-dependencies-architecture)
5. [ADR-005: Feature Flags - Optional vs Required Subsystems](#adr-005-feature-flags---optional-vs-required-subsystems)

---

## ADR-001: Graphics Backend - wgpu vs vulkano

### Context

The engine requires a modern graphics API abstraction. Two primary options exist:

1. **vulkano** - Safe Rust wrapper around Vulkan API
2. **wgpu** - Cross-platform graphics abstraction (Vulkan/Metal/DX12/WebGPU)

### Decision Factors

#### 1. Platform Portability

**wgpu Advantages:**
- Cross-platform by design: Windows (DX12), macOS (Metal), Linux (Vulkan), Web (WebGPU)
- Single codebase for all platforms
- Lower barrier to entry for contributors on non-Vulkan platforms
- Future-proof for web deployment

**vulkano Advantages:**
- Explicit Vulkan-only (no hidden platform differences)
- More predictable behavior across deployments
- Desktop-focused (no web constraints)

**Analysis**: For a game engine targeting desktop primarily, cross-platform benefits are significant but not critical. Most modern gaming platforms support Vulkan. Web deployment is uncommon for serious game engines.

#### 2. Vulkan Control & Features

**wgpu Limitations:**
- Abstraction layer hides Vulkan-specific features
- Lowest-common-denominator API (limited by WebGPU spec)
- Cannot use Vulkan extensions (ray tracing, mesh shaders, etc.)
- Indirect control over synchronization primitives
- No direct access to VkQueue, VkFence, VkSemaphore

**vulkano Advantages:**
- Direct Vulkan mapping - full API access
- Explicit control over command buffers, pipelines, synchronization
- Access to latest Vulkan features and extensions
- Educational value: teaches real Vulkan patterns
- Better for advanced rendering techniques (deferred, ray tracing, compute-heavy)

**Analysis**: Praxis is an *educational* engine focused on teaching modern graphics techniques. Direct Vulkan exposure is a feature, not a bug.

#### 3. Performance Characteristics

**wgpu:**
- Additional abstraction layer overhead (minimal but present)
- Runtime backend selection overhead
- Validation in both wgpu layer and native driver
- Shader translation (WGSL → SPIR-V/MSL/HLSL)

**vulkano:**
- Zero-cost abstractions over Vulkan (compiles to direct Vulkan calls)
- Single validation layer (Vulkan validation)
- Direct SPIR-V compilation via `vulkano-shaders` macro
- Compile-time optimizations possible

**Analysis**: For a performance-focused educational engine, eliminating abstraction layers provides clearer profiling and optimization opportunities.

#### 4. Type Safety & Rust Integration

**Both provide:**
- Memory-safe GPU resource management
- Compile-time pipeline validation
- Safe command buffer construction

**vulkano specific:**
- Tighter integration with Vulkan object model
- More explicit lifetime tracking for GPU resources
- Strongly-typed descriptor sets via macros
- Compile-time shader reflection

**wgpu specific:**
- Simpler API surface (fewer concepts to learn)
- More Rust-idiomatic (less Vulkan ceremony)
- Better error messages for beginners

**Analysis**: vulkano's complexity is acceptable given educational goals. Seeing explicit lifetimes and descriptor management teaches important GPU concepts.

#### 5. Ecosystem & Maturity

**wgpu:**
- Active development (part of WebGPU standardization)
- Strong community (used by Bevy engine)
- Excellent documentation
- Frequent breaking changes (0.x versions)

**vulkano:**
- Mature (stable 0.35.x, approaching 1.0)
- Comprehensive Vulkan coverage
- Good documentation with Vulkan spec references
- Slower-moving (stability-focused)

**Analysis**: Both are production-ready. vulkano's maturity is preferable for educational stability.

#### 6. Implementation Status

**Current State:**
- Praxis uses vulkano 0.35.1
- 18+ months of development invested
- Custom pipelines: forward, deferred, shadows, HDR, post-processing
- Advanced features: GPU culling, LOD, Hi-Z occlusion, mesh streaming
- Procedural texture system with runtime GLSL→SPIR-V compilation

**Migration Cost:**
- High: ~3,000+ LOC in `praxis_graphics`
- Rewrite all shader code (GLSL → WGSL)
- Rebuild pipeline abstractions
- Lose Vulkan-specific optimizations
- Re-test all rendering paths

### Decision: **KEEP vulkano**

**Rationale:**

1. **Educational Focus**: Direct Vulkan exposure teaches industry-standard API patterns
2. **Feature Access**: Required for advanced techniques (compute, ray tracing future)
3. **Performance**: Zero-cost abstractions enable clear optimization examples
4. **Investment Protection**: Thousands of LOC working; migration cost unjustified
5. **Portability Reality**: Desktop focus acceptable; Linux/Windows Vulkan coverage sufficient

**Trade-offs Accepted:**
- macOS users need MoltenVK (acceptable for educational/desktop engine)
- No native web deployment (not a goal for this engine)
- Steeper learning curve (intentional - teaches Vulkan)

**Rejected Alternative (wgpu):**
- Would simplify cross-platform
- Would reduce API surface
- But loses educational value, feature access, and investment

---

## ADR-002: Crate Structure - Granularity vs Consolidation

### Context

Current structure: **19 crates** organized by subsystem. Question: Is this optimal or should crates be consolidated?

### Current Structure (19 Crates)

#### Foundation Layer (3)
- `praxis_utils` - Logging, errors, timing
- `praxis_math` - Math wrappers (glam)
- `praxis_core` - Engine lifecycle

#### Platform Layer (2)
- `praxis_window` - Windowing (winit)
- `praxis_input` - Input handling

#### Engine Layer (8)
- `praxis_ecs` - Entity-Component-System
- `praxis_graphics` - Rendering
- `praxis_scene` - Transforms, animation
- `praxis_spatial` - Octree, BVH
- `praxis_assets` - Asset loading
- `praxis_physics` - Physics (Rapier3D)
- `praxis_audio` - Audio (Kira)
- `praxis_procedural` - Procedural textures

#### Application Layer (6)
- `praxis_gui` - Immediate-mode UI
- `praxis_profiling` - Performance monitoring
- `praxis_scripting` - Lua integration
- `praxis_networking` - Multiplayer
- `praxis_terrain` - Terrain system
- `praxis_editor` - Editor tools

### Alternative Structures

#### Option A: Consolidated (8-10 Crates)

**Merge:**
- `praxis_window` + `praxis_input` → `praxis_platform`
- `praxis_scene` + `praxis_spatial` → `praxis_scene` (spatial is scene optimization)
- `praxis_assets` + `praxis_procedural` → `praxis_assets` (both asset sources)
- `praxis_gui` + `praxis_profiling` → `praxis_gui` (profiling UI)
- `praxis_editor` remains separate
- Optional: `praxis_scripting`, `praxis_networking`, `praxis_terrain`

**Result: ~10 crates**

#### Option B: Minimal (5-6 Crates)

**Merge:**
- `praxis_core` + `praxis_utils` + `praxis_math` → `praxis_core`
- `praxis_window` + `praxis_input` → `praxis_platform`
- `praxis_ecs` + `praxis_scene` + `praxis_spatial` → `praxis_ecs`
- `praxis_graphics` + `praxis_procedural` + `praxis_gui` → `praxis_graphics`
- `praxis_physics` + `praxis_audio` + `praxis_assets` → `praxis_runtime`
- Optional: `praxis_editor`, `praxis_scripting`, `praxis_networking`, `praxis_terrain`

**Result: ~5-9 crates depending on features**

#### Option C: Monolithic (1-2 Crates)

- Single `praxis` crate with internal modules
- Optional features toggle subsystems
- Similar to small engines (Macroquad, Pixels)

**Result: 1-2 crates**

### Evaluation Criteria

#### 1. Compilation Time

**19 Crates (Current):**
- Parallel compilation: Foundation → Platform → Engine → Application
- Incremental: Changing `praxis_audio` doesn't rebuild graphics
- Clean build: ~2-3 minutes (M1 Max)
- Incremental: ~5-30s depending on crate

**Consolidated (8-10 Crates):**
- Still good parallelization
- Slightly larger incremental rebuilds (merged crates)
- Clean build: Similar (~2-3 min)
- Incremental: ~10-45s (larger crates)

**Minimal (5-6 Crates):**
- Reduced parallelization (fewer dependency branches)
- Large incremental rebuilds (big crates)
- Clean build: ~2-4 minutes
- Incremental: ~30-90s (much larger crates)

**Monolithic (1-2 Crates):**
- No parallelization
- Full crate rebuild on any change
- Clean build: ~3-5 minutes
- Incremental: ~1-3 minutes (unacceptable for development)

**Winner: Current (19) or Consolidated (8-10)**

#### 2. Dependency Management

**19 Crates:**
- ✅ Clear boundaries prevent circular dependencies
- ✅ Easy to see dependency graph
- ✅ Force explicit coupling
- ❌ More `Cargo.toml` to maintain
- ❌ Version coordination across workspace

**Consolidated:**
- ✅ Fewer dependency declarations
- ✅ Simpler version management
- ⚠️ Hidden internal coupling
- ❌ Harder to prevent circular logic

**Minimal/Monolithic:**
- ❌ Internal modules can couple arbitrarily
- ❌ No enforced architecture
- ❌ Circular dependencies possible within crate

**Winner: Current (19) - Enforces clean architecture**

#### 3. Discoverability & Learning

**19 Crates:**
- ✅ Self-documenting: `praxis_audio` clearly handles audio
- ✅ New contributors find relevant code quickly
- ✅ Each crate has focused README
- ✅ Clear separation of concerns
- ❌ Can overwhelm beginners ("19 crates??")

**Consolidated:**
- ⚠️ Reasonable: 8-10 crates still navigable
- ⚠️ Some crates become multi-purpose
- ⚠️ READMEs cover more ground

**Minimal/Monolithic:**
- ❌ Large crates require module diving
- ❌ Less clear where functionality lives
- ❌ Harder to understand overall structure

**Winner: Current (19) for educational clarity**

#### 4. Reusability

**19 Crates:**
- ✅ Use only what you need: `praxis_math` + `praxis_graphics` without physics
- ✅ Other projects can depend on specific crates
- ✅ Clear API boundaries encourage reuse
- ✅ Easy to extract and publish specific functionality

**Consolidated:**
- ⚠️ Larger dependencies (bring more than needed)
- ⚠️ Still modular but less granular

**Minimal/Monolithic:**
- ❌ All-or-nothing dependencies
- ❌ Hard to use subsystems independently
- ❌ Discourages external reuse

**Winner: Current (19) for maximum flexibility**

#### 5. Testing & CI

**19 Crates:**
- ✅ Unit tests scoped to crate
- ✅ Can run `cargo test -p praxis_audio` in isolation
- ✅ Clear test failure source
- ⚠️ Integration tests span crates

**Consolidated:**
- ⚠️ Tests still focused but larger scope
- ⚠️ Slower per-crate test runs

**Minimal/Monolithic:**
- ❌ Large test suite runs for any change
- ❌ Harder to isolate failures
- ❌ CI takes longer

**Winner: Current (19) for test isolation**

#### 6. Maintenance Overhead

**19 Crates:**
- ❌ 19 × Cargo.toml to maintain
- ❌ 19 × lib.rs with lints/exports
- ❌ Workspace version coordination
- ✅ Clear ownership per crate
- ✅ Smaller codebases easier to refactor

**Consolidated:**
- ✅ Fewer manifests
- ✅ Less version coordination
- ⚠️ Larger refactors span more code

**Minimal/Monolithic:**
- ✅ Minimal configuration
- ❌ Massive refactors
- ❌ Unclear module ownership

**Winner: Consolidated (8-10) for maintenance**

### Case Studies

#### Similar Engines

**Bevy** (Rust, ECS-based):
- ~40+ crates (even more granular than Praxis)
- Rationale: Maximum modularity, opt-in features
- Success: Widely adopted, clear architecture

**Fyrox** (Rust, 3D):
- ~15 crates
- Similar granularity to Praxis
- Success: Mature, production-ready

**Amethyst** (Rust, archived):
- ~30+ crates (very granular)
- Rationale: Extreme modularity
- Issue: Contributed to complexity and eventual archival

**Macroquad** (Rust, simple):
- 1 crate with modules
- Rationale: Simplicity for small games
- Trade-off: Not suitable for large projects

### Specific Merge Analysis

#### Viable Merges

**1. `praxis_window` + `praxis_input` → `praxis_platform`**
- ✅ Tight coupling (input needs window events)
- ✅ Both thin wrappers around winit
- ⚠️ Lose separate testability
- **Recommendation**: Viable, but current separation is fine

**2. `praxis_scene` + `praxis_spatial`**
- ✅ Spatial structures optimize scene operations
- ✅ Both work on transforms
- ❌ Spatial has independent value (physics, AI can use)
- **Recommendation**: Keep separate for reusability

**3. `praxis_assets` + `praxis_procedural`**
- ❌ Different concerns: loading vs generation
- ❌ Procedural depends on graphics (compute shaders)
- ❌ Assets is pure data processing
- **Recommendation**: Keep separate

**4. `praxis_gui` + `praxis_profiling`**
- ❌ Profiling useful without GUI (headless profiling, file export)
- ❌ GUI used beyond profiling (editor, debug tools)
- **Recommendation**: Keep separate

#### Unviable Merges

- Core layer merges: Foundation crates used independently
- Engine layer merges: Too much functionality, lose parallelization
- Application layer merges: Different optional features

### Decision: **KEEP 19-crate structure with minor consolidation allowed**

**Rationale:**

1. **Educational Value**: Clear separation teaches subsystem boundaries
2. **Compilation Speed**: Parallel builds and incremental compilation optimize development
3. **Architectural Enforcement**: Prevents circular dependencies via cargo
4. **Discoverability**: Self-documenting structure helps contributors
5. **Flexibility**: Use only needed subsystems
6. **Proven Pattern**: Bevy and Fyrox validate this granularity

**Guidelines for Future Crates:**

- New crate justified if:
  - Independently testable subsystem (e.g., networking, scripting)
  - Optional feature users might not want
  - Clear API boundary with single responsibility
  - Non-trivial implementation (>1000 LOC)

- Merge crates if:
  - Always used together (window/input borderline)
  - Circular dependency needs broken
  - One crate is just types for another (<200 LOC)

**Trade-offs Accepted:**
- More Cargo.toml files to maintain (automation helps)
- Workspace version coordination needed
- Initial intimidation factor for new contributors (docs mitigate)

**Rejected Alternatives:**
- Consolidated structure: Loses architectural benefits
- Minimal structure: Too coarse for educational goals
- Monolithic: Unacceptable compilation times

---

## ADR-003: ECS Framework Selection

### Context

Praxis uses Entity-Component-System architecture. Must choose:
1. Custom ECS implementation
2. Existing ECS library

If library: **bevy_ecs** vs alternatives (hecs, legion, specs, shipyard)

### Requirements

- **Performance**: 10,000+ entities with dynamic queries
- **Ergonomics**: Rust-friendly API, minimal boilerplate
- **Features**: Change detection, events, resources, commands
- **Stability**: Production-ready, maintained
- **Serialization**: Save/load game state
- **Flexibility**: Support for complex queries, system ordering

### Option A: Custom ECS Implementation

**Pros:**
- Full control over implementation
- Educational value (teach ECS internals)
- Tailored to Praxis needs
- No external dependencies

**Cons:**
- Months of development time
- Likely slower than battle-tested libraries
- Complex to implement correctly (archetypes, queries, scheduling)
- Maintenance burden
- Reinventing well-solved problem

**Analysis**: Not justified unless teaching ECS implementation is a goal. Praxis teaches engine architecture, not ECS algorithms.

### Option B: bevy_ecs (Current Choice)

**Version**: 0.14 (latest stable)

**Architecture**:
- Archetype-based storage (cache-friendly)
- Compile-time system parameters
- Query filters and iteration
- Change detection (Added/Changed/Mutated)
- Events system
- Commands for deferred spawning/despawning
- Schedules and system ordering

**Pros:**
- ✅ **Performance**: Industry-leading (beats specs, hecs, legion in benchmarks)
- ✅ **Ergonomics**: Excellent API, minimal boilerplate, strong typing
- ✅ **Features**: Comprehensive (everything needed + more)
- ✅ **Stability**: Used in production (Bevy engine, multiple games)
- ✅ **Serialization**: Built-in with `serialize` feature
- ✅ **Documentation**: Excellent docs, many examples
- ✅ **Community**: Large, active, responsive maintainers
- ✅ **Integration**: Designed to be used standalone (not just for Bevy)

**Cons:**
- ⚠️ Part of Bevy (versioning tied to Bevy releases)
- ⚠️ Breaking changes between major versions
- ⚠️ Some features Bevy-specific (Bevy-specific traits, Assets integration)
- ⚠️ Learning curve (powerful but complex API)

**Performance Data** (from Bevy benchmarks):
- Simple iteration: ~2.5 ns/entity (10M entities/sec)
- Complex queries: ~15-30 ns/entity
- Change detection overhead: ~5-10%
- Archetype migration: Fast (rare operation)

**Current Usage in Praxis:**
- ~50+ component types
- ~20+ systems
- Transform hierarchy with change tracking
- Physics/graphics sync via change detection
- Serialization for save/load

### Option C: hecs

**Architecture**: Archetype-based (similar to bevy_ecs)

**Pros:**
- ✅ Lightweight (~5K LOC vs bevy_ecs ~15K LOC)
- ✅ Simple API
- ✅ Good performance
- ✅ Stable 0.10.x

**Cons:**
- ❌ Minimal features (no built-in events, resources, commands)
- ❌ No change detection
- ❌ Manual system scheduling
- ❌ Smaller community
- ❌ Less documentation

**Analysis**: Too minimal. Would need to build events, resources, commands, change detection - defeating the purpose.

### Option D: legion

**Status**: Maintenance mode (archived), recommend using bevy_ecs instead

**Analysis**: Not viable due to archived status.

### Option E: specs

**Architecture**: Storage-based (DenseVecStorage, HashMapStorage, etc.)

**Pros:**
- ✅ Mature (used in Amethyst)
- ✅ Flexible storage types
- ✅ Good documentation

**Cons:**
- ❌ Slower than archetype-based (worse cache locality)
- ❌ More boilerplate (register components, storages)
- ❌ Less active development
- ❌ Older design patterns

**Analysis**: Archetype-based ECS proven superior for game use cases.

### Option F: shipyard

**Architecture**: Sparse set-based

**Pros:**
- ✅ Fast component addition/removal
- ✅ Unique features (unique components, workloads)
- ✅ Good documentation

**Cons:**
- ❌ Slower iteration than archetypes (less cache-friendly)
- ⚠️ Smaller community than bevy_ecs
- ⚠️ Less production usage

**Analysis**: Sparse sets trade iteration speed for mutation speed. Games iterate more than mutate.

### Comparison Matrix

| Feature | bevy_ecs | hecs | specs | shipyard |
|---------|----------|------|-------|----------|
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Ergonomics** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Features** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Stability** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **Community** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Documentation** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

### Integration Considerations

**Tight Integration Required:**
- Transform hierarchy (Parent/Children components)
- Physics sync (change detection for Transform → RigidBody)
- Animation blending (query multiple components)
- Save/load (serialize entire world)
- Editor tools (spawn/despawn with commands)

**bevy_ecs Advantages:**
- Change detection: Detect Transform changes without manual tracking
- Commands: Defer entity operations during iteration
- Events: Decouple systems (collision events, input events)
- Resources: Singleton data (PhysicsWorld, AudioManager)
- Schedules: Declare system ordering constraints

**Example** (why bevy_ecs features matter):

```rust
// With bevy_ecs change detection
fn sync_physics_system(
    query: Query<(&Transform, &mut RigidBody), Changed<Transform>>,
) {
    // Only iterate entities where Transform changed
    // 10x faster than checking all entities
}

// With hecs (no change detection)
fn sync_physics_system(world: &World) {
    // Must iterate ALL entities, check manually
    for (id, (transform, body)) in world.query::<(&Transform, &mut RigidBody)>() {
        // Check if transform changed somehow? (how?)
    }
}
```

### Decision: **KEEP bevy_ecs**

**Rationale:**

1. **Performance**: Best-in-class for game workloads
2. **Features**: Change detection, events, commands essential for engine
3. **Ergonomics**: Productive API, minimal boilerplate
4. **Stability**: Production-proven, active development
5. **Community**: Large ecosystem, easy to find help
6. **Investment**: Already integrated, working well

**Trade-offs Accepted:**
- Version coupling with Bevy (manageable, ~6mo release cycle)
- Breaking changes require updates (acceptable for 0.x engine)
- Larger dependency than minimal alternatives (worth it for features)

**Rejected Alternatives:**
- Custom ECS: Too much work, worse performance, no educational value
- hecs: Too minimal, missing essential features
- specs: Outdated, slower architecture
- shipyard: Smaller community, less production validation

**Future Consideration:**
- Monitor bevy_ecs stability as approaches 1.0
- If Bevy makes bevy_ecs hard to use standalone, consider fork or migration
- Unlikely: bevy_ecs designed for standalone use

---

## ADR-004: Core Dependencies Architecture

### Context

Must prevent circular dependencies and establish clear dependency flow. Critical for:
- Compilation order
- Code maintainability
- Architectural clarity

### Problem: Circular Dependency Risks

**Potential Circular Patterns:**

1. **graphics ↔ window**
   - Graphics needs window surface
   - Window might want to query graphics state

2. **graphics ↔ core**
   - Core manages engine lifecycle, needs graphics
   - Graphics might need core utilities

3. **ecs ↔ graphics**
   - Graphics needs ECS components (MeshComponent, Transform)
   - ECS might want graphics types (Texture, Material)

4. **core ↔ everything**
   - Core is "god object" depending on all subsystems
   - Subsystems depend on core utilities

### Current Architecture (Layered)

```
┌────────────────────────────────────┐
│     APPLICATION LAYER              │
│  editor, terrain, scripting, etc.  │
└────────────┬───────────────────────┘
             │ depends on
             ▼
┌────────────────────────────────────┐
│        ENGINE LAYER                │
│  graphics, ecs, physics, scene,    │
│  audio, spatial, assets, gui       │
└────────────┬───────────────────────┘
             │ depends on
             ▼
┌────────────────────────────────────┐
│       PLATFORM LAYER               │
│    window, input                   │
└────────────┬───────────────────────┘
             │ depends on
             ▼
┌────────────────────────────────────┐
│      FOUNDATION LAYER              │
│   core, utils, math                │
└────────────────────────────────────┘
```

**Rule**: Higher layers depend on lower layers only (no reverse dependencies).

### Foundation Layer Design

#### praxis_utils (No Dependencies)

**Responsibilities:**
- Logging (`tracing`)
- Error handling (`color-eyre`)
- Filesystem utilities
- Timing/clocks

**Why Independent:**
- Needed by all crates
- No domain logic
- Pure utilities

#### praxis_math (No Dependencies)

**Responsibilities:**
- Thin wrapper around `glam`
- Re-export math types (Vec3, Mat4, Quat)
- Engine-specific math helpers

**Why Independent:**
- Needed by graphics, physics, scene, spatial
- No side effects
- Pure functions

**Anti-pattern Avoided:**
```rust
// BAD: math depending on ECS
impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        // Transform is ECS component!
        // math should not depend on ecs
    }
}

// GOOD: math is pure
pub fn transform_to_matrix(pos: Vec3, rot: Quat, scale: Vec3) -> Mat4 {
    // Pure function, no dependencies
}
```

#### praxis_core (Minimal Dependencies)

**Current Dependencies:**
- `praxis_utils` ✅
- `praxis_window` ✅
- `praxis_graphics` ✅
- `praxis_ecs` ✅
- `praxis_input` ✅
- `praxis_audio` ✅

**Problem**: Core depends on almost everything - becomes "god crate"

**Analysis**:

**Option A: Core as Integration Layer (Current)**
```rust
// Core brings subsystems together
pub struct Engine {
    window: Window,
    graphics: RenderContext,
    ecs: World,
    audio: AudioManager,
}

impl Engine {
    pub fn run(&mut self) {
        // Main loop integrates all systems
    }
}
```

**Pros:**
- ✅ Convenient: One struct to rule them all
- ✅ Clear entry point for users

**Cons:**
- ❌ Core becomes bottleneck (depends on everything)
- ❌ Changing any subsystem rebuilds core
- ❌ Hard to use subsystems independently

**Option B: Core as Lifecycle Only (Recommended)**
```rust
// Core provides lifecycle hooks, no subsystem dependencies
pub trait EngineSubsystem {
    fn initialize(&mut self) -> Result<()>;
    fn update(&mut self, delta: f32);
    fn shutdown(&mut self);
}

pub struct EngineRunner {
    subsystems: Vec<Box<dyn EngineSubsystem>>,
}

// Users compose in their own code:
// let mut engine = EngineRunner::new();
// engine.add(WindowSubsystem::new());
// engine.add(GraphicsSubsystem::new(&window));
// engine.run();
```

**Pros:**
- ✅ Core has zero subsystem dependencies
- ✅ Users compose only needed parts
- ✅ Subsystems independently testable

**Cons:**
- ⚠️ More setup code for users
- ⚠️ Manual integration needed

### Graphics-Window Relationship

**Current**:
```
praxis_graphics depends on praxis_window (for Surface creation)
```

**Problem**: Graphics tightly coupled to windowing

**Solution**: Abstract surface creation

```rust
// In praxis_graphics
pub trait SurfaceProvider {
    fn create_surface(&self, instance: &Arc<Instance>) -> Result<Arc<Surface>>;
}

// In praxis_window
impl SurfaceProvider for Window {
    fn create_surface(&self, instance: &Arc<Instance>) -> Result<Arc<Surface>> {
        // winit-specific implementation
    }
}

// In praxis_graphics
impl RenderContext {
    pub fn new(surface_provider: &impl SurfaceProvider) -> Result<Self> {
        // No direct window dependency
    }
}
```

**Result**: Graphics depends on trait, not concrete window type
- Enables headless rendering
- Allows alternative windowing backends
- Testable without window

### ECS-Graphics Relationship

**Problem**: Where do rendering components live?

**Option A: Components in ECS Crate (Current)**
```rust
// praxis_ecs/src/components.rs
pub struct MeshComponent {
    pub mesh: Arc<Mesh>,  // Arc<Mesh> from praxis_graphics
    pub material: MaterialId,
}
```

**Issue**: ECS depends on Graphics types
- ❌ Circular if Graphics queries ECS components
- ❌ Changes to Mesh force ECS rebuild

**Option B: Components in Graphics Crate**
```rust
// praxis_graphics/src/components.rs
pub struct MeshComponent {
    pub mesh: Arc<Mesh>,
    pub material: MaterialId,
}

// ECS doesn't know about rendering components
```

**Issue**: Components separated from other ECS components
- ❌ Confusing: Some components in ECS, some in Graphics
- ❌ Hard to discover rendering components

**Option C: Handle-Based Indirection (Recommended)**
```rust
// praxis_graphics/src/handles.rs
#[derive(Component, Copy, Clone)]
pub struct MeshHandle(pub u64);

#[derive(Component, Copy, Clone)]
pub struct MaterialHandle(pub u64);

// praxis_graphics manages actual Mesh/Material data internally
// ECS only stores lightweight handles

// No circular dependency:
// - ECS provides Component trait
// - Graphics uses Component derive macro
// - Graphics stores actual data
```

**Benefits**:
- ✅ ECS stores lightweight handles (Copy types)
- ✅ Graphics owns heavy data (meshes, textures)
- ✅ No circular dependencies
- ✅ Clear ownership model

### Dependency Rules

#### Layer Rules

1. **Foundation Layer** (`utils`, `math`, `core`)
   - **Allowed dependencies**: None (except `core` → `utils`)
   - **Rationale**: Foundation used by everything

2. **Platform Layer** (`window`, `input`)
   - **Allowed dependencies**: Foundation layer
   - **Rationale**: Platform abstraction over OS

3. **Engine Layer** (`graphics`, `ecs`, `scene`, `physics`, `audio`, `assets`, `spatial`, `procedural`, `profiling`, `gui`)
   - **Allowed dependencies**: Foundation + Platform layers, other Engine crates
   - **Prohibited**: No dependencies on Application layer
   - **Rationale**: Core engine functionality

4. **Application Layer** (`editor`, `terrain`, `scripting`, `networking`)
   - **Allowed dependencies**: All layers
   - **Rationale**: Optional high-level features

#### Specific Rules

1. **utils and math have ZERO internal dependencies**
   - Exception: `math` can depend on `glam` (external)
   - Rationale: Prevent any possible circularity

2. **core has minimal dependencies**
   - Should only depend on: `utils`, `window` (maybe)
   - Should NOT depend on: `graphics`, `ecs`, domain crates
   - Rationale: Core is lifecycle, not integration

3. **ecs is independent of domain logic**
   - Provides: World, Component trait, Query, System
   - Should NOT provide: MeshComponent, Transform (these belong in scene/graphics)
   - Exception: Basic ECS-specific components (Entity, Parent, Children)

4. **graphics depends on graphics abstractions only**
   - Depends on: `math`, `utils`, `window` (for surface)
   - Exposes: Handles (MeshHandle, TextureHandle, MaterialHandle)
   - Should NOT depend on: High-level components (Transform, Camera)
   - Exception: Can provide component derive macro

5. **Use trait abstractions to break coupling**
   - Example: SurfaceProvider trait lets graphics avoid window dependency
   - Example: AssetLoader trait lets assets avoid file format dependencies

### Circular Dependency Detection

**Cargo enforces at compile time:**
```bash
# This will error if circular dependency exists
cargo build --workspace
```

**Visual check:**
```bash
# Generate dependency graph
cargo tree --workspace
cargo depgraph | dot -Tsvg > deps.svg
```

### Decision: **Strict Layered Architecture with Handle-Based Indirection**

**Key Decisions:**

1. **Foundation Layer (utils, math)**: Zero dependencies
   - Utilities and math are pure, no domain knowledge

2. **Core Refactor**: Remove subsystem dependencies
   - Core provides lifecycle hooks and traits
   - Users compose subsystems in application code
   - Enables independent subsystem usage

3. **Handle-Based Components**: Break ECS-Graphics circularity
   - Graphics exposes handles (MeshHandle, MaterialHandle, TextureHandle)
   - ECS stores handles as components
   - Graphics maintains handle → data mapping internally

4. **Trait Abstractions**: Break concrete dependencies
   - SurfaceProvider: Graphics doesn't directly depend on Window
   - AssetLoader: Assets don't depend on file format crates
   - Enables testing, alternative implementations

5. **Cargo Workspace Enforcement**:
   - Workspace dependency resolution catches cycles at compile time
   - CI fails on circular dependencies

**Rationale:**

- **Compilation Speed**: Layered architecture maximizes parallel compilation
- **Testability**: Independent crates testable in isolation
- **Reusability**: Graphics, ECS, Math usable in other projects
- **Maintainability**: Clear responsibility, no hidden coupling
- **Reliability**: Cargo enforces architecture at compile time

**Trade-offs Accepted:**
- More abstraction (traits for decoupling)
- Handle indirection (negligible performance cost, better design)
- Manual composition (more setup code, but clearer)

**Implementation Tasks:**

1. Refactor `praxis_core` to remove graphics/ecs/audio dependencies
2. Move integration to `praxis` root crate or examples
3. Implement handle-based component pattern in graphics
4. Add SurfaceProvider trait to decouple window/graphics
5. Document dependency rules in CLAUDE.md
6. Add CI check for dependency graph visualization

---

## ADR-005: Feature Flags - Optional vs Required Subsystems

### Context

19 crates, not all needed for every project. Must decide:
- Which subsystems are optional (feature flags)?
- Which are required (always compiled)?
- What's the default experience?

### Current Feature Structure

```toml
[features]
default = []
editor = ["praxis_editor"]
networking = ["praxis_networking"]
scripting = ["praxis_scripting", "praxis_gui/scripting"]
terrain = ["praxis_terrain", "praxis_editor?/terrain"]
headless = []
```

**Optional (Feature-Gated):**
- `praxis_editor` - Editor tools
- `praxis_networking` - Multiplayer
- `praxis_scripting` - Lua integration
- `praxis_terrain` - Terrain generation

**Always Included:**
- Core: `praxis_core`, `praxis_utils`, `praxis_math`
- Platform: `praxis_window`, `praxis_input`
- Engine: `praxis_graphics`, `praxis_ecs`, `praxis_scene`, `praxis_spatial`, `praxis_assets`, `praxis_physics`, `praxis_audio`, `praxis_procedural`, `praxis_profiling`, `praxis_gui`

### Evaluation Per Subsystem

#### praxis_editor (Optional ✅)

**Rationale for Optional:**
- ✅ Games don't ship with editor
- ✅ Editor increases compile time (complex UI)
- ✅ Editor depends on many subsystems (coupling)
- ✅ Development-only tool

**Use Cases:**
- Tool developers: Enable
- Game developers: Disable for release builds
- Engine contributors: Enable for testing

**Decision: KEEP OPTIONAL**

**Consideration**: Should default include editor?
- **No**: Default should be minimal runtime
- **Yes**: New users expect editor
- **Compromise**: Document editor feature prominently

#### praxis_networking (Optional ✅)

**Rationale for Optional:**
- ✅ Not all games are multiplayer
- ✅ Large dependency (`tokio` async runtime)
- ✅ Complex implementation (increases compile time)
- ✅ Security surface (not needed for single-player)

**Use Cases:**
- Multiplayer games: Enable
- Single-player games: Disable
- Prototypes: Usually disable

**Impact:**
- Compile time: +30-45s (tokio is large)
- Binary size: +2-3 MB
- Dependencies: tokio, serde, bincode

**Decision: KEEP OPTIONAL**

**Alternative Considered**: Make networking required
- Rejected: Too many single-player use cases
- Rejected: Tokio is heavyweight dependency

#### praxis_scripting (Optional ✅)

**Rationale for Optional:**
- ✅ Not all games use scripting
- ✅ Some devs prefer pure Rust
- ✅ Adds Lua VM overhead
- ⚠️ Security concerns (sandbox needed)

**Use Cases:**
- Moddable games: Enable
- Rapid prototyping: Enable
- Pure Rust games: Disable
- Performance-critical: Disable

**Impact:**
- Compile time: +15-20s (mlua)
- Binary size: +1-2 MB (Lua VM)
- Runtime overhead: Minimal if not used

**Decision: KEEP OPTIONAL**

**Consideration**: Educational value of scripting integration
- Teaches: Engine-script bridge, hot-reload, sandboxing
- But: Not fundamental to engine architecture
- Conclusion: Optional but well-documented

#### praxis_terrain (Optional ✅)

**Rationale for Optional:**
- ✅ Not all games use terrain
- ✅ Indoor games don't need it
- ✅ Specialized system (not general-purpose)
- ✅ Can increase compile time

**Use Cases:**
- Open-world games: Enable
- Indoor games: Disable
- Space games: Disable
- Terrain-focused projects: Enable

**Impact:**
- Compile time: +5-10s
- Binary size: +500 KB
- Coupling: Depends on graphics, scene

**Decision: KEEP OPTIONAL**

**Alternative Considered**: Include in default
- Rejected: Too specialized
- Rejected: Not needed for many game types

#### praxis_profiling (Required ✅)

**Rationale for Required:**
- ✅ Performance monitoring essential for game dev
- ✅ Minimal overhead when not actively profiling
- ✅ Small crate (fast compilation)
- ✅ Educational: Teaches performance awareness

**Use Cases:**
- All development: Always useful
- Release builds: Can compile out with cfg flags
- Debugging: Essential

**Impact:**
- Compile time: +2-5s (minimal)
- Runtime overhead: Zero unless enabled
- Binary size: +100 KB

**Decision: KEEP REQUIRED**

**Alternative Considered**: Make optional
- Rejected: Too useful, too small
- Rejected: Want to encourage performance awareness

#### praxis_gui (Required, but should it be?)

**Current**: Always included

**Arguments for Optional:**
- Not all games need debug GUI
- Headless servers don't need GUI
- `egui` adds dependency weight
- Some devs prefer external tools

**Arguments for Required:**
- Profiling needs GUI visualization
- Editor needs GUI
- Debug overlays common in games
- Small overhead if not rendered

**Impact:**
- Compile time: +10-15s (egui)
- Binary size: +800 KB
- Dependencies: egui, egui-winit, egui_vulkano

**Decision: CONSIDER MAKING OPTIONAL**

**Proposed Feature Structure:**
```toml
[features]
default = ["gui"]
gui = ["praxis_gui"]
editor = ["praxis_editor", "gui"]
```

**Rationale:**
- Default includes GUI (most users want it)
- Can disable for headless/minimal builds
- Editor requires GUI (using `"gui"` in its dependencies)

#### praxis_physics (Required, but should it be?)

**Current**: Always included

**Arguments for Optional:**
- Not all games need physics (puzzle, turn-based)
- `rapier3d` is large dependency
- 2D games might want different physics

**Arguments for Required:**
- Most 3D games use physics
- Collision detection widely needed
- Teaches important game engine concept

**Impact:**
- Compile time: +20-30s (rapier3d)
- Binary size: +2-3 MB
- Dependencies: rapier3d, parry3d

**Decision: CONSIDER MAKING OPTIONAL**

**Proposed Feature Structure:**
```toml
[features]
default = ["physics"]
physics = ["praxis_physics"]
```

**Rationale:**
- Default includes physics (most 3D games need it)
- Can disable for physics-free games (rare but valid)
- Significant compile time savings if disabled

#### praxis_audio (Required, but should it be?)

**Current**: Always included

**Arguments for Optional:**
- Headless servers don't need audio
- Some games are silent (abstract, puzzle)
- `kira` adds dependency

**Arguments for Required:**
- Almost all games have audio
- Small overhead
- Important for game feel

**Impact:**
- Compile time: +10-15s
- Binary size: +1 MB
- Dependencies: kira, symphonia

**Decision: KEEP REQUIRED**

**Rationale:**
- Audio almost universal in games
- Overhead acceptable
- Can be disabled at runtime if not needed
- Not worth complicating feature flags

#### praxis_procedural (Required, questionable)

**Current**: Always included

**Arguments for Optional:**
- Not all games use procedural textures
- Adds complexity (GLSL compilation)
- Games can use pre-made assets

**Arguments for Required:**
- Useful for prototyping (quick textures)
- Teaches important technique
- Showcases GPU compute

**Impact:**
- Compile time: +8-12s
- Binary size: +600 KB
- Dependencies: shaderc (shader compilation)

**Decision: CONSIDER MAKING OPTIONAL**

**Proposed Feature Structure:**
```toml
[features]
default = ["procedural"]
procedural = ["praxis_procedural", "praxis_graphics/procedural"]
```

**Rationale:**
- Default includes it (useful for examples)
- Can disable for asset-only games
- Saves shaderc compilation if not needed

### Proposed Feature Structure

```toml
[features]
# Default: Reasonable set for most games
default = ["gui", "physics", "procedural", "audio"]

# Core optional features
gui = ["praxis_gui"]
physics = ["praxis_physics"]
procedural = ["praxis_procedural", "praxis_graphics/procedural"]
audio = ["praxis_audio"]

# Advanced optional features (not in default)
editor = ["praxis_editor", "gui"]
networking = ["praxis_networking"]
scripting = ["praxis_scripting", "praxis_gui?/scripting"]
terrain = ["praxis_terrain", "praxis_editor?/terrain"]

# Special build modes
headless = []  # Disable rendering (for dedicated servers)
minimal = []   # No optional features (smallest build)

# Convenience feature sets
full = ["gui", "physics", "procedural", "audio", "editor", "networking", "scripting", "terrain"]
game = ["gui", "physics", "procedural", "audio", "scripting", "networking"]
```

### Feature Combinations

**Common Use Cases:**

1. **Minimal Single-Player Game**
   ```bash
   cargo build --no-default-features --features "gui,physics"
   ```
   Result: GUI, physics, no audio, no procedural, no scripting

2. **Multiplayer Game**
   ```bash
   cargo build --features "game"
   ```
   Result: All common game features (GUI, physics, audio, scripting, networking)

3. **Full Development Build**
   ```bash
   cargo build --features "full"
   # or
   cargo build --all-features
   ```
   Result: Everything including editor and terrain

4. **Dedicated Server**
   ```bash
   cargo build --no-default-features --features "headless,physics,networking"
   ```
   Result: No GUI, no audio, just networking and physics

5. **Quick Prototype**
   ```bash
   cargo build --features "default"
   ```
   Result: GUI, physics, procedural, audio (good balance)

### Compile Time Impact

**Estimated clean build times** (M1 Max, 10 cores):

| Feature Set | Time | Components |
|-------------|------|------------|
| `minimal` | ~45s | Core, window, graphics, ECS, scene, spatial, assets, input, utils, math |
| `default` | ~90s | + GUI, physics, procedural, audio |
| `game` | ~120s | + Scripting, networking |
| `full` | ~150s | + Editor, terrain |

**Incremental build times**: ~5-30s depending on changed crate

### Binary Size Impact

**Estimated release binary sizes** (x86_64, `--release`, stripped):

| Feature Set | Size | Difference |
|-------------|------|------------|
| `minimal` | ~8 MB | Baseline |
| `default` | ~15 MB | +7 MB (GUI, physics, procedural, audio) |
| `game` | ~20 MB | +5 MB (scripting, networking) |
| `full` | ~25 MB | +5 MB (editor, terrain) |

### Dependency Impact

**Heavy Dependencies:**

- `tokio` (networking): +40s compile, +2 MB binary
- `rapier3d` (physics): +25s compile, +2 MB binary
- `mlua` (scripting): +15s compile, +1 MB binary
- `egui` (GUI): +12s compile, +800 KB binary
- `kira` (audio): +10s compile, +1 MB binary
- `shaderc` (procedural): +8s compile, +600 KB binary

### Default Feature Selection

**Philosophy**: Default should be:
1. Useful for most games
2. Reasonable compile time (~90s clean build)
3. Educational (shows common features)
4. Not overwhelming for beginners

**Chosen Default:**
```toml
default = ["gui", "physics", "procedural", "audio"]
```

**Rationale:**
- GUI: Debug tools, profiling visualization
- Physics: Most 3D games need collision at minimum
- Procedural: Useful for prototyping, teaches GPU compute
- Audio: Almost universal in games

**Not in Default:**
- Editor: Development tool, not needed for games
- Networking: Not all games are multiplayer
- Scripting: Not all games need scripting
- Terrain: Specialized system

**Beginners can enable more:**
```bash
# Enable editor for development
cargo build --features "editor"

# Enable everything
cargo build --all-features
```

### Documentation Impact

**Must Document:**

1. **Feature matrix** in README:
   ```markdown
   ## Features
   
   - `gui` - Debug UI and profiling visualization (default)
   - `physics` - 3D physics with Rapier (default)
   - `audio` - 3D spatial audio (default)
   - `procedural` - Procedural texture generation (default)
   - `editor` - Editor tools and workflows
   - `scripting` - Lua scripting integration
   - `networking` - Multiplayer networking
   - `terrain` - Terrain generation and rendering
   ```

2. **Common builds** in docs:
   - Minimal build
   - Game build
   - Full build
   - Server build

3. **Feature dependencies** (editor requires gui, etc.)

4. **Compile time expectations** per feature set

### Decision: **Refine Feature Flags for Flexibility**

**Changes from Current:**

1. **Make GUI optional** (but in default)
   ```toml
   default = ["gui", "physics", "procedural", "audio"]
   gui = ["praxis_gui"]
   ```

2. **Make physics optional** (but in default)
   ```toml
   physics = ["praxis_physics"]
   ```

3. **Make procedural optional** (but in default)
   ```toml
   procedural = ["praxis_procedural"]
   ```

4. **Keep audio required** (for now)
   - Too universal, too small
   - Can revisit if feedback suggests otherwise

5. **Add convenience feature sets**
   ```toml
   full = ["gui", "physics", "procedural", "audio", "editor", "networking", "scripting", "terrain"]
   game = ["gui", "physics", "procedural", "audio", "scripting", "networking"]
   minimal = []  # Just use --no-default-features
   ```

**Rationale:**

1. **Flexibility**: Users can disable heavy dependencies
2. **Defaults**: Sensible for most use cases
3. **Education**: Full feature set demonstrates all capabilities
4. **Performance**: Minimal builds for specific needs (servers, benchmarks)
5. **Maintainability**: Clear feature boundaries

**Trade-offs Accepted:**
- More feature flag complexity (manageable with docs)
- Must test combinations (CI tests default, full, minimal)
- Feature flag propagation (editor/terrain must handle optional deps)

**Implementation Tasks:**

1. Update root `Cargo.toml` with new feature structure
2. Make `praxis_gui`, `praxis_physics`, `praxis_procedural` optional
3. Add conditional compilation in dependent crates
4. Update documentation (README, CLAUDE.md)
5. CI: Test `default`, `full`, `--no-default-features`, `game`
6. Update examples to specify required features

---

## Summary of Decisions

| Decision Area | Choice | Rationale |
|---------------|--------|-----------|
| **Graphics Backend** | vulkano | Direct Vulkan control, educational value, feature access, existing investment |
| **Crate Count** | Keep 19 crates | Architectural clarity, compilation speed, discoverability, flexibility |
| **ECS Framework** | bevy_ecs | Best performance, features, ergonomics, community, production-proven |
| **Dependency Architecture** | Strict layers + handles | Prevent circular deps, enable independent testing, maximize parallel builds |
| **Optional Features** | Refined flags | GUI/physics/procedural optional but in default, advanced features opt-in |

### Principles Guiding Decisions

1. **Education First**: Decisions prioritize learning value (Vulkan exposure, clear architecture)
2. **Pragmatism**: Use battle-tested libraries (bevy_ecs, vulkano, glam, rapier3d)
3. **Flexibility**: Modular design allows using only needed parts
4. **Performance**: Architecture enables optimization (layered compilation, zero-cost abstractions)
5. **Maintainability**: Clear boundaries, enforced by cargo, documented
6. **Developer Experience**: Fast incremental builds, reasonable defaults, good docs

### Related Documentation

- [Crate Dependency Graph](crate-dependency-graph.md) - Visual dependency structure
- [Architecture Overview](../architecture.md) - High-level design
- [CLAUDE.md](../../CLAUDE.md) - Development guidance
- [Project Structure](../getting-started/project-structure.md) - Workspace layout

### Revision History

| Date | Changes | Reason |
|------|---------|--------|
| 2024 | Initial document | Evaluate rebuild decisions |

---

*This is a living document. Decisions may be revisited as engine matures and requirements change.*
