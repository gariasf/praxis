//! Bounding Volume Hierarchy (BVH) for efficient ray tracing and spatial queries.
//!
//! A BVH is a tree structure where each node contains a bounding volume that encloses
//! its children. BVHs are typically faster than octrees for ray tracing and nearest neighbor queries.
//!
//! # How BVHs Work
//!
//! Unlike octrees which divide *space*, BVHs divide *objects*. Each node contains a bounding
//! box that tightly fits its children, creating a hierarchy where spatial proximity in 3D
//! space maps to tree proximity, but without fixed spatial divisions.
//!
//! ```text
//! BVH Structure:
//!
//!                    Root (bounds all objects)
//!                   /                          \
//!            Left Subtree                    Right Subtree
//!        (bounds left half)              (bounds right half)
//!            /         \                      /           \
//!      Left-Left  Left-Right          Right-Left   Right-Right
//!      (bounds)    (bounds)            (bounds)      (bounds)
//!         |           |                   |              |
//!      Objects     Objects             Objects        Objects
//! ```
//!
//! # BVH Construction Algorithm (Bottom-Up, Recursive)
//!
//! The `build_recursive` function implements a top-down construction strategy:
//!
//! ## Algorithm Steps:
//!
//! 1. **Base Case**: If only 1 entity, create leaf node with entity and its bounds
//!
//! 2. **Compute Combined Bounds**: Union of all entity bounding boxes
//!    - This becomes the parent node's bounds
//!    - Tightly encloses all children (no wasted space)
//!
//! 3. **Choose Split Axis**: Select axis with largest spatial extent
//!    - Compute size = (max - min) for each axis
//!    - Split along longest dimension (X, Y, or Z)
//!    - **Why?** Maximizes separation between children, minimizes overlap
//!    - **Example**: Linear arrangement along X → split on X axis
//!
//! 4. **Partition Objects**: Sort entities along chosen axis by their center points
//!    - Use quicksort/mergesort: O(n log n)
//!    - Split at median: left half gets first n/2, right half gets rest
//!    - **Why median?** Creates balanced tree (depth = log₂ n)
//!
//! 5. **Recurse**: Build left and right subtrees from partitioned entity lists
//!    - Left child: entities [0..mid]
//!    - Right child: entities [mid..n]
//!    - Each child recursively applies steps 1-5
//!
//! 6. **Create Internal Node**: Node with combined bounds and two child pointers
//!
//! ## Complexity Analysis:
//!
//! - **Time**: O(n log² n)
//!   - O(n log n) to sort at each level
//!   - O(log n) tree levels
//!   - Total: O(n log n) × O(log n) = O(n log² n)
//! - **Space**: O(n) for tree nodes
//! - **Tree Height**: O(log n) for balanced tree
//!
//! ## Construction Example:
//!
//! ```text
//! Given 4 objects with centers: A(0,0,0), B(10,0,0), C(0,10,0), D(10,10,0)
//!
//! Step 1: Compute combined bounds: [min:(0,0,0), max:(10,10,0)]
//! Step 2: Choose split axis: X and Y equal (10), choose X arbitrarily
//! Step 3: Sort by X: [A(0,_,_), C(0,_,_), B(10,_,_), D(10,_,_)]
//! Step 4: Split at median (2): Left=[A,C], Right=[B,D]
//! Step 5: Recurse:
//!         Left subtree: Bounds(A∪C), split on Y, children: Leaf(A), Leaf(C)
//!         Right subtree: Bounds(B∪D), split on Y, children: Leaf(B), Leaf(D)
//! Step 6: Root = Internal(bounds, left, right)
//! ```
//!
//! # BVH Traversal Algorithm (Ray Queries)
//!
//! BVHs excel at ray tracing because binary structure enables efficient traversal:
//!
//! ```text
//! function RayQuery(node, ray):
//!     // Test ray against node's bounding box
//!     if ray does NOT intersect node.bounds:
//!         return []  // Early rejection (KEY optimization)
//!
//!     if node is Leaf:
//!         return [node.entity]  // Found intersection
//!
//!     // Test both children (binary tree = 2 tests, not 8 like octree)
//!     results = []
//!     results += RayQuery(node.left, ray)
//!     results += RayQuery(node.right, ray)
//!     return results
//! ```
//!
//! ## Performance Analysis:
//!
//! - **Average case**: O(log n) ray-box tests
//! - **Worst case**: O(n) if ray pierces many nodes
//! - **Typical**: 10-30 tests for 1000-object scene
//!
//! ## Why BVH is Better for Rays:
//!
//! 1. **Tighter Bounds**: Boxes fit objects exactly (no empty space like octree)
//! 2. **Binary Branching**: Test 2 children vs 8 (better cache locality)
//! 3. **Adaptive Structure**: Automatically balances to object distribution
//!
//! # BVH vs Octree Trade-offs
//!
//! ## BVH Advantages:
//! - **Tight-fitting bounds**: No wasted space testing empty regions
//! - **Better ray tracing**: Near-optimal O(log n) average case
//! - **Adapts to clustering**: Naturally handles non-uniform object distribution
//! - **Binary tree**: Better CPU cache performance (2 children vs 8)
//! - **Frustum culling**: Tighter bounds = fewer false positives
//!
//! ## BVH Disadvantages:
//! - **Rebuild cost**: Full tree rebuild on any change (O(n log n))
//! - **Construction complexity**: More complex than octree subdivision
//! - **Memory overhead**: Stores explicit bounds per internal node
//! - **Less intuitive**: Spatial partitioning not as obvious as octree
//!
//! ## When to Use BVH:
//! - Ray tracing / ray casting (e.g., mouse picking, line-of-sight)
//! - Static or infrequently changing scenes
//! - Frustum culling for rendering
//! - Objects with non-uniform distribution (cities, forests)
//!
//! ## When to Use Octree Instead:
//! - Volumetric data (voxels, particles)
//! - Uniform object distribution
//! - Simpler implementation needs
//! - Educational purposes (more intuitive)
//!
//! # Integration with Rendering Pipeline
//!
//! BVH integrates into frustum culling via `query_with_predicate`:
//!
//! ```text
//! 1. Extract Frustum: Compute 6 planes from camera view-projection matrix
//! 2. Query BVH: bvh.query_with_predicate(|bounds| frustum.intersects(bounds))
//! 3. Hierarchical Test:
//!    - Test root bounds against frustum
//!    - If outside: cull entire scene
//!    - If inside/intersecting: recursively test left and right children
//!    - Accumulate visible entities from leaf nodes
//! 4. Result: List of potentially visible entities (ready for LOD selection)
//! ```
//!
//! **Performance Example**:
//! - 5,000 objects, 150 visible (3%)
//! - Brute force: 5,000 frustum tests
//! - BVH: ~25 node tests + 150 entity tests = 175 tests
//! - **Speedup**: 28× faster culling

