# Engine Terminology Glossary

A comprehensive cross-engine reference defining core game engine concepts with equivalent terms across Unity, Unreal Engine, Godot, and Praxis.

## Table of Contents

- [Core Architecture](#core-architecture)
- [Scene & Transform System](#scene--transform-system)
- [Rendering](#rendering)
- [Graphics Pipeline](#graphics-pipeline)
- [Materials & Shaders](#materials--shaders)
- [Lighting](#lighting)
- [Animation](#animation)
- [Physics](#physics)
- [Audio](#audio)
- [Input](#input)
- [Scripting](#scripting)
- [Editor](#editor)
- [Asset Pipeline](#asset-pipeline)

---

## Core Architecture

### Entity

**Definition**: A unique identifier representing a game object or scene element.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Entity` | Lightweight ID in ECS (bevy_ecs), just an integer with generation |
| **Unity** | `GameObject` | Heavier object with built-in Transform, can have components |
| **Unreal** | `AActor` | C++ class, heavier than Entity, has Transform and networking |
| **Godot** | `Node` | Tree-based, all objects inherit from Node |

**Key Differences**:
- **Praxis/ECS**: Entity is just an ID, all data in components
- **Unity**: GameObject is an object with Transform always present
- **Unreal**: Actor is a full C++ object with extensive built-in functionality
- **Godot**: Node is part of a tree hierarchy, has built-in properties

---

### Component

**Definition**: A data container attached to an entity that defines a specific aspect or behavior.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Component` | Pure data struct with `#[derive(Component)]` |
| **Unity** | `Component` / `MonoBehaviour` | C# class, can contain logic (not pure data) |
| **Unreal** | `UActorComponent` | C++ class attached to Actor |
| **Godot** | Node property / Script | Properties on Node or GDScript attached |

**Key Differences**:
- **Praxis**: Components are pure data, systems contain logic (strict ECS)
- **Unity**: Components can contain logic (hybrid approach)
- **Unreal**: Components are full objects with logic and lifecycle
- **Godot**: Logic typically in scripts attached to nodes

---

### System

**Definition**: Logic that operates on entities with specific components.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `System` (function) | Functions that query components and process them |
| **Unity** | System (ECS) / Update methods | ISystem in DOTS, or MonoBehaviour.Update() |
| **Unreal** | `Tick()` / Subsystem | Actor/Component Tick functions, or USubsystem |
| **Godot** | `_process()` / `_physics_process()` | Node lifecycle methods |

**Key Differences**:
- **Praxis**: Explicit system functions, data-oriented
- **Unity**: Either DOTS systems or per-component Update calls
- **Unreal**: Per-object Tick functions or global subsystems
- **Godot**: Per-node process callbacks

---

### World / Scene

**Definition**: Container for all entities, components, and game state.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `World` (bevy_ecs) | ECS world containing all entities/components |
| **Unity** | `Scene` | Container for GameObjects, can load/unload |
| **Unreal** | `UWorld` / `ULevel` | World contains Levels, hierarchical |
| **Godot** | `Scene` / `SceneTree` | Tree of nodes, scenes can be instanced |

**Key Differences**:
- **Praxis**: Flat ECS storage with optional hierarchy via Parent/Children
- **Unity**: Scene graph with hierarchical GameObjects
- **Unreal**: World/Level hierarchy, streaming architecture
- **Godot**: Strict tree hierarchy, scenes are nodes

---

## Scene & Transform System

### Transform

**Definition**: Position, rotation, and scale of an entity in 3D space.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Transform` (local) | Component with translation (Vec3), rotation (Quat), scale (Vec3) |
| **Unity** | `Transform` | Built-in component on all GameObjects |
| **Unreal** | `FTransform` / `USceneComponent` | Struct for transform data, SceneComponent for hierarchy |
| **Godot** | `Transform3D` / `Node3D.transform` | Property on 3D nodes |

**Key Differences**:
- **Praxis**: Separate Transform (local) and GlobalTransform (world) components
- **Unity**: Single Transform with localPosition/position, localRotation/rotation
- **Unreal**: FTransform struct, relative/world transform on SceneComponent
- **Godot**: Transform3D, global_transform automatically computed

---

### Parent / Child

**Definition**: Hierarchical relationship where child transforms are relative to parent.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Parent`, `Children` components | Explicit components for hierarchy |
| **Unity** | `Transform.parent`, `Transform.GetChild()` | Built into Transform |
| **Unreal** | `AttachToComponent()`, `GetAttachChildren()` | SceneComponent attachment |
| **Godot** | Node tree structure | Nodes have `get_parent()`, `get_children()` |

**Key Differences**:
- **Praxis**: Optional hierarchy via components, propagation via systems
- **Unity/Unreal/Godot**: Hierarchy is fundamental to architecture

---

### Global Transform / World Transform

**Definition**: Final computed transform in world space (after hierarchy propagation).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `GlobalTransform` | Computed by transform propagation systems |
| **Unity** | `Transform.position/rotation/scale` | Automatic, accessed via non-local properties |
| **Unreal** | `GetComponentTransform()` | Computed on access or cached |
| **Godot** | `global_transform` | Automatically maintained |

---

## Rendering

### Mesh

**Definition**: 3D geometry data (vertices, indices, normals, UVs).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `MeshHandle` component | Reference to mesh in MeshManager |
| **Unity** | `MeshFilter` / `Mesh` | MeshFilter component references Mesh asset |
| **Unreal** | `UStaticMesh` / `UStaticMeshComponent` | Mesh asset and component |
| **Godot** | `MeshInstance3D` / `Mesh` | Node with mesh resource |

---

### Material

**Definition**: Defines surface appearance (colors, textures, shader parameters).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `MaterialProperties` | Component or struct with PBR parameters |
| **Unity** | `Material` | Asset with shader and properties |
| **Unreal** | `UMaterial` / `UMaterialInstance` | Material graph and instances |
| **Godot** | `Material` (ShaderMaterial, StandardMaterial3D) | Material resource |

---

### Camera

**Definition**: Viewpoint for rendering the scene.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Camera` component | Component with projection settings |
| **Unity** | `Camera` | Component with projection, culling, rendering settings |
| **Unreal** | `UCameraComponent` | Component on CameraActor or Pawn |
| **Godot** | `Camera3D` | Node with projection and environment |

---

### Renderer

**Definition**: System that draws meshes to screen.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `RenderContext`, `DeferredRenderer` | Vulkan-based rendering system |
| **Unity** | Universal Render Pipeline (URP) / HDRP | Scriptable render pipelines |
| **Unreal** | Rendering subsystem | Built-in forward/deferred renderer |
| **Godot** | `RenderingServer` | Rendering backend (Vulkan, OpenGL) |

---

## Graphics Pipeline

### Render Pass

**Definition**: A stage in rendering that writes to framebuffer attachments.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `RenderPass` (Vulkan) | Vulkan render pass with attachments |
| **Unity** | Render Pass / SubPass | URP render passes |
| **Unreal** | RenderPass | Rendering stage in frame |
| **Godot** | RenderingServer pass | Internal rendering stage |

**Key Differences**:
- **Praxis**: Direct Vulkan render pass (explicit attachments, load/store ops)
- **Unity**: High-level URP passes with automatic management
- **Unreal/Godot**: Abstracted, engine manages details

---

### Framebuffer

**Definition**: Collection of image attachments that rendering outputs to.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Framebuffer` (Vulkan) | Explicit framebuffer with image views |
| **Unity** | `RenderTexture` | High-level render target |
| **Unreal** | `FRenderTarget` | Render target texture |
| **Godot** | `Viewport` / RenderTarget | Viewport renders to texture |

**Key Differences**:
- **Praxis**: Low-level Vulkan framebuffer (must match render pass)
- **Unity/Unreal/Godot**: Higher-level abstractions

---

### Swapchain

**Definition**: Queue of images for presenting to the screen.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Swapchain` (Vulkan) | Explicit swapchain management |
| **Unity** | N/A (abstracted) | Handled internally by graphics API |
| **Unreal** | N/A (abstracted) | Handled by RHI (Rendering Hardware Interface) |
| **Godot** | N/A (abstracted) | Handled by rendering backend |

**Key Differences**:
- **Praxis**: Direct Vulkan swapchain control (acquire, present, recreation)
- Other engines abstract this away

---

### Pipeline / Pipeline State

**Definition**: Complete GPU rendering configuration (shaders, rasterization, blending, etc.).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `GraphicsPipeline` (Vulkan) | Immutable pipeline state object |
| **Unity** | Shader pass | Shader defines rendering state |
| **Unreal** | Pipeline State Object (PSO) | Cached rendering state |
| **Godot** | RenderingDevice pipeline | Internal pipeline state |

---

### Descriptor Set

**Definition**: Binding of resources (buffers, textures) to shaders.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `DescriptorSet` (Vulkan) | Explicit resource binding |
| **Unity** | Material properties / Shader uniforms | High-level property setting |
| **Unreal** | Shader parameters | Uniform/texture binding |
| **Godot** | Shader parameters | Resource binding to shader |

**Key Differences**:
- **Praxis**: Explicit Vulkan descriptor sets with layouts
- Other engines abstract binding through material/shader APIs

---

### Command Buffer

**Definition**: Recorded GPU commands for execution.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `CommandBuffer` (Vulkan) | Explicit command recording |
| **Unity** | `CommandBuffer` | Scriptable command recording |
| **Unreal** | `FRHICommandList` | RHI command list |
| **Godot** | N/A (abstracted) | Handled by RenderingServer |

---

## Materials & Shaders

### Shader

**Definition**: GPU program (vertex, fragment, compute, etc.).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | GLSL → SPIR-V | Shaders compiled via vulkano-shaders macro |
| **Unity** | ShaderLab / HLSL / Shader Graph | Multiple authoring methods |
| **Unreal** | Material Graph / HLSL | Visual or code shaders |
| **Godot** | Godot Shading Language (GSL) / VisualShader | Custom shading language |

---

### Vertex Shader

**Definition**: Processes vertices (position, normal, UV).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `vertex.glsl` | GLSL vertex shader |
| **Unity** | Vertex function in shader | `vert()` function |
| **Unreal** | Vertex Shader | Material graph or HLSL |
| **Godot** | `vertex()` function | In shader code |

---

### Fragment Shader / Pixel Shader

**Definition**: Computes color for each pixel.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `fragment.glsl` | GLSL fragment shader |
| **Unity** | Fragment function | `frag()` function |
| **Unreal** | Pixel Shader | Material graph or HLSL |
| **Godot** | `fragment()` function | In shader code |

---

### Uniform / Constant Buffer

**Definition**: Read-only data passed to shaders.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `uniform` in GLSL | Uploaded via descriptor sets |
| **Unity** | Shader properties | Set via Material or CommandBuffer |
| **Unreal** | Uniform Buffer | Shader parameters |
| **Godot** | `uniform` in shader | Shader parameters |

---

### Texture / Sampler

**Definition**: Image data sampled in shaders.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Image` + `Sampler` | Vulkan image and sampler, bound via descriptor |
| **Unity** | `Texture2D` + sampler state | Texture asset with import settings |
| **Unreal** | `UTexture2D` + sampler | Texture asset |
| **Godot** | `Texture2D` + sampler | Texture resource |

---

## Lighting

### Light

**Definition**: Light source that illuminates the scene.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `DirectionalLight`, `PointLight` | Components with color, intensity |
| **Unity** | `Light` | Component with type (Directional, Point, Spot) |
| **Unreal** | `ULightComponent` | DirectionalLight, PointLight, SpotLight |
| **Godot** | `Light3D` subclasses | DirectionalLight3D, OmniLight3D, SpotLight3D |

---

### Directional Light

**Definition**: Infinite-distance light (sun).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `DirectionalLight` | Direction, color, intensity |
| **Unity** | `Light` (Type: Directional) | Direction from Transform rotation |
| **Unreal** | `DirectionalLight` | Sun/moon lighting |
| **Godot** | `DirectionalLight3D` | Sun lighting |

---

### Point Light

**Definition**: Omnidirectional light with attenuation.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `PointLight` | Position from Transform, range, intensity |
| **Unity** | `Light` (Type: Point) | Range and intensity |
| **Unreal** | `PointLight` | Attenuation radius |
| **Godot** | `OmniLight3D` | Range and attenuation |

---

### Shadow

**Definition**: Darkening caused by light occlusion.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Shadow mapping (manual) | Cascaded shadow maps for directional lights |
| **Unity** | Shadows (realtime/baked) | Automatic shadow rendering |
| **Unreal** | Dynamic/Static shadows | Cascaded, ray-traced, or baked |
| **Godot** | Shadow settings on lights | Automatic shadow rendering |

---

## Animation

### Skeleton / Armature

**Definition**: Hierarchical bone structure for skeletal animation.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Skeleton` | Component with bone hierarchy |
| **Unity** | `Humanoid` / `Generic` rig | Imported with model |
| **Unreal** | `USkeleton` | Shared skeleton asset |
| **Godot** | `Skeleton3D` | Node with bone hierarchy |

---

### Animation Clip

**Definition**: Keyframe data for animating properties over time.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `AnimationClip` | Keyframes for bone transforms |
| **Unity** | `AnimationClip` | Keyframes for any property |
| **Unreal** | `UAnimSequence` | Animation sequence |
| **Godot** | `Animation` | Animation resource |

---

### Animation Player / Controller

**Definition**: System that plays and blends animations.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `AnimationPlayer` | Component managing clip playback |
| **Unity** | `Animator` | Component with AnimatorController |
| **Unreal** | `UAnimInstance` | Animation Blueprint instance |
| **Godot** | `AnimationPlayer` / `AnimationTree` | Nodes for playback and blending |

---

### Blend Tree

**Definition**: Graph for blending multiple animations.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Blend tree (manual) | Custom blending in AnimationPlayer |
| **Unity** | Blend Tree | Part of AnimatorController |
| **Unreal** | Blend Space | 1D/2D blending |
| **Godot** | `AnimationTree` | Node-based blending |

---

## Physics

### Rigid Body

**Definition**: Object with physics simulation (mass, velocity, forces).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `RigidBody` | Component (Rapier3D backend) |
| **Unity** | `Rigidbody` | Component (PhysX backend) |
| **Unreal** | `UPrimitiveComponent` (Simulate Physics) | PhysX/Chaos backend |
| **Godot** | `RigidBody3D` | Node with physics |

---

### Collider

**Definition**: Shape for collision detection.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `Collider` | Component (box, sphere, capsule, mesh) |
| **Unity** | `BoxCollider`, `SphereCollider`, etc. | Separate components per shape |
| **Unreal** | `UShapeComponent` (Box, Sphere, Capsule) | Collision components |
| **Godot** | `CollisionShape3D` with `Shape3D` resource | Node with shape resource |

---

### Physics World

**Definition**: Simulation space for physics calculations.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `PhysicsWorld` (Rapier3D) | Resource managing Rapier pipeline |
| **Unity** | Physics scene | Implicit, managed by engine |
| **Unreal** | `UWorld` physics scene | Integrated with world |
| **Godot** | Physics server | Backend simulation |

---

### Collision Layer / Mask

**Definition**: Bit flags determining which objects collide.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Collision groups (Rapier) | Membership and filter bits |
| **Unity** | Layer Collision Matrix | Set in Physics settings |
| **Unreal** | Collision channels | Trace/Object channels |
| **Godot** | Collision layers and masks | Bit flags |

---

## Audio

### Audio Source

**Definition**: Emitter of sound in 3D space.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Audio playback (Kira) | Spatial or 2D playback |
| **Unity** | `AudioSource` | Component with spatial settings |
| **Unreal** | `UAudioComponent` | Spatialized audio component |
| **Godot** | `AudioStreamPlayer3D` | 3D audio node |

---

### Audio Listener

**Definition**: "Ears" for hearing audio (typically camera).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Listener position (manual) | Set via Kira API |
| **Unity** | `AudioListener` | Component (one per scene) |
| **Unreal** | Camera location | Automatic listener |
| **Godot** | `AudioListener3D` | Node for audio perspective |

---

## Input

### Input Action

**Definition**: Abstract input (e.g., "Jump") mapped to keys/buttons.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Manual mapping | Check `InputState` for keys |
| **Unity** | Input Action (New Input System) | Configured in Input Actions asset |
| **Unreal** | Input Action | Configured in Project Settings |
| **Godot** | Input Map | Configured in Project Settings |

---

### Key / Button

**Definition**: Specific keyboard key or gamepad button.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `KeyCode` (winit) | Enum for keyboard keys |
| **Unity** | `KeyCode` | Enum for keys |
| **Unreal** | `EKeys` | Key enumeration |
| **Godot** | `KEY_*` constants | Key constants |

---

## Scripting

### Script

**Definition**: Code that defines behavior, typically in a high-level language.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Lua script | Scripts via `praxis_scripting` (mlua) |
| **Unity** | C# script | MonoBehaviour or ECS system |
| **Unreal** | Blueprint / C++ | Visual or code scripting |
| **Godot** | GDScript / C# | Built-in scripting languages |

---

### Hot Reload

**Definition**: Updating code/scripts without restarting.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Hot reload (Lua) | File watching, automatic reload |
| **Unity** | Domain Reload | C# compilation and reload |
| **Unreal** | Hot Reload / Live Coding | C++ hot reload |
| **Godot** | Script reload | GDScript auto-reload on save |

---

## Editor

### Inspector / Properties Panel

**Definition**: UI for editing component values.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Inspector (egui) | Panel for editing components |
| **Unity** | Inspector | Properties panel |
| **Unreal** | Details Panel | Properties for selected actor |
| **Godot** | Inspector | Properties dock |

---

### Hierarchy / Outliner

**Definition**: Tree view of entities/objects in scene.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Hierarchy panel (egui) | Entity tree with parent/child |
| **Unity** | Hierarchy | GameObject tree |
| **Unreal** | Outliner | Actor list and hierarchy |
| **Godot** | Scene dock | Node tree |

---

### Viewport / Scene View

**Definition**: 3D view of the scene for editing.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Main render window | Rendered scene with editor overlay |
| **Unity** | Scene view | 3D editing viewport |
| **Unreal** | Viewport | Level editing view |
| **Godot** | 3D viewport | Scene editing view |

---

### Gizmo

**Definition**: Visual widget for manipulating transforms.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Transform gizmo | Move/rotate/scale widgets |
| **Unity** | Transform tools | Move, Rotate, Scale, Rect tools |
| **Unreal** | Transform gizmo | Move, Rotate, Scale widgets |
| **Godot** | Transform gizmo | Manipulation widgets |

---

### Selection

**Definition**: Currently active entity/object being edited.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `SelectionSystem` | Tracks selected entities |
| **Unity** | `Selection.activeGameObject` | Current selection |
| **Unreal** | Selected actors | Editor selection |
| **Godot** | Scene tree selection | Selected nodes |

---

### Undo / Redo

**Definition**: Reversible operations in editor.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | `UndoRedoSystem` | Command pattern with history |
| **Unity** | `Undo.RecordObject()` | Automatic undo system |
| **Unreal** | Transaction system | Automatic undo/redo |
| **Godot** | `UndoRedo` | Manual undo/redo API |

---

## Asset Pipeline

### Asset

**Definition**: External resource (mesh, texture, audio, etc.).

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Asset (loaded via loaders) | Files loaded from disk |
| **Unity** | Asset | Files in Assets folder |
| **Unreal** | `UObject` asset | .uasset files |
| **Godot** | Resource | .tres, .res files |

---

### Mesh Asset

**Definition**: 3D model file.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | OBJ, GLTF | Loaded via MeshLoader |
| **Unity** | FBX, OBJ, GLTF | Imported as mesh assets |
| **Unreal** | FBX, OBJ | Imported as StaticMesh |
| **Godot** | GLTF, OBJ, FBX | Imported as mesh resources |

---

### Texture Asset

**Definition**: Image file for materials.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | PNG, JPG | Loaded via image crate |
| **Unity** | PNG, JPG, TGA, etc. | Imported as Texture2D |
| **Unreal** | PNG, TGA, EXR, etc. | Imported as Texture2D |
| **Godot** | PNG, JPG, WebP, etc. | Imported as Texture2D |

---

### Prefab / Blueprint / Scene Instance

**Definition**: Reusable entity/object template.

| Engine | Term | Notes |
|--------|------|-------|
| **Praxis** | Scene serialization | Save/load entity hierarchies |
| **Unity** | Prefab | Reusable GameObject template |
| **Unreal** | Blueprint Class | C++ or Blueprint actor template |
| **Godot** | Scene (instanced) | Scenes are reusable node trees |

---

## Quick Reference: Common Workflows

### Creating a Lit 3D Object

| Engine | Steps |
|--------|-------|
| **Praxis** | 1. Spawn entity<br>2. Add `Transform`, `GlobalTransform`<br>3. Add `MeshHandle` (reference to loaded mesh)<br>4. Add `MaterialProperties` (optional)<br>5. Add light entities (`DirectionalLight` or `PointLight`) |
| **Unity** | 1. Create GameObject<br>2. Add MeshFilter + MeshRenderer<br>3. Assign Material to MeshRenderer<br>4. Add Light component to another GameObject |
| **Unreal** | 1. Place StaticMeshActor<br>2. Assign StaticMesh<br>3. Assign Material<br>4. Add Light Actor |
| **Godot** | 1. Create MeshInstance3D node<br>2. Assign Mesh resource<br>3. Assign Material<br>4. Add Light3D node |

---

### Setting Up Physics Collision

| Engine | Steps |
|--------|-------|
| **Praxis** | 1. Add `RigidBody` component (Dynamic/Static/Kinematic)<br>2. Add `Collider` component (shape)<br>3. Physics syncs with Transform via systems |
| **Unity** | 1. Add Rigidbody component<br>2. Add Collider component (BoxCollider, SphereCollider, etc.)<br>3. Set mass and drag |
| **Unreal** | 1. StaticMeshComponent has collision by default<br>2. Enable "Simulate Physics" on component<br>3. Set collision channels |
| **Godot** | 1. Add RigidBody3D node<br>2. Add CollisionShape3D child<br>3. Assign Shape3D resource |

---

### Playing an Animation

| Engine | Steps |
|--------|-------|
| **Praxis** | 1. Add `Skeleton` component<br>2. Add `AnimationPlayer` component<br>3. Load `AnimationClip`<br>4. Call `play()` on AnimationPlayer |
| **Unity** | 1. Add Animator component<br>2. Assign AnimatorController<br>3. Set animation states in controller<br>4. Trigger transitions |
| **Unreal** | 1. Skeletal Mesh has Skeleton<br>2. Create Animation Blueprint<br>3. Play animation in AnimGraph |
| **Godot** | 1. Add AnimationPlayer node<br>2. Create Animation resource<br>3. Add keyframes<br>4. Call `play()` |

---

## Terminology Comparison Chart

### Quick Lookup Table

| Concept | Praxis | Unity | Unreal | Godot |
|---------|--------|-------|--------|-------|
| **Basic object** | Entity | GameObject | AActor | Node |
| **Data on object** | Component | Component | UActorComponent | Node property/script |
| **Logic processor** | System (function) | System/Update() | Tick()/Subsystem | _process() |
| **Container** | World | Scene | UWorld/ULevel | Scene/SceneTree |
| **3D position** | Transform | Transform | FTransform | Transform3D |
| **World position** | GlobalTransform | position/rotation | GetComponentTransform() | global_transform |
| **3D model** | MeshHandle | MeshFilter+Mesh | UStaticMesh | MeshInstance3D |
| **Appearance** | MaterialProperties | Material | UMaterial | Material |
| **View** | Camera | Camera | UCameraComponent | Camera3D |
| **Bones** | Skeleton | Humanoid/Generic | USkeleton | Skeleton3D |
| **Animation data** | AnimationClip | AnimationClip | UAnimSequence | Animation |
| **Animation player** | AnimationPlayer | Animator | UAnimInstance | AnimationPlayer |
| **Physics object** | RigidBody | Rigidbody | Simulate Physics | RigidBody3D |
| **Collision shape** | Collider | BoxCollider/etc. | UShapeComponent | CollisionShape3D |
| **Sun** | DirectionalLight | Light (Directional) | DirectionalLight | DirectionalLight3D |
| **Bulb** | PointLight | Light (Point) | PointLight | OmniLight3D |
| **Sound source** | Audio playback | AudioSource | UAudioComponent | AudioStreamPlayer3D |
| **Code** | Lua script | C# script | Blueprint/C++ | GDScript/C# |
| **GPU program** | GLSL shader | ShaderLab/HLSL | Material/HLSL | GSL |
| **Reusable template** | Scene file | Prefab | Blueprint | Scene (instanced) |

---

## See Also

- **[Language Guide](LANGUAGE_GUIDE.md)** - Help translating concepts to different programming languages
- **[Curriculum](CURRICULUM.md)** - Language-agnostic game engine architecture course
- **[Code Examples](CODE_EXAMPLES.md)** - Side-by-side implementations in Rust, C++, C#
- **[Universal Patterns](patterns/)** - Design patterns independent of engine or language
