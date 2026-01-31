# praxis_math

Math library for Praxis engine using glam.

## Overview

Thin wrapper around `glam` providing SIMD-optimized 3D mathematics with serialization support.
Includes coordinate space utilities, interpolation helpers, and geometric primitives.

## Core Types

### Vectors
- **`Vec2`, `Vec3`, `Vec4`**: 2D, 3D, and 4D vectors (f32)
- **`DVec2`, `DVec3`, `DVec4`**: Double-precision vectors
- **`IVec2`, `IVec3`, `IVec4`**: Integer vectors
- **`UVec2`, `UVec3`, `UVec4`**: Unsigned integer vectors

### Matrices
- **`Mat2`, `Mat3`, `Mat4`**: 2×2, 3×3, and 4×4 matrices
- **`DMat2`, `DMat3`, `DMat4`**: Double-precision matrices

### Quaternions
- **`Quat`**: Unit quaternion for rotations (f32)
- **`DQuat`**: Double-precision quaternion

### Affine Transforms
- **`Affine2`, `Affine3A`**: Efficient 2D and 3D affine transformations

## Geometric Primitives

### AABB (Axis-Aligned Bounding Box)
```rust
use praxis_math::{Aabb, Vec3};

let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
assert!(aabb.contains_point(Vec3::splat(0.5)));
```

### Ray
```rust
use praxis_math::{Ray, Vec3};

let ray = Ray::new(Vec3::ZERO, Vec3::X);
let point = ray.at(5.0);
```

### Frustum & Plane
```rust
use praxis_math::{Frustum, Mat4, Vec3};

let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0/9.0, 0.1, 100.0);
let frustum = Frustum::from_view_projection(proj * view);

assert!(frustum.contains_point(Vec3::ZERO));
```

## Coordinate Space Utilities

Transform between different coordinate spaces:

```rust
use praxis_math::{CoordinateSpace, CoordinateSpaceExt, Mat4, Vec3};

// Local to world transformation
let model_matrix = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
let local_pos = Vec3::new(5.0, 0.0, 0.0);
let world_pos = Mat4::local_to_world(local_pos, &model_matrix);

// Build transforms easily
use praxis_math::coordinate_spaces::TransformBuilder;

let transform = TransformBuilder::new()
    .with_translation(Vec3::new(1.0, 2.0, 3.0))
    .with_uniform_scale(2.0)
    .build();
```

## Common Helpers

### Interpolation
```rust
use praxis_math::{lerp, slerp, Quat};

// Linear interpolation
let value = lerp(0.0, 10.0, 0.5); // 5.0

// Spherical linear interpolation (quaternions)
let start = Quat::IDENTITY;
let end = Quat::from_rotation_y(std::f32::consts::PI);
let halfway = slerp(start, end, 0.5);
```

### Smoothing
```rust
use praxis_math::{smoothstep, exp_decay, spring_damper_vec3, Vec3};

// Smoothstep (ease in/out)
let smooth = smoothstep(0.0, 1.0, 0.5);

// Exponential decay (frame-rate independent)
let damped = exp_decay(current, target, 5.0, delta_time);

// Spring damper (physics-based)
let mut velocity = Vec3::ZERO;
let position = spring_damper_vec3(
    current_pos,
    target_pos,
    &mut velocity,
    5.0,  // omega (stiffness)
    1.0,  // zeta (damping)
    0.016 // dt
);
```

### Angle Utilities
```rust
use praxis_math::{deg_to_rad, wrap_angle, lerp_angle};

let radians = deg_to_rad(90.0);
let wrapped = wrap_angle(std::f32::consts::PI * 3.0); // Wraps to [-PI, PI]
let interpolated = lerp_angle(0.0, std::f32::consts::PI, 0.5);
```

### Utility Functions
```rust
use praxis_math::{clamp, remap, snap, move_towards};

let clamped = clamp(15.0, 0.0, 10.0); // 10.0
let remapped = remap(5.0, 0.0, 10.0, 0.0, 100.0); // 50.0
let snapped = snap(7.3, 5.0); // 5.0
let moved = move_towards(0.0, 10.0, 3.0); // 3.0
```

## Features

- **SIMD optimization** for high performance
- **Serialization** via `serde`
- **Comprehensive 3D math** operations
- **Column-major matrices** (Vulkan-compatible)
- **Coordinate space conversions**
- **Geometric primitives** for collision detection
- **Interpolation helpers** (lerp, slerp, smoothstep, spring damping)

## Coordinate System

Praxis uses a **right-handed, Y-up coordinate system**:
- **+X**: Right
- **+Y**: Up (vertical axis)
- **-Z**: Forward (into the screen in view space)

This is consistent with OpenGL/Vulkan conventions and common 3D modeling tools.

## Dependencies

- `glam`: SIMD-optimized 3D math library
- `serde`: Serialization support

## Usage

```toml
praxis_math = { path = "../praxis_math", version = "0.1.0" }
```

## Examples

### Basic Vector Operations
```rust
use praxis_math::Vec3;

let a = Vec3::new(1.0, 2.0, 3.0);
let b = Vec3::new(4.0, 5.0, 6.0);

let sum = a + b;
let dot = a.dot(b);
let cross = a.cross(b);
let normalized = a.normalize();
```

### Transform Hierarchy
```rust
use praxis_math::{Mat4, Vec3};

// Parent transform (world space)
let parent = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));

// Child transform (local space)
let child_local = Mat4::from_translation(Vec3::new(0.0, 5.0, 0.0));

// Child in world space
let child_world = parent * child_local;
```

### Camera Matrices
```rust
use praxis_math::{Mat4, Vec3};

// View matrix
let view = Mat4::look_at_rh(
    Vec3::new(0.0, 10.0, 10.0), // Eye position
    Vec3::ZERO,                  // Look at target
    Vec3::Y                      // Up direction
);

// Perspective projection
let projection = Mat4::perspective_rh(
    45.0_f32.to_radians(), // FOV
    16.0 / 9.0,            // Aspect ratio
    0.1,                    // Near plane
    100.0                   // Far plane
);
```

### Ray Casting
```rust
use praxis_math::{Ray, Aabb, Vec3};

let ray = Ray::new(Vec3::ZERO, Vec3::X);
let aabb = Aabb::from_min_max(Vec3::new(5.0, -1.0, -1.0), Vec3::new(6.0, 1.0, 1.0));

if let Some(distance) = ray.intersection_distance_aabb(&aabb, 100.0) {
    let hit_point = ray.at(distance);
    println!("Hit at: {:?}", hit_point);
}
```
