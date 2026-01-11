//! Audio system demonstration with positional 3D sounds.
//!
//! This example demonstrates the audio system with:
//! - 3D positional audio that follows moving entities
//! - Distance-based attenuation
//! - Stereo panning based on listener position
//! - Doppler effect for moving sources
//! - Multiple audio sources with different behaviors
//! - Interactive listener movement with WASD controls

use praxis_audio::{
    play_sound_system, update_listener_system, AudioListener, AudioManager, AudioSource,
};
use praxis_ecs::{
    Commands, Component, Entity, Query, Res, ResMut, Resource, Schedule, Transform, With, Without,
    World,
};
use praxis_input::InputState;
use praxis_math::Vec3;
use praxis_utils::{info, Result};
use std::time::{Duration, Instant};
use winit::keyboard::KeyCode;

#[derive(Resource)]
struct DemoState {
    last_update: Instant,
    elapsed_time: f32,
}

impl DemoState {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            elapsed_time: 0.0,
        }
    }

    fn update(&mut self) -> f32 {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        self.elapsed_time += delta_time;
        delta_time
    }
}

/// Marker component for moving audio sources
#[derive(Component)]
struct MovingSource {
    movement_type: MovementType,
    speed: f32,
}

#[derive(Debug, Clone, Copy)]
enum MovementType {
    Circular {
        radius: f32,
        start_angle: f32,
    },
    BackAndForth {
        axis: Vec3,
        distance: f32,
    },
    Spiral {
        radius_speed: f32,
        angular_speed: f32,
    },
}

/// Component for audio sources that orbit around a point
#[derive(Component)]
struct OrbitingSource {
    center: Vec3,
    radius: f32,
    speed: f32,
    angle: f32,
}

/// Component for audio sources that follow a path
#[derive(Component)]
struct PathFollower {
    waypoints: Vec<Vec3>,
    current_waypoint: usize,
    speed: f32,
}

fn setup_audio_scene(mut commands: Commands) {
    info!("Setting up audio demo scene");

    // Spawn audio listener (camera)
    info!("Creating audio listener at origin");
    commands.spawn((Transform::from_xyz(0.0, 1.8, 0.0), AudioListener));

    info!("Note: This demo requires audio files to be placed in assets/sounds/");
    info!("Example files needed:");
    info!("  - ambient.ogg (looping background ambience)");
    info!("  - beep.ogg (short beep sound)");
    info!("  - wind.ogg (wind/whoosh sound)");
    info!("  - music.ogg (background music)");

    // Stationary ambient source at the origin
    info!("Creating stationary ambient source at origin");
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_volume(0.5)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(30.0)
            .with_reference_distance(3.0),
    ));

    // Orbiting sound source (circling around the listener)
    info!("Creating orbiting sound source");
    commands.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/beep.ogg")
            .with_volume(0.7)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0)
            .with_doppler(true)
            .with_doppler_scale(1.5),
        OrbitingSource {
            center: Vec3::ZERO,
            radius: 10.0,
            speed: 0.5, // radians per second
            angle: 0.0,
        },
    ));

    // Fast-moving source with strong doppler
    info!("Creating fast-moving source with doppler effect");
    commands.spawn((
        Transform::from_xyz(15.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/wind.ogg")
            .with_volume(0.8)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(60.0)
            .with_reference_distance(8.0)
            .with_doppler(true)
            .with_doppler_scale(2.0),
        MovingSource {
            movement_type: MovementType::Circular {
                radius: 15.0,
                start_angle: 0.0,
            },
            speed: 8.0, // units per second
        },
    ));

    // Source moving back and forth
    info!("Creating back-and-forth moving source");
    commands.spawn((
        Transform::from_xyz(-10.0, 0.0, 0.0),
        AudioSource::new("assets/sounds/beep.ogg")
            .with_volume(0.6)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(40.0)
            .with_reference_distance(5.0)
            .with_doppler(true)
            .with_doppler_scale(1.0),
        MovingSource {
            movement_type: MovementType::BackAndForth {
                axis: Vec3::new(1.0, 0.0, 0.0),
                distance: 20.0,
            },
            speed: 5.0,
        },
    ));

    // Spiral moving source
    info!("Creating spiral moving source");
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 5.0),
        AudioSource::new("assets/sounds/wind.ogg")
            .with_volume(0.7)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(6.0)
            .with_doppler(true)
            .with_doppler_scale(1.2),
        MovingSource {
            movement_type: MovementType::Spiral {
                radius_speed: 2.0,
                angular_speed: 2.0,
            },
            speed: 1.0,
        },
    ));

    // Path-following source
    info!("Creating path-following source");
    let waypoints = vec![
        Vec3::new(10.0, 0.0, 10.0),
        Vec3::new(-10.0, 0.0, 10.0),
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(10.0, 0.0, -10.0),
    ];
    commands.spawn((
        Transform::from_xyz(10.0, 0.0, 10.0),
        AudioSource::new("assets/sounds/ambient.ogg")
            .with_volume(0.6)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(45.0)
            .with_reference_distance(7.0)
            .with_doppler(false),
        PathFollower {
            waypoints,
            current_waypoint: 0,
            speed: 4.0,
        },
    ));

    // Stationary music source far away
    info!("Creating distant music source");
    commands.spawn((
        Transform::from_xyz(0.0, 5.0, 25.0),
        AudioSource::new("assets/sounds/music.ogg")
            .with_volume(0.4)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(80.0)
            .with_reference_distance(15.0),
    ));

    info!("Audio scene setup complete");
    info!("");
    info!("Controls:");
    info!("  W/A/S/D - Move listener horizontally");
    info!("  Space - Move listener up");
    info!("  Shift - Move listener down");
    info!("  ESC - Exit demo");
    info!("");
    info!("Listen for:");
    info!("  - Distance-based volume changes");
    info!("  - Stereo panning as sounds move left/right");
    info!("  - Doppler pitch shifts on fast-moving sources");
}

