# Editor Camera Guide

## Overview

The Editor Camera Controller provides intuitive orbit-based camera controls for scene editing, completely separate from game cameras. It features smooth interpolated movement, focus-on-selection, and configurable sensitivity.

## Design Philosophy

The editor camera is designed around the **orbit camera** paradigm, where:
- Camera orbits around a **target point** in space
- **Distance** from target is adjustable (zoom)
- **Yaw** and **pitch** angles control viewing direction
- All movements are **smoothly interpolated** for professional feel

This differs from game cameras (FPS, third-person) which are designed for gameplay, not editing.

## Core Components

### `EditorCameraController` (Resource)

Manages camera state and processes input:

```rust
pub struct EditorCameraController {
    // Current state
    target: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
    
    // Desired state (for interpolation)
    desired_target: Vec3,
    desired_distance: f32,
    desired_yaw: f32,
    desired_pitch: f32,
    
    // Configuration
    orbit_sensitivity: f32,
    pan_sensitivity: f32,
    zoom_sensitivity: f32,
    min_distance: f32,
    max_distance: f32,
    min_pitch: f32,
    max_pitch: f32,
    smoothness: f32,
    
    // Interaction state
    is_orbiting: bool,
    is_panning: bool,
}
```

### `EditorCamera` (Component)

Marker component distinguishing editor cameras from game cameras:

```rust
#[derive(Component)]
pub struct EditorCamera;
```

This enables queries like:
```rust
// Query only editor camera
Query<&mut Transform, With<EditorCamera>>

// Query only game cameras
Query<&mut Transform, Without<EditorCamera>>
```

## Controls

### Mouse Controls

| Input | Action | Description |
|-------|--------|-------------|
| **Alt+LMB** + Drag | Orbit | Rotate camera around target |
| **Alt+MMB** + Drag | Pan | Move camera and target together |
| **Scroll Wheel** | Zoom | Move closer/farther from target |

### Keyboard Shortcuts

| Key | Action | Description |
|-----|--------|-------------|
| **F** | Focus | Frame selected entities in view |

### Modifier Keys

| Modifier | Effect |
|----------|--------|
| **Alt** | Required for orbit/pan (prevents conflict with other tools) |
| **Shift** | (Future) Faster movement |
| **Ctrl** | (Future) Slower movement |

## Basic Usage

### Setup

```rust
use praxis_editor::{EditorCameraController, EditorCamera, update_editor_camera_system};
use praxis_ecs::{World, Schedule};
use praxis_scene::PerspectiveCameraBundle;
use praxis_math::Vec3;

// Create editor camera controller
let mut world = World::new();
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
let mut schedule = Schedule::default();
schedule.add_systems(update_editor_camera_system);
```

### Update Loop

```rust
// Each frame
schedule.run(world.inner_mut());
```

The system automatically:
1. Processes input (orbit, pan, zoom, focus)
2. Updates interpolation state
3. Computes camera position from target + angles
4. Updates camera transform

## Camera Model

### Position Calculation

Camera position is computed from orbital parameters:

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
- **Pitch**: Rotation around X-axis (vertical, clamped)
- **Offset**: Local-space vector from target to camera

## Configuration

### Default Settings

```rust
EditorCameraController {
    target: Vec3::ZERO,
    distance: 10.0,
    yaw: 0.0,
    pitch: -0.3,  // Slight downward angle
    
    orbit_sensitivity: 0.005,
    pan_sensitivity: 0.01,
    zoom_sensitivity: 0.1,
    
    min_distance: 1.0,
    max_distance: 1000.0,
    min_pitch: -1.5,   // ~-85 degrees
    max_pitch: 1.5,    // ~+85 degrees
    
    smoothness: 10.0,  // Higher = slower/smoother
    
    // ... (internal state)
}
```

### Customizing Settings

```rust
let mut controller = EditorCameraController::new();

// Adjust sensitivities
controller.set_orbit_sensitivity(0.003);  // Slower orbit
controller.set_pan_sensitivity(0.02);     // Faster pan
controller.set_zoom_sensitivity(0.2);     // Faster zoom

// Change distance constraints
controller.set_min_distance(0.5);
controller.set_max_distance(500.0);

// Adjust pitch limits
controller.set_min_pitch(-1.4);  // Less downward
controller.set_max_pitch(1.4);   // Less upward

// Adjust smoothness (higher = smoother but slower)
controller.set_smoothness(15.0);  // Very smooth
controller.set_smoothness(5.0);   // More responsive
```

## Smooth Interpolation

All camera movements use smooth interpolation to avoid jarring changes:

### Dual-State System

The controller maintains two sets of state:
- **Current**: Actual camera state (what you see)
- **Desired**: Target state (where camera wants to be)

### Interpolation Formula

