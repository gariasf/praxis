//! Editor camera controller with orbit controls.
//!
//! This module implements an **Orbit Camera Controller** specifically designed for the editor,
//! providing intuitive 3D scene navigation separate from game cameras. The controller follows
//! common 3D modeling software conventions (Maya, Blender, etc.).
//!
//! # Orbit Camera Pattern
//!
//! The orbit camera maintains these key properties:
//! - **Target Point**: The 3D point the camera orbits around (default: origin)
//! - **Distance**: How far the camera is from the target
//! - **Yaw/Pitch**: Spherical coordinates defining camera orientation around target
//!
//! ## Spherical Coordinate System
//!
//! The camera position is computed using spherical coordinates:
//! ```text
//! position = target + rotation * Vec3(0, 0, distance)
//! where rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch)
//! ```
//!
//! This ensures the camera always faces the target while allowing free rotation.
//!
//! # Input Controls
//!
//! ## Orbit (Alt+LMB)
//! - **Action**: Rotate camera around target point
//! - **Implementation**: Modify yaw and pitch angles based on mouse delta
//! - **Constraints**: Pitch clamped to prevent camera flipping (±89 degrees)
//! - **Use case**: Inspect objects from all angles
//!
//! ## Pan (Alt+MMB)
//! - **Action**: Move camera and target together in screen space
//! - **Implementation**:
//!   1. Calculate camera right and up vectors from rotation
//!   2. Offset target by (right * mouse_x + up * mouse_y)
//!   3. Scale offset by distance (farther = faster pan)
//! - **Use case**: Center different objects in view
//!
//! ## Zoom (Scroll Wheel)
//! - **Action**: Move camera closer or farther from target
//! - **Implementation**: Adjust distance along view direction
//! - **Constraints**: Clamped between min_distance and max_distance
//! - **Use case**: Get close-up or overview of scene
//!
//! ## Focus (F Key)
//! - **Action**: Frame selected entities in view
//! - **Implementation**:
//!   1. Compute bounding box of all selected entities
//!   2. Calculate center point
//!   3. Estimate viewing distance from bounding box size
//!   4. Smoothly interpolate to new target/distance
//! - **Use case**: Quickly navigate to selection
//!
//! # Smooth Interpolation
//!
//! All camera movements use **exponential smoothing** (ease-out):
//! ```text
//! current_value = current_value + (desired_value - current_value) * smoothness * dt
//! ```
//!
//! Benefits:
//! - **Natural feel**: Gradual acceleration and deceleration
//! - **Responsive**: Quickly approaches target (not linear interpolation)
//! - **Frame-rate independent**: Uses delta time for consistent behavior
//! - **Tunable**: `smoothness` parameter controls responsiveness
//!
//! ## Interpolated Properties
//! - Target position (pan, focus)
//! - Distance (zoom, focus)
//! - Yaw angle (orbit)
//! - Pitch angle (orbit)
//!
//! # Separation from Game Cameras
//!
//! The editor camera is distinguished from game cameras using:
//! - **`EditorCamera` marker component**: Only this camera is controlled
//! - **Independent resource**: `EditorCameraController` doesn't affect game camera systems
//! - **Separate schedule**: Updated in editor schedule, not game schedule
//!
//! This allows:
//! - Game cameras to exist and be configured without interference
//! - Editor camera to persist across play mode transitions
//! - Preview of game camera views while editing
//!
//! # Architecture
//!
//! The editor camera is managed through:
//! - **`EditorCameraController`**: Resource managing camera state and movement
//! - **`EditorCamera`**: Marker component identifying the editor camera entity
//! - **`update_editor_camera_system`**: ECS system that applies camera updates each frame
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_editor::{EditorCameraController, update_editor_camera_system};
//! use praxis_ecs::{World, Schedule};
//!
//! let mut world = World::new();
//! world.insert_resource(EditorCameraController::new());
//!
//! let mut schedule = Schedule::default();
//! schedule.add_systems(update_editor_camera_system);
//! ```

use bevy_ecs::component::Component;
use bevy_ecs::system::{Query, Res, ResMut, Resource};
use praxis_ecs::{Camera, GlobalTransform, Transform};
use praxis_input::{InputState, MouseButton};
use praxis_math::{Quat, Vec3};
use winit::keyboard::KeyCode;

