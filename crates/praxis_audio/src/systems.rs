//! Audio systems for the ECS.

use crate::{AudioListener, AudioManager, AudioSource, AudioState, PlaybackSettings, SoundHandle};
use praxis_ecs::{Changed, Query, ResMut, Transform, With};
use praxis_math::Vec3;

/// Speed of sound in world units per second.
/// Default is 343.0 (approximate speed of sound in air at 20°C in m/s).
const SPEED_OF_SOUND: f32 = 343.0;

/// System that processes audio playback requests with full 3D spatial audio support.
///
/// This system should be added to your schedule to handle audio playback:
/// - Starts playing sounds when `AudioSource.state` is set to `Playing`
/// - Updates spatial audio based on entity position and listener position
/// - Applies distance attenuation, panning, and doppler effect
/// - Stops sounds when requested
/// - Cleans up finished sounds
///
/// # Example
///
/// ```rust,no_run
/// use praxis_audio::play_sound_system;
/// use praxis_ecs::Schedule;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(play_sound_system);
/// ```
pub fn play_sound_system(
    mut audio_manager: ResMut<AudioManager>,
    mut audio_sources: Query<(&mut AudioSource, Option<&Transform>)>,
    listener_query: Query<&Transform, With<AudioListener>>,
) {
    let listener_transform = listener_query.iter().next();

    for (mut source, transform) in &mut audio_sources {
        match source.state {
            AudioState::Playing => {
                if source.sound_handle.is_none() {
                    // Start playing new sound
                    let mut settings = PlaybackSettings::new()
                        .with_volume(source.volume)
                        .with_looping(source.looping);

                    if source.spatial {
                        if let (Some(source_pos), Some(listener_trans)) =
                            (transform.map(|t| t.translation), listener_transform)
                        {
                            let spatial_params = calculate_spatial_params(
                                source_pos,
                                listener_trans.translation,
                                source.reference_distance,
                                source.max_distance,
                            );
                            settings.volume = source.volume * spatial_params.attenuation;
                            settings.panning = spatial_params.panning;

                            // Initialize previous position for doppler
                            source.previous_position = Some(source_pos);
                        }
                    }

                    if let Ok(sound_id) = audio_manager.play_sound(&source.path, settings) {
                        source.sound_handle = Some(SoundHandle { id: sound_id });
                    }
                } else if source.spatial {
                    // Update spatial audio for playing sounds
                    if let (Some(source_pos), Some(listener_trans)) =
                        (transform.map(|t| t.translation), listener_transform)
                    {
                        if let Some(handle) = &source.sound_handle {
                            // Calculate and apply spatial parameters
                            let spatial_params = calculate_spatial_params(
                                source_pos,
                                listener_trans.translation,
                                source.reference_distance,
                                source.max_distance,
                            );

                            let final_volume = source.volume * spatial_params.attenuation;
                            let _ = audio_manager.set_sound_volume(handle.id, final_volume);
                            let _ =
                                audio_manager.set_sound_panning(handle.id, spatial_params.panning);

                            // Apply doppler effect if enabled
                            if source.doppler_enabled {
                                if let Some(prev_pos) = source.previous_position {
                                    let doppler_factor = calculate_doppler_factor(
                                        prev_pos,
                                        source_pos,
                                        listener_trans.translation,
                                        source.doppler_scale,
                                    );
                                    let _ = audio_manager
                                        .set_sound_playback_rate(handle.id, doppler_factor);
                                }
                                source.previous_position = Some(source_pos);
                            }
                        }
                    }
                }
            }
            AudioState::Paused => {
                if let Some(handle) = &source.sound_handle {
                    let _ = audio_manager.pause_sound(handle.id);
                }
            }
            AudioState::Stopped => {
                if let Some(handle) = source.sound_handle.take() {
                    let _ = audio_manager.stop_sound(handle.id);
                    source.previous_position = None;
                }
            }
        }
    }

    audio_manager.cleanup_finished_sounds();
}

