# Audio Guide

Practical guide to using the audio system in Praxis for sound playback, 3D spatial audio, and the doppler effect.

## Quick Start

### Initialize Audio System

```rust
use praxis_audio::{AudioManager, play_sound_system, update_listener_system};
use praxis_ecs::{World, Schedule};

let mut world = World::new();

// Initialize audio manager
let audio_manager = AudioManager::new()?;
world.insert_resource(audio_manager);

// Add audio systems to schedule
schedule.add_systems((
    play_sound_system,
    update_listener_system,
).chain());
```

### Play a Simple Sound

```rust
use praxis_audio::AudioSource;

// Non-spatial background music
world.spawn(
    AudioSource::new("assets/audio/music.ogg")
        .with_volume(0.7)
        .with_looping(true)
);
```

## Audio Listener

The listener represents where sounds are heard from (typically the camera):

```rust
use praxis_audio::AudioListener;
use praxis_ecs::Transform;

world.spawn((
    Transform::from_xyz(0.0, 1.8, 0.0),  // Ear height
    AudioListener,
));
```

**Important**: Only one active listener should exist in the scene.

## Audio Sources

### Basic Audio Source

```rust
// Simple one-shot sound
world.spawn(
    AudioSource::new("assets/audio/explosion.ogg")
        .with_volume(1.0)
);
```

### Looping Audio

```rust
// Ambient sound loop
world.spawn(
    AudioSource::new("assets/audio/ambient.ogg")
        .with_volume(0.5)
        .with_looping(true)
);
```

## 3D Spatial Audio

### Basic Spatial Sound

```rust
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),  // Position in world
    AudioSource::new("assets/audio/campfire.ogg")
        .with_spatial(true)
        .with_volume(0.8)
        .with_looping(true)
        .with_max_distance(50.0)
        .with_reference_distance(5.0),
));
```

### Distance Attenuation

Sound volume decreases with distance:

```rust
AudioSource::new("sound.ogg")
    .with_spatial(true)
    .with_reference_distance(10.0)  // Full volume up to 10 units
    .with_max_distance(100.0)        // Silent beyond 100 units
```

**Distance formula**: `volume = base_volume × (reference_distance / distance)²`

```
Distance from listener:
  0-10 units:   Full volume
  10-20 units:  ~25% volume
  20-40 units:  ~6% volume
  40+ units:    Very quiet
  100+ units:   Silent
```

### Stereo Panning

Spatial audio automatically pans left/right based on position:

```rust
// Sound to the left → left speaker louder
Transform::from_xyz(-10.0, 0.0, 0.0)

// Sound to the right → right speaker louder
Transform::from_xyz(10.0, 0.0, 0.0)

// Sound in front/back → centered
Transform::from_xyz(0.0, 0.0, 10.0)
```

## Doppler Effect

Pitch changes based on relative velocity between source and listener:

```rust
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    AudioSource::new("assets/audio/engine.ogg")
        .with_spatial(true)
        .with_doppler(true)
        .with_doppler_scale(1.0)  // 1.0 = realistic, higher = exaggerated
        .with_looping(true),
));
```

### Doppler Behavior

- **Approaching**: Pitch increases (higher frequency)
- **Receding**: Pitch decreases (lower frequency)
- **Stationary**: No pitch change
- **Perpendicular motion**: Minimal pitch change

### Doppler Scale

```rust
.with_doppler_scale(0.0)   // Disabled
.with_doppler_scale(1.0)   // Physically accurate
.with_doppler_scale(2.0)   // Exaggerated (for gameplay)
```

## Controlling Playback

### Play/Pause/Stop

```rust
fn control_audio(mut query: Query<&mut AudioSource>) {
    for mut source in query.iter_mut() {
        source.play();
        source.pause();
        source.stop();
    }
}
```

### Volume Control

```rust
fn fade_in(
    time: Res<Time>,
    mut query: Query<&mut AudioSource, With<Music>>,
) {
    for mut source in query.iter_mut() {
        let new_volume = (source.volume() + time.delta_seconds() * 0.5).min(1.0);
        source.set_volume(new_volume);
    }
}
```

## Common Patterns

### Footstep Sounds

