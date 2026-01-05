//! Axis-Aligned Bounding Box (AABB) implementation.
//!
//! AABBs are simple bounding volumes that tightly fit objects using box-shaped volumes
//! aligned with the world axes. They're efficient for intersection tests and spatial queries.

use praxis_math::{Mat4, Vec3};

/// Axis-Aligned Bounding Box.
///
/// An AABB is defined by its minimum and maximum corners. All edges are parallel to
/// the world coordinate axes, making intersection tests very fast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Minimum corner of the box.
    pub min: Vec3,
    /// Maximum corner of the box.
    pub max: Vec3,
}

impl Aabb {
    /// Creates a new AABB from minimum and maximum points.
    pub fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates a new AABB from a center point and half-extents.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Creates an AABB that contains all the given points.
    ///
    /// # Panics
    ///
    /// Panics if the points slice is empty when accessing the first element.
    /// This is prevented by the early return check, so it should never happen.
    pub fn from_points(points: &[Vec3]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for point in points.iter().skip(1) {
            min = min.min(*point);
            max = max.max(*point);
        }

        Some(Self { min, max })
    }

    /// Creates an empty AABB at the origin.
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Returns the center point of the AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-extents of the AABB.
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Returns the full size of the AABB.
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Returns the volume of the AABB.
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// Returns the surface area of the AABB.
    pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * size
            .x
            .mul_add(size.y, size.y.mul_add(size.z, size.z * size.x))
    }

    /// Tests if a point is inside the AABB.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Tests if this AABB intersects another AABB.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Tests if this AABB fully contains another AABB.
    pub fn contains(&self, other: &Self) -> bool {
        self.min.x <= other.min.x
            && self.max.x >= other.max.x
            && self.min.y <= other.min.y
            && self.max.y >= other.max.y
            && self.min.z <= other.min.z
            && self.max.z >= other.max.z
    }

    /// Expands the AABB to include another AABB.
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Expands the AABB to include a point.
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Grows the AABB by the given amount in all directions.
    pub fn grow(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec3::splat(amount),
            max: self.max + Vec3::splat(amount),
        }
    }

    /// Transforms the AABB by a matrix.
    pub fn transform(&self, transform: &Mat4) -> Self {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let transformed_corners: Vec<Vec3> = corners
            .iter()
            .map(|&corner| transform.transform_point3(corner))
            .collect();

        Self::from_points(&transformed_corners).unwrap_or_else(Self::empty)
    }

    /// Returns the closest point on the AABB to the given point.
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        point.clamp(self.min, self.max)
    }

    /// Returns the squared distance from a point to the AABB.
    pub fn distance_squared(&self, point: Vec3) -> f32 {
        let closest = self.closest_point(point);
        point.distance_squared(closest)
    }

    /// Returns the distance from a point to the AABB.
    pub fn distance(&self, point: Vec3) -> f32 {
        self.distance_squared(point).sqrt()
    }

    /// Returns the eight corner points of the AABB.
    pub fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    /// Tests if a ray intersects the AABB.
    ///
    /// Returns true if the ray intersects within `max_distance`.
    /// Uses the slab method for efficient ray-box intersection.
    pub fn intersects_ray(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> bool {
        let inv_dir = Vec3::new(
            if direction.x == 0.0 {
                f32::INFINITY
            } else {
                1.0 / direction.x
            },
            if direction.y == 0.0 {
                f32::INFINITY
            } else {
                1.0 / direction.y
            },
            if direction.z == 0.0 {
                f32::INFINITY
            } else {
                1.0 / direction.z
            },
        );

        let t1 = (self.min.x - origin.x) * inv_dir.x;
        let t2 = (self.max.x - origin.x) * inv_dir.x;
        let t3 = (self.min.y - origin.y) * inv_dir.y;
        let t4 = (self.max.y - origin.y) * inv_dir.y;
        let t5 = (self.min.z - origin.z) * inv_dir.z;
        let t6 = (self.max.z - origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        tmax >= 0.0 && tmin <= tmax && tmin <= max_distance
    }

    /// Computes the intersection distance of a ray with the AABB.
    ///
    /// Returns Some(distance) if the ray intersects within `max_distance`, None otherwise.
    pub fn ray_intersection_distance(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<f32> {
        let inv_dir = Vec3::new(
            if direction.x == 0.0 {
                f32::INFINITY
            } else {
                1.0 / direction.x
            },
            if direction.y == 0.0 {
                f32::INFINITY
            } else {
                1.0 / direction.y
            },
            if direction.z == 0.0 {
                f32::INFINITY
            } else {
                1.0 / direction.z
            },
        );

        let t1 = (self.min.x - origin.x) * inv_dir.x;
        let t2 = (self.max.x - origin.x) * inv_dir.x;
        let t3 = (self.min.y - origin.y) * inv_dir.y;
        let t4 = (self.max.y - origin.y) * inv_dir.y;
        let t5 = (self.min.z - origin.z) * inv_dir.z;
        let t6 = (self.max.z - origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax >= 0.0 && tmin <= tmax && tmin <= max_distance {
            Some(tmin.max(0.0))
        } else {
            None
        }
    }
}

/// Trait for objects that have bounding volumes.
pub trait BoundingVolume {
    /// Returns the AABB that bounds this object.
    fn aabb(&self) -> Aabb;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_creation() {
        let aabb = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.center(), Vec3::ZERO);
        assert_eq!(aabb.size(), Vec3::splat(2.0));
    }

    #[test]
    fn test_aabb_from_center() {
        let aabb = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.min, Vec3::splat(-1.0));
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains_point(Vec3::splat(0.5)));
        assert!(!aabb.contains_point(Vec3::splat(2.0)));
    }

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let aabb2 = Aabb::from_min_max(Vec3::splat(0.5), Vec3::splat(1.5));
        let aabb3 = Aabb::from_min_max(Vec3::splat(2.0), Vec3::splat(3.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_aabb_volume() {
        let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(2.0));
        assert_eq!(aabb.volume(), 8.0);
    }

    #[test]
    fn test_aabb_union() {
        let aabb1 = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let aabb2 = Aabb::from_min_max(Vec3::splat(0.5), Vec3::splat(1.5));
        let union = aabb1.union(&aabb2);
        assert_eq!(union.min, Vec3::ZERO);
        assert_eq!(union.max, Vec3::splat(1.5));
    }

    #[test]
    fn test_aabb_distance() {
        let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let point = Vec3::new(2.0, 0.5, 0.5);
        assert!((aabb.distance(point) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_ray_intersection() {
        let aabb = Aabb::from_min_max(Vec3::new(5.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0));

        let origin = Vec3::ZERO;
        let direction = Vec3::X;

        assert!(aabb.intersects_ray(origin, direction, 100.0));
        assert!(!aabb.intersects_ray(origin, direction, 3.0));

        let wrong_direction = Vec3::Y;
        assert!(!aabb.intersects_ray(origin, wrong_direction, 100.0));
    }

    #[test]
    fn test_aabb_ray_intersection_distance() {
        let aabb = Aabb::from_min_max(Vec3::new(5.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0));

        let origin = Vec3::ZERO;
        let direction = Vec3::X;

        let distance = aabb.ray_intersection_distance(origin, direction, 100.0);
        assert!(distance.is_some());

        let dist = distance.unwrap();
        assert!((dist - 5.0).abs() < 0.001);

        let no_hit = aabb.ray_intersection_distance(origin, direction, 3.0);
        assert!(no_hit.is_none());
    }

    #[test]
    fn test_aabb_ray_from_inside() {
        let aabb = Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));

        let origin = Vec3::ZERO;
        let direction = Vec3::X;

        assert!(aabb.intersects_ray(origin, direction, 100.0));

        let distance = aabb.ray_intersection_distance(origin, direction, 100.0);
        assert!(distance.is_some());
        assert_eq!(distance.unwrap(), 0.0);
    }
}