/// System that automatically updates spatial audio when transforms change.
///
/// This is an optimized system that only processes audio sources whose
/// transforms have changed, or when the listener transform changes, avoiding
/// unnecessary calculations.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_audio::{play_sound_system, update_spatial_audio_system};
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     play_sound_system,
///     update_spatial_audio_system,
/// ).chain());
/// ```
pub fn update_spatial_audio_system(
    mut audio_manager: ResMut<AudioManager>,
    mut audio_sources: Query<(&mut AudioSource, &Transform), Changed<Transform>>,
    listener_query: Query<&Transform, With<AudioListener>>,
) {
    let listener_transform = listener_query.iter().next();

    if let Some(listener_trans) = listener_transform {
        for (mut source, transform) in &mut audio_sources {
            if source.spatial && source.is_playing() {
                if let Some(handle) = &source.sound_handle {
                    let source_pos = transform.translation;

                    // Calculate spatial parameters
                    let spatial_params = calculate_spatial_params(
                        source_pos,
                        listener_trans.translation,
                        source.reference_distance,
                        source.max_distance,
                    );

                    let final_volume = source.volume * spatial_params.attenuation;
                    let _ = audio_manager.set_sound_volume(handle.id, final_volume);
                    let _ = audio_manager.set_sound_panning(handle.id, spatial_params.panning);

                    // Apply doppler effect if enabled
                    if source.doppler_enabled {
                        if let Some(prev_pos) = source.previous_position {
                            let doppler_factor = calculate_doppler_factor(
                                prev_pos,
                                source_pos,
                                listener_trans.translation,
                                source.doppler_scale,
                            );
                            let _ =
                                audio_manager.set_sound_playback_rate(handle.id, doppler_factor);
                        }
                        source.previous_position = Some(source_pos);
                    }
                }
            }
        }
    }
}

