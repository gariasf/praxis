# Learning Paths Glossary

Key terms and concepts used throughout the Praxis learning paths.

## General Terms

**Beginner / Intermediate / Advanced**
- Skill level classifications for learning path sections
- Beginner: Core concepts and basic usage (no prior knowledge needed)
- Intermediate: Integration and optimization (requires beginner completion)
- Advanced: Custom extensions and production features (requires intermediate)

**Prerequisites**
- Required knowledge before starting a section
- Listed at the beginning of each path and level

**Learning Outcomes**
- Skills and knowledge gained after completing a section
- Presented as checkboxes for progress tracking

**Checkpoint**
- End-of-section assessment to verify understanding
- Includes self-assessment questions and capstone projects

**Cross-References**
- Related topics in other learning paths
- Links to relevant documentation

**Time Estimate**
- Expected hours to complete a section
- Assumes focused, hands-on learning with examples

## Engine Architecture Terms

**ECS (Entity-Component-System)**
- Architectural pattern for game engine data organization
- Entity: Unique ID representing a game object
- Component: Data attached to entities (Transform, MeshHandle, etc.)
- System: Logic that operates on entities with specific components

**World**
- Container for all entities and components
- Central hub for ECS operations

**Query**
- ECS operation to find entities with specific components
- Example: `Query<(&Transform, &MeshHandle)>`

**Resource**
- Global data accessible to all systems
- Example: Time, PhysicsWorld, AssetManager

**Schedule**
- Ordered execution of systems each frame
- Defines when systems run and in what order

## Rendering Terms

**Forward Rendering**
- Render each object with all lighting calculations
- Cost: O(objects × triangles × lights)

**Deferred Rendering**
- Two-pass: geometry to G-buffer, then lighting
- Cost: O(objects × triangles) + O(pixels × lights)

**G-Buffer (Geometry Buffer)**
- Textures storing surface properties (albedo, normals, material)
- Used in deferred rendering

**PBR (Physically-Based Rendering)**
- Rendering approach based on real-world physics
- Properties: albedo, metallic, roughness, emissive

**HDR (High Dynamic Range)**
- Color values beyond [0,1] range
- Enables realistic bright/dark representation

**Tone Mapping**
- Convert HDR values to displayable [0,1] range
- Algorithms: ACES, Reinhard

**IBL (Image-Based Lighting)**
- Lighting from environment maps
- Realistic reflections and ambient lighting

**Frustum Culling**
- Skip rendering objects outside camera view
- Major performance optimization

**LOD (Level of Detail)**
- Use simpler models/assets at greater distances
- Reduces rendering cost

**Descriptor Set**
- Vulkan concept: bindings for shaders (uniforms, textures)
- Groups GPU resources for shader access

**Render Pass**
- Vulkan concept: stage of rendering (geometry, lighting, post-process)

## Animation Terms

**Skeletal Animation**
- Animation technique using hierarchical bones
- Bones deform mesh vertices

**Skeleton**
- Hierarchical structure of bones
- Defines character's underlying structure

**Joint / Bone**
- Node in skeleton hierarchy
- Has local transform relative to parent

**Animation Clip**
- Sequence of keyframes defining motion over time

**Keyframe**
- Transform snapshot at specific time
- Interpolated between to create smooth motion

**Interpolation**
- Calculating values between keyframes
- Types: Linear (LERP), Spherical (SLERP), Cubic

**Blending**
- Combining multiple animations smoothly
- Types: Cross-fade, layered, additive

**Blend Tree**
- Structure for mixing animations based on parameters
- Example: Walk → Run → Sprint based on speed

**State Machine**
- Graph of animation states with transition conditions
- Example: Idle → Walk → Jump → Fall → Land

**IK (Inverse Kinematics)**
- Calculate joint angles to reach target position
- Example: Foot placement on terrain

**Root Motion**
- Movement encoded in animation itself
- Extracted and applied to entity transform

**Retargeting**
- Applying animation from one skeleton to another
- Useful for reusing animations across characters

## Physics Terms

**Rigid Body**
- Physics object with mass that can move/rotate
- Types: Dynamic, Static, Kinematic

**Dynamic Body**
- Affected by forces, gravity, and collisions
- Use for: projectiles, ragdolls, vehicles

**Static Body**
- Never moves, has infinite mass
- Use for: walls, floors, terrain