use crate::aabb::Aabb;
use bevy_ecs::entity::Entity;
use praxis_math::Vec3;
use std::collections::HashMap;

/// Node in the BVH tree.
#[derive(Debug, Clone)]
pub enum BvhNode {
    /// Leaf node containing a single entity.
    Leaf {
        /// Entity contained in this leaf.
        entity: Entity,
        /// Bounding box of the entity.
        bounds: Aabb,
    },
    /// Internal node with two children.
    Internal {
        /// Bounding box that contains both children.
        bounds: Aabb,
        /// Left child node.
        left: Box<BvhNode>,
        /// Right child node.
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    /// Returns the bounding box of this node.
    pub fn bounds(&self) -> &Aabb {
        match self {
            Self::Leaf { bounds, .. } | Self::Internal { bounds, .. } => bounds,
        }
    }

    /// Queries all entities that intersect the given bounds.
    ///
    /// # Hierarchical Query Algorithm
    ///
    /// 1. **Early Rejection**: Test node bounds against query bounds first
    ///    - If no intersection, skip this entire subtree (KEY optimization)
    ///    - One AABB test eliminates entire branch
    /// 2. **Leaf Case**: If leaf node, add entity to results
    /// 3. **Internal Case**: Recursively query both left and right children
    ///    - Binary branching: exactly 2 recursive calls per internal node
    ///    - Each child performs its own bounds test (step 1)
    ///
    /// # Performance
    ///
    /// - **Best case**: O(log n) - query region intersects only one leaf path
    /// - **Average case**: O(log n + k) - k = number of results
    /// - **Worst case**: O(n) - query region overlaps entire tree
    ///
    /// Compare to octree's 8-way branching:
    /// - BVH: 2 children = better cache locality
    /// - Octree: 8 children = more branches to test
    pub fn query(&self, query_bounds: &Aabb, results: &mut Vec<Entity>) {
        if !self.bounds().intersects(query_bounds) {
            return;
        }

        match self {
            Self::Leaf { entity, .. } => {
                results.push(*entity);
            }
            Self::Internal { left, right, .. } => {
                left.query(query_bounds, results);
                right.query(query_bounds, results);
            }
        }
    }

    /// Queries entities within a radius of a point.
    pub fn query_radius(&self, point: Vec3, radius: f32, results: &mut Vec<Entity>) {
        let radius_sq = radius * radius;

        if self.bounds().distance_squared(point) > radius_sq {
            return;
        }

        match self {
            Self::Leaf { entity, bounds } => {
                if bounds.distance_squared(point) <= radius_sq {
                    results.push(*entity);
                }
            }
            Self::Internal { left, right, .. } => {
                left.query_radius(point, radius, results);
                right.query_radius(point, radius, results);
            }
        }
    }

    /// Returns the number of entities in this subtree.
    pub fn entity_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Internal { left, right, .. } => left.entity_count() + right.entity_count(),
        }
    }

    /// Queries all entities that intersect with a ray.
    pub fn query_ray(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        results: &mut Vec<Entity>,
    ) {
        if !self
            .bounds()
            .intersects_ray(origin, direction, max_distance)
        {
            return;
        }

        match self {
            Self::Leaf { entity, .. } => {
                results.push(*entity);
            }
            Self::Internal { left, right, .. } => {
                left.query_ray(origin, direction, max_distance, results);
                right.query_ray(origin, direction, max_distance, results);
            }
        }
    }
}