```rust
#[derive(Component)]
struct FootstepTimer {
    timer: f32,
    interval: f32,
}

fn play_footsteps(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&Transform, &Velocity, &mut FootstepTimer)>,
) {
    for (transform, velocity, mut timer) in query.iter_mut() {
        let speed = velocity.linear.xz().length();
        
        if speed > 0.1 {
            timer.timer += time.delta_seconds();
            
            if timer.timer >= timer.interval {
                timer.timer = 0.0;
                
                // Spawn footstep sound at player position
                commands.spawn((
                    Transform::from_translation(transform.translation),
                    AudioSource::new("assets/audio/footstep.ogg")
                        .with_spatial(true)
                        .with_volume(0.3)
                        .with_max_distance(20.0),
                ));
            }
        }
    }
}
```

### Music Manager

```rust
#[derive(Resource)]
struct MusicManager {
    current_track: Option<Entity>,
}

fn change_music(
    mut commands: Commands,
    mut manager: ResMut<MusicManager>,
    query: Query<Entity, With<Music>>,
) {
    // Stop current track
    if let Some(current) = manager.current_track {
        commands.entity(current).despawn();
    }
    
    // Start new track
    let new_track = commands.spawn((
        AudioSource::new("assets/audio/level_music.ogg")
            .with_volume(0.6)
            .with_looping(true),
        Music,
    )).id();
    
    manager.current_track = Some(new_track);
}
```

### 3D Sound Emitter

Attach sounds to moving objects:

```rust
#[derive(Component)]
struct Engine;

fn spawn_vehicle(commands: &mut Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Vehicle,
    )).with_children(|parent| {
        parent.spawn((
            Transform::default(),
            AudioSource::new("assets/audio/engine.ogg")
                .with_spatial(true)
                .with_doppler(true)
                .with_looping(true)
                .with_max_distance(100.0),
            Engine,
        ));
    });
}
```

### Environmental Ambience

```rust
fn setup_ambience(mut commands: Commands) {
    // Wind
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        AudioSource::new("assets/audio/wind.ogg")
            .with_spatial(true)
            .with_volume(0.4)
            .with_looping(true)
            .with_max_distance(100.0)
            .with_reference_distance(20.0),
        Wind,
    ));
    
    // Waterfall
    commands.spawn((
        Transform::from_xyz(50.0, 0.0, 0.0),
        AudioSource::new("assets/audio/waterfall.ogg")
            .with_spatial(true)
            .with_volume(0.8)
            .with_looping(true)
            .with_max_distance(80.0)
            .with_reference_distance(15.0),
    ));
}
```

### Weapon Fire Sound

```rust
fn shoot_weapon(
    mut commands: Commands,
    query: Query<&Transform, With<Player>>,
) {
    for transform in query.iter() {
        // Play gunshot at player position
        commands.spawn((
            Transform::from_translation(transform.translation),
            AudioSource::new("assets/audio/gunshot.ogg")
                .with_spatial(true)
                .with_volume(1.0)
                .with_max_distance(100.0),
        ));
    }
}
```

### UI Sounds

Non-spatial sounds for UI interactions:

```rust
fn button_click(
    mut commands: Commands,
    interaction_query: Query<&Interaction, Changed<Interaction>>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            commands.spawn(
                AudioSource::new("assets/audio/click.ogg")
                    .with_volume(0.5)
            );
        }
    }
}
```

### Dynamic Music Layers

```rust
#[derive(Resource)]
struct MusicLayers {
    base: Entity,
    combat: Entity,
}

fn update_music_intensity(
    mut query: Query<&mut AudioSource>,
    layers: Res<MusicLayers>,
    combat_active: Res<CombatState>,
) {
    if let Ok(mut combat_layer) = query.get_mut(layers.combat) {
        let target_volume = if combat_active.in_combat { 1.0 } else { 0.0 };
        
        // Smooth transition
        let current = combat_layer.volume();
        let new_volume = current.lerp(target_volume, 0.05);
        combat_layer.set_volume(new_volume);
    }
}
```

## Supported Audio Formats

Via Kira audio library:
- **OGG Vorbis** - Recommended for music (good compression, looping support)
- **WAV** - Recommended for short sound effects (no decompression overhead)
- **MP3** - Supported but prefer OGG
- **FLAC** - Lossless, large files

## Performance Considerations

### Audio Source Limits

```rust
// Track active sources
#[derive(Resource)]
struct AudioStats {
    active_sources: usize,
}

fn monitor_audio(
    query: Query<&AudioSource>,
    mut stats: ResMut<AudioStats>,
) {
    stats.active_sources = query.iter().count();
    
    if stats.active_sources > 50 {
        tracing::warn!("Many audio sources active: {}", stats.active_sources);
    }
}
```

### Distance-Based Culling

Sounds beyond `max_distance` are automatically culled:

