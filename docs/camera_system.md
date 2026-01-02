# Camera System

The Praxis ECS camera system provides a flexible and ergonomic way to work with cameras in 3D scenes.

## Components

### Camera

The `Camera` component marks an entity as a camera and controls its active state and rendering priority.

```rust
pub struct Camera {
    pub is_active: bool,
    pub priority: i32,
}
```

**Features:**
- Active/inactive state control
- Priority-based rendering order (higher priority renders last)
- Methods: `new()`, `with_priority()`, `activate()`, `deactivate()`, `is_active()`

### PerspectiveProjection

Defines perspective projection parameters for 3D rendering with depth perception.

```rust
pub struct PerspectiveProjection {
    pub fov: f32,           // Field of view in radians
    pub aspect_ratio: f32,  // Width / height
    pub near: f32,          // Near clipping plane
    pub far: f32,           // Far clipping plane
}
```

**Features:**
- `compute_matrix()` - Generates the projection matrix
- `set_aspect_ratio()` - Update aspect ratio (e.g., on window resize)
- Default: 70° FOV, 16:9 aspect, 0.1-1000.0 depth range

### OrthographicProjection

Defines orthographic projection for 2D games, UI, or isometric views.

```rust
pub struct OrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}
```

**Features:**
- `compute_matrix()` - Generates the projection matrix
- `from_size()` - Create centered projection from width/height
- `set_bounds()` - Update projection bounds
- Default: 20x20 centered view, 0.1-1000.0 depth range

### CameraMatrices

Automatically computed and cached camera matrices.

```rust
pub struct CameraMatrices {
    pub view: Mat4,                // World to view space
    pub projection: Mat4,          // View to clip space
    pub view_projection: Mat4,     // Combined transformation
}
```

**Note:** This component is automatically updated by camera systems. You typically don't need to create or modify it manually.

## Systems

### update_perspective_cameras

Updates view and projection matrices for all active perspective cameras.

**When to use:** Add to your schedule to keep perspective camera matrices up to date.

```rust
schedule.add_systems(update_perspective_cameras);
```

### update_orthographic_cameras

Updates view and projection matrices for all active orthographic cameras.

**When to use:** Add to your schedule to keep orthographic camera matrices up to date.

```rust
schedule.add_systems(update_orthographic_cameras);
```

## Bundles

### PerspectiveCameraBundle

Complete bundle for spawning a perspective camera.

```rust
world.spawn(PerspectiveCameraBundle::new(
    Vec3::new(0.0, 5.0, 10.0),  // position
    70.0_f32.to_radians(),       // FOV
    16.0 / 9.0,                  // aspect ratio
));
```

**Includes:**
- Camera
- Transform
- GlobalTransform
- PerspectiveProjection
- CameraMatrices

### OrthographicCameraBundle

Complete bundle for spawning an orthographic camera.

```rust
world.spawn(OrthographicCameraBundle::new(
    Vec3::new(0.0, 10.0, 0.0),  // position
    20.0,                        // width
    10.0,                        // height
));
```

**Includes:**
- Camera
- Transform
- GlobalTransform
- OrthographicProjection
- CameraMatrices

## Query Helpers

The `camera` module provides convenient query types and helper functions.

### ActivePerspectiveCameras

Query data for active perspective cameras:

```rust
fn render_system(cameras: Query<camera::ActivePerspectiveCameras>) {
    for (entity, camera, transform, projection, matrices) in cameras.iter() {
        // Render with this camera
    }
}
```

### ActiveOrthographicCameras

Query data for active orthographic cameras:

```rust
fn render_system(cameras: Query<camera::ActiveOrthographicCameras>) {
    for (entity, camera, transform, projection, matrices) in cameras.iter() {
        // Render with this camera
    }
}
```

### Helper Functions

#### primary_perspective_camera

Gets the highest priority active perspective camera:

```rust
if let Some((entity, camera, matrices)) = camera::primary_perspective_camera(&cameras) {
    // Use primary camera for rendering
}
```

#### primary_orthographic_camera

Gets the highest priority active orthographic camera.

#### sorted_perspective_cameras

Gets all active perspective cameras sorted by priority (low to high):

```rust
let sorted = camera::sorted_perspective_cameras(&cameras);
for (entity, camera, matrices) in sorted {
    // Render in priority order
}
```

#### sorted_orthographic_cameras

Gets all active orthographic cameras sorted by priority.

## Usage Examples

### Basic Perspective Camera

```rust
use praxis_ecs::{World, PerspectiveCameraBundle, Schedule};
use praxis_ecs::systems::{update_perspective_cameras};
use praxis_math::Vec3;

let mut world = World::new();
let mut schedule = Schedule::default();
schedule.add_systems(update_perspective_cameras);

// Create camera
world.spawn(PerspectiveCameraBundle::new(
    Vec3::new(0.0, 5.0, 10.0),
    70.0_f32.to_radians(),
    16.0 / 9.0,
));

// Update camera matrices
world.inner_mut().run_schedule(&mut schedule);
```

### Multiple Cameras with Priorities

```rust
// Main camera (default priority 0)
let main_camera = world.spawn(PerspectiveCameraBundle::new(
    Vec3::new(0.0, 5.0, 10.0),
    70.0_f32.to_radians(),
    16.0 / 9.0,
));

// Minimap camera (higher priority)
let mut minimap_bundle = OrthographicCameraBundle::new(
    Vec3::new(0.0, 100.0, 0.0),
    50.0,
    50.0,
);
minimap_bundle.camera.priority = 10;
let minimap_camera = world.spawn(minimap_bundle);
```

### Camera Activation/Deactivation

```rust
if let Some(mut camera) = world.inner_mut().get_mut::<Camera>(camera_entity) {
    camera.deactivate();  // Temporarily disable camera
    
    // Later...
    camera.activate();    // Re-enable camera
}
```

### Accessing Camera Matrices for Rendering

```rust
use praxis_ecs::{Query, camera};

fn render_system(cameras: Query<camera::ActivePerspectiveCameras>) {
    if let Some((entity, camera, matrices)) = camera::primary_perspective_camera(&cameras) {
        // Use matrices.view_projection for rendering
        let view_proj = matrices.view_projection;
        
        // Pass to renderer...
    }
}
```

### Camera with Parent Transform

Cameras can have parent entities, and their view matrix will automatically use the GlobalTransform:

```rust
// Create parent object
let parent = world.spawn((
    Transform::from_xyz(10.0, 0.0, 0.0),
    GlobalTransform::default(),
));

// Camera attached to parent
let camera = world.spawn((
    Camera::default(),
    Transform::from_xyz(0.0, 5.0, 5.0),  // Local offset
    GlobalTransform::default(),
    PerspectiveProjection::default(),
    CameraMatrices::default(),
    Parent(parent),
));

// Camera will move with parent
```

### Updating Aspect Ratio on Window Resize

```rust
fn handle_resize(
    mut cameras: Query<&mut PerspectiveProjection>,
    new_width: f32,
    new_height: f32,
) {
    for mut projection in cameras.iter_mut() {
        projection.set_aspect_ratio(new_width / new_height);
    }
}
```

## Architecture Notes

- Camera systems use change detection to only update matrices when necessary
- View matrix is computed as the inverse of the camera's transform matrix
- Cameras can be part of the transform hierarchy (with Parent/Children components)
- Inactive cameras are skipped by the update systems
- Priority determines rendering order: lower priority renders first, higher priority renders last
- Multiple projection types can coexist in the same scene

## See Also

- [Transform System](transform_system.md) - For camera positioning and hierarchy
- [Mesh System](mesh_system.md) - For rendering with cameras
- [ECS Overview](../README.md) - General ECS architecture
