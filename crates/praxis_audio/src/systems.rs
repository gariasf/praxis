//! Audio systems for the ECS.
//!
//! # System Architecture
//!
//! The audio system uses three complementary ECS systems to handle spatial audio:
//!
//! 1. **`play_sound_system`**: Main system that handles sound playback lifecycle
//!    - Starts new sounds when AudioSource.state changes to Playing
//!    - Updates spatial parameters for all playing sounds
//!    - Stops sounds when requested
//!    - Cleans up finished sounds
//!
//! 2. **`update_spatial_audio_system`**: Optimized system for transform changes
//!    - Only processes `AudioSources` with changed Transforms
//!    - Uses ECS change detection to minimize work
//!    - Recalculates spatial parameters when source moves
//!
//! 3. **`update_listener_system`**: Handles listener (camera) movement
//!    - Only runs when listener Transform changes
//!    - Updates all spatial audio sources relative to new listener position
//!    - Ensures consistent audio positioning during camera movement
//!
//! These systems work together to provide efficient, reactive spatial audio
//! processing without unnecessary calculations.
//!
//! # Spatial Audio Pipeline
//!
//! ```text
//! Frame Start
//!     ↓
//! [Check for Transform changes] ──→ update_spatial_audio_system
//!     ↓                                      ↓
//! [Check for listener changes] ──→ update_listener_system
//!     ↓                                      ↓
//! [Process audio state changes] ──→ play_sound_system
//!     ↓                                      ↓
//! [Calculate spatial parameters] ────────────┘
//!     ↓
//! [Apply to AudioManager]
//!     ↓
//! [Kira applies changes to audio thread]
//!     ↓
//! Audio Output
//! ```

use crate::{AudioListener, AudioManager, AudioSource, AudioState, PlaybackSettings, SoundHandle};
use praxis_ecs::{Changed, Query, ResMut, Transform, With};
use praxis_math::Vec3;

