# Concept-to-Code Mapping Guide

This guide demonstrates how abstract game engine concepts translate to concrete code implementations across different languages and engine architectures. Each example shows the same algorithm implemented in multiple styles to help you understand universal patterns and language-specific idioms.

## How to Read This Guide

Each section presents an algorithm in four forms:

1. **Pseudocode** - Abstract, language-agnostic description of the algorithm
2. **Rust (Praxis)** - ECS-based implementation using bevy_ecs
3. **C++ (Unreal-style)** - Object-oriented implementation with UObject hierarchy
4. **C# (Unity-style)** - Component-based implementation with GameObject/MonoBehaviour

### MkDocs Content Tabs (Optional Enhancement)

This document is structured to work with MkDocs Material's [content tabs feature](https://squidfunk.github.io/mkdocs-material/reference/content-tabs/). If you're using MkDocs Material, the code examples will appear as interactive tabs:

```markdown
=== "Pseudocode"
    Language-agnostic algorithm description
=== "Rust (Praxis)"
    ECS implementation with bevy_ecs
=== "C++ (Unreal)"
    Object-oriented approach
=== "C# (Unity)"
    Component-based approach
```

**Without MkDocs**: All implementations appear sequentially (works perfectly as plain markdown).  
**With MkDocs Material**: Implementations appear as clickable tabs for easy comparison.

To enable content tabs in MkDocs, add to your `mkdocs.yml`:
```yaml
markdown_extensions:
  - pymdownx.superfences
  - pymdownx.tabbed:
      alternate_style: true
```

---

## Transform Propagation

Transform propagation updates world-space transforms based on local transforms and parent-child hierarchies. This is fundamental to scene graphs in every game engine.

### Algorithm Overview

Transform propagation ensures that when a parent object moves, rotates, or scales, all its children update accordingly. The core algorithm:

1. Identify root entities (no parent)
2. Update each root's global transform from its local transform
3. For each root's children, recursively compute: `child_global = parent_global * child_local`
4. Handle change detection to avoid redundant updates

### Pseudocode

```
FUNCTION propagate_transforms:
    // Phase 1: Update changed roots
    FOR EACH root entity WITH changed local_transform:
        root.global_transform = root.local_transform.to_matrix()
        propagate_to_children(root, root.global_transform)
    
    // Phase 2: Update reparented entities
    FOR EACH entity WITH changed parent:
        parent_global = entity.parent.global_transform
        entity.global_transform = parent_global * entity.local_transform.to_matrix()
        propagate_to_children(entity, entity.global_transform)

FUNCTION propagate_to_children(entity, parent_matrix):
    work_queue = [(child, parent_matrix) FOR child IN entity.children]
    
    WHILE work_queue NOT empty:
        (current, parent_matrix) = work_queue.pop()
        current.global_transform = parent_matrix * current.local_transform.to_matrix()
        
        FOR EACH child IN current.children:
            work_queue.append((child, current.global_transform))
```

### Rust (Praxis)

The Praxis implementation uses an ECS approach with change detection and archetype storage:

```rust
use bevy_ecs::prelude::*;
use praxis_math::{Mat4, Quat, Vec3};

// Components
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn compute_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

#[derive(Component)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);

// System implementation
pub fn propagate_transforms(
    mut root_query: Query<
        (Entity, &Transform, &mut GlobalTransform, Option<&Children>),
        (Without<Parent>, Changed<Transform>)
    >,
    mut child_query: Query<
        (&Transform, &mut GlobalTransform, Option<&Children>),
        With<Parent>
    >,
) {
    // Update changed roots
    for (entity, transform, mut global_transform, children) in root_query.iter_mut() {
        global_transform.matrix = transform.compute_matrix();
        
        if let Some(children) = children {
            propagate_recursive(&children.0, &global_transform.matrix, &mut child_query);
        }
    }
}

fn propagate_recursive(
    children: &[Entity],
    parent_matrix: &Mat4,
    child_query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>), With<Parent>>,
) {
    let mut work_queue: Vec<(Entity, Mat4)> = children
        .iter()
        .map(|&child| (child, *parent_matrix))
        .collect();
    
    while let Some((entity, parent_matrix)) = work_queue.pop() {
        if let Ok((transform, mut global_transform, maybe_children)) = child_query.get_mut(entity) {
            let child_matrix = parent_matrix * transform.compute_matrix();
            global_transform.matrix = child_matrix;
            
            if let Some(children) = maybe_children {
                for &child in children.0.iter() {
                    work_queue.push((child, child_matrix));
                }
            }
        }
    }
}
```

**Key Rust/ECS Patterns**:
- `Query<T, Filter>` enables efficient component iteration over matching archetypes
- `Changed<T>` filter automatically detects modified components
- `With<T>` and `Without<T>` filter entities by component presence
- Ownership system ensures no data races (mutable reference to `GlobalTransform`, immutable to `Transform`)
- Work queue avoids recursion and borrow checker issues