Each frame, current state moves toward desired state:

```rust
let t = (smoothness * delta_time).min(1.0);

target = target.lerp(desired_target, t);
distance = distance + (desired_distance - distance) * t;
yaw = yaw + (desired_yaw - yaw) * t;
pitch = pitch + (desired_pitch - pitch) * t;
```

### Adjusting Smoothness

- **Lower smoothness** (1-5): Snappy, responsive
- **Medium smoothness** (5-15): Balanced, professional
- **Higher smoothness** (15-30): Very smooth, cinematic

## Focus on Selection

### Basic Focus

Focus camera on a world position:

```rust
// Focus on origin
controller.focus_on(Vec3::ZERO, None);

// Focus with specific distance
controller.focus_on(Vec3::new(10.0, 0.0, 5.0), Some(15.0));
```

### Auto-Frame Selection

Focus on selected entities with automatic distance calculation:

```rust
// In your system
fn focus_on_selection_system(
    input: Res<InputState>,
    mut controller: ResMut<EditorCameraController>,
    selection: Res<SelectionSystem>,
    transform_query: Query<&GlobalTransform>,
) {
    // Press F to focus
    if input.key_just_pressed(KeyCode::F) {
        controller.focus_on_selection(&selection, &transform_query);
    }
}
```

### Focus Algorithm

The focus system:
1. Computes bounding box of all selected entities
2. Finds center point
3. Calculates size (diagonal length)
4. Sets distance to `size × 2.0` (fits objects in view)
5. Smoothly transitions to new view

```rust
// Simplified algorithm
let mut min = Vec3::splat(f32::MAX);
let mut max = Vec3::splat(f32::MIN);

for entity in selection.selected_entities() {
    let pos = transform_query.get(entity).translation();
    min = min.min(pos);
    max = max.max(pos);
}

let center = (min + max) * 0.5;
let size = (max - min).length();
let distance = if size > 0.1 { size * 2.0 } else { 5.0 };

controller.focus_on(center, Some(distance));
```

## Advanced Usage

### Manual Control

Programmatically control camera:

```rust
// Set target position
controller.set_target(Vec3::new(10.0, 0.0, 0.0));

// Set distance
controller.set_distance(20.0);

// Set angles (radians)
controller.set_angles(
    0.0,      // Yaw: 0 = looking along +Z
    -0.5,     // Pitch: negative = looking down
);

// Get current state
let target = controller.target();
let distance = controller.distance();
let (yaw, pitch) = controller.angles();
```

### Camera Presets

Implement preset views:

```rust
enum CameraPreset {
    Top,
    Front,
    Side,
    Perspective,
}

fn apply_preset(controller: &mut EditorCameraController, preset: CameraPreset, target: Vec3) {
    match preset {
        CameraPreset::Top => {
            controller.set_target(target);
            controller.set_angles(0.0, -std::f32::consts::FRAC_PI_2);  // Looking down
            controller.set_distance(20.0);
        }
        CameraPreset::Front => {
            controller.set_target(target);
            controller.set_angles(0.0, 0.0);  // Looking along +Z
            controller.set_distance(20.0);
        }
        CameraPreset::Side => {
            controller.set_target(target);
            controller.set_angles(std::f32::consts::FRAC_PI_2, 0.0);  // Looking along +X
            controller.set_distance(20.0);
        }
        CameraPreset::Perspective => {
            controller.set_target(target);
            controller.set_angles(0.785, -0.6);  // 45° angles
            controller.set_distance(15.0);
        }
    }
}
```

### Camera Animation

Animate camera to specific views:

```rust
struct CameraAnimation {
    start_target: Vec3,
    end_target: Vec3,
    start_distance: f32,
    end_distance: f32,
    start_yaw: f32,
    end_yaw: f32,
    start_pitch: f32,
    end_pitch: f32,
    duration: f32,
    elapsed: f32,
}

impl CameraAnimation {
    fn update(&mut self, controller: &mut EditorCameraController, delta: f32) -> bool {
        self.elapsed += delta;
        let t = (self.elapsed / self.duration).min(1.0);
        
        // Smooth ease-in-out
        let t = t * t * (3.0 - 2.0 * t);
        
        let target = self.start_target.lerp(self.end_target, t);
        let distance = self.start_distance + (self.end_distance - self.start_distance) * t;
        let yaw = self.start_yaw + (self.end_yaw - self.start_yaw) * t;
        let pitch = self.start_pitch + (self.end_pitch - self.start_pitch) * t;
        
        controller.set_target(target);
        controller.set_distance(distance);
        controller.set_angles(yaw, pitch);
        
        self.elapsed >= self.duration  // Return true when complete
    }
}
```

### State Serialization

