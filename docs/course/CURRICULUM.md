# Game Engine Architecture Curriculum

A language-agnostic educational framework for understanding fundamental game engine concepts through the Praxis engine architecture.

## Overview

This curriculum provides a structured approach to learning game engine design principles. Each module focuses on universal concepts applicable across programming languages and platforms, using Praxis as a reference implementation to illustrate these patterns in practice.

**Target Audience**: Engine developers, graphics programmers, technical artists, and students learning game engine architecture.

**Prerequisites**: Basic programming knowledge, linear algebra fundamentals (vectors, matrices), and understanding of 3D coordinate systems.

## Curriculum Structure

The curriculum consists of 12 concept-focused modules organized into four tiers:

- **Foundation** (Modules 1-3): Core engine architecture and runtime fundamentals
- **Rendering** (Modules 4-5): Graphics pipeline and visual systems
- **Simulation** (Modules 6-8): Physics, animation, and data management
- **Integration** (Modules 9-12): Extended systems and production workflows

---

## Module 1: Game Loop Fundamentals

**Duration**: 2-3 weeks  
**Complexity**: Beginner

### Learning Objectives

By the end of this module, students will be able to:

1. Explain the purpose and structure of a game loop
2. Compare fixed vs. variable timestep approaches and their trade-offs
3. Implement basic frame timing and delta time calculations
4. Describe the update/render separation pattern
5. Identify common game loop anti-patterns
6. Analyze frame budget allocation for 60 FPS targets

### Core Concepts

#### The Game Loop Pattern

- **Initialization**: One-time setup of engine subsystems
- **Main Loop**: Continuous execution cycle
- **Shutdown**: Clean resource deallocation

#### Timestep Strategies

- **Variable Timestep**: Update based on actual elapsed time
  - Pros: Smooth rendering, simple implementation
  - Cons: Non-deterministic simulation, physics instability
- **Fixed Timestep**: Update in constant time increments
  - Pros: Deterministic physics, stable simulation
  - Cons: Requires interpolation for rendering
- **Semi-Fixed**: Fixed physics step with variable rendering
  - Pros: Best of both worlds
  - Cons: More complex implementation

#### Event Processing

- **Polling vs. Event-Driven**: When to check state vs. react to changes
- **Input Buffering**: Managing input across frame boundaries
- **Event Queue Management**: Ordering and priority

### Praxis Implementation Reference

```text
Core Loop in Praxis:
  1. Event handling (window, input)
  2. Fixed timestep physics updates (60 Hz)
  3. Variable timestep game logic
  4. Transform hierarchy updates
  5. Rendering pipeline
  6. Frame synchronization
```

**Relevant Code**: `praxis_core`, `praxis_window`, `praxis_utils::timing`

### Assessment Criteria

- Can implement a basic game loop with proper timing
- Can explain why physics requires fixed timesteps
- Can calculate frame budgets and identify bottlenecks
- Can debug common timing-related issues (spiral of death, frame drops)

### Advanced Topics

- Frame pacing techniques
- Multi-threaded update loops
- Adaptive quality based on performance
- Frame rate independent game logic

---

## Module 2: Rendering Architecture Patterns

**Duration**: 3-4 weeks  
**Complexity**: Intermediate

### Learning Objectives

By the end of this module, students will be able to:

1. Compare immediate mode vs. retained mode rendering architectures
2. Design a command buffer system for deferred rendering
3. Explain GPU-driven rendering and its benefits
4. Implement a render graph abstraction
5. Describe shader compilation and pipeline state management
6. Analyze rendering performance and identify GPU bottlenecks

### Core Concepts

#### Rendering Paradigms

- **Immediate Mode**: Direct rendering commands
  - Simple mental model
  - Difficult to optimize
  - Legacy approach
- **Retained Mode**: Scene graph with deferred rendering
  - Better batching opportunities
  - Easier multi-threading
  - Modern approach
- **Hybrid Approaches**: Command lists with state caching

#### Graphics API Abstraction Layers

- **Low-Level (Vulkan, DirectX 12, Metal)**
  - Explicit control over GPU resources
  - Manual synchronization
  - Verbose but performant
- **Mid-Level (DirectX 11, OpenGL)**
  - Implicit state management
  - Simpler API surface
  - Less control
- **High-Level (Game Engine APIs)**
  - Material-based rendering
  - Automatic batching
  - Platform abstraction

#### Pipeline State Objects

- **What They Contain**: Shaders, vertex input, rasterization state, blending, depth/stencil
- **Why They're Expensive**: Validation, compilation, driver overhead
- **Caching Strategies**: Pre-compilation, runtime compilation, persistent caches

