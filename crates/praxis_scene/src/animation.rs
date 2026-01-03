//! Skeletal animation system for the Praxis engine.
//!
//! This module provides components and systems for skeletal animation, including:
//! - Skeleton: Defines bone hierarchy and bind poses
//! - AnimationClip: Stores keyframe data for animation sequences
//! - AnimationPlayer: Controls animation playback on entities
//!
//! # Overview
//!
//! Skeletal animation works by defining a hierarchy of bones (joints) and animating
//! their transforms over time using keyframe interpolation. Each bone has a bind pose
//! (rest position) and can be animated independently. The bone transforms are then
//! applied to skinned meshes to deform vertices.
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

            world_transforms[i] = if let Some(parent_idx) = bone.parent_index {
                world_transforms[parent_idx] * local_transform
            } else {
                local_transform
            };
        }

        // Invert to get bone space from world space
        world_transforms.iter().map(|m| m.inverse()).collect()
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
        self.translation_keyframes.push(Keyframe::new(time, translation));
        // Keep keyframes sorted by time
        self.translation_keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Adds a rotation keyframe.
    pub fn add_rotation_keyframe(&mut self, time: f32, rotation: Quat) {
        self.rotation_keyframes.push(Keyframe::new(time, rotation));
        self.rotation_keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Adds a scale keyframe.
    pub fn add_scale_keyframe(&mut self, time: f32, scale: Vec3) {
        self.scale_keyframes.push(Keyframe::new(time, scale));
        self.scale_keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
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
            (Some(k), _) | (_, Some(k)) => {
                Some(k.value)
            }
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
            (Some(k), _) | (_, Some(k)) => {
                Some(k.value)
            }
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
        self.bone_tracks.entry(bone_index).or_insert_with(BoneTrack::new)
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
        self.add_bone_track(bone_index).add_translation_keyframe(time, translation);
    }

    /// Adds a rotation keyframe to a bone track.
    pub fn add_rotation_keyframe(&mut self, bone_index: usize, time: f32, rotation: Quat) {
        self.add_bone_track(bone_index).add_rotation_keyframe(time, rotation);
    }

    /// Adds a scale keyframe to a bone track.
    pub fn add_scale_keyframe(&mut self, bone_index: usize, time: f32, scale: Vec3) {
        self.add_bone_track(bone_index).add_scale_keyframe(time, scale);
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
            .map_or(false, |p| p.state == PlaybackState::Playing)
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
                self.apply_clip_to_pose(clip, playing.time, playing.weight, &mut pose, skeleton);
            }
        }

        // Update world transforms and skinning matrices
        pose.update_world_transforms(skeleton);
        pose.update_skinning_matrices(skeleton);

        pose
    }

    /// Applies a single animation clip to a pose with blending.
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
                // Sample the track at the current time
                let translation = track
                    .sample_translation(time)
                    .unwrap_or(bone.bind_pose_translation);
                let rotation = track
                    .sample_rotation(time)
                    .unwrap_or(bone.bind_pose_rotation);
                let scale = track
                    .sample_scale(time)
                    .unwrap_or(bone.bind_pose_scale);

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
        assert_eq!(clip.duration(), 2.0);

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
        assert_eq!(player.current_time("Test"), Some(0.0));

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
}
