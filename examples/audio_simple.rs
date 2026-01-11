//! Simple audio system demonstration.
//!
//! This example demonstrates basic audio system setup without requiring audio files.
//! It shows the component and system structure.

use praxis_audio::{play_sound_system, AudioListener, AudioManager, AudioSource};
use praxis_ecs::{Schedule, Transform, World};
use praxis_utils::{info, Result};

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_audio::init()?;

    info!("=== Praxis Audio Simple Demo ===");

    let mut world = World::new();

    info!("Creating audio manager...");
    // Try to create audio manager, but gracefully handle failure
    // (e.g., in CI environments or systems without audio output)
    let audio_manager = match AudioManager::new() {
        Ok(manager) => {
            info!("Audio manager initialized successfully");
            manager
        }
        Err(e) => {
            info!("Failed to initialize audio manager: {}", e);
            info!("This is expected in environments without audio output (e.g., CI)");
            info!("The demo will continue to show the structure, but audio won't play.");
            // Return early since we can't proceed without audio manager
            return Ok(());
        }
    };
    
    world.insert_resource(audio_manager);

    info!("Setting up audio scene...");

    world.spawn((Transform::from_xyz(0.0, 1.8, 0.0), AudioListener));
    info!("Created audio listener at (0.0, 1.8, 0.0)");

    world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/test.ogg")
            .with_volume(0.7)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));
    info!("Created spatial audio source at (10.0, 0.0, 0.0)");

    world.spawn((
        Transform::from_xyz(-10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_volume(0.5)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));
    info!("Created spatial audio source at (-10.0, 0.0, 0.0)");

    let mut schedule = Schedule::default();
    schedule.add_systems(play_sound_system);

    info!("Running audio system once to verify integration...");
    schedule.run(world.inner_mut());

    info!("Audio system setup complete!");
    info!("Note: Audio files would need to exist at the specified paths to actually play sounds.");

    let mut audio_sources = world.inner_mut().query::<(&AudioSource, &Transform)>();
    let count = audio_sources.iter(world.inner()).count();
    info!("Total audio sources in scene: {}", count);

    let mut listeners = world.inner_mut().query::<(&AudioListener, &Transform)>();
    let listener_count = listeners.iter(world.inner()).count();
    info!("Total audio listeners in scene: {}", listener_count);

    info!("Demo complete!");

    Ok(())
}
