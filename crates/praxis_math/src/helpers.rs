//! Common mathematical helper functions.
//!
//! This module provides convenience functions for common mathematical operations
//! used throughout the engine, including interpolation, clamping, and angle utilities.

use crate::{Quat, Vec2, Vec3, Vec4};

/// Linear interpolation between two values.
///
/// Returns `a` when `t = 0.0` and `b` when `t = 1.0`.
/// Values outside [0, 1] will extrapolate.
///
/// # Examples
///
/// ```
/// use praxis_math::lerp;
///
/// assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
/// assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
/// assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
/// ```
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation for Vec2.
pub fn lerp_vec2(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a.lerp(b, t)
}

/// Linear interpolation for Vec3.
pub fn lerp_vec3(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a.lerp(b, t)
}

/// Linear interpolation for Vec4.
pub fn lerp_vec4(a: Vec4, b: Vec4, t: f32) -> Vec4 {
    a.lerp(b, t)
}

/// Spherical linear interpolation for quaternions.
///
/// Provides smooth rotation interpolation between two orientations.
/// This is the preferred method for interpolating rotations.
///
/// # Examples
///
/// ```
/// use praxis_math::{slerp, Quat};
///
/// let start = Quat::IDENTITY;
/// let end = Quat::from_rotation_y(std::f32::consts::PI);
/// let halfway = slerp(start, end, 0.5);
/// ```
pub fn slerp(a: Quat, b: Quat, t: f32) -> Quat {
    a.slerp(b, t)
}

/// Normalized linear interpolation for quaternions.
///
/// Faster than slerp but less accurate for large rotations.
/// Good for small angular differences or when performance is critical.
pub fn nlerp(a: Quat, b: Quat, t: f32) -> Quat {
    a.lerp(b, t).normalize()
}

/// Clamps a value between a minimum and maximum.
///
/// # Examples
///
/// ```
/// use praxis_math::clamp;
///
/// assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
/// assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
/// assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
/// ```
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.clamp(min, max)
}

/// Clamps a Vec3 component-wise between minimum and maximum vectors.
pub fn clamp_vec3(value: Vec3, min: Vec3, max: Vec3) -> Vec3 {
    value.clamp(min, max)
}

/// Smoothstep interpolation (cubic Hermite interpolation).
///
/// Provides smooth acceleration and deceleration between 0 and 1.
/// Returns 0 for t <= 0, 1 for t >= 1, and smoothly interpolates in between.
///
/// # Examples
///
/// ```
/// use praxis_math::smoothstep;
///
/// assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
/// assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
/// // Middle value is smoother than linear interpolation
/// ```
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Smoother interpolation than smoothstep (quintic interpolation).
///
/// Provides even smoother acceleration and deceleration with zero 2nd derivative at edges.
pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Inverse linear interpolation - finds t for a value between a and b.
///
/// Returns the parameter t such that `lerp(a, b, t) = value`.
///
/// # Examples
///
/// ```
/// use praxis_math::inverse_lerp;
///
/// assert_eq!(inverse_lerp(0.0, 10.0, 5.0), 0.5);
/// assert_eq!(inverse_lerp(0.0, 10.0, 0.0), 0.0);
/// assert_eq!(inverse_lerp(0.0, 10.0, 10.0), 1.0);
/// ```
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < 1e-6 {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Remaps a value from one range to another.
///
/// # Examples
///
/// ```
/// use praxis_math::remap;
///
/// // Map 5 from range [0, 10] to range [0, 100]
/// assert_eq!(remap(5.0, 0.0, 10.0, 0.0, 100.0), 50.0);
/// ```
pub fn remap(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = inverse_lerp(in_min, in_max, value);
    lerp(out_min, out_max, t)
}

/// Converts degrees to radians.
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees.to_radians()
}

/// Converts radians to degrees.
pub fn rad_to_deg(radians: f32) -> f32 {
    radians.to_degrees()
}

/// Wraps an angle to the range [-PI, PI].
pub fn wrap_angle(angle: f32) -> f32 {
    // Use atan2 for robust angle wrapping to (-π, π]
    angle.sin().atan2(angle.cos())
}

/// Computes the shortest angular difference between two angles.
///
/// Returns a value in the range [-PI, PI].
pub fn angle_difference(from: f32, to: f32) -> f32 {
    wrap_angle(to - from)
}

/// Linearly interpolates between two angles, taking the shortest path.
pub fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    a + angle_difference(a, b) * t
}

/// Checks if two floating point values are approximately equal.
pub fn approx_equal(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// Checks if a floating point value is approximately zero.
pub fn approx_zero(value: f32, epsilon: f32) -> bool {
    value.abs() < epsilon
}

/// Returns the sign of a number (-1, 0, or 1).
pub fn sign(value: f32) -> f32 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// Snaps a value to the nearest multiple of step.
///
/// # Examples
///
/// ```
/// use praxis_math::snap;
///
/// assert_eq!(snap(7.3, 5.0), 5.0);
/// assert_eq!(snap(8.0, 5.0), 10.0);
/// ```
pub fn snap(value: f32, step: f32) -> f32 {
    if step == 0.0 {
        value
    } else {
        (value / step).round() * step
    }
}

/// Moves a value towards a target by a maximum delta.
///
/// Useful for smooth following behavior.
pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}

/// Exponential decay interpolation for smooth damping.
///
/// Provides frame-rate independent smooth interpolation.
/// `lambda` controls the speed (higher = faster convergence).
pub fn exp_decay(a: f32, b: f32, decay: f32, dt: f32) -> f32 {
    b + (a - b) * (-decay * dt).exp()
}