### C++ (Unreal-style)

Unreal Engine uses an object-oriented hierarchy with `USceneComponent` as the base:

```cpp
// SceneComponent.h
class USceneComponent : public UObject {
protected:
    FTransform RelativeTransform;  // Local transform
    FTransform ComponentToWorld;   // Global/world transform (cached)
    USceneComponent* AttachParent; // Pointer to parent
    TArray<USceneComponent*> AttachChildren; // Array of children
    bool bDirtyTransform;          // Dirty flag for optimization

public:
    void UpdateComponentToWorld();
    void PropagateTransformUpdate(bool bTransformChanged);
    
    // Getters/setters
    void SetRelativeTransform(const FTransform& NewTransform);
    FTransform GetComponentToWorld() const { return ComponentToWorld; }
};

// SceneComponent.cpp
void USceneComponent::UpdateComponentToWorld() {
    if (AttachParent != nullptr) {
        // Child: combine with parent's world transform
        ComponentToWorld = RelativeTransform * AttachParent->GetComponentToWorld();
    } else {
        // Root: local transform IS world transform
        ComponentToWorld = RelativeTransform;
    }
    bDirtyTransform = false;
}

void USceneComponent::PropagateTransformUpdate(bool bTransformChanged) {
    if (bTransformChanged || bDirtyTransform) {
        UpdateComponentToWorld();
        
        // Recursively update all children
        for (USceneComponent* Child : AttachChildren) {
            if (Child != nullptr) {
                Child->PropagateTransformUpdate(true);
            }
        }
    }
}

void USceneComponent::SetRelativeTransform(const FTransform& NewTransform) {
    RelativeTransform = NewTransform;
    bDirtyTransform = true;
    PropagateTransformUpdate(true);
}
```

**Key C++/Unreal Patterns**:
- Object-oriented hierarchy with pointers to parent/children
- Dirty flags (`bDirtyTransform`) track when updates are needed
- Virtual functions allow polymorphic behavior in derived classes
- Manual memory management through `UObject` garbage collection
- Direct recursion is acceptable (no borrow checker concerns)
- `FTransform` multiplication operator handles matrix math

### C# (Unity-style)

Unity uses a component-based system with `Transform` components attached to `GameObject`:

```csharp
// Transform.cs (simplified Unity-style implementation)
public class Transform : Component {
    // Local space
    public Vector3 localPosition;
    public Quaternion localRotation;
    public Vector3 localScale = Vector3.one;
    
    // World space (cached)
    private Matrix4x4 _localToWorldMatrix;
    private bool _isDirty = true;
    
    // Hierarchy
    private Transform _parent;
    private List<Transform> _children = new List<Transform>();
    
    // Public world-space accessors
    public Vector3 position {
        get => localToWorldMatrix.GetPosition();
        set {
            if (_parent != null) {
                localPosition = _parent.worldToLocalMatrix.MultiplyPoint(value);
            } else {
                localPosition = value;
            }
            MarkDirty();
        }
    }
    
    public Matrix4x4 localToWorldMatrix {
        get {
            if (_isDirty) {
                UpdateLocalToWorldMatrix();
            }
            return _localToWorldMatrix;
        }
    }
    
    private void UpdateLocalToWorldMatrix() {
        Matrix4x4 localMatrix = Matrix4x4.TRS(localPosition, localRotation, localScale);
        
        if (_parent != null) {
            _localToWorldMatrix = _parent.localToWorldMatrix * localMatrix;
        } else {
            _localToWorldMatrix = localMatrix;
        }
        
        _isDirty = false;
    }
    
    private void MarkDirty() {
        _isDirty = true;
        
        // Propagate dirty flag to all children
        foreach (Transform child in _children) {
            child.MarkDirty();
        }
    }
    
    public void SetParent(Transform newParent) {
        // Remove from old parent
        if (_parent != null) {
            _parent._children.Remove(this);
        }
        
        _parent = newParent;
        
        // Add to new parent
        if (_parent != null) {
            _parent._children.Add(this);
        }
        
        MarkDirty();
    }
}

// Unity's internal transform update system
class TransformSystem {
    public static void UpdateTransforms() {
        // Unity internally batches transform updates
        // This is a simplified version
        foreach (Transform root in FindRootTransforms()) {
            if (root._isDirty) {
                UpdateTransformHierarchy(root);
            }
        }
    }
    
    private static void UpdateTransformHierarchy(Transform transform) {
        transform.UpdateLocalToWorldMatrix();
        
        foreach (Transform child in transform._children) {
            UpdateTransformHierarchy(child);
        }
    }
}
```

