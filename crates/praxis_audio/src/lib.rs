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
//! use praxis_audio::{AudioManager, AudioSource, play_sound_system};
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
//! // Spawn an entity with spatial audio
//! world.spawn((
//!     praxis_ecs::Transform::from_xyz(5.0, 0.0, 0.0),
//!     AudioSource::new("explosion.ogg")
//!         .with_volume(0.8)
//!         .with_spatial(true),
//! ));
//! ```
//!
//! # Spatial Audio
//!
//! Spatial audio automatically adjusts volume and panning based on the distance
//! between the audio source and the listener (camera). The attenuation follows
//! an inverse square law for realistic distance falloff.
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

mod components;
mod manager;
mod systems;

pub use components::*;
pub use manager::*;
pub use systems::*;

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
mod tests {
    use super::*;

    #[test]
    fn test_audio_system_initialization() {
        let result = init();
        assert!(result.is_ok());
    }
}
