# Praxis Math

Mathematics library for the Praxis game engine, providing vectors, matrices, quaternions, and geometric primitives.

## Features

- **Vector Math**: Vec2, Vec3, Vec4 with standard operations
- **Matrix Math**: Mat3, Mat4 for transformations
- **Quaternions**: Quat for rotations
- **Geometric Primitives**: AABB, spheres, planes, rays
- **Transformations**: Translation, rotation, scaling, perspective projection
- **Spatial Queries**: Distance, intersection, containment tests
- **Serialization**: Built-in serde support for all types

## Architecture

Built on top of `glam` 0.30, providing:
- SIMD acceleration for performance
- Consistent API across all types
- Memory-efficient representations
- Column-major matrices (matching Vulkan conventions)

## Usage

### Vectors

```rust
use praxis_math::{Vec2, Vec3, Vec4};

let v1 = Vec3::new(1.0, 2.0, 3.0);
let v2 = Vec3::new(4.0, 5.0, 6.0);

let sum = v1 + v2;
let dot = v1.dot(v2);
let cross = v1.cross(v2);
let length = v1.length();
let normalized = v1.normalize();
```

### Matrices

```rust
use praxis_math::{Mat4, Vec3};

// Translation
let translation = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));

// Rotation
let rotation = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);

// Scale
let scale = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));

// Combined transform
let transform = translation * rotation * scale;

// Projection
let proj = Mat4::perspective_rh(
    70.0_f32.to_radians(),
    16.0 / 9.0,
    0.1,
    1000.0
);
```

### Quaternions

```rust
use praxis_math::{Quat, Vec3};

// Create from axis-angle
let axis = Vec3::Y;
let angle = std::f32::consts::FRAC_PI_2;
let quat = Quat::from_axis_angle(axis, angle);

// Create from euler angles
let quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, angle, 0.0);

// Interpolation
let q1 = Quat::IDENTITY;
let q2 = Quat::from_rotation_y(std::f32::consts::PI);
let interpolated = q1.slerp(q2, 0.5);
```

### Geometric Primitives

```rust
use praxis_math::{Aabb, Vec3};

// Axis-aligned bounding box
let min = Vec3::new(-1.0, -1.0, -1.0);
let max = Vec3::new(1.0, 1.0, 1.0);
let aabb = Aabb::from_min_max(min, max);

// Test containment
let point = Vec3::new(0.5, 0.5, 0.5);
if aabb.contains_point(point) {
    println!("Point is inside AABB");
}

// Test intersection
let other = Aabb::from_min_max(Vec3::ZERO, Vec3::splat(2.0));
if aabb.intersects(&other) {
    println!("AABBs intersect");
}
```

## Common Constants

```rust
use praxis_math::{Vec3, Mat4, Quat};

// Vector constants
Vec3::ZERO;      // (0, 0, 0)
Vec3::ONE;       // (1, 1, 1)
Vec3::X;         // (1, 0, 0)
Vec3::Y;         // (0, 1, 0)
Vec3::Z;         // (0, 0, 1)

// Matrix constants
Mat4::IDENTITY;  // Identity matrix

// Quaternion constants
Quat::IDENTITY;  // No rotation
```

## Coordinate System

Praxis uses a **right-handed coordinate system**:
- X axis: Right
- Y axis: Up
- Z axis: Forward (into the screen)

This matches Vulkan's coordinate conventions.

## Performance

All types use SIMD acceleration when available:
- SSE2 on x86/x86_64
- NEON on ARM
- Automatic fallback to scalar math

Operations are optimized for:
- Cache efficiency
- Minimal allocations
- Inlining of hot paths

## Dependencies

- `glam` 0.30.4: Core math library with SIMD support
- `praxis_utils`: Error handling and logging

## Examples

See transform and math usage in:

```bash
cargo run --example transform_propagation_demo
cargo run --example comprehensive_scene_demo
cargo run --example animation_demo
```

## See Also

- [Transform System](../praxis_ecs/README.md#transform-propagation-system)
- [glam Documentation](https://docs.rs/glam)
- [Math Concepts](../../docs/concepts/coordinate-systems.md)
