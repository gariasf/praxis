//! Audio components for the ECS.
//!
//! # Component-Based Audio Architecture
//!
//! This module defines ECS components for attaching audio to entities.
//! The component-based approach provides several key benefits:
//!
//! ## Benefits of Component-Based Audio
//!
//! 1. **Automatic Position Tracking**
//!    - Audio sources inherit transform from entity hierarchy
//!    - No manual position synchronization required
//!    - Works seamlessly with parent-child relationships
//!
//! 2. **Lifecycle Management**
//!    - Audio cleaned up automatically when entity despawns
//!    - No orphaned sound handles or memory leaks
//!    - Natural coupling between game objects and their sounds
//!
//! 3. **Data-Driven Configuration**
//!    - Audio properties stored as data (serializable)
//!    - Can be tweaked in editor or config files
//!    - Supports hot-reloading and runtime modification
//!
//! 4. **ECS Query Performance**
//!    - Systems efficiently process only relevant entities
//!    - Change detection minimizes unnecessary work
//!    - Parallel iteration where possible
//!
//! ## Component Workflow
//!
//! ```text
//! Entity Creation
//!     ↓
//! Add AudioSource component (configuration)
//!     ↓
//! Add Transform component (position)
//!     ↓
//! Set state to Playing
//!     ↓
//! play_sound_system detects state change
//!     ↓
//! AudioManager creates sound handle
//!     ↓
//! Handle stored in component
//!     ↓
//! System updates spatial params each frame
//!     ↓
//! Entity despawns → Component dropped → Handle cleaned up
//! ```

use praxis_ecs::Component;

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

