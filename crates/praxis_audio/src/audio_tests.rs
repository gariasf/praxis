//! Comprehensive tests for audio system.

#[cfg(test)]
mod tests {
    use crate::*;
    use crate::systems::{calculate_spatial_params, calculate_doppler_factor};
    use praxis_math::Vec3;

    // ============================================================================
    // Audio Source Management Tests
    // ============================================================================

    #[test]
    fn test_audio_source_creation() {
        let source = AudioSource::new("test.ogg");
        
        assert_eq!(source.path, "test.ogg");
        assert_eq!(source.volume, 1.0);
        assert_eq!(source.spatial, false);
        assert_eq!(source.looping, false);
        assert!(matches!(source.state, AudioState::Stopped));
        assert!(source.sound_handle.is_none());
    }

    #[test]
    fn test_audio_source_builder_pattern() {
        let source = AudioSource::new("test.ogg")
            .with_volume(0.5)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(5.0)
            .with_doppler(true)
            .with_doppler_scale(1.5);

        assert_eq!(source.path, "test.ogg");
        assert_eq!(source.volume, 0.5);
        assert_eq!(source.spatial, true);
        assert_eq!(source.looping, true);
        assert_eq!(source.max_distance, 50.0);
        assert_eq!(source.reference_distance, 5.0);
        assert_eq!(source.doppler_enabled, true);
        assert_eq!(source.doppler_scale, 1.5);
    }

    #[test]
    fn test_audio_source_volume_clamping() {
        let source = AudioSource::new("test.ogg")
            .with_volume(2.0);
        
        // Volume should be clamped to 1.0
        assert_eq!(source.volume, 1.0);

        let source = AudioSource::new("test.ogg")
            .with_volume(-0.5);
        
        // Volume should be clamped to 0.0
        assert_eq!(source.volume, 0.0);
    }

    #[test]
    fn test_audio_source_state_transitions() {
        let mut source = AudioSource::new("test.ogg");
        
        assert!(source.is_stopped());
        assert!(!source.is_playing());
        assert!(!source.is_paused());

        source.play();
        assert!(matches!(source.state, AudioState::Playing));
        assert!(source.is_playing());
        assert!(!source.is_stopped());
        assert!(!source.is_paused());

        source.pause();
        assert!(matches!(source.state, AudioState::Paused));
        assert!(source.is_paused());
        assert!(!source.is_playing());
        assert!(!source.is_stopped());

        source.stop();
        assert!(matches!(source.state, AudioState::Stopped));
        assert!(source.is_stopped());
        assert!(!source.is_playing());
        assert!(!source.is_paused());
    }

    #[test]
    fn test_audio_listener_creation() {
        let listener = AudioListener;
        // Just verify it can be created
        let _ = listener;
    }

    #[test]
    fn test_playback_settings_defaults() {
        let settings = PlaybackSettings::default();
        
        assert_eq!(settings.volume, 1.0);
        assert_eq!(settings.looping, false);
        assert_eq!(settings.panning, 0.0);
    }

    #[test]
    fn test_playback_settings_builder() {
        let settings = PlaybackSettings::new()
            .with_volume(0.7)
            .with_looping(true)
            .with_panning(0.5);

        assert_eq!(settings.volume, 0.7);
        assert_eq!(settings.looping, true);
        assert_eq!(settings.panning, 0.5);
    }

    #[test]
    fn test_playback_settings_volume_clamping() {
        let settings = PlaybackSettings::new().with_volume(2.0);
        assert_eq!(settings.volume, 1.0);

        let settings = PlaybackSettings::new().with_volume(-1.0);
        assert_eq!(settings.volume, 0.0);
    }

    #[test]
    fn test_playback_settings_panning_clamping() {
        let settings = PlaybackSettings::new().with_panning(2.0);
        assert_eq!(settings.panning, 1.0);

        let settings = PlaybackSettings::new().with_panning(-2.0);
        assert_eq!(settings.panning, -1.0);
    }