/// Bounding Volume Hierarchy for spatial partitioning.
///
/// BVHs are constructed bottom-up by recursively grouping entities based on spatial proximity.
pub struct Bvh {
    /// Root node of the BVH.
    root: Option<BvhNode>,
    /// Map from entity to its bounding box for dynamic updates.
    entity_bounds: HashMap<Entity, Aabb>,
}

impl Bvh {
    /// Creates a new empty BVH.
    pub fn new() -> Self {
        Self {
            root: None,
            entity_bounds: HashMap::new(),
        }
    }

    /// Builds a BVH from a list of entities and their bounding boxes.
    pub fn build(&mut self, entities: Vec<(Entity, Aabb)>) {
        if entities.is_empty() {
            self.root = None;
            self.entity_bounds.clear();
            return;
        }

        self.entity_bounds.clear();
        for (entity, bounds) in &entities {
            self.entity_bounds.insert(*entity, *bounds);
        }

        self.root = Some(Self::build_recursive(entities));
    }

    /// Recursively builds the BVH tree.
    ///
    /// # Construction Algorithm (Top-Down, Recursive)
    ///
    /// This implements a **Surface Area Heuristic (SAH)-lite** approach:
    ///
    /// 1. **Base Case**: Single entity → create leaf node
    ///
    /// 2. **Compute Bounds**: Union all entity AABBs to get parent bounds
    ///    - Tight fit: no empty space unlike octree's fixed cells
    ///
    /// 3. **Choose Split Axis**: Pick axis with largest extent (X, Y, or Z)
    ///    - **Goal**: Maximize separation between children
    ///    - **Why largest axis?** Objects spread out more on this axis
    ///    - **Example**: 100 objects in line along X-axis → split on X
    ///
    /// 4. **Sort and Partition**: Sort entities by center position on split axis
    ///    - Sort cost: O(n log n) per level
    ///    - Split at median: balanced tree (depth = log₂ n)
    ///    - Left subtree: first half, Right subtree: second half
    ///
    /// 5. **Recurse**: Build left and right children from partitions
    ///    - Each child is an independent BVH of its partition
    ///    - Tree naturally balances due to median split
    ///
    /// 6. **Create Internal Node**: Store combined bounds and child pointers
    ///
    /// # Complexity
    ///
    /// - **Time**: O(n log² n)
    ///   - Tree depth: O(log n) levels
    ///   - Sort at each level: O(n log n)
    ///   - Total: O(log n) × O(n log n) = O(n log² n)
    /// - **Space**: O(n) nodes in tree
    ///
    /// # Optimizations (Not Implemented)
    ///
    /// Advanced BVH builders use Surface Area Heuristic (SAH):
    /// - Test multiple split positions, choose one minimizing cost
    /// - Cost function: `surface_area(left)` × `count(left)` + `surface_area(right)` × `count(right)`
    /// - Improves ray tracing by 2-3×, but 10× slower to build
    /// - Trade-off: construction time vs query performance
    fn build_recursive(mut entities: Vec<(Entity, Aabb)>) -> BvhNode {
        if entities.len() == 1 {
            let (entity, bounds) = entities.pop().unwrap();
            return BvhNode::Leaf { entity, bounds };
        }

        let mut combined_bounds = entities[0].1;
        for (_, bounds) in &entities[1..] {
            combined_bounds = combined_bounds.union(bounds);
        }

        let size = combined_bounds.size();

        let split_axis = if size.x >= size.y && size.x >= size.z {
            0
        } else if size.y >= size.z {
            1
        } else {
            2
        };

        entities.sort_by(|(_, a), (_, b)| {
            let a_center = a.center();
            let b_center = b.center();

            let a_val = match split_axis {
                0 => a_center.x,
                1 => a_center.y,
                _ => a_center.z,
            };

            let b_val = match split_axis {
                0 => b_center.x,
                1 => b_center.y,
                _ => b_center.z,
            };

            a_val.partial_cmp(&b_val).unwrap()
        });

        let mid = entities.len() / 2;
        let left_entities = entities[..mid].to_vec();
        let right_entities = entities[mid..].to_vec();

        let left = Box::new(Self::build_recursive(left_entities));
        let right = Box::new(Self::build_recursive(right_entities));

        let bounds = left.bounds().union(right.bounds());

        BvhNode::Internal {
            bounds,
            left,
            right,
        }
    }