**Key C#/Unity Patterns**:
- Component attached to `GameObject`, managed by Unity's internal systems
- Lazy evaluation with dirty flags (computed on access)
- Properties (`position`, `localToWorldMatrix`) provide clean API
- Automatic garbage collection simplifies memory management
- Internal batching system updates all transforms at specific frame points
- `Matrix4x4.TRS()` helper for transform composition

### Comparison Table

| Aspect | Rust (Praxis) | C++ (Unreal) | C# (Unity) |
|--------|---------------|---------------|-------------|
| **Data Layout** | Separate component arrays (archetype storage) | Objects with embedded data | Components on GameObjects |
| **Ownership** | Compile-time borrow checker | Raw pointers with GC | Managed references with GC |
| **Change Detection** | Automatic ECS tracking | Manual dirty flags | Manual dirty flags + lazy eval |
| **Hierarchy Storage** | Entity IDs in components | Pointer to parent/children | Reference to parent/children |
| **Iteration** | Query-based (cache-friendly) | Pointer traversal | Collection traversal |
| **Update Strategy** | Push (immediate propagation) | Push (immediate propagation) | Pull (lazy on access) |
| **Parallelization** | Easy (ECS queries are thread-safe by design) | Difficult (requires careful locking) | Difficult (Unity handles internally) |

---

## Spatial Culling (Frustum Culling)

Frustum culling eliminates objects outside the camera's view volume before rendering, significantly improving performance.

### Algorithm Overview

The camera's view frustum is defined by six planes (near, far, left, right, top, bottom). For each renderable object:

1. Extract the object's bounding volume (sphere or AABB)
2. Test the bounding volume against each frustum plane
3. If entirely outside any plane, cull the object
4. Otherwise, mark for rendering

### Pseudocode

```
STRUCTURE Frustum:
    planes: [Plane; 6]  // near, far, left, right, top, bottom

STRUCTURE Plane:
    normal: Vec3
    distance: float

STRUCTURE BoundingSphere:
    center: Vec3
    radius: float

FUNCTION is_visible(frustum, bounding_sphere):
    FOR EACH plane IN frustum.planes:
        // Distance from plane to sphere center
        distance = dot(plane.normal, bounding_sphere.center) + plane.distance
        
        // If sphere is entirely on negative side of plane, it's outside
        IF distance < -bounding_sphere.radius:
            RETURN false  // Culled
    
    RETURN true  // Visible or partially visible

FUNCTION cull_objects(camera, objects):
    frustum = extract_frustum(camera)
    visible_objects = []
    
    FOR EACH object IN objects:
        bounds = object.bounding_sphere
        world_center = object.global_transform * bounds.center
        world_bounds = BoundingSphere(world_center, bounds.radius * object.global_scale)
        
        IF is_visible(frustum, world_bounds):
            visible_objects.append(object)
    
    RETURN visible_objects
```

### Rust (Praxis)

```rust
use bevy_ecs::prelude::*;
use praxis_math::{Mat4, Vec3, Vec4};

// Components
#[derive(Component)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Component)]
pub struct Renderable;

#[derive(Component)]
pub struct Visible;

// Frustum structure
pub struct Frustum {
    planes: [Plane; 6],
}

pub struct Plane {
    normal: Vec3,
    distance: f32,
}

impl Frustum {
    pub fn from_view_projection(view_proj: &Mat4) -> Self {
        let planes = [
            // Left plane: row4 + row1
            Self::normalize_plane(view_proj.row(3) + view_proj.row(0)),
            // Right plane: row4 - row1
            Self::normalize_plane(view_proj.row(3) - view_proj.row(0)),
            // Bottom plane: row4 + row2
            Self::normalize_plane(view_proj.row(3) + view_proj.row(1)),
            // Top plane: row4 - row2
            Self::normalize_plane(view_proj.row(3) - view_proj.row(1)),
            // Near plane: row4 + row3
            Self::normalize_plane(view_proj.row(3) + view_proj.row(2)),
            // Far plane: row4 - row3
            Self::normalize_plane(view_proj.row(3) - view_proj.row(2)),
        ];
        
        Self { planes }
    }
    
    fn normalize_plane(plane: Vec4) -> Plane {
        let normal = Vec3::new(plane.x, plane.y, plane.z);
        let length = normal.length();
        Plane {
            normal: normal / length,
            distance: plane.w / length,
        }
    }
    
    pub fn is_visible(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.normal.dot(center) + plane.distance;
            if distance < -radius {
                return false; // Completely outside this plane
            }
        }
        true // Visible or partially visible
    }
}

// System that performs frustum culling
pub fn frustum_culling_system(
    mut commands: Commands,
    camera_query: Query<&CameraMatrices, With<Camera>>,
    mut objects: Query<
        (Entity, &GlobalTransform, &BoundingSphere),
        With<Renderable>
    >,
) {
    // Get active camera's view-projection matrix
    let camera_matrices = match camera_query.get_single() {
        Ok(matrices) => matrices,
        Err(_) => return, // No camera or multiple cameras
    };
    
    let frustum = Frustum::from_view_projection(&camera_matrices.view_projection);
    
    // Test each renderable object
    for (entity, global_transform, bounding_sphere) in objects.iter_mut() {
        let world_center = global_transform.transform_point(bounding_sphere.center);
        let world_radius = bounding_sphere.radius * global_transform.max_scale();
        
        if frustum.is_visible(world_center, world_radius) {
            // Add Visible component for rendering
            commands.entity(entity).insert(Visible);
        } else {
            // Remove Visible component (culled)
            commands.entity(entity).remove::<Visible>();
        }
    }
}
```

