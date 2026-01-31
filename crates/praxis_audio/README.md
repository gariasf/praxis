# praxis_audio

Audio system for Praxis engine using Kira.

## Overview

Provides audio playback, spatial audio, and sound management through the Kira audio engine.

## Features

- **Sound Playback**: One-shot and looping sounds
- **Spatial Audio**: 3D positioned sounds with attenuation
- **Music Streaming**: Background music with crossfading
- **Volume Control**: Master, music, and effects volumes
- **Sound Pooling**: Efficient sound instance management
- **Format Support**: MP3, OGG, FLAC, WAV

## Example

```rust
use praxis_audio::{AudioManager, Sound};

// Initialize audio system
let mut audio_manager = AudioManager::new()?;

// Play one-shot sound effect
audio_manager.play_sound("explosion.ogg", 1.0)?;

// Play spatial sound
audio_manager.play_spatial_sound(
    "footstep.wav",
    position,
    attenuation_distance,
)?;

// Play background music
audio_manager.play_music("theme.mp3", true)?;
```

## Architecture

```
AudioManager
    ├── Kira AudioManager
    ├── Sound Cache
    ├── Active Sound Instances
    └── Listener Transform
```

## Dependencies

- `kira`: Audio engine
- `serde`: Serialization support

## Usage

```toml
praxis_audio = { path = "../praxis_audio", version = "0.1.0" }
```
