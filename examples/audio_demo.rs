//! Audio system demonstration.
//!
//! This example demonstrates the audio system with spatial audio and doppler effect.
//! It spawns several audio sources at different positions and a listener (camera).
//! Some sources are stationary while others move to demonstrate the doppler effect.
//! Use WASD to move the camera and hear how the spatial audio changes.

use praxis_audio::{play_sound_system, update_listener_system, AudioListener, AudioManager, AudioSource};
use praxis_ecs::{
    Commands, IntoSystemConfigs, Query, Res, ResMut, Resource, Schedule, Transform, With, Without, World,
};
use praxis_input::{InputState, KeyCode};
use praxis_math::{Quat, Vec3};
use praxis_utils::{info, Result};
use std::time::{Duration, Instant};

#[derive(Resource)]
struct DemoState {
    last_update: Instant,
}

impl DemoState {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }
}

/// Marker component for moving audio sources
#[derive(praxis_ecs::Component)]
struct MovingSource {
    speed: f32,
    direction: Vec3,
}

fn setup_audio_scene(mut commands: Commands, mut audio_manager: ResMut<AudioManager>) {
    info!("Setting up audio demo scene");

    info!("Creating audio listener (camera)");
    commands.spawn((Transform::from_xyz(0.0, 1.8, 0.0), AudioListener));

    info!("Note: This demo requires audio files to be placed in assets/sounds/");
    info!("Example files: ambient.ogg, beep.ogg, etc.");

    info!("Creating stationary spatial audio sources");
    // Stationary source to the right
    commands.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_volume(0.7)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));

    // Stationary source to the left
    commands.spawn((
        Transform::from_xyz(-10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/beep.ogg")
            .with_volume(0.5)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));

    info!("Creating moving audio sources with doppler effect");
    // Moving source with doppler effect (circling around)
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 15.0),
        AudioSource::new("assets/sounds/wind.ogg")
            .with_volume(0.6)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(60.0)
            .with_reference_distance(8.0)
            .with_doppler(true)
            .with_doppler_scale(1.5), // Exaggerated for demonstration
        MovingSource {
            speed: 10.0,
            direction: Vec3::new(1.0, 0.0, 0.0),
        },
    ));

    // Fast-moving source with strong doppler effect
    commands.spawn((
        Transform::from_xyz(20.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_volume(0.8)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(80.0)
            .with_reference_distance(10.0)
            .with_doppler(true)
            .with_doppler_scale(2.0), // Strong doppler
        MovingSource {
            speed: 20.0,
            direction: Vec3::new(-1.0, 0.0, 1.0).normalize(),
        },
    ));

    info!("Audio scene setup complete");
    info!("Use WASD to move the listener");
    info!("Use Space/Shift to move up/down");
    info!("Watch for moving sources with doppler effect!");
}

fn update_listener_position(
    input: Res<InputState>,
    mut listener_query: Query<&mut Transform, With<AudioListener>>,
    mut state: ResMut<DemoState>,
) {
    let now = Instant::now();
    let delta_time = now.duration_since(state.last_update).as_secs_f32();
    state.last_update = now;

    if let Some(mut transform) = listener_query.iter_mut().next() {
        let move_speed = 5.0;
        let mut movement = Vec3::ZERO;

        if input.is_key_pressed(KeyCode::KeyW) {
            movement.z -= 1.0;
        }
        if input.is_key_pressed(KeyCode::KeyS) {
            movement.z += 1.0;
        }
        if input.is_key_pressed(KeyCode::KeyA) {
            movement.x -= 1.0;
        }
        if input.is_key_pressed(KeyCode::KeyD) {
            movement.x += 1.0;
        }
        if input.is_key_pressed(KeyCode::Space) {
            movement.y += 1.0;
        }
        if input.is_key_pressed(KeyCode::ShiftLeft) {
            movement.y -= 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize() * move_speed * delta_time;
            transform.translation += movement;

            if delta_time > 0.1 {
                info!(
                    "Listener position: ({:.1}, {:.1}, {:.1})",
                    transform.translation.x, transform.translation.y, transform.translation.z
                );
            }
        }
    }
}

fn start_audio_sources(mut audio_sources: Query<&mut AudioSource>) {
    for mut source in audio_sources.iter_mut() {
        if source.is_stopped() {
            source.play();
        }
    }
}

fn update_moving_sources(
    mut moving_sources: Query<(&mut Transform, &MovingSource), Without<AudioListener>>,
    state: Res<DemoState>,
) {
    let now = Instant::now();
    let delta_time = now.duration_since(state.last_update).as_secs_f32();
    
    for (mut transform, moving) in moving_sources.iter_mut() {
        // Move in the direction
        let movement = moving.direction * moving.speed * delta_time;
        transform.translation += movement;
        
        // Bounce off boundaries
        if transform.translation.length() > 30.0 {
            // Reverse direction when hitting boundary
            let to_center = -transform.translation.normalize();
            // Update the MovingSource direction (note: this is read-only, so we just move back)
            transform.translation = transform.translation.clamp_length_max(30.0);
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;

    info!("=== Praxis Audio Demo ===");
    info!("This demo showcases 3D positional audio with:");
    info!("  - Distance-based attenuation");
    info!("  - Stereo panning");
    info!("  - Doppler effect for moving sources");
    info!("  - Listener transform synchronization");

    let mut world = World::new();

    let audio_manager = AudioManager::new()?;
    world.insert_resource(audio_manager);

    let input_state = InputState::new();
    world.insert_resource(input_state);

    world.insert_resource(DemoState::new());

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            setup_audio_scene,
            start_audio_sources,
            update_listener_position,
            update_moving_sources,
            play_sound_system,
            update_listener_system,
        )
            .chain(),
    );

    info!("Starting audio demo loop");
    info!("Press Ctrl+C to exit");

    for _ in 0..600 {
        world.inner_mut().run_schedule(&mut schedule);

        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Audio demo finished");
    Ok(())
}