/// Component that marks an entity as an audio source.
///
/// When attached to an entity with a `Transform`, the audio system will
/// play spatial audio positioned at the entity's location.
///
/// # Component Fields
///
/// ## Configuration Fields (User-Facing)
/// - `path`: Audio file to play
/// - `volume`: Base volume level (0.0 to 1.0)
/// - `spatial`: Enable 3D positioning
/// - `looping`: Repeat playback continuously
/// - `state`: Playback control (Playing/Paused/Stopped)
/// - `max_distance`: Sound inaudible beyond this distance
/// - `reference_distance`: Distance for full volume
/// - `doppler_enabled`: Enable pitch shifting for velocity
/// - `doppler_scale`: Intensity of doppler effect
///
/// ## Internal Fields (System-Managed)
/// - `sound_handle`: Kira sound control handle
/// - `previous_position`: Cached position for velocity calculation
///
/// These internal fields are managed by the audio system and should not
/// be modified directly by user code.
///
/// # Usage Patterns
///
/// ## Pattern 1: One-Shot Sound
/// ```rust,no_run
/// use praxis_audio::AudioSource;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Explosion that plays once at a location
/// world.spawn((
///     Transform::from_xyz(10.0, 0.0, 5.0),
///     AudioSource::new("assets/sounds/explosion.ogg")
///         .with_volume(0.8)
///         .with_spatial(true),
/// ));
/// ```
///
/// ## Pattern 2: Looping Ambient Sound
/// ```rust,no_run
/// # use praxis_audio::AudioSource;
/// # use praxis_ecs::{World, Transform};
/// # let mut world = World::new();
/// // Fire crackling sound that loops
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     AudioSource::new("assets/sounds/fire.ogg")
///         .with_volume(0.5)
///         .with_spatial(true)
///         .with_looping(true)
///         .with_max_distance(50.0)
///         .with_reference_distance(2.0),
/// ));
/// ```
///
/// ## Pattern 3: Moving Sound with Doppler
/// ```rust,no_run
/// # use praxis_audio::AudioSource;
/// # use praxis_ecs::{World, Transform};
/// # let mut world = World::new();
/// // Car engine that changes pitch with movement
/// world.spawn((
///     Transform::from_xyz(0.0, 0.0, 0.0),
///     AudioSource::new("assets/sounds/engine.ogg")
///         .with_volume(0.7)
///         .with_spatial(true)
///         .with_looping(true)
///         .with_doppler(true)
///         .with_doppler_scale(1.0),
/// ));
/// ```
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
    /// Relative to the working directory or assets folder.
    /// Cached by `AudioManager` for efficient playback.
    pub path: String,

    /// Volume level (0.0 to 1.0).
    /// This is the base volume before spatial attenuation is applied.
    /// For spatial sounds: `final_volume` = volume × attenuation
    pub volume: f32,

    /// Whether to enable spatial audio positioning.
    /// - true: Volume and panning based on position relative to listener
    /// - false: Sound plays at full volume with center panning
    pub spatial: bool,

    /// Whether the audio should loop continuously.
    /// - true: Sound restarts from beginning when it ends
    /// - false: Sound plays once and stops
    pub looping: bool,

    /// Playback state.
    /// Controls whether the sound is playing, paused, or stopped.
    /// Modified by user code to control playback.
    pub state: AudioState,

    /// Maximum distance for audio attenuation (in world units).
    /// Beyond this distance, the sound is completely inaudible (volume = 0).
    /// Typical values: 50-200 units depending on sound type.
    pub max_distance: f32,

    /// Reference distance for audio attenuation (in world units).
    /// At distances ≤ `reference_distance`, the volume is at maximum.
    /// Between reference and max distance, inverse square law applies.
    /// Typical values: 1-10 units depending on sound intensity.
    pub reference_distance: f32,

    /// Whether to enable doppler effect for this source.
    /// Doppler causes pitch to shift based on velocity relative to listener.
    /// Best used for fast-moving objects (vehicles, projectiles).
    pub doppler_enabled: bool,

    /// Doppler scale factor (0.0 to disable, 1.0 for normal, higher for exaggerated).
    /// Scales the intensity of the doppler effect.
    /// - 0.0: No doppler (same as `doppler_enabled` = false)
    /// - 1.0: Realistic doppler
    /// - 2.0: Exaggerated doppler (arcade style)
    pub doppler_scale: f32,

    /// Handle to the playing sound instance (internal use).
    /// Managed by the audio system - do not modify directly.
    /// Contains the unique ID used to control the sound in `AudioManager`.
    pub(crate) sound_handle: Option<SoundHandle>,

    /// Previous position for velocity calculation (internal use).
    /// Managed by the audio system - do not modify directly.
    /// Used to calculate velocity for doppler effect.
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
    #[allow(clippy::missing_const_for_fn)] // clamp is not const in stable Rust
    pub fn with_volume(mut self, volume: f32) -> Self {
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

    /// Gets the audio file path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Gets the volume level.
    #[must_use]
    pub const fn volume(&self) -> f32 {
        self.volume
    }

    /// Returns whether spatial audio is enabled.
    #[must_use]
    pub const fn is_spatial(&self) -> bool {
        self.spatial
    }

    /// Returns whether the audio is looping.
    #[must_use]
    pub const fn is_looping(&self) -> bool {
        self.looping
    }

    /// Gets the maximum audible distance.
    #[must_use]
    pub const fn max_distance(&self) -> f32 {
        self.max_distance
    }

    /// Gets the reference distance for attenuation.
    #[must_use]
    pub const fn reference_distance(&self) -> f32 {
        self.reference_distance
    }
}

/// Playback state of an audio source.
///
/// This enum controls the lifecycle of sound playback:
/// - **Playing**: Sound is actively playing (or will start next frame)
/// - **Paused**: Sound is paused and can be resumed
/// - **Stopped**: Sound is not playing and handle is cleaned up
///
/// State transitions are handled by the audio system based on this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum AudioState {
    /// Audio is currently playing.
    /// The audio system will start playback if no handle exists,
    /// or continue updating spatial parameters if already playing.
    Playing,

    /// Audio is paused.
    /// The sound handle is preserved and can be resumed from the current position.
    Paused,

    /// Audio is stopped.
    /// The sound handle is cleaned up. Starting again will play from the beginning.
    Stopped,
}

/// Internal handle to a playing sound instance.
///
/// This is used internally by the audio system to manage sound playback.
/// Contains the unique ID that maps to a `StaticSoundHandle` in `AudioManager`.
///
/// # Lifecycle
///
/// 1. Created when `AudioManager.play_sound()` succeeds
/// 2. Stored in `AudioSource.sound_handle`
/// 3. Used to control sound (volume, pause, stop, etc.)
/// 4. Dropped when sound stops or entity despawns
///
/// # Note
///
/// User code should not create or modify `SoundHandle` directly.
/// It is managed entirely by the audio system.
#[derive(Debug, Clone)]
pub struct SoundHandle {
    /// Unique identifier for the sound instance.
    /// Maps to a `StaticSoundHandle` in `AudioManager.playing_sounds`.
    pub id: u64,
}