#### Render Passes and Framebuffers

- **Single-Pass Rendering**: Forward rendering
- **Multi-Pass Rendering**: Deferred rendering, shadow mapping, post-processing
- **Render Target Management**: Color, depth, stencil attachments
- **Load/Store Operations**: Optimizing memory bandwidth

### Praxis Implementation Reference

```text
Rendering Flow:
  1. Command buffer recording (CPU)
  2. Descriptor set binding (uniforms, textures)
  3. Pipeline state binding
  4. Draw call submission
  5. GPU execution (async)
  6. Swapchain presentation
  7. Frame synchronization (fences, semaphores)
```

**Relevant Code**: `praxis_graphics::RenderContext`, Vulkano abstractions

### Assessment Criteria

- Can design a rendering architecture suitable for a specific game genre
- Can explain the costs of different rendering approaches
- Can implement basic draw call batching
- Can profile rendering performance and identify optimization opportunities

### Advanced Topics

- Bindless rendering
- GPU-driven culling and LOD
- Virtual shadow maps
- Nanite-style geometry streaming

---

## Module 3: Entity Management Systems

**Duration**: 3-4 weeks  
**Complexity**: Intermediate

### Learning Objectives

By the end of this module, students will be able to:

1. Compare object-oriented vs. data-oriented game architectures
2. Implement a basic ECS (Entity-Component-System)
3. Explain archetype-based storage and its performance benefits
4. Design systems with optimal data access patterns
5. Manage entity lifetimes and handle cleanup
6. Analyze cache efficiency in component storage

### Core Concepts

#### Architecture Patterns

- **Object-Oriented Hierarchy**
  - Inheritance-based game objects
  - Virtual functions and polymorphism
  - Pros: Familiar, encapsulation
  - Cons: Deep hierarchies, cache misses, inflexible
- **Component-Based Architecture**
  - Composition over inheritance
  - Components as data bags
  - Game objects as containers
  - Pros: Flexible, reusable
  - Cons: Message passing overhead
- **Entity-Component-System (ECS)**
  - Entities as IDs
  - Components as pure data
  - Systems as pure logic
  - Pros: Cache-friendly, parallel-friendly, flexible
  - Cons: Less intuitive, indirection

#### ECS Implementation Strategies

- **Table-Based Storage**: Components organized by archetype
- **Sparse Set Storage**: O(1) component lookup
- **Hybrid Approaches**: Different storage for different component types

#### Archetype Storage

```text
Archetype: (Transform, Mesh, Rigidbody)
┌─────────────────────────────────────┐
│ Entity IDs:  [E0] [E1] [E2] [E3]    │
│ Transforms:  [T0] [T1] [T2] [T3]    │  ← Contiguous in memory
│ Meshes:      [M0] [M1] [M2] [M3]    │  ← Cache-friendly iteration
│ Rigidbodies: [R0] [R1] [R2] [R3]    │
└─────────────────────────────────────┘
```

#### Query Patterns

- **Iteration**: Processing all entities with specific components
- **Filtering**: Excluding components (Without)
- **Change Detection**: Only entities with modified components
- **Optional Components**: Handle presence/absence gracefully

### Praxis Implementation Reference

```text
Praxis uses bevy_ecs:
  - Archetype-based storage
  - Parallel system execution
  - Change detection
  - Query caching
  - Generational entity IDs
```

**Relevant Code**: `praxis_ecs`, `bevy_ecs` integration

### Assessment Criteria

- Can explain why ECS improves cache performance
- Can design components with appropriate granularity
- Can write systems that avoid unnecessary queries
- Can profile ECS system performance
- Can handle entity spawning/despawning correctly

### Advanced Topics

- Parallel system scheduling
- System ordering and dependencies
- Entity relationships (graphs, hierarchies)
- Component serialization and reflection
- Networked ECS replication

---

## Module 4: Transform Hierarchies

**Duration**: 2-3 weeks  
**Complexity**: Intermediate

### Learning Objectives

By the end of this module, students will be able to:

1. Implement parent-child transform relationships
2. Explain local vs. world space transformations
3. Optimize transform propagation for large hierarchies
4. Handle transform caching and dirty flags
5. Integrate transforms with rendering and physics
6. Debug common transform issues (gimbal lock, scale inheritance)

### Core Concepts

#### Transform Representation

- **Translation**: Position in 3D space (Vec3)
- **Rotation**: Orientation
  - Euler Angles: Intuitive but gimbal lock
  - Quaternions: No gimbal lock, efficient interpolation
  - Rotation Matrices: Direct multiplication, 9 components