**Key Rust/ECS Patterns**:
- Frustum extracted from camera's view-projection matrix
- Query iterates only entities with required components (`Renderable`, `GlobalTransform`, `BoundingSphere`)
- Visibility tracked via marker component (`Visible`) added/removed dynamically
- Rendering system queries `With<Visible>` to skip culled objects
- No allocations in hot path (work directly with query iterators)

### C++ (Unreal-style)

```cpp
// FrustumCulling.h
struct FPlane {
    FVector Normal;
    float Distance;
    
    FPlane() : Normal(FVector::ZeroVector), Distance(0.0f) {}
    FPlane(const FVector& InNormal, float InDistance) 
        : Normal(InNormal), Distance(InDistance) {}
    
    float DistanceToPoint(const FVector& Point) const {
        return Normal | Point + Distance;  // Dot product operator
    }
};

struct FFrustum {
    FPlane Planes[6]; // Near, Far, Left, Right, Top, Bottom
    
    static FFrustum FromViewProjection(const FMatrix& ViewProjectionMatrix);
    bool IsVisible(const FSphere& BoundingSphere) const;
};

// FrustumCulling.cpp
FFrustum FFrustum::FromViewProjection(const FMatrix& ViewProj) {
    FFrustum Result;
    
    // Extract frustum planes from view-projection matrix
    // Left plane: row4 + row1
    Result.Planes[0] = NormalizePlane(
        FPlane(
            FVector(ViewProj.M[0][3] + ViewProj.M[0][0],
                    ViewProj.M[1][3] + ViewProj.M[1][0],
                    ViewProj.M[2][3] + ViewProj.M[2][0]),
            ViewProj.M[3][3] + ViewProj.M[3][0]
        )
    );
    
    // Right plane: row4 - row1
    Result.Planes[1] = NormalizePlane(
        FPlane(
            FVector(ViewProj.M[0][3] - ViewProj.M[0][0],
                    ViewProj.M[1][3] - ViewProj.M[1][0],
                    ViewProj.M[2][3] - ViewProj.M[2][0]),
            ViewProj.M[3][3] - ViewProj.M[3][0]
        )
    );
    
    // ... similar for top, bottom, near, far planes
    
    return Result;
}

FPlane FFrustum::NormalizePlane(const FPlane& Plane) {
    float Length = Plane.Normal.Size();
    return FPlane(Plane.Normal / Length, Plane.Distance / Length);
}

bool FFrustum::IsVisible(const FSphere& BoundingSphere) const {
    for (int32 i = 0; i < 6; ++i) {
        float Distance = Planes[i].DistanceToPoint(BoundingSphere.Center);
        if (Distance < -BoundingSphere.W) {  // W stores radius
            return false; // Outside this plane
        }
    }
    return true;
}

// Scene component with culling
class UPrimitiveComponent : public USceneComponent {
protected:
    FSphere Bounds; // Local space bounding sphere
    bool bIsVisible;

public:
    virtual void UpdateVisibility(const FFrustum& CameraFrustum) {
        // Transform bounds to world space
        FSphere WorldBounds = Bounds.TransformBy(GetComponentToWorld());
        bIsVisible = CameraFrustum.IsVisible(WorldBounds);
    }
    
    bool IsVisible() const { return bIsVisible; }
};

// Renderer performs culling
class URenderer {
public:
    void CullAndRender(UWorld* World, UCameraComponent* Camera) {
        FMatrix ViewProjection = Camera->GetViewProjectionMatrix();
        FFrustum Frustum = FFrustum::FromViewProjection(ViewProjection);
        
        // Iterate all primitive components in the world
        TArray<UPrimitiveComponent*> Primitives;
        GetAllPrimitives(World, Primitives);
        
        for (UPrimitiveComponent* Primitive : Primitives) {
            Primitive->UpdateVisibility(Frustum);
            
            if (Primitive->IsVisible()) {
                RenderPrimitive(Primitive);
            }
        }
    }
};
```