    /// Queries all entities that intersect the given bounds.
    pub fn query(&self, bounds: &Aabb) -> Vec<Entity> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.query(bounds, &mut results);
        }
        results
    }

    /// Queries all entities within the given radius of a point.
    pub fn query_radius(&self, point: Vec3, radius: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.query_radius(point, radius, &mut results);
        }
        results
    }

    /// Returns the total number of entities in the BVH.
    pub fn entity_count(&self) -> usize {
        self.root.as_ref().map_or(0, BvhNode::entity_count)
    }

    /// Clears the BVH.
    pub fn clear(&mut self) {
        self.root = None;
        self.entity_bounds.clear();
    }

    /// Returns true if the BVH is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the root bounding box, if any.
    pub fn bounds(&self) -> Option<&Aabb> {
        self.root.as_ref().map(BvhNode::bounds)
    }

    /// Queries all entities that intersect with a ray.
    pub fn query_ray(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Vec<Entity> {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            root.query_ray(origin, direction, max_distance, &mut results);
        }
        results
    }

    /// Queries all entities that intersect with a ray and returns them sorted by distance.
    pub fn query_ray_sorted(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Vec<(Entity, f32)> {
        let entities = self.query_ray(origin, direction, max_distance);
        let mut results = Vec::new();

        for entity in entities {
            if let Some(bounds) = self.entity_bounds.get(&entity) {
                if let Some(distance) =
                    bounds.ray_intersection_distance(origin, direction, max_distance)
                {
                    results.push((entity, distance));
                }
            }
        }

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Adds a single entity to the BVH (triggers rebuild).
    pub fn insert(&mut self, entity: Entity, bounds: Aabb) {
        self.entity_bounds.insert(entity, bounds);
        self.rebuild();
    }

    /// Removes an entity from the BVH (triggers rebuild).
    pub fn remove(&mut self, entity: Entity) -> bool {
        if self.entity_bounds.remove(&entity).is_some() {
            self.rebuild();
            true
        } else {
            false
        }
    }

    /// Updates an entity's bounds in the BVH (triggers rebuild).
    pub fn update(&mut self, entity: Entity, new_bounds: Aabb) {
        self.entity_bounds.insert(entity, new_bounds);
        self.rebuild();
    }

    /// Rebuilds the BVH from current entity bounds.
    pub fn rebuild(&mut self) {
        let entities: Vec<_> = self.entity_bounds.iter().map(|(&e, &b)| (e, b)).collect();
        self.build(entities);
    }

    /// Returns true if the BVH contains the entity.
    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_bounds.contains_key(&entity)
    }

    /// Gets the bounds of an entity if it exists in the BVH.
    pub fn get_bounds(&self, entity: Entity) -> Option<&Aabb> {
        self.entity_bounds.get(&entity)
    }

    /// Queries entities using a custom predicate for bounds testing.
    ///
    /// This enables hierarchical culling by testing node bounds against
    /// arbitrary predicates (e.g., frustum intersection) before descending.
    ///
    /// # Use Case: Frustum Culling Integration
    ///
    /// This is THE key function for rendering pipeline integration:
    ///
    /// ```rust,ignore
    /// // Extract frustum planes from camera
    /// let frustum = camera.extract_frustum();
    ///
    /// // Query BVH with frustum test as predicate
    /// let visible = bvh.query_with_predicate(|bounds| {
    ///     frustum.intersects_aabb(bounds)
    /// });
    ///
    /// // Result: only entities potentially visible to camera
    /// for entity in visible {
    ///     renderer.draw(entity);
    /// }
    /// ```
    ///
    /// # How It Works
    ///
    /// 1. Test root bounds against predicate (e.g., frustum intersection)
    /// 2. If fails: entire scene culled (early exit)
    /// 3. If passes: recursively test children
    /// 4. Leaf nodes: add entity if parent bounds passed
    ///
    /// # Performance
    ///
    /// Hierarchical testing dramatically reduces predicate evaluations:
    /// - **Without hierarchy**: Test all 10,000 entities
    /// - **With BVH**: Test ~25 internal nodes + 150 visible entities = 175 tests
    /// - **Speedup**: 57× fewer tests
    pub fn query_with_predicate<F>(&self, predicate: &F) -> Vec<Entity>
    where
        F: Fn(&Aabb) -> bool,
    {
        let mut results = Vec::new();
        if let Some(root) = &self.root {
            Self::query_node_with_predicate(root, predicate, &mut results);
        }
        results
    }

    /// Recursively queries a node with a predicate.
    ///
    /// # Hierarchical Culling in Action
    ///
    /// This function implements the core hierarchical culling optimization:
    /// - Test parent bounds BEFORE descending to children
    /// - If parent fails predicate, skip entire subtree
    /// - This single test can eliminate thousands of objects
    fn query_node_with_predicate<F>(node: &BvhNode, predicate: &F, results: &mut Vec<Entity>)
    where
        F: Fn(&Aabb) -> bool,
    {
        let bounds = node.bounds();
        
        // Validate bounds are finite before testing predicate
        if !bounds.min.is_finite() || !bounds.max.is_finite() {
            return;
        }

        // Test node bounds first (hierarchical culling)
        // This one test can cull entire subtree - THE key optimization
        if !predicate(bounds) {
            return;
        }

        match node {
            BvhNode::Leaf { entity, .. } => {
                results.push(*entity);
            }
            BvhNode::Internal { left, right, .. } => {
                Self::query_node_with_predicate(left, predicate, results);
                Self::query_node_with_predicate(right, predicate, results);
            }
        }
    }

    /// Returns a reference to the root node if available.
    pub fn root(&self) -> Option<&BvhNode> {
        self.root.as_ref()
    }
}