    #[test]
    fn test_sound_handle_creation() {
        let handle = SoundHandle { id: 42 };
        assert_eq!(handle.id, 42);
    }

    // ============================================================================
    // Spatial Audio Attenuation Tests
    // ============================================================================

    #[test]
    fn test_spatial_attenuation_at_reference_distance() {
        let source_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // At reference distance, attenuation should be 1.0
        assert!((params.attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_attenuation_closer_than_reference() {
        let source_pos = Vec3::new(5.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Closer than reference distance should still be 1.0
        assert!((params.attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_attenuation_inverse_square_law() {
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        // Test at 20 units (2x reference)
        let source_pos_20 = Vec3::new(20.0, 0.0, 0.0);
        let params_20 = calculate_spatial_params(
            source_pos_20,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Attenuation should be (10/20)^2 = 0.25
        assert!((params_20.attenuation - 0.25).abs() < 0.001);

        // Test at 40 units (4x reference)
        let source_pos_40 = Vec3::new(40.0, 0.0, 0.0);
        let params_40 = calculate_spatial_params(
            source_pos_40,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Attenuation should be (10/40)^2 = 0.0625
        assert!((params_40.attenuation - 0.0625).abs() < 0.001);
    }

    #[test]
    fn test_spatial_attenuation_at_max_distance() {
        let source_pos = Vec3::new(100.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // At max distance, attenuation should be 0.0
        assert_eq!(params.attenuation, 0.0);
    }

    #[test]
    fn test_spatial_attenuation_beyond_max_distance() {
        let source_pos = Vec3::new(150.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Beyond max distance, attenuation should be 0.0
        assert_eq!(params.attenuation, 0.0);
    }

    #[test]
    fn test_spatial_attenuation_3d_distance() {
        let source_pos = Vec3::new(3.0, 4.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 5.0;
        let max_distance = 100.0;

        // Distance is sqrt(3^2 + 4^2) = 5.0
        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // At reference distance, attenuation should be 1.0
        assert!((params.attenuation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_spatial_attenuation_diagonal_distance() {
        let source_pos = Vec3::new(10.0, 10.0, 10.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        // Distance is sqrt(10^2 + 10^2 + 10^2) = sqrt(300) ≈ 17.32
        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        let expected_distance = (300.0_f32).sqrt();
        let expected_attenuation = (reference_distance / expected_distance).powi(2);
        assert!((params.attenuation - expected_attenuation).abs() < 0.01);
    }

    #[test]
    fn test_spatial_panning_center() {
        let source_pos = Vec3::new(0.0, 0.0, 10.0); // In front of listener
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Panning should be close to center (0.0)
        assert!(params.panning.abs() < 0.01);
    }

    #[test]
    fn test_spatial_panning_right() {
        let source_pos = Vec3::new(10.0, 0.0, 0.0); // To the right
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Panning should be positive (right)
        assert!(params.panning > 0.0);
        assert!(params.panning <= 1.0);
    }

    #[test]
    fn test_spatial_panning_left() {
        let source_pos = Vec3::new(-10.0, 0.0, 0.0); // To the left
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Panning should be negative (left)
        assert!(params.panning < 0.0);
        assert!(params.panning >= -1.0);
    }

    #[test]
    fn test_spatial_panning_clamping() {
        let source_pos = Vec3::new(200.0, 0.0, 0.0); // Far to the right
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 100.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Panning should be clamped to 1.0
        assert!((params.panning - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_doppler_approaching() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(9.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let doppler_scale = 1.0;

        let factor = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            doppler_scale,
        );

        // Should be > 1.0 (higher pitch) when approaching
        assert!(factor > 1.0);
        assert!(factor <= 2.0);
    }

    #[test]
    fn test_doppler_receding() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(11.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let doppler_scale = 1.0;

        let factor = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            doppler_scale,
        );

        // Should be < 1.0 (lower pitch) when receding
        assert!(factor < 1.0);
        assert!(factor >= 0.5);
    }

    #[test]
    fn test_doppler_stationary() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(10.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let doppler_scale = 1.0;

        let factor = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            doppler_scale,
        );

        // Should be approximately 1.0 when not moving
        assert!((factor - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_doppler_disabled() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(9.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;
        let doppler_scale = 0.0;

        let factor = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            doppler_scale,
        );

        // Should be 1.0 when disabled
        assert_eq!(factor, 1.0);
    }

    #[test]
    fn test_doppler_perpendicular_motion() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(10.0, 1.0, 0.0); // Moving perpendicular
        let listener_pos = Vec3::ZERO;
        let doppler_scale = 1.0;

        let factor = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            doppler_scale,
        );

        // Should be approximately 1.0 for perpendicular motion
        assert!((factor - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_doppler_scale_effect() {
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(9.0, 0.0, 0.0);
        let listener_pos = Vec3::ZERO;

        let factor_normal = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            1.0,
        );

        let factor_exaggerated = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            2.0,
        );

        // Exaggerated scale should produce more extreme doppler shift
        assert!(factor_exaggerated > factor_normal || factor_exaggerated < 1.0);
    }

    #[test]
    fn test_doppler_clamping() {
        // Very fast approaching velocity
        let previous_pos = Vec3::new(10.0, 0.0, 0.0);
        let current_pos = Vec3::new(0.0, 0.0, 0.0); // Extreme velocity
        let listener_pos = Vec3::ZERO;
        let doppler_scale = 10.0; // High scale

        let factor = calculate_doppler_factor(
            previous_pos,
            current_pos,
            listener_pos,
            doppler_scale,
        );

        // Should be clamped to reasonable range
        assert!(factor >= 0.5);
        assert!(factor <= 2.0);
    }

    #[test]
    fn test_combined_spatial_effects() {
        // Test a realistic scenario with distance and panning
        let source_pos = Vec3::new(15.0, 0.0, 5.0);
        let listener_pos = Vec3::ZERO;
        let reference_distance = 10.0;
        let max_distance = 50.0;

        let params = calculate_spatial_params(
            source_pos,
            listener_pos,
            reference_distance,
            max_distance,
        );

        // Distance is sqrt(15^2 + 5^2) ≈ 15.81
        let distance = (15.0_f32.powi(2) + 5.0_f32.powi(2)).sqrt();
        let expected_attenuation = (reference_distance / distance).powi(2);

        assert!((params.attenuation - expected_attenuation).abs() < 0.01);
        assert!(params.panning > 0.0); // Should be panned right
        assert!(params.panning < 1.0);
    }

    #[test]
    fn test_audio_source_default_distances() {
        let source = AudioSource::new("test.ogg");
        
        assert_eq!(source.max_distance, 100.0);
        assert_eq!(source.reference_distance, 1.0);
    }

    #[test]
    fn test_audio_source_custom_distances() {
        let source = AudioSource::new("test.ogg")
            .with_max_distance(250.0)
            .with_reference_distance(10.0);
        
        assert_eq!(source.max_distance, 250.0);
        assert_eq!(source.reference_distance, 10.0);
    }

    #[test]
    fn test_audio_manager_sound_counters() {
        // This test would require an actual AudioManager instance
        // which requires audio backend initialization
        // For now, we just verify the API exists
        let result = AudioManager::new();
        if let Ok(manager) = result {
            assert_eq!(manager.loaded_sound_count(), 0);
            assert_eq!(manager.playing_sound_count(), 0);
        }
    }

    #[test]
    fn test_multiple_audio_sources() {
        let source1 = AudioSource::new("sound1.ogg");
        let source2 = AudioSource::new("sound2.ogg");
        let source3 = AudioSource::new("sound3.ogg");

        assert_eq!(source1.path, "sound1.ogg");
        assert_eq!(source2.path, "sound2.ogg");
        assert_eq!(source3.path, "sound3.ogg");
    }
}
