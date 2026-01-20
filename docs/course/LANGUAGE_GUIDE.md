# Language Translation Guide

A practical guide for translating game engine concepts from Praxis (Rust) to other popular programming languages used in game development.

## Table of Contents

- [Overview](#overview)
- [Language-Specific Quick Start](#language-specific-quick-start)
- [Core Concepts by Language](#core-concepts-by-language)
  - [Memory Management](#memory-management)
  - [ECS Architecture](#ecs-architecture)
  - [Transform System](#transform-system)
  - [Rendering Pipeline](#rendering-pipeline)
  - [Component Definition](#component-definition)
  - [System Implementation](#system-implementation)
  - [Asset Loading](#asset-loading)
  - [Error Handling](#error-handling)
- [Common Patterns](#common-patterns)
- [Pitfalls and Solutions](#pitfalls-and-solutions)

---

## Overview

This guide helps you translate concepts learned in Praxis to your target language/engine:

- **From Rust to C++**: Unreal Engine, custom engines
- **From Rust to C#**: Unity, Godot (C#)
- **From Rust to GDScript**: Godot
- **From Rust to TypeScript/JavaScript**: Web-based engines (Three.js, Babylon.js)
- **From Rust to Python**: Educational/prototyping engines

Each section provides:
- **Conceptual mapping**: How the concept translates
- **Code examples**: Side-by-side implementations
- **Gotchas**: Common mistakes when translating
- **Best practices**: Idiomatic approaches in each language

---

## Language-Specific Quick Start

### If You're Learning C++ (Unreal Engine)

**Key differences from Rust:**
- Manual memory management (unless using smart pointers)
- Raw pointers vs. Rust references
- Header/source file split
- Macros for reflection (UPROPERTY, UFUNCTION)

**What translates well:**
- Struct-based components
- System functions operating on components
- Value semantics with structs

**What's different:**
- Ownership: Manual delete or shared_ptr instead of automatic Drop
- Null safety: Optional pointers (can be null) vs. Option<T>
- Inheritance: Common in C++, avoided in Rust

**Start here**: [Memory Management (C++)](#c-unreal-style)

---

### If You're Learning C# (Unity)

**Key differences from Rust:**
- Garbage collected (no manual memory management)
- Null references possible (until nullable reference types)
- Object-oriented by default
- MonoBehaviour lifecycle methods

**What translates well:**
- Structs for data (components in Unity DOTS)
- LINQ for queries (similar to ECS queries)
- Value types vs reference types

**What's different:**
- Ownership: Garbage collector vs. borrow checker
- Mutability: Mutable by default vs. immutable by default
- Polymorphism: Interface-based vs. trait-based

**Start here**: [Memory Management (C#)](#c-unity-style)

---

### If You're Learning GDScript (Godot)

**Key differences from Rust:**
- Dynamically typed (unless using typed GDScript)
- Reference counting for objects
- Node-based architecture
- Built-in signals for events

**What translates well:**
- Signal pattern similar to Rust event systems
- Structs as dictionaries or custom classes
- Functional programming with lambdas

**What's different:**
- Types: Dynamic typing vs. static typing
- Memory: Reference counting vs. ownership
- Architecture: Node tree vs. ECS

**Start here**: [ECS Architecture (GDScript)](#gdscript-godot-style)

---

### If You're Learning TypeScript (Web Engines)

**Key differences from Rust:**
- Garbage collected
- Prototype-based objects
- Asynchronous by default (Promises, async/await)
- No value types (everything is a reference except primitives)

**What translates well:**
- Type annotations (TypeScript)
- Functional programming patterns
- Entity-component systems (libraries like Miniplex, bitECS)

**What's different:**
- Async: JavaScript event loop vs. Rust async
- Types: Structural typing vs. nominal typing
- Performance: JIT compilation vs. ahead-of-time compilation

**Start here**: [Component Definition (TypeScript)](#typescript-web-engines)

---

### If You're Learning Python

**Key differences from Rust:**
- Dynamically typed (unless using type hints)
- Garbage collected
- Interpreted (slower than compiled)
- Duck typing

**What translates well:**
- Type hints (Python 3.5+)
- Dataclasses for components
- Functional programming

**What's different:**
- Performance: ~10-100x slower than Rust
- Types: Optional vs. required
- Concurrency: GIL (Global Interpreter Lock) vs. fearless concurrency

**Start here**: [Component Definition (Python)](#python-educational)

---

## Core Concepts by Language

### Memory Management

Memory management is fundamental to understanding how concepts translate across languages.

---

#### Rust (Praxis)

**Concept**: Ownership, borrowing, lifetimes

```rust
// Ownership: value moved to new owner
let mesh = load_mesh("cube.obj");
let mesh_handle = mesh_manager.add(mesh); // mesh moved, can't use it anymore

// Borrowing: temporary access without ownership
fn render_mesh(mesh: &Mesh) { // Immutable borrow
    // Read-only access
}

fn transform_mesh(mesh: &mut Mesh) { // Mutable borrow
    // Can modify
}

// Lifetimes: ensure references are valid
struct MeshRef<'a> {
    mesh: &'a Mesh, // Reference valid for lifetime 'a
}

// Arc: shared ownership with reference counting
use std::sync::Arc;
let device: Arc<Device> = ...;
let queue = device.clone(); // Reference count incremented
```

**Key points**:
- Values are moved by default
- Borrow checker prevents data races at compile time
- Automatic cleanup when owner goes out of scope
- Arc for shared ownership across threads

---

#### C++ (Unreal Style)

**Concept**: Manual memory or smart pointers

```cpp
// Manual memory management (old style, avoid)
Mesh* mesh = new Mesh();
// ... use mesh ...
delete mesh; // Must manually free

// Unique ownership: std::unique_ptr (similar to Rust Box)
std::unique_ptr<Mesh> mesh = std::make_unique<Mesh>();
// Automatically deleted when mesh goes out of scope

// Shared ownership: std::shared_ptr (similar to Rust Arc)
std::shared_ptr<Device> device = std::make_shared<Device>();
std::shared_ptr<Device> queue = device; // Reference count incremented

// Unreal Engine smart pointers
TSharedPtr<FMesh> mesh = MakeShared<FMesh>();
TSharedRef<FMesh> meshRef = mesh.ToSharedRef(); // Must be non-null

// Unreal garbage-collected objects (UObject)
UPROPERTY()
UStaticMesh* mesh; // Garbage collected, don't delete manually

// Const references (similar to Rust &T)
void RenderMesh(const FMesh& mesh) { // Read-only
    // Cannot modify mesh
}

// Mutable references (similar to Rust &mut T)
void TransformMesh(FMesh& mesh) { // Mutable
    // Can modify mesh
}
```

**Gotchas**:
- Null pointers possible (unlike Rust Option)
- Must remember to delete manually allocated memory
- Unreal's garbage collector only works for UObject-derived classes
- Shared pointers have runtime overhead (unlike compile-time Rust checks)

**Best practices**:
- Prefer smart pointers over raw pointers
- Use const references for read-only access
- Use Unreal's smart pointers (TSharedPtr, TUniquePtr) in Unreal projects
- Mark UObject properties with UPROPERTY() for GC

---

#### C# (Unity Style)

**Concept**: Garbage collection

```csharp
// Reference types (class): garbage collected, can be null
class Mesh {
    public Vector3[] vertices;
}

Mesh mesh = new Mesh(); // Allocated on heap
// No need to manually free, GC handles it

// Null safety (C# 8.0+)
Mesh? nullableMesh = null; // Explicitly nullable
Mesh mesh = new Mesh();    // Non-nullable (but runtime checked)

// Value types (struct): stack allocated, copied on assignment
struct Transform {
    public Vector3 position;
    public Quaternion rotation;
}

Transform t1 = new Transform();
Transform t2 = t1; // Copy, not reference
t2.position = new Vector3(1, 0, 0); // t1 unchanged

// Readonly references (similar to Rust &T)
void RenderMesh(in Mesh mesh) { // Readonly reference
    // Cannot modify mesh
}

// Unity object lifetime
public class MyComponent : MonoBehaviour {
    public GameObject prefab; // Unity manages lifetime
    
    void Start() {
        GameObject instance = Instantiate(prefab);
        Destroy(instance, 5.0f); // Destroyed after 5 seconds
    }
}
```

**Gotchas**:
- Null reference exceptions at runtime (no compile-time prevention)
- GC pauses can cause frame hitches
- Struct vs class behavior different (value vs reference)
- Unity objects require Destroy(), not GC

**Best practices**:
- Use nullable reference types (C# 8.0+)
- Minimize allocations in hot loops (avoid GC pressure)
- Use struct for small data types (< 16 bytes)
- Never use null for Unity objects (use != null checks)

---

#### GDScript (Godot Style)

**Concept**: Reference counting

```gdscript
# Objects are reference counted
var mesh = Mesh.new() # Reference count = 1
var mesh_copy = mesh  # Reference count = 2
mesh_copy = null      # Reference count = 1
mesh = null           # Reference count = 0, freed

# Nodes managed by scene tree
var sprite = Sprite.new()
add_child(sprite) # Scene tree owns it now
sprite.queue_free() # Schedule for deletion

# Weak references (don't increment count)
var weak_ref = weakref(mesh)
if weak_ref.get_ref():
    # Object still alive
    pass

# Pass by reference (all objects)
func modify_mesh(mesh_param):
    mesh_param.scale = Vector3(2, 2, 2) # Modifies original

# Value types (built-in): copied on assignment
var pos1 = Vector3(1, 0, 0)
var pos2 = pos1 # Copy
pos2.x = 2      # pos1 unchanged
```

**Gotchas**:
- Circular references cause memory leaks (no cycle detection)
- Nodes in scene tree require queue_free() or remove_child()
- No explicit mutability (everything mutable)
- Weak references needed for observer patterns

**Best practices**:
- Use signals instead of storing references
- Always queue_free() removed nodes
- Use weakref() for non-owning references
- Avoid circular references between scripts

---

#### TypeScript (Web Engines)

**Concept**: Garbage collection + references

```typescript
// All objects are references (garbage collected)
class Mesh {
    vertices: Float32Array;
    
    constructor() {
        this.vertices = new Float32Array(100);
    }
}

let mesh = new Mesh(); // Heap allocated, GC managed
let meshRef = mesh;    // Reference, not copy
meshRef.vertices[0] = 1.0; // Modifies original mesh

// Primitives are copied
let x = 5;
let y = x; // Copy
y = 10;    // x still 5

// Optional types (TypeScript)
let mesh: Mesh | null = null; // Explicitly nullable
let meshNotNull: Mesh = new Mesh(); // Still can be null at runtime

// Readonly (TypeScript, not enforced at runtime)
function renderMesh(mesh: Readonly<Mesh>): void {
    // mesh.vertices = new Float32Array(); // TypeScript error
    // mesh.vertices[0] = 1.0; // Allowed (Readonly is shallow)
}

// Immutability with const
const mesh = new Mesh();
// mesh = new Mesh(); // Error: can't reassign
mesh.vertices[0] = 1.0; // Allowed: const is shallow

// Deep readonly (custom type)
type DeepReadonly<T> = {
    readonly [P in keyof T]: DeepReadonly<T[P]>;
};

function renderMeshStrict(mesh: DeepReadonly<Mesh>): void {
    // mesh.vertices[0] = 1.0; // TypeScript error
}
```

**Gotchas**:
- TypeScript types erased at runtime (no runtime safety)
- Const and readonly are shallow by default
- No true ownership concept
- Null/undefined checks needed everywhere

**Best practices**:
- Enable strict null checks in tsconfig.json
- Use readonly for function parameters that shouldn't mutate
- Prefer immutable patterns (return new objects)
- Use const for all variables unless reassignment needed

---

#### Python (Educational)

**Concept**: Garbage collection + reference counting

```python
# Objects are references (garbage collected + refcounting)
class Mesh:
    def __init__(self):
        self.vertices = []

mesh = Mesh()  # Reference count = 1
mesh_ref = mesh  # Reference count = 2
mesh_ref.vertices.append([0, 0, 0])  # Modifies original
del mesh  # Reference count = 1
del mesh_ref  # Reference count = 0, freed

# Immutable types: copied on assignment
x = 5
y = x  # Copy
y = 10  # x still 5

# Type hints (Python 3.5+)
from typing import Optional

def render_mesh(mesh: Mesh) -> None:
    # mesh is hinted as Mesh, but can still be None at runtime
    pass

def render_mesh_safe(mesh: Optional[Mesh]) -> None:
    if mesh is not None:
        # mesh is Mesh here (type narrowing)
        pass

# Dataclasses for immutability (Python 3.7+)
from dataclasses import dataclass

@dataclass(frozen=True)
class Transform:
    position: tuple[float, float, float]
    rotation: tuple[float, float, float, float]

transform = Transform((0, 0, 0), (0, 0, 0, 1))
# transform.position = (1, 0, 0)  # Error: frozen

# Shallow vs deep copy
import copy

mesh1 = Mesh()
mesh1.vertices = [[1, 2, 3]]

mesh2 = mesh1  # Reference (same object)
mesh3 = copy.copy(mesh1)  # Shallow copy (shares vertices list)
mesh4 = copy.deepcopy(mesh1)  # Deep copy (independent)
```

**Gotchas**:
- Type hints not enforced at runtime (use mypy for checking)
- Everything is mutable by default
- Circular references cause memory retention until GC runs
- Performance: ~10-100x slower than Rust/C++

**Best practices**:
- Use type hints everywhere (checked with mypy)
- Use dataclasses with frozen=True for immutability
- Avoid mutable default arguments
- Profile before optimizing (use cProfile)

---

### ECS Architecture

Entity-Component-System is central to Praxis. Here's how to implement or emulate it in other languages.

---

#### Rust (Praxis)

```rust
use bevy_ecs::prelude::*;

// Define components
#[derive(Component)]
struct Transform {
    position: Vec3,
    rotation: Quat,
}

#[derive(Component)]
struct Velocity {
    value: Vec3,
}

// Define system
fn movement_system(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.position += velocity.value * time.delta_seconds();
    }
}

// Create world and run system
let mut world = World::new();
let mut schedule = Schedule::default();
schedule.add_systems(movement_system);

// Spawn entities
world.spawn((
    Transform { position: Vec3::ZERO, rotation: Quat::IDENTITY },
    Velocity { value: Vec3::new(1.0, 0.0, 0.0) },
));

schedule.run(&mut world);
```

---

#### C++ (Unreal Style)

```cpp
// Unreal doesn't use strict ECS, but Actor-Component model

// Component definition (data + logic)
UCLASS()
class UTransformComponent : public UActorComponent {
    GENERATED_BODY()

public:
    UPROPERTY(EditAnywhere)
    FVector Position;

    UPROPERTY(EditAnywhere)
    FQuat Rotation;

    virtual void TickComponent(float DeltaTime, ...) override;
};

UCLASS()
class UVelocityComponent : public UActorComponent {
    GENERATED_BODY()

public:
    UPROPERTY(EditAnywhere)
    FVector Value;
};

// "System" as component tick (not pure ECS)
void UTransformComponent::TickComponent(float DeltaTime, ...) {
    UVelocityComponent* Velocity = GetOwner()->FindComponentByClass<UVelocityComponent>();
    if (Velocity) {
        Position += Velocity->Value * DeltaTime;
    }
}

// Or as subsystem (closer to ECS)
UCLASS()
class UMovementSubsystem : public UWorldSubsystem {
    GENERATED_BODY()

public:
    virtual void Tick(float DeltaTime) override {
        // Iterate all actors with Transform + Velocity
        for (TActorIterator<AActor> It(GetWorld()); It; ++It) {
            UTransformComponent* Transform = It->FindComponentByClass<UTransformComponent>();
            UVelocityComponent* Velocity = It->FindComponentByClass<UVelocityComponent>();
            
            if (Transform && Velocity) {
                Transform->Position += Velocity->Value * DeltaTime;
            }
        }
    }
};
```

**Adapting ECS to Unreal**:
- Components have logic (not pure data)
- "Systems" are either component Tick() or subsystems
- Actors are heavier than ECS entities
- Use TActorIterator for system-like queries

---

#### C# (Unity Style)

```csharp
using Unity.Entities;
using Unity.Transforms;
using Unity.Mathematics;

// Define components (pure data)
public struct Velocity : IComponentData {
    public float3 Value;
}

// System (pure logic)
public partial class MovementSystem : SystemBase {
    protected override void OnUpdate() {
        float deltaTime = Time.DeltaTime;
        
        // Query and iterate
        Entities.ForEach((ref Translation translation, in Velocity velocity) => {
            translation.Value += velocity.Value * deltaTime;
        }).ScheduleParallel();
    }
}

// Or traditional MonoBehaviour (OOP style)
public class VelocityComponent : MonoBehaviour {
    public Vector3 velocity;
    
    void Update() {
        transform.position += velocity * Time.deltaTime;
    }
}
```

**Unity's two worlds**:
- **DOTS** (Data-Oriented Tech Stack): True ECS, high performance
- **MonoBehaviour**: Traditional OOP, easier but slower

**Translating Praxis to Unity**:
- Praxis ECS → Unity DOTS
- Components → struct IComponentData
- Systems → SystemBase
- Queries → Entities.ForEach() or Entities.WithAll()

---

#### GDScript (Godot Style)

```gdscript
# Godot uses Node-based architecture, not ECS
# But can emulate ECS patterns

# Component as script
class_name VelocityComponent extends Node

var value: Vector3 = Vector3.ZERO

# Entity as Node
class_name Entity extends Node3D

var velocity_component: VelocityComponent

func _ready():
    velocity_component = VelocityComponent.new()
    add_child(velocity_component)

# System as autoload singleton
# Create MovementSystem.gd and add to AutoLoad

extends Node

func _process(delta):
    # Query all entities with velocity
    for entity in get_tree().get_nodes_in_group("entities"):
        if entity.has_node("VelocityComponent"):
            var velocity = entity.get_node("VelocityComponent")
            entity.position += velocity.value * delta

# In Entity script, add to group
func _ready():
    add_to_group("entities")
```

**Emulating ECS in Godot**:
- Entities → Nodes with group membership
- Components → Child nodes or script properties
- Systems → AutoLoad singletons or scene-level scripts
- Queries → get_tree().get_nodes_in_group()

**Limitations**:
- Not cache-friendly (nodes are scattered in memory)
- Slower than true ECS
- More GDScript overhead

**Alternative**: Use Godot addon like "godot-ecs" for true ECS

---

#### TypeScript (Web Engines)

```typescript
// Using bitECS (entity component system library)
import { createWorld, defineComponent, defineQuery, addEntity, addComponent, pipe } from 'bitecs';

// Define components (struct-of-arrays)
const Position = defineComponent({ x: Types.f32, y: Types.f32, z: Types.f32 });
const Velocity = defineComponent({ x: Types.f32, y: Types.f32, z: Types.f32 });

// Create world
const world = createWorld();

// Define query
const movementQuery = defineQuery([Position, Velocity]);

// Define system
const movementSystem = (world) => {
    const entities = movementQuery(world);
    const deltaTime = 0.016; // ~60 FPS
    
    for (let i = 0; i < entities.length; i++) {
        const eid = entities[i];
        Position.x[eid] += Velocity.x[eid] * deltaTime;
        Position.y[eid] += Velocity.y[eid] * deltaTime;
        Position.z[eid] += Velocity.z[eid] * deltaTime;
    }
    
    return world;
};

// Create pipeline
const pipeline = pipe(movementSystem /* , otherSystems... */);

// Spawn entity
const entity = addEntity(world);
addComponent(world, Position, entity);
addComponent(world, Velocity, entity);
Position.x[entity] = 0;
Position.y[entity] = 0;
Position.z[entity] = 0;
Velocity.x[entity] = 1;

// Run pipeline
pipeline(world);
```

**Libraries**:
- **bitECS**: Archetype-based, very fast
- **Miniplex**: More ergonomic, slower
- **ECSY**: Object-oriented ECS

**Translating Praxis to TypeScript**:
- Use bitECS for performance-critical projects
- Components → defineComponent()
- Systems → functions that accept world
- Queries → defineQuery()

---

#### Python (Educational)

```python
from dataclasses import dataclass
from typing import List, Set

# Define components
@dataclass
class Transform:
    position: tuple[float, float, float]
    rotation: tuple[float, float, float, float]

@dataclass
class Velocity:
    value: tuple[float, float, float]

# Entity is just an ID + components
class Entity:
    _next_id = 0
    
    def __init__(self):
        self.id = Entity._next_id
        Entity._next_id += 1
        self.components = {}
    
    def add_component(self, component):
        self.components[type(component)] = component
    
    def get_component(self, component_type):
        return self.components.get(component_type)
    
    def has_component(self, component_type):
        return component_type in self.components

# World contains entities
class World:
    def __init__(self):
        self.entities: List[Entity] = []
    
    def spawn(self, *components) -> Entity:
        entity = Entity()
        for component in components:
            entity.add_component(component)
        self.entities.append(entity)
        return entity
    
    def query(self, *component_types):
        """Yield entities with all specified components"""
        for entity in self.entities:
            if all(entity.has_component(ct) for ct in component_types):
                yield entity

# System is a function
def movement_system(world: World, delta_time: float):
    for entity in world.query(Transform, Velocity):
        transform = entity.get_component(Transform)
        velocity = entity.get_component(Velocity)
        
        # Create new transform (immutable)
        new_pos = (
            transform.position[0] + velocity.value[0] * delta_time,
            transform.position[1] + velocity.value[1] * delta_time,
            transform.position[2] + velocity.value[2] * delta_time,
        )
        entity.add_component(Transform(new_pos, transform.rotation))

# Usage
world = World()
world.spawn(
    Transform((0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0)),
    Velocity((1.0, 0.0, 0.0))
)

movement_system(world, 0.016)
```

**This is educational, not production-ready**:
- Very slow compared to Rust ECS
- Not cache-friendly
- Linear search for queries

**Production alternatives**:
- Use Rust + PyO3 bindings
- Use C++ ECS library with Python bindings
- Accept performance limitations for prototyping

---

### Transform System

Transforms are fundamental to 3D engines. Here's how to handle them in different languages.

---

#### Rust (Praxis)

```rust
use glam::{Vec3, Quat, Mat4};

#[derive(Component)]
struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.translation,
        )
    }
}

#[derive(Component)]
struct GlobalTransform {
    pub matrix: Mat4,
}

#[derive(Component)]
struct Parent(pub Entity);

#[derive(Component)]
struct Children(pub Vec<Entity>);

// System: propagate transforms through hierarchy
fn propagate_transforms(
    mut root_query: Query<
        (&Transform, &mut GlobalTransform, Option<&Children>),
        (Changed<Transform>, Without<Parent>)
    >,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>)>,
) {
    for (transform, mut global, children) in root_query.iter_mut() {
        global.matrix = transform.to_matrix();
        
        if let Some(children) = children {
            propagate_recursive(&mut child_query, &children.0, &global);
        }
    }
}

fn propagate_recursive(
    query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>)>,
    children: &[Entity],
    parent_global: &GlobalTransform,
) {
    for &child in children {
        if let Ok((transform, mut global, grandchildren)) = query.get_mut(child) {
            global.matrix = parent_global.matrix * transform.to_matrix();
            
            if let Some(grandchildren) = grandchildren {
                propagate_recursive(query, &grandchildren.0, &global);
            }
        }
    }
}
```

---

#### C++ (Unreal Style)

```cpp
// Unreal has built-in transform hierarchy via USceneComponent

UCLASS()
class AMyActor : public AActor {
    GENERATED_BODY()

public:
    UPROPERTY()
    USceneComponent* Root;

    UPROPERTY()
    USceneComponent* Child;

    AMyActor() {
        Root = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
        RootComponent = Root;

        Child = CreateDefaultSubobject<USceneComponent>(TEXT("Child"));
        Child->SetupAttachment(Root); // Child transform relative to Root
    }

    void UpdateTransforms() {
        // Set local transform
        Root->SetRelativeLocation(FVector(1, 0, 0));
        Root->SetRelativeRotation(FRotator(0, 90, 0));

        // Get world transform (automatic propagation)
        FTransform WorldTransform = Root->GetComponentTransform();

        // Child world transform
        FTransform ChildWorldTransform = Child->GetComponentTransform();
        // Automatically: Root world * Child relative
    }
};

// Custom transform (if not using SceneComponent)
struct FMyTransform {
    FVector Translation;
    FQuat Rotation;
    FVector Scale;

    FMatrix ToMatrix() const {
        return FScaleRotationTranslationMatrix(Scale, Rotation, Translation);
    }

    FMyTransform operator*(const FMyTransform& Other) const {
        FMatrix M1 = this->ToMatrix();
        FMatrix M2 = Other.ToMatrix();
        FMatrix Result = M1 * M2;
        
        return FMyTransform {
            Result.GetOrigin(),
            Result.ToQuat(),
            Result.GetScaleVector()
        };
    }
};
```

**Unreal's built-in hierarchy**:
- USceneComponent handles transforms automatically
- SetRelativeLocation/Rotation/Scale for local
- GetComponentTransform() for world (cached)
- Attachment via SetupAttachment() or AttachToComponent()

---

#### C# (Unity Style)

```csharp
using UnityEngine;

// Unity has built-in Transform component on all GameObjects

public class TransformExample : MonoBehaviour {
    void Start() {
        // Local transform (relative to parent)
        transform.localPosition = new Vector3(1, 0, 0);
        transform.localRotation = Quaternion.Euler(0, 90, 0);
        transform.localScale = new Vector3(1, 1, 1);

        // World transform
        Vector3 worldPos = transform.position;
        Quaternion worldRot = transform.rotation;
        Vector3 worldScale = transform.lossyScale; // Readonly!

        // Hierarchy
        Transform child = transform.GetChild(0);
        Transform parent = transform.parent;

        // Matrix
        Matrix4x4 localToWorld = transform.localToWorldMatrix;
        Matrix4x4 worldToLocal = transform.worldToLocalMatrix;
    }
}

// Custom transform (Unity DOTS)
using Unity.Mathematics;

public struct LocalTransform : IComponentData {
    public float3 Position;
    public quaternion Rotation;
    public float3 Scale;

    public float4x4 ToMatrix() {
        return float4x4.TRS(Position, Rotation, Scale);
    }
}

public struct GlobalTransform : IComponentData {
    public float4x4 Matrix;
}

// Transform propagation system
public partial class TransformSystem : SystemBase {
    protected override void OnUpdate() {
        // Root entities (no parent)
        Entities
            .WithNone<Parent>()
            .ForEach((ref GlobalTransform global, in LocalTransform local) => {
                global.Matrix = local.ToMatrix();
            }).ScheduleParallel();

        // Child entities
        Entities
            .WithAll<Parent>()
            .ForEach((ref GlobalTransform global, in LocalTransform local, in Parent parent) => {
                // Simplified: assumes parent GlobalTransform already updated
                // Real implementation needs dependency ordering
                global.Matrix = GetComponent<GlobalTransform>(parent.Value).Matrix * local.ToMatrix();
            }).Schedule(); // Can't parallelize due to parent dependency
    }
}
```

**Unity's two approaches**:
- **GameObject Transform**: Automatic, always present, hierarchy built-in
- **DOTS LocalTransform**: Manual system needed, more control

---

#### GDScript (Godot Style)

```gdscript
extends Node3D

# Godot's built-in transform system

func _ready():
    # Local transform (relative to parent)
    position = Vector3(1, 0, 0)
    rotation = Vector3(0, PI/2, 0) # Euler angles
    scale = Vector3(1, 1, 1)
    
    # Or use Transform3D directly
    transform = Transform3D(Basis(), Vector3(1, 0, 0))
    
    # World (global) transform
    var world_pos = global_position
    var world_rot = global_rotation
    var world_transform = global_transform
    
    # Hierarchy
    var child = get_child(0)
    var parent = get_parent()
    
    # Matrix operations
    var local_to_world = global_transform
    var world_to_local = global_transform.affine_inverse()

# Custom transform (for ECS-like system)
class_name MyTransform

var translation: Vector3
var rotation: Quat
var scale: Vector3

func to_matrix() -> Transform3D:
    var basis = Basis(rotation).scaled(scale)
    return Transform3D(basis, translation)

static func multiply(parent: MyTransform, child: MyTransform) -> MyTransform:
    var parent_mat = parent.to_matrix()
    var child_mat = child.to_matrix()
    var result_mat = parent_mat * child_mat
    
    return MyTransform.new(
        result_mat.origin,
        result_mat.basis.get_rotation_quaternion(),
        result_mat.basis.get_scale()
    )
```

**Godot's built-in hierarchy**:
- Node3D has transform automatically
- position/rotation/scale for local
- global_position/global_rotation/global_transform for world
- Hierarchy via Node tree (add_child/get_parent)

---

### Rendering Pipeline

Understanding how to submit rendering commands.

---

#### Rust (Praxis)

```rust
use praxis_graphics::{RenderContext, DrawCommand, RenderCommands};

// Unified rendering API
let draw_commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::IDENTITY,
        texture_name: Some("wall_texture".to_string()),
        material_properties: None,
    },
];

let cmds = RenderCommands {
    view: camera_view_matrix,
    proj: camera_projection_matrix,
    draw_commands: &draw_commands,
    lighting: Some(&lighting_uniforms),
};

render_context.render(&cmds)?;
```

**Key concepts**:
- DrawCommand: per-object data (mesh, transform, material)
- RenderCommands: per-frame data (camera, lights, objects)
- Single render() call handles everything

---

#### C++ (Unreal Style)

```cpp
// Unreal's rendering is event-driven via delegates

void AMyActor::Render() {
    // High-level: Unreal handles rendering automatically
    // Components register themselves for rendering

    UStaticMeshComponent* Mesh = GetComponentByClass<UStaticMeshComponent>();
    Mesh->SetStaticMesh(CubeMesh);
    Mesh->SetMaterial(0, WallMaterial);
    Mesh->SetWorldTransform(FTransform::Identity);
    // Rendering happens automatically in engine tick
}

// Low-level custom rendering (advanced)
class FMySceneProxy : public FPrimitiveSceneProxy {
public:
    virtual void GetDynamicMeshElements(...) const override {
        // Submit draw commands to renderer
        FMeshBatch MeshBatch;
        MeshBatch.VertexFactory = VertexFactory;
        MeshBatch.MaterialRenderProxy = Material->GetRenderProxy();
        // ... setup mesh batch

        Collector.AddMesh(ViewIndex, MeshBatch);
    }
};
```

**Unreal's approach**:
- High-level: Set component properties, engine renders
- Low-level: Implement SceneProxy for custom rendering
- No explicit render() call in game code

---

#### C# (Unity Style)

```csharp
using UnityEngine;

// Unity's automatic rendering
public class RenderExample : MonoBehaviour {
    void Start() {
        // High-level: Add MeshRenderer + MeshFilter
        var meshFilter = gameObject.AddComponent<MeshFilter>();
        var meshRenderer = gameObject.AddComponent<MeshRenderer>();

        meshFilter.mesh = cubeMesh;
        meshRenderer.material = wallMaterial;
        // Unity renders automatically
    }
}

// Manual rendering with Graphics API
public class ManualRender : MonoBehaviour {
    public Mesh mesh;
    public Material material;

    void Update() {
        Matrix4x4 matrix = transform.localToWorldMatrix;
        
        // Draw mesh manually
        Graphics.DrawMesh(
            mesh,
            matrix,
            material,
            layer: 0,
            camera: null, // All cameras
            submeshIndex: 0
        );
    }
}

// Scriptable Render Pipeline (advanced)
using UnityEngine.Rendering;

public class MyRenderPass : ScriptableRenderPass {
    public override void Execute(ScriptableRenderContext context, ref RenderingData renderingData) {
        CommandBuffer cmd = CommandBufferPool.Get("MyPass");

        // Clear
        cmd.ClearRenderTarget(true, true, Color.black);

        // Draw
        cmd.DrawMesh(mesh, Matrix4x4.identity, material);

        context.ExecuteCommandBuffer(cmd);
        CommandBufferPool.Release(cmd);
    }
}
```

**Unity's approaches**:
- **Automatic**: MeshRenderer component
- **Manual**: Graphics.DrawMesh()
- **Custom pipeline**: Scriptable Render Pipeline

---

#### TypeScript (Three.js)

```typescript
import * as THREE from 'three';

// Three.js scene graph
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);

// Create mesh
const geometry = new THREE.BoxGeometry(1, 1, 1);
const material = new THREE.MeshStandardMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

// Camera and lights
camera.position.z = 5;
const light = new THREE.DirectionalLight(0xffffff, 1);
light.position.set(1, 1, 1);
scene.add(light);

// Renderer
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Render loop
function animate() {
    requestAnimationFrame(animate);

    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;

    renderer.render(scene, camera);
}
animate();
```

**Three.js approach**:
- Scene graph with Mesh objects
- Automatic frustum culling
- Single renderer.render() call
- WebGL abstraction

---

### Component Definition

How to define components in each language.

---

#### Rust (Praxis)

```rust
use bevy_ecs::component::Component;
use glam::{Vec3, Quat};

// Simple component
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

// Component with methods
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { translation, rotation, scale }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            translation: Vec3::new(x, y, z),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

// Component with lifetime (rare)
#[derive(Component)]
pub struct MeshRef<'a> {
    pub mesh: &'a Mesh,
}

// Component with generics
#[derive(Component)]
pub struct ResourceHandle<T> {
    pub id: String,
    _phantom: std::marker::PhantomData<T>,
}

// Tag component (zero-size)
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;
```

---

#### C++ (Unreal Style)

```cpp
// Unreal component (UActorComponent)
UCLASS()
class MYGAME_API UHealthComponent : public UActorComponent {
    GENERATED_BODY()

public:
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Health")
    float Current = 100.0f;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Health")
    float Max = 100.0f;

    UFUNCTION(BlueprintCallable, Category = "Health")
    void TakeDamage(float Amount) {
        Current = FMath::Max(0.0f, Current - Amount);
    }

    // Tick function (system-like)
    virtual void TickComponent(float DeltaTime, ...) override {
        // Logic here
    }
};

// POD struct (closer to pure data component)
USTRUCT(BlueprintType)
struct FTransformData {
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    FVector Translation;

    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    FQuat Rotation;

    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    FVector Scale;

    FMatrix ToMatrix() const {
        return FScaleRotationTranslationMatrix(Scale, Rotation, Translation);
    }
};

// Tag component (empty)
UCLASS()
class UPlayerTag : public UActorComponent {
    GENERATED_BODY()
    // No data, just marker
};
```

---

#### C# (Unity Style)

```csharp
// Unity DOTS component (pure data)
using Unity.Entities;
using Unity.Mathematics;

public struct Health : IComponentData {
    public float Current;
    public float Max;
}

public struct TransformData : IComponentData {
    public float3 Translation;
    public quaternion Rotation;
    public float3 Scale;

    public float4x4 ToMatrix() {
        return float4x4.TRS(Translation, Rotation, Scale);
    }
}

// Tag component (zero size)
public struct PlayerTag : IComponentData { }

// MonoBehaviour component (OOP style, has logic)
public class HealthComponent : MonoBehaviour {
    public float current = 100f;
    public float max = 100f;

    public void TakeDamage(float amount) {
        current = Mathf.Max(0, current - amount);
    }

    void Update() {
        // Logic here
    }
}

// Shared component (all entities with same value share instance)
public struct RenderMesh : ISharedComponentData {
    public Mesh mesh;
    public Material material;
}
```

---

#### GDScript (Godot Style)

```gdscript
# Component as custom class
class_name HealthComponent extends Node

var current: float = 100.0
var max: float = 100.0

func take_damage(amount: float) -> void:
    current = max(0.0, current - amount)

# Or as resource (data-only)
class_name HealthData extends Resource

@export var current: float = 100.0
@export var max: float = 100.0

# Component as dictionary (dynamic)
var health_component = {
    "current": 100.0,
    "max": 100.0
}

# Tag as group membership
func _ready():
    add_to_group("player")
    add_to_group("damageable")
```

---

#### TypeScript (Web Engines)

```typescript
// bitECS component (struct-of-arrays)
import { defineComponent, Types } from 'bitecs';

const Health = defineComponent({
    current: Types.f32,
    max: Types.f32,
});

const Transform = defineComponent({
    posX: Types.f32,
    posY: Types.f32,
    posZ: Types.f32,
    rotX: Types.f32,
    rotY: Types.f32,
    rotZ: Types.f32,
    rotW: Types.f32,
    scaleX: Types.f32,
    scaleY: Types.f32,
    scaleZ: Types.f32,
});

// Tag component (empty object)
const PlayerTag = defineComponent();

// Object-oriented component (class)
class HealthComponent {
    current: number = 100;
    max: number = 100;

    takeDamage(amount: number): void {
        this.current = Math.max(0, this.current - amount);
    }
}
```

---

#### Python (Educational)

```python
from dataclasses import dataclass

# Component as dataclass
@dataclass
class Health:
    current: float
    max: float

    def take_damage(self, amount: float) -> None:
        self.current = max(0.0, self.current - amount)

@dataclass
class Transform:
    translation: tuple[float, float, float]
    rotation: tuple[float, float, float, float]
    scale: tuple[float, float, float]

# Tag component (singleton)
class PlayerTag:
    pass

# Or as type alias
PlayerTag = type('PlayerTag', (), {})
```

---

### System Implementation

How to write systems that process components.

---

#### Rust (Praxis)

```rust
use bevy_ecs::prelude::*;

// Simple system
fn damage_system(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        health.current -= 1.0;
    }
}

// System with multiple queries
fn collision_system(
    mut players: Query<&mut Health, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for mut player_health in players.iter_mut() {
        for enemy_transform in enemies.iter() {
            // Check collision, apply damage
        }
    }
}

// System with resources
fn movement_system(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.value * dt;
    }
}

// System with event reader
fn damage_event_system(
    mut events: EventReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for event in events.iter() {
        if let Ok(mut health) = query.get_mut(event.entity) {
            health.current -= event.amount;
        }
    }
}

// System with commands (spawn/despawn)
fn spawner_system(
    mut commands: Commands,
    time: Res<Time>,
) {
    if time.elapsed_seconds() > 5.0 {
        commands.spawn((
            Transform::default(),
            Health { current: 100.0, max: 100.0 },
            Enemy,
        ));
    }
}
```

---

#### C++ (Unreal Style)

```cpp
// System as subsystem Tick
UCLASS()
class UMyGameSubsystem : public UWorldSubsystem {
    GENERATED_BODY()

public:
    virtual void Tick(float DeltaTime) override {
        DamageSystem(DeltaTime);
        MovementSystem(DeltaTime);
    }

private:
    void DamageSystem(float DeltaTime) {
        for (TActorIterator<AActor> It(GetWorld()); It; ++It) {
            UHealthComponent* Health = It->FindComponentByClass<UHealthComponent>();
            if (Health) {
                Health->Current -= 1.0f * DeltaTime;
            }
        }
    }

    void MovementSystem(float DeltaTime) {
        for (TActorIterator<AActor> It(GetWorld()); It; ++It) {
            UMyTransformComponent* Transform = It->FindComponentByClass<UMyTransformComponent>();
            UVelocityComponent* Velocity = It->FindComponentByClass<UVelocityComponent>();

            if (Transform && Velocity) {
                Transform->Position += Velocity->Value * DeltaTime;
            }
        }
    }
};
```

---

#### C# (Unity DOTS)

```csharp
using Unity.Entities;
using Unity.Transforms;

public partial class DamageSystem : SystemBase {
    protected override void OnUpdate() {
        float deltaTime = Time.DeltaTime;

        Entities.ForEach((ref Health health) => {
            health.Current -= 1.0f * deltaTime;
        }).ScheduleParallel();
    }
}

public partial class MovementSystem : SystemBase {
    protected override void OnUpdate() {
        float deltaTime = Time.DeltaTime;

        Entities.ForEach((ref Translation translation, in Velocity velocity) => {
            translation.Value += velocity.Value * deltaTime;
        }).ScheduleParallel();
    }
}

// With EntityCommandBuffer for structural changes
public partial class SpawnerSystem : SystemBase {
    protected override void OnUpdate() {
        if (Time.ElapsedTime > 5.0) {
            var ecb = new EntityCommandBuffer(Allocator.Temp);

            ecb.CreateEntity();
            ecb.AddComponent(new Health { Current = 100, Max = 100 });
            ecb.AddComponent(new PlayerTag());

            ecb.Playback(EntityManager);
            ecb.Dispose();
        }
    }
}
```

---

#### GDScript (Godot Style)

```gdscript
# System as AutoLoad singleton
extends Node

func _process(delta):
    damage_system(delta)
    movement_system(delta)

func damage_system(delta: float) -> void:
    for entity in get_tree().get_nodes_in_group("damageable"):
        if entity.has_node("HealthComponent"):
            var health = entity.get_node("HealthComponent")
            health.current -= 1.0 * delta

func movement_system(delta: float) -> void:
    for entity in get_tree().get_nodes_in_group("movable"):
        if entity.has_method("get_velocity"):
            var velocity = entity.get_velocity()
            entity.position += velocity * delta
```

---

#### TypeScript (bitECS)

```typescript
import { defineQuery, enterQuery } from 'bitecs';

// System function
const damageSystem = (world) => {
    const query = defineQuery([Health]);
    const entities = query(world);

    for (let i = 0; i < entities.length; i++) {
        const eid = entities[i];
        Health.current[eid] -= 1.0 * deltaTime;
    }

    return world;
};

const movementSystem = (world) => {
    const query = defineQuery([Position, Velocity]);
    const entities = query(world);

    for (let i = 0; i < entities.length; i++) {
        const eid = entities[i];
        Position.x[eid] += Velocity.x[eid] * deltaTime;
        Position.y[eid] += Velocity.y[eid] * deltaTime;
        Position.z[eid] += Velocity.z[eid] * deltaTime;
    }

    return world;
};

// Enter/exit queries for spawning
const entitySpawnedSystem = (world) => {
    const query = enterQuery(defineQuery([Health]));
    const entities = query(world);

    for (let i = 0; i < entities.length; i++) {
        const eid = entities[i];
        console.log(`Entity ${eid} spawned with health`);
    }

    return world;
};
```

---

### Asset Loading

Loading meshes, textures, and other assets.

---

#### Rust (Praxis)

```rust
use praxis_assets::{MeshLoader, Mesh};

// Synchronous loading
let mesh = MeshLoader::load_obj("assets/models/cube.obj")?;
let mesh_handle = mesh_manager.add_mesh("cube", mesh)?;

// Asynchronous loading (future)
use tokio::task;

async fn load_asset_async(path: &str) -> Result<Mesh> {
    task::spawn_blocking(move || {
        MeshLoader::load_obj(path)
    }).await?
}

// Usage in async context
let mesh = load_asset_async("assets/models/cube.obj").await?;
```

---

#### C++ (Unreal Style)

```cpp
// Synchronous loading (blocks game thread - avoid!)
UStaticMesh* Mesh = LoadObject<UStaticMesh>(nullptr, TEXT("/Game/Models/Cube.Cube"));

// Asynchronous loading (preferred)
FStreamableManager& Streamable = UAssetManager::GetStreamableManager();
TSoftObjectPtr<UStaticMesh> AssetPtr(FSoftObjectPath(TEXT("/Game/Models/Cube.Cube")));

TSharedPtr<FStreamableHandle> Handle = Streamable.RequestAsyncLoad(
    AssetPtr.ToSoftObjectPath(),
    [this, AssetPtr]() {
        UStaticMesh* LoadedMesh = AssetPtr.Get();
        if (LoadedMesh) {
            // Asset loaded, use it
        }
    }
);

// Or with delegates
FStreamableDelegate Delegate = FStreamableDelegate::CreateUObject(this, &AMyActor::OnAssetLoaded);
Streamable.RequestAsyncLoad(AssetPtr.ToSoftObjectPath(), Delegate);
```

---

#### C# (Unity Style)

```csharp
using UnityEngine;

// Synchronous loading (blocks main thread)
Mesh mesh = Resources.Load<Mesh>("Models/Cube");

// Asynchronous loading with Resources
IEnumerator LoadAssetAsync() {
    ResourceRequest request = Resources.LoadAsync<Mesh>("Models/Cube");
    yield return request;

    Mesh mesh = request.asset as Mesh;
    // Use mesh
}

// Addressables (modern approach)
using UnityEngine.AddressableAssets;
using UnityEngine.ResourceManagement.AsyncOperations;

AsyncOperationHandle<Mesh> handle = Addressables.LoadAssetAsync<Mesh>("Assets/Models/Cube.fbx");
yield return handle;

if (handle.Status == AsyncOperationStatus.Succeeded) {
    Mesh mesh = handle.Result;
    // Use mesh
}

Addressables.Release(handle); // Release when done
```

---

#### GDScript (Godot Style)

```gdscript
# Synchronous loading (blocks)
var mesh = load("res://models/cube.obj")

# Preload (loaded at scene load time)
const CUBE_MESH = preload("res://models/cube.obj")

# Asynchronous loading with ResourceLoader
func load_async():
    ResourceLoader.load_threaded_request("res://models/cube.obj")

func _process(delta):
    var status = ResourceLoader.load_threaded_get_status("res://models/cube.obj")
    
    if status == ResourceLoader.THREAD_LOAD_LOADED:
        var mesh = ResourceLoader.load_threaded_get("res://models/cube.obj")
        # Use mesh
```

---

#### TypeScript (Three.js)

```typescript
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

// Asynchronous loading (callback)
const loader = new GLTFLoader();
loader.load(
    'models/cube.gltf',
    (gltf) => {
        // Success
        scene.add(gltf.scene);
    },
    (progress) => {
        // Progress callback
        console.log(`Loading: ${(progress.loaded / progress.total * 100)}%`);
    },
    (error) => {
        // Error callback
        console.error('Error loading model:', error);
    }
);

// Promise-based (async/await)
async function loadModel(url: string) {
    const loader = new GLTFLoader();
    return new Promise((resolve, reject) => {
        loader.load(url, resolve, undefined, reject);
    });
}

const gltf = await loadModel('models/cube.gltf');
scene.add(gltf.scene);

// Or with third-party promise wrapper
import { useGLTF } from '@react-three/drei'; // In React Three Fiber
const { scene } = useGLTF('models/cube.gltf');
```

---

### Error Handling

How to handle errors in each language.

---

#### Rust (Praxis)

```rust
// Result type for recoverable errors
fn load_mesh(path: &str) -> Result<Mesh, AssetError> {
    let data = std::fs::read(path)?; // ? propagates error
    let mesh = parse_obj(&data)?;
    Ok(mesh)
}

// Usage with pattern matching
match load_mesh("cube.obj") {
    Ok(mesh) => println!("Loaded!"),
    Err(e) => eprintln!("Error: {}", e),
}

// Usage with ? operator
let mesh = load_mesh("cube.obj")?; // Returns error if failed

// Panic for unrecoverable errors
assert!(mesh.vertices.len() > 0, "Mesh has no vertices");
mesh.vertices[100]; // Panic if out of bounds

// Custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
```

---

#### C++ (Unreal Style)

```cpp
// Return value + output parameter
bool LoadMesh(const FString& Path, FMesh& OutMesh) {
    if (!FPaths::FileExists(Path)) {
        UE_LOG(LogTemp, Error, TEXT("File not found: %s"), *Path);
        return false;
    }

    // Load mesh
    OutMesh = ParseOBJ(Path);
    return true;
}

// Usage
FMesh Mesh;
if (LoadMesh(TEXT("cube.obj"), Mesh)) {
    // Success
} else {
    // Failure
}

// Exceptions (rarely used in Unreal)
class FAssetException : public std::exception {
    virtual const char* what() const throw() {
        return "Asset loading failed";
    }
};

try {
    FMesh Mesh = LoadMeshOrThrow("cube.obj");
} catch (const FAssetException& e) {
    UE_LOG(LogTemp, Error, TEXT("Exception: %s"), ANSI_TO_TCHAR(e.what()));
}

// Check macros
check(Mesh.Vertices.Num() > 0); // Debug only, removed in shipping
checkf(Mesh.Vertices.Num() > 0, TEXT("Mesh has no vertices")); // With message
verify(LoadMesh(TEXT("cube.obj"), Mesh)); // Executes in shipping, asserts in debug
```

**Unreal conventions**:
- Return bool for success/failure
- Output parameters for results
- Logging with UE_LOG
- Assertions with check/checkf

---

#### C# (Unity Style)

```csharp
// Exceptions for errors
public Mesh LoadMesh(string path) {
    if (!File.Exists(path)) {
        throw new FileNotFoundException($"Mesh not found: {path}");
    }

    byte[] data = File.ReadAllBytes(path);
    Mesh mesh = ParseOBJ(data);
    return mesh;
}

// Usage with try-catch
try {
    Mesh mesh = LoadMesh("cube.obj");
    // Success
} catch (FileNotFoundException e) {
    Debug.LogError($"Error: {e.Message}");
} catch (Exception e) {
    Debug.LogError($"Unexpected error: {e}");
}

// Nullable return (C# 8.0+)
public Mesh? LoadMeshSafe(string path) {
    if (!File.Exists(path)) {
        return null;
    }

    return ParseOBJ(File.ReadAllBytes(path));
}

// Usage
Mesh? mesh = LoadMeshSafe("cube.obj");
if (mesh != null) {
    // Use mesh
}

// Unity assertions
Debug.Assert(mesh.vertexCount > 0, "Mesh has no vertices");

// Result pattern (custom)
public struct Result<T, E> {
    public bool IsOk;
    public T Value;
    public E Error;

    public static Result<T, E> Ok(T value) => new Result<T, E> { IsOk = true, Value = value };
    public static Result<T, E> Err(E error) => new Result<T, E> { IsOk = false, Error = error };
}

Result<Mesh, string> result = LoadMeshResult("cube.obj");
if (result.IsOk) {
    Mesh mesh = result.Value;
}
```

---

#### GDScript (Godot Style)

```gdscript
# Return null or error code
func load_mesh(path: String) -> Mesh:
    if not FileAccess.file_exists(path):
        push_error("File not found: " + path)
        return null
    
    var file = FileAccess.open(path, FileAccess.READ)
    if file == null:
        push_error("Failed to open file: " + path)
        return null
    
    var data = file.get_buffer(file.get_length())
    file.close()
    
    return parse_obj(data)

# Usage
var mesh = load_mesh("cube.obj")
if mesh == null:
    print("Failed to load mesh")
else:
    print("Mesh loaded successfully")

# Assert for debugging
assert(mesh.get_surface_count() > 0, "Mesh has no surfaces")

# Error code pattern
enum LoadError {
    OK,
    FILE_NOT_FOUND,
    PARSE_ERROR,
}

func load_mesh_with_error(path: String) -> Array: # [Mesh, LoadError]
    if not FileAccess.file_exists(path):
        return [null, LoadError.FILE_NOT_FOUND]
    
    var mesh = parse_obj_internal(path)
    if mesh == null:
        return [null, LoadError.PARSE_ERROR]
    
    return [mesh, LoadError.OK]

# Usage
var result = load_mesh_with_error("cube.obj")
var mesh = result[0]
var error = result[1]

if error != LoadError.OK:
    print("Load error: ", error)
```

---

#### TypeScript (Web Engines)

```typescript
// Exceptions (try-catch)
function loadMesh(path: string): Mesh {
    if (!fs.existsSync(path)) {
        throw new Error(`File not found: ${path}`);
    }

    const data = fs.readFileSync(path);
    return parseOBJ(data);
}

// Usage
try {
    const mesh = loadMesh('cube.obj');
    // Success
} catch (error) {
    console.error('Error:', error);
}

// Promise rejection (async)
async function loadMeshAsync(path: string): Promise<Mesh> {
    const response = await fetch(path);
    if (!response.ok) {
        throw new Error(`Failed to load: ${response.statusText}`);
    }

    const data = await response.arrayBuffer();
    return parseOBJ(new Uint8Array(data));
}

// Usage with try-catch
try {
    const mesh = await loadMeshAsync('cube.obj');
} catch (error) {
    console.error('Error:', error);
}

// Result pattern (custom)
type Result<T, E> = 
    | { ok: true; value: T }
    | { ok: false; error: E };

function loadMeshSafe(path: string): Result<Mesh, string> {
    if (!fs.existsSync(path)) {
        return { ok: false, error: `File not found: ${path}` };
    }

    try {
        const mesh = parseOBJ(fs.readFileSync(path));
        return { ok: true, value: mesh };
    } catch (error) {
        return { ok: false, error: String(error) };
    }
}

// Usage with type guard
const result = loadMeshSafe('cube.obj');
if (result.ok) {
    const mesh = result.value; // Type: Mesh
} else {
    console.error(result.error); // Type: string
}

// Assertions
console.assert(mesh.vertices.length > 0, 'Mesh has no vertices');
```

---

## Common Patterns

### Singleton Pattern

**Rust (Resource)**:
```rust
#[derive(Resource)]
struct GameState {
    score: i32,
}

world.insert_resource(GameState { score: 0 });

fn scoring_system(mut state: ResMut<GameState>) {
    state.score += 10;
}
```

**C++ (Unreal Subsystem)**:
```cpp
UCLASS()
class UGameStateSubsystem : public UGameInstanceSubsystem {
    GENERATED_BODY()
public:
    int32 Score = 0;
};

// Access
UGameStateSubsystem* GameState = GetGameInstance()->GetSubsystem<UGameStateSubsystem>();
GameState->Score += 10;
```

**C# (Unity Singleton)**:
```csharp
public class GameState : MonoBehaviour {
    public static GameState Instance { get; private set; }

    public int score = 0;

    void Awake() {
        if (Instance == null) {
            Instance = this;
            DontDestroyOnLoad(gameObject);
        } else {
            Destroy(gameObject);
        }
    }
}

// Access
GameState.Instance.score += 10;
```

**GDScript (Autoload)**:
```gdscript
# GameState.gd (added to AutoLoad in Project Settings)
extends Node

var score: int = 0

# Access from any script
GameState.score += 10
```

---

### Observer Pattern (Events)

**Rust (Events)**:
```rust
#[derive(Event)]
struct DamageEvent {
    entity: Entity,
    amount: f32,
}

fn damage_sender(mut events: EventWriter<DamageEvent>) {
    events.send(DamageEvent { entity, amount: 10.0 });
}

fn damage_receiver(mut events: EventReader<DamageEvent>) {
    for event in events.iter() {
        println!("Damage: {}", event.amount);
    }
}
```

**C++ (Unreal Delegates)**:
```cpp
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FDamageEvent, AActor*, Actor, float, Amount);

UCLASS()
class AMyActor : public AActor {
    GENERATED_BODY()
public:
    UPROPERTY(BlueprintAssignable)
    FDamageEvent OnDamage;

    void TakeDamage(float Amount) {
        OnDamage.Broadcast(this, Amount);
    }
};

// Subscribe
Actor->OnDamage.AddDynamic(this, &AMyListener::HandleDamage);

void AMyListener::HandleDamage(AActor* Actor, float Amount) {
    UE_LOG(LogTemp, Log, TEXT("Damage: %f"), Amount);
}
```

**C# (Unity Events)**:
```csharp
using UnityEngine.Events;

[System.Serializable]
public class DamageEvent : UnityEvent<float> { }

public class Health : MonoBehaviour {
    public DamageEvent onDamage = new DamageEvent();

    public void TakeDamage(float amount) {
        onDamage.Invoke(amount);
    }
}

// Subscribe
health.onDamage.AddListener((amount) => {
    Debug.Log($"Damage: {amount}");
});
```

**GDScript (Signals)**:
```gdscript
signal damage_taken(amount)

func take_damage(amount: float) -> void:
    damage_taken.emit(amount)

# Subscribe
health_component.damage_taken.connect(_on_damage_taken)

func _on_damage_taken(amount: float) -> void:
    print("Damage: ", amount)
```

---

## Pitfalls and Solutions

### Null/None Checks

**Rust**: Uses `Option<T>`, compile-time enforced
```rust
let mesh: Option<Mesh> = mesh_manager.get("cube");
match mesh {
    Some(m) => render(m),
    None => println!("Mesh not found"),
}

// Or with if let
if let Some(m) = mesh {
    render(m);
}
```

**Other languages**: Require runtime checks
```cpp
// C++: Can be null, check at runtime
Mesh* mesh = GetMesh("cube");
if (mesh != nullptr) {
    Render(mesh);
}

// C#: Nullable reference types help but not enforced
Mesh? mesh = GetMesh("cube");
if (mesh != null) {
    Render(mesh);
}

// GDScript: Everything can be null
var mesh = get_mesh("cube")
if mesh != null:
    render(mesh)
```

**Solution**: Always check for null in languages without compile-time null safety

---

### Mutable Aliasing

**Rust**: Prevented at compile time
```rust
let mut x = 5;
let r1 = &mut x;
// let r2 = &mut x; // ERROR: cannot borrow as mutable more than once
*r1 += 1;
```

**Other languages**: Possible, requires discipline
```cpp
int x = 5;
int* r1 = &x;
int* r2 = &x; // Both can modify x
*r1 += 1;
*r2 += 1; // x is now 7
```

**Solution**: 
- Minimize mutable state
- Document ownership clearly
- Use const/readonly where possible

---

### Async Complexity

**Rust**: Explicit async/await
```rust
async fn load_asset(path: &str) -> Result<Mesh> {
    let data = tokio::fs::read(path).await?;
    Ok(parse_obj(&data))
}

// Must await
let mesh = load_asset("cube.obj").await?;
```

**JavaScript/TypeScript**: Implicit async
```typescript
async function loadAsset(path: string): Promise<Mesh> {
    const data = await fs.promises.readFile(path);
    return parseOBJ(data);
}

// Can forget to await (compiles but wrong!)
const mesh = loadAsset("cube.obj"); // mesh is Promise<Mesh>, not Mesh!
const mesh = await loadAsset("cube.obj"); // Correct
```

**Solution**: TypeScript ESLint rule "no-floating-promises"

---

## See Also

- **[Glossary](glossary.md)** - Cross-engine terminology reference
- **[Curriculum](CURRICULUM.md)** - Language-agnostic architecture course
- **[Code Examples](CODE_EXAMPLES.md)** - Side-by-side Rust/C++/C# implementations
- **[Universal Patterns](patterns/)** - Design patterns across engines