use crate::selection::SelectionSystem;

/// Marker component for the editor camera entity.
///
/// Only the camera with this component will be controlled by the editor camera controller.
/// This allows game cameras to exist separately without interference.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct EditorCamera;

/// Editor camera controller resource managing orbit camera behavior.
///
/// This controller provides an orbit camera specifically for the editor viewport,
/// with smooth interpolation and focus capabilities.
///
/// # Features
///
/// - **Orbit rotation**: Alt+LMB drag to rotate around target
/// - **Pan movement**: Alt+MMB drag to pan camera
/// - **Zoom**: Mouse scroll wheel to move closer/farther
/// - **Focus on selection**: F key to frame selected entities
/// - **Smooth interpolation**: Smooth camera movement to target state
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::EditorCameraController;
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// let mut controller = EditorCameraController::new();
///
/// // Set focus target
/// controller.set_target(praxis_math::Vec3::new(0.0, 0.0, 0.0));
/// ```
#[derive(Resource, Debug, Clone)]
pub struct EditorCameraController {
    /// Target position the camera is orbiting around
    target: Vec3,
    /// Distance from target
    distance: f32,
    /// Horizontal angle (yaw) in radians
    yaw: f32,
    /// Vertical angle (pitch) in radians
    pitch: f32,
    /// Mouse sensitivity for orbit rotation
    orbit_sensitivity: f32,
    /// Mouse sensitivity for pan movement
    pan_sensitivity: f32,
    /// Scroll sensitivity for zoom
    zoom_sensitivity: f32,
    /// Minimum distance from target
    min_distance: f32,
    /// Maximum distance from target
    max_distance: f32,
    /// Minimum pitch angle in radians
    min_pitch: f32,
    /// Maximum pitch angle in radians
    max_pitch: f32,
    /// Interpolation speed (0 = instant, higher = smoother)
    smoothness: f32,
    /// Desired target position for smooth transitions
    desired_target: Vec3,
    /// Desired distance for smooth transitions
    desired_distance: f32,
    /// Desired yaw for smooth transitions
    desired_yaw: f32,
    /// Desired pitch for smooth transitions
    desired_pitch: f32,
    /// Whether the camera is currently in an operation
    is_orbiting: bool,
    is_panning: bool,
}

impl Default for EditorCameraController {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCameraController {
    /// Creates a new editor camera controller with default settings.
    pub fn new() -> Self {
        let initial_distance = 10.0;
        let initial_yaw = std::f32::consts::PI * 0.25; // 45 degrees
        let initial_pitch = std::f32::consts::PI * 0.25; // 45 degrees

        Self {
            target: Vec3::ZERO,
            distance: initial_distance,
            yaw: initial_yaw,
            pitch: initial_pitch,
            orbit_sensitivity: 0.005,
            pan_sensitivity: 0.01,
            zoom_sensitivity: 1.0,
            min_distance: 0.5,
            max_distance: 1000.0,
            min_pitch: -std::f32::consts::FRAC_PI_2 + 0.01,
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.01,
            smoothness: 10.0,
            desired_target: Vec3::ZERO,
            desired_distance: initial_distance,
            desired_yaw: initial_yaw,
            desired_pitch: initial_pitch,
            is_orbiting: false,
            is_panning: false,
        }
    }

    /// Sets the target position to orbit around.
    pub fn set_target(&mut self, target: Vec3) {
        self.desired_target = target;
    }

    /// Gets the current target position.
    pub fn target(&self) -> Vec3 {
        self.target
    }

    /// Sets the distance from the target.
    pub fn set_distance(&mut self, distance: f32) {
        self.desired_distance = distance.clamp(self.min_distance, self.max_distance);
    }

    /// Gets the current distance from the target.
    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// Sets the orbit angles directly.
    pub fn set_angles(&mut self, yaw: f32, pitch: f32) {
        self.desired_yaw = yaw;
        self.desired_pitch = pitch.clamp(self.min_pitch, self.max_pitch);
    }

    /// Gets the current orbit angles (yaw, pitch).
    pub fn angles(&self) -> (f32, f32) {
        (self.yaw, self.pitch)
    }

