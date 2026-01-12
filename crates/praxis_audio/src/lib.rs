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
//! # Component-Based Audio Attachment Pattern
//!
//! Praxis uses an ECS-based approach to audio where sounds are attached to entities
//! as components. This pattern provides several benefits:
//!
//! 1. **Entity Coupling**: Audio sources naturally move with their entities through
//!    the Transform hierarchy, eliminating the need for manual position synchronization.
//!
//! 2. **Lifecycle Management**: Audio sources are automatically cleaned up when
//!    entities are despawned, preventing orphaned sound handles and memory leaks.
//!
//! 3. **Query-Based Processing**: The audio system uses ECS queries to efficiently
//!    process only active audio sources, avoiding iteration over inactive sounds.
//!
//! 4. **Serialization**: Audio configurations can be saved with scene data and
//!    restored on load, enabling persistent audio setups.
//!
//! The typical workflow is:
//! ```text
//! Entity Spawn → AudioSource Component → play_sound_system → AudioManager → Kira Backend
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_audio::{AudioManager, AudioSource, AudioListener, play_sound_system};
//! use praxis_ecs::{World, Schedule};
//! use praxis_math::Vec3;
//!
//! let mut world = World::new();
//!
//! // AudioManager initialization may fail in headless environments
//! // The audio systems will gracefully handle the absence of AudioManager
//! if let Ok(audio_manager) = AudioManager::new() {
//!     world.insert_resource(audio_manager);
//! }
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
//! ## Attenuation Curve
//!
//! The audio system implements a physically-based attenuation model:
//!
//! - **Below reference distance**: Volume remains at maximum (no attenuation)
//! - **Between reference and max distance**: Inverse square law attenuation
//!   - Formula: `attenuation = (reference_distance / distance)²`
//!   - This mimics real-world sound propagation through air
//! - **Beyond max distance**: Sound is completely inaudible (0.0 volume)
//!
//! Example attenuation curve with `reference_distance=1.0`, `max_distance=100.0`:
//! ```text
//! Distance (units) | Attenuation | Effective Volume
//! -----------------+-------------+-----------------
//! 0.0 - 1.0        | 1.00        | 100%
//! 2.0              | 0.25        | 25%
//! 5.0              | 0.04        | 4%
//! 10.0             | 0.01        | 1%
//! 100.0+           | 0.00        | 0%
//! ```
//!
//! ## Stereo Panning
//!
//! Simple left-right panning based on the X-axis relative position:
//! - Sound to the left of listener: negative panning (-1.0)
//! - Sound centered: zero panning (0.0)
//! - Sound to the right of listener: positive panning (+1.0)
//!
//! The panning is scaled by the `max_distance` to prevent extreme values for nearby sources.
//!
//! # Doppler Effect
//!
//! The doppler effect simulates pitch changes based on relative velocity between
//! the audio source and listener. Enable it with `.with_doppler(true)` on the
//! `AudioSource` component. The effect scales with velocity and can be adjusted
//! with `.with_doppler_scale()`.
//!
//! ## Doppler Formula
//!
//! The classic doppler shift formula is applied:
//! ```text
//! f' = f × (speed_of_sound) / (speed_of_sound - radial_velocity)
//! ```
//!
//! Where:
//! - `f'` is the perceived frequency (playback rate)
//! - `f` is the actual frequency (1.0 = normal playback)
//! - `radial_velocity` is the velocity component toward the listener
//!
//! The result is clamped to [0.5, 2.0] to prevent extreme pitch shifts.
//!
//! # 3D Listener Management
//!
//! The audio system uses a single-listener model:
//!
//! 1. **Listener Entity**: Typically attached to the camera entity with the
//!    `AudioListener` marker component.
//!
//! 2. **Position Tracking**: The listener's Transform is queried each frame
//!    to calculate spatial audio parameters.
//!
//! 3. **Multiple Listeners**: If multiple `AudioListener` components exist,
//!    only the first one found is used. This prevents ambiguity in spatial
//!    calculations.
//!
//! 4. **No Listener Fallback**: If no listener exists, spatial audio sources
//!    still play but without spatial effects applied (center panning, full volume).
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
//! ## Audio Source Pooling
//!
//! The `AudioManager` maintains two internal hash maps for efficient sound management:
//!
//! 1. **`loaded_sounds: HashMap<String, StaticSoundData>`**
//!    - Caches decoded audio data by file path
//!    - Prevents redundant disk I/O and decoding for reused sounds
//!    - Persists for the lifetime of the `AudioManager`
//!    - Memory trade-off: faster playback vs. RAM usage
//!
//! 2. **`playing_sounds: HashMap<u64, StaticSoundHandle>`**
//!    - Tracks active sound instances by unique ID
//!    - Enables real-time control (volume, pause, stop) of playing sounds
//!    - Cleaned up periodically via `cleanup_finished_sounds()`
//!    - Handles are automatically invalidated when sounds finish playing
//!
//! Sound ID Generation:
//! - Simple incrementing counter (`next_sound_id`) ensures unique IDs
//! - IDs are never reused, preventing handle confusion
//! - Allows multiple instances of the same sound to play simultaneously
//!
//! Example workflow:
//! ```text
//! play_sound("explosion.ogg")
//!   ↓
//! Check loaded_sounds cache
//!   ↓ (miss)
//! Load from disk → Store in loaded_sounds
//!   ↓
//! Clone StaticSoundData → Send to Kira
//!   ↓
//! Receive StaticSoundHandle
//!   ↓
//! Store in playing_sounds[id] → Return id to caller
//! ```
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
