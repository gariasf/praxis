# Exercise 36: Octree Spatial Partitioning

**Difficulty**: 🔴 Advanced | **Estimated Time**: 5-6h | **Subsystem**: Spatial

## Overview

Implement an octree data structure for efficient spatial queries in 3D space. Octrees are fundamental for broad-phase collision detection, frustum culling, and spatial searching.

## Learning Objectives

- Understand hierarchical space partitioning
- Implement recursive tree construction
- Learn spatial query optimization
- Balance tree depth vs object count

## Requirements

### Functional Requirements

1. **Tree Construction**
   - Build octree from collection of AABBs/points
   - Subdivide nodes when capacity exceeded
   - Set maximum depth to prevent infinite subdivision

2. **Spatial Queries**
   - Point query: Find all objects containing point
   - Range query: Find all objects in AABB
   - Frustum query: Find all objects in view frustum
   - Raycast: Find objects intersecting ray

3. **Dynamic Updates**
   - Insert object into octree
   - Remove object from octree
   - Update object position (remove + reinsert)

### Non-Functional Requirements

- **Performance**: Query 10,000 objects in < 1ms
- **Memory**: O(n) space complexity
- **Scalability**: Handle 100,000+ objects

## API Design

```rust
pub struct Octree<T> {
    root: OctreeNode<T>,
    bounds: AABB,
    max_objects_per_node: usize,
    max_depth: usize,
}

struct OctreeNode<T> {
    bounds: AABB,
    objects: Vec<(AABB, T)>,
    children: Option<Box<[OctreeNode<T>; 8]>>,
}

impl<T> Octree<T> {
    pub fn new(bounds: AABB, max_objects: usize, max_depth: usize) -> Self;
    
    pub fn insert(&mut self, bounds: AABB, object: T);
    pub fn remove(&mut self, bounds: AABB, object: &T) -> bool where T: PartialEq;
    pub fn clear(&mut self);
    
    pub fn query_point(&self, point: Vec3) -> Vec<&T>;
    pub fn query_range(&self, range: AABB) -> Vec<&T>;
    pub fn query_ray(&self, origin: Vec3, direction: Vec3) -> Vec<&T>;
    
    pub fn object_count(&self) -> usize;
    pub fn depth(&self) -> usize;
}
```

## Validation Criteria

### Correctness
- [ ] All objects inserted are findable via queries
- [ ] Range queries return only objects in range
- [ ] Removed objects not returned in queries
- [ ] Handles objects spanning multiple nodes

### Performance
- [ ] Insert 10,000 objects in < 100ms
- [ ] Range query on 10,000 objects in < 1ms
- [ ] Scales logarithmically with object count

## Test Cases

```rust
#[test]
fn test_basic_insertion_and_query() {
    let mut octree = Octree::new(
        AABB::new(Vec3::splat(-100.0), Vec3::splat(100.0)),
        8,
        5
    );
    
    let obj_bounds = AABB::new(Vec3::ZERO, Vec3::ONE);
    octree.insert(obj_bounds, 42);
    
    let results = octree.query_point(Vec3::new(0.5, 0.5, 0.5));
    assert_eq!(results.len(), 1);
    assert_eq!(*results[0], 42);
}

#[test]
fn test_subdivision() {
    let mut octree = Octree::new(
        AABB::new(Vec3::splat(-10.0), Vec3::splat(10.0)),
        4, // Max 4 objects before subdivision
        5
    );
    
    // Insert 5 objects in same region - should trigger subdivision
    for i in 0..5 {
        let bounds = AABB::new(
            Vec3::new(i as f32, 0.0, 0.0),
            Vec3::new(i as f32 + 0.5, 0.5, 0.5)
        );
        octree.insert(bounds, i);
    }
    
    assert!(octree.depth() > 1);
}

#[test]
fn test_range_query() {
    let mut octree = Octree::new(
        AABB::new(Vec3::splat(-100.0), Vec3::splat(100.0)),
        8,
        5
    );
    
    // Insert objects at various positions
    for i in 0..100 {
        let pos = Vec3::new((i % 10) as f32 * 10.0, 0.0, 0.0);
        let bounds = AABB::new(pos, pos + Vec3::ONE);
        octree.insert(bounds, i);
    }
    
    // Query small region
    let query_range = AABB::new(Vec3::ZERO, Vec3::new(15.0, 5.0, 5.0));
    let results = octree.query_range(query_range);
    
    // Should only get objects in range
    assert!(results.len() < 100);
    assert!(results.len() > 0);
}

#[test]
fn test_removal() {
    let mut octree = Octree::new(
        AABB::new(Vec3::splat(-10.0), Vec3::splat(10.0)),
        8,
        5
    );
    
    let bounds = AABB::new(Vec3::ZERO, Vec3::ONE);
    octree.insert(bounds, 42);
    
    assert_eq!(octree.object_count(), 1);
    
    assert!(octree.remove(bounds, &42));
    assert_eq!(octree.object_count(), 0);
}
```

