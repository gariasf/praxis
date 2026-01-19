# Editor Camera

Dedicated orbit-style camera controller for the editor viewport, providing intuitive scene navigation separate from game cameras.

## Overview

The Editor Camera Controller uses an orbit camera model for natural scene editing. The camera rotates around a target point, providing smooth interpolated movement and focus-on-selection functionality.

## Features

- **Orbit Camera Model**: Camera orbits around a target point in space
- **Smooth Interpolation**: All movements smoothly interpolate for professional feel
- **Focus on Selection**: Frame selected entities in view with F key
- **Configurable Sensitivity**: Adjustable orbit, pan, and zoom speeds
- **Separate from Game Cameras**: Uses `EditorCamera` marker component
- **Distance Constraints**: Configurable min/max zoom distances
- **Pitch Clamping**: Prevents camera from flipping upside down

## Controls

### Mouse Controls

| Input | Action | Description |
|-------|--------|-------------|
| **Alt+LMB Drag** | Orbit | Rotate camera around target point |
| **Alt+MMB Drag** | Pan | Move camera and target together |
| **Scroll Wheel** | Zoom | Move closer to or farther from target |

### Keyboard Controls

| Key | Action | Description |
|-----|--------|-------------|
| **F** | Focus | Frame selected entities in view |

**Note**: Alt modifier required for mouse controls to avoid conflicts with other tools (gizmos, selection, etc.)

## Architecture

### EditorCameraController (Resource)

Manages camera state and processes input:

```rust
pub struct EditorCameraController {
    // Current state
    target: Vec3,      // Point camera orbits around
    distance: f32,     // Radius from target
    yaw: f32,          // Horizontal angle
    pitch: f32,        // Vertical angle
    
    // Desired state (for interpolation)
    desired_target: Vec3,
    desired_distance: f32,
    // ...
    
    // Configuration
    orbit_sensitivity: f32,
    pan_sensitivity: f32,
    zoom_sensitivity: f32,
    smoothness: f32,
}
```

### EditorCamera (Component)

Marker component identifying editor camera entities:

```rust
#[derive(Component)]
pub struct EditorCamera;
```

Enables queries like:
```rust
// Query only editor camera
Query<&Transform, With<EditorCamera>>

// Query only game cameras
Query<&Transform, Without<EditorCamera>>
```

### update_editor_camera_system (System)

ECS system that updates camera transform based on controller state. Add to your schedule:

```rust
use praxis_editor::update_editor_camera_system;

schedule.add_systems(update_editor_camera_system);
```

## Usage

### Basic Setup

```rust
use praxis_editor::{EditorCameraController, EditorCamera, update_editor_camera_system};
use praxis_scene::PerspectiveCameraBundle;
use praxis_math::Vec3;

// Create controller resource
world.insert_resource(EditorCameraController::new());

// Spawn editor camera entity
world.spawn((
    PerspectiveCameraBundle::new(
        Vec3::new(0.0, 5.0, 10.0),  // Initial position
        70.0_f32.to_radians(),       // FOV
        16.0 / 9.0,                  // Aspect ratio
    ),
    EditorCamera,  // Marker component
));

// Add system to schedule
schedule.add_systems(update_editor_camera_system);
```

### Programmatic Control

```rust
let mut controller = world.resource_mut::<EditorCameraController>();

// Set target position
controller.set_target(Vec3::new(10.0, 0.0, 0.0));

// Set distance from target
controller.set_distance(20.0);

// Set angles (yaw, pitch in radians)
controller.set_angles(0.0, -0.5);

// Focus on specific position
controller.focus_on(Vec3::new(5.0, 2.0, 3.0), Some(15.0));
```

### Focus on Selection

Automatically frame selected entities:

```rust
// Press F to focus on selection
if input.key_just_pressed(KeyCode::F) {
    controller.focus_on_selection(&selection, &transform_query);
}
```

The focus algorithm:
1. Computes bounding box of all selected entities
2. Calculates center point and size
3. Sets distance to `size × 2.0` (fits objects in view)
4. Smoothly transitions to new view

## Configuration

### Default Settings

```rust
EditorCameraController {
    target: Vec3::ZERO,
    distance: 10.0,
    yaw: 0.0,
    pitch: -0.3,
    
    orbit_sensitivity: 0.005,
    pan_sensitivity: 0.01,
    zoom_sensitivity: 0.1,
    
    min_distance: 1.0,
    max_distance: 1000.0,
    min_pitch: -1.5,   // ~-85 degrees
    max_pitch: 1.5,    // ~+85 degrees
    
    smoothness: 10.0,  // Higher = smoother/slower
}
```

