//! Audio manager for loading and managing sounds.

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
    manager: KiraAudioManager,
    loaded_sounds: HashMap<String, StaticSoundData>,
    playing_sounds: HashMap<u64, StaticSoundHandle>,
    next_sound_id: u64,
}

impl AudioManager {
    /// Creates a new audio manager.
    ///
    /// # Errors
    ///
    /// Returns an error if the audio backend fails to initialize.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_audio::AudioManager;
    ///
    /// let manager = AudioManager::new().expect("Failed to initialize audio");
    /// ```
    pub fn new() -> Result<Self> {
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
    /// If the sound is already loaded, returns the cached version.
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

        if self.loaded_sounds.contains_key(&path_str) {
            return Ok(());
        }

        let sound_data = StaticSoundData::from_file(path)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to load sound '{}': {}", path_str, e))?;

        self.loaded_sounds.insert(path_str, sound_data);
        Ok(())
    }

    /// Plays a sound with the given settings.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file
    /// * `settings` - Playback settings (volume, looping, etc.)
    ///
    /// # Returns
    ///
    /// Returns a unique sound ID that can be used to control playback.
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

        if !self.loaded_sounds.contains_key(&path_str) {
            self.load_sound(path.as_ref())?;
        }

        let sound_data = self
            .loaded_sounds
            .get(&path_str)
            .ok_or_else(|| praxis_utils::eyre::eyre!("Sound not loaded: {}", path_str))?;

        let mut sound_settings = StaticSoundSettings::new();
        sound_settings = sound_settings.volume(Volume::Amplitude(settings.volume.into()));

        if settings.looping {
            sound_settings = sound_settings.loop_region(..);
        }

        let handle = self
            .manager
            .play(sound_data.clone().with_settings(sound_settings))
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to play sound '{}': {}", path_str, e))?;

        let sound_id = self.next_sound_id;
        self.next_sound_id += 1;
        self.playing_sounds.insert(sound_id, handle);

        Ok(sound_id)
    }

    /// Stops a playing sound.
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
            handle.stop(Tween::default());
        }
        Ok(())
    }

    /// Pauses a playing sound.
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
    pub const fn with_volume(mut self, volume: f32) -> Self {
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
    pub const fn with_panning(mut self, panning: f32) -> Self {
        self.panning = panning.clamp(-1.0, 1.0);
        self
    }
}
