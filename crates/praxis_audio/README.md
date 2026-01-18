# Praxis Audio

Audio playback and 3D spatial audio for the Praxis game engine.

## Overview

Audio system with spatial positioning, distance attenuation, and doppler effect.

**Key Features:**
- Audio playback (OGG, MP3, WAV, FLAC)
- 3D spatial audio with distance attenuation
- Stereo panning based on position
- Doppler effect for moving sources
- Efficient transform synchronization via ECS change detection

## Quick Start

```rust
use praxis_audio::{AudioManager, AudioSource, AudioListener};

// Initialize
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);

// Add listener (camera)
world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),
    GlobalTransform::default(),
    AudioListener,
));

// Spatial audio source
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    GlobalTransform::default(),
    AudioSource::new("assets/sounds/ambient.ogg")
        .with_spatial(true)
        .with_looping(true)
        .with_max_distance(50.0)
        .with_doppler(true),
));
```

## Spatial Audio

**Distance Attenuation:**
```
volume = base_volume * (reference_distance / distance)²
```

**Doppler Effect:**
- Approaching: Pitch increases
- Receding: Pitch decreases
- Uses classic doppler formula with configurable scale

## Documentation

**Comprehensive Guides:**
- [Audio Guide](../../docs/guides/audio.md) - Complete audio system guide

**Concepts:**
- [Spatial Audio Concepts](../../docs/concepts/spatial-audio.md)

**Reference:**
- [Audio API Reference](../../docs/reference/audio-api.md)

## Examples

```bash
cargo run --example audio_simple
cargo run --example audio_demo
```

## Dependencies

- `kira` 0.9: Audio backend
- `bevy_ecs` 0.14: ECS integration