- **Scale**: Size multiplier (Vec3 or scalar)

#### Hierarchical Transforms

- **Local Transform**: Relative to parent
- **World Transform**: Absolute position in world
- **Propagation**: Parent * Child = World

```text
Parent (world)        Child (local)        Child (world)
Translation: (5,0,0)  Translation: (0,2,0) Translation: (5,2,0)
Rotation: 90° Y       Rotation: 0°         Rotation: 90° Y
```

#### Update Strategies

- **Eager Propagation**: Update all descendants immediately
  - Simple, always up-to-date
  - Wasted work if multiple parent changes
- **Lazy Propagation**: Mark dirty, update on read
  - Minimal work
  - Complex bookkeeping
- **Batched Propagation**: Update once per frame
  - Balance simplicity and efficiency
  - Used by most engines

#### Common Patterns

- **Attach Points**: Weapons, cameras, particles
- **Bone Hierarchies**: Skeletal animation
- **Scene Graphs**: Level organization
- **Physics Synchronization**: Rigidbody to transform

### Praxis Implementation Reference

```text
Transform Components:
  - Transform: Local position, rotation, scale
  - GlobalTransform: Cached world matrix
  - Parent: Entity ID of parent
  - Children: Vec of child entity IDs

Propagation:
  1. Detect changed root transforms
  2. Compute GlobalTransform = Transform.to_matrix()
  3. Recursively update children: Child.global = Parent.global * Child.local
```

**Relevant Code**: `praxis_scene::transform`, ECS systems for propagation

### Assessment Criteria

- Can implement transform hierarchy updates
- Can explain matrix multiplication order
- Can optimize for minimal recalculations
- Can debug transform issues (wrong space, missing updates)
- Can handle edge cases (orphaned entities, cycles)

### Advanced Topics

- Transform interpolation for networking
- Constraint systems (look-at, follow)
- Inverse kinematics (IK)
- Transform animation blending

---

## Module 5: Physics Integration Strategies

**Duration**: 3-4 weeks  
**Complexity**: Intermediate to Advanced

### Learning Objectives

By the end of this module, students will be able to:

1. Integrate a third-party physics engine with an ECS
2. Implement bidirectional transform synchronization
3. Explain collision detection algorithms (broad phase, narrow phase)
4. Design character controllers with physics
5. Handle physics simulation timing (fixed timestep)
6. Optimize physics performance for many objects

### Core Concepts

#### Physics Engine Architecture

- **Rigid Body Dynamics**: Forces, velocities, momentum
- **Collision Detection**
  - Broad Phase: Spatial partitioning (grid, octree, BVH)
  - Narrow Phase: Shape intersection tests
- **Constraint Solver**: Resolving overlaps, joints
- **Integration**: Euler, Verlet, Runge-Kutta methods

#### Transform Synchronization

```text
Two-Way Sync:
  Physics → Transform: Dynamic rigidbodies update game entities
  Transform → Physics: Kinematic objects driven by animation
```

- **Dynamic Bodies**: Physics controls transform
- **Kinematic Bodies**: Transform controls physics
- **Static Bodies**: Never move

#### Fixed Timestep Necessity

- **Determinism**: Same inputs = same outputs
- **Stability**: Prevents tunneling, explosion
- **Implementation**: Accumulator pattern

```text
accumulator += delta_time
while accumulator >= FIXED_DT {
    physics.step(FIXED_DT)
    accumulator -= FIXED_DT
}
```

#### Character Controllers

- **Capsule Collider**: Smooth movement over steps
- **Ground Detection**: Raycasting, sphere casting
- **Slope Handling**: Max angle, slide vs. stick
- **Step Offset**: Auto-climb small obstacles

### Praxis Implementation Reference

```text
Praxis + Rapier3D:
  1. sync_transforms_to_physics (kinematic entities)
  2. physics.step(1/60)
  3. sync_transforms_from_physics (dynamic entities)
  4. handle_collision_events
```

**Relevant Code**: `praxis_physics`, Rapier3D integration

### Assessment Criteria

- Can integrate physics with an existing ECS
- Can implement stable character movement
- Can debug physics issues (tunneling, jitter, instability)
- Can profile physics performance
- Can design physics-driven gameplay mechanics

### Advanced Topics

- Continuous collision detection (CCD)
- Ragdoll physics
- Soft body simulation
- Fluid simulation integration
- Networking physics (client prediction, server reconciliation)

---

## Module 6: Asset Pipeline Design

**Duration**: 3-4 weeks  
**Complexity**: Intermediate

