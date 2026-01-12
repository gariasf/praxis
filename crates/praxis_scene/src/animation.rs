//! Skeletal animation system for the Praxis engine.
//!
//! This module provides components and systems for skeletal animation, including:
//! - Skeleton: Defines bone hierarchy and bind poses
//! - `AnimationClip`: Stores keyframe data for animation sequences
//! - `AnimationPlayer`: Controls animation playback on entities
//! - `AnimationBlender`: Advanced blending with cross-fades, blend trees, and layers
//!
//! # Overview
//!
//! Skeletal animation works by defining a hierarchy of bones (joints) and animating
//! their transforms over time using keyframe interpolation. Each bone has a bind pose
//! (rest position) and can be animated independently. The bone transforms are then
//! applied to skinned meshes to deform vertices.
//!
//! # Animation Blending
//!
//! The `AnimationBlender` component provides advanced blending capabilities:
//! - **Cross-fade transitions**: Smooth transitions between animations over time
//! - **Blend trees**: 1D/2D blend spaces for parameter-driven blending
//! - **Layered animation**: Multiple layers with bone masking and weights
//! - **Additive blending**: Add animations on top of base animations

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::float_cmp)]
#![allow(clippy::unused_self)]
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use praxis_scene::{Skeleton, AnimationClip, AnimationPlayer, AnimatedPose, Bone, update_animations};
//! use praxis_ecs::{World, Query};
//! use praxis_math::{Vec3, Quat};
//!
//! // Define skeleton
//! let skeleton = Skeleton::new(vec![
//!     Bone {
//!         name: "Root".to_string(),
//!         parent_index: None,
//!         bind_pose_translation: Vec3::ZERO,
//!         bind_pose_rotation: Quat::IDENTITY,
//!         bind_pose_scale: Vec3::ONE,
//!     },
//!     Bone {
//!         name: "Spine".to_string(),
//!         parent_index: Some(0),
//!         bind_pose_translation: Vec3::new(0.0, 1.0, 0.0),
//!         bind_pose_rotation: Quat::IDENTITY,
//!         bind_pose_scale: Vec3::ONE,
//!     },
//! ]);
//!
//! // Create animation clip
//! let mut clip = AnimationClip::new("Walk".to_string(), 2.0);
//! clip.add_bone_track(0);
//! clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
//! clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));
//!
//! // Create animation player
//! let mut player = AnimationPlayer::new();
//! player.add_clip("Walk".to_string(), clip);
//!
//! // Spawn entity
//! let mut world = World::new();
//! let pose = AnimatedPose::new(skeleton.bone_count());
//! world.spawn((skeleton, player, pose));
//!
//! // In your game loop, update animations
//! fn update_system(mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>) {
//!     let delta_time = 0.016; // Get from timing system
//!     update_animations(delta_time, &mut query);
//! }
//! ```

use bevy_ecs::component::Component;
use bevy_ecs::system::Query;
use praxis_math::{Mat4, Quat, Vec3};
use std::collections::HashMap;

// ============================================================================
// Core Animation Components
// ============================================================================

/// Represents a single bone in a skeleton.
///
/// Each bone has a name, optional parent bone, and a bind pose (rest position).
/// The bind pose defines the bone's default transformation relative to its parent.
#[derive(Debug, Clone)]
pub struct Bone {
    /// Name of the bone for identification.
    pub name: String,

    /// Index of the parent bone in the skeleton, or None for root bones.
    pub parent_index: Option<usize>,

    /// Bind pose translation relative to parent.
    pub bind_pose_translation: Vec3,

    /// Bind pose rotation relative to parent.
    pub bind_pose_rotation: Quat,

    /// Bind pose scale relative to parent.
    pub bind_pose_scale: Vec3,
}

impl Bone {
    /// Creates a new bone with the given name and parent index.
    pub fn new(name: String, parent_index: Option<usize>) -> Self {
        Self {
            name,
            parent_index,
            bind_pose_translation: Vec3::ZERO,
            bind_pose_rotation: Quat::IDENTITY,
            bind_pose_scale: Vec3::ONE,
        }
    }

    /// Creates a bone with specified bind pose.
    pub fn with_bind_pose(
        name: String,
        parent_index: Option<usize>,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) -> Self {
        Self {
            name,
            parent_index,
            bind_pose_translation: translation,
            bind_pose_rotation: rotation,
            bind_pose_scale: scale,
        }
    }

    /// Computes the local bind pose matrix.
    pub fn bind_pose_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.bind_pose_scale,
            self.bind_pose_rotation,
            self.bind_pose_translation,
        )
    }
}

/// Skeleton component defining the bone hierarchy and bind poses.
///
/// A skeleton is a hierarchical structure of bones used for skeletal animation.
/// It stores the bind pose (rest position) for each bone and the parent-child
/// relationships between bones.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{Skeleton, Bone};
/// use praxis_math::{Vec3, Quat};
///
/// let skeleton = Skeleton::new(vec![
///     Bone::with_bind_pose(
///         "Root".to_string(),
///         None,
///         Vec3::ZERO,
///         Quat::IDENTITY,
///         Vec3::ONE,
///     ),
///     Bone::with_bind_pose(
///         "Spine".to_string(),
///         Some(0),
///         Vec3::new(0.0, 1.0, 0.0),
///         Quat::IDENTITY,
///         Vec3::ONE,
///     ),
/// ]);
/// ```
#[derive(Component, Debug, Clone)]
pub struct Skeleton {
    /// List of bones in the skeleton.
    bones: Vec<Bone>,

    /// Mapping from bone names to their indices.
    bone_name_to_index: HashMap<String, usize>,

    /// Inverse bind pose matrices for skinning (world to bone space).
    inverse_bind_matrices: Vec<Mat4>,
}

impl Skeleton {
    /// Creates a new skeleton from a list of bones.
    ///
    /// The inverse bind matrices are computed automatically from the bone hierarchy.
    pub fn new(bones: Vec<Bone>) -> Self {
        let mut bone_name_to_index = HashMap::new();
        for (i, bone) in bones.iter().enumerate() {
            bone_name_to_index.insert(bone.name.clone(), i);
        }

        let inverse_bind_matrices = Self::compute_inverse_bind_matrices(&bones);

        Self {
            bones,
            bone_name_to_index,
            inverse_bind_matrices,
        }
    }

    /// Returns the number of bones in the skeleton.
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Gets a bone by index.
    pub fn bone(&self, index: usize) -> Option<&Bone> {
        self.bones.get(index)
    }

    /// Gets a mutable reference to a bone by index.
    pub fn bone_mut(&mut self, index: usize) -> Option<&mut Bone> {
        self.bones.get_mut(index)
    }

    /// Gets all bones.
    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }

    /// Finds a bone index by name.
    pub fn find_bone(&self, name: &str) -> Option<usize> {
        self.bone_name_to_index.get(name).copied()
    }

    /// Gets the inverse bind matrix for a bone.
    pub fn inverse_bind_matrix(&self, index: usize) -> Option<Mat4> {
        self.inverse_bind_matrices.get(index).copied()
    }

    /// Gets all inverse bind matrices.
    pub fn inverse_bind_matrices(&self) -> &[Mat4] {
        &self.inverse_bind_matrices
    }

    /// Computes inverse bind matrices from the bone hierarchy.
    ///
    /// For each bone, this computes the transform from world space to bone local space.
    fn compute_inverse_bind_matrices(bones: &[Bone]) -> Vec<Mat4> {
        let mut world_transforms = vec![Mat4::IDENTITY; bones.len()];

        // Compute world space bind pose for each bone
        for i in 0..bones.len() {
            let bone = &bones[i];
            let local_transform = bone.bind_pose_matrix();

            world_transforms[i] = bone.parent_index.map_or(local_transform, |parent_idx| {
                world_transforms[parent_idx] * local_transform
            });
        }

        // Invert to get bone space from world space
        world_transforms
            .iter()
            .map(praxis_math::Mat4::inverse)
            .collect()
    }

    /// Recomputes inverse bind matrices after bone modifications.
    pub fn recompute_inverse_bind_matrices(&mut self) {
        self.inverse_bind_matrices = Self::compute_inverse_bind_matrices(&self.bones);
    }
}

/// Keyframe data for animation tracks.
///
/// A keyframe stores the value of an animated property at a specific time.
/// Multiple keyframes define an animation curve that can be interpolated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe<T> {
    /// Time in seconds when this keyframe occurs.
    pub time: f32,

    /// Value at this keyframe.
    pub value: T,
}

impl<T> Keyframe<T> {
    /// Creates a new keyframe.
    pub fn new(time: f32, value: T) -> Self {
        Self { time, value }
    }
}

/// Animation track for a single bone.
///
/// Contains keyframe data for translation, rotation, and scale channels.
/// Each channel is optional and can be animated independently.
#[derive(Debug, Clone, Default)]
pub struct BoneTrack {
    /// Translation keyframes (position over time).
    pub translation_keyframes: Vec<Keyframe<Vec3>>,

    /// Rotation keyframes (orientation over time).
    pub rotation_keyframes: Vec<Keyframe<Quat>>,

    /// Scale keyframes (size over time).
    pub scale_keyframes: Vec<Keyframe<Vec3>>,
}

