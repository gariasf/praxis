//! Audio manager for loading and managing sounds.
//!
//! # Kira Audio Backend Integration
//!
//! This module wraps the Kira audio library, providing a simplified interface
//! for the Praxis engine. Kira is a modern, real-time audio library for Rust
//! that handles audio playback, mixing, and effects.
//!
//! ## Architecture Overview
//!
//! ```text
//! AudioManager
//!     ↓
//! KiraAudioManager<DefaultBackend>
//!     ↓
//! [Audio Thread Pool] ─→ [Mixing Engine] ─→ [System Audio Output]
//!     ↓                         ↓
//! [Sound Handles]         [Volume/Pan/Rate Controls]
//! ```
//!
//! ## Key Kira Components Used
//!
//! 1. **`KiraAudioManager<DefaultBackend>`**: Core manager that handles audio output
//!    - `DefaultBackend`: Uses the system's default audio driver (cpal internally)
//!    - Manages audio thread and mixing
//!    - Provides API for playing and controlling sounds
//!
//! 2. **`StaticSoundData`**: Decoded audio data loaded from files
//!    - Loaded once and cached in memory
//!    - Can be cloned cheaply (uses Arc internally)
//!    - Supports multiple concurrent playbacks of the same data
//!
//! 3. **`StaticSoundHandle`**: Control handle for playing sound instances
//!    - Allows real-time control (volume, playback rate, panning)
//!    - Uses message-passing to audio thread (lock-free)
//!    - Automatically cleaned up when sound finishes
//!
//! 4. **`StaticSoundSettings`**: Configuration for sound playback
//!    - Volume (amplitude or decibel)
//!    - Looping (with optional loop region)
//!    - Playback rate (for pitch shifting / doppler)
//!    - Panning (stereo positioning)
//!
//! ## Thread Safety
//!
//! - Audio processing runs on a dedicated thread managed by Kira
//! - Control messages (play, stop, `set_volume`) use lock-free channels
//! - `StaticSoundData` uses Arc for safe sharing across threads
//! - No blocking operations in the audio thread
//!
//! ## Performance Characteristics
//!
//! - **Sound Loading**: Synchronous I/O + decoding (can be slow for large files)
//! - **Sound Playback**: Lock-free message passing (~microseconds)
//! - **Control Updates**: Lock-free message passing (~microseconds)
//! - **Memory**: Decoded audio stored uncompressed in RAM (high quality)
//!
//! Recommendation: Load sounds during initialization or loading screens,
//! not during gameplay for best performance.

use kira::{
    manager::{backend::DefaultBackend, AudioManager as KiraAudioManager, AudioManagerSettings},
    sound::{
        static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings},
        PlaybackRate,
    },
    tween::Tween,
    Volume,
};
use praxis_ecs::Resource;
use praxis_utils::Result;
use std::collections::HashMap;
use std::path::Path;

/// Central resource for managing audio playback.
///
/// The `AudioManager` wraps the Kira audio backend and provides
/// sound loading, caching, and playback management.
///
/// # Sound Pooling Strategy
///
/// The manager uses two-tier pooling for efficient memory usage:
///
/// 1. **Sound Data Cache** (`loaded_sounds`):
///    - Stores decoded audio data by file path
///    - Data is loaded once and reused for multiple playbacks
///    - Uses Arc-based sharing (cheap clones)
///    - Persists until manager is dropped
///
/// 2. **Active Sound Pool** (`playing_sounds`):
///    - Tracks currently playing sound instances
///    - Maps unique IDs to control handles
///    - Enables pause/resume/stop/volume control
///    - Automatically cleaned up when sounds finish
///
/// # Example
///
/// ```rust,no_run
/// use praxis_audio::AudioManager;
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// let audio_manager = AudioManager::new().expect("Failed to create audio manager");
///
/// world.insert_resource(audio_manager);
/// ```
#[derive(Resource)]
pub struct AudioManager {
    /// Kira's audio manager - handles audio thread and mixing.
    /// Uses `DefaultBackend` which automatically selects the system's audio driver.
    manager: KiraAudioManager<DefaultBackend>,

    /// Cache of loaded sound data keyed by file path.
    /// Prevents redundant disk I/O and decoding for frequently played sounds.
    /// `StaticSoundData` uses Arc internally, so clones are cheap.
    loaded_sounds: HashMap<String, StaticSoundData>,