### Learning Objectives

By the end of this module, students will be able to:

1. Design an asset loading and caching system
2. Implement asynchronous asset loading
3. Explain common 3D file formats (GLTF, OBJ, FBX)
4. Optimize asset loading for different platforms
5. Handle hot-reload for rapid iteration
6. Design asset dependency graphs

### Core Concepts

#### Asset Types

- **Meshes**: Vertices, indices, attributes
- **Textures**: Formats (PNG, JPG, DDS, KTX), compression
- **Materials**: Shaders, parameters, textures
- **Audio**: Formats (WAV, MP3, OGG), streaming
- **Scenes**: Hierarchies, prefabs
- **Scripts**: Source code, bytecode

#### Loading Strategies

- **Synchronous Loading**
  - Simple, blocking
  - Causes frame hitches
  - Only for small assets or load screens
- **Asynchronous Loading**
  - Non-blocking, uses threads or async/await
  - Requires streaming architecture
  - Modern approach
- **Streaming**
  - Progressive loading (LODs)
  - Memory budget management
  - Essential for open worlds

#### Caching and Lifetime Management

- **Asset Handles**: Reference-counted IDs
- **Reference Counting**: Automatic unload when unused
- **Manual Management**: Explicit load/unload
- **Weak References**: Allow garbage collection

#### File Formats

- **GLTF**: Modern, extensible, supports PBR
- **OBJ**: Simple, widespread, no animation
- **FBX**: Industry standard, complex, proprietary
- **Custom Formats**: Optimized for runtime, fast loading

### Praxis Implementation Reference

```text
Asset Loading:
  1. MeshLoader::load(path) → Parse file
  2. Upload to GPU (vertex/index buffers)
  3. Store handle in MeshManager
  4. Return handle to user

Async Pattern:
  - Background thread: File I/O, parsing
  - GPU upload: Main thread (Vulkan requirement)
  - Handle: Immediately available, status queryable
```

**Relevant Code**: `praxis_assets`, `MeshManager`, `TextureManager`

### Assessment Criteria

- Can implement basic asset loading
- Can parse a common file format (OBJ, GLTF)
- Can design async loading without race conditions
- Can optimize loading times
- Can implement hot-reload for development

### Advanced Topics

- Asset bundles and packaging
- Procedural asset generation
- Runtime asset compilation (shaders)
- Asset dependency tracking
- Cross-platform asset formats
- Virtual file systems

---

## Module 7: Memory Management Patterns

**Duration**: 2-3 weeks  
**Complexity**: Intermediate to Advanced

### Learning Objectives

By the end of this module, students will be able to:

1. Explain GPU memory types and their use cases
2. Implement efficient buffer allocation strategies
3. Design memory pools and allocators
4. Optimize for cache coherence
5. Profile memory usage and identify leaks
6. Handle out-of-memory conditions gracefully

### Core Concepts

#### GPU Memory Types

- **Device-Local**: VRAM, fastest for GPU, no CPU access
  - Use: Vertex buffers, index buffers, textures
- **Host-Visible**: RAM, CPU can map, slower GPU access
  - Use: Uniform buffers, staging buffers
- **Host-Coherent**: Automatically synced, no manual flushing
- **Host-Cached**: CPU reads optimized

#### Allocation Strategies

- **Naive**: Allocate per-object
  - Simple but slow
  - Fragmentation
  - Not production-ready
- **Pooling**: Pre-allocate large blocks
  - Faster allocation
  - Reduces fragmentation
  - Requires tuning
- **Ring Buffers**: Circular allocation for per-frame data
  - No per-frame allocation
  - Fixed memory footprint
  - Requires frame-in-flight tracking

#### Ring Buffer Pattern

```text
Frame 0    Frame 1    Frame 2
[------]   [------]   [------]
  CPU        GPU       Available
writing    reading
```

- 3 frames in flight: CPU writes frame N while GPU reads frame N-2
- No synchronization needed

#### Cache Optimization

- **Structure of Arrays (SoA)**: Components stored separately
  - Better cache utilization for iteration
  - Harder to manage
- **Array of Structures (AoS)**: Components grouped by entity
  - Easier to manage
  - More cache misses
- **Hybrid**: Hot/cold data separation

### Praxis Implementation Reference

```text
Vulkan Memory Management:
  - StandardMemoryAllocator: Pooling allocator
  - Buffer creation: Specify usage + memory type
  - Subbuffers: Typed views into buffers
  - Arc: Reference counting for shared resources
```

**Relevant Code**: `praxis_graphics::buffer`, Vulkano allocators

