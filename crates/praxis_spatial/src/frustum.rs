//! View frustum culling for camera visibility testing.
//!
//! The frustum represents the visible volume of space from a camera's perspective.
//! Objects outside this volume are not visible and can be culled from rendering.

use crate::aabb::Aabb;
use praxis_math::{Mat4, Vec3};

/// A plane in 3D space defined by a normal and distance from origin.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector (should be normalized).
    pub normal: Vec3,
    /// Distance from the origin along the normal.
    pub distance: f32,
}

impl Plane {
    /// Creates a new plane from a normal and distance.
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Creates a plane from the coefficients (a, b, c, d) in the equation ax + by + cz + d = 0.
    pub fn from_coefficients(a: f32, b: f32, c: f32, d: f32) -> Self {
        let normal = Vec3::new(a, b, c);
        let length = normal.length();

        // Validate: ensure length is non-zero and normal is valid
        if length > 1e-6 && normal.is_finite() && d.is_finite() {
            Self {
                normal: normal / length,
                distance: d / length,
            }
        } else {
            // Return a safe default plane if inputs are invalid
            Self {
                normal: Vec3::Y,
                distance: 0.0,
            }
        }
    }

    /// Returns the signed distance from a point to the plane.
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    /// Tests if a point is in front of the plane (positive side).
    pub fn is_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }

    /// Normalizes the plane equation.
    pub fn normalize(&mut self) {
        let length = self.normal.length();
        self.normal /= length;
        self.distance /= length;
    }
}

/// View frustum for culling tests.
///
/// A frustum is a truncated pyramid that represents the visible space from a camera.
/// It's defined by six planes: near, far, left, right, top, and bottom.
#[derive(Debug, Clone)]
pub struct Frustum {
    /// The six frustum planes: [near, far, left, right, top, bottom]
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Creates a frustum from a view-projection matrix.
    pub fn from_view_projection(view_proj: Mat4) -> Self {
        // Validate the view-projection matrix first
        if !Self::is_valid_matrix(&view_proj) {
            // Return a default frustum if matrix is invalid
            return Self {
                planes: [
                    Plane::new(Vec3::NEG_Z, -0.1), // Near
                    Plane::new(Vec3::Z, -1000.0),  // Far
                    Plane::new(Vec3::X, 0.0),      // Left
                    Plane::new(Vec3::NEG_X, 0.0),  // Right
                    Plane::new(Vec3::NEG_Y, 0.0),  // Top
                    Plane::new(Vec3::Y, 0.0),      // Bottom
                ],
            };
        }

        let m = view_proj.to_cols_array_2d();

        let planes = [
            // Near plane
            Plane::from_coefficients(
                m[0][3] + m[0][2],
                m[1][3] + m[1][2],
                m[2][3] + m[2][2],
                m[3][3] + m[3][2],
            ),
            // Far plane
            Plane::from_coefficients(
                m[0][3] - m[0][2],
                m[1][3] - m[1][2],
                m[2][3] - m[2][2],
                m[3][3] - m[3][2],
            ),
            // Left plane
            Plane::from_coefficients(
                m[0][3] + m[0][0],
                m[1][3] + m[1][0],
                m[2][3] + m[2][0],
                m[3][3] + m[3][0],
            ),
            // Right plane
            Plane::from_coefficients(
                m[0][3] - m[0][0],
                m[1][3] - m[1][0],
                m[2][3] - m[2][0],
                m[3][3] - m[3][0],
            ),
            // Top plane
            Plane::from_coefficients(
                m[0][3] - m[0][1],
                m[1][3] - m[1][1],
                m[2][3] - m[2][1],
                m[3][3] - m[3][1],
            ),
            // Bottom plane
            Plane::from_coefficients(
                m[0][3] + m[0][1],
                m[1][3] + m[1][1],
                m[2][3] + m[2][1],
                m[3][3] + m[3][1],
            ),
        ];

        Self { planes }
    }

    /// Validates that a matrix contains finite values and is not degenerate.
    fn is_valid_matrix(mat: &Mat4) -> bool {
        let arr = mat.to_cols_array();
        for &val in &arr {
            if !val.is_finite() {
                return false;
            }
        }
        true
    }

    /// Tests if a point is inside the frustum.
    pub fn contains_point(&self, point: Vec3) -> bool {
        // Validate point is finite
        if !point.is_finite() {
            return false;
        }

        for plane in &self.planes {
            // Validate plane normal is finite
            if !plane.normal.is_finite() {
                continue;
            }

            if !plane.is_in_front(point) {
                return false;
            }
        }
        true
    }

