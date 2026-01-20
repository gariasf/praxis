# Frustum Culling

<span class="difficulty-badge difficulty-intermediate">Intermediate</span>

Frustum culling eliminates objects outside the camera's view volume before rendering, significantly improving performance by avoiding unnecessary draw calls.

## Overview

The camera's view frustum is defined by six planes (near, far, left, right, top, bottom). For each renderable object:

1. Extract the object's bounding volume (sphere or AABB)
2. Test the bounding volume against each frustum plane
3. If entirely outside any plane, cull the object
4. Otherwise, mark for rendering

## Algorithm

=== "Pseudocode"

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

=== "Rust (Praxis)"

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
            Err(_) => return,
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

    **Key Patterns**:
    
    - Frustum extracted from camera's view-projection matrix
    - Query iterates only entities with required components
    - Visibility tracked via marker component (`Visible`)
    - No allocations in hot path

=== "C++ (Unreal)"

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
        
        // Left plane: row4 + row1
        Result.Planes[0] = NormalizePlane(
            FPlane(
                FVector(ViewProj.M[0][3] + ViewProj.M[0][0],
                        ViewProj.M[1][3] + ViewProj.M[1][0],
                        ViewProj.M[2][3] + ViewProj.M[2][0]),
                ViewProj.M[3][3] + ViewProj.M[3][0]
            )
        );
        
        // ... similar for other planes
        
        return Result;
    }

    bool FFrustum::IsVisible(const FSphere& BoundingSphere) const {
        for (int32 i = 0; i < 6; ++i) {
            float Distance = Planes[i].DistanceToPoint(BoundingSphere.Center);
            if (Distance < -BoundingSphere.W) {  // W stores radius
                return false;
            }
        }
        return true;
    }

    // Renderer performs culling
    class URenderer {
    public:
        void CullAndRender(UWorld* World, UCameraComponent* Camera) {
            FMatrix ViewProjection = Camera->GetViewProjectionMatrix();
            FFrustum Frustum = FFrustum::FromViewProjection(ViewProjection);
            
            TArray<UPrimitiveComponent*> Primitives;
            GetAllPrimitives(World, Primitives);
            
            for (UPrimitiveComponent* Primitive : Primitives) {
                FSphere WorldBounds = Primitive->Bounds.TransformBy(
                    Primitive->GetComponentToWorld()
                );
                
                if (Frustum.IsVisible(WorldBounds)) {
                    RenderPrimitive(Primitive);
                }
            }
        }
    };
    ```

    **Key Patterns**:
    
    - Frustum stored as struct with six planes
    - Direct iteration over array of component pointers
    - Unreal's custom types with operator overloads

=== "C# (Unity)"

    ```csharp
    using UnityEngine;
    using System.Collections.Generic;

    public struct Frustum {
        private Plane[] planes;
        
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

    public class FrustumCuller : MonoBehaviour {
        public Camera mainCamera;
        private List<Renderer> allRenderers = new List<Renderer>();
        
        void Start() {
            allRenderers.AddRange(FindObjectsOfType<Renderer>());
        }
        
        void Update() {
            if (mainCamera == null) return;
            
            Frustum frustum = Frustum.FromCamera(mainCamera);
            
            foreach (Renderer renderer in allRenderers) {
                bool isVisible = frustum.IsVisible(renderer.bounds);
                renderer.enabled = isVisible;
            }
        }
    }

    // Unity's automatic culling
    public class UnityBuiltInCulling : MonoBehaviour {
        void OnBecameVisible() {
            Debug.Log($"{gameObject.name} became visible");
        }
        
        void OnBecameInvisible() {
            Debug.Log($"{gameObject.name} became invisible");
        }
    }
    ```

    **Key Patterns**:
    
    - Unity provides built-in `GeometryUtility.CalculateFrustumPlanes()`
    - `Renderer.isVisible` automatically maintained
    - Callbacks for visibility events

## Comparison

| Aspect | Rust (Praxis) | C++ (Unreal) | C# (Unity) |
|--------|---------------|---------------|-------------|
| **Frustum Storage** | Struct with plane array | Struct with plane array | Built-in utilities |
| **Visibility Tracking** | Marker component | Boolean flag | `Renderer.enabled` |
| **Bounds Representation** | `BoundingSphere` | `FSphere` | `Bounds` (AABB) |
| **Culling Trigger** | System each frame | Manual in render loop | Automatic |
| **Parallel Potential** | High | Low | Low (Unity internal) |

## Optimization Strategies

### Spatial Partitioning
Combine with octrees or BVH to avoid testing all objects:

```
spatial_tree.query_frustum(frustum, |entity| {
    // Only test objects in potentially visible regions
});
```

### Hierarchical Culling
Test parent bounding volumes first; skip children if parent is culled:

```
if !frustum.is_visible(parent.bounds) {
    // Skip entire subtree
    continue;
}
```

### GPU Culling
Move frustum tests to GPU via compute shaders (advanced):

```
// Dispatch compute shader that outputs visible indices
// Rendering uses indirect draw with culled list
```

## Performance Metrics

Typical improvements with frustum culling:

- **Open world scenes**: 50-80% reduction in draw calls
- **Indoor scenes**: 70-90% reduction
- **Dense forests/cities**: 60-85% reduction

!!! tip "When to Use"
    Frustum culling is essential for any 3D scene with more than ~100 objects.

!!! warning "Don't Over-Optimize"
    For scenes with < 50 objects, the overhead may exceed the benefit.

## Common Pitfalls

!!! danger "Incorrect Plane Extraction"
    Extracting frustum planes incorrectly leads to objects popping in/out.
    Always test with wireframe rendering!

!!! danger "Forgetting Scale"
    Bounding volumes must account for non-uniform scale transformations.

!!! danger "Precision Issues"
    Use appropriate epsilon values when comparing distances to avoid Z-fighting.

## Further Reading

- [Spatial Optimization Guide](../../guides/spatial-optimization.md)
- [GPU Culling](../../guides/rendering/gpu-culling.md)
- [Octree Implementation](../../patterns/spatial-partitioning.md)

## Exercises

1. **Implement AABB culling** - Extend the example to use axis-aligned bounding boxes
2. **Add hierarchical culling** - Skip testing children when parent is culled
3. **Profile performance** - Measure draw call reduction in a complex scene
4. **Visualize frustum** - Render frustum planes for debugging

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="transform-propagation.html" class="md-button">← Previous: Transform Propagation</a>
  <a href="fixed-timestep-physics.html" class="md-button">Next: Fixed Timestep Physics →</a>
</div>
