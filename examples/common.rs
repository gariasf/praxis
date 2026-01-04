//! Common utilities shared across examples.
//!
//! This module contains reusable components used in multiple examples,
//! such as camera controllers and input handling utilities.

use praxis_ecs::Resource;
use praxis_math::Quat;

/// FPS-style camera controller with mouse look and WASD movement.
///
/// This controller provides standard FPS game camera controls:
/// - Mouse look with configurable sensitivity
/// - WASD movement in camera-relative directions
/// - Vertical look clamping to prevent over-rotation
/// - Sprint mode with configurable multiplier
/// - Optional entity tracking for ECS integration
///
/// # Usage
///
/// ```rust,ignore
/// mod common;
/// use common::CameraController;
///
/// let mut controller = CameraController::default();
/// controller.update_rotation(mouse_delta_x, mouse_delta_y);
/// let rotation = controller.get_rotation();
/// ```
#[derive(Resource)]
pub struct CameraController {
    /// Base movement speed in units per second
    pub move_speed: f32,
    /// Multiplier applied when sprint is active
    pub sprint_multiplier: f32,
    /// Mouse sensitivity (radians per pixel)
    pub mouse_sensitivity: f32,
    /// Current pitch angle (up/down rotation) in radians
    pub pitch: f32,
    /// Current yaw angle (left/right rotation) in radians
    pub yaw: f32,
    /// Maximum pitch angle to prevent over-rotation (typically ~89 degrees)
    pub max_pitch: f32,
    /// Optional camera entity for ECS integration
    pub camera_entity: Option<praxis_ecs::Entity>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            sprint_multiplier: 2.0,
            mouse_sensitivity: 0.002,
            pitch: 0.0,
            yaw: std::f32::consts::PI, // Start facing forward
            max_pitch: std::f32::consts::FRAC_PI_2 - 0.01, // ~89 degrees
            camera_entity: None,
        }
    }
}

impl CameraController {
    /// Updates the camera rotation based on mouse delta movement.
    ///
    /// # Arguments
    ///
    /// * `delta_x` - Horizontal mouse movement (positive = right)
    /// * `delta_y` - Vertical mouse movement (positive = down)
    ///
    /// The pitch is automatically clamped to prevent over-rotation.
    pub fn update_rotation(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw -= delta_x * self.mouse_sensitivity;
        self.pitch -= delta_y * self.mouse_sensitivity;
        self.pitch = self.pitch.clamp(-self.max_pitch, self.max_pitch);
    }

    /// Returns the current camera rotation as a quaternion.
    ///
    /// The rotation is computed as yaw (Y-axis) followed by pitch (X-axis),
    /// which creates the standard FPS camera behavior.
    pub fn get_rotation(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
    }
}

/// This file provides common utilities for examples.
/// Run other examples like `cargo run --example physics_demo` instead.
fn main() {
    eprintln!("common.rs is a utility module, not a standalone example.");
    eprintln!("Run other examples like: cargo run --example physics_demo");
}
