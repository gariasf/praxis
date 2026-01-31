//! Ray primitives for raycasting and intersection tests.

use crate::{Aabb, Vec3};
use serde::{Deserialize, Serialize};

/// A ray in 3D space, defined by an origin and a direction.
///
/// Rays are commonly used for:
/// - Mouse picking (selecting objects by clicking)
/// - Line-of-sight checks
/// - Projectile trajectories
/// - Collision detection
///
/// # Examples
///
/// ```
/// use praxis_math::{Ray, Vec3};
///
/// // Create a ray from the origin pointing forward
/// let ray = Ray::new(Vec3::ZERO, Vec3::NEG_Z);
///
/// // Get a point along the ray at distance t
/// let point = ray.at(5.0);
/// assert_eq!(point, Vec3::new(0.0, 0.0, -5.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    /// The origin point of the ray.
    pub origin: Vec3,
    /// The direction of the ray (should be normalized for distance calculations to be meaningful).
    pub direction: Vec3,
}

impl Ray {
    /// Creates a new ray from an origin and direction.
    ///
    /// Note: This does not normalize the direction. Use `new_normalized` if you need
    /// the direction to be normalized.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Creates a new ray with a normalized direction.
    pub fn new_normalized(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize_or_zero(),
        }
    }

    /// Returns the point along the ray at parameter `t`.
    ///
    /// The point is calculated as: `origin + t * direction`
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Tests if the ray intersects an AABB within the given maximum distance.
    pub fn intersects_aabb(&self, aabb: &Aabb, max_distance: f32) -> bool {
        aabb.intersects_ray(self.origin, self.direction, max_distance)
    }

    /// Computes the intersection distance with an AABB.
    ///
    /// Returns `Some(distance)` if the ray intersects within `max_distance`, `None` otherwise.
    pub fn intersection_distance_aabb(&self, aabb: &Aabb, max_distance: f32) -> Option<f32> {
        aabb.ray_intersection_distance(self.origin, self.direction, max_distance)
    }

    /// Tests if the ray intersects a sphere.
    ///
    /// # Returns
    ///
    /// `Some((t1, t2))` where t1 and t2 are the near and far intersection distances.
    /// Returns `None` if there's no intersection.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> Option<(f32, f32)> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let sqrt_d = discriminant.sqrt();
            let t1 = (-b - sqrt_d) / (2.0 * a);
            let t2 = (-b + sqrt_d) / (2.0 * a);
            Some((t1, t2))
        }
    }

    /// Tests if the ray intersects a plane.
    ///
    /// A plane is defined by a normal and distance from origin.
    /// Returns the distance along the ray to the intersection point, or `None` if parallel.
    pub fn intersects_plane(&self, plane_normal: Vec3, plane_distance: f32) -> Option<f32> {
        let denom = self.direction.dot(plane_normal);

        if denom.abs() < 1e-6 {
            None
        } else {
            let t = -(self.origin.dot(plane_normal) + plane_distance) / denom;
            Some(t)
        }
    }

    /// Transforms the ray by a matrix.
    ///
    /// This is useful when casting rays in different coordinate spaces.
    pub fn transform(&self, transform: &crate::Mat4) -> Self {
        Self {
            origin: transform.transform_point3(self.origin),
            direction: transform.transform_vector3(self.direction),
        }
    }

    /// Returns the closest point on the ray to a given point.
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        let to_point = point - self.origin;
        let t = to_point.dot(self.direction).max(0.0);
        self.at(t)
    }

    /// Returns the distance from the ray to a point.
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let closest = self.closest_point(point);
        point.distance(closest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_ray_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        assert_eq!(ray.at(0.0), Vec3::ZERO);
        assert_eq!(ray.at(5.0), Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let aabb = Aabb::from_min_max(Vec3::new(5.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0));

        assert!(ray.intersects_aabb(&aabb, 100.0));
        assert!(!ray.intersects_aabb(&aabb, 3.0));

        let distance = ray.intersection_distance_aabb(&aabb, 100.0);
        assert!(distance.is_some());
        assert!((distance.unwrap() - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_ray_sphere_intersection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let center = Vec3::new(5.0, 0.0, 0.0);
        let radius = 1.0;

        let result = ray.intersects_sphere(center, radius);
        assert!(result.is_some());

        let (t1, t2) = result.unwrap();
        assert!((t1 - 4.0).abs() < EPSILON);
        assert!((t2 - 6.0).abs() < EPSILON);
    }

    #[test]
    fn test_ray_sphere_no_intersection() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let center = Vec3::new(0.0, 10.0, 0.0);
        let radius = 1.0;

        let result = ray.intersects_sphere(center, radius);
        assert!(result.is_none());
    }

    #[test]
    fn test_ray_plane_intersection() {
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
        let plane_normal = Vec3::Y;
        let plane_distance = 0.0;

        let t = ray.intersects_plane(plane_normal, plane_distance);
        assert!(t.is_some());
        assert!((t.unwrap() - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_ray_plane_parallel() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let plane_normal = Vec3::Y;
        let plane_distance = 0.0;

        let t = ray.intersects_plane(plane_normal, plane_distance);
        assert!(t.is_none());
    }

    #[test]
    fn test_ray_closest_point() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let point = Vec3::new(5.0, 3.0, 0.0);

        let closest = ray.closest_point(point);
        assert_eq!(closest, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_ray_distance_to_point() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let point = Vec3::new(5.0, 3.0, 0.0);

        let distance = ray.distance_to_point(point);
        assert!((distance - 3.0).abs() < EPSILON);
    }
}
