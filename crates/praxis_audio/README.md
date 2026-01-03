# praxis_audio

Audio system for the Praxis game engine, providing sound playback and spatial audio capabilities.

## Features

- **Audio Playback**: Load and play audio files (OGG, MP3, WAV, FLAC)
- **Spatial Audio**: 3D positioned audio with distance attenuation
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
use praxis_audio::{AudioManager, AudioSource, AudioListener, play_sound_system};
use praxis_ecs::{World, Schedule, Transform};

// Initialize the audio manager
let mut world = World::new();
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);

// Add the audio system to your schedule
let mut schedule = Schedule::default();
schedule.add_systems(play_sound_system);
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

Spatial audio automatically adjusts volume based on the distance between the audio source and the listener.

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

### Panning

Spatial audio also applies stereo panning based on the horizontal position of the sound relative to the listener.

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
- Use `update_spatial_audio_system` for optimized updates when transforms change

## Example

See `examples/audio_demo.rs` for a complete demonstration of the audio system with spatial audio and listener movement.

## Dependencies

- `kira`: Audio playback backend
- `praxis_ecs`: ECS integration
- `praxis_math`: Math utilities for spatial calculations
- `praxis_utils`: Error handling and logging
