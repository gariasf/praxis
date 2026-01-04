# Spatial Audio

3D audio positioning and propagation in Praxis, powered by the Kira audio library.

## Core Concepts

### Audio Listener
The "ears" in the 3D world—typically attached to the camera or player entity.

```rust
#[derive(Component)]
pub struct AudioListener;

// Attach to camera
world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    AudioListener,
));
```

### Audio Source
Sound emitters positioned in 3D space.

```rust
#[derive(Component)]
pub struct AudioSource {
    pub path: String,
    pub volume: f32,
    pub spatial: bool,
    pub looping: bool,
    pub max_distance: f32,
    pub reference_distance: f32,
}
```

## Distance Attenuation

Sound volume decreases with distance using inverse square law:

```
volume = base_volume × (reference_distance / distance)²
```

- **Reference Distance**: Distance at which volume equals base volume (default: 1.0)
- **Max Distance**: Beyond this, sound is inaudible (culled for performance)

### Attenuation Curve

```
Volume
  │
1 ┤────┐
  │    ╲
  │     ╲
  │      ╲
  │       ╲_______________
0 ┼────────────────────────→ Distance
     ref    max
```

## Stereo Panning

Sound is panned left/right based on the source's horizontal position relative to the listener:

```rust
// Simplified panning calculation
let direction = (source_pos - listener_pos).normalize();
let right_dot = direction.dot(listener_right);
let pan = right_dot.clamp(-1.0, 1.0);
```

- Source on left → pan = -1.0 (left channel louder)
- Source on right → pan = 1.0 (right channel louder)
- Source in front/back → pan = 0.0 (balanced)

## Doppler Effect

Pitch shifts based on relative velocity between source and listener:

```rust
// Classic Doppler formula
let radial_velocity = relative_velocity.dot(direction);
let pitch_scale = SPEED_OF_SOUND / (SPEED_OF_SOUND - radial_velocity);
```

- Source approaching → higher pitch
- Source receding → lower pitch

Enable with:
```rust
AudioSource::new("engine.ogg")
    .with_doppler(true)
    .with_doppler_scale(1.0)
```

## Update Systems

### play_sound_system
Processes audio playback and initial spatial positioning.

### update_spatial_audio_system
Efficiently updates spatial parameters when source transforms change.

### update_listener_system
Updates all sources when the listener moves.

## Usage Example

```rust
use praxis_audio::{AudioManager, AudioSource, AudioListener};

// Initialize audio
world.insert_resource(AudioManager::new()?);

// Create listener (camera)
world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    AudioListener,
));

// Create spatial sound source
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    AudioSource::new("ambient.ogg")
        .with_volume(0.8)
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(50.0),
));
```

## Performance Considerations

1. **Max Distance Culling**: Sources beyond max_distance skip processing
2. **Change Detection**: Only update when transforms change
3. **Audio Pooling**: Kira reuses sound instances internally

## Supported Formats

- OGG Vorbis (recommended for music)
- WAV (recommended for short effects)
- MP3
- FLAC

## See Also

- [praxis_audio crate](../../crates/praxis_audio/README.md) - API documentation
- [audio_demo example](../../examples/audio_demo.rs) - Working example