    /// Tests if an AABB intersects the frustum.
    ///
    /// Returns true if the AABB is partially or fully inside the frustum.
    pub fn intersects_aabb(&self, aabb: &Aabb) -> bool {
        // Validate AABB bounds are finite
        if !aabb.min.is_finite() || !aabb.max.is_finite() {
            return false;
        }

        for plane in &self.planes {
            // Validate plane normal is finite
            if !plane.normal.is_finite() {
                continue;
            }

            let positive_vertex = Vec3::new(
                if plane.normal.x >= 0.0 {
                    aabb.max.x
                } else {
                    aabb.min.x
                },
                if plane.normal.y >= 0.0 {
                    aabb.max.y
                } else {
                    aabb.min.y
                },
                if plane.normal.z >= 0.0 {
                    aabb.max.z
                } else {
                    aabb.min.z
                },
            );

            let distance = plane.distance_to_point(positive_vertex);

            // Validate distance is finite before comparison
            if distance.is_finite() && distance < 0.0 {
                return false;
            }
        }
        true
    }

    /// Tests if a sphere intersects the frustum.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        // Validate inputs are finite
        if !center.is_finite() || !radius.is_finite() || radius < 0.0 {
            return false;
        }

        for plane in &self.planes {
            // Validate plane normal is finite
            if !plane.normal.is_finite() {
                continue;
            }

            let distance = plane.distance_to_point(center);

            // Validate distance is finite before comparison
            if distance.is_finite() && distance < -radius {
                return false;
            }
        }
        true
    }
}

/// Frustum culling system for efficient visibility tests.
pub struct FrustumCuller {
    /// Current camera frustum
    frustum: Frustum,
}

impl FrustumCuller {
    /// Creates a new frustum culler.
    pub fn new() -> Self {
        Self {
            frustum: Frustum::from_view_projection(Mat4::IDENTITY),
        }
    }

    /// Updates the frustum from the current camera view-projection matrix.
    pub fn update(&mut self, view_proj: Mat4) {
        self.frustum = Frustum::from_view_projection(view_proj);
    }

    /// Tests if an AABB is visible in the current frustum.
    pub fn is_visible(&self, aabb: &Aabb) -> bool {
        self.frustum.intersects_aabb(aabb)
    }

    /// Tests if a sphere is visible in the current frustum.
    pub fn is_sphere_visible(&self, center: Vec3, radius: f32) -> bool {
        self.frustum.intersects_sphere(center, radius)
    }

    /// Tests if a point is visible in the current frustum.
    pub fn is_point_visible(&self, point: Vec3) -> bool {
        self.frustum.contains_point(point)
    }

    /// Returns a reference to the current frustum.
    pub fn frustum(&self) -> &Frustum {
        &self.frustum
    }
}

impl Default for FrustumCuller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_plane_distance() {
        let plane = Plane::new(Vec3::Y, 0.0);
        assert!((plane.distance_to_point(Vec3::ZERO) - 0.0).abs() < EPSILON);
        assert!((plane.distance_to_point(Vec3::new(0.0, 5.0, 0.0)) - 5.0).abs() < EPSILON);
        assert!((plane.distance_to_point(Vec3::new(0.0, -3.0, 0.0)) - (-3.0)).abs() < EPSILON);
    }

    #[test]
    fn test_plane_is_in_front() {
        let plane = Plane::new(Vec3::Y, -5.0);
        assert!(plane.is_in_front(Vec3::new(0.0, 10.0, 0.0)));
        assert!(!plane.is_in_front(Vec3::new(0.0, 0.0, 0.0)));
    }

    #[test]
    fn test_frustum_from_view_projection() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(proj * view);

        assert_eq!(frustum.planes.len(), 6);
    }

    #[test]
    fn test_frustum_contains_point() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(proj * view);

        assert!(frustum.contains_point(Vec3::ZERO));
    }

    #[test]
    fn test_frustum_intersects_aabb() {
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(proj * view);

        let aabb = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(frustum.intersects_aabb(&aabb));
    }

    #[test]
    fn test_frustum_culler_update() {
        let mut culler = FrustumCuller::new();

        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        culler.update(proj * view);

        let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        assert!(culler.is_visible(&aabb));
    }
}
