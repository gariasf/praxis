# Editor Camera

Orbit-style camera controller for the editor viewport.

## Features

- **Orbit rotation**: Alt+LMB to rotate around target
- **Pan movement**: Alt+MMB to move view
- **Zoom**: Scroll wheel to adjust distance
- **Focus on selection**: F key to frame selected entities
- **Smooth interpolation**: Polished camera movements

## Controls

| Input | Action |
|-------|--------|
| Alt+LMB | Orbit rotation |
| Alt+MMB | Pan movement |
| Scroll | Zoom in/out |
| F | Focus on selection |

## Setup

```rust
use praxis_editor::{EditorCameraController, EditorCamera, update_editor_camera_system};

// Add controller resource
world.insert_resource(EditorCameraController::new());

// Create editor camera entity
world.spawn((
    PerspectiveCameraBundle::new(
        Vec3::new(0.0, 5.0, 10.0),
        70.0_f32.to_radians(),
        16.0 / 9.0,
    ),
    EditorCamera, // Marker component
));

// Add system
schedule.add_systems(update_editor_camera_system);
```

## Configuration

```rust
let mut controller = world.resource_mut::<EditorCameraController>();

// Sensitivity
controller.orbit_sensitivity = 0.01;
controller.pan_sensitivity = 0.02;
controller.zoom_sensitivity = 2.0;

// Distance limits
controller.min_distance = 1.0;
controller.max_distance = 500.0;

// Smoothing (higher = smoother)
controller.smoothness = 15.0;
```

## Programmatic Control

```rust
// Set target position
controller.set_target(Vec3::new(10.0, 0.0, 0.0));

// Set distance from target
controller.set_distance(15.0);

// Set orbit angles (yaw, pitch in radians)
controller.set_angles(0.0, PI / 4.0);

// Focus on a position
controller.focus_on(Vec3::new(5.0, 2.0, 3.0), Some(20.0));
```

## Example

```bash
cargo run --example editor_camera_demo
```

## See Also

- [crates/praxis_editor/EDITOR_CAMERA.md](../../crates/praxis_editor/EDITOR_CAMERA.md) - Full documentation
