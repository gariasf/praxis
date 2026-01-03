//! Audio system demonstration.
//!
//! This example demonstrates the audio system with spatial audio.
//! It spawns several audio sources at different positions and a listener (camera).
//! Use WASD to move the camera and hear how the spatial audio changes.

use praxis_audio::{play_sound_system, AudioListener, AudioManager, AudioSource};
use praxis_ecs::{
    Commands, IntoSystemConfigs, Query, Res, ResMut, Resource, Schedule, Transform, With, World,
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

fn setup_audio_scene(mut commands: Commands, mut audio_manager: ResMut<AudioManager>) {
    info!("Setting up audio demo scene");

    info!("Creating audio listener (camera)");
    commands.spawn((Transform::from_xyz(0.0, 1.8, 0.0), AudioListener));

    info!("Note: This demo requires audio files to be placed in assets/sounds/");
    info!("Example files: ambient.ogg, beep.ogg, etc.");

    info!("Creating spatial audio sources");
    commands.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_volume(0.7)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));

    commands.spawn((
        Transform::from_xyz(-10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/beep.ogg")
            .with_volume(0.5)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0),
    ));

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 15.0),
        AudioSource::new("assets/sounds/wind.ogg")
            .with_volume(0.6)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(60.0)
            .with_reference_distance(8.0),
    ));

    info!("Audio scene setup complete");
    info!("Use WASD to move the listener");
    info!("Use Space to jump");
    info!("Use Escape to exit");
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

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;

    info!("=== Praxis Audio Demo ===");

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
            play_sound_system,
        )
            .chain(),
    );

    info!("Starting audio demo loop");
    info!("Press Ctrl+C to exit");

    for _ in 0..300 {
        world.inner_mut().run_schedule(&mut schedule);

        std::thread::sleep(Duration::from_millis(16));
    }

    info!("Audio demo finished");
    Ok(())
}
