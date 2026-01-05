# Audio Learning Path

Create immersive soundscapes with spatial audio positioning and effects.

## Path Overview

**Time Investment**: 1 week  
**Prerequisites**: Basic understanding of 3D space  
**Final Goal**: Production-ready audio system

## Progression Map

```
Beginner (2-3 days)
├── Audio loading
├── Playback control
├── Volume management
└── Basic mixing
    ↓
Intermediate (2-3 days)
├── Spatial positioning
├── Distance attenuation
├── Listener configuration
└── Audio effects
    ↓
Advanced (2-3 days)
├── Performance optimization
├── Audio pooling
├── LOD for sounds
└── Music system
```

---

## Beginner: Audio Playback

**Theory** (2 hours):
- Read [Spatial Audio Concepts](../concepts/spatial-audio.md)
- Read [Audio Guide](../guides/audio.md)

**Practice** (4-6 hours):
```rust
use praxis_audio::{AudioManager, Sound};

// Load audio
let sound = audio_manager.load("sounds/jump.ogg")?;

// Play
audio_manager.play(sound);

// Control playback
handle.set_volume(0.8);
handle.set_pitch(1.2);
handle.stop();
```

**Examples**:
```bash
cargo run --example audio_simple
```

### Checkpoint
- [ ] Can load and play sounds
- [ ] Understand volume control
- [ ] Know basic audio formats

**Time**: 6-8 hours

---

## Intermediate: Spatial Audio

**Theory** (2-3 hours):
- Continue [Audio Guide: Spatial Audio](../guides/audio.md)

**Practice** (6-8 hours):
```rust
// 3D positioned sound
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    AudioSource::new("sounds/ambient.ogg"),
    SpatialAudio {
        max_distance: 50.0,
        rolloff_factor: 1.0,
    },
));

// Configure listener (camera)
world.spawn((
    Transform::default(),
    Camera::default(),
    AudioListener,
));
```

**Example**:
```bash
cargo run --example audio_demo
```

### Checkpoint
- [ ] Spatial positioning working
- [ ] Distance attenuation correct
- [ ] Listener tracks camera

**Time**: 10-12 hours

---

## Advanced: Optimization

**Practice** (6-8 hours):
- Audio pooling
- Sound LOD (distance-based)
- Priority system
- Music streaming

**Optimization**:
```rust
// Only play closest N sounds
let config = AudioConfig {
    max_simultaneous_sources: 32,
    distance_culling: true,
    ..Default::default()
};

// Audio LOD
if distance < 10.0 {
    play_high_quality();
} else if distance < 50.0 {
    play_medium_quality();
} else {
    // Don't play or play very low quality
}
```

### Checkpoint
- [ ] Optimized for many sounds
- [ ] LOD system working
- [ ] Music system integrated

**Time**: 8-10 hours

---

## Cross-References

- [Physics Path](physics.md) - Sound on collision
- [Animation Path](animation.md) - Footstep sounds
- [Scripting Path](scripting.md) - Trigger sounds from Lua

---

[← Back to Learning Paths](README.md) | [Next: Editor Path →](editor.md)
