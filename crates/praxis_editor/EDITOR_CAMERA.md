# Editor Camera Controller

The editor camera controller provides a dedicated orbit-style camera for the editor viewport, separate from game cameras. It features smooth interpolated movement, focus on selection, and intuitive controls.

## Features

- **Orbit rotation**: Rotate around a target point using Alt+LMB
- **Pan movement**: Move the camera view using Alt+MMB
- **Zoom**: Adjust distance from target using scroll wheel
- **Focus on selection**: Frame selected entities in view with F key
- **Smooth interpolation**: All movements smoothly interpolate to target state
- **Separate from game cameras**: Uses EditorCamera marker to distinguish from game cameras

## Architecture

The editor camera system consists of:

1. **`EditorCameraController`** (Resource): Manages camera state and processes input
2. **`EditorCamera`** (Component): Marker component identifying the editor camera entity
3. **`update_editor_camera_system`** (System): Updates camera transform based on controller state

## Setup

### 1. Add Resources

```rust
use praxis_editor::{EditorCameraController, SelectionSystem};
use praxis_ecs::World;

let mut world = World::new();
world.insert_resource(EditorCameraController::new());
world.insert_resource(SelectionSystem::new());
```

### 2. Create Editor Camera Entity

```rust
use praxis_editor::EditorCamera;
use praxis_ecs::PerspectiveCameraBundle;
use praxis_math::Vec3;

world.spawn((
    PerspectiveCameraBundle::new(
        Vec3::new(0.0, 5.0, 10.0),
        70.0_f32.to_radians(),
        16.0 / 9.0,
    ),
    EditorCamera, // Marker component
));
```

### 3. Add System to Schedule

```rust
use praxis_editor::update_editor_camera_system;
use praxis_ecs::Schedule;

let mut schedule = Schedule::default();
schedule.add_systems(update_editor_camera_system);
```

## Controls

### Mouse Controls

- **Alt+LMB**: Orbit rotation - drag to rotate camera around target
- **Alt+MMB**: Pan movement - drag to move camera and target
- **Scroll Wheel**: Zoom - scroll to move closer/farther from target

### Keyboard Controls

- **F**: Focus on selection - frames all selected entities in view

## Usage Examples

### Basic Setup

```rust
use praxis_editor::{EditorCameraController, EditorCamera, update_editor_camera_system};
use praxis_ecs::{World, Schedule, PerspectiveCameraBundle};
use praxis_math::Vec3;

fn setup_editor_camera() -> (World, Schedule) {
    let mut world = World::new();
    
    // Add controller resource
    world.insert_resource(EditorCameraController::new());
    
    // Create editor camera
    world.spawn((
        PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 10.0),
            70.0_f32.to_radians(),
            16.0 / 9.0,
        ),
        EditorCamera,
    ));
    
    // Add system to schedule
    let mut schedule = Schedule::default();
    schedule.add_systems(update_editor_camera_system);
    
    (world, schedule)
}
```

### Programmatic Control

```rust
use praxis_editor::EditorCameraController;
use praxis_math::Vec3;

fn control_camera(controller: &mut EditorCameraController) {
    // Set target position
    controller.set_target(Vec3::new(10.0, 0.0, 0.0));
    
    // Set distance from target
    controller.set_distance(15.0);
    
    // Set orbit angles (yaw, pitch in radians)
    controller.set_angles(0.0, std::f32::consts::FRAC_PI_4);
    
    // Focus on a specific position
    controller.focus_on(Vec3::new(5.0, 2.0, 3.0), Some(20.0));
}
```

### Focus on Selection

```rust
use praxis_editor::{EditorCameraController, SelectionSystem};
use praxis_ecs::{Query, GlobalTransform};

fn focus_camera_on_selection(
    controller: &mut EditorCameraController,
    selection: &SelectionSystem,
    transform_query: &Query<&GlobalTransform>,
) {
    if !selection.is_empty() {
        controller.focus_on_selection(selection, transform_query);
    }
}
```

## Configuration