**Kinematic Body**
- Moved by code, affects others but isn't affected
- Use for: moving platforms, doors

**Collider**
- Shape defining collision boundaries
- Types: Sphere, Box, Capsule, Mesh, Compound

**Collision Detection**
- Determining when objects intersect
- Phases: Broad phase (AABB), narrow phase (detailed)

**Raycast**
- Shooting a ray to detect intersections
- Returns: hit point, normal, distance, entity

**Physics Material**
- Surface properties: friction, restitution (bounciness)

**Joint / Constraint**
- Connection between rigid bodies
- Types: Fixed, Revolute (hinge), Prismatic (slider), Spherical

**Ragdoll**
- Character made of connected rigid bodies
- Used for death/impact animations

**Character Controller**
- Kinematic body for player movement
- Handles: ground detection, slopes, stairs

## Scripting Terms

**Lua**
- Lightweight scripting language
- Used for runtime game logic in Praxis

**Scripting Context**
- Lua environment with engine bindings
- Interface between Rust and Lua

**Hot-Reload**
- Automatically reload scripts when files change
- Enables rapid iteration without restart

**Sandboxing**
- Restricting script capabilities for security
- Levels: None, Moderate, Strict

**UserData**
- Rust types exposed to Lua
- Example: Vec3, Transform, custom types

## Networking Terms

**Client-Server Architecture**
- Server is authoritative, clients are views
- Server validates all actions

**Entity Replication**
- Automatically synchronize entities across network
- Registered components sync automatically

**Interpolation**
- Smooth remote entity movement between updates
- Delays slightly for smoothness

**Extrapolation**
- Predict movement during packet loss
- Dead reckoning based on velocity

**Lag Compensation**
- Server rewinds time for fair hit detection
- Accounts for network latency

**Client Prediction**
- Client simulates actions immediately
- Server validates and corrects if needed

**Network Profiler**
- Tool for monitoring bandwidth and latency
- Identifies network bottlenecks

## Performance Terms

**Profiling**
- Measuring code execution time
- Identifies performance bottlenecks

**Frame Time**
- Time to render one frame (milliseconds)
- Target: < 16.67ms for 60 FPS

**Draw Call**
- GPU command to render geometry
- Reducing draw calls improves performance

**Batching**
- Combining multiple objects into single draw call
- Major performance optimization

**Memory Allocation**
- Requesting memory from OS/allocator
- Frequent allocations cause slowdowns

**Object Pooling**
- Reusing objects instead of allocating new
- Reduces allocation overhead

**Cache-Friendly**
- Data layout optimized for CPU cache
- Improves iteration performance

**Multi-Threading**
- Using multiple CPU cores simultaneously
- Parallel processing of independent work

**GPU Profiling**
- Measuring GPU execution time
- Tools: RenderDoc, Nsight, Radeon GPU Profiler

## Editor Terms

**Gizmo**
- Visual tool for manipulating transforms
- Types: Translate, Rotate, Scale

**Hierarchy Panel**
- Tree view of entity parent-child relationships
- Allows drag-and-drop reparenting

**Inspector**
- Panel showing entity's components
- Edit component values in real-time

**Asset Browser**
- Panel for managing game assets
- Supports drag-and-drop into scene

**Undo/Redo**
- Revert/reapply editor actions
- Implemented via Command pattern

**Command Pattern**
- Design pattern for undoable operations
- Each action is a reversible command

**Selection**
- Currently selected entities in editor
- Can manipulate multiple at once

## Asset Terms

**Asset Pipeline**
- Process of loading and preparing game assets
- Includes: loading, parsing, optimizing, caching

**GLTF (GL Transmission Format)**
- Standard 3D model format
- Supports: meshes, materials, animations, scenes

**OBJ**
- Simple 3D model format
- Text-based, easy to parse

**Asset Handle**
- Reference to asset stored in manager
- Lightweight (just an ID or name)

**Asset Manager**
- Centralized storage for loaded assets
- Manages: loading, caching, lifetime

**Hot-Reload (Assets)**
- Automatically reload assets when files change
- Useful for iterating on textures, models

## Vulkan Terms

**Vulkan**
- Low-level graphics API
- Explicit control over GPU

**Vulkano**
- Safe Rust wrapper around Vulkan
- Used by Praxis

**Pipeline**
- Complete GPU state configuration
- Includes: shaders, render state, descriptor layouts