impl BoneTrack {
    /// Creates a new empty bone track.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a translation keyframe.
    pub fn add_translation_keyframe(&mut self, time: f32, translation: Vec3) {
        self.translation_keyframes
            .push(Keyframe::new(time, translation));
        // Keep keyframes sorted by time
        self.translation_keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Adds a rotation keyframe.
    pub fn add_rotation_keyframe(&mut self, time: f32, rotation: Quat) {
        self.rotation_keyframes.push(Keyframe::new(time, rotation));
        self.rotation_keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Adds a scale keyframe.
    pub fn add_scale_keyframe(&mut self, time: f32, scale: Vec3) {
        self.scale_keyframes.push(Keyframe::new(time, scale));
        self.scale_keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Samples translation at a given time using linear interpolation.
    pub fn sample_translation(&self, time: f32) -> Option<Vec3> {
        if self.translation_keyframes.is_empty() {
            return None;
        }

        // Find the keyframes to interpolate between
        let mut before = None;
        let mut after = None;

        for keyframe in &self.translation_keyframes {
            if keyframe.time <= time {
                before = Some(keyframe);
            }
            if keyframe.time >= time && after.is_none() {
                after = Some(keyframe);
            }
        }

        match (before, after) {
            (Some(b), Some(a)) if b.time != a.time => {
                // Interpolate between keyframes
                let t = (time - b.time) / (a.time - b.time);
                Some(b.value.lerp(a.value, t))
            }
            (Some(k), _) | (_, Some(k)) => {
                // At or beyond a keyframe
                Some(k.value)
            }
            _ => None,
        }
    }

    /// Samples rotation at a given time using spherical linear interpolation.
    pub fn sample_rotation(&self, time: f32) -> Option<Quat> {
        if self.rotation_keyframes.is_empty() {
            return None;
        }

        let mut before = None;
        let mut after = None;

        for keyframe in &self.rotation_keyframes {
            if keyframe.time <= time {
                before = Some(keyframe);
            }
            if keyframe.time >= time && after.is_none() {
                after = Some(keyframe);
            }
        }

        match (before, after) {
            (Some(b), Some(a)) if b.time != a.time => {
                let t = (time - b.time) / (a.time - b.time);
                Some(b.value.slerp(a.value, t))
            }
            (Some(k), _) | (_, Some(k)) => Some(k.value),
            _ => None,
        }
    }

    /// Samples scale at a given time using linear interpolation.
    pub fn sample_scale(&self, time: f32) -> Option<Vec3> {
        if self.scale_keyframes.is_empty() {
            return None;
        }

        let mut before = None;
        let mut after = None;

        for keyframe in &self.scale_keyframes {
            if keyframe.time <= time {
                before = Some(keyframe);
            }
            if keyframe.time >= time && after.is_none() {
                after = Some(keyframe);
            }
        }

        match (before, after) {
            (Some(b), Some(a)) if b.time != a.time => {
                let t = (time - b.time) / (a.time - b.time);
                Some(b.value.lerp(a.value, t))
            }
            (Some(k), _) | (_, Some(k)) => Some(k.value),
            _ => None,
        }
    }

    /// Returns true if this track has any keyframes.
    pub fn has_keyframes(&self) -> bool {
        !self.translation_keyframes.is_empty()
            || !self.rotation_keyframes.is_empty()
            || !self.scale_keyframes.is_empty()
    }
}

/// Animation clip storing keyframe animation data.
///
/// An animation clip contains animation tracks for multiple bones, defining
/// how the skeleton should be posed over time. Clips can be played, looped,
/// and blended together.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::AnimationClip;
/// use praxis_math::{Vec3, Quat};
///
/// let mut clip = AnimationClip::new("Walk".to_string(), 2.0);
///
/// // Add a track for bone 0 (root)
/// clip.add_bone_track(0);
/// clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
/// clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));
/// clip.add_translation_keyframe(0, 2.0, Vec3::new(2.0, 0.0, 0.0));
/// ```
#[derive(Debug, Clone)]
pub struct AnimationClip {
    /// Name of the animation clip.
    name: String,

    /// Duration of the animation in seconds.
    duration: f32,

    /// Animation tracks indexed by bone index.
    bone_tracks: HashMap<usize, BoneTrack>,
}

impl AnimationClip {
    /// Creates a new animation clip with the given name and duration.
    pub fn new(name: String, duration: f32) -> Self {
        Self {
            name,
            duration,
            bone_tracks: HashMap::new(),
        }
    }

    /// Gets the clip name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the clip duration in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Sets the clip duration.
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }

    /// Adds a new bone track for the given bone index.
    pub fn add_bone_track(&mut self, bone_index: usize) -> &mut BoneTrack {
        self.bone_tracks.entry(bone_index).or_default()
    }

    /// Gets a bone track by bone index.
    pub fn bone_track(&self, bone_index: usize) -> Option<&BoneTrack> {
        self.bone_tracks.get(&bone_index)
    }

    /// Gets a mutable bone track by bone index.
    pub fn bone_track_mut(&mut self, bone_index: usize) -> Option<&mut BoneTrack> {
        self.bone_tracks.get_mut(&bone_index)
    }

    /// Gets all bone tracks.
    pub fn bone_tracks(&self) -> &HashMap<usize, BoneTrack> {
        &self.bone_tracks
    }

    /// Adds a translation keyframe to a bone track.
    pub fn add_translation_keyframe(&mut self, bone_index: usize, time: f32, translation: Vec3) {
        self.add_bone_track(bone_index)
            .add_translation_keyframe(time, translation);
    }

    /// Adds a rotation keyframe to a bone track.
    pub fn add_rotation_keyframe(&mut self, bone_index: usize, time: f32, rotation: Quat) {
        self.add_bone_track(bone_index)
            .add_rotation_keyframe(time, rotation);
    }

    /// Adds a scale keyframe to a bone track.
    pub fn add_scale_keyframe(&mut self, bone_index: usize, time: f32, scale: Vec3) {
        self.add_bone_track(bone_index)
            .add_scale_keyframe(time, scale);
    }

    /// Returns the number of bone tracks in this clip.
    pub fn track_count(&self) -> usize {
        self.bone_tracks.len()
    }
}

/// Computed bone poses for the current animation state.
///
/// This component stores the final bone transformations after evaluating
/// all animations. It's used by the skinning system to deform meshes.
#[derive(Component, Debug, Clone)]
pub struct AnimatedPose {
    /// Local space bone transforms (relative to parent).
    local_transforms: Vec<Mat4>,

    /// World space bone transforms (accumulated from root).
    world_transforms: Vec<Mat4>,

    /// Final bone matrices for skinning (world * inverse_bind).
    skinning_matrices: Vec<Mat4>,
}

impl AnimatedPose {
    /// Creates a new animated pose with the given number of bones.
    pub fn new(bone_count: usize) -> Self {
        Self {
            local_transforms: vec![Mat4::IDENTITY; bone_count],
            world_transforms: vec![Mat4::IDENTITY; bone_count],
            skinning_matrices: vec![Mat4::IDENTITY; bone_count],
        }
    }

    /// Gets the local transform for a bone.
    pub fn local_transform(&self, bone_index: usize) -> Option<Mat4> {
        self.local_transforms.get(bone_index).copied()
    }

    /// Sets the local transform for a bone.
    pub fn set_local_transform(&mut self, bone_index: usize, transform: Mat4) {
        if let Some(t) = self.local_transforms.get_mut(bone_index) {
            *t = transform;
        }
    }

    /// Gets the world transform for a bone.
    pub fn world_transform(&self, bone_index: usize) -> Option<Mat4> {
        self.world_transforms.get(bone_index).copied()
    }

    /// Gets a slice of all local transforms.
    pub fn local_transforms(&self) -> &[Mat4] {
        &self.local_transforms
    }

    /// Gets a slice of all world transforms.
    pub fn world_transforms(&self) -> &[Mat4] {
        &self.world_transforms
    }

    /// Gets a slice of all skinning matrices.
    pub fn skinning_matrices(&self) -> &[Mat4] {
        &self.skinning_matrices
    }

    /// Updates world transforms from local transforms using the skeleton hierarchy.
    pub fn update_world_transforms(&mut self, skeleton: &Skeleton) {
        for i in 0..self.local_transforms.len() {
            if let Some(bone) = skeleton.bone(i) {
                self.world_transforms[i] = if let Some(parent_idx) = bone.parent_index {
                    self.world_transforms[parent_idx] * self.local_transforms[i]
                } else {
                    self.local_transforms[i]
                };
            }
        }
    }

    /// Updates skinning matrices from world transforms and inverse bind matrices.
    pub fn update_skinning_matrices(&mut self, skeleton: &Skeleton) {
        for i in 0..self.world_transforms.len() {
            if let Some(inverse_bind) = skeleton.inverse_bind_matrix(i) {
                self.skinning_matrices[i] = self.world_transforms[i] * inverse_bind;
            }
        }
    }
}

// ============================================================================
// Animation Player Component
// ============================================================================

/// Playback state for an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Animation is playing.
    Playing,

    /// Animation is paused at current time.
    Paused,

    /// Animation has stopped and reset to beginning.
    Stopped,
}

/// Current state of a playing animation clip.
#[derive(Debug, Clone)]
struct PlayingClip {
    /// Current playback time in seconds.
    time: f32,

    /// Playback speed multiplier (1.0 = normal speed).
    speed: f32,

    /// Whether the animation should loop.
    looping: bool,

    /// Current playback state.
    state: PlaybackState,

    /// Weight for animation blending (0.0 to 1.0).
    weight: f32,
}

impl PlayingClip {
    fn new() -> Self {
        Self {
            time: 0.0,
            speed: 1.0,
            looping: true,
            state: PlaybackState::Playing,
            weight: 1.0,
        }
    }
}

/// Animation player component for controlling animation playback.
///
/// The AnimationPlayer controls which animation clips are playing on an entity,
/// manages playback state (playing/paused/stopped), and handles looping and blending.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{AnimationPlayer, AnimationClip};
/// use praxis_math::Vec3;
///
/// let mut clip = AnimationClip::new("Walk".to_string(), 2.0);
/// clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
/// clip.add_translation_keyframe(0, 2.0, Vec3::new(2.0, 0.0, 0.0));
///
/// let mut player = AnimationPlayer::new();
/// player.add_clip("Walk".to_string(), clip);
/// player.play("Walk");
/// ```
#[derive(Component, Debug, Clone)]
pub struct AnimationPlayer {
    /// Available animation clips indexed by name.
    clips: HashMap<String, AnimationClip>,