### Customization

```rust
let mut controller = EditorCameraController::new();

// Adjust sensitivities
controller.set_orbit_sensitivity(0.003);  // Slower orbit
controller.set_pan_sensitivity(0.02);     // Faster pan
controller.set_zoom_sensitivity(0.2);     // Faster zoom

// Change distance constraints
controller.set_min_distance(0.5);
controller.set_max_distance(500.0);

// Adjust smoothness
controller.set_smoothness(15.0);  // Very smooth
controller.set_smoothness(5.0);   // More responsive
```

## Smooth Interpolation

The controller maintains two sets of state:
- **Current**: Actual camera state (what you see)
- **Desired**: Target state (where camera wants to be)

Each frame, current state moves toward desired state:
```rust
let t = (smoothness * delta_time).min(1.0);
current = current.lerp(desired, t);
```

**Smoothness values**:
- **1-5**: Snappy, responsive
- **5-15**: Balanced, professional
- **15-30**: Very smooth, cinematic

## Camera Model

### Position Calculation

Camera position computed from orbital parameters:

```rust
// Convert angles to rotation
let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

// Offset from target
let offset = rotation * Vec3::new(0.0, 0.0, distance);

// Final position
let position = target + offset;
```

### Coordinate System

- **Target**: World-space point camera orbits around
- **Distance**: Radius of orbit (always positive)
- **Yaw**: Rotation around Y-axis (horizontal)
- **Pitch**: Rotation around X-axis (vertical, clamped to prevent flipping)

## Advanced Usage

### Camera Presets

Implement preset views:

```rust
fn apply_top_view(controller: &mut EditorCameraController, target: Vec3) {
    controller.set_target(target);
    controller.set_angles(0.0, -std::f32::consts::FRAC_PI_2);  // Looking down
    controller.set_distance(20.0);
}

fn apply_front_view(controller: &mut EditorCameraController, target: Vec3) {
    controller.set_target(target);
    controller.set_angles(0.0, 0.0);  // Looking along +Z
    controller.set_distance(20.0);
}
```

### State Serialization

Save and restore camera state:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CameraState {
    target: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

// Save
let state = CameraState {
    target: controller.target(),
    distance: controller.distance(),
    yaw: controller.angles().0,
    pitch: controller.angles().1,
};
let json = serde_json::to_string(&state)?;

// Load
let state: CameraState = serde_json::from_str(&json)?;
controller.set_target(state.target);
controller.set_distance(state.distance);
controller.set_angles(state.yaw, state.pitch);
```

## Integration

### With Selection System

```rust
if input.key_just_pressed(KeyCode::F) {
    camera_controller.focus_on_selection(&selection, &transform_query);
}
```

### With Scene View

Only process input when viewport is focused:

```rust
if scene_view.is_focused() {
    camera_controller.process_input(&input, delta_time);
}
camera_controller.update(delta_time);
```

### With Gizmos

Gizmos take priority over camera input:

```rust
if gizmo_system.is_interacting() {
    // Don't process camera input
} else if input.key_pressed(KeyCode::Alt) {
    camera_controller.process_input(&input, delta_time);
}
```

## Performance

- **Per-frame cost**: ~0.01ms (lightweight)
- **State updates**: Only when input received
- **Transform calculation**: Simple matrix math

## Troubleshooting

### Camera Not Moving
- Verify `update_editor_camera_system` is in schedule
- Check Alt key is held for orbit/pan
- Ensure camera has `EditorCamera` component
- Verify input state is properly passed

### Jittery Movement
- Increase `smoothness` value
- Ensure delta_time is stable
- Check for conflicting camera systems

### Can't Orbit Vertically
- Check pitch constraints (`min_pitch`, `max_pitch`)
- Verify mouse input is being processed

### Focus Not Working
- Verify entities have `GlobalTransform` component
- Check selection is not empty
- Ensure focus system/key handler is called

## Examples

See `examples/editor_camera_demo.rs` for a complete working example.

## Technical Details

For implementation details, see:
- [crates/praxis_editor/EDITOR_CAMERA.md](../../crates/praxis_editor/EDITOR_CAMERA.md) - Complete implementation documentation
- Transform computation details
- Interpolation algorithms
- Pan movement in camera space

## See Also

- [Scene View Panel](panels.md#scene-view) - Viewport where camera operates
- [Selection System](selection-system.md) - Focus on selected entities
- [Gizmos](gizmos.md) - Transform manipulation in viewport