/// Speed of sound in world units per second.
/// Default is 343.0 (approximate speed of sound in air at 20°C in m/s).
///
/// This constant is used in doppler effect calculations. Adjust if your
/// world units don't match meters (e.g., if 1 unit = 1 cm, use 34300.0).
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
/// # Graceful Degradation
///
/// The system accepts `Option<ResMut<AudioManager>>` instead of `ResMut<AudioManager>`,
/// which means it will gracefully skip processing if no `AudioManager` resource is present.
/// This allows:
/// - Running in headless/CI environments without audio hardware
/// - Conditional audio system initialization
/// - Better error handling when audio backend fails to initialize
///
/// # Processing Logic
///
/// For each `AudioSource`:
/// 1. **Playing State**: Start new sound or update existing spatial parameters
/// 2. **Paused State**: Pause the sound in the audio backend
/// 3. **Stopped State**: Stop the sound and clean up the handle
///
/// # Spatial Audio Parameters
///
/// When spatial audio is enabled, the system calculates:
/// - **Distance Attenuation**: Volume reduction based on distance (inverse square law)
/// - **Stereo Panning**: Left/right positioning based on X-axis offset
/// - **Doppler Effect**: Pitch shift based on velocity (if enabled)
///
/// # Performance Characteristics
///
/// - O(n) where n = number of `AudioSource` components
/// - Early exits for non-spatial or stopped sounds
/// - Cleanup runs once per frame (O(m) where m = playing sounds)
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
#[allow(clippy::needless_pass_by_value)] // Query must be passed by value for ECS systems
pub fn play_sound_system(
    audio_manager: Option<ResMut<AudioManager>>,
    mut audio_sources: Query<(&mut AudioSource, Option<&Transform>)>,
    listener_query: Query<&Transform, With<AudioListener>>,
) {
    // Early return if no audio manager is available
    // This allows the system to run without panicking in environments
    // where audio initialization failed (e.g., headless CI)
    let Some(mut audio_manager) = audio_manager else {
        return;
    };

    // Find the listener (camera) position
    // If multiple listeners exist, only the first is used
    // If no listener exists, spatial audio won't have positioning
    let listener_transform = listener_query.iter().next();

    for (mut source, transform) in &mut audio_sources {
        match source.state {
            AudioState::Playing => {
                if source.sound_handle.is_none() {
                    // === START NEW SOUND ===
                    // This is the first frame where the sound is playing

                    // Initialize playback settings from component configuration
                    let mut settings = PlaybackSettings::new()
                        .with_volume(source.volume)
                        .with_looping(source.looping);

                    // Apply spatial audio if enabled and both positions are available
                    if source.spatial {
                        if let (Some(source_pos), Some(listener_trans)) =
                            (transform.map(|t| t.translation), listener_transform)
                        {
                            // Calculate initial spatial parameters
                            let spatial_params = calculate_spatial_params(
                                source_pos,
                                listener_trans.translation,
                                source.reference_distance,
                                source.max_distance,
                            );

                            // Apply attenuation and panning to initial settings
                            settings.volume = source.volume * spatial_params.attenuation;
                            settings.panning = spatial_params.panning;

                            // Store initial position for doppler velocity calculation
                            source.previous_position = Some(source_pos);
                        }
                    }

                    // Send play command to audio manager
                    if let Ok(sound_id) = audio_manager.play_sound(&source.path, settings) {
                        // Store handle for future control operations
                        source.sound_handle = Some(SoundHandle { id: sound_id });
                    }
                } else if source.spatial {
                    // === UPDATE EXISTING SPATIAL SOUND ===
                    // Sound is already playing, update its spatial parameters

                    if let (Some(source_pos), Some(listener_trans)) =
                        (transform.map(|t| t.translation), listener_transform)
                    {
                        if let Some(handle) = &source.sound_handle {
                            // Recalculate spatial parameters based on current positions
                            let spatial_params = calculate_spatial_params(
                                source_pos,
                                listener_trans.translation,
                                source.reference_distance,
                                source.max_distance,
                            );

                            // Apply updated volume (base volume × distance attenuation)
                            let final_volume = source.volume * spatial_params.attenuation;
                            let _ = audio_manager.set_sound_volume(handle.id, final_volume);

                            // Apply updated stereo panning
                            let _ =
                                audio_manager.set_sound_panning(handle.id, spatial_params.panning);

                            // Apply doppler effect if enabled
                            if source.doppler_enabled {
                                if let Some(prev_pos) = source.previous_position {
                                    // Calculate pitch shift based on velocity
                                    let doppler_factor = calculate_doppler_factor(
                                        prev_pos,
                                        source_pos,
                                        listener_trans.translation,
                                        source.doppler_scale,
                                    );
                                    let _ = audio_manager
                                        .set_sound_playback_rate(handle.id, doppler_factor);
                                }
                                // Update previous position for next frame's velocity calculation
                                source.previous_position = Some(source_pos);
                            }
                        }
                    }
                }
            }
            AudioState::Paused => {
                // Pause the sound in the audio backend
                if let Some(handle) = &source.sound_handle {
                    let _ = audio_manager.pause_sound(handle.id);
                }
            }
            AudioState::Stopped => {
                // Stop and clean up the sound
                if let Some(handle) = source.sound_handle.take() {
                    let _ = audio_manager.stop_sound(handle.id);
                    // Clear cached position data
                    source.previous_position = None;
                }
            }
        }
    }

    // Clean up finished sounds from the manager's pool
    // This removes handles for sounds that have completed playback
    audio_manager.cleanup_finished_sounds();
}

