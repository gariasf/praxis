//! Coordinate space utilities for transforming between different coordinate systems.
//!
//! In 3D graphics, we work with multiple coordinate spaces:
//! - **Local/Object Space**: Coordinates relative to an object's center
//! - **World Space**: Coordinates in the global scene
//! - **View/Camera Space**: Coordinates relative to the camera
//! - **Clip Space**: Homogeneous coordinates after projection
//! - **Screen Space**: 2D pixel coordinates
//!
//! This module provides utilities for converting between these spaces and
//! understanding their relationships.

use crate::{Mat4, Quat, Vec3};

/// Represents different coordinate spaces in a 3D rendering pipeline.
///
/// Each space has its own coordinate system and is related to others through
/// transformation matrices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoordinateSpace {
    /// Local/Object space - coordinates relative to the object's origin.
    Local,
    /// World space - global scene coordinates.
    World,
    /// View/Camera space - coordinates relative to the camera.
    View,
    /// Clip space - homogeneous coordinates after projection.
    Clip,
    /// Screen/NDC space - normalized device coordinates or pixel coordinates.
    Screen,
}

/// Type alias for coordinate space marker.
pub type Space = CoordinateSpace;

/// Helper functions for working with coordinate space transformations.
pub trait CoordinateSpaceExt {
    /// Transforms a position from local space to world space.
    fn local_to_world(position: Vec3, transform: &Mat4) -> Vec3;

    /// Transforms a direction from local space to world space.
    fn local_to_world_direction(direction: Vec3, transform: &Mat4) -> Vec3;

    /// Transforms a position from world space to view space.
    fn world_to_view(position: Vec3, view_matrix: &Mat4) -> Vec3;

    /// Transforms a position from view space to clip space.
    fn view_to_clip(position: Vec3, projection_matrix: &Mat4) -> Vec3;

    /// Transforms a position from local space directly to clip space.
    fn local_to_clip(position: Vec3, model: &Mat4, view: &Mat4, projection: &Mat4) -> Vec3;

    /// Transforms a position from clip space to NDC (normalized device coordinates).
    fn clip_to_ndc(clip_pos: Vec3) -> Vec3;

    /// Transforms from NDC to screen coordinates.
    fn ndc_to_screen(ndc: Vec3, viewport_width: u32, viewport_height: u32) -> Vec3;

    /// Creates a look-at view matrix (right-handed).
    fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4;

    /// Creates a perspective projection matrix (right-handed).
    fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4;

    /// Creates an orthographic projection matrix (right-handed).
    fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32)
        -> Mat4;
}

impl CoordinateSpaceExt for Mat4 {
    fn local_to_world(position: Vec3, transform: &Mat4) -> Vec3 {
        transform.transform_point3(position)
    }

    fn local_to_world_direction(direction: Vec3, transform: &Mat4) -> Vec3 {
        transform.transform_vector3(direction)
    }

    fn world_to_view(position: Vec3, view_matrix: &Mat4) -> Vec3 {
        view_matrix.transform_point3(position)
    }

    fn view_to_clip(position: Vec3, projection_matrix: &Mat4) -> Vec3 {
        projection_matrix.transform_point3(position)
    }

    fn local_to_clip(position: Vec3, model: &Mat4, view: &Mat4, projection: &Mat4) -> Vec3 {
        let world_pos = Self::local_to_world(position, model);
        let view_pos = Self::world_to_view(world_pos, view);
        Self::view_to_clip(view_pos, projection)
    }

    fn clip_to_ndc(clip_pos: Vec3) -> Vec3 {
        clip_pos
    }

    fn ndc_to_screen(ndc: Vec3, viewport_width: u32, viewport_height: u32) -> Vec3 {
        let x = (ndc.x + 1.0) * 0.5 * viewport_width as f32;
        let y = (1.0 - ndc.y) * 0.5 * viewport_height as f32;
        Vec3::new(x, y, ndc.z)
    }

    fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_at_rh(eye, target, up)
    }

    fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
        Mat4::perspective_rh(fov_y_radians, aspect_ratio, near, far)
    }

    fn orthographic_rh(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Mat4 {
        Mat4::orthographic_rh(left, right, bottom, top, near, far)
    }
}

