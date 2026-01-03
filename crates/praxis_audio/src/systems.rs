//! Audio systems for the ECS.

use crate::{AudioListener, AudioManager, AudioSource, AudioState, PlaybackSettings, SoundHandle};
use praxis_ecs::{Changed, Query, ResMut, Transform, With};
use praxis_math::Vec3;

/// System that processes audio playback requests.
///
/// This system should be added to your schedule to handle audio playback:
/// - Starts playing sounds when `AudioSource.state` is set to `Playing`
/// - Updates spatial audio based on entity position and listener position
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
    listener_query: &Query<&Transform, With<AudioListener>>,
) {
    let listener_position = listener_query.iter().next().map(|t| t.translation);

    for (mut source, transform) in &mut audio_sources {
        match source.state {
            AudioState::Playing => {
                if source.sound_handle.is_none() {
                    let mut settings = PlaybackSettings::new()
                        .with_volume(source.volume)
                        .with_looping(source.looping);

                    if source.spatial {
                        if let (Some(source_pos), Some(listener_pos)) =
                            (transform.map(|t| t.translation), listener_position)
                        {
                            let adjusted_settings = apply_spatial_audio(
                                settings,
                                source_pos,
                                listener_pos,
                                source.reference_distance,
                                source.max_distance,
                            );
                            settings = adjusted_settings;
                        }
                    }

                    if let Ok(sound_id) = audio_manager.play_sound(&source.path, settings) {
                        source.sound_handle = Some(SoundHandle { id: sound_id });
                    }
                } else if source.spatial {
                    if let (Some(source_pos), Some(listener_pos)) =
                        (transform.map(|t| t.translation), listener_position)
                    {
                        if let Some(handle) = &source.sound_handle {
                            let volume = calculate_spatial_volume(
                                source.volume,
                                source_pos,
                                listener_pos,
                                source.reference_distance,
                                source.max_distance,
                            );
                            let _ = audio_manager.set_sound_volume(handle.id, volume);
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
                }
            }
        }
    }

    audio_manager.cleanup_finished_sounds();
}

/// System that automatically updates spatial audio when transforms change.
///
/// This is an optimized system that only processes audio sources whose
/// transforms have changed, avoiding unnecessary calculations.
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
    listener_query: &Query<&Transform, With<AudioListener>>,
) {
    let listener_position = listener_query.iter().next().map(|t| t.translation);

    if let Some(listener_pos) = listener_position {
        for (source, transform) in &mut audio_sources {
            if source.spatial && source.is_playing() {
                if let Some(handle) = &source.sound_handle {
                    let volume = calculate_spatial_volume(
                        source.volume,
                        transform.translation,
                        listener_pos,
                        source.reference_distance,
                        source.max_distance,
                    );
                    let _ = audio_manager.set_sound_volume(handle.id, volume);
                }
            }
        }
    }
}

/// Calculates the volume for spatial audio based on distance.
///
/// Uses inverse square law for realistic audio attenuation:
/// `volume = base_volume * (reference_distance / distance)^2`
///
/// # Arguments
///
/// * `base_volume` - The volume at reference distance (0.0 to 1.0)
/// * `source_pos` - Position of the audio source
/// * `listener_pos` - Position of the listener
/// * `reference_distance` - Distance at which the base volume applies
/// * `max_distance` - Distance beyond which the sound is inaudible
///
/// # Returns
///
/// The calculated volume (0.0 to 1.0)
fn calculate_spatial_volume(
    base_volume: f32,
    source_pos: Vec3,
    listener_pos: Vec3,
    reference_distance: f32,
    max_distance: f32,
) -> f32 {
    let distance = source_pos.distance(listener_pos);

    if distance >= max_distance {
        return 0.0;
    }

    if distance <= reference_distance {
        return base_volume;
    }

    let attenuation = reference_distance / distance;
    let volume = base_volume * attenuation * attenuation;
    volume.clamp(0.0, 1.0)
}

/// Applies spatial audio settings to playback settings.
///
/// This includes volume attenuation and panning based on the source
/// position relative to the listener.
///
/// # Arguments
///
/// * `settings` - Base playback settings
/// * `source_pos` - Position of the audio source
/// * `listener_pos` - Position of the listener
/// * `reference_distance` - Distance at which the base volume applies
/// * `max_distance` - Distance beyond which the sound is inaudible
///
/// # Returns
///
/// Updated playback settings with spatial audio applied
fn apply_spatial_audio(
    mut settings: PlaybackSettings,
    source_pos: Vec3,
    listener_pos: Vec3,
    reference_distance: f32,
    max_distance: f32,
) -> PlaybackSettings {
    let volume = calculate_spatial_volume(
        settings.volume,
        source_pos,
        listener_pos,
        reference_distance,
        max_distance,
    );
    settings.volume = volume;

    let relative_pos = source_pos - listener_pos;
    let panning = relative_pos.x.clamp(-1.0, 1.0) * 0.5;
    settings.panning = panning;

    settings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_spatial_volume_at_reference_distance() {
        let base_volume = 0.8;
        let source_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let volume = calculate_spatial_volume(
            base_volume,
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        assert!((volume - base_volume).abs() < 0.001);
    }

    #[test]
    fn test_calculate_spatial_volume_closer_than_reference() {
        let base_volume = 0.8;
        let source_pos = Vec3::new(5.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let volume = calculate_spatial_volume(
            base_volume,
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        assert!((volume - base_volume).abs() < 0.001);
    }

    #[test]
    fn test_calculate_spatial_volume_farther_than_reference() {
        let base_volume = 0.8;
        let source_pos = Vec3::new(20.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let volume = calculate_spatial_volume(
            base_volume,
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        assert!(volume < base_volume);
        assert!(volume > 0.0);
    }

    #[test]
    fn test_calculate_spatial_volume_beyond_max_distance() {
        let base_volume = 0.8;
        let source_pos = Vec3::new(150.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let volume = calculate_spatial_volume(
            base_volume,
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        assert_eq!(volume, 0.0);
    }

    #[test]
    fn test_apply_spatial_audio() {
        let settings = PlaybackSettings::new().with_volume(0.8);
        let source_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::new(0.0, 0.0, 0.0);
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let result = apply_spatial_audio(
            settings,
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        assert!((result.volume - 0.8).abs() < 0.001);
        assert!(result.panning.abs() > 0.0);
    }
}