**Key C++/Unreal Patterns**:
- Frustum stored as struct with six planes
- Virtual function `UpdateVisibility` allows per-component customization
- Direct iteration over array of component pointers
- Visibility flag cached in component
- Unreal's custom types (`FVector`, `FMatrix`, `FSphere`) with operator overloads
- Manual iteration over world's primitive components

### C# (Unity-style)

```csharp
// Frustum culling in Unity
using UnityEngine;
using System.Collections.Generic;

public struct Frustum {
    private Plane[] planes; // Unity's built-in Plane struct
    
    public static Frustum FromCamera(Camera camera) {
        Frustum frustum = new Frustum();
        frustum.planes = GeometryUtility.CalculateFrustumPlanes(camera);
        return frustum;
    }
    
    public bool IsVisible(Bounds bounds) {
        // Unity's built-in method
        return GeometryUtility.TestPlanesAABB(planes, bounds);
    }
}

// Component that performs culling
public class FrustumCuller : MonoBehaviour {
    public Camera mainCamera;
    private List<Renderer> allRenderers = new List<Renderer>();
    
    void Start() {
        // Cache all renderers in scene
        allRenderers.AddRange(FindObjectsOfType<Renderer>());
    }
    
    void Update() {
        if (mainCamera == null) return;
        
        Frustum frustum = Frustum.FromCamera(mainCamera);
        
        foreach (Renderer renderer in allRenderers) {
            // Get world-space bounds
            Bounds bounds = renderer.bounds;
            
            // Test visibility
            bool isVisible = frustum.IsVisible(bounds);
            
            // Enable/disable renderer
            renderer.enabled = isVisible;
        }
    }
}

// Alternative: Unity's built-in culling
public class UnityBuiltInCulling : MonoBehaviour {
    // Unity automatically culls objects outside camera frustum
    // Renderers are only processed if visible to at least one camera
    
    void OnBecameVisible() {
        // Called when renderer enters camera frustum
        Debug.Log($"{gameObject.name} became visible");
    }
    
    void OnBecameInvisible() {
        // Called when renderer exits camera frustum
        Debug.Log($"{gameObject.name} became invisible");
    }
}

// Custom culling with Renderer.isVisible
public class CustomCullingLogic : MonoBehaviour {
    private Renderer myRenderer;
    
    void Start() {
        myRenderer = GetComponent<Renderer>();
    }
    
    void Update() {
        // Unity automatically maintains Renderer.isVisible
        if (myRenderer.isVisible) {
            // This object is visible to at least one camera
            // Perform visibility-dependent logic (LOD updates, etc.)
        }
    }
}

// Manual frustum plane extraction
public static class FrustumUtility {
    public static Plane[] ExtractFrustumPlanes(Matrix4x4 viewProj) {
        Plane[] planes = new Plane[6];
        
        // Left plane
        planes[0] = new Plane(
            new Vector3(viewProj.m30 + viewProj.m00,
                       viewProj.m31 + viewProj.m01,
                       viewProj.m32 + viewProj.m02),
            viewProj.m33 + viewProj.m03
        );
        
        // Right plane
        planes[1] = new Plane(
            new Vector3(viewProj.m30 - viewProj.m00,
                       viewProj.m31 - viewProj.m01,
                       viewProj.m32 - viewProj.m02),
            viewProj.m33 - viewProj.m03
        );
        
        // Bottom, top, near, far planes...
        // (similar extraction)
        
        // Normalize all planes
        for (int i = 0; i < 6; i++) {
            planes[i] = NormalizePlane(planes[i]);
        }
        
        return planes;
    }
    
    private static Plane NormalizePlane(Plane plane) {
        float length = plane.normal.magnitude;
        return new Plane(plane.normal / length, plane.distance / length);
    }
}
```

**Key C#/Unity Patterns**:
- Unity provides built-in `GeometryUtility.CalculateFrustumPlanes(Camera)`
- Built-in `Renderer.isVisible` automatically maintained by Unity
- `Renderer.bounds` provides world-space AABB
- Callbacks `OnBecameVisible`/`OnBecameInvisible` for event-driven logic
- Manual control via `Renderer.enabled` for custom culling
- Unity's internal culling happens automatically; manual culling for special cases

### Comparison Table

| Aspect | Rust (Praxis) | C++ (Unreal) | C# (Unity) |
|--------|---------------|---------------|-------------|
| **Frustum Storage** | Struct with plane array | Struct with plane array | Built-in `GeometryUtility` or manual |
| **Visibility Tracking** | Marker component (`Visible`) | Boolean flag in component | `Renderer.enabled` or `isVisible` |
| **Bounds Representation** | `BoundingSphere` component | `FSphere` member variable | `Bounds` from `Renderer` |
| **Culling Trigger** | System runs each frame | Manual call in render loop | Automatic + manual override |
| **Parallel Potential** | High (query-based iteration) | Low (requires locking) | Low (Unity manages internally) |
| **Integration** | Explicit system in schedule | Virtual method override | Component callbacks + flags |