/// Helper for building transformation matrices.
pub struct TransformBuilder {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl TransformBuilder {
    /// Creates a new transform builder with identity values.
    pub fn new() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Sets the translation.
    pub fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self
    }

    /// Sets the rotation from a quaternion.
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the rotation from Euler angles (yaw, pitch, roll in radians).
    pub fn with_euler_angles(mut self, yaw: f32, pitch: f32, roll: f32) -> Self {
        self.rotation = Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll);
        self
    }

    /// Sets the scale.
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Sets uniform scale.
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec3::splat(scale);
        self
    }

    /// Builds the transformation matrix.
    pub fn build(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for TransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decomposes a transformation matrix into translation, rotation, and scale.
///
/// Note: This assumes the matrix is a valid TRS (Translation-Rotation-Scale) matrix.
/// Non-uniform scaling with rotation may not decompose correctly.
pub fn decompose_transform(matrix: &Mat4) -> (Vec3, Quat, Vec3) {
    let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
    (translation, rotation, scale)
}

/// Extracts the translation component from a transformation matrix.
pub fn extract_translation(matrix: &Mat4) -> Vec3 {
    Vec3::new(matrix.w_axis.x, matrix.w_axis.y, matrix.w_axis.z)
}

/// Extracts the scale component from a transformation matrix.
pub fn extract_scale(matrix: &Mat4) -> Vec3 {
    let scale_x = matrix.x_axis.truncate().length();
    let scale_y = matrix.y_axis.truncate().length();
    let scale_z = matrix.z_axis.truncate().length();
    Vec3::new(scale_x, scale_y, scale_z)
}

/// Extracts the rotation component from a transformation matrix.
pub fn extract_rotation(matrix: &Mat4) -> Quat {
    let (_scale, rotation, _translation) = matrix.to_scale_rotation_translation();
    rotation
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_transform_builder() {
        let transform = TransformBuilder::new()
            .with_translation(Vec3::new(1.0, 2.0, 3.0))
            .with_uniform_scale(2.0)
            .build();

        let translation = extract_translation(&transform);
        assert!((translation.x - 1.0).abs() < EPSILON);
        assert!((translation.y - 2.0).abs() < EPSILON);
        assert!((translation.z - 3.0).abs() < EPSILON);
    }

    #[test]
    fn test_decompose_transform() {
        let original_translation = Vec3::new(1.0, 2.0, 3.0);
        let original_rotation = Quat::from_rotation_y(45.0_f32.to_radians());
        let original_scale = Vec3::splat(2.0);

        let matrix =
            Mat4::from_scale_rotation_translation(original_scale, original_rotation, original_translation);

        let (translation, rotation, scale) = decompose_transform(&matrix);

        assert!((translation - original_translation).length() < EPSILON);
        assert!((scale - original_scale).length() < EPSILON);
        assert!((rotation.xyz() - original_rotation.xyz()).length() < EPSILON);
    }

    #[test]
    fn test_local_to_world() {
        let transform = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let local_pos = Vec3::new(5.0, 0.0, 0.0);

        let world_pos = Mat4::local_to_world(local_pos, &transform);

        assert_eq!(world_pos, Vec3::new(15.0, 0.0, 0.0));
    }

    #[test]
    fn test_ndc_to_screen() {
        let ndc = Vec3::new(0.0, 0.0, 0.5);
        let screen = Mat4::ndc_to_screen(ndc, 1920, 1080);

        assert!((screen.x - 960.0).abs() < EPSILON);
        assert!((screen.y - 540.0).abs() < EPSILON);
    }

    #[test]
    fn test_look_at() {
        let eye = Vec3::new(0.0, 0.0, 5.0);
        let target = Vec3::ZERO;
        let up = Vec3::Y;

        let view = Mat4::look_at_rh(eye, target, up);

        let origin_in_view = Mat4::world_to_view(Vec3::ZERO, &view);
        assert!(origin_in_view.z < 0.0);
    }
}