    /// Currently playing clips with their playback state.
    playing_clips: HashMap<String, PlayingClip>,
}

impl AnimationPlayer {
    /// Creates a new empty animation player.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            playing_clips: HashMap::new(),
        }
    }

    /// Adds an animation clip to the player's library.
    pub fn add_clip(&mut self, name: String, clip: AnimationClip) {
        self.clips.insert(name, clip);
    }

    /// Builder method to add a clip.
    ///
    /// # Must Use
    ///
    /// This method consumes `self` and returns a new instance with the clip added.
    /// Ignoring the return value will discard the clip and may result in missing animations.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_clip(mut self, name: String, clip: AnimationClip) -> Self {
        self.add_clip(name, clip);
        self
    }

    /// Gets a clip by name.
    pub fn clip(&self, name: &str) -> Option<&AnimationClip> {
        self.clips.get(name)
    }

    /// Gets a mutable clip by name.
    pub fn clip_mut(&mut self, name: &str) -> Option<&mut AnimationClip> {
        self.clips.get_mut(name)
    }

    /// Gets all available clips.
    pub fn clips(&self) -> &HashMap<String, AnimationClip> {
        &self.clips
    }

    /// Starts playing an animation clip.
    pub fn play(&mut self, name: &str) {
        if self.clips.contains_key(name) {
            let mut playing = PlayingClip::new();
            playing.state = PlaybackState::Playing;
            self.playing_clips.insert(name.to_string(), playing);
        }
    }

    /// Pauses a playing animation.
    pub fn pause(&mut self, name: &str) {
        if let Some(playing) = self.playing_clips.get_mut(name) {
            playing.state = PlaybackState::Paused;
        }
    }

    /// Resumes a paused animation.
    pub fn resume(&mut self, name: &str) {
        if let Some(playing) = self.playing_clips.get_mut(name) {
            if playing.state == PlaybackState::Paused {
                playing.state = PlaybackState::Playing;
            }
        }
    }

    /// Stops an animation and resets it to the beginning.
    pub fn stop(&mut self, name: &str) {
        self.playing_clips.remove(name);
    }

    /// Sets whether an animation should loop.
    pub fn set_looping(&mut self, name: &str, looping: bool) {
        if let Some(playing) = self.playing_clips.get_mut(name) {
            playing.looping = looping;
        }
    }

    /// Sets the playback speed multiplier for an animation.
    pub fn set_speed(&mut self, name: &str, speed: f32) {
        if let Some(playing) = self.playing_clips.get_mut(name) {
            playing.speed = speed;
        }
    }

    /// Sets the blend weight for an animation (0.0 to 1.0).
    pub fn set_weight(&mut self, name: &str, weight: f32) {
        if let Some(playing) = self.playing_clips.get_mut(name) {
            playing.weight = weight.clamp(0.0, 1.0);
        }
    }

    /// Gets the current playback time for an animation.
    pub fn current_time(&self, name: &str) -> Option<f32> {
        self.playing_clips.get(name).map(|p| p.time)
    }

    /// Sets the current playback time for an animation.
    pub fn set_time(&mut self, name: &str, time: f32) {
        if let Some(playing) = self.playing_clips.get_mut(name) {
            playing.time = time;
        }
    }

    /// Returns true if the animation is currently playing.
    pub fn is_playing(&self, name: &str) -> bool {
        self.playing_clips
            .get(name)
            .is_some_and(|p| p.state == PlaybackState::Playing)
    }

    /// Gets all currently playing clip names.
    pub fn playing_clips(&self) -> Vec<&str> {
        self.playing_clips
            .iter()
            .filter(|(_, p)| p.state == PlaybackState::Playing)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Updates animation playback times based on delta time.
    pub fn update(&mut self, delta_time: f32) {
        let mut clips_to_remove = Vec::new();

        for (name, playing) in &mut self.playing_clips {
            if playing.state != PlaybackState::Playing {
                continue;
            }

            // Update playback time
            playing.time += delta_time * playing.speed;

            // Handle looping and clip end
            if let Some(clip) = self.clips.get(name) {
                if playing.time >= clip.duration() {
                    if playing.looping {
                        playing.time %= clip.duration();
                    } else {
                        playing.time = clip.duration();
                        playing.state = PlaybackState::Stopped;
                        clips_to_remove.push(name.clone());
                    }
                }
            }
        }

        // Remove stopped non-looping animations
        for name in clips_to_remove {
            self.playing_clips.remove(&name);
        }
    }

    /// Evaluates all playing animations and produces an animated pose.
    ///
    /// This samples all playing animations at their current times and blends
    /// them together according to their weights.
    pub fn evaluate(&self, skeleton: &Skeleton) -> AnimatedPose {
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        // Initialize with bind pose
        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        // Blend all playing animations
        for (clip_name, playing) in &self.playing_clips {
            if let Some(clip) = self.clips.get(clip_name) {
                Self::apply_clip_to_pose(clip, playing.time, playing.weight, &mut pose, skeleton);
            }
        }

        // Update world transforms and skinning matrices
        pose.update_world_transforms(skeleton);
        pose.update_skinning_matrices(skeleton);

        pose
    }

    /// Applies a single animation clip to a pose with blending.
    fn apply_clip_to_pose(
        clip: &AnimationClip,
        time: f32,
        weight: f32,
        pose: &mut AnimatedPose,
        skeleton: &Skeleton,
    ) {
        for (bone_index, track) in clip.bone_tracks() {
            if let Some(bone) = skeleton.bone(*bone_index) {
                // Sample the track at the current time
                let translation = track
                    .sample_translation(time)
                    .unwrap_or(bone.bind_pose_translation);
                let rotation = track
                    .sample_rotation(time)
                    .unwrap_or(bone.bind_pose_rotation);
                let scale = track.sample_scale(time).unwrap_or(bone.bind_pose_scale);

                // If weight is 1.0, just set the transform directly
                if weight >= 0.999 {
                    let transform =
                        Mat4::from_scale_rotation_translation(scale, rotation, translation);
                    pose.set_local_transform(*bone_index, transform);
                } else if weight > 0.001 {
                    // Blend with existing transform
                    if let Some(current) = pose.local_transform(*bone_index) {
                        // Extract current TRS
                        let current_translation = current.col(3).truncate();
                        let current_scale = Vec3::new(
                            current.col(0).truncate().length(),
                            current.col(1).truncate().length(),
                            current.col(2).truncate().length(),
                        );
                        let current_rotation = Quat::from_mat4(&current);

                        // Blend
                        let blended_translation = current_translation.lerp(translation, weight);
                        let blended_rotation = current_rotation.slerp(rotation, weight);
                        let blended_scale = current_scale.lerp(scale, weight);

                        let blended = Mat4::from_scale_rotation_translation(
                            blended_scale,
                            blended_rotation,
                            blended_translation,
                        );
                        pose.set_local_transform(*bone_index, blended);
                    }
                }
            }
        }
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Animation Systems
// ============================================================================

/// Updates animation players with the given delta time.
///
/// This function should be called each frame to advance animation playback.
/// You can create a system that calls this function with the appropriate delta time.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Query, Schedule, IntoSystemConfigs};
/// use praxis_scene::{Skeleton, AnimationPlayer, AnimatedPose, update_animations};
///
/// fn animation_system(
///     mut query: Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>
/// ) {
///     let delta_time = 0.016; // Get from your timing system
///     update_animations(delta_time, &mut query);
/// }
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(animation_system);
/// ```
pub fn update_animations(
    delta_time: f32,
    query: &mut Query<(&Skeleton, &mut AnimationPlayer, &mut AnimatedPose)>,
) {
    for (skeleton, mut player, mut pose) in query.iter_mut() {
        // Update animation playback times
        player.update(delta_time);

        // Evaluate animations and update pose
        *pose = player.evaluate(skeleton);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_bone_creation() {
        let bone = Bone::new("Root".to_string(), None);
        assert_eq!(bone.name, "Root");
        assert_eq!(bone.parent_index, None);
        assert_eq!(bone.bind_pose_translation, Vec3::ZERO);
    }

    #[test]
    fn test_bone_bind_pose_matrix() {
        let bone = Bone::with_bind_pose(
            "Test".to_string(),
            None,
            Vec3::new(1.0, 2.0, 3.0),
            Quat::IDENTITY,
            Vec3::ONE,
        );

        let matrix = bone.bind_pose_matrix();
        let translation = matrix.col(3).truncate();
        assert_eq!(translation, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_skeleton_creation() {
        let bones = vec![
            Bone::new("Root".to_string(), None),
            Bone::new("Child".to_string(), Some(0)),
        ];

        let skeleton = Skeleton::new(bones);
        assert_eq!(skeleton.bone_count(), 2);
        assert_eq!(skeleton.find_bone("Root"), Some(0));
        assert_eq!(skeleton.find_bone("Child"), Some(1));
    }

    #[test]
    fn test_keyframe_interpolation() {
        let mut track = BoneTrack::new();
        track.add_translation_keyframe(0.0, Vec3::ZERO);
        track.add_translation_keyframe(1.0, Vec3::new(10.0, 0.0, 0.0));

        let result = track.sample_translation(0.5);
        assert!(result.is_some());
        let value = result.unwrap();
        assert!((value.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_clip() {
        let mut clip = AnimationClip::new("Test".to_string(), 2.0);
        assert_eq!(clip.name(), "Test");
        assert!((clip.duration() - 2.0).abs() < EPSILON);

        clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
        clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));

        assert_eq!(clip.track_count(), 1);
        assert!(clip.bone_track(0).is_some());
    }

    #[test]
    fn test_animated_pose() {
        let mut pose = AnimatedPose::new(2);

        pose.set_local_transform(0, Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)));

        let transform = pose.local_transform(0);
        assert!(transform.is_some());
    }

    #[test]
    fn test_animation_player_creation() {
        let player = AnimationPlayer::new();
        assert_eq!(player.clips().len(), 0);
        assert_eq!(player.playing_clips().len(), 0);
    }

    #[test]
    fn test_animation_player_add_clip() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Walk".to_string(), 2.0);

        player.add_clip("Walk".to_string(), clip);

        assert!(player.clip("Walk").is_some());
        assert_eq!(player.clips().len(), 1);
    }

    #[test]
    fn test_animation_player_playback() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Walk".to_string(), 2.0);
        player.add_clip("Walk".to_string(), clip);

        assert!(!player.is_playing("Walk"));

        player.play("Walk");
        assert!(player.is_playing("Walk"));

        player.pause("Walk");
        assert!(!player.is_playing("Walk"));

        player.resume("Walk");
        assert!(player.is_playing("Walk"));

        player.stop("Walk");
        assert!(!player.is_playing("Walk"));
    }

    #[test]
    fn test_animation_player_update() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Test".to_string(), 2.0);
        player.add_clip("Test".to_string(), clip);

        player.play("Test");
        let time = player.current_time("Test").unwrap();
        assert!((time - 0.0).abs() < EPSILON);

        player.update(0.5);
        let time = player.current_time("Test").unwrap();
        assert!((time - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_animation_player_looping() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Loop".to_string(), 1.0);
        player.add_clip("Loop".to_string(), clip);

        player.play("Loop");
        player.set_looping("Loop", true);

        player.update(1.5);

        let time = player.current_time("Loop").unwrap();
        assert!((time - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_animation_player_speed() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Fast".to_string(), 2.0);
        player.add_clip("Fast".to_string(), clip);

        player.play("Fast");
        player.set_speed("Fast", 2.0);

        player.update(0.5);

        let time = player.current_time("Fast").unwrap();
        assert!((time - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_player_weight() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Weighted".to_string(), 1.0);
        player.add_clip("Weighted".to_string(), clip);

        player.play("Weighted");
        player.set_weight("Weighted", 0.5);

        let weights: Vec<f32> = player.playing_clips.values().map(|p| p.weight).collect();

        assert_eq!(weights.len(), 1);
        assert!((weights[0] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_animation_player_weight_clamping() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Test".to_string(), 1.0);
        player.add_clip("Test".to_string(), clip);

        player.play("Test");
        player.set_weight("Test", -0.5);
        assert!((player.playing_clips.get("Test").unwrap().weight - 0.0).abs() < EPSILON);

        player.set_weight("Test", 1.5);
        assert!((player.playing_clips.get("Test").unwrap().weight - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_animation_player_set_time() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Test".to_string(), 2.0);
        player.add_clip("Test".to_string(), clip);

        player.play("Test");
        player.set_time("Test", 1.5);

        let time = player.current_time("Test").unwrap();
        assert!((time - 1.5).abs() < EPSILON);
    }

    #[test]
    fn test_animation_player_non_looping_stops() {
        let mut player = AnimationPlayer::new();
        let clip = AnimationClip::new("Once".to_string(), 1.0);
        player.add_clip("Once".to_string(), clip);

        player.play("Once");
        player.set_looping("Once", false);

        player.update(1.5);

        assert!(!player.is_playing("Once"));
        assert_eq!(player.playing_clips().len(), 0);
    }

    #[test]
    fn test_animation_player_multiple_clips() {
        let mut player = AnimationPlayer::new();
        let clip1 = AnimationClip::new("Walk".to_string(), 1.0);
        let clip2 = AnimationClip::new("Run".to_string(), 0.8);
        player.add_clip("Walk".to_string(), clip1);
        player.add_clip("Run".to_string(), clip2);

        player.play("Walk");
        player.play("Run");

        assert!(player.is_playing("Walk"));
        assert!(player.is_playing("Run"));
        assert_eq!(player.playing_clips().len(), 2);
    }

    #[test]
    fn test_animation_player_evaluate_with_skeleton() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::ZERO,
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child".to_string(),
                Some(0),
                Vec3::new(0.0, 1.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];
        let skeleton = Skeleton::new(bones);

        let mut clip = AnimationClip::new("Test".to_string(), 1.0);
        clip.add_translation_keyframe(0, 0.0, Vec3::ZERO);
        clip.add_translation_keyframe(0, 1.0, Vec3::new(1.0, 0.0, 0.0));

        let mut player = AnimationPlayer::new();
        player.add_clip("Test".to_string(), clip);
        player.play("Test");

        player.update(0.5);
        let pose = player.evaluate(&skeleton);

        assert_eq!(pose.local_transforms().len(), 2);
        assert!(pose.world_transform(0).is_some());
        assert!(pose.world_transform(1).is_some());
    }

    #[test]
    fn test_skeleton_hierarchy_propagation() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::ZERO,
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child1".to_string(),
                Some(0),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child2".to_string(),
                Some(1),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];
        let skeleton = Skeleton::new(bones);
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        pose.update_world_transforms(&skeleton);

        let root_world = pose.world_transform(0).unwrap();
        let child1_world = pose.world_transform(1).unwrap();
        let child2_world = pose.world_transform(2).unwrap();

        let root_pos = root_world.col(3).truncate();
        let child1_pos = child1_world.col(3).truncate();
        let child2_pos = child2_world.col(3).truncate();

        assert_eq!(root_pos, Vec3::ZERO);
        assert_eq!(child1_pos, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(child2_pos, Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_skeleton_hierarchy_with_rotation() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::ZERO,
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child".to_string(),
                Some(0),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];
        let skeleton = Skeleton::new(bones);
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        pose.update_world_transforms(&skeleton);

        let child_world = pose.world_transform(1).unwrap();
        let child_pos = child_world.col(3).truncate();

        assert!((child_pos.x - 0.0).abs() < 0.001);
        assert!((child_pos.z + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_skeleton_hierarchy_with_scale() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::ZERO,
                Quat::IDENTITY,
                Vec3::splat(2.0),
            ),
            Bone::with_bind_pose(
                "Child".to_string(),
                Some(0),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];
        let skeleton = Skeleton::new(bones);
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        pose.update_world_transforms(&skeleton);

        let child_world = pose.world_transform(1).unwrap();
        let child_pos = child_world.col(3).truncate();

        assert!((child_pos.x - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_cross_fade_transition_creation() {
        let transition = CrossFadeTransition::new("Idle".to_string(), "Walk".to_string(), 0.5);

        assert_eq!(transition.from_clip, "Idle");
        assert_eq!(transition.to_clip, "Walk");
        assert!((transition.duration - 0.5).abs() < EPSILON);
        assert!((transition.elapsed - 0.0).abs() < EPSILON);
        assert!(!transition.is_complete());
    }

    #[test]
    fn test_cross_fade_blend_weight() {
        let mut transition = CrossFadeTransition::new("A".to_string(), "B".to_string(), 1.0);

        assert!((transition.blend_weight() - 0.0).abs() < EPSILON);

        transition.elapsed = 0.5;
        assert!((transition.blend_weight() - 0.5).abs() < EPSILON);

        transition.elapsed = 1.0;
        assert!((transition.blend_weight() - 1.0).abs() < EPSILON);
        assert!(transition.is_complete());

        transition.elapsed = 1.5;
        assert!((transition.blend_weight() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_cross_fade_zero_duration() {
        let transition = CrossFadeTransition::new("A".to_string(), "B".to_string(), 0.0);

        assert!((transition.blend_weight() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_cross_fade_update() {
        let mut transition = CrossFadeTransition::new("A".to_string(), "B".to_string(), 1.0);

        transition.update(0.3);
        assert!((transition.elapsed - 0.3).abs() < EPSILON);
        assert!(!transition.is_complete());

        transition.update(0.5);
        assert!((transition.elapsed - 0.8).abs() < EPSILON);
        assert!(!transition.is_complete());

        transition.update(0.5);
        assert!((transition.elapsed - 1.3).abs() < EPSILON);
        assert!(transition.is_complete());
    }

    #[test]
    fn test_blend_node_1d_creation() {
        let node = BlendNode1D::new();
        assert_eq!(node.clips.len(), 0);
        assert!((node.parameter - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_1d_add_clips() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 1.0);
        node.add_clip("Run", 2.0);

        assert_eq!(node.clips.len(), 3);
    }

    #[test]
    fn test_blend_node_1d_single_clip() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);

        let weights = node.compute_weights();
        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Idle");
        assert!((weights[0].1 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_1d_interpolation() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 1.0);
        node.add_clip("Run", 2.0);

        node.set_parameter(0.5);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 2);
        assert_eq!(weights[0].0, "Idle");
        assert_eq!(weights[1].0, "Walk");
        assert!((weights[0].1 - 0.5).abs() < 0.001);
        assert!((weights[1].1 - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_blend_node_1d_exact_match() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 1.0);
        node.add_clip("Run", 2.0);

        node.set_parameter(1.0);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Walk");
        assert!((weights[0].1 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_1d_below_range() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 1.0);

        node.set_parameter(-0.5);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Idle");
        assert!((weights[0].1 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_1d_above_range() {
        let mut node = BlendNode1D::new();
        node.add_clip("Walk", 1.0);
        node.add_clip("Run", 2.0);

        node.set_parameter(3.0);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Run");
        assert!((weights[0].1 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_1d_three_clips_middle() {
        let mut node = BlendNode1D::new();
        node.add_clip("Idle", 0.0);
        node.add_clip("Walk", 1.0);
        node.add_clip("Run", 2.0);

        node.set_parameter(1.5);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 2);
        assert_eq!(weights[0].0, "Walk");
        assert_eq!(weights[1].0, "Run");
        assert!((weights[0].1 - 0.5).abs() < 0.001);
        assert!((weights[1].1 - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_blend_node_2d_creation() {
        let node = BlendNode2D::new();
        assert_eq!(node.clips.len(), 0);
        assert!((node.parameter_x - 0.0).abs() < EPSILON);
        assert!((node.parameter_y - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_2d_add_clips() {
        let mut node = BlendNode2D::new();
        node.add_clip("Forward", 0.0, 1.0);
        node.add_clip("Backward", 0.0, -1.0);
        node.add_clip("Left", -1.0, 0.0);
        node.add_clip("Right", 1.0, 0.0);

        assert_eq!(node.clips.len(), 4);
    }

    #[test]
    fn test_blend_node_2d_single_clip() {
        let mut node = BlendNode2D::new();
        node.add_clip("Idle", 0.0, 0.0);

        let weights = node.compute_weights();
        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Idle");
        assert!((weights[0].1 - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_blend_node_2d_exact_match() {
        let mut node = BlendNode2D::new();
        node.add_clip("Forward", 0.0, 1.0);
        node.add_clip("Backward", 0.0, -1.0);

        node.set_parameters(0.0, 1.0);
        let weights = node.compute_weights();

        assert_eq!(weights.len(), 1);
        assert_eq!(weights[0].0, "Forward");
        assert!((weights[0].1 - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_blend_node_2d_distance_weighting() {
        let mut node = BlendNode2D::new();
        node.add_clip("NE", 1.0, 1.0);
        node.add_clip("NW", -1.0, 1.0);
        node.add_clip("SE", 1.0, -1.0);
        node.add_clip("SW", -1.0, -1.0);

        node.set_parameters(0.5, 0.5);
        let weights = node.compute_weights();

        let total_weight: f32 = weights.iter().map(|(_, w)| w).sum();
        assert!((total_weight - 1.0).abs() < 0.001);

        let ne_weight = weights.iter().find(|(n, _)| n == "NE").map(|(_, w)| w);
        assert!(ne_weight.is_some());
    }

    #[test]
    fn test_blend_node_2d_filters_low_weights() {
        let mut node = BlendNode2D::new();
        node.add_clip("Close", 0.0, 0.0);
        node.add_clip("Far", 10.0, 10.0);

        node.set_parameters(0.1, 0.1);
        let weights = node.compute_weights();

        let has_far = weights.iter().any(|(n, _)| n == "Far");
        assert!(!has_far);
    }

    #[test]
    fn test_additive_blend_node() {
        let mut node = AdditiveBlendNode::new();
        node.set_base("Walk");
        node.set_additive("HeadNod");
        node.set_weight(0.7);

        let (base, additive, weight) = node.get_clips();
        assert_eq!(base, Some("Walk".to_string()));
        assert_eq!(additive, Some("HeadNod".to_string()));
        assert!((weight - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_additive_blend_node_weight_clamping() {
        let mut node = AdditiveBlendNode::new();
        node.set_weight(-0.5);
        let (_, _, weight) = node.get_clips();
        assert!((weight - 0.0).abs() < EPSILON);

        node.set_weight(1.5);
        let (_, _, weight) = node.get_clips();
        assert!((weight - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_bone_mask_creation() {
        let mask = BoneMask::with_bone_count(5);
        assert_eq!(mask.enabled_bones.len(), 5);
        assert!(!mask.is_bone_enabled(0));
    }

    #[test]
    fn test_bone_mask_all_enabled() {
        let mask = BoneMask::all_enabled(3);
        assert!(mask.is_bone_enabled(0));
        assert!(mask.is_bone_enabled(1));
        assert!(mask.is_bone_enabled(2));
    }

    #[test]
    fn test_bone_mask_enable_disable() {
        let mut mask = BoneMask::with_bone_count(3);

        assert!(!mask.is_bone_enabled(0));
        mask.enable_bone(0);
        assert!(mask.is_bone_enabled(0));

        mask.disable_bone(0);
        assert!(!mask.is_bone_enabled(0));
    }

    #[test]
    fn test_bone_mask_bone_weight() {
        let mut mask = BoneMask::with_bone_count(3);

        assert!((mask.bone_weight(0) - 0.0).abs() < EPSILON);

        mask.enable_bone(0);
        assert!((mask.bone_weight(0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_bone_mask_auto_resize() {
        let mut mask = BoneMask::new();

        mask.enable_bone(5);
        assert!(mask.is_bone_enabled(5));
        assert_eq!(mask.enabled_bones.len(), 6);
    }

    #[test]
    fn test_bone_mask_with_skeleton() {
        let bones = vec![
            Bone::new("Root".to_string(), None),
            Bone::new("Child1".to_string(), Some(0)),
            Bone::new("Child2".to_string(), Some(0)),
            Bone::new("GrandChild".to_string(), Some(1)),
        ];
        let skeleton = Skeleton::new(bones);

        let mut mask = BoneMask::with_bone_count(skeleton.bone_count());
        mask.enable_bone_and_children_with_skeleton(1, &skeleton);

        assert!(!mask.is_bone_enabled(0));
        assert!(mask.is_bone_enabled(1));
        assert!(!mask.is_bone_enabled(2));
        assert!(mask.is_bone_enabled(3));
    }

    #[test]
    fn test_animation_layer_creation() {
        let layer = AnimationLayer::new(0.5);
        assert!((layer.weight() - 0.5).abs() < EPSILON);
        assert_eq!(layer.blend_mode(), LayerBlendMode::Override);
        assert!(layer.current_clip().is_none());
    }

    #[test]
    fn test_animation_layer_play_stop() {
        let mut layer = AnimationLayer::new(1.0);

        assert!(layer.current_clip().is_none());

        layer.play("Wave");
        assert_eq!(layer.current_clip(), Some("Wave"));
        assert!((layer.time() - 0.0).abs() < EPSILON);

        layer.stop();
        assert!(layer.current_clip().is_none());
    }

    #[test]
    fn test_animation_layer_update() {
        let mut layer = AnimationLayer::new(1.0);
        layer.play("Test");

        layer.update(0.5, 2.0);
        assert!((layer.time() - 0.5).abs() < EPSILON);

        layer.update(0.5, 2.0);
        assert!((layer.time() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_animation_layer_looping() {
        let mut layer = AnimationLayer::new(1.0);
        layer.play("Loop");
        layer.set_looping(true);

        layer.update(2.5, 1.0);
        assert!((layer.time() - 0.5).abs() < 0.001);
        assert!(layer.current_clip().is_some());
    }

    #[test]
    fn test_animation_layer_non_looping() {
        let mut layer = AnimationLayer::new(1.0);
        layer.play("Once");
        layer.set_looping(false);

        layer.update(2.0, 1.0);
        assert_eq!(layer.time(), 1.0);
        assert!(layer.current_clip().is_none());
    }

    #[test]
    fn test_animation_layer_speed() {
        let mut layer = AnimationLayer::new(1.0);
        layer.play("Fast");
        layer.set_speed(2.0);

        layer.update(0.5, 2.0);
        assert!((layer.time() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_animation_layer_with_mask() {
        let mut layer = AnimationLayer::new(1.0);
        let mut mask = BoneMask::with_bone_count(5);
        mask.enable_bone(0);
        mask.enable_bone(2);

        layer.set_mask(mask);

        let layer_mask = layer.mask().unwrap();
        assert!(layer_mask.is_bone_enabled(0));
        assert!(!layer_mask.is_bone_enabled(1));
        assert!(layer_mask.is_bone_enabled(2));
    }

    #[test]
    fn test_animation_blender_creation() {
        let blender = AnimationBlender::new();
        assert_eq!(blender.clips.len(), 0);
        assert!(blender.current_clip().is_none());
        assert!(!blender.is_cross_fading());
    }

    #[test]
    fn test_animation_blender_add_clip() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Test".to_string(), 1.0);

        blender.add_clip("Test", clip);
        assert!(blender.clip("Test").is_some());
    }

    #[test]
    fn test_animation_blender_play() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Walk".to_string(), 1.0);
        blender.add_clip("Walk", clip);

        blender.play("Walk");
        assert_eq!(blender.current_clip(), Some("Walk"));
        assert!((blender.current_time() - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_animation_blender_cross_fade() {
        let mut blender = AnimationBlender::new();
        let clip1 = AnimationClip::new("Idle".to_string(), 1.0);
        let clip2 = AnimationClip::new("Walk".to_string(), 1.0);
        blender.add_clip("Idle", clip1);
        blender.add_clip("Walk", clip2);

        blender.play("Idle");
        blender.cross_fade("Idle", "Walk", 0.5);

        assert!(blender.is_cross_fading());
        assert_eq!(blender.current_clip(), Some("Idle"));
    }

    #[test]
    fn test_animation_blender_cross_fade_completion() {
        let mut blender = AnimationBlender::new();
        let clip1 = AnimationClip::new("Idle".to_string(), 2.0);
        let clip2 = AnimationClip::new("Walk".to_string(), 2.0);
        blender.add_clip("Idle", clip1);
        blender.add_clip("Walk", clip2);

        blender.play("Idle");
        blender.update(0.5);
        blender.cross_fade("Idle", "Walk", 0.5);

        assert!(blender.is_cross_fading());

        blender.update(0.6);

        assert!(!blender.is_cross_fading());
        assert_eq!(blender.current_clip(), Some("Walk"));
    }

    #[test]
    fn test_animation_blender_layers() {
        let mut blender = AnimationBlender::new();
        let layer1 = AnimationLayer::new(1.0);
        let layer2 = AnimationLayer::new(0.5);

        blender.add_layer(layer1);
        blender.add_layer(layer2);

        assert_eq!(blender.layer_count(), 2);
        assert!((blender.layer(0).unwrap().weight() - 1.0).abs() < EPSILON);
        assert!((blender.layer(1).unwrap().weight() - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_animation_blender_play_on_layer() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Wave".to_string(), 1.0);
        blender.add_clip("Wave", clip);

        let layer = AnimationLayer::new(1.0);
        blender.add_layer(layer);

        blender.play_on_layer(0, "Wave");

        assert_eq!(blender.layer(0).unwrap().current_clip(), Some("Wave"));
    }

    #[test]
    fn test_animation_blender_update_base_layer() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Test".to_string(), 2.0);
        blender.add_clip("Test", clip);

        blender.play("Test");
        blender.update(0.5);

        assert!((blender.current_time() - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_animation_blender_looping() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Loop".to_string(), 1.0);
        blender.add_clip("Loop", clip);

        blender.play("Loop");
        blender.set_looping(true);
        blender.update(1.5);

        assert!((blender.current_time() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_animation_blender_non_looping() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Once".to_string(), 1.0);
        blender.add_clip("Once", clip);

        blender.play("Once");
        blender.set_looping(false);
        blender.update(2.0);

        assert!((blender.current_time() - 1.0).abs() < EPSILON);
        assert!(blender.current_clip().is_none());
    }

    #[test]
    fn test_animation_blender_speed() {
        let mut blender = AnimationBlender::new();
        let clip = AnimationClip::new("Fast".to_string(), 2.0);
        blender.add_clip("Fast", clip);

        blender.play("Fast");
        blender.set_speed(2.0);
        blender.update(0.5);

        assert!((blender.current_time() - 1.0).abs() < EPSILON);
    }

    #[test]
    fn test_animation_blender_blend_tree_1d() {
        let mut blender = AnimationBlender::new();
        let clip1 = AnimationClip::new("Idle".to_string(), 1.0);
        let clip2 = AnimationClip::new("Walk".to_string(), 1.0);
        blender.add_clip("Idle", clip1);
        blender.add_clip("Walk", clip2);

        let mut blend_1d = BlendNode1D::new();
        blend_1d.add_clip("Idle", 0.0);
        blend_1d.add_clip("Walk", 1.0);
        blend_1d.set_parameter(0.5);

        blender.add_blend_tree("Movement", BlendNode::Blend1D(blend_1d));
        blender.activate_blend_tree("Movement");

        assert_eq!(blender.active_blend_tree(), Some("Movement"));
    }

    #[test]
    fn test_animation_blender_set_blend_parameter() {
        let mut blender = AnimationBlender::new();
        let mut blend_1d = BlendNode1D::new();
        blend_1d.add_clip("Idle", 0.0);
        blend_1d.add_clip("Walk", 1.0);

        blender.add_blend_tree("Movement", BlendNode::Blend1D(blend_1d));
        blender.set_blend_parameter("Movement", 0.75);

        if let Some(BlendNode::Blend1D(node)) = blender.blend_trees.get("Movement") {
            assert!((node.parameter() - 0.75).abs() < EPSILON);
        } else {
            panic!("Expected Blend1D node");
        }
    }

    #[test]
    fn test_animation_blender_set_blend_parameters_2d() {
        let mut blender = AnimationBlender::new();
        let mut blend_2d = BlendNode2D::new();
        blend_2d.add_clip("Forward", 0.0, 1.0);

        blender.add_blend_tree("Movement", BlendNode::Blend2D(blend_2d));
        blender.set_blend_parameters_2d("Movement", 0.5, 0.8);

        if let Some(BlendNode::Blend2D(node)) = blender.blend_trees.get("Movement") {
            let (x, y) = node.parameters();
            assert!((x - 0.5).abs() < EPSILON);
            assert!((y - 0.8).abs() < EPSILON);
        } else {
            panic!("Expected Blend2D node");
        }
    }

    #[test]
    fn test_animated_pose_skinning_matrices() {
        let bones = vec![
            Bone::with_bind_pose(
                "Root".to_string(),
                None,
                Vec3::ZERO,
                Quat::IDENTITY,
                Vec3::ONE,
            ),
            Bone::with_bind_pose(
                "Child".to_string(),
                Some(0),
                Vec3::new(1.0, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        ];
        let skeleton = Skeleton::new(bones);
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        pose.update_world_transforms(&skeleton);
        pose.update_skinning_matrices(&skeleton);

        assert_eq!(pose.skinning_matrices().len(), 2);
        assert!(pose.skinning_matrices()[0] != Mat4::ZERO);
        assert!(pose.skinning_matrices()[1] != Mat4::ZERO);
    }
}

// ============================================================================
// Animation Blending System
// ============================================================================

/// State of a cross-fade transition between two animations.
#[derive(Debug, Clone)]
pub struct CrossFadeTransition {
    /// Name of the source animation (fading out).
    pub from_clip: String,

    /// Name of the target animation (fading in).
    pub to_clip: String,

    /// Total duration of the transition in seconds.
    pub duration: f32,

    /// Current elapsed time in the transition.
    pub elapsed: f32,

    /// Playback time in the source animation when transition started.
    pub from_time: f32,

    /// Playback time to start at in the target animation.
    pub to_time: f32,
}

impl CrossFadeTransition {
    /// Creates a new cross-fade transition.
    pub fn new(from_clip: String, to_clip: String, duration: f32) -> Self {
        Self {
            from_clip,
            to_clip,
            duration,
            elapsed: 0.0,
            from_time: 0.0,
            to_time: 0.0,
        }
    }

    /// Gets the blend weight (0.0 = fully from, 1.0 = fully to).
    pub fn blend_weight(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Returns true if the transition is complete.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Updates the transition with delta time.
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed += delta_time;
    }
}

/// 1D blend space for blending animations along a single parameter.
///
/// Useful for blending between animations based on a single value like speed,
/// turn rate, or any other scalar parameter.
#[derive(Debug, Clone)]
pub struct BlendNode1D {
    /// Clips with their parameter values.
    clips: Vec<(String, f32)>,

    /// Current blend parameter value.
    parameter: f32,
}

impl BlendNode1D {
    /// Creates a new 1D blend node.
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            parameter: 0.0,
        }
    }

    /// Adds a clip at a specific parameter value.
    pub fn add_clip(&mut self, name: impl Into<String>, parameter: f32) {
        self.clips.push((name.into(), parameter));
        self.clips.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    }

    /// Sets the current blend parameter.
    pub fn set_parameter(&mut self, value: f32) {
        self.parameter = value;
    }

    /// Gets the current blend parameter.
    pub fn parameter(&self) -> f32 {
        self.parameter
    }

    /// Computes blend weights for all clips based on the current parameter.
    pub fn compute_weights(&self) -> Vec<(String, f32)> {
        if self.clips.is_empty() {
            return Vec::new();
        }

        if self.clips.len() == 1 {
            return vec![(self.clips[0].0.clone(), 1.0)];
        }

        let mut result = Vec::new();

        let mut before_idx = None;
        let mut after_idx = None;

        for (i, (_, param)) in self.clips.iter().enumerate() {
            if *param <= self.parameter {
                before_idx = Some(i);
            }
            if *param >= self.parameter && after_idx.is_none() {
                after_idx = Some(i);
                break;
            }
        }

        match (before_idx, after_idx) {
            (Some(before), Some(after)) if before != after => {
                let (_, param_before) = &self.clips[before];
                let (_, param_after) = &self.clips[after];
                let range = param_after - param_before;

                if range > 0.0 {
                    let t = (self.parameter - param_before) / range;
                    result.push((self.clips[before].0.clone(), 1.0 - t));
                    result.push((self.clips[after].0.clone(), t));
                } else {
                    result.push((self.clips[before].0.clone(), 1.0));
                }
            }
            (Some(idx), _) | (_, Some(idx)) => {
                result.push((self.clips[idx].0.clone(), 1.0));
            }
            _ => {}
        }

        result
    }
}

impl Default for BlendNode1D {
    fn default() -> Self {
        Self::new()
    }
}

/// 2D blend space for blending animations along two parameters.
///
/// Useful for directional movement (forward/strafe), aiming (horizontal/vertical),
/// or any other dual-parameter blending scenario.
#[derive(Debug, Clone)]
pub struct BlendNode2D {
    /// Clips with their 2D parameter positions.
    clips: Vec<(String, f32, f32)>,

    /// Current X parameter value.
    parameter_x: f32,

    /// Current Y parameter value.
    parameter_y: f32,
}

impl BlendNode2D {
    /// Creates a new 2D blend node.
    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            parameter_x: 0.0,
            parameter_y: 0.0,
        }
    }

    /// Adds a clip at a specific 2D position.
    pub fn add_clip(&mut self, name: impl Into<String>, x: f32, y: f32) {
        self.clips.push((name.into(), x, y));
    }

    /// Sets the current blend parameters.
    pub fn set_parameters(&mut self, x: f32, y: f32) {
        self.parameter_x = x;
        self.parameter_y = y;
    }

    /// Gets the current blend parameters.
    pub fn parameters(&self) -> (f32, f32) {
        (self.parameter_x, self.parameter_y)
    }

    /// Computes blend weights using inverse distance weighting.
    pub fn compute_weights(&self) -> Vec<(String, f32)> {
        if self.clips.is_empty() {
            return Vec::new();
        }

        if self.clips.len() == 1 {
            return vec![(self.clips[0].0.clone(), 1.0)];
        }

        let mut result = Vec::new();
        let mut total_weight = 0.0;

        for (name, x, y) in &self.clips {
            let dx = x - self.parameter_x;
            let dy = y - self.parameter_y;
            let dist_sq = dx.mul_add(dx, dy * dy);

            let weight = if dist_sq < 0.0001 {
                1000.0
            } else {
                1.0 / dist_sq
            };

            result.push((name.clone(), weight));
            total_weight += weight;
        }

        if total_weight > 0.0 {
            for (_, weight) in &mut result {
                *weight /= total_weight;
            }
        }

        result.retain(|(_, w)| *w > 0.01);

        total_weight = result.iter().map(|(_, w)| w).sum();
        if total_weight > 0.0 {
            for (_, weight) in &mut result {
                *weight /= total_weight;
            }
        }

        result
    }
}

impl Default for BlendNode2D {
    fn default() -> Self {
        Self::new()
    }
}

/// Additive blend node for layering animations additively.
///
/// Additive blending adds the animation's delta from a reference pose
/// to the base animation, rather than replacing it.
#[derive(Debug, Clone)]
pub struct AdditiveBlendNode {
    /// Base clip name.
    base_clip: Option<String>,

    /// Additive clip name.
    additive_clip: Option<String>,

    /// Weight of the additive animation (0.0 to 1.0).
    weight: f32,
}

impl AdditiveBlendNode {
    /// Creates a new additive blend node.
    pub fn new() -> Self {
        Self {
            base_clip: None,
            additive_clip: None,
            weight: 1.0,
        }
    }

    /// Sets the base clip.
    pub fn set_base(&mut self, clip_name: impl Into<String>) {
        self.base_clip = Some(clip_name.into());
    }

    /// Sets the additive clip.
    pub fn set_additive(&mut self, clip_name: impl Into<String>) {
        self.additive_clip = Some(clip_name.into());
    }

    /// Sets the additive weight.
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight.clamp(0.0, 1.0);
    }

    /// Gets the base and additive clips with weight.
    pub fn get_clips(&self) -> (Option<String>, Option<String>, f32) {
        (
            self.base_clip.clone(),
            self.additive_clip.clone(),
            self.weight,
        )
    }
}

impl Default for AdditiveBlendNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Blend tree node types.
#[derive(Debug, Clone)]
pub enum BlendNode {
    /// 1D blend space.
    Blend1D(BlendNode1D),

    /// 2D blend space.
    Blend2D(BlendNode2D),

    /// Additive blend.
    Additive(AdditiveBlendNode),
}

impl From<BlendNode1D> for BlendNode {
    fn from(node: BlendNode1D) -> Self {
        Self::Blend1D(node)
    }
}

impl From<BlendNode2D> for BlendNode {
    fn from(node: BlendNode2D) -> Self {
        Self::Blend2D(node)
    }
}

impl From<AdditiveBlendNode> for BlendNode {
    fn from(node: AdditiveBlendNode) -> Self {
        Self::Additive(node)
    }
}

/// Bone mask for controlling which bones an animation layer affects.
#[derive(Debug, Clone)]
pub struct BoneMask {
    /// Bone indices that are enabled in this mask.
    enabled_bones: Vec<bool>,
}

impl BoneMask {
    /// Creates a new bone mask with all bones disabled.
    pub fn new() -> Self {
        Self {
            enabled_bones: Vec::new(),
        }
    }

    /// Creates a bone mask for a specific bone count.
    pub fn with_bone_count(bone_count: usize) -> Self {
        Self {
            enabled_bones: vec![false; bone_count],
        }
    }

    /// Creates a bone mask with all bones enabled.
    pub fn all_enabled(bone_count: usize) -> Self {
        Self {
            enabled_bones: vec![true; bone_count],
        }
    }

    /// Enables a specific bone.
    pub fn enable_bone(&mut self, bone_index: usize) {
        if bone_index >= self.enabled_bones.len() {
            self.enabled_bones.resize(bone_index + 1, false);
        }
        self.enabled_bones[bone_index] = true;
    }

    /// Disables a specific bone.
    pub fn disable_bone(&mut self, bone_index: usize) {
        if bone_index < self.enabled_bones.len() {
            self.enabled_bones[bone_index] = false;
        }
    }

    /// Enables a bone and all its children recursively.
    pub fn enable_bone_and_children(&mut self, bone_index: usize) {
        self.enable_bone(bone_index);
    }

    /// Enables a bone and all its children recursively using skeleton information.
    pub fn enable_bone_and_children_with_skeleton(
        &mut self,
        bone_index: usize,
        skeleton: &Skeleton,
    ) {
        self.enable_bone(bone_index);

        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                if bone.parent_index == Some(bone_index) {
                    self.enable_bone_and_children_with_skeleton(i, skeleton);
                }
            }
        }
    }

    /// Checks if a bone is enabled.
    pub fn is_bone_enabled(&self, bone_index: usize) -> bool {
        self.enabled_bones.get(bone_index).copied().unwrap_or(false)
    }

    /// Gets the weight for a specific bone (0.0 or 1.0).
    pub fn bone_weight(&self, bone_index: usize) -> f32 {
        if self.is_bone_enabled(bone_index) {
            1.0
        } else {
            0.0
        }
    }
}

impl Default for BoneMask {
    fn default() -> Self {
        Self::new()
    }
}

/// Blend mode for animation layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerBlendMode {
    /// Override the base animation (default).
    Override,

    /// Add to the base animation (additive blending).
    Additive,
}

/// Animation layer for layered animation playback.
///
/// Layers allow multiple animations to play simultaneously on different
/// parts of the skeleton. For example, an upper body aiming animation
/// can play on top of a lower body walking animation.
#[derive(Debug, Clone)]
pub struct AnimationLayer {
    /// Layer weight (0.0 to 1.0).
    weight: f32,

    /// Bone mask for this layer.
    mask: Option<BoneMask>,

    /// Blend mode for this layer.
    blend_mode: LayerBlendMode,

    /// Currently playing clip on this layer.
    current_clip: Option<String>,

    /// Playback time for the current clip.
    time: f32,

    /// Playback speed multiplier.
    speed: f32,

    /// Whether the animation should loop.
    looping: bool,
}

impl AnimationLayer {
    /// Creates a new animation layer with the given weight.
    pub fn new(weight: f32) -> Self {
        Self {
            weight: weight.clamp(0.0, 1.0),
            mask: None,
            blend_mode: LayerBlendMode::Override,
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            looping: true,
        }
    }

    /// Sets the layer weight.
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight.clamp(0.0, 1.0);
    }

    /// Gets the layer weight.
    pub fn weight(&self) -> f32 {
        self.weight
    }

    /// Sets the bone mask.
    pub fn set_mask(&mut self, mask: BoneMask) {
        self.mask = Some(mask);
    }

    /// Gets the bone mask.
    pub fn mask(&self) -> Option<&BoneMask> {
        self.mask.as_ref()
    }

    /// Sets the blend mode.
    pub fn set_blend_mode(&mut self, mode: LayerBlendMode) {
        self.blend_mode = mode;
    }

    /// Gets the blend mode.
    pub fn blend_mode(&self) -> LayerBlendMode {
        self.blend_mode
    }

    /// Plays a clip on this layer.
    pub fn play(&mut self, clip_name: impl Into<String>) {
        self.current_clip = Some(clip_name.into());
        self.time = 0.0;
    }

    /// Stops playback on this layer.
    pub fn stop(&mut self) {
        self.current_clip = None;
        self.time = 0.0;
    }

    /// Gets the currently playing clip.
    pub fn current_clip(&self) -> Option<&str> {
        self.current_clip.as_deref()
    }

    /// Gets the playback time.
    pub fn time(&self) -> f32 {
        self.time
    }

    /// Sets the playback time.
    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    /// Sets the playback speed.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Sets whether the animation should loop.
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Updates the layer with delta time.
    pub fn update(&mut self, delta_time: f32, clip_duration: f32) {
        if self.current_clip.is_some() {
            self.time += delta_time * self.speed;

            if self.time >= clip_duration {
                if self.looping {
                    self.time %= clip_duration;
                } else {
                    self.time = clip_duration;
                    self.current_clip = None;
                }
            }
        }
    }
}

/// Advanced animation blender component with cross-fading, blend trees, and layers.
///
/// This component provides sophisticated animation blending capabilities:
/// - Cross-fade transitions between animations
/// - Blend trees for parameter-driven blending
/// - Layered animation with bone masking
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{AnimationBlender, AnimationLayer};
/// use praxis_ecs::World;
///
/// let mut blender = AnimationBlender::new();
///
/// // Simple cross-fade
/// blender.cross_fade("Idle", "Walk", 0.3);
///
/// // Add a layer for upper body
/// let layer = AnimationLayer::new(1.0);
/// blender.add_layer(layer);
/// blender.play_on_layer(1, "Wave");
/// ```
#[derive(Component, Debug, Clone)]
pub struct AnimationBlender {
    /// Animation clips library.
    clips: HashMap<String, AnimationClip>,

    /// Base layer (layer 0) currently playing clip.
    base_clip: Option<String>,

    /// Base layer playback time.
    base_time: f32,

    /// Base layer playback speed.
    base_speed: f32,

    /// Base layer looping.
    base_looping: bool,

    /// Active cross-fade transition.
    cross_fade: Option<CrossFadeTransition>,

    /// Blend trees indexed by name.
    blend_trees: HashMap<String, BlendNode>,

    /// Currently active blend tree.
    active_blend_tree: Option<String>,

    /// Additional animation layers.
    layers: Vec<AnimationLayer>,
}

impl AnimationBlender {
    /// Creates a new animation blender.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            base_clip: None,
            base_time: 0.0,
            base_speed: 1.0,
            base_looping: true,
            cross_fade: None,
            blend_trees: HashMap::new(),
            active_blend_tree: None,
            layers: Vec::new(),
        }
    }

    /// Adds an animation clip to the library.
    pub fn add_clip(&mut self, name: impl Into<String>, clip: AnimationClip) {
        self.clips.insert(name.into(), clip);
    }

    /// Builder method to add a clip.
    ///
    /// # Must Use
    ///
    /// This method consumes `self` and returns a new instance with the clip added.
    /// Ignoring the return value will discard the clip and may result in missing animations.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_clip(mut self, name: impl Into<String>, clip: AnimationClip) -> Self {
        self.add_clip(name, clip);
        self
    }

    /// Gets a clip by name.
    pub fn clip(&self, name: &str) -> Option<&AnimationClip> {
        self.clips.get(name)
    }

    /// Plays an animation on the base layer immediately.
    pub fn play(&mut self, clip_name: impl Into<String>) {
        self.base_clip = Some(clip_name.into());
        self.base_time = 0.0;
        self.cross_fade = None;
        self.active_blend_tree = None;
    }

    /// Starts a cross-fade transition to a new animation.
    pub fn cross_fade(&mut self, from: impl Into<String>, to: impl Into<String>, duration: f32) {
        let from_name = from.into();
        let to_name = to.into();

        let mut transition = CrossFadeTransition::new(from_name.clone(), to_name, duration);
        transition.from_time = self.base_time;

        self.cross_fade = Some(transition);
        self.base_clip = Some(from_name);
    }

    /// Adds a blend tree.
    pub fn add_blend_tree(&mut self, name: impl Into<String>, node: BlendNode) {
        self.blend_trees.insert(name.into(), node);
    }

    /// Activates a blend tree.
    pub fn activate_blend_tree(&mut self, name: impl Into<String>) {
        let name = name.into();
        if self.blend_trees.contains_key(&name) {
            self.active_blend_tree = Some(name);
            self.base_clip = None;
            self.cross_fade = None;
        }
    }

    /// Sets a 1D blend parameter.
    pub fn set_blend_parameter(&mut self, tree_name: &str, value: f32) {
        if let Some(BlendNode::Blend1D(node)) = self.blend_trees.get_mut(tree_name) {
            node.set_parameter(value);
        }
    }

    /// Sets 2D blend parameters.
    pub fn set_blend_parameters_2d(&mut self, tree_name: &str, x: f32, y: f32) {
        if let Some(BlendNode::Blend2D(node)) = self.blend_trees.get_mut(tree_name) {
            node.set_parameters(x, y);
        }
    }

    /// Adds an animation layer.
    pub fn add_layer(&mut self, layer: AnimationLayer) {
        self.layers.push(layer);
    }

    /// Gets a layer by index.
    pub fn layer(&self, index: usize) -> Option<&AnimationLayer> {
        self.layers.get(index)
    }

    /// Gets a mutable layer by index.
    pub fn layer_mut(&mut self, index: usize) -> Option<&mut AnimationLayer> {
        self.layers.get_mut(index)
    }

    /// Plays an animation on a specific layer.
    pub fn play_on_layer(&mut self, layer_index: usize, clip_name: impl Into<String>) {
        if let Some(layer) = self.layers.get_mut(layer_index) {
            layer.play(clip_name);
        }
    }

    /// Sets the base layer looping.
    pub fn set_looping(&mut self, looping: bool) {
        self.base_looping = looping;
    }

    /// Sets the base layer playback speed.
    pub fn set_speed(&mut self, speed: f32) {
        self.base_speed = speed;
    }

    /// Gets the current base layer clip.
    pub fn current_clip(&self) -> Option<&str> {
        self.base_clip.as_deref()
    }

    /// Gets the current base layer time.
    pub fn current_time(&self) -> f32 {
        self.base_time
    }

    /// Checks if a cross-fade is currently active.
    pub fn is_cross_fading(&self) -> bool {
        self.cross_fade.is_some()
    }

    /// Gets the active blend tree name if any.
    pub fn active_blend_tree(&self) -> Option<&str> {
        self.active_blend_tree.as_deref()
    }

    /// Gets the number of layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Updates the blender with delta time.
    pub fn update(&mut self, delta_time: f32) {
        if let Some(ref mut transition) = self.cross_fade {
            transition.update(delta_time);

            if transition.is_complete() {
                self.base_clip = Some(transition.to_clip.clone());
                self.base_time = transition.to_time;
                self.cross_fade = None;
            }
        }

        if let Some(clip_name) = &self.base_clip {
            if let Some(clip) = self.clips.get(clip_name) {
                self.base_time += delta_time * self.base_speed;

                if self.base_time >= clip.duration() {
                    if self.base_looping {
                        self.base_time %= clip.duration();
                    } else {
                        self.base_time = clip.duration();
                        self.base_clip = None;
                    }
                }
            }
        }

        for layer in &mut self.layers {
            if let Some(clip_name) = layer.current_clip() {
                if let Some(clip) = self.clips.get(clip_name) {
                    layer.update(delta_time, clip.duration());
                }
            }
        }
    }

    /// Evaluates the blender and produces an animated pose.
    pub fn evaluate(&self, skeleton: &Skeleton) -> AnimatedPose {
        let mut pose = AnimatedPose::new(skeleton.bone_count());

        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        if let Some(ref transition) = self.cross_fade {
            self.evaluate_cross_fade(transition, &mut pose, skeleton);
        } else if let Some(ref tree_name) = self.active_blend_tree {
            self.evaluate_blend_tree(tree_name, &mut pose, skeleton);
        } else if let Some(ref clip_name) = self.base_clip {
            if let Some(clip) = self.clips.get(clip_name) {
                self.apply_clip_to_pose(clip, self.base_time, 1.0, &mut pose, skeleton);
            }
        }

        for layer in &self.layers {
            if let Some(clip_name) = layer.current_clip() {
                if let Some(clip) = self.clips.get(clip_name) {
                    self.apply_layer_to_pose(layer, clip, &mut pose, skeleton);
                }
            }
        }

        pose.update_world_transforms(skeleton);
        pose.update_skinning_matrices(skeleton);

        pose
    }

    fn evaluate_cross_fade(
        &self,
        transition: &CrossFadeTransition,
        pose: &mut AnimatedPose,
        skeleton: &Skeleton,
    ) {
        let blend_weight = transition.blend_weight();

        if let Some(from_clip) = self.clips.get(&transition.from_clip) {
            self.apply_clip_to_pose(
                from_clip,
                transition.from_time + transition.elapsed,
                1.0 - blend_weight,
                pose,
                skeleton,
            );
        }

        if let Some(to_clip) = self.clips.get(&transition.to_clip) {
            self.apply_clip_to_pose(
                to_clip,
                transition.to_time + transition.elapsed,
                blend_weight,
                pose,
                skeleton,
            );
        }
    }

    fn evaluate_blend_tree(&self, tree_name: &str, pose: &mut AnimatedPose, skeleton: &Skeleton) {
        if let Some(blend_node) = self.blend_trees.get(tree_name) {
            match blend_node {
                BlendNode::Blend1D(node) => {
                    let weights = node.compute_weights();
                    for (clip_name, weight) in weights {
                        if let Some(clip) = self.clips.get(&clip_name) {
                            self.apply_clip_to_pose(clip, self.base_time, weight, pose, skeleton);
                        }
                    }
                }
                BlendNode::Blend2D(node) => {
                    let weights = node.compute_weights();
                    for (clip_name, weight) in weights {
                        if let Some(clip) = self.clips.get(&clip_name) {
                            self.apply_clip_to_pose(clip, self.base_time, weight, pose, skeleton);
                        }
                    }
                }
                BlendNode::Additive(node) => {
                    let (base, additive, weight) = node.get_clips();

                    if let Some(base_name) = base {
                        if let Some(clip) = self.clips.get(&base_name) {
                            self.apply_clip_to_pose(clip, self.base_time, 1.0, pose, skeleton);
                        }
                    }

                    if let Some(additive_name) = additive {
                        if let Some(clip) = self.clips.get(&additive_name) {
                            self.apply_clip_to_pose(clip, self.base_time, weight, pose, skeleton);
                        }
                    }
                }
            }
        }
    }

    fn apply_layer_to_pose(
        &self,
        layer: &AnimationLayer,
        clip: &AnimationClip,
        pose: &mut AnimatedPose,
        skeleton: &Skeleton,
    ) {
        for (bone_index, track) in clip.bone_tracks() {
            let bone_weight = layer
                .mask()
                .map_or(1.0, |mask| mask.bone_weight(*bone_index));

            if bone_weight <= 0.0 {
                continue;
            }

            let final_weight = layer.weight() * bone_weight;

            if final_weight > 0.001 {
                if let Some(bone) = skeleton.bone(*bone_index) {
                    let translation = track
                        .sample_translation(layer.time())
                        .unwrap_or(bone.bind_pose_translation);
                    let rotation = track
                        .sample_rotation(layer.time())
                        .unwrap_or(bone.bind_pose_rotation);
                    let scale = track
                        .sample_scale(layer.time())
                        .unwrap_or(bone.bind_pose_scale);

                    if let Some(current) = pose.local_transform(*bone_index) {
                        let current_translation = current.col(3).truncate();
                        let current_scale = Vec3::new(
                            current.col(0).truncate().length(),
                            current.col(1).truncate().length(),
                            current.col(2).truncate().length(),
                        );
                        let current_rotation = Quat::from_mat4(&current);

                        let blended_translation =
                            current_translation.lerp(translation, final_weight);
                        let blended_rotation = current_rotation.slerp(rotation, final_weight);
                        let blended_scale = current_scale.lerp(scale, final_weight);

                        let blended = Mat4::from_scale_rotation_translation(
                            blended_scale,
                            blended_rotation,
                            blended_translation,
                        );
                        pose.set_local_transform(*bone_index, blended);
                    }
                }
            }
        }
    }

    fn apply_clip_to_pose(
        &self,
        clip: &AnimationClip,
        time: f32,
        weight: f32,
        pose: &mut AnimatedPose,
        skeleton: &Skeleton,
    ) {
        for (bone_index, track) in clip.bone_tracks() {
            if let Some(bone) = skeleton.bone(*bone_index) {
                let translation = track
                    .sample_translation(time)
                    .unwrap_or(bone.bind_pose_translation);
                let rotation = track
                    .sample_rotation(time)
                    .unwrap_or(bone.bind_pose_rotation);
                let scale = track.sample_scale(time).unwrap_or(bone.bind_pose_scale);

                if weight >= 0.999 {
                    let transform =
                        Mat4::from_scale_rotation_translation(scale, rotation, translation);
                    pose.set_local_transform(*bone_index, transform);
                } else if weight > 0.001 {
                    if let Some(current) = pose.local_transform(*bone_index) {
                        let current_translation = current.col(3).truncate();
                        let current_scale = Vec3::new(
                            current.col(0).truncate().length(),
                            current.col(1).truncate().length(),
                            current.col(2).truncate().length(),
                        );
                        let current_rotation = Quat::from_mat4(&current);

                        let blended_translation = current_translation.lerp(translation, weight);
                        let blended_rotation = current_rotation.slerp(rotation, weight);
                        let blended_scale = current_scale.lerp(scale, weight);

                        let blended = Mat4::from_scale_rotation_translation(
                            blended_scale,
                            blended_rotation,
                            blended_translation,
                        );
                        pose.set_local_transform(*bone_index, blended);
                    }
                }
            }
        }
    }
}

impl Default for AnimationBlender {
    fn default() -> Self {
        Self::new()
    }
}

/// Updates animation blenders with the given delta time.
///
/// This function should be called each frame to advance animation blending.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{Query, Schedule, IntoSystemConfigs};
/// use praxis_scene::{Skeleton, AnimationBlender, AnimatedPose, update_animation_blenders};
///
/// fn blender_system(
///     mut query: Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>
/// ) {
///     let delta_time = 0.016;
///     update_animation_blenders(delta_time, &mut query);
/// }
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(blender_system);
/// ```
pub fn update_animation_blenders(
    delta_time: f32,
    query: &mut Query<(&Skeleton, &mut AnimationBlender, &mut AnimatedPose)>,
) {
    for (skeleton, mut blender, mut pose) in query.iter_mut() {
        blender.update(delta_time);
        *pose = blender.evaluate(skeleton);
    }
}

// ============================================================================
// Inverse Kinematics (IK) System
// ============================================================================

/// IK constraint types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IkConstraintType {
    /// Two-bone IK (e.g., arm, leg).
    TwoBone,

    /// Chain IK for multiple bones (e.g., spine, tail).
    Chain,

    /// Look-at IK for orienting a bone toward a target.
    LookAt,
}

/// IK constraint for procedural bone positioning.
///
/// IK (Inverse Kinematics) allows you to specify an end effector position
/// and have the system automatically compute the bone rotations to reach it.
#[derive(Debug, Clone)]
pub struct IkConstraint {
    /// Type of IK constraint.
    constraint_type: IkConstraintType,

    /// End effector bone index (the bone that should reach the target).
    end_effector_bone: usize,

    /// Target position in world space.
    target_position: Vec3,

    /// Optional pole target for controlling the bend direction.
    pole_target: Option<Vec3>,

    /// Weight of the IK constraint (0.0 to 1.0).
    weight: f32,

    /// Maximum number of iterations for iterative solvers.
    max_iterations: u32,

    /// Tolerance for convergence (distance threshold).
    tolerance: f32,
}

impl IkConstraint {
    /// Creates a new two-bone IK constraint.
    pub fn new_two_bone(end_effector_bone: usize, target_position: Vec3) -> Self {
        Self {
            constraint_type: IkConstraintType::TwoBone,
            end_effector_bone,
            target_position,
            pole_target: None,
            weight: 1.0,
            max_iterations: 10,
            tolerance: 0.001,
        }
    }

    /// Creates a new chain IK constraint.
    pub fn new_chain(end_effector_bone: usize, target_position: Vec3, max_iterations: u32) -> Self {
        Self {
            constraint_type: IkConstraintType::Chain,
            end_effector_bone,
            target_position,
            pole_target: None,
            weight: 1.0,
            max_iterations,
            tolerance: 0.001,
        }
    }

    /// Creates a new look-at IK constraint.
    pub fn new_look_at(bone: usize, target_position: Vec3) -> Self {
        Self {
            constraint_type: IkConstraintType::LookAt,
            end_effector_bone: bone,
            target_position,
            pole_target: None,
            weight: 1.0,
            max_iterations: 1,
            tolerance: 0.001,
        }
    }

    /// Sets the pole target for controlling bend direction.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with the pole target set.
    /// Ignoring the return value will discard the configuration and the IK constraint will not use the pole target.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_pole_target(mut self, pole_target: Vec3) -> Self {
        self.pole_target = Some(pole_target);
        self
    }

    /// Sets the constraint weight.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with the weight set.
    /// Ignoring the return value will discard the weight configuration and the IK constraint will use the default weight.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Sets the target position.
    pub fn set_target(&mut self, target: Vec3) {
        self.target_position = target;
    }

    /// Sets the pole target.
    pub fn set_pole_target(&mut self, pole: Vec3) {
        self.pole_target = Some(pole);
    }

    /// Sets the weight.
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight.clamp(0.0, 1.0);
    }

    /// Gets the target position.
    pub fn target(&self) -> Vec3 {
        self.target_position
    }

    /// Gets the weight.
    pub fn weight(&self) -> f32 {
        self.weight
    }
}

/// IK solver for computing bone transforms.
pub struct IkSolver;

impl IkSolver {
    /// Solves a two-bone IK chain.
    ///
    /// This is commonly used for arms and legs where you have:
    /// - Root bone (shoulder/hip)
    /// - Middle bone (elbow/knee)
    /// - End effector (hand/foot)
    pub fn solve_two_bone(constraint: &IkConstraint, pose: &mut AnimatedPose, skeleton: &Skeleton) {
        let end_bone_idx = constraint.end_effector_bone;

        let Some(end_bone) = skeleton.bone(end_bone_idx) else {
            return;
        };

        let Some(middle_bone_idx) = end_bone.parent_index else {
            return;
        };

        let Some(middle_bone) = skeleton.bone(middle_bone_idx) else {
            return;
        };

        let Some(root_bone_idx) = middle_bone.parent_index else {
            return;
        };

        let root_pos = pose
            .world_transform(root_bone_idx)
            .map_or(Vec3::ZERO, |m| m.col(3).truncate());
        let middle_pos = pose
            .world_transform(middle_bone_idx)
            .map_or(Vec3::ZERO, |m| m.col(3).truncate());
        let end_pos = pose
            .world_transform(end_bone_idx)
            .map_or(Vec3::ZERO, |m| m.col(3).truncate());

        let upper_length = (middle_pos - root_pos).length();
        let lower_length = (end_pos - middle_pos).length();

        let target = constraint.target_position;
        let target_dir = (target - root_pos).normalize_or_zero();
        let target_dist = (target - root_pos).length();

        let chain_length = upper_length + lower_length;
        let clamped_dist = target_dist.min(chain_length - 0.001).max(0.001);

        // Validate bone lengths to avoid division by zero or NaN
        if upper_length < 0.001 || lower_length < 0.001 {
            return;
        }

        let cos_angle = (clamped_dist.mul_add(
            -clamped_dist,
            upper_length.mul_add(upper_length, lower_length * lower_length),
        ) / (2.0 * upper_length * lower_length))
            .clamp(-1.0, 1.0);
        let elbow_angle = std::f32::consts::PI - cos_angle.acos();

        let cos_root_angle = (lower_length.mul_add(
            -lower_length,
            upper_length.mul_add(upper_length, clamped_dist * clamped_dist),
        ) / (2.0 * upper_length * clamped_dist))
            .clamp(-1.0, 1.0);
        let root_angle = cos_root_angle.acos();

        let pole_dir = constraint.pole_target.map_or(Vec3::Y, |pole| {
            let to_pole = (pole - root_pos).normalize_or_zero();
            let perp = to_pole - target_dir * target_dir.dot(to_pole);
            perp.normalize_or_zero()
        });

        let bend_axis = target_dir.cross(pole_dir).normalize_or_zero();

        // Validate bend axis - if it's zero, we can't compute a valid rotation
        if bend_axis.length_squared() < 0.001 {
            return;
        }

        let root_rot = Quat::from_axis_angle(bend_axis, -root_angle);
        let root_final = Quat::from_rotation_arc(Vec3::X, target_dir) * root_rot;

        let middle_rot = Quat::from_axis_angle(bend_axis, elbow_angle);

        if let Some(current_root) = pose.local_transform(root_bone_idx) {
            let root_trans = current_root.col(3).truncate();
            let root_scale = Vec3::new(
                current_root.col(0).truncate().length(),
                current_root.col(1).truncate().length(),
                current_root.col(2).truncate().length(),
            );

            let blended_rot = Quat::from_mat4(&current_root).slerp(root_final, constraint.weight);
            pose.set_local_transform(
                root_bone_idx,
                Mat4::from_scale_rotation_translation(root_scale, blended_rot, root_trans),
            );
        }

        if let Some(current_middle) = pose.local_transform(middle_bone_idx) {
            let middle_trans = current_middle.col(3).truncate();
            let middle_scale = Vec3::new(
                current_middle.col(0).truncate().length(),
                current_middle.col(1).truncate().length(),
                current_middle.col(2).truncate().length(),
            );

            let current_middle_rot = Quat::from_mat4(&current_middle);
            let blended_rot = current_middle_rot.slerp(middle_rot, constraint.weight);
            pose.set_local_transform(
                middle_bone_idx,
                Mat4::from_scale_rotation_translation(middle_scale, blended_rot, middle_trans),
            );
        }

        pose.update_world_transforms(skeleton);
    }

    /// Solves a chain IK using FABRIK (Forward And Backward Reaching Inverse Kinematics).
    pub fn solve_chain(constraint: &IkConstraint, pose: &mut AnimatedPose, skeleton: &Skeleton) {
        let end_bone_idx = constraint.end_effector_bone;

        let mut chain = Vec::new();
        let mut current_idx = end_bone_idx;

        while let Some(bone) = skeleton.bone(current_idx) {
            chain.push(current_idx);
            if let Some(parent) = bone.parent_index {
                current_idx = parent;
            } else {
                break;
            }
        }
        chain.reverse();

        if chain.len() < 2 {
            return;
        }

        let mut positions: Vec<Vec3> = chain
            .iter()
            .map(|&idx| {
                pose.world_transform(idx)
                    .map_or(Vec3::ZERO, |m| m.col(3).truncate())
            })
            .collect();

        let bone_lengths: Vec<f32> = (0..chain.len() - 1)
            .map(|i| (positions[i + 1] - positions[i]).length())
            .collect();

        let root_pos = positions[0];
        let target = constraint.target_position;

        for _ in 0..constraint.max_iterations {
            positions[chain.len() - 1] = target;

            for i in (1..chain.len()).rev() {
                let dir = (positions[i - 1] - positions[i]).normalize_or_zero();
                positions[i - 1] = positions[i] + dir * bone_lengths[i - 1];
            }

            positions[0] = root_pos;

            for i in 0..chain.len() - 1 {
                let dir = (positions[i + 1] - positions[i]).normalize_or_zero();
                positions[i + 1] = positions[i] + dir * bone_lengths[i];
            }

            let end_dist = (positions[chain.len() - 1] - target).length();
            if end_dist < constraint.tolerance {
                break;
            }
        }

        for i in 0..chain.len() - 1 {
            let bone_idx = chain[i];
            let next_idx = chain[i + 1];

            let original_dir = pose
                .world_transform(next_idx)
                .and_then(|next| {
                    pose.world_transform(bone_idx).map(|current| {
                        (next.col(3).truncate() - current.col(3).truncate()).normalize_or_zero()
                    })
                })
                .unwrap_or(Vec3::X);

            let new_dir = (positions[i + 1] - positions[i]).normalize_or_zero();

            // Skip if directions are invalid (zero length)
            if new_dir.length_squared() < 0.001 || original_dir.length_squared() < 0.001 {
                continue;
            }

            let rotation = Quat::from_rotation_arc(original_dir, new_dir);

            if let Some(current) = pose.local_transform(bone_idx) {
                let trans = current.col(3).truncate();
                let scale = Vec3::new(
                    current.col(0).truncate().length(),
                    current.col(1).truncate().length(),
                    current.col(2).truncate().length(),
                );
                let current_rot = Quat::from_mat4(&current);

                let blended_rot = current_rot.slerp(rotation * current_rot, constraint.weight);
                pose.set_local_transform(
                    bone_idx,
                    Mat4::from_scale_rotation_translation(scale, blended_rot, trans),
                );
            }
        }

        pose.update_world_transforms(skeleton);
    }

    /// Solves look-at IK to orient a bone toward a target.
    pub fn solve_look_at(constraint: &IkConstraint, pose: &mut AnimatedPose, skeleton: &Skeleton) {
        let bone_idx = constraint.end_effector_bone;

        let bone_pos = pose
            .world_transform(bone_idx)
            .map_or(Vec3::ZERO, |m| m.col(3).truncate());

        let target = constraint.target_position;
        let direction = (target - bone_pos).normalize_or_zero();

        // Skip if direction is invalid (zero length)
        if direction.length_squared() < 0.001 {
            return;
        }

        let rotation = Quat::from_rotation_arc(Vec3::Z, direction);

        if let Some(current) = pose.local_transform(bone_idx) {
            let trans = current.col(3).truncate();
            let scale = Vec3::new(
                current.col(0).truncate().length(),
                current.col(1).truncate().length(),
                current.col(2).truncate().length(),
            );
            let current_rot = Quat::from_mat4(&current);

            let blended_rot = current_rot.slerp(rotation, constraint.weight);
            pose.set_local_transform(
                bone_idx,
                Mat4::from_scale_rotation_translation(scale, blended_rot, trans),
            );
        }

        pose.update_world_transforms(skeleton);
    }

    /// Applies an IK constraint to a pose.
    pub fn apply_constraint(
        constraint: &IkConstraint,
        pose: &mut AnimatedPose,
        skeleton: &Skeleton,
    ) {
        match constraint.constraint_type {
            IkConstraintType::TwoBone => Self::solve_two_bone(constraint, pose, skeleton),
            IkConstraintType::Chain => Self::solve_chain(constraint, pose, skeleton),
            IkConstraintType::LookAt => Self::solve_look_at(constraint, pose, skeleton),
        }
    }
}

/// Component for managing IK constraints on an entity.
#[derive(Component, Debug, Clone)]
pub struct IkController {
    /// Active IK constraints.
    constraints: Vec<IkConstraint>,
}

impl IkController {
    /// Creates a new IK controller.
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Adds an IK constraint.
    pub fn add_constraint(&mut self, constraint: IkConstraint) {
        self.constraints.push(constraint);
    }

    /// Removes all constraints.
    pub fn clear_constraints(&mut self) {
        self.constraints.clear();
    }

    /// Gets all constraints.
    pub fn constraints(&self) -> &[IkConstraint] {
        &self.constraints
    }

    /// Gets a mutable reference to a constraint.
    pub fn constraint_mut(&mut self, index: usize) -> Option<&mut IkConstraint> {
        self.constraints.get_mut(index)
    }

    /// Applies all IK constraints to a pose.
    pub fn apply(&self, pose: &mut AnimatedPose, skeleton: &Skeleton) {
        for constraint in &self.constraints {
            if constraint.weight > 0.001 {
                IkSolver::apply_constraint(constraint, pose, skeleton);
            }
        }
    }
}

impl Default for IkController {
    fn default() -> Self {
        Self::new()
    }
}

/// System to apply IK constraints after animation evaluation.
pub fn apply_ik_constraints(query: &mut Query<(&Skeleton, &IkController, &mut AnimatedPose)>) {
    for (skeleton, ik_controller, mut pose) in query.iter_mut() {
        ik_controller.apply(&mut pose, skeleton);
        pose.update_skinning_matrices(skeleton);
    }
}

// ============================================================================
// Animation Retargeting
// ============================================================================

/// Bone mapping for animation retargeting.
///
/// Maps bone indices from a source skeleton to a target skeleton.
#[derive(Debug, Clone)]
pub struct BoneMapping {
    /// Maps source bone index to target bone index.
    source_to_target: HashMap<usize, usize>,

    /// Maps bone names from source to target.
    name_mapping: HashMap<String, String>,
}

impl BoneMapping {
    /// Creates a new empty bone mapping.
    pub fn new() -> Self {
        Self {
            source_to_target: HashMap::new(),
            name_mapping: HashMap::new(),
        }
    }

    /// Adds a bone mapping by index.
    pub fn map_bones(&mut self, source_idx: usize, target_idx: usize) {
        self.source_to_target.insert(source_idx, target_idx);
    }

    /// Adds a bone mapping by name.
    pub fn map_bone_names(&mut self, source_name: String, target_name: String) {
        self.name_mapping.insert(source_name, target_name);
    }

    /// Gets the target bone index for a source bone.
    pub fn get_target_bone(&self, source_idx: usize) -> Option<usize> {
        self.source_to_target.get(&source_idx).copied()
    }

    /// Creates an automatic bone mapping based on bone names.
    ///
    /// Attempts to match bones with identical or similar names between skeletons.
    pub fn auto_map(source_skeleton: &Skeleton, target_skeleton: &Skeleton) -> Self {
        let mut mapping = Self::new();

        for source_idx in 0..source_skeleton.bone_count() {
            if let Some(source_bone) = source_skeleton.bone(source_idx) {
                let source_name = &source_bone.name;

                if let Some(target_idx) = target_skeleton.find_bone(source_name) {
                    mapping.map_bones(source_idx, target_idx);
                    continue;
                }

                let source_lower = source_name.to_lowercase();
                for target_idx in 0..target_skeleton.bone_count() {
                    if let Some(target_bone) = target_skeleton.bone(target_idx) {
                        let target_lower = target_bone.name.to_lowercase();
                        if source_lower == target_lower
                            || source_lower.contains(&target_lower)
                            || target_lower.contains(&source_lower)
                        {
                            mapping.map_bones(source_idx, target_idx);
                            break;
                        }
                    }
                }
            }
        }

        mapping
    }
}

impl Default for BoneMapping {
    fn default() -> Self {
        Self::new()
    }
}

/// Animation retargeter for applying animations to different skeletons.
pub struct AnimationRetargeter {
    /// Bone mapping from source to target skeleton.
    bone_mapping: BoneMapping,
}

impl AnimationRetargeter {
    /// Creates a new retargeter with the given bone mapping.
    pub fn new(bone_mapping: BoneMapping) -> Self {
        Self { bone_mapping }
    }

    /// Creates a retargeter with automatic bone mapping.
    pub fn auto(source_skeleton: &Skeleton, target_skeleton: &Skeleton) -> Self {
        Self {
            bone_mapping: BoneMapping::auto_map(source_skeleton, target_skeleton),
        }
    }

    /// Retargets an animation clip from source to target skeleton.
    pub fn retarget_clip(
        &self,
        source_clip: &AnimationClip,
        _target_skeleton: &Skeleton,
    ) -> AnimationClip {
        let mut target_clip = AnimationClip::new(
            format!("{}_retargeted", source_clip.name()),
            source_clip.duration(),
        );

        for (source_bone_idx, source_track) in source_clip.bone_tracks() {
            if let Some(target_bone_idx) = self.bone_mapping.get_target_bone(*source_bone_idx) {
                let mut target_track = BoneTrack::new();

                for keyframe in &source_track.translation_keyframes {
                    target_track.add_translation_keyframe(keyframe.time, keyframe.value);
                }

                for keyframe in &source_track.rotation_keyframes {
                    target_track.add_rotation_keyframe(keyframe.time, keyframe.value);
                }

                for keyframe in &source_track.scale_keyframes {
                    target_track.add_scale_keyframe(keyframe.time, keyframe.value);
                }

                if target_track.has_keyframes() {
                    target_clip
                        .bone_tracks
                        .insert(target_bone_idx, target_track);
                }
            }
        }

        target_clip
    }

    /// Retargets a pose from source to target skeleton.
    pub fn retarget_pose(
        &self,
        source_pose: &AnimatedPose,
        target_skeleton: &Skeleton,
    ) -> AnimatedPose {
        let mut target_pose = AnimatedPose::new(target_skeleton.bone_count());

        for i in 0..target_skeleton.bone_count() {
            if let Some(bone) = target_skeleton.bone(i) {
                target_pose.set_local_transform(i, bone.bind_pose_matrix());
            }
        }

        for (source_idx, target_idx) in &self.bone_mapping.source_to_target {
            if let Some(transform) = source_pose.local_transform(*source_idx) {
                target_pose.set_local_transform(*target_idx, transform);
            }
        }

        target_pose.update_world_transforms(target_skeleton);
        target_pose.update_skinning_matrices(target_skeleton);

        target_pose
    }

    /// Gets the bone mapping.
    pub fn bone_mapping(&self) -> &BoneMapping {
        &self.bone_mapping
    }
}

// ============================================================================
// Enhanced Additive Animation Blending
// ============================================================================

/// Additive animation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdditiveMode {
    /// Local space additive (adds to local transforms).
    Local,

    /// World space additive (adds to world transforms).
    World,
}

/// Enhanced additive animation blending with reference pose.
#[derive(Debug, Clone)]
pub struct AdditiveAnimation {
    /// Name of the base animation clip.
    #[allow(dead_code)]
    base_clip_name: String,

    /// Name of the additive animation clip.
    #[allow(dead_code)]
    additive_clip_name: String,

    /// Reference pose for computing deltas (usually bind pose).
    reference_pose: Option<AnimatedPose>,

    /// Weight of the additive animation.
    weight: f32,

    /// Additive mode (local or world space).
    mode: AdditiveMode,
}

impl AdditiveAnimation {
    /// Creates a new additive animation.
    pub fn new(base_clip: String, additive_clip: String) -> Self {
        Self {
            base_clip_name: base_clip,
            additive_clip_name: additive_clip,
            reference_pose: None,
            weight: 1.0,
            mode: AdditiveMode::Local,
        }
    }

    /// Sets the reference pose.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with the reference pose set.
    /// Ignoring the return value will discard the reference pose and the additive animation will not work correctly.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_reference_pose(mut self, reference: AnimatedPose) -> Self {
        self.reference_pose = Some(reference);
        self
    }

    /// Sets the weight.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with the weight set.
    /// Ignoring the return value will discard the weight configuration and the additive animation will use the default weight.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Sets the additive mode.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with the additive mode set.
    /// Ignoring the return value will discard the mode configuration and the additive animation will use the default mode.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_mode(mut self, mode: AdditiveMode) -> Self {
        self.mode = mode;
        self
    }

    /// Computes the reference pose from a skeleton (bind pose).
    pub fn compute_reference_from_skeleton(&mut self, skeleton: &Skeleton) {
        let mut reference = AnimatedPose::new(skeleton.bone_count());
        for i in 0..skeleton.bone_count() {
            if let Some(bone) = skeleton.bone(i) {
                reference.set_local_transform(i, bone.bind_pose_matrix());
            }
        }
        reference.update_world_transforms(skeleton);
        self.reference_pose = Some(reference);
    }

    /// Applies additive blending to a pose.
    pub fn apply(
        &self,
        base_pose: &mut AnimatedPose,
        additive_clip: &AnimationClip,
        additive_time: f32,
        skeleton: &Skeleton,
    ) {
        let Some(reference) = &self.reference_pose else {
            return;
        };

        for (bone_idx, track) in additive_clip.bone_tracks() {
            if let Some(bone) = skeleton.bone(*bone_idx) {
                let additive_trans = track
                    .sample_translation(additive_time)
                    .unwrap_or(bone.bind_pose_translation);
                let additive_rot = track
                    .sample_rotation(additive_time)
                    .unwrap_or(bone.bind_pose_rotation);
                let additive_scale = track
                    .sample_scale(additive_time)
                    .unwrap_or(bone.bind_pose_scale);

                let ref_trans = reference
                    .local_transform(*bone_idx)
                    .map_or(bone.bind_pose_translation, |m| m.col(3).truncate());
                let ref_rot = reference
                    .local_transform(*bone_idx)
                    .map_or(bone.bind_pose_rotation, |m| Quat::from_mat4(&m));
                let ref_scale =
                    reference
                        .local_transform(*bone_idx)
                        .map_or(bone.bind_pose_scale, |m| {
                            Vec3::new(
                                m.col(0).truncate().length(),
                                m.col(1).truncate().length(),
                                m.col(2).truncate().length(),
                            )
                        });

                let delta_trans = additive_trans - ref_trans;
                let delta_rot = ref_rot.inverse() * additive_rot;

                // Safely compute delta scale, avoiding division by zero
                let delta_scale = Vec3::new(
                    if ref_scale.x.abs() > 0.001 {
                        additive_scale.x / ref_scale.x
                    } else {
                        1.0
                    },
                    if ref_scale.y.abs() > 0.001 {
                        additive_scale.y / ref_scale.y
                    } else {
                        1.0
                    },
                    if ref_scale.z.abs() > 0.001 {
                        additive_scale.z / ref_scale.z
                    } else {
                        1.0
                    },
                );

                if let Some(current) = base_pose.local_transform(*bone_idx) {
                    let current_trans = current.col(3).truncate();
                    let current_rot = Quat::from_mat4(&current);
                    let current_scale = Vec3::new(
                        current.col(0).truncate().length(),
                        current.col(1).truncate().length(),
                        current.col(2).truncate().length(),
                    );

                    let final_trans = current_trans + delta_trans * self.weight;
                    let final_rot = current_rot * Quat::IDENTITY.slerp(delta_rot, self.weight);
                    let final_scale = current_scale * Vec3::ONE.lerp(delta_scale, self.weight);

                    base_pose.set_local_transform(
                        *bone_idx,
                        Mat4::from_scale_rotation_translation(final_scale, final_rot, final_trans),
                    );
                }
            }
        }

        base_pose.update_world_transforms(skeleton);
    }
}

// ============================================================================
// Root Motion Extraction
// ============================================================================

/// Root motion data extracted from an animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RootMotion {
    /// Translation delta.
    pub translation: Vec3,

    /// Rotation delta.
    pub rotation: Quat,

    /// Whether this root motion has been consumed.
    pub consumed: bool,
}

impl RootMotion {
    /// Creates a new root motion.
    pub fn new(translation: Vec3, rotation: Quat) -> Self {
        Self {
            translation,
            rotation,
            consumed: false,
        }
    }

    /// Creates an identity root motion (no movement).
    pub fn identity() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            consumed: false,
        }
    }

    /// Marks this root motion as consumed.
    pub fn consume(&mut self) {
        self.consumed = true;
    }

    /// Resets the consumed flag.
    pub fn reset(&mut self) {
        self.consumed = false;
    }
}

impl Default for RootMotion {
    fn default() -> Self {
        Self::identity()
    }
}

/// Component for extracting and applying root motion from animations.
#[derive(Component, Debug, Clone)]
pub struct RootMotionExtractor {
    /// Index of the root bone to extract motion from.
    root_bone_index: usize,

    /// Whether to extract translation.
    extract_translation: bool,

    /// Whether to extract rotation.
    extract_rotation: bool,

    /// Previous root bone position (for computing deltas).
    previous_position: Vec3,

    /// Previous root bone rotation (for computing deltas).
    previous_rotation: Quat,

    /// Extracted root motion for this frame.
    current_motion: RootMotion,

    /// Whether to apply motion to the entity's transform.
    apply_to_transform: bool,
}

impl RootMotionExtractor {
    /// Creates a new root motion extractor.
    pub fn new(root_bone_index: usize) -> Self {
        Self {
            root_bone_index,
            extract_translation: true,
            extract_rotation: true,
            previous_position: Vec3::ZERO,
            previous_rotation: Quat::IDENTITY,
            current_motion: RootMotion::identity(),
            apply_to_transform: true,
        }
    }

    /// Enables or disables translation extraction.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with translation extraction configured.
    /// Ignoring the return value will discard the configuration and the extractor will use the default setting.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_translation(mut self, enabled: bool) -> Self {
        self.extract_translation = enabled;
        self
    }

    /// Enables or disables rotation extraction.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with rotation extraction configured.
    /// Ignoring the return value will discard the configuration and the extractor will use the default setting.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_rotation(mut self, enabled: bool) -> Self {
        self.extract_rotation = enabled;
        self
    }

    /// Enables or disables automatic transform application.
    ///
    /// # Must Use
    ///
    /// This builder method consumes `self` and returns a new instance with auto-apply configured.
    /// Ignoring the return value will discard the configuration and the extractor will use the default setting.
    #[must_use = "builder methods return a new value and do not modify the original"]
    pub fn with_auto_apply(mut self, enabled: bool) -> Self {
        self.apply_to_transform = enabled;
        self
    }

    /// Extracts root motion from a pose.
    pub fn extract(&mut self, pose: &mut AnimatedPose, skeleton: &Skeleton) {
        let Some(root_transform) = pose.local_transform(self.root_bone_index) else {
            self.current_motion = RootMotion::identity();
            return;
        };

        let position = root_transform.col(3).truncate();
        let rotation = Quat::from_mat4(&root_transform);

        let translation_delta = if self.extract_translation {
            position - self.previous_position
        } else {
            Vec3::ZERO
        };

        let rotation_delta = if self.extract_rotation {
            self.previous_rotation.inverse() * rotation
        } else {
            Quat::IDENTITY
        };

        self.current_motion = RootMotion::new(translation_delta, rotation_delta);

        self.previous_position = position;
        self.previous_rotation = rotation;

        if self.extract_translation {
            let zero_translation =
                Mat4::from_scale_rotation_translation(Vec3::ONE, rotation, Vec3::ZERO);
            pose.set_local_transform(self.root_bone_index, zero_translation);
        }

        if self.extract_rotation {
            let zero_rotation = Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                Quat::IDENTITY,
                if self.extract_translation {
                    Vec3::ZERO
                } else {
                    position
                },
            );
            pose.set_local_transform(self.root_bone_index, zero_rotation);
        }

        pose.update_world_transforms(skeleton);
    }

    /// Gets the current root motion.
    pub fn motion(&self) -> &RootMotion {
        &self.current_motion
    }

    /// Gets a mutable reference to the current root motion.
    pub fn motion_mut(&mut self) -> &mut RootMotion {
        &mut self.current_motion
    }

    /// Resets the extractor state.
    pub fn reset(&mut self) {
        self.previous_position = Vec3::ZERO;
        self.previous_rotation = Quat::IDENTITY;
        self.current_motion = RootMotion::identity();
    }
}

impl Default for RootMotionExtractor {
    fn default() -> Self {
        Self::new(0)
    }
}