---

## Fixed Timestep Physics Update

Physics simulations require consistent timestep for stability and determinism, regardless of frame rate.

### Algorithm Overview

The fixed timestep accumulator pattern decouples physics updates from rendering:

1. Measure elapsed time since last frame (`delta_time`)
2. Accumulate time in a buffer
3. While accumulated time ≥ fixed step, run physics update and subtract step
4. Remaining time carries over to next frame

This ensures physics runs at constant rate (e.g., 60 Hz) even if rendering is 144 Hz or 30 Hz.

### Pseudocode

```
CONSTANT FIXED_TIMESTEP = 1.0 / 60.0  // 60 Hz physics

GLOBAL accumulator = 0.0

FUNCTION game_loop:
    last_time = current_time()
    
    LOOP:
        current_time = current_time()
        delta_time = current_time - last_time
        last_time = current_time
        
        // Clamp delta_time to prevent spiral of death
        IF delta_time > 0.25:
            delta_time = 0.25  // Max 4 missed frames
        
        accumulator += delta_time
        
        // Fixed timestep updates
        WHILE accumulator >= FIXED_TIMESTEP:
            update_physics(FIXED_TIMESTEP)
            accumulator -= FIXED_TIMESTEP
        
        // Variable timestep rendering
        render(delta_time)

FUNCTION update_physics(dt):
    // Apply forces
    FOR EACH rigidbody:
        rigidbody.velocity += rigidbody.force * dt
        rigidbody.force = Vec3.ZERO
    
    // Integrate positions
    FOR EACH rigidbody:
        rigidbody.position += rigidbody.velocity * dt
    
    // Resolve collisions
    detect_and_resolve_collisions()
```

### Rust (Praxis)

```rust
use bevy_ecs::prelude::*;
use praxis_math::Vec3;
use std::time::{Duration, Instant};

// Physics configuration resource
#[derive(Resource)]
pub struct PhysicsConfig {
    pub timestep: f32,        // Fixed timestep (e.g., 1/60)
    pub max_substeps: u32,    // Prevent spiral of death
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            timestep: 1.0 / 60.0,  // 60 Hz
            max_substeps: 4,        // Max 4 physics steps per frame
        }
    }
}

// Time accumulator resource
#[derive(Resource)]
pub struct PhysicsAccumulator {
    accumulator: f32,
}

impl Default for PhysicsAccumulator {
    fn default() -> Self {
        Self { accumulator: 0.0 }
    }
}

// Physics components
#[derive(Component)]
pub struct RigidBody {
    pub velocity: Vec3,
    pub force: Vec3,
    pub mass: f32,
}

#[derive(Component)]
pub struct Position(pub Vec3);

// Main physics system
pub fn physics_system(
    time: Res<Time>,
    config: Res<PhysicsConfig>,
    mut accumulator: ResMut<PhysicsAccumulator>,
    mut rigidbodies: Query<(&mut RigidBody, &mut Position)>,
) {
    // Add frame time to accumulator
    let delta_time = time.delta_seconds().min(0.25); // Clamp to 250ms
    accumulator.accumulator += delta_time;
    
    // Run fixed timestep updates
    let mut substeps = 0;
    while accumulator.accumulator >= config.timestep {
        if substeps >= config.max_substeps {
            // Spiral of death prevention: discard remaining time
            accumulator.accumulator = 0.0;
            break;
        }
        
        physics_step(config.timestep, &mut rigidbodies);
        accumulator.accumulator -= config.timestep;
        substeps += 1;
    }
}

fn physics_step(
    dt: f32,
    rigidbodies: &mut Query<(&mut RigidBody, &mut Position)>,
) {
    // Apply forces (F = ma, so a = F/m)
    for (mut rb, _) in rigidbodies.iter_mut() {
        let acceleration = rb.force / rb.mass;
        rb.velocity += acceleration * dt;
        rb.force = Vec3::ZERO; // Clear forces after applying
    }
    
    // Integrate positions (Euler integration)
    for (rb, mut pos) in rigidbodies.iter_mut() {
        pos.0 += rb.velocity * dt;
    }
    
    // Collision detection and resolution would happen here
    // detect_collisions(rigidbodies);
    // resolve_collisions();
}

// Example: applying forces in gameplay systems
pub fn apply_gravity(mut rigidbodies: Query<&mut RigidBody>) {
    const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);
    
    for mut rb in rigidbodies.iter_mut() {
        rb.force += GRAVITY * rb.mass;
    }
}

// Schedule configuration
pub fn configure_physics_schedule(schedule: &mut Schedule) {
    schedule.add_systems(
        (
            apply_gravity,        // Accumulate forces
            physics_system,       // Fixed timestep integration
        ).chain()
    );
}
```