### Assessment Criteria

- Can choose appropriate memory types for different resources
- Can implement a ring buffer for uniform data
- Can profile memory usage
- Can optimize for cache performance
- Can debug memory leaks and over-allocation

### Advanced Topics

- Virtual memory and sparse resources
- Memory defragmentation
- Texture streaming and residency
- GPU-driven memory management
- Platform-specific optimizations (ReBAR, unified memory)

---

## Module 8: Input Abstraction

**Duration**: 1-2 weeks  
**Complexity**: Beginner to Intermediate

### Learning Objectives

By the end of this module, students will be able to:

1. Design an input abstraction layer
2. Implement action mapping systems
3. Handle multiple input devices (keyboard, mouse, gamepad)
4. Manage input state across frames
5. Integrate input with gameplay systems
6. Support input rebinding and device hot-swapping

### Core Concepts

#### Input Models

- **Raw Input**: Direct hardware events
  - Low-latency
  - Platform-specific
  - Hard to manage
- **Action Mapping**: Bind actions to inputs
  - "Jump" → Spacebar, A button, etc.
  - Rebindable
  - Higher-level abstraction
- **State vs. Events**
  - State: "Is button pressed?"
  - Event: "Button was just pressed this frame"

#### Device Types

- **Keyboard**: Digital inputs, text entry
- **Mouse**: 2D movement, buttons, scroll
- **Gamepad**: Analog sticks, triggers, buttons, rumble
- **Touch**: Multi-touch, gestures
- **VR Controllers**: 6DOF, buttons, haptics

#### Frame-Based Input

```text
Frame N:
  1. Poll device state
  2. Compare with previous frame
  3. Generate events (pressed, released, held)
  4. Process input in gameplay systems
  5. Store state for next frame
```

#### Action Mapping System

```text
Action: "Jump"
  Bindings:
    - Keyboard: Space
    - Gamepad: A button
    - Touch: Tap anywhere

When any binding triggered → Fire "Jump" action
```

### Praxis Implementation Reference

```text
Input System:
  - InputState: Current frame state
  - is_key_pressed(key) → bool
  - is_key_just_pressed(key) → bool (rising edge)
  - is_key_just_released(key) → bool (falling edge)
  - mouse_delta() → Vec2
  - gamepad support via winit
```

**Relevant Code**: `praxis_input`

### Assessment Criteria

- Can implement basic input handling
- Can design an action mapping system
- Can support multiple devices
- Can handle input edge cases (focus loss, device disconnect)

### Advanced Topics

- Input prediction for networking
- Dead zones and sensitivity curves
- Gesture recognition
- Accessibility features (remapping, auto-aim)
- Input recording and playback

---

## Module 9: Audio Architectures

**Duration**: 2 weeks  
**Complexity**: Intermediate

### Learning Objectives

By the end of this module, students will be able to:

1. Integrate an audio middleware (FMOD, Wwise, Kira)
2. Implement 3D spatial audio
3. Design audio resource management
4. Handle audio mixing and prioritization
5. Optimize audio performance
6. Synchronize audio with gameplay events

### Core Concepts

#### Audio Systems Architecture

- **Audio Engine**: Low-level playback (OpenAL, XAudio2)
- **Middleware**: High-level features (FMOD, Wwise, Kira)
- **Game Integration**: Trigger sounds, update positions

#### Audio Types

- **Sound Effects (SFX)**: One-shot, fire-and-forget
- **Music**: Looping, streaming, crossfade
- **Ambient**: Background, looping, 3D positioned
- **Voice/Dialogue**: Narrative, subtitles

#### Spatial Audio

- **Attenuation**: Volume reduction over distance
  - Linear, inverse, exponential curves
- **Doppler Effect**: Frequency shift for moving sources
- **Reverb**: Environmental acoustics
- **Occlusion**: Sound blocked by geometry

#### Resource Management

- **Memory**: Load all vs. streaming
- **Channels**: Limit simultaneous sounds
- **Prioritization**: Important sounds interrupt less important
- **Pooling**: Reuse audio sources

### Praxis Implementation Reference

```text
Audio System (Kira):
  - AudioManager: Resource loading, playback
  - 3D spatial positioning
  - Volume/pitch control
  - Attenuation settings
  - Background music management
```

**Relevant Code**: `praxis_audio`, Kira integration

### Assessment Criteria

- Can integrate audio middleware
- Can implement 3D positioned sounds
- Can manage audio resources efficiently
- Can design audio for gameplay feel

### Advanced Topics