**Shader**
- Program running on GPU
- Types: Vertex shader, Fragment shader, Compute shader

**Uniform Buffer**
- GPU memory for read-only shader data
- Contains: matrices, lighting data, material properties

**Swapchain**
- Series of images for presentation
- Double/triple buffering

**Command Buffer**
- Recorded GPU commands
- Submit to queue for execution

**Synchronization**
- Coordinating CPU and GPU work
- Uses: Fences, Semaphores

## Common Abbreviations

- **ECS**: Entity-Component-System
- **PBR**: Physically-Based Rendering
- **HDR**: High Dynamic Range
- **IBL**: Image-Based Lighting
- **LOD**: Level of Detail
- **IK**: Inverse Kinematics
- **FK**: Forward Kinematics
- **FPS**: Frames Per Second (or First-Person Shooter)
- **MSAA**: Multi-Sample Anti-Aliasing
- **PCF**: Percentage Closer Filtering (shadows)
- **CSM**: Cascaded Shadow Maps
- **GLTF**: GL Transmission Format
- **API**: Application Programming Interface
- **GPU**: Graphics Processing Unit
- **CPU**: Central Processing Unit
- **RAM**: Random Access Memory
- **VRAM**: Video RAM (GPU memory)

## Skill Level Definitions

### Beginner
- New to concept/system
- No prior knowledge required
- Focus: Core concepts, basic usage
- Outcome: Can use system for simple tasks

### Intermediate
- Familiar with basics
- Prerequisites: Beginner complete
- Focus: Integration, optimization, patterns
- Outcome: Can build production features

### Advanced
- Proficient with system
- Prerequisites: Intermediate complete
- Focus: Architecture, extensions, performance
- Outcome: Can customize and optimize

## Time Estimates

**Hours per Section**:
- Beginner: 15-25 hours
- Intermediate: 20-35 hours
- Advanced: 20-40 hours

**Hours per Path** (all levels):
- Major systems (Rendering, Animation, Physics): 50-100 hours
- Medium systems (Scripting, Networking): 45-75 hours
- Minor systems (Audio, Editor, Assets): 20-50 hours

**Total Mastery**: 400-600 hours (6-12 months part-time)

## Learning Modalities

**Theory**
- Reading documentation
- Understanding concepts
- Learning "why" not just "how"

**Practice**
- Writing code
- Running examples
- Hands-on experimentation

**Exercises**
- Structured challenges
- Apply learned concepts
- Build mini-projects

**Examples**
- Working code to study
- Run with `cargo run --example <name>`
- Reference implementations

**Projects**
- Larger applications of knowledge
- Integration of multiple systems
- Portfolio pieces

## Documentation Types

**Concepts**
- Educational explanations
- Theory and design
- "Why" questions

**Guides**
- Task-oriented tutorials
- Step-by-step instructions
- "How" questions

**Reference**
- API documentation
- Quick lookups
- "What" questions

**Learning Paths**
- Structured progressions
- Clear prerequisites
- Time-based progression

## Common Patterns

**Query Pattern**
```rust
Query<(&Transform, &MeshHandle)>
// Find entities with both Transform and MeshHandle
```

**Changed Detection**
```rust
Query<&Transform, Changed<Transform>>
// Only entities where Transform changed
```

**Optional Components**
```rust
Query<(&Transform, Option<&Parent>)>
// Transform required, Parent optional
```

**Resource Access**
```rust
fn system(time: Res<Time>, mut physics: ResMut<PhysicsWorld>)
// Read time, write physics
```

## Troubleshooting Terms

**Bottleneck**
- System/code limiting overall performance
- Found via profiling

**Artifact**
- Visual glitch or error in rendering
- Examples: Z-fighting, shadow acne

**Memory Leak**
- Memory not properly released
- Causes increasing memory usage

**Race Condition**
- Timing-dependent bug in concurrent code
- Result depends on execution order

**Blocking**
- Operation that prevents other work
- Should avoid in main loop

---

## Using This Glossary

- **First time seeing a term?** Look it up here for quick definition
- **Confused by abbreviation?** Check abbreviations section
- **Unfamiliar pattern?** See common patterns
- **Not sure of skill level?** Review skill level definitions

## Navigation

- [Back to Learning Paths](README.md)
- [Learning Paths Roadmap](roadmap.md) - Visual progression guide
- [Beginner's Guide](../beginners-guide.md) - Comprehensive introduction
