# Configuration Reference

Configurable constants and settings in Praxis.

## Window Settings

Default window configuration in `praxis_window`:

| Setting | Default | Description |
|---------|---------|-------------|
| Width | 1920 | Initial window width |
| Height | 1080 | Initial window height |
| Title | "Praxis" | Window title |
| VSync | true | Vertical sync enabled |
| Resizable | true | Window can be resized |

## Graphics Settings

### Lighting Limits

| Constant | Value | Location |
|----------|-------|----------|
| MAX_DIRECTIONAL_LIGHTS | 8 | praxis_graphics |
| MAX_POINT_LIGHTS | 32 | praxis_graphics |

### Shadow Configuration

```rust
pub struct ShadowConfig {
    pub resolution: u32,        // Default: 2048
    pub cascade_count: usize,   // Default: 3
    pub cascade_distances: Vec<f32>, // [20.0, 100.0, 500.0]
    pub bias: f32,              // Default: 0.005
    pub pcf_samples: u32,       // Default: 4 (2x2)
}
```

### HDR Configuration

```rust
pub struct HdrConfig {
    pub tone_mapping_operator: ToneMappingOperator, // ACES
    pub exposure_mode: ExposureMode,  // Automatic
    pub gamma: f32,                   // 2.2
}
```

## Physics Settings

```rust
pub struct PhysicsConfig {
    pub timestep: f32,          // Default: 1.0/60.0 (60 Hz)
    pub gravity: Vec3,          // Default: (0, -9.81, 0)
    pub max_velocity: f32,      // Default: 100.0
}
```

## Audio Settings

| Setting | Default | Description |
|---------|---------|-------------|
| SPEED_OF_SOUND | 343.0 | Units per second |
| MAX_SOURCES | 256 | Simultaneous sources |

### AudioSource Defaults

```rust
AudioSource {
    volume: 1.0,
    spatial: false,
    looping: false,
    max_distance: 100.0,
    reference_distance: 1.0,
    doppler_enabled: false,
    doppler_scale: 1.0,
}
```

## Editor Settings

### Editor Camera

```rust
pub struct EditorCameraController {
    pub orbit_sensitivity: f32,  // Default: 0.005
    pub pan_sensitivity: f32,    // Default: 0.01
    pub zoom_sensitivity: f32,   // Default: 1.0
    pub min_distance: f32,       // Default: 0.5
    pub max_distance: f32,       // Default: 1000.0
    pub smoothness: f32,         // Default: 10.0
}
```

### Undo/Redo

| Setting | Default | Description |
|---------|---------|-------------|
| MAX_HISTORY | 100 | Maximum undo steps |

### Gizmo

| Setting | Default | Description |
|---------|---------|-------------|
| AXIS_LENGTH | 1.0 | Gizmo axis length |
| PICK_THRESHOLD | 0.2 | Click tolerance (% of size) |

## Input Settings

### Mouse

| Setting | Default | Description |
|---------|---------|-------------|
| DOUBLE_CLICK_TIME | 300ms | Max time between clicks |

### Gamepad

Deadzone settings in `praxis_input`:

| Axis | Deadzone |
|------|----------|
| Left Stick | 0.15 |
| Right Stick | 0.15 |
| Triggers | 0.1 |

## Environment Variables

| Variable | Description |
|----------|-------------|
| RUST_LOG | Logging level (trace, debug, info, warn, error) |
| VULKAN_SDK | Path to Vulkan SDK |

## Compile-Time Features

Cargo features for optional functionality:

```toml
[features]
default = []
editor = ["praxis_editor"]
physics = ["praxis_physics"]
audio = ["praxis_audio"]
```

## Runtime Configuration

Most settings can be modified at runtime through resources:

```rust
// Modify physics
let mut config = world.resource_mut::<PhysicsConfig>();
config.gravity = Vec3::new(0.0, -20.0, 0.0);

// Modify camera
let mut camera = world.resource_mut::<EditorCameraController>();
camera.smoothness = 15.0;
```

## See Also

- [Crates Reference](crates.md) - Where settings are defined
- [Architecture](../ARCHITECTURE.md) - System organization
