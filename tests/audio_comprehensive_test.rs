//! Comprehensive integration tests for audio system functionality.

use praxis_audio::{AudioListener, AudioManager, AudioSource, PlaybackSettings};
use serial_test::serial;

/// Helper macro to conditionally skip tests when audio backend is unavailable
macro_rules! skip_if_no_audio_backend {
    ($manager_result:expr) => {
        if $manager_result.is_err() {
            eprintln!("Audio backend not available, skipping test");
            return;
        }
    };
}

#[test]
#[serial]
fn test_audio_manager_creation() {
    let result = AudioManager::new();

    // Audio manager creation may fail if no audio backend is available
    // This is expected in CI environments
    skip_if_no_audio_backend!(result);

    let manager = result.unwrap();
    assert_eq!(manager.loaded_sound_count(), 0);
    assert_eq!(manager.playing_sound_count(), 0);
}

#[test]
fn test_playback_settings_configuration() {
    let settings = PlaybackSettings::new()
        .with_volume(0.8)
        .with_looping(true)
        .with_panning(0.5);

    assert_eq!(settings.volume, 0.8);
    assert!(settings.looping);
    assert_eq!(settings.panning, 0.5);
}

#[test]
fn test_audio_source_configuration() {
    let source = AudioSource::new("test.ogg")
        .with_volume(0.6)
        .with_spatial(true)
        .with_looping(false)
        .with_max_distance(75.0)
        .with_reference_distance(7.5)
        .with_doppler(true)
        .with_doppler_scale(1.2);

    assert_eq!(source.path, "test.ogg");
    assert_eq!(source.volume, 0.6);
    assert!(source.spatial);
    assert!(!source.looping);
    assert_eq!(source.max_distance, 75.0);
    assert_eq!(source.reference_distance, 7.5);
    assert!(source.doppler_enabled);
    assert_eq!(source.doppler_scale, 1.2);
}

#[test]
fn test_audio_listener_component() {
    let listener = AudioListener;
    // Verify the listener can be created and used
    let _ = listener;
}

#[test]
fn test_audio_source_state_management() {
    let mut source = AudioSource::new("ambient.ogg");

    // Initial state
    assert!(source.is_stopped());

    // Play
    source.play();
    assert!(source.is_playing());
    assert!(!source.is_stopped());
    assert!(!source.is_paused());

    // Pause
    source.pause();
    assert!(source.is_paused());
    assert!(!source.is_playing());
    assert!(!source.is_stopped());

    // Stop
    source.stop();
    assert!(source.is_stopped());
    assert!(!source.is_playing());
    assert!(!source.is_paused());
}

#[test]
fn test_multiple_audio_sources_management() {
    let mut sources = Vec::new();

    for i in 0..10 {
        let source = AudioSource::new(format!("sound_{i}.ogg"))
            .with_volume(0.5 + (i as f32 * 0.05))
            .with_spatial(i % 2 == 0);
        sources.push(source);
    }

    assert_eq!(sources.len(), 10);

    // Verify each source has correct properties
    for (i, source) in sources.iter().enumerate() {
        assert_eq!(source.path, format!("sound_{i}.ogg"));
        assert_eq!(source.spatial, i % 2 == 0);
    }
}

#[test]
fn test_spatial_audio_distance_configuration() {
    let source = AudioSource::new("distant.ogg")
        .with_spatial(true)
        .with_max_distance(200.0)
        .with_reference_distance(20.0);

    assert_eq!(source.max_distance, 200.0);
    assert_eq!(source.reference_distance, 20.0);
    assert!(source.spatial);
}

#[test]
fn test_doppler_configuration() {
    let source = AudioSource::new("moving.ogg")
        .with_doppler(true)
        .with_doppler_scale(1.5);

    assert!(source.doppler_enabled);
    assert_eq!(source.doppler_scale, 1.5);
}

#[test]
fn test_audio_source_volume_limits() {
    // Test upper limit
    let source_high = AudioSource::new("loud.ogg").with_volume(5.0);
    assert_eq!(source_high.volume, 1.0);

    // Test lower limit
    let source_low = AudioSource::new("silent.ogg").with_volume(-1.0);
    assert_eq!(source_low.volume, 0.0);

    // Test valid range
    let source_mid = AudioSource::new("normal.ogg").with_volume(0.7);
    assert_eq!(source_mid.volume, 0.7);
}

#[test]
fn test_playback_settings_panning_limits() {
    // Test left limit
    let settings_left = PlaybackSettings::new().with_panning(-5.0);
    assert_eq!(settings_left.panning, -1.0);

    // Test right limit
    let settings_right = PlaybackSettings::new().with_panning(5.0);
    assert_eq!(settings_right.panning, 1.0);

    // Test valid range
    let settings_center = PlaybackSettings::new().with_panning(0.3);
    assert_eq!(settings_center.panning, 0.3);
}

#[test]
fn test_audio_source_doppler_configuration() {
    // Test that doppler can be enabled/disabled via the public API
    let source = AudioSource::new("moving.ogg").with_doppler(true);
    assert!(source.doppler_enabled);

    let source_no_doppler = AudioSource::new("static.ogg").with_doppler(false);
    assert!(!source_no_doppler.doppler_enabled);

    // Test doppler scale configuration
    let source_scaled = AudioSource::new("scaled.ogg")
        .with_doppler(true)
        .with_doppler_scale(0.5);
    assert!(source_scaled.doppler_enabled);
    assert_eq!(source_scaled.doppler_scale, 0.5);
}

#[test]
fn test_audio_system_integration_setup() {
    // Test that all components can be created together
    let listener = AudioListener;

    let source1 = AudioSource::new("bgm.ogg")
        .with_volume(0.5)
        .with_looping(true);

    let source2 = AudioSource::new("sfx.ogg")
        .with_volume(0.8)
        .with_spatial(true)
        .with_max_distance(50.0);

    let settings = PlaybackSettings::new().with_volume(0.7);

    // Verify all components exist
    let _ = listener;
    let _ = source1;
    let _ = source2;
    let _ = settings;
}

// Tests that require actual audio device interaction
// These are skipped on Windows where audio device availability may vary

#[test]
#[serial]
#[cfg(not(target_os = "windows"))]
fn test_audio_manager_with_device() {
    let result = AudioManager::new();
    skip_if_no_audio_backend!(result);

    let manager = result.unwrap();

    // Verify manager starts with clean state
    assert_eq!(manager.loaded_sound_count(), 0);
    assert_eq!(manager.playing_sound_count(), 0);
}

#[test]
#[serial]
#[cfg(not(target_os = "windows"))]
fn test_audio_backend_initialization() {
    let result = AudioManager::new();
    skip_if_no_audio_backend!(result);

    // If we reach here, the audio backend was successfully initialized
    let _manager = result.unwrap();
}