    /// Pool of currently playing sounds keyed by unique ID.
    /// Enables real-time control of active sounds (volume, pause, stop).
    /// Cleaned up periodically to remove finished sounds.
    playing_sounds: HashMap<u64, StaticSoundHandle>,

    /// Incrementing counter for generating unique sound IDs.
    /// Never reused to prevent handle confusion.
    next_sound_id: u64,
}

impl AudioManager {
    /// Creates a new audio manager.
    ///
    /// This initializes the Kira audio backend with default settings:
    /// - Default audio device (system's preferred output)
    /// - Default buffer size (balanced latency/performance)
    /// - Default sample rate (typically 44.1kHz or 48kHz)
    ///
    /// The audio backend spawns a dedicated thread for audio processing,
    /// which runs independently of the game loop.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No audio output device is available
    /// - Audio backend initialization fails
    /// - System audio drivers are unavailable
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_audio::AudioManager;
    ///
    /// let manager = AudioManager::new().expect("Failed to initialize audio");
    /// ```
    pub fn new() -> Result<Self> {
        // Initialize Kira with default settings
        // DefaultBackend uses cpal internally, which supports Windows, macOS, Linux, etc.
        let manager = KiraAudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create audio manager: {}", e))?;

        Ok(Self {
            manager,
            loaded_sounds: HashMap::new(),
            playing_sounds: HashMap::new(),
            next_sound_id: 0,
        })
    }

    /// Loads a sound from a file.
    ///
    /// If the sound is already loaded, returns immediately without reloading.
    /// This implements the caching strategy that prevents redundant disk I/O.
    ///
    /// # Audio Format Support
    ///
    /// Kira supports the following formats through various decoders:
    /// - **OGG Vorbis**: Compressed, good quality, widely supported
    /// - **MP3**: Compressed, universal compatibility
    /// - **WAV**: Uncompressed, highest quality, large file size
    /// - **FLAC**: Lossless compression, excellent quality
    ///
    /// # Loading Process
    ///
    /// 1. Check if sound is already in cache
    /// 2. Read file from disk
    /// 3. Decode audio data (decompress if needed)
    /// 4. Store decoded data in RAM (uncompressed)
    /// 5. Cache for future playbacks
    ///
    /// # Performance Note
    ///
    /// Loading is synchronous and can block for several milliseconds to seconds
    /// depending on file size. Recommended to load sounds during initialization.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be loaded or decoded.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_audio::AudioManager;
    ///
    /// let mut manager = AudioManager::new().unwrap();
    /// manager.load_sound("assets/sounds/explosion.ogg").unwrap();
    /// ```
    pub fn load_sound(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Check cache first - early return if already loaded
        if self.loaded_sounds.contains_key(&path_str) {
            return Ok(());
        }

        // Load and decode the audio file
        // This reads the entire file and decodes it into memory
        let sound_data = StaticSoundData::from_file(path)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to load sound '{}': {}", path_str, e))?;

        // Cache the decoded data for future use
        self.loaded_sounds.insert(path_str, sound_data);
        Ok(())
    }

    /// Plays a sound with the given settings.
    ///
    /// This is the primary method for triggering sound playback. It handles:
    /// 1. Automatic loading (if not already cached)
    /// 2. Applying playback settings (volume, looping, etc.)
    /// 3. Sending play command to audio thread
    /// 4. Returning control handle for real-time manipulation
    ///
    /// # Sound Instance Management
    ///
    /// Each call creates a new sound instance with a unique ID, even for the
    /// same sound file. This allows multiple concurrent playbacks:
    /// ```text
    /// play_sound("gun.ogg") → ID 0
    /// play_sound("gun.ogg") → ID 1  // Plays simultaneously with ID 0
    /// play_sound("gun.ogg") → ID 2  // All three play at once
    /// ```
    ///
    /// # Performance
    ///
    /// - Cache hit: ~microseconds (lock-free message passing)
    /// - Cache miss: milliseconds to seconds (loads from disk)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file
    /// * `settings` - Playback settings (volume, looping, etc.)
    ///
    /// # Returns
    ///
    /// Returns a unique sound ID that can be used to control playback via
    /// `set_sound_volume()`, `pause_sound()`, `stop_sound()`, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if the sound cannot be loaded or played.
    pub fn play_sound(
        &mut self,
        path: impl AsRef<Path>,
        settings: PlaybackSettings,
    ) -> Result<u64> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Ensure sound is loaded (no-op if already cached)
        if !self.loaded_sounds.contains_key(&path_str) {
            self.load_sound(path.as_ref())?;
        }