impl Default for Bvh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvh_creation() {
        let bvh = Bvh::new();
        assert!(bvh.is_empty());
        assert_eq!(bvh.entity_count(), 0);
    }

    #[test]
    fn test_bvh_build() {
        let mut bvh = Bvh::new();
        let entities = vec![
            (
                Entity::from_raw(1),
                Aabb::from_min_max(Vec3::ZERO, Vec3::ONE),
            ),
            (
                Entity::from_raw(2),
                Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0)),
            ),
        ];

        bvh.build(entities);
        assert_eq!(bvh.entity_count(), 2);
    }

    #[test]
    fn test_bvh_query() {
        let mut bvh = Bvh::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entities = vec![
            (entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
            (
                entity2,
                Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0)),
            ),
        ];

        bvh.build(entities);

        let query_bounds =
            Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let results = bvh.query(&query_bounds);

        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_bvh_query_radius() {
        let mut bvh = Bvh::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entities = vec![
            (entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
            (
                entity2,
                Aabb::from_min_max(Vec3::new(20.0, 0.0, 0.0), Vec3::new(21.0, 1.0, 1.0)),
            ),
        ];

        bvh.build(entities);

        let results = bvh.query_radius(Vec3::ZERO, 10.0);

        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_bvh_clear() {
        let mut bvh = Bvh::new();
        let entities = vec![(
            Entity::from_raw(1),
            Aabb::from_min_max(Vec3::ZERO, Vec3::ONE),
        )];

        bvh.build(entities);
        assert!(!bvh.is_empty());

        bvh.clear();
        assert!(bvh.is_empty());
    }

    #[test]
    fn test_bvh_insert_remove() {
        let mut bvh = Bvh::new();
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        bvh.insert(entity, bounds);
        assert!(bvh.contains(entity));
        assert_eq!(bvh.entity_count(), 1);

        assert!(bvh.remove(entity));
        assert!(!bvh.contains(entity));
        assert_eq!(bvh.entity_count(), 0);
    }

    #[test]
    fn test_bvh_update() {
        let mut bvh = Bvh::new();
        let entity = Entity::from_raw(1);
        let bounds1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let bounds2 = Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0));

        bvh.insert(entity, bounds1);
        bvh.update(entity, bounds2);

        let stored = bvh.get_bounds(entity);
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().min, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_bvh_ray_query() {
        let mut bvh = Bvh::new();

        for i in 0..5 {
            let entity = Entity::from_raw(i);
            let x = (i as f32).mul_add(10.0, 5.0);
            let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(2.0));
            bvh.insert(entity, bounds);
        }

        let origin = Vec3::ZERO;
        let direction = Vec3::X;
        let results = bvh.query_ray(origin, direction, 100.0);

        assert!(!results.is_empty());
    }

    #[test]
    fn test_bvh_ray_sorted() {
        let mut bvh = Bvh::new();

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        bvh.insert(
            entity1,
            Aabb::from_center_half_extents(Vec3::new(10.0, 0.0, 0.0), Vec3::splat(1.0)),
        );
        bvh.insert(
            entity2,
            Aabb::from_center_half_extents(Vec3::new(30.0, 0.0, 0.0), Vec3::splat(1.0)),
        );
        bvh.insert(
            entity3,
            Aabb::from_center_half_extents(Vec3::new(20.0, 0.0, 0.0), Vec3::splat(1.0)),
        );

        let results = bvh.query_ray_sorted(Vec3::ZERO, Vec3::X, 100.0);

        assert_eq!(results.len(), 3);

        for i in 0..results.len() - 1 {
            assert!(results[i].1 <= results[i + 1].1);
        }
    }

    #[test]
    fn test_bvh_get_bounds() {
        let mut bvh = Bvh::new();
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

        bvh.insert(entity, bounds);

        let retrieved = bvh.get_bounds(entity);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().min, Vec3::ZERO);
        assert_eq!(retrieved.unwrap().max, Vec3::ONE);
    }

    #[test]
    fn test_bvh_query_with_predicate() {
        let mut bvh = Bvh::new();

        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);

        bvh.insert(entity1, Aabb::from_min_max(Vec3::ZERO, Vec3::ONE));
        bvh.insert(
            entity2,
            Aabb::from_min_max(Vec3::new(10.0, 0.0, 0.0), Vec3::new(11.0, 1.0, 1.0)),
        );
        bvh.insert(
            entity3,
            Aabb::from_min_max(Vec3::new(-10.0, 0.0, 0.0), Vec3::new(-9.0, 1.0, 1.0)),
        );

        let query_bounds =
            Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
        let results = bvh.query_with_predicate(&|bounds| query_bounds.intersects(bounds));

        assert!(results.contains(&entity1));
        assert!(!results.contains(&entity2));
    }

    #[test]
    fn test_bvh_hierarchical_culling() {
        let mut bvh = Bvh::new();

        for i in 0..100 {
            let entity = Entity::from_raw(i);
            let x = (i as f32).mul_add(2.0, -100.0);
            let bounds = Aabb::from_center_half_extents(Vec3::new(x, 0.0, 0.0), Vec3::splat(0.5));
            bvh.insert(entity, bounds);
        }

        let query_bounds =
            Aabb::from_min_max(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        let results = bvh.query_with_predicate(&|bounds| query_bounds.intersects(bounds));

        assert!(!results.is_empty());
        assert!(results.len() < 100);
    }
}