/// Component that marks an entity as the audio listener.
///
/// Typically attached to the camera entity. The audio system uses the
/// listener's position to calculate spatial audio parameters.
///
/// # Single Listener Model
///
/// Only one listener should be active at a time. If multiple listeners
/// exist, the system uses the first one found. This prevents ambiguity
/// in spatial audio calculations.
///
/// # 3D Listener Management
///
/// The listener's Transform is queried each frame to:
/// 1. Calculate distance to each audio source
/// 2. Compute attenuation based on distance
/// 3. Calculate stereo panning based on relative position
/// 4. Apply doppler effect based on relative velocity
///
/// # Listener Positioning
///
/// The listener should be positioned where the player's "ears" are:
/// - First-person: Attach to camera (player's head position)
/// - Third-person: Attach to camera (slightly behind character)
/// - Top-down: Attach to camera (bird's eye view position)
///
/// # Performance
///
/// Listener queries are very fast (single entity lookup).
/// The system caches the listener transform for the frame to avoid
/// repeated queries when processing multiple audio sources.
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

/// Serialization support for audio components.
///
/// When the "serialization" feature is enabled, audio components can be
/// serialized to/from RON format for scene saving and loading.
///
/// # Serialization Strategy
///
/// Only user-facing configuration fields are serialized:
/// - path, volume, spatial, looping, state
/// - `max_distance`, `reference_distance`
/// - `doppler_enabled`, `doppler_scale`
///
/// Internal runtime fields are NOT serialized:
/// - `sound_handle`: Would be invalid after deserialization
/// - `previous_position`: Recalculated on first frame
///
/// This ensures serialized data is portable and deterministic.
#[cfg(feature = "serialization")]
mod serialization {
    use super::{AudioSource, AudioState};
    use bevy_ecs::entity::Entity;
    use praxis_ecs::{DeserializeContext, SerializableComponent};
    use praxis_utils::Result;
    use serde::{Deserialize, Serialize};

    type InsertComponentFn = Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>;

    /// Serializable representation of `AudioSource` (excludes internal runtime fields).
    #[derive(Serialize, Deserialize)]
    struct SerializableAudioSource {
        path: String,
        volume: f32,
        spatial: bool,
        looping: bool,
        state: AudioState,
        max_distance: f32,
        reference_distance: f32,
        doppler_enabled: bool,
        doppler_scale: f32,
    }

    impl From<&AudioSource> for SerializableAudioSource {
        fn from(source: &AudioSource) -> Self {
            Self {
                path: source.path.clone(),
                volume: source.volume,
                spatial: source.spatial,
                looping: source.looping,
                state: source.state,
                max_distance: source.max_distance,
                reference_distance: source.reference_distance,
                doppler_enabled: source.doppler_enabled,
                doppler_scale: source.doppler_scale,
            }
        }
    }

    impl From<SerializableAudioSource> for AudioSource {
        fn from(serializable: SerializableAudioSource) -> Self {
            Self {
                path: serializable.path,
                volume: serializable.volume,
                spatial: serializable.spatial,
                looping: serializable.looping,
                state: serializable.state,
                max_distance: serializable.max_distance,
                reference_distance: serializable.reference_distance,
                doppler_enabled: serializable.doppler_enabled,
                doppler_scale: serializable.doppler_scale,
                sound_handle: None,
                previous_position: None,
            }
        }
    }

    impl SerializableComponent for AudioSource {
        fn serialize_component(&self) -> Result<String> {
            let serializable = SerializableAudioSource::from(self);
            Ok(ron::to_string(&serializable)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<InsertComponentFn>
        where
            Self: Sized + 'static,
        {
            let serializable: SerializableAudioSource = ron::from_str(data)?;
            let component = Self::from(serializable);
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "AudioSource"
        }
    }
}

#[cfg(not(feature = "serialization"))]
mod serialization {
    // Empty module when serialization feature is disabled
}