        // Retrieve cached sound data
        let sound_data = self
            .loaded_sounds
            .get(&path_str)
            .ok_or_else(|| praxis_utils::eyre::eyre!("Sound not loaded: {}", path_str))?;

        // Configure Kira's playback settings
        let mut sound_settings = StaticSoundSettings::new();
        // Volume in amplitude (linear scale, 0.0 to 1.0)
        sound_settings = sound_settings.volume(Volume::Amplitude(settings.volume.into()));

        // Set up looping - ".." means loop entire sound
        if settings.looping {
            sound_settings = sound_settings.loop_region(..);
        }

        // Send play command to audio thread
        // Returns a handle for controlling the sound in real-time
        let handle = self
            .manager
            .play(sound_data.clone().with_settings(sound_settings))
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to play sound '{}': {}", path_str, e))?;

        // Generate unique ID and store handle in pool
        let sound_id = self.next_sound_id;
        self.next_sound_id += 1;
        self.playing_sounds.insert(sound_id, handle);

        Ok(sound_id)
    }

    /// Stops a playing sound.
    ///
    /// Sends a stop command to the audio thread and removes the handle from
    /// the playing sounds pool. The sound will fade out using the default tween.
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID returned from `play_sound`
    ///
    /// # Errors
    ///
    /// Returns an error if the sound cannot be stopped.
    pub fn stop_sound(&mut self, sound_id: u64) -> Result<()> {
        if let Some(mut handle) = self.playing_sounds.remove(&sound_id) {
            // Tween::default() provides a smooth fade-out to prevent clicking
            handle.stop(Tween::default());
        }
        Ok(())
    }

    /// Pauses a playing sound.
    ///
    /// The sound can be resumed later from the same position with `resume_sound()`.
    /// Pausing does not remove the handle from the pool.
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID returned from `play_sound`
    ///
    /// # Errors
    ///
    /// Returns an error if the sound cannot be paused.
    pub fn pause_sound(&mut self, sound_id: u64) -> Result<()> {
        if let Some(handle) = self.playing_sounds.get_mut(&sound_id) {
            handle.pause(Tween::default());
        }
        Ok(())
    }

    /// Resumes a paused sound.
    ///
    /// Continues playback from where the sound was paused.
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID returned from `play_sound`
    ///
    /// # Errors
    ///
    /// Returns an error if the sound cannot be resumed.
    pub fn resume_sound(&mut self, sound_id: u64) -> Result<()> {
        if let Some(handle) = self.playing_sounds.get_mut(&sound_id) {
            handle.resume(Tween::default());
        }
        Ok(())
    }

    /// Sets the volume of a playing sound.
    ///
    /// Volume changes are applied smoothly using the default tween to prevent
    /// audible clicking or popping artifacts.
    ///
    /// This is used by the spatial audio system to implement distance attenuation.
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID returned from `play_sound`
    /// * `volume` - Volume level from 0.0 to 1.0
    ///
    /// # Errors
    ///
    /// Returns an error if the volume cannot be set.
    pub fn set_sound_volume(&mut self, sound_id: u64, volume: f32) -> Result<()> {
        if let Some(handle) = self.playing_sounds.get_mut(&sound_id) {
            handle.set_volume(Volume::Amplitude(volume.into()), Tween::default());
        }
        Ok(())
    }

    /// Sets the playback rate of a playing sound.
    ///
    /// This is used for doppler effect simulation. A rate of 1.0 is normal speed,
    /// values > 1.0 are faster (higher pitch), values < 1.0 are slower (lower pitch).
    ///
    /// # Doppler Effect Implementation
    ///
    /// The doppler effect is achieved by dynamically adjusting playback rate:
    /// - Source approaching listener: rate > 1.0 (higher pitch)
    /// - Source receding from listener: rate < 1.0 (lower pitch)
    /// - Stationary source: rate = 1.0 (normal pitch)
    ///
    /// The rate is calculated using the classic doppler formula in `systems.rs`.
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID returned from `play_sound`
    /// * `rate` - Playback rate (0.5 to 2.0 recommended range)
    ///
    /// # Errors
    ///
    /// Returns an error if the playback rate cannot be set.
    pub fn set_sound_playback_rate(&mut self, sound_id: u64, rate: f32) -> Result<()> {
        if let Some(handle) = self.playing_sounds.get_mut(&sound_id) {
            handle.set_playback_rate(PlaybackRate::Factor(rate.into()), Tween::default());
        }
        Ok(())
    }

    /// Sets the panning of a playing sound.
    ///
    /// Panning controls stereo positioning for spatial audio.
    ///
    /// # Spatial Audio Panning
    ///
    /// Used by the spatial audio system to position sounds in stereo space:
    /// - -1.0: Hard left (sound is to the left of listener)
    /// - 0.0: Center (sound is directly in front/behind listener)
    /// - +1.0: Hard right (sound is to the right of listener)
    ///
    /// Note: This is a simple stereo panning model. For true 3D audio,
    /// consider using HRTF or ambisonic techniques (not currently implemented).
    ///
    /// # Arguments
    ///
    /// * `sound_id` - ID returned from `play_sound`
    /// * `panning` - Pan value (-1.0 left, 0.0 center, 1.0 right)
    ///
    /// # Errors
    ///
    /// Returns an error if the panning cannot be set.
    pub fn set_sound_panning(&mut self, sound_id: u64, panning: f32) -> Result<()> {
        if let Some(handle) = self.playing_sounds.get_mut(&sound_id) {
            handle.set_panning(f64::from(panning), Tween::default());
        }
        Ok(())
    }

    /// Removes finished sounds from the internal tracking.
    ///
    /// This should be called periodically to clean up memory.
    ///
    /// # Memory Management
    ///
    /// Sound handles are kept in the `playing_sounds` pool even after they
    /// finish playing. This method removes handles for sounds that have
    /// reached the Stopped state, freeing up memory.
    ///
    /// The `play_sound_system` calls this automatically each frame, but
    /// you can call it manually if needed for more aggressive cleanup.
    ///
    /// # Performance
    ///
    /// Uses `retain()` which is O(n) where n is the number of playing sounds.
    /// Typically very fast (microseconds) unless hundreds of sounds are active.
    pub fn cleanup_finished_sounds(&mut self) {
        self.playing_sounds
            .retain(|_, handle| handle.state() != kira::sound::PlaybackState::Stopped);
    }

    /// Returns the number of currently loaded sounds.
    #[must_use]
    pub fn loaded_sound_count(&self) -> usize {
        self.loaded_sounds.len()
    }

    /// Returns the number of currently playing sounds.
    #[must_use]
    pub fn playing_sound_count(&self) -> usize {
        self.playing_sounds.len()
    }
}

