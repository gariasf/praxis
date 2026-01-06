//! Audio system for the Praxis engine.
//!
//! This crate provides audio playback capabilities using the Kira audio library,
//! including spatial audio support for 3D sound positioning.
//!
//! # Architecture
//!
//! The audio system consists of:
//! - **`AudioManager`**: Central resource managing the audio backend and loaded sounds
//! - **`AudioSource`**: Component for spatial audio attached to entities
//! - **`play_sound_system`**: System that processes audio playback requests
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_audio::{AudioManager, AudioSource, AudioListener, play_sound_system};
//! use praxis_ecs::{World, Schedule};
//! use praxis_math::Vec3;
//!
//! let mut world = World::new();
//! let audio_manager = AudioManager::new().expect("Failed to initialize audio");
//! world.insert_resource(audio_manager);
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems(play_sound_system);
//!
//! // Spawn a listener (typically attached to the camera)
//! world.spawn((
//!     praxis_ecs::Transform::from_xyz(0.0, 0.0, 0.0),
//!     AudioListener,
//! ));
//!
//! // Spawn an entity with spatial audio
//! world.spawn((
//!     praxis_ecs::Transform::from_xyz(5.0, 0.0, 0.0),
//!     AudioSource::new("explosion.ogg")
//!         .with_volume(0.8)
//!         .with_spatial(true)
//!         .with_doppler(true),
//! ));
//! ```
//!
//! # Spatial Audio
//!
//! Spatial audio automatically adjusts volume and panning based on the distance
//! between the audio source and the listener (camera). The attenuation follows
//! an inverse square law for realistic distance falloff.
//!
//! # Doppler Effect
//!
//! The doppler effect simulates pitch changes based on relative velocity between
//! the audio source and listener. Enable it with `.with_doppler(true)` on the
//! `AudioSource` component. The effect scales with velocity and can be adjusted
//! with `.with_doppler_scale()`.
//!
//! # Loading Sounds
//!
//! The `AudioManager` supports loading various audio formats through Kira:
//! - OGG Vorbis
//! - MP3
//! - WAV
//! - FLAC
//!
//! Sounds are cached by path to avoid redundant loading.
//!
//! # Testing and Mocking
//!
//! For headless testing without audio backend initialization, use `MockAudioManager`:
//!
//! ```rust,no_run
//! # #[cfg(test)]
//! # {
//! use praxis_audio::MockAudioManager;
//!
//! let mut manager = MockAudioManager::new();
//! // All audio operations are no-ops
//! // Suitable for testing game logic without audio hardware
//! manager.load_sound("test.ogg").unwrap();
//! let id = manager.play_sound("test.ogg", Default::default()).unwrap();
//! # }
//! ```
//!
//! The mock provides the same API surface as `AudioManager` but with all
//! operations as no-ops, allowing tests to run in CI environments without
//! audio hardware.

mod components;
mod manager;
mod systems;

#[cfg(test)]
mod audio_tests;

pub use components::*;
pub use manager::*;
pub use systems::*;

#[cfg(test)]
pub use manager::MockAudioManager;

use praxis_utils::{info, Result};

/// Initializes the audio system.
///
/// This function sets up any necessary global state for the audio system.
///
/// # Example
///
/// ```rust,no_run
/// praxis_audio::init().expect("Failed to initialize audio system");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails.
pub fn init() -> Result<()> {
    info!("Audio system initialized");
    Ok(())
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_audio_system_initialization() {
        let result = init();
        assert!(result.is_ok());
    }
}
