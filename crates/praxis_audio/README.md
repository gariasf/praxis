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
use praxis_audio::{play_sound_system, update_listener_system};
use praxis_ecs::{World, Schedule, Transform, GlobalTransform};
use color_eyre::Result;

fn setup_audio(world: &mut World, schedule: &mut Schedule) -> Result<()> {
    // Initialize audio manager
    let audio_manager = AudioManager::new()?;
    world.insert_resource(audio_manager);
    
    // Add audio systems to schedule
    schedule.add_systems((
        play_sound_system,
        update_listener_system,
    ).chain());
    
    // Add audio listener (typically attached to camera)
    world.spawn((
        Transform::from_xyz(0.0, 1.8, 0.0),  // Ear height
        GlobalTransform::default(),
        AudioListener,
    ));
    
    // Spawn a spatial audio source
    world.spawn((
        Transform::from_xyz(10.0, 0.0, 5.0),
        GlobalTransform::default(),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(10.0)
            .with_doppler(true)
            .with_doppler_scale(1.0),
    ));
    
    Ok(())
}
```

## Basic Audio Playback

```rust
use praxis_audio::AudioSource;
use praxis_ecs::World;

fn play_simple_sound(world: &mut World) {
    // Non-spatial background music
    world.spawn(
        AudioSource::new("assets/audio/music.ogg")
            .with_volume(0.7)
            .with_looping(true)
    );
    
    // One-shot sound effect
    world.spawn(
        AudioSource::new("assets/audio/explosion.ogg")
            .with_volume(1.0)
    );
}
```

## Spatial Audio

```rust
use praxis_audio::AudioSource;
use praxis_ecs::{World, Transform, GlobalTransform};

fn spawn_3d_sound(world: &mut World) {
    world.spawn((
        // Position in 3D space
        Transform::from_xyz(10.0, 0.0, 5.0),
        GlobalTransform::default(),
        
        AudioSource::new("assets/sounds/campfire.ogg")
            .with_spatial(true)          // Enable 3D positioning
            .with_volume(0.8)
            .with_looping(true)
            .with_max_distance(50.0)     // Silent beyond 50 units
            .with_reference_distance(5.0), // Full volume up to 5 units
    ));
}
```

**Distance Attenuation:**
```
volume = base_volume * (reference_distance / distance)²

Example with reference_distance = 10.0:
  Distance 0-10:  Full volume
  Distance 20:    ~25% volume
  Distance 40:    ~6% volume
  Distance 100+:  Silent
```

## Doppler Effect

```rust
use praxis_audio::AudioSource;
use praxis_ecs::{World, Transform, GlobalTransform};

fn create_moving_sound(world: &mut World) {
    world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        
        AudioSource::new("assets/sounds/engine.ogg")
            .with_spatial(true)
            .with_doppler(true)         // Enable doppler effect
            .with_doppler_scale(1.0)    // 1.0 = realistic, >1.0 = exaggerated
            .with_looping(true),
    ));
}
```

**Doppler Effect:**
- Approaching: Pitch increases (higher frequency)
- Receding: Pitch decreases (lower frequency)
- Stationary: No pitch change
- Perpendicular motion: Minimal pitch change

Uses classic doppler formula with configurable scale factor.

## Controlling Playback

```rust
use praxis_audio::AudioSource;
use praxis_ecs::Query;

fn control_audio(mut query: Query<&mut AudioSource>) {
    for mut source in query.iter_mut() {
        // Playback control
        source.play();
        source.pause();
        source.stop();
        
        // Volume control
        source.set_volume(0.5);
        
        // Check state
        let is_playing = source.is_playing();
        let is_finished = source.is_finished();
    }
}
```

## Documentation

**Comprehensive Guides:**
- [Audio Guide](../../docs/guides/audio.md) - Complete audio system guide

**Concepts:**
- [Spatial Audio Concepts](../../docs/concepts/spatial-audio.md)

**Reference:**
- [Audio API Reference](../../docs/reference/audio-api.md)

## Examples

```bash
# Simple audio playback
cargo run --example audio_simple

# Spatial audio with doppler effect
cargo run --example audio_demo
```

## Dependencies

- `kira` 0.9: Audio backend
- `bevy_ecs` 0.14: ECS integration