/// Settings for sound playback.
#[derive(Debug, Clone, Copy)]
pub struct PlaybackSettings {
    /// Volume level (0.0 to 1.0).
    pub volume: f32,

    /// Whether the sound should loop.
    pub looping: bool,

    /// Panning (-1.0 left, 0.0 center, 1.0 right).
    pub panning: f32,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            volume: 1.0,
            looping: false,
            panning: 0.0,
        }
    }
}

impl PlaybackSettings {
    /// Creates new playback settings with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the volume level.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // clamp is not const in stable Rust
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets whether the sound should loop.
    #[must_use]
    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Sets the panning.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // clamp is not const in stable Rust
    pub fn with_panning(mut self, panning: f32) -> Self {
        self.panning = panning.clamp(-1.0, 1.0);
        self
    }
}

/// Mock audio manager for headless testing.
///
/// This is a no-op implementation that allows tests to run without actual audio backend initialization.
/// All audio operations are no-ops, and all query methods return empty or default values.
///
/// # Example
///
/// ```rust
/// use praxis_audio::MockAudioManager;
///
/// let mut manager = MockAudioManager::new();
/// // All operations are no-ops, suitable for testing game logic without audio
/// ```
#[cfg(test)]
#[allow(clippy::must_use_candidate, clippy::missing_errors_doc)]
#[derive(Default)]
pub struct MockAudioManager {
    loaded_sound_count: usize,
    next_sound_id: u64,
}

