//! Audio components for the ECS.

use praxis_ecs::Component;

/// Component that marks an entity as an audio source.
///
/// When attached to an entity with a `Transform`, the audio system will
/// play spatial audio positioned at the entity's location.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_audio::AudioSource;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Create a spatial audio source
/// world.spawn((
///     Transform::from_xyz(10.0, 0.0, 5.0),
///     AudioSource::new("assets/sounds/ambient.ogg")
///         .with_volume(0.5)
///         .with_spatial(true)
///         .with_looping(true),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct AudioSource {
    /// Path to the audio file to play.
    pub path: String,

    /// Volume level (0.0 to 1.0).
    pub volume: f32,

    /// Whether to enable spatial audio positioning.
    pub spatial: bool,

    /// Whether the audio should loop continuously.
    pub looping: bool,

    /// Playback state.
    pub state: AudioState,

    /// Maximum distance for audio attenuation (in world units).
    /// Beyond this distance, the sound is inaudible.
    pub max_distance: f32,

    /// Reference distance for audio attenuation (in world units).
    /// At this distance, the volume is at the specified level.
    pub reference_distance: f32,

    /// Whether to enable doppler effect for this source.
    pub doppler_enabled: bool,

    /// Doppler scale factor (0.0 to disable, 1.0 for normal, higher for exaggerated).
    pub doppler_scale: f32,

    /// Handle to the playing sound instance (internal use).
    pub(crate) sound_handle: Option<SoundHandle>,

    /// Previous position for velocity calculation (internal use).
    pub(crate) previous_position: Option<praxis_math::Vec3>,
}

impl AudioSource {
    /// Creates a new audio source with the given file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file (relative to the working directory)
    ///
    /// # Example
    ///
    /// ```rust
    /// use praxis_audio::AudioSource;
    ///
    /// let source = AudioSource::new("assets/sounds/explosion.ogg");
    /// ```
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            volume: 1.0,
            spatial: false,
            looping: false,
            state: AudioState::Stopped,
            max_distance: 100.0,
            reference_distance: 1.0,
            doppler_enabled: false,
            doppler_scale: 1.0,
            sound_handle: None,
            previous_position: None,
        }
    }

    /// Sets the volume level.
    ///
    /// # Arguments
    ///
    /// * `volume` - Volume level from 0.0 (silent) to 1.0 (full volume)
    #[must_use]
    pub const fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Enables or disables spatial audio.
    ///
    /// When spatial is true, the audio will be positioned in 3D space
    /// based on the entity's Transform component.
    #[must_use]
    pub const fn with_spatial(mut self, spatial: bool) -> Self {
        self.spatial = spatial;
        self
    }

    /// Sets whether the audio should loop.
    #[must_use]
    pub const fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Sets the maximum audible distance.
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance in world units beyond which the sound is inaudible
    #[must_use]
    pub const fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance;
        self
    }

    /// Sets the reference distance for attenuation.
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance in world units at which the volume is at the specified level
    #[must_use]
    pub const fn with_reference_distance(mut self, distance: f32) -> Self {
        self.reference_distance = distance;
        self
    }

    /// Enables or disables the doppler effect.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable doppler effect
    #[must_use]
    pub const fn with_doppler(mut self, enabled: bool) -> Self {
        self.doppler_enabled = enabled;
        self
    }

    /// Sets the doppler scale factor.
    ///
    /// # Arguments
    ///
    /// * `scale` - Doppler scale (0.0 to disable, 1.0 for normal, higher for exaggerated)
    #[must_use]
    pub const fn with_doppler_scale(mut self, scale: f32) -> Self {
        self.doppler_scale = scale;
        self
    }

    /// Requests the audio to start playing.
    pub const fn play(&mut self) {
        self.state = AudioState::Playing;
    }

    /// Requests the audio to pause.
    pub const fn pause(&mut self) {
        self.state = AudioState::Paused;
    }

    /// Requests the audio to stop.
    pub const fn stop(&mut self) {
        self.state = AudioState::Stopped;
    }

    /// Returns whether the audio is currently playing.
    #[must_use]
    pub const fn is_playing(&self) -> bool {
        matches!(self.state, AudioState::Playing)
    }

    /// Returns whether the audio is currently paused.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        matches!(self.state, AudioState::Paused)
    }

    /// Returns whether the audio is stopped.
    #[must_use]
    pub const fn is_stopped(&self) -> bool {
        matches!(self.state, AudioState::Stopped)
    }
}

/// Playback state of an audio source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioState {
    /// Audio is currently playing.
    Playing,

    /// Audio is paused.
    Paused,

    /// Audio is stopped.
    Stopped,
}

/// Internal handle to a playing sound instance.
///
/// This is used internally by the audio system to manage sound playback.
#[derive(Debug, Clone)]
pub struct SoundHandle {
    /// Unique identifier for the sound instance.
    pub id: u64,
}

/// Component that marks an entity as the audio listener.
///
/// Typically attached to the camera entity. The audio system uses the
/// listener's position to calculate spatial audio parameters.
///
/// Only one listener should be active at a time. If multiple listeners
/// exist, the system uses the first one found.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_audio::AudioListener;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Attach listener to the camera
/// world.spawn((
///     Transform::from_xyz(0.0, 1.8, 0.0),
///     AudioListener,
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AudioListener;
