# Audio System Documentation

## Overview

The `praxis_audio` crate provides audio playback capabilities for the Praxis game engine, including spatial audio support for 3D positioned sounds.

## Architecture

### Components

The audio system uses three main components:

1. **`AudioSource`** - Marks an entity as an audio emitter
   - Path to audio file
   - Volume (0.0 to 1.0)
   - Spatial audio flag
   - Looping flag
   - Max distance and reference distance for attenuation
   - Playback state (Playing, Paused, Stopped)

2. **`AudioListener`** - Marks an entity as the audio listener
   - Typically attached to the camera
   - Only one listener should be active at a time

3. **`AudioState`** - Enum for playback state
   - Playing
   - Paused
   - Stopped

### Resources

**`AudioManager`** - ECS resource managing the audio backend
- Wraps the Kira audio library
- Handles sound loading and caching
- Manages playback of sound instances
- Provides playback control (play, pause, stop, volume)
- Automatically cleans up finished sounds

### Systems

1. **`play_sound_system`** - Main audio system
   - Processes AudioSource components
   - Starts playing sounds when state is set to Playing
   - Updates spatial audio based on entity positions
   - Applies distance attenuation and panning
   - Stops sounds when requested
   - Cleans up finished sounds

2. **`update_spatial_audio_system`** - Optimized spatial audio updater
   - Only processes audio sources whose transforms have changed
   - Uses bevy_ecs `Changed<Transform>` filter for efficiency
   - Updates volume and panning based on new positions

## Spatial Audio

### Distance Attenuation

Spatial audio uses inverse square law for realistic distance-based volume attenuation:

```
volume = base_volume * (reference_distance / distance)^2
```

**Key distances:**
- **reference_distance**: Distance at which the sound plays at base volume
- **max_distance**: Distance beyond which the sound is inaudible

**Behavior:**
- Within reference distance: Sound plays at base volume
- Between reference and max distance: Volume decreases with distance squared
- Beyond max distance: Sound is silent

### Panning

Stereo panning is calculated based on the horizontal (X-axis) position of the sound relative to the listener:
- Negative X: Sound pans to the left
- Positive X: Sound pans to the right
- Panning is clamped to [-1.0, 1.0]

## Usage

### Initialization

The audio system is initialized as part of the engine startup sequence in `praxis_core::run()`:

```rust
praxis_audio::init()?;
```

### Creating the Audio Manager

```rust
use praxis_audio::AudioManager;
use praxis_ecs::World;

let mut world = World::new();
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);
```

### Adding Audio Systems

```rust
use praxis_audio::{play_sound_system, update_spatial_audio_system};
use praxis_ecs::{Schedule, IntoSystemConfigs};

let mut schedule = Schedule::default();
schedule.add_systems((
    play_sound_system,
    update_spatial_audio_system,
).chain());
```

### Creating an Audio Listener

```rust
use praxis_audio::AudioListener;
use praxis_ecs::{World, Transform};

world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    AudioListener,
));
```

### Playing Non-Spatial Audio

```rust
use praxis_audio::AudioSource;

world.spawn(AudioSource::new("assets/sounds/music.ogg")
    .with_volume(0.8)
    .with_looping(true));
```

### Playing Spatial Audio

```rust
use praxis_audio::AudioSource;
use praxis_ecs::Transform;

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
use praxis_audio::AudioSource;
use praxis_ecs::Query;

fn control_system(mut audio_sources: Query<&mut AudioSource>) {
    for mut source in audio_sources.iter_mut() {
        if some_condition {
            source.play();
        } else if other_condition {
            source.pause();
        } else {
            source.stop();
        }
    }
}
```

## Supported Audio Formats

Via Kira audio library:
- OGG Vorbis
- MP3
- WAV
- FLAC

## Performance Considerations

1. **Sound Caching**: Sounds are cached in the AudioManager after first load to avoid redundant I/O operations

2. **Spatial Audio**: Only enabled when the `spatial` flag is true, avoiding unnecessary calculations

3. **Optimized Updates**: The `update_spatial_audio_system` only processes sources whose transforms have changed

4. **Automatic Cleanup**: Finished sounds are automatically removed from tracking to prevent memory leaks

5. **ECS Integration**: Uses bevy_ecs queries for efficient iteration over audio sources

## Implementation Details

### File Structure

```
crates/praxis_audio/
├── Cargo.toml          # Crate dependencies and metadata
├── README.md           # User-facing documentation
└── src/
    ├── lib.rs          # Crate entry point and initialization
    ├── components.rs   # AudioSource, AudioListener, AudioState
    ├── manager.rs      # AudioManager resource
    └── systems.rs      # play_sound_system, update_spatial_audio_system
```

### Dependencies

- **kira**: Audio playback backend with CPAL support
- **praxis_ecs**: ECS integration (bevy_ecs wrapper)
- **praxis_math**: Math utilities (glam wrapper)
- **praxis_utils**: Error handling and logging

### Integration Points

1. **Workspace**: Added to `Cargo.toml` workspace members
2. **Engine Core**: Initialized in `praxis_core::run()`
3. **Main Crate**: Added as dependency to root `praxis` crate
4. **Examples**: `audio_demo.rs` and `audio_simple.rs` demonstrate usage
5. **Documentation**: Added to `CLAUDE.md` architecture documentation

## Examples

See the following example files for complete demonstrations:

- `examples/audio_simple.rs` - Basic audio system setup
- `examples/audio_demo.rs` - Full interactive demo with spatial audio and listener movement

## Future Enhancements

Potential future improvements:
- Audio effects (reverb, echo, filters)
- Audio groups/buses for mixing
- 3D audio with HRTF
- Audio streaming for large files
- Audio triggers based on events
- Audio occlusion based on physics raycasts
- Dynamic audio mixing
- Audio visualization tools
