# Entity Bounds Integration for Selection System

## Summary

Successfully implemented entity bounds integration for the selection system in `praxis_editor/src/selection.rs`, replacing the hard-coded pick radius with actual mesh bounding box queries for accurate 3D picking.

## Changes Made

### 1. Updated Dependencies (`crates/praxis_editor/Cargo.toml`)

Added `praxis_spatial` dependency to access the `Aabb` type for ray-box intersection tests:

```toml
praxis_spatial = { path = "../praxis_spatial" }
```

### 2. Enhanced `raycast_pick` Method (`crates/praxis_editor/src/selection.rs`)

**Previous Implementation:**
- Used hard-coded sphere-based picking with radius = 1.0
- No consideration of actual mesh geometry
- Line 541 contained: `let pick_radius = 1.0; // TODO: Use actual entity bounds`

**New Implementation:**

The method now queries entities in the following priority order:

1. **BoundingBox Component** (Primary): Entities with explicit `BoundingBox` component
   - Transforms the bounding box to world space using the entity's global transform matrix
   - Uses `Aabb::transform()` to apply scale, rotation, and translation
   
2. **Mesh Component** (Secondary): Entities with `Mesh` component
   - Computes AABB from mesh vertices using `Aabb::from_points()`
   - Transforms the computed bounds to world space
   
3. **Fallback** (Tertiary): Entities without bounds information
   - Uses simple sphere test with 1.0 radius (same as before)
   - Ensures backward compatibility with entities that lack bounds data

**Ray-AABB Intersection:**
- Utilizes `Aabb::ray_intersection_distance()` from `praxis_spatial`
- Returns accurate intersection distance for closest-entity selection
- Properly handles transformed bounding boxes (scaled, rotated, translated)

### 3. Updated Method Signature

**Old Signature:**
```rust
pub fn raycast_pick(
    &self,
    screen_pos: Vec2,
    viewport_size: Vec2,
    camera_transform: &Transform,
    camera_matrices: &CameraMatrices,
    selectable_query: &Query<(Entity, &GlobalTransform), With<Selectable>>,
) -> Option<Entity>
```

**New Signature:**
```rust
pub fn raycast_pick(
    &self,
    screen_pos: Vec2,
    viewport_size: Vec2,
    camera_transform: &Transform,
    camera_matrices: &CameraMatrices,
    selectable_query: &Query<(Entity, &GlobalTransform), With<Selectable>>,
    bounds_query: &Query<&BoundingBox>,      // NEW
    mesh_query: &Query<&Mesh>,                // NEW
) -> Option<Entity>
```

### 4. Added Imports

```rust
use praxis_ecs::{BoundingBox, CameraMatrices, GlobalTransform, Mesh, Transform};
use praxis_spatial::Aabb;
```

### 5. Enhanced Documentation

- Updated module-level documentation to explain the raycast picking strategy
- Added detailed method documentation with examples
- Documented the three-tier picking approach (BoundingBox → Mesh → Fallback)
- Added usage example showing how to prepare and pass queries

## Technical Details

### Ray-Box Intersection Algorithm

The implementation uses the efficient slab method from `praxis_spatial::Aabb`:
- Computes intersection distances with all 6 box planes
- Returns the closest intersection point within `max_distance`
- Handles edge cases (ray parallel to planes, ray origin inside box)

### Transform Handling

Bounding boxes are properly transformed to world space:
```rust
let world_matrix = global_transform.matrix;
Aabb::from_min_max(bounding_box.min, bounding_box.max).transform(&world_matrix)
```

This ensures accurate picking for:
- Non-uniform scales (e.g., [2.0, 1.0, 0.5])
- Arbitrary rotations
- Complex transform hierarchies

### Performance Considerations

1. **Query-based approach**: Leverages ECS queries for efficient component access
2. **Early rejection**: Entities without valid bounds are skipped
3. **Distance culling**: Only tests against entities closer than current closest
4. **Lazy computation**: Mesh bounds computed only when BoundingBox absent

## Breaking Changes

**API Change**: The `raycast_pick` method now requires two additional query parameters:
- `bounds_query: &Query<&BoundingBox>`
- `mesh_query: &Query<&Mesh>`

**Migration Guide for Callers:**

```rust
// Before:
let picked = selection.raycast_pick(
    screen_pos,
    viewport_size,
    camera_transform,
    camera_matrices,
    &selectable_query,
);

// After:
let bounds_query = world.query::<&BoundingBox>();
let mesh_query = world.query::<&Mesh>();

let picked = selection.raycast_pick(
    screen_pos,
    viewport_size,
    camera_transform,
    camera_matrices,
    &selectable_query,
    &bounds_query,  // NEW
    &mesh_query,    // NEW
);
```

## Benefits

1. **Accuracy**: Precise picking based on actual geometry, not approximations
2. **Flexibility**: Supports both explicit bounds and computed mesh bounds
3. **Compatibility**: Fallback ensures entities without bounds still work
4. **Performance**: Efficient ray-AABB tests using industry-standard algorithm
5. **Robustness**: Handles complex transforms (scale, rotation, translation)

## Testing

No existing tests required updates as `raycast_pick` was not previously tested. The method signature change is breaking but the implementation maintains backward compatibility through the fallback mechanism.

## Files Modified

1. `crates/praxis_editor/Cargo.toml` - Added `praxis_spatial` dependency
2. `crates/praxis_editor/src/selection.rs` - Updated `raycast_pick` implementation and documentation

## Related Components

- `praxis_spatial::Aabb` - Provides ray-box intersection tests
- `praxis_ecs::BoundingBox` - Component for explicit entity bounds
- `praxis_ecs::Mesh` - Component with vertex data for bounds computation
- `praxis_ecs::GlobalTransform` - Used for transforming bounds to world space