The controller can be configured through its public fields (accessed through `ResMut<EditorCameraController>`):

```rust
use praxis_editor::EditorCameraController;

fn configure_camera(controller: &mut EditorCameraController) {
    // Adjust sensitivity
    controller.orbit_sensitivity = 0.01;  // Default: 0.005
    controller.pan_sensitivity = 0.02;    // Default: 0.01
    controller.zoom_sensitivity = 2.0;    // Default: 1.0
    
    // Adjust constraints
    controller.min_distance = 1.0;        // Default: 0.5
    controller.max_distance = 500.0;      // Default: 1000.0
    
    // Adjust smoothing
    controller.smoothness = 15.0;         // Default: 10.0 (higher = smoother)
}
```

## How It Works

### Orbit Camera Model

The editor camera uses an orbit camera model where:
- The camera orbits around a **target point**
- **Distance** from target determines zoom level
- **Yaw** (horizontal angle) and **pitch** (vertical angle) determine viewing direction
- All movements are **smoothly interpolated** for a polished feel

### Smooth Interpolation

The controller uses a dual-state system:
1. **Current state**: The actual camera position/rotation
2. **Desired state**: The target position/rotation

Each frame, the current state smoothly interpolates toward the desired state using the `smoothness` parameter. This provides:
- Smooth camera movements
- No jarring transitions
- Responsive but polished feel

### Focus on Selection

When pressing F (or calling `focus_on_selection`):
1. Computes bounding box of all selected entities
2. Calculates center point and size
3. Sets target to center
4. Sets distance based on size (to frame entities)
5. Smoothly interpolates to new position

## Integration with Editor

The editor camera is designed to work seamlessly with the editor:

### Separate from Game Cameras

The `EditorCamera` marker component ensures the editor camera is independent from game cameras:

```rust
// Editor camera (controlled by EditorCameraController)
world.spawn((
    PerspectiveCameraBundle::new(...),
    EditorCamera,  // <-- Marker component
));

// Game camera (not affected by editor controller)
world.spawn((
    PerspectiveCameraBundle::new(...),
    // No EditorCamera marker
));
```

### Selection Integration

The camera automatically responds to the F key to focus on selected entities:

```rust
// In update_editor_camera_system:
if input.is_key_just_pressed(KeyCode::KeyF) && !selection.is_empty() {
    controller.focus_on_selection(&selection, &transform_query);
}
```

### Input Integration

The system uses `InputState` from `praxis_input` to process mouse and keyboard input:
- Mouse delta for orbit and pan
- Scroll delta for zoom
- Key presses for focus

## Tips and Best Practices

1. **Smoothness**: Higher smoothness values (10-20) provide smoother motion but slower response
2. **Sensitivity**: Adjust sensitivity values based on your UI scale and user preference
3. **Distance limits**: Set appropriate min/max distance based on your scene scale
4. **Focus distance**: The auto-calculated focus distance is `size * 2.0`, which works well for most scenes

## Example

See `examples/editor_camera_demo.rs` for a complete working example demonstrating all features.

Run with:
```bash
cargo run --example editor_camera_demo
```

## Technical Details

### Transform Computation

The camera position is computed from orbit parameters:

```rust
// Compute rotation from yaw and pitch
let rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);

// Compute offset from target
let offset = rotation * Vec3::new(0.0, 0.0, distance);

// Final position
let position = target + offset;
```

### Pan Movement

Pan movement moves in camera space:
- Right vector: `rotation * Vec3::X`
- Up vector: `rotation * Vec3::Y`
- Movement scaled by distance for consistent feel

### Zoom Behavior

Zoom changes distance from target:
- Scroll up: Move closer (decrease distance)
- Scroll down: Move farther (increase distance)
- Clamped to min/max distance

## Future Enhancements

Potential future improvements:
- Camera presets (top, front, side views)
- Frame rate independent delta time
- Camera animation/tweening
- Mouse wheel pan (Shift+Scroll)
- Right-click pan (alternative to Alt+MMB)
- Camera state saving/loading