#[cfg(test)]
#[allow(clippy::missing_errors_doc)]
impl MockAudioManager {
    /// Creates a new mock audio manager.
    ///
    /// All internal state is initialized to empty/default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            loaded_sound_count: 0,
            next_sound_id: 0,
        }
    }

    /// Mock sound loading that increments an internal counter.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the audio file (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn load_sound(&mut self, _path: impl AsRef<Path>) -> Result<()> {
        self.loaded_sound_count += 1;
        Ok(())
    }

    /// Mock sound playback that returns a unique sound ID.
    ///
    /// # Arguments
    ///
    /// * `_path` - Path to the audio file (ignored)
    /// * `_settings` - Playback settings (ignored)
    ///
    /// # Returns
    ///
    /// Returns a unique sound ID (incrementing counter).
    pub fn play_sound(
        &mut self,
        _path: impl AsRef<Path>,
        _settings: PlaybackSettings,
    ) -> Result<u64> {
        let sound_id = self.next_sound_id;
        self.next_sound_id += 1;
        Ok(sound_id)
    }

    /// Mock sound stopping (no-op).
    ///
    /// # Arguments
    ///
    /// * `_sound_id` - Sound ID (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn stop_sound(&mut self, _sound_id: u64) -> Result<()> {
        Ok(())
    }

    /// Mock sound pausing (no-op).
    ///
    /// # Arguments
    ///
    /// * `_sound_id` - Sound ID (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn pause_sound(&mut self, _sound_id: u64) -> Result<()> {
        Ok(())
    }

    /// Mock sound resuming (no-op).
    ///
    /// # Arguments
    ///
    /// * `_sound_id` - Sound ID (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn resume_sound(&mut self, _sound_id: u64) -> Result<()> {
        Ok(())
    }

    /// Mock volume setting (no-op).
    ///
    /// # Arguments
    ///
    /// * `_sound_id` - Sound ID (ignored)
    /// * `_volume` - Volume level (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn set_sound_volume(&mut self, _sound_id: u64, _volume: f32) -> Result<()> {
        Ok(())
    }

    /// Mock playback rate setting (no-op).
    ///
    /// # Arguments
    ///
    /// * `_sound_id` - Sound ID (ignored)
    /// * `_rate` - Playback rate (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn set_sound_playback_rate(&mut self, _sound_id: u64, _rate: f32) -> Result<()> {
        Ok(())
    }

    /// Mock panning setting (no-op).
    ///
    /// # Arguments
    ///
    /// * `_sound_id` - Sound ID (ignored)
    /// * `_panning` - Panning value (ignored)
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())`.
    pub fn set_sound_panning(&mut self, _sound_id: u64, _panning: f32) -> Result<()> {
        Ok(())
    }

    /// Mock cleanup (no-op).
    pub fn cleanup_finished_sounds(&mut self) {
        // No-op
    }

    /// Returns the number of loaded sounds (mock counter).
    pub fn loaded_sound_count(&self) -> usize {
        self.loaded_sound_count
    }

    /// Returns the number of playing sounds (always 0 in mock).
    pub fn playing_sound_count(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[test]
    fn test_mock_audio_manager_creation() {
        let manager = MockAudioManager::new();
        assert_eq!(manager.loaded_sound_count(), 0);
        assert_eq!(manager.playing_sound_count(), 0);
    }

    #[test]
    fn test_mock_audio_manager_load_sound() {
        let mut manager = MockAudioManager::new();

        manager.load_sound("test.ogg").unwrap();
        assert_eq!(manager.loaded_sound_count(), 1);

        manager.load_sound("test2.ogg").unwrap();
        assert_eq!(manager.loaded_sound_count(), 2);
    }

    #[test]
    fn test_mock_audio_manager_play_sound() {
        let mut manager = MockAudioManager::new();
        let settings = PlaybackSettings::default();

        let id1 = manager.play_sound("test.ogg", settings).unwrap();
        assert_eq!(id1, 0);

        let id2 = manager.play_sound("test2.ogg", settings).unwrap();
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_mock_audio_manager_control_operations() {
        let mut manager = MockAudioManager::new();
        let settings = PlaybackSettings::default();
        let sound_id = manager.play_sound("test.ogg", settings).unwrap();

        // All operations should succeed without errors
        manager.stop_sound(sound_id).unwrap();
        manager.pause_sound(sound_id).unwrap();
        manager.resume_sound(sound_id).unwrap();
        manager.set_sound_volume(sound_id, 0.5).unwrap();
        manager.set_sound_playback_rate(sound_id, 1.5).unwrap();
        manager.set_sound_panning(sound_id, 0.5).unwrap();
    }

    #[test]
    fn test_mock_audio_manager_cleanup() {
        let mut manager = MockAudioManager::new();
        // Should not panic
        manager.cleanup_finished_sounds();
    }

    #[test]
    fn test_mock_audio_manager_default() {
        let manager = MockAudioManager::default();
        assert_eq!(manager.loaded_sound_count(), 0);
    }
}