/// System that updates spatial audio when listener transform changes.
///
/// This system detects when the listener (camera) moves and updates all
/// spatial audio sources accordingly. This is important for maintaining
/// correct audio positioning relative to the listener.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_audio::{play_sound_system, update_listener_system};
/// use praxis_ecs::{Schedule, IntoSystemConfigs};
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems((
///     play_sound_system,
///     update_listener_system,
/// ).chain());
/// ```
pub fn update_listener_system(
    mut audio_manager: ResMut<AudioManager>,
    mut audio_sources: Query<(&mut AudioSource, &Transform)>,
    listener_query: Query<&Transform, (With<AudioListener>, Changed<Transform>)>,
) {
    // Only update if listener transform changed
    if let Some(listener_trans) = listener_query.iter().next() {
        for (mut source, transform) in &mut audio_sources {
            if source.spatial && source.is_playing() {
                if let Some(handle) = &source.sound_handle {
                    let source_pos = transform.translation;

                    // Calculate spatial parameters
                    let spatial_params = calculate_spatial_params(
                        source_pos,
                        listener_trans.translation,
                        source.reference_distance,
                        source.max_distance,
                    );

                    let final_volume = source.volume * spatial_params.attenuation;
                    let _ = audio_manager.set_sound_volume(handle.id, final_volume);
                    let _ = audio_manager.set_sound_panning(handle.id, spatial_params.panning);

                    // Apply doppler effect if enabled
                    if source.doppler_enabled {
                        if let Some(prev_pos) = source.previous_position {
                            let doppler_factor = calculate_doppler_factor(
                                prev_pos,
                                source_pos,
                                listener_trans.translation,
                                source.doppler_scale,
                            );
                            let _ =
                                audio_manager.set_sound_playback_rate(handle.id, doppler_factor);
                        }
                        source.previous_position = Some(source_pos);
                    }
                }
            }
        }
    }
}

/// Spatial audio parameters calculated from source and listener positions.
#[derive(Debug, Clone, Copy)]
pub struct SpatialParams {
    /// Volume attenuation factor (0.0 to 1.0).
    pub attenuation: f32,
    /// Stereo panning (-1.0 left, 0.0 center, 1.0 right).
    pub panning: f32,
}

/// Calculates spatial audio parameters based on source and listener positions.
///
/// Uses inverse square law for realistic audio attenuation and computes
/// stereo panning based on relative position.
///
/// # Arguments
///
/// * `source_pos` - Position of the audio source
/// * `listener_pos` - Position of the listener
/// * `reference_distance` - Distance at which the base volume applies
/// * `max_distance` - Distance beyond which the sound is inaudible
///
/// # Returns
///
/// Spatial parameters including attenuation and panning
#[must_use]
pub fn calculate_spatial_params(
    source_pos: Vec3,
    listener_pos: Vec3,
    reference_distance: f32,
    max_distance: f32,
) -> SpatialParams {
    let distance = source_pos.distance(listener_pos);

    // Calculate attenuation
    let attenuation = if distance >= max_distance {
        0.0
    } else if distance <= reference_distance {
        1.0
    } else {
        let ratio = reference_distance / distance;
        (ratio * ratio).clamp(0.0, 1.0)
    };

    // Calculate panning based on relative position
    let relative_pos = source_pos - listener_pos;
    // Simple left-right panning based on X-axis
    let panning = (relative_pos.x / max_distance).clamp(-1.0, 1.0);

    SpatialParams {
        attenuation,
        panning,
    }
}

/// Calculates the doppler effect factor for pitch shifting.
///
/// The doppler effect causes the pitch of a sound to increase as the source
/// approaches the listener and decrease as it moves away. This is calculated
/// using the classic doppler formula.
///
/// # Arguments
///
/// * `previous_pos` - Previous position of the source
/// * `current_pos` - Current position of the source
/// * `listener_pos` - Position of the listener
/// * `doppler_scale` - Scale factor for the doppler effect (0.0 to disable, 1.0 for normal)
///
/// # Returns
///
/// Playback rate factor (1.0 = normal, >1.0 = higher pitch, <1.0 = lower pitch)
#[must_use]
pub fn calculate_doppler_factor(
    previous_pos: Vec3,
    current_pos: Vec3,
    listener_pos: Vec3,
    doppler_scale: f32,
) -> f32 {
    if doppler_scale <= 0.0 {
        return 1.0;
    }

    // Calculate velocity (assuming fixed timestep approximation)
    let velocity = current_pos - previous_pos;

    // Calculate direction from source to listener
    let to_listener = listener_pos - current_pos;
    let distance = to_listener.length();

    if distance < 0.001 {
        return 1.0; // Avoid division by zero
    }

    let direction = to_listener / distance;

    // Calculate radial velocity (component of velocity towards listener)
    let radial_velocity = velocity.dot(direction);

    // Apply doppler formula: f' = f * (v + v_observer) / (v + v_source)
    // Where v is speed of sound, v_observer is 0 (stationary listener),
    // and v_source is the radial velocity
    let doppler_shift = SPEED_OF_SOUND / radial_velocity.mul_add(-doppler_scale, SPEED_OF_SOUND);

    // Clamp to reasonable range to avoid extreme pitch shifts
    doppler_shift.clamp(0.5, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_spatial_params_at_reference_distance() {
        let source_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params =
            calculate_spatial_params(source_pos, listener_pos, reference_distance, max_distance);

        assert!((params.attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_spatial_params_closer_than_reference() {
        let source_pos = Vec3::new(5.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params =
            calculate_spatial_params(source_pos, listener_pos, reference_distance, max_distance);

        assert!((params.attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_spatial_params_farther_than_reference() {
        let source_pos = Vec3::new(20.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params =
            calculate_spatial_params(source_pos, listener_pos, reference_distance, max_distance);

        assert!(params.attenuation < 1.0);
        assert!(params.attenuation > 0.0);
    }

    #[test]
    fn test_calculate_spatial_params_beyond_max_distance() {
        let source_pos = Vec3::new(150.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params =
            calculate_spatial_params(source_pos, listener_pos, reference_distance, max_distance);

        assert_eq!(params.attenuation, 0.0);
    }

    #[test]
    fn test_calculate_spatial_params_panning() {
        let source_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params =
            calculate_spatial_params(source_pos, listener_pos, reference_distance, max_distance);

        assert!(params.panning > 0.0); // Right side
        assert!(params.panning <= 1.0);
    }

    #[test]
    fn test_calculate_doppler_factor_approaching() {
        // Source moving towards listener
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(9.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let doppler_scale = 1.0;

        let factor =
            calculate_doppler_factor(previous_pos, current_pos, listener_pos, doppler_scale);

        // Should be > 1.0 (higher pitch) when approaching
        assert!(factor > 1.0);
        assert!(factor <= 2.0);
    }

    #[test]
    fn test_calculate_doppler_factor_receding() {
        // Source moving away from listener
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(11.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let doppler_scale = 1.0;

        let factor =
            calculate_doppler_factor(previous_pos, current_pos, listener_pos, doppler_scale);

        // Should be < 1.0 (lower pitch) when receding
        assert!(factor < 1.0);
        assert!(factor >= 0.5);
    }

    #[test]
    fn test_calculate_doppler_factor_stationary() {
        // Source not moving
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let doppler_scale = 1.0;

        let factor =
            calculate_doppler_factor(previous_pos, current_pos, listener_pos, doppler_scale);

        // Should be approximately 1.0 when not moving
        assert!((factor - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_doppler_factor_disabled() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(9.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let doppler_scale = 0.0;

        let factor =
            calculate_doppler_factor(previous_pos, current_pos, listener_pos, doppler_scale);

        // Should be 1.0 when disabled
        assert_eq!(factor, 1.0);
    }
}
