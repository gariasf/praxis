# Transform Propagation

<span class="difficulty-badge difficulty-intermediate">Intermediate</span>

Transform propagation updates world-space transforms based on local transforms and parent-child hierarchies. This is fundamental to scene graphs in every game engine.

## Overview

When a parent object moves, rotates, or scales, all its children must update accordingly. The core algorithm:

1. Identify root entities (no parent)
2. Update each root's global transform from its local transform
3. For each root's children, recursively compute: `child_global = parent_global * child_local`
4. Handle change detection to avoid redundant updates

## Algorithm

=== "Pseudocode"

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

=== "Rust (Praxis)"

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
            Mat4::from_scale_rotation_translation(
                self.scale, 
                self.rotation, 
                self.translation
            )
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

    **Key Patterns**:
    
    - `Query<T, Filter>` enables efficient component iteration
    - `Changed<T>` automatically detects modifications
    - `With<T>` and `Without<T>` filter by component presence
    - Work queue avoids recursion and borrow checker issues

=== "C++ (Unreal)"

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

    **Key Patterns**:
    
    - Object-oriented hierarchy with pointers to parent/children
    - Dirty flags track when updates are needed
    - Virtual functions allow polymorphic behavior
    - Manual recursion (no borrow checker concerns)

=== "C# (Unity)"

    ```csharp
    // Transform.cs (simplified Unity-style)
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
    ```

    **Key Patterns**:
    
    - Lazy evaluation with dirty flags
    - Properties provide clean API
    - Automatic garbage collection
    - Internal batching system for updates

## Comparison

| Aspect | Rust (Praxis) | C++ (Unreal) | C# (Unity) |
|--------|---------------|---------------|-------------|
| **Data Layout** | Separate component arrays | Objects with embedded data | Components on GameObjects |
| **Ownership** | Compile-time borrow checker | Raw pointers with GC | Managed references with GC |
| **Change Detection** | Automatic ECS tracking | Manual dirty flags | Manual dirty flags + lazy eval |
| **Update Strategy** | Push (immediate) | Push (immediate) | Pull (lazy on access) |
| **Parallelization** | Easy (thread-safe queries) | Difficult (requires locking) | Difficult (Unity handles internally) |

## Performance Considerations

### Rust (Praxis)
- ✅ Cache-friendly archetype storage
- ✅ Automatic parallelization possible
- ✅ Zero-cost abstractions
- ⚠️ Requires careful query design

### C++ (Unreal)
- ✅ Direct memory access
- ✅ Virtual function flexibility
- ⚠️ Cache misses from pointer chasing
- ⚠️ Difficult to parallelize safely

### C# (Unity)
- ✅ Lazy evaluation reduces redundant updates
- ✅ Simple API for users
- ⚠️ GC pressure from allocations
- ⚠️ Properties have hidden costs

## Common Pitfalls

!!! warning "Circular Dependencies"
    Never create parent-child cycles! They cause infinite loops or stack overflows.

!!! warning "Dirty Flag Forgetting"
    In manual systems (C++/C#), forgetting to set dirty flags leads to stale transforms.

!!! warning "Premature Optimization"
    Don't optimize transform updates until profiling shows they're a bottleneck.

## Further Reading

- [Transform Hierarchy Concepts](../../concepts/transform-hierarchy.md)
- [ECS Architecture](../../concepts/ecs-architecture.md)
- [Scene Graphs](../../patterns/scene-graph-patterns.md)

## Exercises

1. **Implement in your language of choice** - Follow the pattern for your preferred language
2. **Add scale propagation** - Extend the example to handle non-uniform scale
3. **Profile performance** - Compare update strategies with 1000+ entities
4. **Add dirty flags optimization** - Implement change detection in C++ version

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="frustum-culling.html" class="md-button">Next: Frustum Culling →</a>
</div>