fn update_listener_position(
    input: Res<InputState>,
    mut listener_query: Query<&mut Transform, With<AudioListener>>,
    mut state: ResMut<DemoState>,
) {
    let delta_time = state.update();

    for mut transform in listener_query.iter_mut() {
        let move_speed = 8.0;
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
        if input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight) {
            movement.y -= 1.0;
        }

        if movement.length_squared() > 0.0 {
            movement = movement.normalize() * move_speed * delta_time;
            transform.translation += movement;

            // Clamp to reasonable bounds
            transform.translation.x = transform.translation.x.clamp(-50.0, 50.0);
            transform.translation.y = transform.translation.y.clamp(0.5, 20.0);
            transform.translation.z = transform.translation.z.clamp(-50.0, 50.0);
        }
    }
}

fn start_audio_sources(_audio_manager: Option<ResMut<AudioManager>>, mut audio_sources: Query<&mut AudioSource>) {
    // Early return if no audio manager available
    if _audio_manager.is_none() {
        return;
    }

    for mut source in audio_sources.iter_mut() {
        if source.is_stopped() {
            source.play();
        }
    }
}

fn update_orbiting_sources(
    mut query: Query<(&mut Transform, &mut OrbitingSource), Without<AudioListener>>,
    state: Res<DemoState>,
) {
    for (mut transform, mut orbit) in query.iter_mut() {
        let delta_time = state.last_update.elapsed().as_secs_f32().min(0.1);

        orbit.angle += orbit.speed * delta_time;

        // Update position in circular orbit
        let x = orbit.center.x + orbit.radius * orbit.angle.cos();
        let z = orbit.center.z + orbit.radius * orbit.angle.sin();

        transform.translation = Vec3::new(x, orbit.center.y, z);
    }
}

fn update_moving_sources(
    mut query: Query<(&mut Transform, &MovingSource, Entity)>,
    state: Res<DemoState>,
) {
    let time = state.elapsed_time;

    for (mut transform, moving, _entity) in query.iter_mut() {
        match moving.movement_type {
            MovementType::Circular {
                radius,
                start_angle,
            } => {
                let angle = start_angle + time * moving.speed;
                transform.translation.x = radius * angle.cos();
                transform.translation.z = radius * angle.sin();
            }
            MovementType::BackAndForth { axis, distance } => {
                let offset = (time * moving.speed).sin() * distance;
                transform.translation = axis * offset;
            }
            MovementType::Spiral {
                radius_speed,
                angular_speed,
            } => {
                let angle = time * angular_speed;
                let radius = 5.0 + (time * radius_speed).sin() * 10.0;
                transform.translation.x = radius * angle.cos();
                transform.translation.z = radius * angle.sin();
            }
        }
    }
}