## Performance Targets

| Operation | Target |
|-----------|--------|
| Insert 10K objects | < 100ms |
| Range query (10% overlap) | < 1ms |
| Point query | < 0.1ms |
| Ray query | < 1ms |

## Hints & Guidance

### Octree Child Indexing
```
Children are indexed 0-7 based on which octant they occupy:
  bit 0: x axis (0=left, 1=right)
  bit 1: y axis (0=bottom, 1=top)
  bit 2: z axis (0=front, 1=back)

Example: child 5 = binary 101 = right, bottom, back
```

### Subdivision Strategy
- Only subdivide when node exceeds max_objects AND depth < max_depth
- Objects spanning multiple children stay in parent node
- Consider "loose octree" variant for moving objects

### Query Optimization
- Early exit if node doesn't intersect query
- Recursively search only relevant children
- Collect results into vector (avoid allocations)

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use glam::Vec3;

#[derive(Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
    
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    
    pub fn intersects(&self, other: &AABB) -> bool {
        self.max.x >= other.min.x && self.min.x <= other.max.x &&
        self.max.y >= other.min.y && self.min.y <= other.max.y &&
        self.max.z >= other.min.z && self.min.z <= other.max.z
    }
    
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
}

pub struct Octree<T> {
    root: OctreeNode<T>,
    bounds: AABB,
    max_objects_per_node: usize,
    max_depth: usize,
}

struct OctreeNode<T> {
    bounds: AABB,
    objects: Vec<(AABB, T)>,
    children: Option<Box<[OctreeNode<T>; 8]>>,
    depth: usize,
}

impl<T> Octree<T> {
    pub fn new(bounds: AABB, max_objects: usize, max_depth: usize) -> Self {
        Self {
            root: OctreeNode {
                bounds: bounds.clone(),
                objects: Vec::new(),
                children: None,
                depth: 0,
            },
            bounds,
            max_objects_per_node: max_objects,
            max_depth,
        }
    }
    
    pub fn insert(&mut self, bounds: AABB, object: T) {
        self.root.insert(
            bounds,
            object,
            self.max_objects_per_node,
            self.max_depth,
        );
    }
    
    pub fn query_range(&self, range: AABB) -> Vec<&T> {
        let mut results = Vec::new();
        self.root.query_range(&range, &mut results);
        results
    }
    
    pub fn query_point(&self, point: Vec3) -> Vec<&T> {
        let mut results = Vec::new();
        self.root.query_point(point, &mut results);
        results
    }
    
    pub fn object_count(&self) -> usize {
        self.root.object_count()
    }
    
    pub fn depth(&self) -> usize {
        self.root.max_depth()
    }
    
    pub fn clear(&mut self) {
        self.root.objects.clear();
        self.root.children = None;
    }
}

impl<T> OctreeNode<T> {
    fn insert(&mut self, bounds: AABB, object: T, max_objects: usize, max_depth: usize) {
        // If we have children, try to insert into appropriate child
        if let Some(ref mut children) = self.children {
            let index = self.get_octant(&bounds);
            if let Some(idx) = index {
                children[idx].insert(bounds, object, max_objects, max_depth);
                return;
            }
            // Object spans multiple octants, keep in this node
        }
        
        // Add to this node
        self.objects.push((bounds, object));
        
        // Check if we need to subdivide
        if self.objects.len() > max_objects && self.depth < max_depth && self.children.is_none() {
            self.subdivide(max_objects, max_depth);
        }
    }
    
