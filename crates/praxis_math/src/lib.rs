//! Math library for the Praxis engine.
//!
//! This crate provides mathematical utilities used throughout the engine, built on
//! the `glam` library. Glam is a high-performance math library designed specifically
//! for game development, offering SIMD-optimized implementations of common math types.
//!
//! # Coordinate System Conventions
//!
//! Praxis uses a **right-handed, Y-up coordinate system**:
//! - **+X**: Right
//! - **+Y**: Up (vertical axis)
//! - **-Z**: Forward (into the screen in view space)
//!
//! This convention is consistent with:
//! - OpenGL and Vulkan standard conventions (with Y-up)
//! - Many 3D modeling tools (Blender, Maya with Y-up settings)
//! - Common game engine practices
//!
//! ## Right-Handed Rule
//!
//! Using your right hand:
//! - Point your thumb along +X (right)
//! - Point your index finger along +Y (up)
//! - Your middle finger naturally points along +Z (out of screen)
//! - Therefore, -Z points forward (into screen)
//!
//! # Core Math Types
//!
//! All types are re-exported from `glam` for consistency:
//!
//! ## Vectors
//! - `Vec2`: 2D vector (f32)
//! - `Vec3`: 3D vector (f32) - Most common for positions and directions
//! - `Vec4`: 4D vector (f32) - Used for homogeneous coordinates and colors
//! - `DVec2`, `DVec3`, `DVec4`: Double-precision variants
//! - `IVec2`, `IVec3`, `IVec4`: Integer variants
//! - `UVec2`, `UVec3`, `UVec4`: Unsigned integer variants
//!
//! ## Quaternions
//! - `Quat`: Unit quaternion for rotations (f32)
//! - `DQuat`: Double-precision quaternion
//!
//! **Why quaternions?** Quaternions are preferred over Euler angles for rotations because:
//! - No gimbal lock (a problem where two rotation axes align, losing a degree of freedom)
//! - Smooth interpolation (slerp - spherical linear interpolation)
//! - Efficient composition of rotations
//! - More numerically stable for cumulative rotations
//! - Compact representation (4 floats vs. 9 for a 3x3 matrix)
//!
//! ## Matrices
//! - `Mat2`: 2x2 matrix
//! - `Mat3`: 3x3 matrix - Used for 2D transforms and rotation-only 3D transforms
//! - `Mat4`: 4x4 matrix - Primary transform matrix for 3D graphics (model, view, projection)
//! - `DMat2`, `DMat3`, `DMat4`: Double-precision variants
//!
//! ## Affine Transforms
//! - `Affine2`: 2D affine transform (more efficient than Mat3)
//! - `Affine3A`: 3D affine transform (more efficient than Mat4 for model transforms)
//!
//! # Common Transform Operations
//!
//! ## Creating Transformations
//!
//! ```rust
//! use praxis_math::*;
//!
//! // Translation (position)
//! let translation = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
//!
//! // Rotation from quaternion
//! let rotation_quat = Quat::from_rotation_y(90_f32.to_radians());
//! let rotation = Mat4::from_quat(rotation_quat);
//!
//! // Rotation from axis-angle
//! let axis = Vec3::Y; // Rotate around Y-axis (vertical)
//! let angle = 90_f32.to_radians();
//! let rotation_axis = Mat4::from_axis_angle(axis, angle);
//!
//! // Scale (uniform and non-uniform)
//! let uniform_scale = Mat4::from_scale(Vec3::splat(2.0)); // Scale 2x in all directions
//! let non_uniform = Mat4::from_scale(Vec3::new(2.0, 1.0, 0.5)); // Different per axis
//!
//! // Combined transform (scale -> rotate -> translate, applied right-to-left)
//! let transform = Mat4::from_scale_rotation_translation(
//!     Vec3::splat(2.0),           // Scale
//!     Quat::from_rotation_y(1.57), // Rotation (90 degrees)
//!     Vec3::new(10.0, 0.0, 0.0),  // Translation
//! );
//! ```
//!
//! ## Quaternion Usage for Rotations
//!
//! ```rust
//! use praxis_math::*;
//!
//! // Create rotations
//! let rot_y = Quat::from_rotation_y(90_f32.to_radians()); // Around Y-axis (vertical)
//! let rot_x = Quat::from_rotation_x(45_f32.to_radians()); // Around X-axis (side-to-side)
//! let rot_z = Quat::from_rotation_z(30_f32.to_radians()); // Around Z-axis (roll)
//!
//! // From axis-angle (most general form)
//! let axis = Vec3::new(1.0, 1.0, 0.0).normalize(); // Must be normalized
//! let angle = 60_f32.to_radians();
//! let rot_axis = Quat::from_axis_angle(axis, angle);
//!
//! // From Euler angles (use sparingly due to gimbal lock)
//! let euler = Quat::from_euler(
//!     glam::EulerRot::YXZ, // Rotation order matters!
//!     90_f32.to_radians(), // Yaw (Y-axis)
//!     45_f32.to_radians(), // Pitch (X-axis)
//!     0_f32.to_radians(),  // Roll (Z-axis)
//! );
//!
//! // Compose rotations (multiply quaternions)
//! let combined = rot_y * rot_x; // Apply rot_x first, then rot_y
//!
//! // Rotate a vector
//! let direction = Vec3::new(0.0, 0.0, -1.0); // Forward in our coordinate system
//! let rotated = rot_y * direction;
//!
//! // Interpolate between rotations (smooth animation)
//! let start = Quat::IDENTITY;
//! let end = Quat::from_rotation_y(std::f32::consts::PI);
//! let t = 0.5; // Halfway
//! let interpolated = start.slerp(end, t); // Spherical linear interpolation
//!
//! // Get rotation axis and angle back
//! let (axis, angle) = interpolated.to_axis_angle();
//!
//! // Convert to matrix for rendering
//! let matrix = Mat4::from_quat(interpolated);
//! ```
//!
//! ## Camera and View Matrices
//!
//! ```rust
//! use praxis_math::*;
//!
//! // Look-at view matrix (right-handed, -Z forward)
//! let eye = Vec3::new(0.0, 10.0, 10.0);    // Camera position
//! let target = Vec3::ZERO;                  // Look at origin
//! let up = Vec3::Y;                         // Up direction
//! let view = Mat4::look_at_rh(eye, target, up);
//!
//! // Perspective projection (right-handed, Z in [0, 1] for Vulkan)
//! let fov_y = 45_f32.to_radians();         // Vertical field of view
//! let aspect = 16.0 / 9.0;                 // Aspect ratio
//! let near = 0.1;                           // Near clip plane
//! let far = 100.0;                          // Far clip plane
//! let projection = Mat4::perspective_rh(fov_y, aspect, near, far);
//!
//! // Orthographic projection (for 2D or isometric views)
//! let ortho = Mat4::orthographic_rh(
//!     -10.0, 10.0,  // Left, right
//!     -10.0, 10.0,  // Bottom, top
//!     -100.0, 100.0 // Near, far
//! );
//! ```
//!
//! ## Vector Operations
//!
//! ```rust
//! use praxis_math::*;
//!
//! let a = Vec3::new(1.0, 2.0, 3.0);
//! let b = Vec3::new(4.0, 5.0, 6.0);
//!
//! // Basic arithmetic
//! let sum = a + b;
//! let diff = a - b;
//! let scaled = a * 2.0;
//!
//! // Dot product (projection, angle between vectors)
//! let dot = a.dot(b);
//! let angle = dot / (a.length() * b.length()); // cos(angle)
//!
//! // Cross product (perpendicular vector, right-hand rule)
//! let perpendicular = a.cross(b); // Points according to right-hand rule
//!
//! // Length and normalization
//! let length = a.length();
//! let length_squared = a.length_squared(); // Faster, avoids sqrt
//! let normalized = a.normalize(); // Unit vector (length = 1)
//! let safe_normalized = a.normalize_or_zero(); // Returns zero if length is 0
//!
//! // Distance between points
//! let distance = a.distance(b);
//! let distance_squared = a.distance_squared(b); // Faster for comparisons
//!
//! // Interpolation
//! let t = 0.5; // Halfway
//! let lerp = a.lerp(b, t); // Linear interpolation
//!
//! // Component-wise operations
//! let min = a.min(b); // Minimum of each component
//! let max = a.max(b); // Maximum of each component
//! let clamped = a.clamp(Vec3::ZERO, Vec3::ONE); // Clamp each component
//! ```
//!
//! # Game Engine-Specific Usage
//!
//! ## Transform Hierarchies
//!
//! When building scene graphs with parent-child relationships:
//! ```rust
//! use praxis_math::*;
//!
//! // Parent transform (world space)
//! let parent_transform = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
//!
//! // Child transform (local space, relative to parent)
//! let child_local = Mat4::from_translation(Vec3::new(0.0, 5.0, 0.0));
//!
//! // Child in world space = parent * child (matrix multiplication)
//! let child_world = parent_transform * child_local;
//! ```
//!
//! ## Physics and Collision Detection
//!
//! ```rust
//! use praxis_math::*;
//!
//! // Ray casting
//! let ray_origin = Vec3::new(0.0, 1.0, 0.0);
//! let ray_direction = Vec3::new(0.0, -1.0, 0.0).normalize();
//!
//! // Bounding box intersection
//! let box_min = Vec3::new(-1.0, -1.0, -1.0);
//! let box_max = Vec3::new(1.0, 1.0, 1.0);
//!
//! // Distance checks (use squared distance to avoid sqrt when possible)
//! let point_a = Vec3::new(0.0, 0.0, 0.0);
//! let point_b = Vec3::new(3.0, 4.0, 0.0);
//! let dist_sq = point_a.distance_squared(point_b);
//! let collision_radius = 5.0;
//! let is_colliding = dist_sq < collision_radius * collision_radius;
//! ```
//!
//! ## Performance Considerations
//!
//! - **Use SIMD types**: `Vec3A`, `Vec4` are SIMD-aligned for better performance
//! - **Avoid unnecessary operations**: Use `length_squared()` instead of `length()` for comparisons
//! - **Quaternions over matrices**: Store rotations as `Quat`, convert to `Mat4` only for rendering
//! - **Affine transforms**: Use `Affine3A` instead of `Mat4` when you don't need projection
//! - **Batch operations**: Process arrays of vectors/matrices together when possible
//! - **Normalize sparingly**: Only normalize when necessary (e.g., after accumulation)