**Key Rust/ECS Patterns**:
- `PhysicsAccumulator` resource persists across frames
- `Time` resource provided by engine, automatically updated
- System runs in schedule, order controlled by `.chain()`
- Mutable queries allow safe parallel iteration within systems
- Clamping prevents "spiral of death" (when physics can't keep up)
- Forces cleared after each substep to prevent accumulation

### C++ (Unreal-style)

```cpp
// PhysicsEngine.h
class UPhysicsEngine : public UObject {
private:
    float FixedTimestep = 1.0f / 60.0f;  // 60 Hz
    float Accumulator = 0.0f;
    int32 MaxSubsteps = 4;

public:
    void Tick(float DeltaTime);
    void PhysicsStep(float Dt);
};

// PhysicsEngine.cpp
void UPhysicsEngine::Tick(float DeltaTime) {
    // Clamp delta time to prevent spiral of death
    DeltaTime = FMath::Min(DeltaTime, 0.25f);
    
    Accumulator += DeltaTime;
    
    int32 Substeps = 0;
    while (Accumulator >= FixedTimestep) {
        if (Substeps >= MaxSubsteps) {
            // Discard remaining time to prevent lockup
            Accumulator = 0.0f;
            UE_LOG(LogPhysics, Warning, TEXT("Physics substep limit reached"));
            break;
        }
        
        PhysicsStep(FixedTimestep);
        Accumulator -= FixedTimestep;
        Substeps++;
    }
}

void UPhysicsEngine::PhysicsStep(float Dt) {
    // Get all physics bodies
    TArray<UPrimitiveComponent*> PhysicsBodies;
    GetAllPhysicsBodies(PhysicsBodies);
    
    // Apply forces
    for (UPrimitiveComponent* Body : PhysicsBodies) {
        if (Body->IsSimulatingPhysics()) {
            FVector Acceleration = Body->GetForce() / Body->GetMass();
            FVector NewVelocity = Body->GetVelocity() + Acceleration * Dt;
            Body->SetVelocity(NewVelocity);
            Body->ClearForces();
        }
    }
    
    // Integrate positions
    for (UPrimitiveComponent* Body : PhysicsBodies) {
        if (Body->IsSimulatingPhysics()) {
            FVector NewPosition = Body->GetPosition() + Body->GetVelocity() * Dt;
            Body->SetPosition(NewPosition);
        }
    }
    
    // Detect and resolve collisions
    DetectCollisions();
    ResolveCollisions();
}

// Game thread calls physics tick
void AGameMode::Tick(float DeltaTime) {
    Super::Tick(DeltaTime);
    
    if (PhysicsEngine) {
        PhysicsEngine->Tick(DeltaTime);
    }
}

// Rigidbody component
class UPrimitiveComponent : public USceneComponent {
protected:
    FVector LinearVelocity;
    FVector AngularVelocity;
    FVector AccumulatedForce;
    FVector AccumulatedTorque;
    float Mass = 1.0f;
    bool bSimulatePhysics = false;

public:
    void AddForce(const FVector& Force) {
        AccumulatedForce += Force;
    }
    
    void ClearForces() {
        AccumulatedForce = FVector::ZeroVector;
        AccumulatedTorque = FVector::ZeroVector;
    }
    
    FVector GetForce() const { return AccumulatedForce; }
    FVector GetVelocity() const { return LinearVelocity; }
    void SetVelocity(const FVector& NewVelocity) { LinearVelocity = NewVelocity; }
    float GetMass() const { return Mass; }
    bool IsSimulatingPhysics() const { return bSimulatePhysics; }
};
```

**Key C++/Unreal Patterns**:
- Physics engine is a UObject with `Tick(float DeltaTime)` method
- Accumulator stored as class member (persists across frames)
- Direct iteration over array of component pointers
- Forces accumulated in component, cleared after each step
- Unreal's `FMath` utilities for clamping
- Physics runs on game thread (though Unreal's real physics uses PhysX on separate thread)

### C# (Unity-style)

```csharp
using UnityEngine;

// Unity's built-in fixed timestep is configured in Project Settings
// Time.fixedDeltaTime controls the physics timestep (default: 0.02 = 50 Hz)

public class PhysicsExample : MonoBehaviour {
    // FixedUpdate automatically runs at fixed timestep
    void FixedUpdate() {
        // Unity's physics runs here automatically
        // This is where you apply forces to Rigidbodies
        
        // Time.fixedDeltaTime is the fixed timestep (e.g., 0.02)
        float dt = Time.fixedDeltaTime;
        
        // Custom physics logic
        ApplyCustomForces();
    }
    
    // Update runs at variable frame rate
    void Update() {
        // Rendering and input handling here
        // Time.deltaTime is the variable frame time
    }
    
    void ApplyCustomForces() {
        // Unity's Rigidbody handles accumulator pattern internally
        Rigidbody rb = GetComponent<Rigidbody>();
        if (rb != null) {
            Vector3 gravity = Physics.gravity; // Default: (0, -9.81, 0)
            rb.AddForce(gravity * rb.mass);
        }
    }
}

// Manual implementation for educational purposes
public class ManualFixedTimestep : MonoBehaviour {
    public float fixedTimestep = 1.0f / 60.0f; // 60 Hz
    private float accumulator = 0.0f;
    
    void Update() {
        float deltaTime = Time.deltaTime;
        
        // Clamp to prevent spiral of death
        if (deltaTime > 0.25f) {
            deltaTime = 0.25f;
        }
        
        accumulator += deltaTime;
        
        // Fixed timestep updates
        while (accumulator >= fixedTimestep) {
            PhysicsStep(fixedTimestep);
            accumulator -= fixedTimestep;
        }
    }
    
    void PhysicsStep(float dt) {
        // Custom physics simulation
        // In practice, Unity's built-in physics is much more robust
    }
}

// Unity's internal physics loop (simplified)
public static class UnityPhysicsLoop {
    // Unity's actual implementation (conceptual):
    // - FixedUpdate is called in a fixed timestep loop
    // - Physics simulation (collision, integration) happens between FixedUpdate calls
    // - Accumulator pattern is built into the engine
    
    public static void InternalGameLoop() {
        float previousTime = Time.realtimeSinceStartup;
        float accumulator = 0.0f;
        
        while (true) {  // Main game loop
            float currentTime = Time.realtimeSinceStartup;
            float deltaTime = currentTime - previousTime;
            previousTime = currentTime;
            
            // Variable timestep: Update, LateUpdate, input
            CallUpdate(deltaTime);
            
            // Fixed timestep: FixedUpdate, physics
            accumulator += deltaTime;
            while (accumulator >= Time.fixedDeltaTime) {
                CallFixedUpdate();
                PhysicsSimulate(Time.fixedDeltaTime);
                accumulator -= Time.fixedDeltaTime;
            }
            
            // Render
            Render();
        }
    }
}

// Configuring physics timestep
public class PhysicsConfiguration : MonoBehaviour {
    void Start() {
        // Set fixed timestep (default: 0.02 = 50 Hz)
        Time.fixedDeltaTime = 1.0f / 60.0f;  // 60 Hz
        
        // Maximum allowed timestep (prevents spiral of death)
        Time.maximumDeltaTime = 0.1f;  // 100ms max
    }
}
```

**Key C#/Unity Patterns**:
- Unity provides `FixedUpdate()` which automatically runs at `Time.fixedDeltaTime`
- Accumulator pattern built into engine (transparent to user)
- `Time.fixedDeltaTime` configurable in project settings or code
- `Time.maximumDeltaTime` prevents spiral of death
- Physics simulation (PhysX) runs automatically between `FixedUpdate` calls
- `Rigidbody.AddForce()` accumulates forces applied during frame

### Comparison Table

| Aspect | Rust (Praxis) | C++ (Unreal) | C# (Unity) |
|--------|---------------|---------------|-------------|
| **Timestep Control** | Manual accumulator in system | Manual accumulator in engine | Built-in `FixedUpdate()` |
| **Configuration** | `PhysicsConfig` resource | Engine settings or code | `Time.fixedDeltaTime` property |
| **Spiral Prevention** | Manual clamping | Manual clamping | `Time.maximumDeltaTime` |
| **Force Accumulation** | Component field, cleared manually | Component field, cleared manually | `Rigidbody.AddForce()` internal |
| **Integration** | System in schedule | `Tick()` method override | `FixedUpdate()` callback |
| **User Visibility** | Explicit (user sees accumulator) | Explicit (user sees accumulator) | Implicit (Unity handles it) |

---

## Conclusion

This guide demonstrates that while the underlying algorithms remain consistent across engines, implementation details vary based on architectural philosophy:

- **Rust/ECS (Praxis)**: Data-oriented, query-based, explicit control
- **C++/OOP (Unreal)**: Object hierarchy, virtual methods, manual management
- **C#/Component (Unity)**: Component-based, callbacks, high-level abstractions

Understanding these patterns allows you to:
1. Read and understand codebases in any engine
2. Translate concepts between engines when switching projects
3. Make informed architectural decisions for custom engines
4. Recognize trade-offs between different approaches

## Further Reading

- [Transform Hierarchy Concepts](../concepts/transform-hierarchy.md)
- [ECS Architecture](../concepts/ecs-architecture.md)
- [Game Loop Patterns](patterns/game-loop-patterns.md)
- [Rendering Architecture Patterns](patterns/rendering-architecture-patterns.md)