Save and load camera state:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CameraState {
    target: Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

impl From<&EditorCameraController> for CameraState {
    fn from(controller: &EditorCameraController) -> Self {
        Self {
            target: controller.target(),
            distance: controller.distance(),
            yaw: controller.angles().0,
            pitch: controller.angles().1,
        }
    }
}

// Save camera state
let state = CameraState::from(&controller);
let json = serde_json::to_string(&state)?;
std::fs::write("camera_state.json", json)?;

// Load camera state
let json = std::fs::read_to_string("camera_state.json")?;
let state: CameraState = serde_json::from_str(&json)?;
controller.set_target(state.target);
controller.set_distance(state.distance);
controller.set_angles(state.yaw, state.pitch);
```

## Integration with Other Systems

### With Selection System

Focus on selection automatically:

```rust
// Add to your editor update
if input.key_just_pressed(KeyCode::F) {
    camera_controller.focus_on_selection(&selection, &transform_query);
}
```

### With Scene View

Only process camera input when scene view is focused:

```rust
if scene_view.is_focused() {
    camera_controller.process_input(&input, delta_time);
}
camera_controller.update(delta_time);
```

### With Gizmos

Switch between camera and gizmo interaction:

```rust
// Gizmos take priority
if gizmo_system.is_interacting() {
    // Don't process camera input
} else if input.key_pressed(KeyCode::Alt) {
    camera_controller.process_input(&input, delta_time);
}
```

## Comparison to Game Cameras

| Feature | Editor Camera | FPS Camera | Third-Person Camera |
|---------|--------------|------------|---------------------|
| **Control** | Orbit around target | Direct movement | Follow character |
| **Purpose** | Scene editing | Gameplay | Gameplay |
| **Marker** | `EditorCamera` | None | `GameCamera` |
| **Input** | Alt+Mouse | Mouse look | Auto-follow |
| **Movement** | Pan/orbit | WASD | Character-driven |
| **Smoothing** | Always smooth | Optional | Always smooth |
| **Focus** | F key | N/A | Auto-center |

## Performance Considerations

### Update Frequency

The camera system is lightweight:
- **Per-frame cost**: ~0.01ms
- **State updates**: Only when input received
- **Transform calculation**: Simple matrix math

### Optimization Tips

1. **Skip updates when inactive**: Don't update if scene view not focused
2. **Lower smoothness**: Reduces computational lag
3. **Batch state changes**: Set multiple properties before update

## Troubleshooting

### Camera Not Moving

**Problem**: Camera doesn't respond to input

**Solutions**:
- Verify `update_editor_camera_system` is in schedule
- Check input state is properly passed
- Ensure Alt key is held for orbit/pan
- Verify camera has `EditorCamera` component

### Jittery Movement

**Problem**: Camera movement is not smooth

**Solutions**:
- Increase `smoothness` value
- Ensure delta_time is stable
- Check for conflicting camera systems
- Verify update called every frame

### Can't Orbit Vertically

**Problem**: Camera won't look up/down

**Solutions**:
- Check pitch constraints (`min_pitch`, `max_pitch`)
- Verify mouse input is being processed
- Ensure sensitivity is not too low

### Focus Not Working

**Problem**: F key doesn't focus on selection

**Solutions**:
- Verify entities have `GlobalTransform` component
- Check selection is not empty
- Ensure focus system is called
- Add debug logging to focus algorithm

## Complete Example

```rust
use praxis_editor::*;
use praxis_ecs::*;
use praxis_scene::*;
use praxis_input::*;
use praxis_math::*;

fn main() {
    // Setup world and resources
    let mut world = World::new();
    world.insert_resource(EditorCameraController::new());
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(InputState::new());
    
    // Spawn editor camera
    world.spawn((
        PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 10.0),
            70.0_f32.to_radians(),
            16.0 / 9.0,
        ),
        EditorCamera,
    ));
    
    // Spawn some objects to look at
    for x in -5..=5 {
        for z in -5..=5 {
            world.spawn((
                Transform::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0),
                Selectable,
            ));
        }
    }
    
    // Setup systems
    let mut schedule = Schedule::default();
    schedule.add_systems((
        handle_input_system,
        update_editor_camera_system,
        handle_focus_key_system,
    ).chain());
    
    // Main loop
    loop {
        schedule.run(world.inner_mut());
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn handle_focus_key_system(
    input: Res<InputState>,
    mut controller: ResMut<EditorCameraController>,
    selection: Res<SelectionSystem>,
    transform_query: Query<&GlobalTransform>,
) {
    if input.key_just_pressed(KeyCode::F) {
        controller.focus_on_selection(&selection, &transform_query);
    }
}
```

## See Also

- [Selection System](selection-system.md)
- [Scene View Panel](panels.md)
- [Gizmos Guide](gizmos.md)
- [Camera System Guide](../guides/camera_system.md)