- Dynamic music systems
- Audio DSP effects
- Voice chat integration
- Audio profiling and optimization
- Platform-specific audio (console certification)

---

## Module 10: Editor Architecture

**Duration**: 3-4 weeks  
**Complexity**: Advanced

### Learning Objectives

By the end of this module, students will be able to:

1. Design editor/runtime separation
2. Implement undo/redo systems
3. Create transform gizmos and manipulation tools
4. Design entity selection and picking
5. Implement serialization for scenes
6. Integrate immediate-mode GUI (ImGui, egui)

### Core Concepts

#### Editor vs. Runtime

- **Runtime Systems**: Core game logic
- **Editor Systems**: Tools, visualization, debugging
- **Separation Strategies**:
  - Conditional compilation (features)
  - Separate binaries
  - Editor-only components

#### Undo/Redo (Command Pattern)

```text
Command: SetTransform
  execute():
    old_transform = entity.transform
    entity.transform = new_transform
  undo():
    entity.transform = old_transform

Command Stack:
  [SetTransform, AddComponent, DeleteEntity, ...]
        ↑ current
  Undo: Execute current.undo(), move pointer back
  Redo: Move pointer forward, execute current
```

#### Selection System

- **Raycasting**: Pick objects with mouse
- **Multi-Selection**: Shift/Ctrl modifiers
- **Hierarchy Selection**: Parents and children
- **Visual Feedback**: Outline, highlight

#### Transform Gizmos

- **Move**: Translate along X/Y/Z axes
- **Rotate**: Rotate around axes
- **Scale**: Uniform or per-axis
- **Interaction**: Mouse drag, snapping

#### Immediate-Mode GUI

- **Pattern**: UI rebuilt every frame
- **Pros**: Simple, no state management
- **Cons**: Less efficient than retained mode
- **Libraries**: ImGui (C++), egui (Rust)

### Praxis Implementation Reference

```text
Editor Systems:
  - SelectionSystem: Raycasting, multi-select
  - UndoRedoSystem: Command history
  - EditorCamera: Orbit, pan, zoom
  - Inspector: Component property editing
  - Hierarchy: Entity tree view
  - Asset Browser: Drag-and-drop
```

**Relevant Code**: `praxis_editor`, `praxis_gui` (egui integration)

### Assessment Criteria

- Can implement basic editor features
- Can design undo/redo for complex operations
- Can create usable transform tools
- Can serialize/deserialize scenes
- Can integrate GUI framework

### Advanced Topics

- Multi-user collaboration
- Editor extensibility (plugins)
- Live game preview
- Profiling visualizations
- Custom property editors

---

## Module 11: Scripting Integration

**Duration**: 2-3 weeks  
**Complexity**: Intermediate to Advanced

### Learning Objectives

By the end of this module, students will be able to:

1. Integrate a scripting language (Lua, Python, Wren)
2. Design script-to-engine bindings
3. Implement hot-reload for rapid iteration
4. Handle script errors and sandboxing
5. Optimize script performance
6. Design scriptable gameplay systems

### Core Concepts

#### Language Selection

- **Lua**: Small, fast, embeddable, game industry standard
- **Python**: Large ecosystem, slower, good for tools
- **JavaScript**: Web integration, familiar to many
- **Wren**: Small, class-based, fiber support
- **Custom DSL**: Domain-specific, ultimate control

#### Binding Strategies

- **Manual Bindings**: Write wrapper functions
  - Full control, type-safe
  - Labor-intensive
- **Automatic Bindings**: Code generation, reflection
  - Fast development
  - Less control over API
- **FFI (Foreign Function Interface)**: C API exposure
  - Language-agnostic
  - Unsafe, requires care

#### Script-Engine Bridge

```text
Script:               Engine:
  player.health = 100  → Set component value
  enemy.take_damage(50)→ Call Rust function
  spawn("Fireball")    → Create entity
```

#### Hot-Reload

- **File Watching**: Monitor script changes
- **Reload Trigger**: Recompile and re-execute
- **State Preservation**: Keep game state across reloads
- **Error Handling**: Graceful degradation on script errors

#### Sandboxing

- **Restrict APIs**: Disable file I/O, networking, etc.
- **Execution Limits**: CPU time, memory usage
- **Isolation**: Prevent scripts from interfering

### Praxis Implementation Reference

```text
Scripting (Lua + mlua):
  - ScriptingContext: Lua VM management
  - ECS bindings: Query/modify components from scripts
  - Hot-reload: File watcher, auto-reload
  - Sandboxing: Configurable security levels
  - Performance monitoring: Track script execution time
```