/// System that automatically updates spatial audio when transforms change.
///
/// This is an optimized system that only processes audio sources whose
/// transforms have changed, or when the listener transform changes, avoiding
/// unnecessary calculations.
///
/// # Graceful Degradation
///
/// Like `play_sound_system`, this system accepts `Option<ResMut<AudioManager>>`
/// and will gracefully skip processing if the audio manager is not available.
///
/// # Change Detection
///
/// Uses ECS change tracking (`Changed<Transform>`) to detect movement:
/// - Only processes entities that moved this frame
/// - Prevents redundant spatial calculations for stationary sources
/// - Significantly reduces CPU usage in scenes with many audio sources
///
/// # When to Use
///
/// Add this system if you have:
/// - Many audio sources in the scene
/// - Sources that don't move every frame
/// - Performance concerns with spatial audio processing
///
/// # System Ordering
///
/// Should run after transform propagation systems to ensure global transforms
/// are up-to-date before spatial calculations.
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
#[allow(clippy::needless_pass_by_value)] // Query must be passed by value for ECS systems
pub fn update_spatial_audio_system(
    audio_manager: Option<ResMut<AudioManager>>,
    mut audio_sources: Query<(&mut AudioSource, &Transform), Changed<Transform>>,
    listener_query: Query<&Transform, With<AudioListener>>,
) {
    // Early return if no audio manager is available
    let Some(mut audio_manager) = audio_manager else {
        return;
    };

    let listener_transform = listener_query.iter().next();

    if let Some(listener_trans) = listener_transform {
        // Only iterate over sources with changed transforms
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
/// # Graceful Degradation
///
/// Like other audio systems, this accepts `Option<ResMut<AudioManager>>`
/// and will gracefully skip processing if the audio manager is not available.
///
/// # Listener Movement Handling
///
/// When the camera moves:
/// 1. Detect listener transform change via `Changed<Transform>`
/// 2. Iterate through ALL spatial audio sources
/// 3. Recalculate spatial parameters relative to new listener position
/// 4. Apply updated volume and panning to audio backend
///
/// This ensures that audio remains correctly positioned even when sources
/// are stationary but the listener moves.
///
/// # Performance Considerations
///
/// - Only runs when listener actually moves (change detection)
/// - Processes all spatial sources when triggered (O(n))
/// - For most games, listener moves more than most audio sources
/// - Consider using `update_spatial_audio_system` for moving sources instead
///
/// # Single Listener Model
///
/// Only one listener is supported. If multiple `AudioListener` components exist,
/// only the first one found is used. This prevents ambiguity in spatial calculations.
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
#[allow(clippy::needless_pass_by_value)] // Query must be passed by value for ECS systems
pub fn update_listener_system(
    audio_manager: Option<ResMut<AudioManager>>,
    mut audio_sources: Query<(&mut AudioSource, &Transform)>,
    listener_query: Query<&Transform, (With<AudioListener>, Changed<Transform>)>,
) {
    // Early return if no audio manager is available
    let Some(mut audio_manager) = audio_manager else {
        return;
    };

    // Only update if listener transform changed this frame
    if let Some(listener_trans) = listener_query.iter().next() {
        // Update all spatial audio sources relative to new listener position
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
/// # Attenuation Model
///
/// The volume attenuation follows three regions:
///
/// 1. **Near Field** (distance ≤ `reference_distance`):
///    - Attenuation = 1.0 (full volume)
///    - Prevents excessive volume at very close distances
///    - Example: `reference_distance` = 1.0 → full volume within 1 unit
///
/// 2. **Far Field** (`reference_distance` < distance < `max_distance`):
///    - Attenuation = (`reference_distance` / distance)²
///    - Inverse square law mimics real-world sound propagation
///    - Sound pressure decreases with distance squared
///    - Example: distance = 2× reference → volume = 0.25 (25%)
///
/// 3. **Out of Range** (distance ≥ `max_distance`):
///    - Attenuation = 0.0 (silent)
///    - Hard cutoff for performance and realism
///    - Distant sounds shouldn't be audible
///
/// # Attenuation Curve Examples
///
/// With `reference_distance` = 1.0, `max_distance` = 100.0:
/// ```text
/// Distance | Attenuation | Volume % | Real World Equivalent
/// ---------+-------------+----------+-----------------------
/// 0.5      | 1.00        | 100%     | Very close (< 1m)
/// 1.0      | 1.00        | 100%     | Reference (1m)
/// 2.0      | 0.25        | 25%      | 2m away
/// 5.0      | 0.04        | 4%       | 5m away
/// 10.0     | 0.01        | 1%       | 10m away
/// 50.0     | 0.0004      | 0.04%    | 50m away (barely audible)
/// 100.0+   | 0.00        | 0%       | Too far (silent)
/// ```
///
/// # Panning Model
///
/// Simple stereo panning based on X-axis offset:
/// - Calculated as: `panning = (source.x - listener.x) / max_distance`
/// - Clamped to [-1.0, 1.0] range
/// - Limitations:
///   - No front/back distinction (uses only X-axis)
///   - No head-related transfer function (HRTF)
///   - Simple stereo model, not true 3D audio
///
/// Future improvements could include:
/// - HRTF for 3D positioning
/// - Ambisonics for surround sound
/// - Elevation-based filtering
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
    // Calculate Euclidean distance between source and listener
    let distance = source_pos.distance(listener_pos);

    // === CALCULATE ATTENUATION ===
    // Three-region model for realistic sound falloff
    let attenuation = if distance >= max_distance {
        // Out of range - complete silence
        0.0
    } else if distance <= reference_distance {
        // Near field - maintain full volume
        1.0
    } else {
        // Far field - inverse square law
        // Formula: attenuation = (r / d)²
        // where r = reference_distance, d = actual distance
        let ratio = reference_distance / distance;
        (ratio * ratio).clamp(0.0, 1.0)
    };

    // === CALCULATE PANNING ===
    // Simple left-right panning based on X-axis relative position
    let relative_pos = source_pos - listener_pos;

    // Normalize by max_distance to prevent extreme panning for nearby sources
    // Positive X = right, Negative X = left
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
/// # Doppler Effect Physics
///
/// The doppler shift formula for sound:
/// ```text
/// f' = f × (v + v_observer) / (v + v_source)
/// ```
///
/// Where:
/// - `f'` = perceived frequency (what listener hears)
/// - `f` = emitted frequency (actual sound)
/// - `v` = speed of sound (343 m/s in air)
/// - `v_observer` = velocity of listener (0 in our case)
/// - `v_source` = radial velocity of source (toward listener)
///
/// Simplified for stationary listener:
/// ```text
/// f' = f × v / (v - v_radial)
/// ```
///
/// # Velocity Calculation
///
/// Since we don't have explicit velocity data, we approximate it:
/// ```text
/// velocity ≈ (current_position - previous_position) / frame_time
/// ```
///
/// Frame time is implicit (assumed constant), so we use position delta directly.
///
/// # Radial Velocity
///
/// Only the component of velocity toward the listener affects doppler:
/// ```text
/// v_radial = velocity · direction_to_listener
/// ```
///
/// This dot product gives us:
/// - Positive: source moving away (lower pitch)
/// - Negative: source approaching (higher pitch)
/// - Zero: perpendicular motion (no doppler)
///
/// # Playback Rate Mapping
///
/// The frequency shift maps to playback rate:
/// - rate > 1.0: higher pitch (approaching)
/// - rate = 1.0: normal pitch (stationary)
/// - rate < 1.0: lower pitch (receding)
///
/// # Clamping
///
/// The result is clamped to [0.5, 2.0] to prevent:
/// - Extreme pitch shifts (unrealistic)
/// - Audio artifacts from very high/low rates
/// - Division by zero issues
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
    // Early exit if doppler is disabled
    if doppler_scale <= 0.0 {
        return 1.0;
    }

    // Calculate velocity vector (approximation using position delta)
    // In reality: velocity = delta_position / delta_time
    // We assume constant frame time, so we use delta_position directly
    let velocity = current_pos - previous_pos;

    // Calculate direction vector from source to listener
    let to_listener = listener_pos - current_pos;
    let distance = to_listener.length();

    // Avoid division by zero for sources at same position as listener
    if distance < 0.001 {
        return 1.0; // No doppler when source and listener are at same location
    }

    // Normalize to get unit direction vector
    let direction = to_listener / distance;

    // Calculate radial velocity (component of velocity toward listener)
    // Dot product projects velocity onto direction vector
    // Positive = moving away, Negative = approaching
    let radial_velocity = velocity.dot(direction);

    // Apply classic doppler formula: f' = f × v / (v - v_radial)
    // Rearranged for playback rate: rate = v / (v - v_radial × doppler_scale)
    // Using fused multiply-add for better precision
    let doppler_shift = SPEED_OF_SOUND / radial_velocity.mul_add(-doppler_scale, SPEED_OF_SOUND);

    // Clamp to reasonable range to prevent extreme pitch shifts
    // 0.5 = half speed (one octave down)
    // 2.0 = double speed (one octave up)
    doppler_shift.clamp(0.5, 2.0)
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
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