fn update_path_followers(
    mut query: Query<(&mut Transform, &mut PathFollower)>,
    state: Res<DemoState>,
) {
    for (mut transform, mut follower) in query.iter_mut() {
        if follower.waypoints.is_empty() {
            continue;
        }

        let target = follower.waypoints[follower.current_waypoint];
        let direction = target - transform.translation;
        let distance = direction.length();

        if distance < 1.0 {
            // Reached waypoint, move to next
            follower.current_waypoint = (follower.current_waypoint + 1) % follower.waypoints.len();
        } else {
            // Move towards waypoint
            let delta_time = state.last_update.elapsed().as_secs_f32().min(0.1);
            let movement = direction.normalize() * follower.speed * delta_time;

            if movement.length() <= distance {
                transform.translation += movement;
            } else {
                transform.translation = target;
            }
        }
    }
}

fn print_status_system(
    listener_query: Query<&Transform, With<AudioListener>>,
    sources_query: Query<&Transform, (With<AudioSource>, Without<AudioListener>)>,
    state: Res<DemoState>,
) {
    // Print status every 2 seconds
    if state.elapsed_time % 2.0 < 0.016 {
        if let Some(listener_transform) = listener_query.iter().next() {
            let source_count = sources_query.iter().count();

            info!("Status Update:");
            info!(
                "  Listener position: ({:.1}, {:.1}, {:.1})",
                listener_transform.translation.x,
                listener_transform.translation.y,
                listener_transform.translation.z
            );
            info!("  Active audio sources: {}", source_count);

            // Print closest source
            let mut closest_distance = f32::MAX;
            for source_transform in sources_query.iter() {
                let distance =
                    (source_transform.translation - listener_transform.translation).length();
                if distance < closest_distance {
                    closest_distance = distance;
                }
            }

            if closest_distance < f32::MAX {
                info!("  Closest source: {:.1} units away", closest_distance);
            }
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;

    info!("=== Praxis Audio Demo ===");
    info!("3D Positional Audio with Moving Sources");
    info!("");

    let mut world = World::new();

    // Initialize resources
    let audio_manager = match AudioManager::new() {
        Ok(manager) => {
            info!("Audio manager initialized successfully");
            manager
        }
        Err(e) => {
            info!("Failed to initialize audio manager: {}", e);
            info!("This is expected in environments without audio output (e.g., CI)");
            info!("Exiting demo gracefully.");
            return Ok(());
        }
    };
    world.insert_resource(audio_manager);

    let input_state = InputState::new();
    world.insert_resource(input_state);

    world.insert_resource(DemoState::new());

    // Create schedule - add systems individually since large tuple chain is not supported
    let mut schedule = Schedule::default();
    schedule.add_systems(setup_audio_scene);
    schedule.add_systems(start_audio_sources);
    schedule.add_systems(update_listener_position);
    schedule.add_systems(update_orbiting_sources);
    schedule.add_systems(update_moving_sources);
    schedule.add_systems(update_path_followers);
    schedule.add_systems(play_sound_system);
    schedule.add_systems(update_listener_system);
    schedule.add_systems(print_status_system);

    info!("Starting audio demo loop...");
    info!("Running for 60 seconds (press Ctrl+C to exit earlier)");
    info!("");

    // Run for 60 seconds at 60 FPS
    let total_frames = 60 * 60; // 60 seconds at 60 FPS

    for frame in 0..total_frames {
        schedule.run(world.inner_mut());

        // Check for exit key
        if let Some(input) = world.inner().get_resource::<InputState>() {
            if input.is_key_pressed(KeyCode::Escape) {
                info!("ESC pressed, exiting demo");
                break;
            }
        }

        // Progress indicator every 5 seconds
        if frame % (60 * 5) == 0 && frame > 0 {
            info!("");
            info!("Demo running... ({} seconds elapsed)", frame / 60);
            info!("");
        }

        std::thread::sleep(Duration::from_millis(16));
    }

    info!("");
    info!("Audio demo finished");
    info!("Total sounds demonstrated:");
    info!("  ✓ Stationary ambient sound");
    info!("  ✓ Orbiting sound with doppler effect");
    info!("  ✓ Fast circular motion with strong doppler");
    info!("  ✓ Back-and-forth movement");
    info!("  ✓ Spiral movement pattern");
    info!("  ✓ Path-following behavior");
    info!("  ✓ Distant stationary music source");

    Ok(())
}