**Relevant Code**: `praxis_scripting`, `mlua` integration

### Assessment Criteria

- Can integrate scripting language
- Can expose engine functionality safely
- Can implement hot-reload
- Can profile script performance
- Can design scriptable game mechanics

### Advanced Topics

- Multi-language support
- Visual scripting (node graphs)
- Script debugging integration
- Script compilation and obfuscation
- Modding API design

---

## Module 12: Networking Foundations

**Duration**: 4-5 weeks  
**Complexity**: Advanced

### Learning Objectives

By the end of this module, students will be able to:

1. Design client-server vs. peer-to-peer architectures
2. Implement entity replication and synchronization
3. Handle client prediction and server reconciliation
4. Design lag compensation systems
5. Optimize network bandwidth usage
6. Debug networking issues (latency, packet loss)

### Core Concepts

#### Network Architectures

- **Client-Server**
  - Server: Authoritative game state
  - Clients: Input + prediction
  - Pros: Anti-cheat, consistent state
  - Cons: Server cost, latency to server
- **Peer-to-Peer**
  - All peers equal (or host-based)
  - Pros: No server cost
  - Cons: Cheating, NAT traversal, inconsistency
- **Hybrid**: Dedicated servers + P2P voice/video

#### Entity Replication

```text
Server:
  Entity 123: Position(10, 0, 5), Velocity(2, 0, 0), Health(80)

Network:
  [Replication Packet]
    Entity 123: Pos(10,0,5), Vel(2,0,0), Health(80)

Client:
  Create/update Entity 123 with server data
```

- **Relevancy**: Only replicate nearby entities
- **Priority**: Replicate important changes first
- **Delta Compression**: Only send changed values

#### Client-Side Prediction

```text
Server tick 100: Player at (10, 0, 5)
Network latency: 50ms (3 ticks)

Client:
  Tick 100: Receive position (10, 0, 5)
  Tick 101: Predict → (11, 0, 5)
  Tick 102: Predict → (12, 0, 5)
  Tick 103: Predict → (13, 0, 5)
  Tick 103: Receive server confirmation (13.1, 0, 5.1)
  Tick 103: Reconcile → Slightly off, correct!
```

#### Lag Compensation

- **Server Rewind**: "Rollback" world state to when client shot
- **Hit Detection**: Check collision in rewound state
- **Fair Gameplay**: High-ping players don't suffer

#### Bandwidth Optimization

- **Quantization**: Reduce precision (16-bit instead of 32-bit)
- **Compression**: Run-length, delta, Huffman encoding
- **Prioritization**: Critical updates first
- **Culling**: Don't send irrelevant data

### Praxis Implementation Reference

```text
Networking (Client-Server):
  - NetworkServer: TCP/UDP transport, connection management
  - ReplicationRegistry: Register replicated components
  - Interpolation: Smooth remote entity movement
  - Lag Compensation: Server-side rewind
  - Network Profiler: Bandwidth and latency monitoring
```

**Relevant Code**: `praxis_networking`

### Assessment Criteria

- Can implement basic client-server networking
- Can replicate entities reliably
- Can implement client prediction
- Can measure and optimize bandwidth
- Can debug common networking issues

### Advanced Topics

- Lockstep deterministic networking
- Rollback netcode (GGPO-style)
- Distributed server architectures
- Voice chat integration
- Network security (encryption, authentication)

---

## Pedagogical Approach

### Teaching Methodology

Each module follows this structure:

1. **Conceptual Foundation**: Theory and design rationale (30%)
2. **Practical Implementation**: Hands-on coding exercises (40%)
3. **Analysis and Optimization**: Profiling, debugging, iteration (20%)
4. **Project Integration**: Combining concepts into larger systems (10%)

### Assessment Methods

- **Formative**: Weekly exercises, code reviews, design discussions
- **Summative**: Module projects demonstrating mastery
- **Capstone**: Integrate all modules into a complete mini-game engine

### Language-Agnostic Focus

While Praxis is written in Rust, this curriculum emphasizes:

- **Universal Patterns**: Applicable to C++, C#, Rust, etc.
- **Conceptual Understanding**: Why, not just how
- **Pseudocode**: High-level algorithms before implementation
- **Comparative Analysis**: Trade-offs across languages/platforms

### Reference Implementation Mapping

Each module includes "Praxis Implementation Reference" sections that:

- Show how Praxis implements the concept
- Reference specific crates/files
- Provide concrete examples
- Highlight language-specific patterns (Rust's ownership, traits)

### Progression Philosophy

- **Beginner** (Modules 1, 8): Fundamental patterns everyone must know
- **Intermediate** (Modules 2-7, 9, 11): Subsystem design and integration
- **Advanced** (Modules 10, 12): Complex, multi-faceted systems

### Cross-Cutting Concerns

Several themes appear across modules:

- **Performance**: Profiling, optimization, cache awareness
- **Debugging**: Tools, techniques, common pitfalls
- **Architecture**: Separation of concerns, abstraction layers
- **Production**: Error handling, logging, telemetry

---

## Learning Path Recommendations

### For Graphics Programmers

**Primary**: Modules 2, 4, 7  
**Secondary**: Modules 1, 3, 5  
**Optional**: Modules 6, 8, 9, 10, 11, 12

### For Gameplay Programmers

**Primary**: Modules 3, 5, 8, 11  
**Secondary**: Modules 1, 4, 6  
**Optional**: Modules 2, 7, 9, 10, 12

### For Engine Architects

**All modules required**, recommended order:
1 → 3 → 4 → 2 → 7 → 5 → 6 → 8 → 9 → 11 → 10 → 12

### For Technical Artists

**Primary**: Modules 6, 10  
**Secondary**: Modules 2, 4, 8, 11  
**Optional**: Modules 1, 3, 5, 7, 9, 12

---

## Appendices

### A. Glossary of Universal Terminology

- **Entity**: Unique identifier for a game object
- **Component**: Data attached to an entity
- **System**: Logic that operates on components
- **Archetype**: Set of entities with identical component types
- **Transform**: Position, rotation, and scale in 3D space
- **Rigidbody**: Physics simulation properties
- **Pipeline**: Sequence of GPU operations
- **Descriptor**: Binding of resources to shaders
- **Uniform**: Read-only data passed to shaders
- **Vertex Buffer**: GPU memory containing vertex data
- **Index Buffer**: GPU memory containing triangle indices
- **Render Pass**: Rendering operations to framebuffer
- **Swapchain**: Queue of images for display
- **Synchronization**: Coordinating CPU and GPU work
- **Timestep**: Time increment per simulation update
- **Delta Time**: Elapsed time since last frame

### B. Essential Mathematics

- **Linear Algebra**: Vectors, matrices, transformations
- **Quaternions**: 4D rotation representation
- **Coordinate Systems**: Right-handed vs. left-handed, row-major vs. column-major
- **Homogeneous Coordinates**: 4x4 matrices for affine transformations
- **Interpolation**: Linear (lerp), spherical (slerp), cubic (splines)

### C. Performance Analysis Tools

- **CPU Profilers**: Sampling profilers, instrumentation profilers
- **GPU Profilers**: RenderDoc, Nsight, PIX, Metal Frame Capture
- **Memory Profilers**: Valgrind, heaptrack, AddressSanitizer
- **Network Analyzers**: Wireshark, Charles Proxy

### D. Further Reading

- **Game Engine Architecture** by Jason Gregory
- **Real-Time Rendering** by Tomas Akenine-Möller et al.
- **Game Programming Patterns** by Robert Nystrom
- **Foundations of Game Engine Development** series by Eric Lengyel
- **Physically Based Rendering: From Theory to Implementation** by Pharr, Jakob, Humphreys
- **Multiplayer Game Programming** by Joshua Glazer & Sanjay Madhav

### E. Industry Resources

- **GDC Talks**: Game Developers Conference presentations
- **Digital Dragons**: European game development conference
- **Graphics Programming Weekly**: Curated graphics resources
- **Real-Time Rendering Resources**: http://www.realtimerendering.com/
- **Learn OpenGL / Vulkan Tutorial**: Graphics API tutorials
- **Bevy Engine**: Modern Rust ECS reference

---

## Curriculum Maintenance

This curriculum is designed to evolve with industry practices and Praxis development:

- **Version**: 1.0 (Initial release)
- **Last Updated**: 2024
- **Maintainers**: Praxis core team
- **Feedback**: Issues and pull requests welcome

### Contribution Guidelines

When updating modules:

1. Maintain language-agnostic focus
2. Provide concrete examples from Praxis
3. Update assessment criteria as needed
4. Add cross-references to new concepts
5. Ensure beginner → intermediate → advanced progression

---

## Conclusion

This curriculum provides a comprehensive, language-agnostic framework for understanding game engine architecture. By focusing on universal concepts while using Praxis as a reference implementation, students gain transferable knowledge applicable to any engine development project.

Whether building a custom engine, extending an existing one, or simply deepening understanding of game technology, this curriculum offers a structured path from fundamentals to advanced topics, emphasizing both theory and practical application.