    /// Processes input and updates desired camera state.
    pub fn process_input(&mut self, input: &InputState, _delta_time: f32) {
        let alt_pressed =
            input.is_key_pressed(KeyCode::AltLeft) || input.is_key_pressed(KeyCode::AltRight);

        // Orbit: Alt+LMB
        let is_orbiting = alt_pressed && input.is_mouse_button_pressed(MouseButton::Left);
        if is_orbiting {
            let delta = input.mouse_delta();
            self.desired_yaw -= delta.0 as f32 * self.orbit_sensitivity;
            self.desired_pitch -= delta.1 as f32 * self.orbit_sensitivity;
            self.desired_pitch = self.desired_pitch.clamp(self.min_pitch, self.max_pitch);
            self.is_orbiting = true;
        } else if self.is_orbiting {
            self.is_orbiting = false;
        }

        // Pan: Alt+MMB
        let is_panning = alt_pressed && input.is_mouse_button_pressed(MouseButton::Middle);
        if is_panning {
            let delta = input.mouse_delta();

            // Calculate camera right and up vectors
            let rotation = self.compute_rotation();
            let right = rotation * Vec3::X;
            let up = rotation * Vec3::Y;

            // Pan in camera space
            let pan_offset = right * (-delta.0 as f32 * self.pan_sensitivity * self.distance * 0.1)
                + up * (delta.1 as f32 * self.pan_sensitivity * self.distance * 0.1);

            self.desired_target += pan_offset;
            self.is_panning = true;
        } else if self.is_panning {
            self.is_panning = false;
        }

        // Zoom: Scroll wheel
        let scroll = input.scroll_delta();
        if scroll.1.abs() > 0.001 {
            let zoom_amount = scroll.1 * self.zoom_sensitivity;
            self.desired_distance =
                (self.desired_distance - zoom_amount).clamp(self.min_distance, self.max_distance);
        }
    }

    /// Focuses the camera on the given position with optional distance.
    pub fn focus_on(&mut self, position: Vec3, distance: Option<f32>) {
        self.desired_target = position;
        if let Some(dist) = distance {
            self.desired_distance = dist.clamp(self.min_distance, self.max_distance);
        }
    }

    /// Focuses the camera on the current selection.
    ///
    /// This computes the bounding box of all selected entities and frames them in view.
    pub fn focus_on_selection(
        &mut self,
        selection: &SelectionSystem,
        transform_query: &Query<&GlobalTransform>,
    ) {
        if selection.is_empty() {
            return;
        }

        // Compute bounding box of selected entities
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        let mut count = 0;

        for entity in selection.selected_entities() {
            if let Ok(transform) = transform_query.get(entity) {
                let pos = transform.translation();
                min = min.min(pos);
                max = max.max(pos);
                count += 1;
            }
        }

        if count == 0 {
            return;
        }

        // Calculate center and size
        let center = (min + max) * 0.5;
        let size = (max - min).length();

        // Focus on center with distance based on size
        let distance = if size > 0.1 { size * 2.0 } else { 5.0 };

        self.focus_on(center, Some(distance));
    }

    /// Updates the camera state with smooth interpolation.
    pub fn update(&mut self, delta_time: f32) {
        let t = (self.smoothness * delta_time).min(1.0);

        // Interpolate target position
        self.target = self.target.lerp(self.desired_target, t);

        // Interpolate distance
        self.distance = self.distance + (self.desired_distance - self.distance) * t;

        // Interpolate angles
        self.yaw = self.yaw + (self.desired_yaw - self.yaw) * t;
        self.pitch = self.pitch + (self.desired_pitch - self.pitch) * t;
    }

    /// Computes the camera rotation quaternion from current angles.
    fn compute_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
    }

    /// Computes the camera position from current state.
    pub fn compute_position(&self) -> Vec3 {
        let rotation = self.compute_rotation();
        let offset = rotation * Vec3::new(0.0, 0.0, self.distance);
        self.target + offset
    }

    /// Computes the camera transform from current state.
    pub fn compute_transform(&self) -> Transform {
        let position = self.compute_position();
        let rotation = self.compute_rotation();

        Transform {
            translation: position,
            rotation,
            scale: Vec3::ONE,
        }
    }
}