```rust
// Near sounds: small max_distance
AudioSource::new("footstep.ogg")
    .with_max_distance(20.0)

// Distant sounds: large max_distance
AudioSource::new("thunder.ogg")
    .with_max_distance(500.0)
```

### One-Shot vs Looping

```rust
// One-shot: automatically cleaned up when finished
AudioSource::new("explosion.ogg")
    .with_looping(false)

// Looping: persists until manually stopped
AudioSource::new("ambient.ogg")
    .with_looping(true)
```

### Cleanup Finished Sounds

```rust
fn cleanup_finished_audio(
    mut commands: Commands,
    query: Query<(Entity, &AudioSource), Without<Looping>>,
) {
    for (entity, source) in query.iter() {
        if source.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

## Debugging

### Visualize Audio Sources

```rust
fn debug_draw_audio(
    query: Query<(&Transform, &AudioSource)>,
    mut debug_lines: ResMut<DebugLines>,
) {
    for (transform, source) in query.iter() {
        if source.is_spatial() {
            let pos = transform.translation;
            
            // Draw reference distance
            debug_lines.circle(pos, source.reference_distance(), Color::GREEN);
            
            // Draw max distance
            debug_lines.circle(pos, source.max_distance(), Color::RED);
        }
    }
}
```

### Log Audio State

```rust
fn log_audio_state(
    query: Query<(&Transform, &AudioSource, Option<&Name>)>,
    listener: Query<&Transform, With<AudioListener>>,
) {
    if let Ok(listener_transform) = listener.get_single() {
        for (transform, source, name) in query.iter() {
            let distance = transform.translation.distance(listener_transform.translation);
            
            tracing::debug!(
                "{}: distance={:.1}, volume={:.2}, spatial={}",
                name.map(|n| n.as_str()).unwrap_or("Unnamed"),
                distance,
                source.volume(),
                source.is_spatial()
            );
        }
    }
}
```

## Troubleshooting

### No Sound Playing

**Problem**: Audio sources spawn but no sound is heard

**Solutions**:
- Verify `AudioManager` resource is initialized
- Check audio systems are in schedule
- Confirm audio file path is correct
- Ensure volume > 0.0
- Check if source is beyond `max_distance`

### Spatial Audio Not Working

**Problem**: All sounds play at same volume regardless of position

**Solutions**:
- Verify `spatial` flag is enabled: `.with_spatial(true)`
- Check `AudioListener` component exists
- Ensure `Transform` component is on source entity
- Verify listener and source have different positions

### Doppler Effect Too Subtle/Strong

**Problem**: Pitch changes are barely noticeable or too extreme

**Solutions**:
- Adjust `doppler_scale`: higher values = more extreme effect
- Ensure objects are moving fast enough (>5 units/sec)
- Verify doppler is enabled: `.with_doppler(true)`
- Check movement is along listener-source line (not perpendicular)

### Audio Stuttering

**Problem**: Audio playback is choppy

**Solutions**:
- Reduce number of active audio sources
- Use WAV for short effects (less CPU for decompression)
- Ensure frame rate is stable
- Check for excessive `max_distance` updates

### Too Many Active Sources

**Problem**: Performance issues with many sounds

**Solutions**:
- Implement audio source pooling
- Use smaller `max_distance` for local sounds
- Clean up finished one-shot sounds
- Limit simultaneous sounds per type (e.g., max 3 explosions)

## Audio Pooling Pattern

Reuse audio source entities:

```rust
#[derive(Resource)]
struct AudioPool {
    available: Vec<Entity>,
}

impl AudioPool {
    fn get_or_spawn(
        &mut self,
        commands: &mut Commands,
        path: &str,
    ) -> Entity {
        self.available.pop().unwrap_or_else(|| {
            commands.spawn(
                AudioSource::new(path)
                    .with_spatial(true)
            ).id()
        })
    }
    
    fn return_to_pool(&mut self, entity: Entity) {
        self.available.push(entity);
    }
}
```

## Examples

See working examples:
- `examples/audio_demo.rs` - Comprehensive audio features
- `examples/audio_simple.rs` - Basic audio playback

Run with:
```bash
cargo run --example audio_demo
```

## See Also

- [Spatial Audio Concepts](../concepts/spatial-audio.md) - Theory and algorithms
- [Audio API Reference](../reference/audio-api.md) - API documentation
- [praxis_audio Crate](../../crates/praxis_audio/README.md) - Crate documentation
- [Kira Documentation](https://docs.rs/kira) - Underlying audio library
