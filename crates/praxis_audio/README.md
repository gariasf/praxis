# Praxis Audio

Audio system for the Praxis game engine, providing sound playback and spatial audio capabilities with doppler effect.

## Features

- **Audio Playback**: Load and play audio files (OGG, MP3, WAV, FLAC)
- **3D Spatial Audio**: Positioned audio with distance attenuation and stereo panning
- **Doppler Effect**: Realistic pitch shifting based on relative velocity
- **Listener Transform Synchronization**: Efficient updates when listener or sources move
- **Audio Manager**: Central resource for managing audio backend and loaded sounds
- **ECS Integration**: Components and systems for seamless integration with the engine

## Architecture

The audio system consists of three main parts:

1. **AudioManager** (Resource): Manages the Kira audio backend, loads sounds, and controls playback
2. **AudioSource** (Component): Attaches to entities to make them emit sounds
3. **AudioListener** (Component): Marks the listener position (typically the camera)

## Usage

### Basic Setup

```rust
use praxis_audio::{AudioManager, AudioSource, AudioListener};
use praxis_audio::{play_sound_system, update_listener_system};
use praxis_ecs::{World, Schedule, Transform, IntoSystemConfigs};

// Initialize the audio manager
let mut world = World::new();
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);

// Add the audio systems to your schedule
let mut schedule = Schedule::default();
schedule.add_systems((
    play_sound_system,
    update_listener_system,
).chain());
```

### Creating an Audio Listener

The audio listener represents the position from which sounds are heard. Typically attached to the camera:

```rust
world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    AudioListener,
));
```

### Playing a Sound

#### Non-Spatial Audio

```rust
world.spawn(AudioSource::new("assets/sounds/music.ogg")
    .with_volume(0.7)
    .with_looping(true));
```

#### Spatial Audio

```rust
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    AudioSource::new("assets/sounds/ambient.ogg")
        .with_volume(0.7)
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(50.0)
        .with_reference_distance(5.0),
));
```

#### Spatial Audio with Doppler Effect

```rust
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    AudioSource::new("assets/sounds/vehicle.ogg")
        .with_volume(0.8)
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(100.0)
        .with_reference_distance(10.0)
        .with_doppler(true)
        .with_doppler_scale(1.0), // 1.0 for realistic, higher for exaggerated
));
```

### Controlling Playback

```rust
fn control_audio(mut audio_sources: Query<&mut AudioSource>) {
    for mut source in audio_sources.iter_mut() {
        source.play();   // Start playing
        source.pause();  // Pause playback
        source.stop();   // Stop playback
    }
}
```

## Spatial Audio

Spatial audio automatically adjusts volume and panning based on the distance and position between the audio source and the listener.

### Distance Attenuation

The system uses inverse square law for realistic audio falloff:

```
volume = base_volume * (reference_distance / distance)^2
```

- **reference_distance**: The distance at which the sound plays at its base volume
- **max_distance**: The distance beyond which the sound is inaudible
- Within reference distance: Sound plays at base volume
- Beyond max distance: Sound is silent
- Between: Volume decreases with distance squared

### Stereo Panning

Spatial audio applies stereo panning based on the horizontal (X-axis) position of the sound relative to the listener:
- Sources to the right pan right (+1.0)
- Sources to the left pan left (-1.0)
- Sources directly in front/behind are centered (0.0)

## Doppler Effect

The doppler effect simulates realistic pitch changes when audio sources or the listener are moving:

### Physics

Uses the classic doppler formula:
```
f' = f * c / (c - v_radial)
```

Where:
- `f'` is the perceived frequency (pitch)
- `f` is the original frequency
- `c` is the speed of sound (343.0 world units/second)
- `v_radial` is the velocity component along the line between source and listener

### Behavior

- **Approaching**: Pitch increases (higher frequency) as source moves toward listener
- **Receding**: Pitch decreases (lower frequency) as source moves away from listener
- **Stationary**: No pitch change when relative velocity is zero
- **Perpendicular motion**: Minimal pitch change when moving perpendicular to listener

### Configuration

```rust
AudioSource::new("sound.ogg")
    .with_doppler(true)           // Enable doppler effect
    .with_doppler_scale(1.0)      // Scale factor (1.0 = realistic, 2.0 = exaggerated)
```

The doppler scale allows you to:
- Set to 0.0 to disable doppler effect
- Set to 1.0 for physically accurate doppler shift
- Set higher (e.g., 2.0) for exaggerated effect in gameplay
- Playback rate is automatically clamped to 0.5-2.0 range for stability

## Listener Transform Synchronization

The audio system provides efficient listener tracking:

### Systems

1. **play_sound_system**: Main system that handles playback and spatial updates
2. **update_spatial_audio_system**: Optimized for audio source transform changes
3. **update_listener_system**: Optimized for listener transform changes

### Efficiency

- Uses ECS change detection to only update when transforms actually change
- Avoids redundant calculations when listener is stationary
- Automatically tracks previous positions for doppler velocity calculation
- Minimal overhead when no movement occurs

## Supported Audio Formats

Via the Kira audio library:
- OGG Vorbis
- MP3
- WAV
- FLAC

## Performance Considerations

- Sounds are cached after first load to avoid redundant I/O
- The audio system automatically cleans up finished sounds
- Spatial audio updates only occur for sources with the `spatial` flag enabled
- Doppler effect only calculated when enabled for a source
- Change detection minimizes updates to only modified transforms
- Previous positions cached per-source for efficient velocity calculation

## System Ordering

For best results, schedule systems in this order:

```rust
schedule.add_systems((
    play_sound_system,
    update_spatial_audio_system,  // Optional: for source transform changes
    update_listener_system,        // Optional: for listener transform changes
).chain());
```

Or use just the main system for simplicity:

```rust
schedule.add_systems(play_sound_system);
```

## Examples

See the audio demos for complete demonstrations:

```bash
# Simple audio demo
cargo run --example audio_simple

# Comprehensive demo with spatial audio and doppler
cargo run --example audio_demo
```

These demonstrate:
- 3D spatial audio with distance attenuation
- Stereo panning based on position
- Doppler effect with moving audio sources
- Listener movement and synchronization
- Multiple audio sources with different configurations

## Dependencies

- `kira` 0.9: Audio playback backend with pitch/pan/volume control
- `bevy_ecs` 0.14: ECS integration
- `praxis_ecs`: World, Query, Components, Systems
- `praxis_math`: Math utilities for spatial calculations (Vec3, distance, dot product)
- `praxis_utils`: Error handling and logging

## See Also

- [Audio Guide](../../docs/guides/audio.md)
- [Spatial Audio Concepts](../../docs/concepts/spatial-audio.md)
- [Kira Documentation](https://docs.rs/kira)