/// System that updates the editor camera based on input and interpolation.
///
/// This system should be added to the editor schedule and will automatically
/// update any camera with the EditorCamera component.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::update_editor_camera_system;
/// use praxis_ecs::Schedule;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(update_editor_camera_system);
/// ```
pub fn update_editor_camera_system(
    mut controller: ResMut<EditorCameraController>,
    input: Res<InputState>,
    selection: Res<SelectionSystem>,
    mut camera_query: Query<(&Camera, &mut Transform), praxis_ecs::With<EditorCamera>>,
    transform_query: Query<&GlobalTransform>,
) {
    // Calculate delta time (assume 60fps for now, should be passed from main loop)
    let delta_time = 1.0 / 60.0;

    // Process input
    controller.process_input(&input, delta_time);

    // Handle focus on selection (F key)
    if input.is_key_just_pressed(KeyCode::KeyF) && !selection.is_empty() {
        controller.focus_on_selection(&selection, &transform_query);
    }

    // Update interpolation
    controller.update(delta_time);

    // Apply to editor camera
    for (camera, mut transform) in camera_query.iter_mut() {
        if camera.is_active {
            *transform = controller.compute_transform();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_camera_controller_creation() {
        let controller = EditorCameraController::new();
        assert_eq!(controller.target(), Vec3::ZERO);
        assert_eq!(controller.distance(), 10.0);
    }

    #[test]
    fn test_set_target() {
        let mut controller = EditorCameraController::new();
        let target = Vec3::new(5.0, 2.0, 3.0);
        controller.set_target(target);

        // Target should be set as desired, actual target interpolates
        controller.update(1.0); // Large delta to complete interpolation
        assert_eq!(controller.target(), target);
    }

    #[test]
    fn test_set_distance() {
        let mut controller = EditorCameraController::new();
        controller.set_distance(15.0);

        controller.update(1.0);
        assert_eq!(controller.distance(), 15.0);
    }

    #[test]
    fn test_distance_clamping() {
        let mut controller = EditorCameraController::new();

        // Test minimum clamping
        controller.set_distance(0.1);
        controller.update(1.0);
        assert!(controller.distance() >= controller.min_distance);

        // Test maximum clamping
        controller.set_distance(10000.0);
        controller.update(1.0);
        assert!(controller.distance() <= controller.max_distance);
    }

    #[test]
    fn test_set_angles() {
        let mut controller = EditorCameraController::new();
        let yaw = std::f32::consts::PI;
        let pitch = std::f32::consts::FRAC_PI_4;

        controller.set_angles(yaw, pitch);
        controller.update(1.0);

        let (actual_yaw, actual_pitch) = controller.angles();
        assert!((actual_yaw - yaw).abs() < 0.001);
        assert!((actual_pitch - pitch).abs() < 0.001);
    }

    #[test]
    fn test_pitch_clamping() {
        let mut controller = EditorCameraController::new();

        // Test that pitch is clamped
        controller.set_angles(0.0, std::f32::consts::PI); // Too high
        controller.update(1.0);

        let (_, pitch) = controller.angles();
        assert!(pitch <= controller.max_pitch);
    }

    #[test]
    fn test_focus_on() {
        let mut controller = EditorCameraController::new();
        let focus_point = Vec3::new(10.0, 5.0, -3.0);
        let distance = 20.0;

        controller.focus_on(focus_point, Some(distance));
        controller.update(1.0);

        assert_eq!(controller.target(), focus_point);
        assert_eq!(controller.distance(), distance);
    }

    #[test]
    fn test_compute_position() {
        let mut controller = EditorCameraController::new();
        controller.set_target(Vec3::ZERO);
        controller.set_distance(10.0);
        controller.set_angles(0.0, 0.0);
        controller.update(1.0);

        let position = controller.compute_position();

        // With yaw=0, pitch=0, camera should be at (0, 0, distance)
        assert!((position.z - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_smooth_interpolation() {
        let mut controller = EditorCameraController::new();
        controller.set_distance(10.0);
        controller.update(1.0);

        // Start at distance 10
        assert_eq!(controller.distance(), 10.0);

        // Request distance 20
        controller.set_distance(20.0);

        // After small update, should be between 10 and 20
        controller.update(0.01);
        let distance_after_small_update = controller.distance();
        assert!(distance_after_small_update > 10.0);
        assert!(distance_after_small_update < 20.0);

        // After large update, should reach target
        controller.update(1.0);
        assert!((controller.distance() - 20.0).abs() < 0.001);
    }
}