use praxis_utils::{info, Result};

// Re-export all glam types and functions for convenient access throughout the engine.
// This allows other crates to use math types via `praxis_math::Vec3` instead of
// depending on glam directly, providing a stable interface even if we change the
// underlying math library in the future.
//
// Glam provides:
// - SIMD-optimized implementations for performance-critical operations
// - Comprehensive set of vector, matrix, and quaternion types
// - No-std support (important for some embedded or console targets)
// - Battle-tested by numerous game engines and graphics applications
pub use glam::*;

// Re-export serde for convenience when defining serializable math types
pub use serde::{Deserialize, Serialize};

// Public modules
pub mod aabb;
pub mod coordinate_spaces;
pub mod frustum;
pub mod helpers;
pub mod ray;

// Re-export commonly used types at crate root
pub use aabb::Aabb;
pub use coordinate_spaces::{CoordinateSpace, Space};
pub use frustum::{Frustum, Plane};
pub use helpers::*;
pub use ray::Ray;

/// Initializes the math library.
///
/// This function sets up any necessary global state for the math library.
/// Currently, it's a placeholder for future initialization needs.
///
/// # Purpose
///
/// The initialization function serves as a centralized entry point for math
/// subsystem setup. Currently, it:
/// - Logs initialization status for debugging and monitoring
/// - Provides a hook for future initialization needs (e.g., SIMD feature detection)
///
/// # Example
///
/// ```rust,no_run
/// praxis_math::init().expect("Failed to initialize math library");
/// ```
///
/// # Errors
///
/// Returns an error if initialization fails. Currently, this function always succeeds.
pub fn init() -> Result<()> {
    info!("Initializing math library");
    Ok(())
}