    fn subdivide(&mut self, max_objects: usize, max_depth: usize) {
        let center = self.bounds.center();
        let min = self.bounds.min;
        let max = self.bounds.max;
        
        // Create 8 children
        let mut children = Box::new([
            // Bottom layer (y = min to center)
            OctreeNode::new(AABB::new(min, center), self.depth + 1),
            OctreeNode::new(AABB::new(Vec3::new(center.x, min.y, min.z), 
                                      Vec3::new(max.x, center.y, center.z)), self.depth + 1),
            OctreeNode::new(AABB::new(Vec3::new(min.x, min.y, center.z), 
                                      Vec3::new(center.x, center.y, max.z)), self.depth + 1),
            OctreeNode::new(AABB::new(Vec3::new(center.x, min.y, center.z), 
                                      Vec3::new(max.x, center.y, max.z)), self.depth + 1),
            // Top layer (y = center to max)
            OctreeNode::new(AABB::new(Vec3::new(min.x, center.y, min.z), 
                                      Vec3::new(center.x, max.y, center.z)), self.depth + 1),
            OctreeNode::new(AABB::new(Vec3::new(center.x, center.y, min.z), 
                                      Vec3::new(max.x, max.y, center.z)), self.depth + 1),
            OctreeNode::new(AABB::new(Vec3::new(min.x, center.y, center.z), 
                                      Vec3::new(center.x, max.y, max.z)), self.depth + 1),
            OctreeNode::new(AABB::new(center, max), self.depth + 1),
        ]);
        
        // Try to move existing objects into children
        let mut objects_to_keep = Vec::new();
        for (bounds, object) in self.objects.drain(..) {
            let index = self.get_octant(&bounds);
            if let Some(idx) = index {
                children[idx].objects.push((bounds, object));
            } else {
                // Spans multiple octants, keep in parent
                objects_to_keep.push((bounds, object));
            }
        }
        
        self.objects = objects_to_keep;
        self.children = Some(children);
    }
    
    fn get_octant(&self, bounds: &AABB) -> Option<usize> {
        let center = self.bounds.center();
        
        // Check if object fully fits in one octant
        let left = bounds.max.x <= center.x;
        let right = bounds.min.x >= center.x;
        let bottom = bounds.max.y <= center.y;
        let top = bounds.min.y >= center.y;
        let front = bounds.max.z <= center.z;
        let back = bounds.min.z >= center.z;
        
        if left && bottom && front {
            Some(0)
        } else if right && bottom && front {
            Some(1)
        } else if left && bottom && back {
            Some(2)
        } else if right && bottom && back {
            Some(3)
        } else if left && top && front {
            Some(4)
        } else if right && top && front {
            Some(5)
        } else if left && top && back {
            Some(6)
        } else if right && top && back {
            Some(7)
        } else {
            None // Spans multiple octants
        }
    }
    
    fn query_range(&self, range: &AABB, results: &mut Vec<&T>) {
        // Check if this node intersects the query range
        if !self.bounds.intersects(range) {
            return;
        }
        
        // Check objects in this node
        for (bounds, object) in &self.objects {
            if bounds.intersects(range) {
                results.push(object);
            }
        }
        
        // Recursively check children
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_range(range, results);
            }
        }
    }
    
    fn query_point(&self, point: Vec3, results: &mut Vec<&T>) {
        if !self.bounds.contains_point(point) {
            return;
        }
        
        for (bounds, object) in &self.objects {
            if bounds.contains_point(point) {
                results.push(object);
            }
        }
        
        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_point(point, results);
            }
        }
    }
    
    fn object_count(&self) -> usize {
        let mut count = self.objects.len();
        if let Some(ref children) = self.children {
            for child in children.iter() {
                count += child.object_count();
            }
        }
        count
    }
    
    fn max_depth(&self) -> usize {
        if let Some(ref children) = self.children {
            let max_child_depth = children.iter().map(|c| c.max_depth()).max().unwrap_or(0);
            1 + max_child_depth
        } else {
            1
        }
    }
    
    fn new(bounds: AABB, depth: usize) -> Self {
        Self {
            bounds,
            objects: Vec::new(),
            children: None,
            depth,
        }
    }
}
```

</details>

## Related Resources

- [Praxis Spatial Documentation](../../reference/crates.md#praxis_spatial)
- [Spatial Partitioning Benchmark](../../benchmarking.md#spatial-queries)
- [Real-Time Rendering - Spatial Data Structures](http://www.realtimerendering.com/)

## Next Steps

- Implement BVH (Exercise 37) for comparison
- Add frustum culling queries
- Integrate with rendering pipeline