/// Exponential decay for Vec3.
pub fn exp_decay_vec3(a: Vec3, b: Vec3, decay: f32, dt: f32) -> Vec3 {
    let factor = (-decay * dt).exp();
    b + (a - b) * factor
}

/// Spring damper for smooth, physics-based interpolation.
///
/// Simulates a spring-damper system for natural-feeling motion.
/// - `current`: Current value
/// - `target`: Target value
/// - `velocity`: Current velocity (updated in-place)
/// - `omega`: Angular frequency (higher = stiffer spring)
/// - `zeta`: Damping ratio (1.0 = critical damping)
/// - `dt`: Time step
pub fn spring_damper(
    current: f32,
    target: f32,
    velocity: &mut f32,
    omega: f32,
    zeta: f32,
    dt: f32,
) -> f32 {
    let f = 1.0 + 2.0 * dt * zeta * omega;
    let oo = omega * omega;
    let hoo = dt * oo;
    let hhoo = dt * hoo;
    let detinv = 1.0 / (f + hhoo);
    let detx = f * current + dt * *velocity + hhoo * target;
    let detv = *velocity + hoo * (target - current);
    let new_pos = detx * detinv;
    *velocity = detv * detinv;
    new_pos
}

/// Spring damper for Vec3.
pub fn spring_damper_vec3(
    current: Vec3,
    target: Vec3,
    velocity: &mut Vec3,
    omega: f32,
    zeta: f32,
    dt: f32,
) -> Vec3 {
    Vec3::new(
        spring_damper(current.x, target.x, &mut velocity.x, omega, zeta, dt),
        spring_damper(current.y, target.y, &mut velocity.y, omega, zeta, dt),
        spring_damper(current.z, target.z, &mut velocity.z, omega, zeta, dt),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
        assert!(smoothstep(0.0, 1.0, 0.5) > 0.4 && smoothstep(0.0, 1.0, 0.5) < 0.6);
    }

    #[test]
    fn test_inverse_lerp() {
        assert_eq!(inverse_lerp(0.0, 10.0, 5.0), 0.5);
        assert_eq!(inverse_lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(inverse_lerp(0.0, 10.0, 10.0), 1.0);
    }

    #[test]
    fn test_remap() {
        assert_eq!(remap(5.0, 0.0, 10.0, 0.0, 100.0), 50.0);
        assert_eq!(remap(0.0, 0.0, 10.0, 0.0, 100.0), 0.0);
        assert_eq!(remap(10.0, 0.0, 10.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn test_deg_rad_conversion() {
        assert!((deg_to_rad(180.0) - std::f32::consts::PI).abs() < EPSILON);
        assert!((rad_to_deg(std::f32::consts::PI) - 180.0).abs() < EPSILON);
    }

    #[test]
    fn test_wrap_angle() {
        let pi = std::f32::consts::PI;
        // atan2 gives range (-π, π], so 3π wraps to π, and -3π wraps to π
        assert!((wrap_angle(pi * 3.0) - pi).abs() < EPSILON || (wrap_angle(pi * 3.0) + pi).abs() < EPSILON);
        assert!((wrap_angle(-pi * 3.0) - pi).abs() < EPSILON || (wrap_angle(-pi * 3.0) + pi).abs() < EPSILON);
        // Test some other values
        assert!((wrap_angle(0.0) - 0.0).abs() < EPSILON);
        assert!((wrap_angle(pi / 2.0) - (pi / 2.0)).abs() < EPSILON);
    }

    #[test]
    fn test_angle_difference() {
        let pi = std::f32::consts::PI;
        // angle_difference should give the shortest path, which for 0 to π is π (or -π, same angle)
        let diff = angle_difference(0.0, pi);
        assert!((diff - pi).abs() < EPSILON || (diff + pi).abs() < EPSILON);
        // 1.5π from 0 should be -0.5π (shortest path)
        assert!((angle_difference(0.0, pi * 1.5).abs() - pi / 2.0).abs() < EPSILON);
    }

    #[test]
    fn test_approx_equal() {
        assert!(approx_equal(1.0, 1.0001, 0.001));
        assert!(!approx_equal(1.0, 1.01, 0.001));
    }

    #[test]
    fn test_sign() {
        assert_eq!(sign(5.0), 1.0);
        assert_eq!(sign(-5.0), -1.0);
        assert_eq!(sign(0.0), 0.0);
    }

    #[test]
    fn test_snap() {
        assert_eq!(snap(7.3, 5.0), 5.0);
        assert_eq!(snap(8.0, 5.0), 10.0);
        assert_eq!(snap(12.4, 5.0), 10.0);
    }

    #[test]
    fn test_move_towards() {
        assert_eq!(move_towards(0.0, 10.0, 5.0), 5.0);
        assert_eq!(move_towards(0.0, 3.0, 5.0), 3.0);
        assert_eq!(move_towards(10.0, 0.0, 5.0), 5.0);
    }

    #[test]
    fn test_slerp() {
        let start = Quat::IDENTITY;
        let end = Quat::from_rotation_y(std::f32::consts::PI);
        let halfway = slerp(start, end, 0.5);

        let angle = halfway.to_axis_angle().1;
        assert!((angle - std::f32::consts::PI / 2.0).abs() < EPSILON);
    }

    #[test]
    fn test_spring_damper() {
        let mut velocity = 0.0;
        let mut current = 0.0;
        let target = 10.0;

        for _ in 0..200 {
            current = spring_damper(current, target, &mut velocity, 5.0, 1.0, 0.016);
        }

        assert!((current - target).abs() < 0.1);
        assert!(velocity.abs() < 0.5);
    }
}
